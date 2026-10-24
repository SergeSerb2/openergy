#!/usr/bin/env python3
"""One finite real app/virtual-daemon idle baseline; private build and runtime roots."""
import argparse
import ctypes as C
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import signal
import socket
import subprocess
import tempfile
import time
import traceback

REPO = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
HEAD = '5a9783738c5b649844805e1614da7ea37cfbf2bb'
OLD_COMMIT = 'e31213ef8edf4979a2487a692114071b97c4c200'
OLD_DIRECTORY = 'docs/verification/macos-idle-resources-2026-10-23'
DAEMON_SHA = '350db4541b95f67b8a295419d71933eb4832d07de37881c781664d212a29d572'
PEER_NAME = 'idle-peer-c57ae290-r2'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def save(path, data):
    path.write_text(json.dumps(data, indent=2) + '\n')


def command(argv, cwd, env=None, timeout=180):
    start = time.monotonic()
    p = subprocess.run(list(map(str, argv)), cwd=cwd, env=env, capture_output=True,
                       text=True, timeout=timeout)
    return dict(argv=list(map(str, argv)), cwd=str(cwd), exit=p.returncode,
                elapsed_seconds=time.monotonic() - start, stdout=p.stdout, stderr=p.stderr)


def compilers():
    r = command(['/bin/ps', '-axo', 'pid=,ppid=,ni=,time=,rss=,state=,comm='], REPO, timeout=3)
    assert r['exit'] == 0, r
    return [line.strip() for line in r['stdout'].splitlines()
            if re.search(r'/(swift(?:c|-frontend|-build)?|clang(?:\+\+)?|rustc|cargo|ld)$', line)]


def priority(root, phase):
    """Absolute priority on this controller only; every child inherits it, no nice wrappers."""
    record = dict(pid=os.getpid(), parent_pid=os.getppid(), uid=os.getuid(),
                  before=os.getpriority(os.PRIO_PROCESS, 0), requested_absolute_ni=15,
                  wrappers=[])
    save(root / f'priority-{phase}.json', record)
    try:
        os.setpriority(os.PRIO_PROCESS, 0, 15)
        record['after'] = os.getpriority(os.PRIO_PROCESS, 0)
        record['ps'] = command(['/bin/ps', '-p', str(os.getpid()), '-o',
                                'pid=,ppid=,pgid=,uid=,ni=,state=,comm='], root)
        assert record['after'] == 15 and record['ps']['exit'] == 0, record
        assert int(record['ps']['stdout'].split()[4]) == 15, record
    except BaseException:
        record['failure'] = traceback.format_exc()
        raise
    finally:
        save(root / f'priority-{phase}.json', record)


