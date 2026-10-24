"""One unchanged packaged AppEndToEnd flow, real peer/daemon, bounded owned cleanup."""
from common import *
import ctypes
import re
import shutil
import signal
import socket
import stat
import threading
import traceback

PYTHON = '/opt/homebrew/bin/python3'
LIBPROC = ctypes.CDLL('/usr/lib/libproc.dylib')
LIBPROC.proc_pidpath.argtypes = [ctypes.c_int, ctypes.c_void_p, ctypes.c_uint32]
LIBPROC.proc_pidpath.restype = ctypes.c_int
prepared = json.loads((RAW / 'prepared.json').read_text())
private = Path(prepared['private_root'])
bundle = Path(prepared['bundle'])
app_binary = bundle / 'Contents/MacOS/Openergy'
daemon_binary = bundle / 'Contents/Helpers/openergy'
runtime = RAW / 'runtime'
processes, identities, samples, reservations, signals = {}, {}, [], {}, []
sample_rounds = 0
flow_start = None


def path_for(pid):
    value = ctypes.create_string_buffer(4096)
    result = LIBPROC.proc_pidpath(pid, value, len(value))
    return value.value.decode() if result > 0 else None


def live_rows():
    output = subprocess.check_output(['/bin/ps', '-axo', 'pid=,ppid=,pgid=,uid=,ni=,stat='], text=True, timeout=2)
    return [dict(zip(['pid', 'ppid', 'pgid', 'uid', 'nice', 'state'],
                     [*map(int, parts[:5]), parts[5]]))
            for line in output.splitlines() if len(parts := line.split()) == 6]


def sample():
    global sample_rounds
    sample_rounds += 1
    assert sample_rounds <= 6000
    rows = live_rows()
    owned = {p.pid for p in processes.values() if p.poll() is None}
    groups = {p.pid for p in processes.values()}
    owned |= {i for i in identities}
    for _ in range(8):
        added = {row['pid'] for row in rows if row['ppid'] in owned or row['pgid'] in groups}
        if added <= owned: break
        owned |= added
    for row in rows:
        if row['pid'] not in owned: continue
        stamp = now()
        executable = path_for(row['pid'])
        entry = dict(**row, executable=executable, monotonic_ns=stamp)
        samples.append(entry)
        assert len(samples) <= 24000
        if executable in [str(app_binary), str(daemon_binary)] and row['pid'] not in identities:
            detail = cmd(['/bin/ps', '-p', str(row['pid']), '-o', 'pid,ppid,pgid,uid,ni,lstart,command'])
            assert detail['code'] == 0
            record = dict(**entry, ps=detail, binary_sha256=sha(Path(executable)))
            if executable == str(daemon_binary):
                is_peer = 'peer' in processes and row['pid'] == processes['peer'].pid
                cli_label = next((label for label, p in processes.items()
                                  if p.pid == row['pid'] and label not in ['peer', 'app']), None)
                if cli_label:
                    assert row['ppid'] == os.getpid() and row['pgid'] == row['pid']
                    record['cli_label'] = cli_label
                elif is_peer:
                    assert row['ppid'] == os.getpid() and row['pgid'] == row['pid']
                    expected_config = private / 'peer/config.toml'
                    expected_socket = private / 'peer/control.sock'
                else:
                    assert row['ppid'] == processes['app'].pid and row['pgid'] == processes['app'].pid
                    expected_config = private / 'app/config/config.toml'
                    expected_socket = private / 'app/config/control.sock'
                    assert '--parent-pid ' + str(processes['app'].pid) in detail['stdout']
                    if not any(i.get('role') == 'initial-supervised-daemon' for i in identities.values()):
                        record['role'] = 'initial-supervised-daemon'
                        config_bytes = expected_config.read_bytes()
                        (runtime / 'app-config-first-daemon.toml').write_bytes(config_bytes)
                        record['config_at_first_observation_sha256'] = sha(expected_config)
                    else:
                        record['role'] = 'recovered-supervised-daemon'
                if not cli_label:
                    for token in [str(expected_config), str(expected_socket), '--platform virtual']:
                        assert token in detail['stdout']
                    record['ownership_verified_before_routine_signals'] = True
                record['environment_inheritance'] = 'DaemonSupervisor passes the complete app environment; source-bound, no replacement launcher'
            identities[row['pid']] = record
            save(runtime / f'identity-{row["pid"]}.json', record)
        if executable in [str(app_binary), str(daemon_binary)]:
            assert row['nice'] == 15 and row['uid'] == os.getuid(), entry


