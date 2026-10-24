#!/usr/bin/env python3
"""Finite real CLI/daemon evidence. Guards prohibit build/flow retries per label."""
import ctypes
import difflib
import hashlib
import json
import os
from pathlib import Path
import shutil
import socket
import subprocess
import sys
import threading
import time
import traceback

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
PRIVATE = REPO / 'target/sb'
RAW = HERE / 'raw'
TOOLCHAIN = Path('/Users/serge/.rustup/toolchains/1.99.1-aarch64-apple-darwin')
ROOT = 'd2ab17d4e053437d9f7568b0f6dbf96727bfd6c9'
SCOPE = ['crates/daemon', 'crates/net', 'crates/session', 'crates/ipc', 'crates/config',
         'crates/platform', 'crates/platform-macos', 'crates/protocol', 'vendor/mdns-sd',
         'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml']


def sha(data):
    return hashlib.sha256(data).hexdigest()


def save(path, data):
    with path.open('x') as file:
        json.dump(data, file, indent=2, sort_keys=True)
        file.write('\n')


def cmd(argv, cwd=REPO):
    run = subprocess.run(list(map(str, argv)), cwd=cwd, capture_output=True, text=True, timeout=20)
    return dict(argv=list(map(str, argv)), code=run.returncode, stdout=run.stdout, stderr=run.stderr)


def identity(pid):
    return dict(monotonic_ns=time.clock_gettime_ns(time.CLOCK_MONOTONIC), pid=pid,
                nice=os.getpriority(os.PRIO_PROCESS, pid),
                ps=cmd(['/bin/ps', '-p', str(pid), '-o', 'pid,ppid,pgid,uid,ni,lstart,command']))


def sources():
    paths = subprocess.check_output(['git', 'ls-files', '-z', '--', *SCOPE], cwd=REPO).split(b'\0')
    result = {}
    for value in paths:
        if not value:
            continue
        name = value.decode()
        data = (REPO / name).read_bytes()
        original = subprocess.check_output(['git', 'show', f'{ROOT}:{name}'], cwd=REPO)
        result[name] = dict(bytes=len(data), sha256=sha(data), root_sha256=sha(original),
                            matches_root=data == original)
    return result


def product_env(fixture):
    env = {k: v for k, v in os.environ.items() if not k.startswith('OPENERGY_')}
    env.update(OPENERGY_CONFIG_DIR=str(fixture), OPENERGY_CONFIG=str(fixture / 'config.toml'),
               OPENERGY_TLS_DIR=str(fixture / 'tls'), OPENERGY_SOCKET=str(fixture / 'i.sock'),
               OPENERGY_DISABLE_DISCOVERY='1', OPENERGY_DENY_PORT_24800='1',
               OPENERGY_LOG='info', HOSTNAME='stop-b205-private')
    return env


def owned(argv, barrier, **kwargs):
    return subprocess.Popen([sys.executable, str(HERE / 'exec_owned.py'), str(barrier),
                             *map(str, argv)], start_new_session=True, **kwargs)