def prepare():
    os.umask(0o077)
    root = Path(tempfile.mkdtemp(prefix='oe-idle-r2-c57ae290-', dir='/tmp')).resolve()
    print(root, flush=True)
    try:
        priority(root, 'prepare')
        head = command(['git', 'rev-parse', 'HEAD'], REPO)['stdout'].strip()
        assert head == HEAD, head
        reference_path = REPO / 'docs/verification/sparkle-integration-2026-10-23/results.json'
        reference = json.loads(reference_path.read_text())
        checked = {name: sha(REPO / name) for name in reference['source_sha256']}
        differences = [name for name, digest in checked.items() if digest != reference['source_sha256'][name]]
        assert differences == ['apps/macos/Sources/Openergy/App/AppEndToEnd.swift'], differences
        # The only changed input is the whole-app verification harness. Every Rust,
        # daemon, dependency, and production app input still matches the reference.
        state = json.loads(Path('/tmp/oe-sparkle-combined-20261023-state.json').read_text())
        bundle = Path(json.loads(Path('/tmp/oe-app-current-flow-20261023-state.json').read_text())['source_app'])
        assert sha(bundle / 'Contents/Helpers/openergy') == DAEMON_SHA
        assert sha(bundle / 'Contents/MacOS/Openergy') == '99f49c30c9e30c82899bd8cd5e2cf4a9e4293a4e8614010edfeb19589b6264ab'
        cache = Path(state['swift_scratch'])
        scratch = root / 'swift'
        scratch.mkdir()
        source = root / 'source'
        shutil.copytree(REPO / 'apps/macos', source / 'apps/macos',
                        ignore=shutil.ignore_patterns('.build', '.swiftpm'))
        cached_hashes = {str(p.relative_to(cache)): sha(p) for part in ('checkouts', 'repositories', 'artifacts')
                         for p in (cache / part).rglob('*') if p.is_file() and not p.is_symlink()}
        for part in ('checkouts', 'repositories', 'artifacts'):
            shutil.copytree(cache / part, scratch / part, symlinks=True)
        (scratch / 'workspace-state.json').write_text(
            (cache / 'workspace-state.json').read_text().replace(str(cache), str(scratch)))
        # A copied symlink must not redirect a build write into a historical cache.
        for path in scratch.rglob('*'):
            if path.is_symlink():
                assert path.resolve().is_relative_to(scratch), str(path)
        patched = command(['/usr/bin/patch', '-p1', '-i', HERE / 'harness-overlay.patch'], source)
        assert patched['exit'] == 0, patched
        copied_hashes = {str(p.relative_to(source)): sha(p) for p in (source / 'apps/macos').rglob('*') if p.is_file()}
        changed = [name for name, digest in copied_hashes.items() if sha(REPO / name) != digest]
        assert changed == ['apps/macos/Sources/Openergy/App/AppEndToEnd.swift'], changed
        daemon = root / 'openergy-daemon'
        shutil.copy2(bundle / 'Contents/Helpers/openergy', daemon)
        env = dict(os.environ, MACOSX_DEPLOYMENT_TARGET='15.0', RUSTUP_TOOLCHAIN='1.99.1',
                   CARGO_BUILD_JOBS='2', CARGO_NET_OFFLINE='true',
                   CLANG_MODULE_CACHE_PATH=str(root / 'clang-cache'),
                   SWIFTPM_MODULECACHE_OVERRIDE=str(root / 'module-cache'))
        argv = ['swift', 'build', '--package-path', source / 'apps/macos',
                '--scratch-path', scratch, '--cache-path', root / 'cache', '--config-path', root / 'config',
                '--security-path', root / 'security', '--manifest-cache', 'local', '-c', 'release',
                '--jobs', '2', '--disable-automatic-resolution', '--skip-update']
        metadata = dict(root=str(root), source_head=head, reference_sha256=sha(reference_path),
                        matched_reference_input_count=len(checked)-len(differences), source_sha256=checked,
                        reference_differences=differences, rebuilt_whole_app_from_current_parent=True,
                        copied_app_source_sha256=copied_hashes, changed_app_inputs=changed,
                        overlay_sha256=sha(HERE / 'harness-overlay.patch'), overlay_apply=patched,
                        cached_swift=str(cache), cached_bundle=str(bundle), daemon_sha256=sha(daemon),
                        original_app_sha256=sha(bundle / 'Contents/MacOS/Openergy'),
                        build_environment={k: env[k] for k in ('MACOSX_DEPLOYMENT_TARGET', 'RUSTUP_TOOLCHAIN',
                            'CARGO_BUILD_JOBS', 'CARGO_NET_OFFLINE', 'CLANG_MODULE_CACHE_PATH', 'SWIFTPM_MODULECACHE_OVERRIDE')},
                        compiler_inventory_before_build=compilers(),
                        os=command(['/usr/bin/sw_vers'], root),
                        machine=command(['/usr/bin/uname', '-sm'], root),
                        swift=command(['swift', '--version'], root),
                        rust=command(['/Users/serge/.cargo/bin/rustc', '+1.99.1', '--version'], root),
                        xcode=command(['/usr/bin/xcodebuild', '-version'], root),
                        sdk=command(['/usr/bin/xcrun', '--show-sdk-version'], root))
        save(root / 'preparation.json', metadata)
        save(root / 'cached-input-hashes.json', cached_hashes)
        build_start = time.monotonic()
        build_samples = []
        with (root / 'build.stdout').open('w') as out, (root / 'build.stderr').open('w') as err:
            process = subprocess.Popen(list(map(str, argv)), cwd=root, env=env,
                                       stdout=out, stderr=err, start_new_session=True)
        try:
            while process.poll() is None:
                build_samples.append(dict(elapsed_seconds=time.monotonic()-build_start,
                                          compiler_inventory=compilers()))
                save(root / 'build-compiler-samples.json', build_samples)
                assert time.monotonic()-build_start < 240, '240s build deadline'
                time.sleep(.5)
        finally:
            if process.poll() is None:
                stop_group(process)
        build = dict(argv=list(map(str, argv)), cwd=str(root), exit=process.returncode,
                     elapsed_seconds=time.monotonic()-build_start,
                     stdout=(root / 'build.stdout').read_text(), stderr=(root / 'build.stderr').read_text(),
                     launcher_pid=process.pid, controller_absolute_ni=15, nice_wrappers=[])
        save(root / 'build.json', build)
        assert build['exit'] == 0, build
        assert all(sha(cache / name) == digest for name, digest in cached_hashes.items()), 'historical cache changed'
        compiled = (scratch / 'release/Openergy').resolve(strict=True)
        app_bundle = root / 'Openergy.app'
        shutil.copytree(bundle, app_bundle, symlinks=True)
        app = app_bundle / 'Contents/MacOS/Openergy'
        shutil.copy2(compiled, app)
        signed = command(['/usr/bin/codesign', '--force', '--sign', '-', app_bundle], root)
        assert signed['exit'] == 0, signed
        save(root / 'ready.json', dict(app=str(app), app_sha256=sha(app), daemon=str(daemon),
              daemon_sha256=sha(daemon), compiled_sha256=sha(compiled), bundle_sign=signed,
              cached_inputs_unchanged_after_build=True, compiler_inventory_after_build=compilers()))
    except BaseException:
        (root / 'preparation-first-failure.txt').write_text(traceback.format_exc())
        raise