def tick(deadline):
    assert time.monotonic() < deadline, 'watchdog deadline reached'
    sample()
    for path in [runtime / 'app.log', runtime / 'peer.log', private / 'app/config/daemon-stderr.log', private / 'app/config/openergy.log']:
        if path.exists(): assert path.stat().st_size <= 8388608, str(path)
    time.sleep(0.02)


def ports_state():
    result = cmd(['/usr/sbin/netstat', '-an', '-p', 'tcp'])
    scoped = [line for line in result['stdout'].splitlines()
              if any(re.search(r'\.' + str(port) + r'\s', line) for port in PORTS)]
    return dict(argv=result['argv'], code=result['code'], scoped_rows=scoped, stderr=result['stderr'],
                monotonic_ns=now(), filter_ports=list(PORTS))


def exclusive_port(port, label, retain):
    state = ports_state()
    handle = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    try:
        handle.bind(('127.0.0.1', port))
    except OSError as error:
        save(runtime / f'port-{port}-{label}.json', dict(state_before_first_bind=state,
             success=False, error=repr(error), no_reuse=True, no_retry=True))
        handle.close()
        raise
    save(runtime / f'port-{port}-{label}.json', dict(state_before_first_bind=state,
         success=True, bound=handle.getsockname(), no_reuse=True, no_retry=True, held_until_launch=retain))
    if retain: reservations[port] = handle
    else: handle.close()