def prepare():
    RAW.mkdir()
    PRIVATE.mkdir(parents=True)
    os.chmod(PRIVATE, 0o700)
    baseline = sources()
    assert all(x['matches_root'] for x in baseline.values())
    save(RAW / 'root-sources.json', baseline)
    save(RAW / 'source-comparisons.json', dict(root=cmd(['git', 'rev-parse', 'HEAD']),
        through_1fd5144=cmd(['git', 'diff', '1fd5144', ROOT, '--', *SCOPE]),
        diagnosis=cmd(['git', 'show', '--format=fuller', '--stat',
                       'a64b33dd53a9ba11f2a887086b2615ef949c80fc']),
        diagnosis_readme_sha256=sha(Path('/Users/serge/.t3/worktrees/openergy/t3-d1ff2556/docs/verification/daemon-stop-latency-source-2026-10-23-t3-d1ff2556/README.md').read_bytes())))
    cargo_home = PRIVATE / 'cargo-home'
    cargo_home.mkdir()
    registry = Path('/Users/serge/.cargo/registry')
    shutil.copytree(registry, cargo_home / 'registry', symlinks=False)
    # A byte-copy only. No old cargo config, credentials, target or Swift state.
    copied = {}
    for path in registry.rglob('*'):
        if path.is_file():
            other = cargo_home / 'registry' / path.relative_to(registry)
            assert not other.is_symlink()
            digest = sha(path.read_bytes())
            assert sha(other.read_bytes()) == digest
            copied[str(path.relative_to(registry))] = digest
    save(RAW / 'copied-registry.json', copied)
    save(RAW / 'preparation.json', dict(private=str(PRIVATE), cargo_home=str(cargo_home),
         rustc=cmd([TOOLCHAIN / 'bin/rustc', '--version', '--verbose']),
         cargo=cmd([TOOLCHAIN / 'bin/cargo', '--version']),
         os=cmd(['/usr/bin/sw_vers']), machine=cmd(['/usr/bin/uname', '-sm']),
         controller=identity(os.getpid()), swift_cache_access=False))
    main = REPO / 'crates/daemon/src/main.rs'
    original = main.read_text()
    (PRIVATE / 'main-original.rs').write_text(original)
    # Private CLI-only probe. Never substitutes IPC callbacks or changes runtime.
    helper = '''fn stop_evidence_trace(event: &str, value: &serde_json::Value) {
    if std::env::var_os("OPENERGY_STOP_EVIDENCE").is_none() {
        return;
    }
    let mut now: libc::timespec = unsafe { std::mem::zeroed() };
    assert_eq!(unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut now) }, 0);
    let monotonic_ns = now.tv_sec as u64 * 1_000_000_000 + now.tv_nsec as u64;
    eprintln!("STOP_EVIDENCE {}", serde_json::json!({"event":event,"monotonic_ns":monotonic_ns,
        "pid":std::process::id(),"nice":unsafe { libc::getpriority(libc::PRIO_PROCESS, 0) },"value":value}));
}

'''
    marker = 'fn ipc_request(socket: &Path, method: &str, params: serde_json::Value) -> Result<Response, String> {'
    traced = original.replace(marker, helper + marker)
    traced = traced.replace('    write_frame(&mut stream, &v).map_err(|e| e.to_string())?;',
        '    stop_evidence_trace("request_start", &v);\n    write_frame(&mut stream, &v).map_err(|e| e.to_string())?;')
    traced = traced.replace('    let resp_v = read_frame(&mut stream).map_err(|e| e.to_string())?;',
        '    let resp_v = read_frame(&mut stream).map_err(|e| e.to_string())?;\n    stop_evidence_trace("reply", &resp_v);')
    assert traced != original
    (HERE / 'cli-trace-overlay.patch').write_text(''.join(difflib.unified_diff(
        original.splitlines(True), traced.splitlines(True),
        fromfile='a/crates/daemon/src/main.rs', tofile='b/crates/daemon/src/main.rs')))
    main.write_text(traced)