class TaskInfo(C.Structure):
    _fields_ = [(name, C.c_uint64) for name in ('virtual_size', 'resident_size', 'total_user',
                 'total_system', 'threads_user', 'threads_system')] + [
                (name, C.c_int32) for name in ('policy', 'faults', 'pageins', 'cow_faults',
                 'messages_sent', 'messages_received', 'syscalls_mach', 'syscalls_unix', 'csw',
                 'threadnum', 'numrunning', 'priority')]


class Usage(C.Structure):
    _fields_ = [('uuid', C.c_uint8 * 16)] + [(name, C.c_uint64) for name in (
        'user_time', 'system_time', 'pkg_idle_wkups', 'interrupt_wkups', 'pageins', 'wired_size',
        'resident_size', 'phys_footprint', 'proc_start_abstime', 'proc_exit_abstime')]


class FDInfo(C.Structure):
    _fields_ = [('fd', C.c_int32), ('kind', C.c_uint32)]


LIB = C.CDLL('/usr/lib/libproc.dylib', use_errno=True)
LIB.proc_pidinfo.argtypes = [C.c_int, C.c_int, C.c_uint64, C.c_void_p, C.c_int]
LIB.proc_pidinfo.restype = C.c_int
LIB.proc_pid_rusage.argtypes = [C.c_int, C.c_int, C.c_void_p]
LIB.proc_pid_rusage.restype = C.c_int
LIB.proc_pidpath.argtypes = [C.c_int, C.c_void_p, C.c_uint32]
LIB.proc_pidpath.restype = C.c_int