def launch(label, argv, env):
    save(runtime / (label + '-command.json'), dict(argv=list(map(str, argv)), environment_allowlist=env,
         exec_helper_sha256=sha(HERE / 'exec_owned.py'), start_monotonic_ns=now(),
         priority='absolute setpriority 15 before exec; no relative nice wrapper'))
    log = (runtime / (label + '.log')).open('xb')
    try:
        process = subprocess.Popen([PYTHON, str(HERE / 'exec_owned.py'),
            str(runtime / (label + '-exec.json')), *map(str, argv)],
            env=env, cwd=private, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
    finally:
        log.close()
    processes[label] = process
    return process


def cli(label, argv, env, deadline):
    process = launch(label, argv, env)
    limit = min(deadline, time.monotonic() + 5)
    while process.poll() is None: tick(limit)
    result = dict(code=process.returncode, end_monotonic_ns=now(),
                  stdout=(runtime / (label + '.log')).read_text())
    save(runtime / (label + '-result.json'), result)
    assert result['code'] == 0, result
    return result['stdout']


def verified_signal(pid, sig):
    original = identities[pid]
    current = cmd(['/bin/ps', '-p', str(pid), '-o', 'pid,ppid,pgid,uid,ni,lstart,command'])
    assert current['code'] == 0 and path_for(pid) == original['executable']
    # lstart, UID, group and command must still match the first post-exec observation.
    assert current['stdout'] == original['ps']['stdout']
    signals.append(dict(pid=pid, signal=sig, verified_identity=current, monotonic_ns=now()))
    os.kill(pid, sig)


def watchdog_fired():
    errors = []
    for pid, original in list(identities.items()):
        if path_for(pid) != original['executable']: continue
        try:
            row = next(r for r in live_rows() if r['pid'] == pid)
            if 'T' in row['state']: verified_signal(pid, signal.SIGCONT)
            verified_signal(pid, signal.SIGTERM)
        except BaseException:
            errors.append(traceback.format_exc())
    save(runtime / 'watchdog-fired.json', dict(monotonic_ns=now(), errors=errors))


def cleanup(peer_env):
    deadline = time.monotonic() + 60
    start = now()
    failures = []
    # Normal peer cleanup uses its public stop command once, with its private socket.
    peer = processes.get('peer')
    if peer is not None and peer.poll() is None:
        try:
            cli('peer-stop', [daemon_binary, 'ctl', 'daemon.stop', '--socket', private / 'peer/control.sock', '--raw'], peer_env, deadline)
        except BaseException:
            failures.append(traceback.format_exc())
    graceful = min(deadline, time.monotonic() + 10)
    while time.monotonic() < graceful:
        sample()
        active = [row for row in live_rows() if row['pid'] in identities and not row['state'].startswith('Z')]
        if not active and all(p.poll() is not None for p in processes.values()): break
        time.sleep(0.02)
    remaining = [pid for pid in identities if path_for(pid) == identities[pid]['executable']]
    if remaining:
        failures.append('Owned products remained after the 10-second normal cleanup allowance')
        for pid in remaining:
            try:
                row = next(r for r in live_rows() if r['pid'] == pid)
                if 'T' in row['state']: verified_signal(pid, signal.SIGCONT)
                verified_signal(pid, signal.SIGTERM)
            except ProcessLookupError: pass
        limit = min(deadline, time.monotonic() + 5)
        while time.monotonic() < limit and any(path_for(p) for p in remaining): time.sleep(0.02)
        for pid in remaining:
            if path_for(pid) == identities[pid]['executable']: verified_signal(pid, signal.SIGKILL)
    for process in processes.values():
        if process.poll() is None:
            # Wrappers may fail before product exec; require exact parent/group and receipt.
            receipt = runtime / (next(k for k, v in processes.items() if v is process) + '-exec.json')
            assert receipt.exists() and json.loads(receipt.read_text())['pid'] == process.pid
            assert os.getpgid(process.pid) == process.pid
            process.terminate()
        process.wait(timeout=max(0.1, min(5, deadline - time.monotonic())))
    for handle in reservations.values(): handle.close()
    reservations.clear()
    rows = live_rows()
    owned_groups = sorted({p.pid for p in processes.values()})
    remaining_rows = [r for r in rows if r['pid'] in identities or r['pgid'] in owned_groups]
    sockets = []
    for path in [private / 'peer/control.sock', private / 'app/config/control.sock']:
        exists = path.exists()
        sockets.append(dict(path=str(path), exists=exists,
             is_socket=exists and stat.S_ISSOCK(path.lstat().st_mode)))
    save(runtime / 'cleanup.json', dict(start_monotonic_ns=start, end_monotonic_ns=now(),
         elapsed_seconds=(now()-start)/1e9, failures=failures, signals=signals,
         collected_exits={k: p.returncode for k, p in processes.items()}, owned_pids=sorted(identities),
         owned_groups=owned_groups, remaining_owned_rows=remaining_rows, sockets=sockets))
    assert not remaining_rows and not any(i['exists'] for i in sockets), 'owned PID/group/socket remains'
    for port in PORTS: exclusive_port(port, 'post', False)
    assert not failures, failures


def main():
    global flow_start
    nice15()
    runtime.mkdir(mode=0o700)  # Exclusive: no second flow, even after setup/runtime failure.
    assert cmd(['git', 'rev-parse', 'HEAD'])['stdout'].strip() == ROOT
    assert tree(bundle) == json.loads((RAW / 'bundle-signed-tree.json').read_text())
    assert sha(app_binary) == prepared['app_sha256'] and sha(daemon_binary) == prepared['daemon_sha256']
    assert not (private / 'app/config').exists()
    save(runtime / 'launch-preflight.json', dict(bundle_tree_unchanged=True,
         app_sha256=sha(app_binary), daemon_sha256=sha(daemon_binary), peer_sha256=sha(daemon_binary),
         config_directory_absent=True, controller_pid=os.getpid(), controller_nice=15,
         flow_count=1, arguments=['--e2e', str(private / 'app'), '--join', '127.0.0.1:65371', '--host-port', '65372', '--require-peer']))
    common = {k: prepared['environment'][k] for k in ['PATH', 'LANG', 'HOME', 'CFFIXED_USER_HOME', 'TMPDIR', 'XDG_CONFIG_HOME', 'XDG_CACHE_HOME', 'MACOSX_DEPLOYMENT_TARGET', 'PYTHONDONTWRITEBYTECODE']}
    common.update(OPENERGY_DISABLE_DISCOVERY='1', OPENERGY_ALLOW_PORT_24800='0',
                  OPENERGY_DAEMON_ARGS='--platform virtual')
    def env_for(directory):
        return dict(common, OPENERGY_CONFIG=str(directory / 'config.toml'),
                    OPENERGY_CONFIG_DIR=str(directory), OPENERGY_TLS_DIR=str(directory / 'tls'),
                    OPENERGY_SOCKET=str(directory / 'control.sock'))
    peer_env = env_for(private / 'peer')
    app_env = dict(env_for(private / 'app/config'), OPENERGY_E2E_NO_RENDER='1')
    peer_config = private / 'peer/config.toml'
    peer_config.write_text('version=1\nname="e2e-peer"\nrole="server"\n[server]\nlisten="127.0.0.1:65371"\nport=65371\n[tls]\nenabled=false\n[[screen]]\nname="e2e-peer"\nrect={x=0,y=0,w=1920,h=1080}\n[[screen]]\nname="app"\nrect={x=1920,y=0,w=1920,h=1080}\n')
    peer_config.chmod(0o600)
    shutil.copyfile(peer_config, runtime / 'peer-config-before.toml')
    failure = None
    watchdog = None
    try:
        for port in PORTS: exclusive_port(port, 'pre', True)
        flow_start = time.monotonic()
        deadline = flow_start + 120
        watchdog = threading.Timer(120, watchdog_fired)
        watchdog.daemon = True
        watchdog.start()
        reservations.pop(PORTS[0]).close()
        peer = launch('peer', [daemon_binary, 'daemon', '--platform', 'virtual', '--config', peer_config,
                     '--socket', private / 'peer/control.sock', '--foreground'], peer_env)
        ready = min(deadline, time.monotonic() + 15)
        while not (private / 'peer/control.sock').exists() or 'listening on 127.0.0.1:65371' not in (runtime / 'peer.log').read_text():
            assert peer.poll() is None, 'peer exited before readiness'
            tick(ready)
        sample()
        status_text = cli('peer-status', [daemon_binary, 'status', '--socket', private / 'peer/control.sock'], peer_env, deadline)
        status = json.loads(status_text)
        assert status['daemon']['pid'] == peer.pid and status['daemon']['platform'] == 'virtual'
        assert status['state'] == 'listening' and peer.pid in identities
        save(runtime / 'peer-ready.json', dict(status=status, monotonic_ns=now(),
             ownership_identity=identities[peer.pid], before_app_launch=True))
        reservations.pop(PORTS[1]).close()
        app = launch('app', [app_binary, '--e2e', private / 'app', '--join', '127.0.0.1:65371',
                     '--host-port', '65372', '--require-peer'], app_env)
        app_start = time.monotonic()
        while app.poll() is None: tick(deadline)
        sample()
        report_path = private / 'app/report.json'
        report = json.loads(report_path.read_text()) if report_path.exists() else []
        if report_path.exists(): shutil.copyfile(report_path, runtime / 'report.json')
        save(runtime / 'app-result.json', dict(code=app.returncode, elapsed_seconds=time.monotonic()-app_start,
             end_monotonic_ns=now(), checks=len(report), passed=sum(i['passed'] for i in report),
             failed=[i for i in report if not i['passed']], partial=len(report) != 25,
             app_sha256=sha(app_binary), daemon_sha256=sha(daemon_binary), peer_sha256=sha(daemon_binary)))
        assert app.returncode == 0 and len(report) == 25 and all(i['passed'] for i in report)
        app_daemons = [i for i in identities.values() if i['executable'] == str(daemon_binary) and i['ppid'] == app.pid]
        assert len(app_daemons) == 2, 'must observe initial and recovered real supervised daemon'
    except BaseException:
        failure = traceback.format_exc()
        (runtime / 'first-runtime-failure.txt').write_text(failure)
    finally:
        if watchdog is not None: watchdog.cancel()
        try:
            cleanup(peer_env)
        except BaseException:
            (runtime / 'first-cleanup-failure.txt').write_text(traceback.format_exc())
            if failure is None: failure = traceback.format_exc()
        save(runtime / 'priority-samples.json', dict(sample_rounds=sample_rounds, samples=samples))
        for name in ['config.toml', 'daemon-stderr.log', 'openergy.log']:
            source = private / 'app/config' / name
            if source.exists(): shutil.copyfile(source, runtime / ('app-' + name))
        save(runtime / 'bundle-after.json', dict(unchanged=tree(bundle) == json.loads((RAW / 'bundle-signed-tree.json').read_text()),
             app_sha256=sha(app_binary), daemon_sha256=sha(daemon_binary)))
        assert sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file()) <= 33554432
    if failure: raise RuntimeError(failure)


if __name__ == '__main__':
    try:
        main()
    except BaseException:
        failure_path = RAW / 'first-controller-failure.txt'
        if not failure_path.exists(): failure_path.write_text(traceback.format_exc())
        raise