def build(label):
    dest = RAW / f'{label}-build'
    dest.mkdir()
    compiler = dest / 'compiler-exec'
    compiler.mkdir()
    env = dict(os.environ, CARGO_HOME=str(PRIVATE / 'cargo-home'),
               CARGO_TARGET_DIR=str(PRIVATE / 'cargo-target'), CARGO_BUILD_JOBS='2',
               CARGO_NET_OFFLINE='true', RUSTUP_TOOLCHAIN='1.99.1',
               MACOSX_DEPLOYMENT_TARGET='15.0',
               RUSTC=str(TOOLCHAIN / 'bin/rustc'), RUSTDOC=str(TOOLCHAIN / 'bin/rustdoc'),
               RUSTC_WRAPPER=str(HERE / 'rustc_owned.py'), STOP_COMPILER_RECEIPTS=str(compiler),
               CARGO_INCREMENTAL='0')
    argv = [TOOLCHAIN / 'bin/cargo', 'build', '--release', '--locked', '--offline',
            '--jobs', '2', '--target', 'aarch64-apple-darwin', '-p', 'openergy-daemon', '--bin', 'openergy']
    save(dest / 'source-bindings.json', sources())
    samples = []
    started = time.clock_gettime_ns(time.CLOCK_MONOTONIC)
    with (dest / 'stdout.txt').open('x') as out, (dest / 'stderr.txt').open('x') as err:
        process = owned(argv, dest / 'cargo-exec.json', cwd=REPO, env=env, stdout=out, stderr=err)
        while process.poll() is None:
            rows = cmd(['/bin/ps', '-axo', 'pid=,ppid=,pgid=,uid=,ni=,lstart=,command='])['stdout']
            owned_rows = [row for row in rows.splitlines()
                          if len(row.split()) >= 6 and row.split()[2] == str(process.pid)]
            samples.append(dict(monotonic_ns=time.clock_gettime_ns(time.CLOCK_MONOTONIC), rows=owned_rows))
            if time.clock_gettime_ns(time.CLOCK_MONOTONIC) - started > 600_000_000_000:
                os.killpg(process.pid, 15)
                raise TimeoutError('owned build exceeded 600 seconds')
            time.sleep(.05)
    save(dest / 'samples.json', samples)
    save(dest / 'result.json', dict(argv=list(map(str, argv)), code=process.returncode,
        start_ns=started, end_ns=time.clock_gettime_ns(time.CLOCK_MONOTONIC),
        env={k: env[k] for k in ['CARGO_HOME', 'CARGO_TARGET_DIR', 'CARGO_BUILD_JOBS',
             'CARGO_NET_OFFLINE', 'RUSTUP_TOOLCHAIN', 'MACOSX_DEPLOYMENT_TARGET', 'RUSTC',
             'RUSTDOC', 'RUSTC_WRAPPER', 'CARGO_INCREMENTAL']},
        source_diff=cmd(['git', 'diff', '--', 'crates/daemon/src/main.rs', 'crates/daemon/src/runtime.rs'])))
    assert process.returncode == 0, f'{label}: build failed; see raw stderr'
    actual = [row for sample in samples for row in sample['rows']
              if str(TOOLCHAIN / 'bin/rustc') in row and 'rustc_owned.py' not in row]
    assert actual, 'no actual rustc sample captured'
    assert all(row.split()[4] == '15' for row in actual), actual
    binary = PRIVATE / f'openergy-{label}'
    shutil.copy2(PRIVATE / 'cargo-target/aarch64-apple-darwin/release/openergy', binary)
    save(dest / 'binary.json', dict(path=str(binary), sha256=sha(binary.read_bytes()),
         actual_rustc_samples=len(actual), all_actual_rustc_ni15=True))


def port_observation(dest, phase, port):
    netstat = cmd(['/usr/sbin/netstat', '-anv', '-p', 'tcp'])
    netstat['stdout'] = '\n'.join(row for row in netstat['stdout'].splitlines()
                                 if f'.{port} ' in row or f':{port} ' in row)
    record = dict(monotonic_ns=time.clock_gettime_ns(time.CLOCK_MONOTONIC), address='127.0.0.1', port=port,
                  netstat=netstat, lsof=cmd(['/usr/sbin/lsof', '-nP', f'-iTCP:{port}']))
    probe = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    try:
        probe.bind(('127.0.0.1', port))
        record['first_exclusive_bind'] = 'passed'
    except OSError:
        record['first_exclusive_bind'] = 'unverified'
        record['first_error'] = traceback.format_exc()
    finally:
        probe.close()
    save(dest / f'port-{phase}.json', record)