def process_sample(pid):
    info, usage, fds, path = TaskInfo(), Usage(), (FDInfo * 4096)(), C.create_string_buffer(4096)
    assert LIB.proc_pidinfo(pid, 4, 0, C.byref(info), C.sizeof(info)) == C.sizeof(info), (pid, C.get_errno())
    assert LIB.proc_pid_rusage(pid, 0, C.byref(usage)) == 0, (pid, C.get_errno())
    count = LIB.proc_pidinfo(pid, 1, 0, fds, C.sizeof(fds))
    assert 0 < count < C.sizeof(fds) and count % C.sizeof(FDInfo) == 0, (pid, count, C.get_errno())
    assert LIB.proc_pidpath(pid, path, len(path)) > 0, (pid, C.get_errno())
    return dict(pid=pid, path=path.value.decode(), task_info={name: getattr(info, name) for name, _ in info._fields_},
                rusage={name: getattr(usage, name) for name, _ in usage._fields_ if name != 'uuid'},
                cpu_cumulative_ns=usage.user_time + usage.system_time,
                rss_bytes=info.resident_size, threads=info.threadnum,
                fds=[dict(fd=fd.fd, kind=fd.kind) for fd in fds[:count // C.sizeof(FDInfo)]])


def identity(pid, cwd):
    path = C.create_string_buffer(4096)
    assert LIB.proc_pidpath(pid, path, len(path)) > 0, (pid, C.get_errno())
    ps = command(['/bin/ps', '-p', str(pid), '-o',
                  'pid=,ppid=,pgid=,uid=,ni=,state=,lstart=,comm='], cwd, timeout=1)
    assert ps['exit'] == 0, ps
    columns = ps['stdout'].split()
    assert len(columns) >= 12, ps
    values = dict(zip(('pid', 'ppid', 'pgid', 'uid', 'ni'), map(int, columns[:5])))
    return dict(**values, path=path.value.decode(), state=columns[5],
                lstart=' '.join(columns[6:11]), ps=ps,
                getpriority=os.getpriority(os.PRIO_PROCESS, pid))


def exec_barrier(root, role, pid, expected_path, parent, groups, deadline, records):
    """Observe only our PID. Launcher transitions are never accepted as the app."""
    start = time.monotonic()
    record = dict(role=role, pid=pid, expected_path=str(expected_path), expected_uid=os.getuid(),
                  expected_parent=parent, allowed_groups=list(groups), max_exec_wait_seconds=3,
                  poll_interval_seconds=.025, observations=[])
    records.append(record)
    try:
        while True:
            observed = identity(pid, root)
            record['observations'].append(dict(elapsed_seconds=time.monotonic()-start, **observed))
            save(root / 'exec-barriers.json', records)
            assert observed['uid'] == os.getuid() and observed['ppid'] == parent, observed
            assert observed['pgid'] in groups, observed
            if observed['path'] == str(expected_path):
                assert observed['ni'] == observed['getpriority'] == 15, observed
                record['accepted'] = observed
                return observed
            assert observed['path'] == '/usr/bin/nice', ('unexpected executable', observed)
            assert time.monotonic() < min(start+3, deadline), 'owned launcher did not exec within 3s'
            time.sleep(.025)
    finally:
        record['elapsed_seconds'] = time.monotonic()-start
        save(root / 'exec-barriers.json', records)


def same_identity(actual, expected):
    for field in ('pid', 'path', 'uid', 'ppid', 'pgid', 'lstart'):
        assert actual[field] == expected[field], (field, actual, expected)
    assert actual['ni'] == actual['getpriority'] == 15, actual


def environment(directory, daemon):
    inherited = {key: value for key, value in os.environ.items()
                 if not key.startswith(('OPENERGY_', 'XDG_'))}
    return dict(inherited, OPENERGY_CONFIG=str(directory / 'config.toml'), OPENERGY_CONFIG_DIR=str(directory),
                OPENERGY_TLS_DIR=str(directory / 'tls'), OPENERGY_SOCKET=str(directory / 'ipc.sock'),
                OPENERGY_DAEMON=str(daemon), OPENERGY_DAEMON_ARGS='--platform virtual',
                OPENERGY_ALLOW_PORT_24800='0', OPENERGY_DISABLE_DISCOVERY='1',
                OPENERGY_E2E_NO_RENDER='1', OPENERGY_IPC_TRACE='1',
                XDG_CONFIG_HOME=str(directory), XDG_RUNTIME_DIR=str(directory),
                XDG_DATA_HOME=str(directory), XDG_CACHE_HOME=str(directory))


def stop_group(process):
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except ProcessLookupError:
        pass
    try:
        process.wait(timeout=1)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        process.wait(timeout=.5)


def measure(root):
    os.umask(0o077)
    ready = json.loads((root / 'ready.json').read_text())
    daemon = Path(ready['daemon'])
    assert sha(daemon) == ready['daemon_sha256'] == DAEMON_SHA
    assert sha(Path(ready['app'])) == ready['app_sha256']
    run = root / 'run'
    run.mkdir(mode=0o700)  # Exclusive, immutable attempt marker: never reuse this root.
    priority(root, 'runtime')  # Stop on a denied absolute priority before launching anything.
    peer_dir, output = run / 'peer', run / 'flow'
    peer_dir.mkdir(mode=0o700)
    output.mkdir(mode=0o700)
    reservations, processes, samples, cli, barriers = [], [], [], [], []
    owned, identities = {}, {}
    start = time.monotonic()
    deadline = start + 52  # Eight seconds reserved for strict cleanup inside 60 seconds.
    failure = None
    ports = []
    steady_start = None
    save(root / 'runtime-entry.json', dict(controller_pid=os.getpid(), start_monotonic=start,
         start_unix_ms=time.time()*1000, action_deadline_seconds=52, wall_limit_seconds=60,
         controller_sha256=sha(HERE / 'measure.py'), overlay_sha256=sha(HERE / 'harness-overlay.patch')))
    try:
        for _ in range(2):
            s = socket.socket()
            s.bind(('127.0.0.1', 0))
            s.listen()
            reservations.append(s)
            port = s.getsockname()[1]
            assert port not in (24800, 5353, 25600) and not 25100 <= port <= 25119, port
        peer_port, host_port = ports = [s.getsockname()[1] for s in reservations]
        assert peer_port != host_port
        (peer_dir / 'config.toml').write_text(f'''version = 1
role = "server"
name = "{PEER_NAME}"
[server]
listen = "127.0.0.1:{peer_port}"
port = {peer_port}
[client]
remote = ""
autoconnect = false
[tls]
enabled = true
require_client_trust = false
[[screen]]
name = "{PEER_NAME}"
rect = {{ x = 0, y = 0, w = 1920, h = 1080 }}
[[screen]]
name = "app"
rect = {{ x = 1920, y = 0, w = 1920, h = 1080 }}
''')
        peer_env = environment(peer_dir, daemon)
        app_env = environment(output / 'app', daemon)
        argv_peer = [str(daemon), 'daemon', '--platform', 'virtual', '--foreground',
                     '--config', peer_env['OPENERGY_CONFIG'], '--socket', peer_env['OPENERGY_SOCKET'],
                     '--parent-pid', str(os.getpid())]

        def requests():
            stdout = (run / 'app.stdout').read_text() if (run / 'app.stdout').exists() else ''
            return [json.loads(line.split('IPC → ', 1)[1]) for line in stdout.splitlines()
                    if line.startswith('IPC → ')]

        def ctl(label, method='status.get'):
            assert len(requests()) + len(cli) < 128, 'public request cap'
            result = command([daemon, 'ctl', method, '--socket', peer_env['OPENERGY_SOCKET'], '--raw'], run, peer_env, 1.5)
            cli.append(dict(label=label, **result))
            save(root / 'public-cli.json', cli)
            assert result['exit'] == 0, result
            return json.loads(result['stdout'])

        reservations[0].close()
        with (run / 'peer.log').open('w') as log:
            peer = subprocess.Popen(argv_peer, cwd=run, env=peer_env, stdout=log, stderr=log, start_new_session=True)
        processes.append(peer)
        owned['peer'] = peer.pid
        identities['peer'] = exec_barrier(root, 'peer', peer.pid, daemon, os.getpid(), {peer.pid}, deadline, barriers)
        while not (peer_dir / 'ipc.sock').exists():
            assert peer.poll() is None and time.monotonic() < min(start+3, deadline), 'peer startup failed'
            time.sleep(.025)
        initial = ctl('peer-before-app')
        assert initial['daemon']['pid'] == peer.pid and initial['daemon']['platform'] == 'virtual', initial
        assert initial['role'] == 'server' and initial['name'] == PEER_NAME and initial['state'] == 'listening', initial
        assert initial['server']['listen'] == f'127.0.0.1:{peer_port}' and initial['tls']['enabled'], initial
        fingerprint = initial['tls']['fingerprint']
        assert re.fullmatch(r'(?:[0-9A-F]{2}:){31}[0-9A-F]{2}', fingerprint), fingerprint
        argv_app = [ready['app'], '--e2e', str(output), '--join', f'127.0.0.1:{peer_port}',
                    '--host-port', str(host_port), '--measure-idle',
                    '--expected-peer-fingerprint', fingerprint, '--expected-peer-name', PEER_NAME]
        save(root / 'run-command.json', dict(peer=argv_peer, app=argv_app, nice_wrappers=[],
             explicit_environment={k: v for k, v in app_env.items() if k.startswith(('OPENERGY_', 'XDG_'))},
             peer_environment={k: v for k, v in peer_env.items() if k.startswith(('OPENERGY_', 'XDG_'))},
             max_public_requests=128, max_daemon_processes=2, runtime_wall_limit_seconds=60,
             expected_peer_fingerprint=fingerprint, ports=ports,
             sampling='libproc rusage v0/TASKINFO/LISTFDS; ps identity/NI/state; compiler inventory',
             sample_target_period_seconds=1, clock='time.monotonic plus Unix wall timestamp',
             cpu_counter_units='nanoseconds, libproc rusage v0',
             cpu_counter_resolution='OS counter resolution not established; integer ns storage',
             ps_cumulative_time_display_resolution_seconds=.01, fd_capacity=4096))
        reservations[1].close()
        with (run / 'app.stdout').open('w') as out, (run / 'app.stderr').open('w') as err:
            app = subprocess.Popen(argv_app, cwd=run, env=app_env, stdout=out, stderr=err, start_new_session=True)
        processes.append(app)
        owned['app'] = app.pid
        identities['app'] = exec_barrier(root, 'app', app.pid, ready['app'], os.getpid(), {app.pid}, deadline, barriers)
        handled = set()
        phase = 'startup'
        next_sample = None
        ready_verified = False

        def sample(label):
            assert ready_verified and set(identities) == {'app', 'daemon', 'peer'}
            now = time.monotonic()
            data = dict(elapsed_seconds=now-start, phase=label, unix_ms=time.time()*1000,
                        steady_elapsed_seconds=None if steady_start is None else now-steady_start,
                        processes={})
            for name, pid in owned.items():
                actual = identity(pid, run)
                same_identity(actual, identities[name])
                resource = process_sample(pid)
                assert resource['path'] == identities[name]['path'], resource
                data['processes'][name] = dict(**resource, identity=actual, effective_ni=actual['ni'],
                                               fd_count=len(resource['fds']))
            data['compiler_inventory'] = compilers()
            data['sample_duration_seconds'] = time.monotonic()-now
            samples.append(data)
            save(root / 'os-samples.json', samples)

        def peer_connected(label, row):
            assert row['app_pid'] == app.pid and row['daemon_pid'] == owned['daemon'] and row['is_ready'], row
            status_app = row['status']
            assert status_app['daemon']['pid'] == owned['daemon'] and status_app['daemon']['platform'] == 'virtual', row
            assert status_app['role'] == 'client' and status_app['state'] == 'connected', row
            app_peers = [p for p in status_app['peers'] if p['connected']]
            assert len(app_peers) == 1 and app_peers[0]['name'] == PEER_NAME, row
            assert app_peers[0]['address'] == f'127.0.0.1:{peer_port}', row
            assert app_peers[0]['tls']['enabled'] and app_peers[0]['tls']['trusted'], row
            assert app_peers[0]['tls']['fingerprint'] == fingerprint, row
            status = ctl(label)
            assert status['daemon']['pid'] == peer.pid and status['daemon']['platform'] == 'virtual', status
            assert status['role'] == 'server' and status['state'] == 'connected', status
            assert status['name'] == PEER_NAME and status['tls']['fingerprint'] == fingerprint, status
            peers = [p for p in status['peers'] if p['connected']]
            assert len(peers) == 1 and peers[0]['name'] == 'app', status
            assert peers[0]['tls']['enabled'] and peers[0]['tls']['fingerprint'] == status_app['tls']['fingerprint'], status
            return status

        while app.poll() is None:
            assert time.monotonic() < deadline, '52s controller action deadline'
            assert peer.poll() is None, 'owned peer exited during app run'
            assert len(requests()) + len(cli) <= 128, 'public request cap'
            events = output / 'idle.jsonl'
            lines = events.read_bytes().split(b'\n')[:-1] if events.exists() else []
            rows = [json.loads(line) for line in lines]
            for row in rows:
                kind = row['kind']
                if kind in handled or kind == 'check':
                    continue
                handled.add(kind)
                if kind == 'begin':
                    owned['daemon'] = int(row['daemon_pid'])
                    assert owned['daemon'] > 0 and row['app_pid'] == app.pid, row
                    identities['daemon'] = exec_barrier(root, 'daemon', owned['daemon'], daemon, app.pid,
                                                        {app.pid, owned['daemon']}, deadline, barriers)
                elif kind == 'ready':
                    peer_connected('peer-connected-before-idle', row)
                    assert set(identities) == {'app', 'daemon', 'peer'}
                    for name, pid in owned.items():
                        same_identity(identity(pid, run), identities[name])
                    ready_verified = True
                    sample('trusted-ready')
                    sockets = command(['/usr/sbin/lsof', '-w', '-nP', '-a', '-p',
                                       ','.join(map(str, owned.values())), '-i'], run, timeout=1.5)
                    save(root / 'lsof-ready.json', sockets)
                    assert sockets['exit'] == 0, sockets
                    assert f'127.0.0.1:{peer_port}' in sockets['stdout'], sockets
                    assert ':24800' not in sockets['stdout'] and ':5353' not in sockets['stdout'], sockets
                    (output / 'peer-ready').touch(exist_ok=False)
                elif kind == 'steady_begin':
                    assert ready_verified
                    steady_start = time.monotonic()
                    phase = 'steady_idle'
                    sample('steady_begin')
                    next_sample = steady_start + 1
                elif kind == 'steady_end':
                    assert phase == 'steady_idle'
                    sample('steady_end')
                    phase = 'recovery'
                    next_sample = None
                    (output / 'idle-sampled').touch(exist_ok=False)
                elif kind == 'rejoined':
                    peer_connected('peer-connected-after-rejoin', row)
                    sample('rejoined')
                    (output / 'rejoin-readback').touch(exist_ok=False)
                elif kind == 'end':
                    phase = 'teardown'
            if next_sample is not None and time.monotonic() >= next_sample and phase == 'steady_idle':
                sample(phase)
                next_sample += 1
            time.sleep(.025)
        assert app.returncode == 0, f'app exit {app.returncode}'
        final_rows = [json.loads(line) for line in (output / 'idle.jsonl').read_text().splitlines()]
        assert {'begin', 'ready', 'steady_begin', 'steady_end', 'rejoined', 'end'} <= {r['kind'] for r in final_rows}
        report = json.loads((output / 'report.json').read_text())
        save(root / 'flow-report.json', report)
        assert report and all(step['passed'] for step in report), report
        ctl('normal-peer-stop', 'daemon.stop')
        assert peer.wait(timeout=1) == 0
    except BaseException:
        failure = traceback.format_exc()
        (root / 'runtime-first-failure.txt').write_text(failure)
        raise
    finally:
        cleanup = dict(first_failure=failure, owned=owned, groups=[], processes=[], socket_paths=[], ports=[], signals=[])
        # Only groups of our Popen children and the identity-verified child daemon.
        groups = {p.pid for p in processes}
        if 'daemon' in identities:
            groups.add(identities['daemon']['pgid'])
        assert os.getpgrp() not in groups
        for process in reversed(processes):
            if process.poll() is None:
                cleanup['signals'].append(dict(group=process.pid, action='SIGTERM; SIGKILL only after 1s'))
            stop_group(process)
        if 'daemon' in identities:
            pid = owned['daemon']
            try:
                actual = identity(pid, run)
                for key in ('path', 'uid', 'pgid', 'lstart'):
                    assert actual[key] == identities['daemon'][key], (key, actual)
                os.kill(pid, signal.SIGTERM)
                cleanup['signals'].append(dict(pid=pid, action='SIGTERM verified owned daemon'))
                until = time.monotonic() + 1
                while time.monotonic() < until:
                    try:
                        os.kill(pid, 0)
                    except ProcessLookupError:
                        break
                    time.sleep(.025)
                else:
                    os.kill(pid, signal.SIGKILL)
                    cleanup['signals'].append(dict(pid=pid, action='SIGKILL verified owned daemon'))
            except (ProcessLookupError, AssertionError) as e:
                # A proc_pidpath failure can also be a zombie: the strict PID check below stays red.
                cleanup['daemon_cleanup_observation'] = str(e)
        for s in reservations:
            s.close()
        cleanup['process_cleanup_elapsed_seconds'] = time.monotonic()-start
        save(root / 'os-samples.json', samples)
        for group in sorted(groups):
            try:
                os.killpg(group, 0)
                alive = True
            except ProcessLookupError:
                alive = False
            cleanup['groups'].append(dict(pgid=group, alive=alive))
        for name, pid in owned.items():
            try:
                os.kill(pid, 0)
                alive = True
            except ProcessLookupError:
                alive = False
            cleanup['processes'].append(dict(name=name, pid=pid, alive=alive,
                 exit=next((p.returncode for p in processes if p.pid == pid), None)))
        for path in (peer_dir / 'ipc.sock', output / 'app/ipc.sock'):
            cleanup['socket_paths'].append(dict(path=str(path), exists=path.exists()))
        for port in ports:
            try:
                result = command(['/usr/sbin/lsof', '-w', '-nP', f'-iTCP:{port}'], run, timeout=1)
                with socket.socket() as probe:
                    probe.bind(('127.0.0.1', port))
                    result['fresh_exclusive_loopback_bind'] = True
            except BaseException:
                result = dict(failure=traceback.format_exc(), fresh_exclusive_loopback_bind=False)
            cleanup['ports'].append(dict(port=port, **result))
        tls_records = []
        for tls in (peer_dir / 'tls', output / 'app/tls'):
            files = [dict(path=str(p.relative_to(root)), size=p.stat().st_size,
                          mode=oct(p.stat().st_mode & 0o777)) for p in tls.rglob('*') if p.is_file()]
            # Record metadata only. Never read or archive TLS key contents.
            save(root / 'tls-removal-pending.json', tls_records + [dict(directory=str(tls), files=files)])
            if tls.exists():
                assert tls.resolve().is_relative_to(run)
                shutil.rmtree(tls)
            tls_records.append(dict(directory=str(tls), files=files, removed=not tls.exists()))
        save(root / 'tls-removal.json', tls_records)
        cleanup['runtime_elapsed_seconds'] = time.monotonic()-start
        cleanup['passed'] = (all(not g['alive'] for g in cleanup['groups'])
             and all(not p['alive'] for p in cleanup['processes'])
             and all(not s['exists'] for s in cleanup['socket_paths'])
             and all(p.get('exit') == 1 and not p.get('stdout') and p['fresh_exclusive_loopback_bind'] for p in cleanup['ports'])
             and all(t['removed'] for t in tls_records) and cleanup['runtime_elapsed_seconds'] <= 60)
        save(root / 'cleanup.json', cleanup)
        stdout = (run / 'app.stdout').read_text() if (run / 'app.stdout').exists() else ''
        observed = requests()
        idle_trace = (stdout.split('IDLE steady_begin ', 1)[1].split('IDLE steady_end ', 1)[0]
                      if 'IDLE steady_begin ' in stdout and 'IDLE steady_end ' in stdout else None)
        idle_requests = sum(line.startswith('IPC → ') for line in idle_trace.splitlines()) if idle_trace is not None else None
        accounting = dict(app_requests=len(observed), cli_requests=len(cli), total_public_requests=len(observed)+len(cli),
             steady_idle_app_requests=idle_requests,
             methods={m: sum(r['method']==m for r in observed) for m in sorted({r['method'] for r in observed})},
             largest_app_request_utf8_bytes=max((len(json.dumps(r, separators=(',', ':')).encode()) for r in observed), default=0))
        save(root / 'request-accounting.json', accounting)
        assert cleanup['passed'], cleanup
        assert accounting['total_public_requests'] <= 128, accounting
        if failure is None:
            assert idle_requests == 0, accounting
    print(root, flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--measure', type=Path, help='run once in an already prepared owned root')
    args = parser.parse_args()
    if args.measure:
        measure(args.measure.resolve(strict=True))
    else:
        prepare()