def flow(label):
    dest = RAW / f'{label}-flow'
    dest.mkdir()
    fixture = PRIVATE / ('b' if label == 'baseline' else 'c')
    fixture.mkdir(mode=0o700)
    (fixture / 'tls').mkdir(mode=0o700)
    port = 65181 if label == 'baseline' else 65182
    config = f'''version = 1
role = "off"
name = "stop-b205-private"
[server]
listen = "127.0.0.1:{port}"
port = {port}
[client]
remote = "127.0.0.1:{port}"
autoconnect = false
[tls]
enabled = true
require_client_trust = true
'''
    (fixture / 'config.toml').write_text(config)
    shutil.copy2(fixture / 'config.toml', dest / 'config.toml')
    binary = PRIVATE / f'openergy-{label}'
    env = product_env(fixture)
    daemon = None
    signals = []
    calls = []

    def cli(name, args, trace=False):
        child_env = dict(env)
        if trace:
            child_env['OPENERGY_STOP_EVIDENCE'] = '1'
        start = time.clock_gettime_ns(time.CLOCK_MONOTONIC)
        child = owned([binary, *args], dest / f'{name}-exec.json', cwd=fixture,
                      env=child_env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        stdout, stderr = child.communicate(timeout=8)
        end = time.clock_gettime_ns(time.CLOCK_MONOTONIC)
        receipt = dict(name=name, argv=[str(binary), *map(str, args)], pid=child.pid,
                       start_ns=start, end_ns=end, code=child.returncode, stdout=stdout, stderr=stderr)
        calls.append(receipt)
        save(dest / f'{name}.json', receipt)
        return receipt

    exit_record = {}
    exited = threading.Event()

    def reap():
        # Independent blocking waitpid, not a daemon log or CLI reply inference.
        code = daemon.wait()
        exit_record.update(code=code, observed_exit_ns=time.clock_gettime_ns(time.CLOCK_MONOTONIC))
        exited.set()

    try:
        port_observation(dest, 'pre', port)
        version = cli('version', ['version'])
        assert version['code'] == 0 and version['stdout'] == 'openergy 0.1.0\n'
        fingerprint = cli('fingerprint', ['fingerprint', '--dir', str(fixture)])
        assert fingerprint['code'] == 0
        assert (fixture / 'tls/cert.pem').is_file() and (fixture / 'tls/key.pem').is_file()
        save(dest / 'prelaunch.json', dict(controller=identity(os.getpid()),
             config_sha256=sha((fixture / 'config.toml').read_bytes()),
             binary_sha256=sha(binary.read_bytes()), socket=str(fixture / 'i.sock'),
             socket_absent=not (fixture / 'i.sock').exists(),
             tls=dict(directory=str(fixture / 'tls'), cert_sha256=sha((fixture / 'tls/cert.pem').read_bytes()),
                      key_mode=oct((fixture / 'tls/key.pem').stat().st_mode & 0o777)),
             environment={k: v for k, v in env.items() if k.startswith('OPENERGY_')},
             trace_overlay_sha256=sha((HERE / 'cli-trace-overlay.patch').read_bytes()),
             workflow_sha256={name: sha((HERE / name).read_bytes())
                              for name in ('collect.py', 'exec_owned.py', 'rustc_owned.py')}))
        argv = [binary, 'daemon', '--platform', 'virtual', '--foreground', '--config',
                fixture / 'config.toml', '--socket', fixture / 'i.sock', '--parent-pid', str(os.getpid())]
        with (dest / 'daemon-stdout.txt').open('x') as out, (dest / 'daemon-stderr.txt').open('x') as err:
            daemon = owned(argv, dest / 'daemon-exec.json', cwd=fixture, env=env, stdout=out, stderr=err)
        waiter = threading.Thread(target=reap, daemon=True)
        waiter.start()
        ready_deadline = time.monotonic() + 5
        # Filesystem readiness only; never retry an IPC request or product flow.
        while not (fixture / 'i.sock').exists():
            assert not exited.is_set(), exit_record
            assert time.monotonic() < ready_deadline, 'private IPC path readiness timeout'
            time.sleep(.01)
        status = cli('status', ['ctl', 'status.get', '--socket', str(fixture / 'i.sock'), '--raw'])
        assert status['code'] == 0, status
        parsed = json.loads(status['stdout'])
        save(dest / 'daemon-identity.json', identity(daemon.pid))
        assert os.getpriority(os.PRIO_PROCESS, daemon.pid) == 15
        assert parsed['daemon']['pid'] == daemon.pid and parsed['role'] == 'off' and parsed['daemon']['platform'] == 'virtual', parsed
        assert parsed['tls']['enabled'] is True, parsed
        config_reply = cli('config', ['ctl', 'config.get', '--socket', str(fixture / 'i.sock'), '--raw'])
        assert config_reply['code'] == 0
        live_config = json.loads(config_reply['stdout'])['config']
        assert live_config['role'] == 'off' and live_config['server']['listen'] == f'127.0.0.1:{port}'
        assert live_config['client']['remote'] == f'127.0.0.1:{port}' and not live_config['client']['autoconnect']
        discovery = cli('discovery', ['ctl', 'discovery.list', '--socket', str(fixture / 'i.sock'), '--raw'])
        assert discovery['code'] == 0 and json.loads(discovery['stdout']) == []
        invalid = cli('invalid', ['ctl', 'stop-b205.invalid', '--socket', str(fixture / 'i.sock'), '--raw'])
        assert invalid['code'] == 1 and json.loads(invalid['stdout'])['error']['code'] == 'unknown_method', invalid
        # Same fixed quiet interval in both cases; no scheduling-alignment retry.
        time.sleep(.2)
        stop_start = time.clock_gettime_ns(time.CLOCK_MONOTONIC)
        deadline_ns = stop_start + 5_000_000_000
        stop = cli('stop', ['ctl', 'daemon.stop', '--socket', str(fixture / 'i.sock'), '--raw'], trace=True)
        events = [json.loads(line.removeprefix('STOP_EVIDENCE ')) for line in stop['stderr'].splitlines()
                  if line.startswith('STOP_EVIDENCE ')]
        assert [x['event'] for x in events] == ['request_start', 'reply'], events
        request, reply = events
        assert request['value']['method'] == 'daemon.stop'
        assert reply['value'] == {'id': request['value']['id'], 'result': {'stopping': True}}
        assert stop['code'] == 0 and stop['stdout'] == '{"stopping":true}\n', stop
        assert all(x['nice'] == 15 and x['pid'] == stop['pid'] for x in events)
        assert stop['start_ns'] <= request['monotonic_ns'] <= reply['monotonic_ns'] <= stop['end_ns']
        normal = exited.wait(max(0, (deadline_ns - time.clock_gettime_ns(time.CLOCK_MONOTONIC)) / 1e9))
        save(dest / 'stop-timing.json', dict(request=request, reply=reply,
             cli_launch_start_ns=stop_start, cli_end_ns=stop['end_ns'],
             normal_exit_deadline_ns=deadline_ns, normal_exit_observed=normal,
             independent_exit=dict(exit_record),
             request_to_reply_ms=(reply['monotonic_ns']-request['monotonic_ns'])/1e6,
             request_to_exit_ms=(exit_record['observed_exit_ns']-request['monotonic_ns'])/1e6 if normal else None,
             reply_to_exit_ms=(exit_record['observed_exit_ns']-reply['monotonic_ns'])/1e6 if normal else None))
        assert normal, 'TIMEOUT: no exact owned normal exit observed within 5 seconds; no retry'
        assert exit_record['code'] == 0, exit_record
    except BaseException:
        (dest / 'first-error.txt').write_text(traceback.format_exc())
        raise
    finally:
        # No cleanup signal before the normal-exit deadline after stop dispatch.
        if daemon and not exited.is_set():
            if 'deadline_ns' in locals():
                exited.wait(max(0, (deadline_ns - time.clock_gettime_ns(time.CLOCK_MONOTONIC)) / 1e9))
            if not exited.is_set():
                signals.append(dict(signal='SIGTERM', pgid=daemon.pid, monotonic_ns=time.clock_gettime_ns(time.CLOCK_MONOTONIC)))
                os.killpg(daemon.pid, 15)
                if not exited.wait(5):
                    signals.append(dict(signal='SIGKILL', pgid=daemon.pid, monotonic_ns=time.clock_gettime_ns(time.CLOCK_MONOTONIC)))
                    os.killpg(daemon.pid, 9)
                    exited.wait(5)
        cleanup = dict(signals=signals, exit=dict(exit_record), calls=len(calls))
        if daemon:
            cleanup['pid_absent'] = cmd(['/bin/ps', '-p', str(daemon.pid), '-o', 'pid='])
            try:
                os.killpg(daemon.pid, 0)
                cleanup['group_absent'] = False
            except ProcessLookupError:
                cleanup['group_absent'] = True
        cleanup['ipc_socket_absent'] = not (fixture / 'i.sock').exists()
        if (fixture / 'openergy.log').exists():
            shutil.copy2(fixture / 'openergy.log', dest / 'daemon-log.txt')
        cleanup['tls_files_removed'] = [p.name for p in (fixture / 'tls').iterdir()]
        shutil.rmtree(fixture / 'tls')
        cleanup['tls_directory_absent'] = not (fixture / 'tls').exists()
        port_observation(dest, 'post', port)
        save(dest / 'cleanup.json', cleanup)


if __name__ == '__main__':
    os.umask(0o077)
    os.setpriority(os.PRIO_PROCESS, 0, 15)
    mode = sys.argv[1]
    try:
        if mode == 'prepare':
            prepare()
        elif mode == 'build':
            build(sys.argv[2])
        elif mode == 'flow':
            assert sys.argv[2] in ('baseline', 'candidate')
            flow(sys.argv[2])
        else:
            raise ValueError(mode)
    except BaseException:
        print(traceback.format_exc(), file=sys.stderr)
        sys.exit(1)
