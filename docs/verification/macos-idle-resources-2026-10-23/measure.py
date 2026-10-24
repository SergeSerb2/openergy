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
HEAD = '8c9f0abbdf4016e22738b5b5a51e7c35185e3596'
DAEMON_SHA = '350db4541b95f67b8a295419d71933eb4832d07de37881c781664d212a29d572'
PEER_NAME = 'idle-peer-9df59d98'


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


def prepare():
    os.umask(0o077)
    root = Path(tempfile.mkdtemp(prefix='oe-idle-9df59d98-', dir='/tmp')).resolve()
    print(root, flush=True)
    try:
        head = command(['git', 'rev-parse', 'HEAD'], REPO)['stdout'].strip()
        assert head == HEAD, head
        reference_path = REPO / 'docs/verification/sparkle-integration-2026-10-23/results.json'
        reference = json.loads(reference_path.read_text())
        checked = {name: sha(REPO / name) for name in reference['source_sha256']}
        assert checked == reference['source_sha256'], 'cached daemon source mismatch'
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
        argv = ['nice', '-n', '15', 'swift', 'build', '--package-path', source / 'apps/macos',
                '--scratch-path', scratch, '--cache-path', root / 'cache', '--config-path', root / 'config',
                '--security-path', root / 'security', '--manifest-cache', 'local', '-c', 'release',
                '--jobs', '2', '--disable-automatic-resolution', '--skip-update']
        metadata = dict(root=str(root), source_head=head, reference_sha256=sha(reference_path),
                        matched_source_input_count=len(checked), source_sha256=checked,
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
        build = command(argv, root, env, timeout=240)
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


def environment(directory, daemon):
    return dict(os.environ, OPENERGY_CONFIG=str(directory / 'config.toml'), OPENERGY_CONFIG_DIR=str(directory),
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
        process.wait(timeout=2)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        process.wait(timeout=2)


def measure(root):
    os.umask(0o077)
    ready = json.loads((root / 'ready.json').read_text())
    daemon = Path(ready['daemon'])
    assert sha(daemon) == ready['daemon_sha256'] == DAEMON_SHA
    assert sha(Path(ready['app'])) == ready['app_sha256']
    run = root / 'run'
    run.mkdir(mode=0o700)  # Exclusive: this root cannot run a second runtime attempt.
    peer_dir, output = run / 'peer', run / 'flow'
    peer_dir.mkdir(mode=0o700)
    output.mkdir(mode=0o700)
    assert not (output / 'app').exists()
    reservations, processes, samples, cli = [], [], [], []
    owned = {}
    start = time.monotonic()
    deadline = start + 55  # Leave five seconds for owned-process teardown within 60 seconds.
    failure = None
    try:
        for _ in range(2):
            s = socket.socket()
            s.bind(('127.0.0.1', 0))
            s.listen()
            reservations.append(s)
            p = s.getsockname()[1]
            assert p not in (24800, 5353, 25600) and not 25100 <= p <= 25119, p
        peer_port, host_port = [s.getsockname()[1] for s in reservations]
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
        argv_peer = ['nice', '-n', '15', str(daemon), 'daemon', '--platform', 'virtual', '--foreground',
                     '--config', peer_env['OPENERGY_CONFIG'], '--socket', peer_env['OPENERGY_SOCKET'],
                     '--parent-pid', str(os.getpid())]

        def ctl(label, method='status.get'):
            result = command([daemon, 'ctl', method, '--socket', peer_env['OPENERGY_SOCKET'], '--raw'], run, peer_env, 2)
            cli.append(dict(label=label, **result))
            save(root / 'public-cli.json', cli)
            assert result['exit'] == 0, result
            return json.loads(result['stdout'])

        reservations[0].close()
        with (run / 'peer.log').open('w') as log:
            peer = subprocess.Popen(argv_peer, cwd=run, env=peer_env, stdout=log, stderr=log, start_new_session=True)
        processes.append(peer)
        owned['peer'] = peer.pid
        while not (peer_dir / 'ipc.sock').exists():
            assert peer.poll() is None and time.monotonic() < deadline, 'peer startup failed'
            time.sleep(.025)
        initial = ctl('peer-before-app')
        assert initial['daemon']['pid'] == peer.pid and initial['daemon']['platform'] == 'virtual', initial
        assert initial['role'] == 'server' and initial['name'] == PEER_NAME and initial['state'] == 'listening', initial
        assert initial['server']['listen'] == f'127.0.0.1:{peer_port}' and initial['tls']['enabled'], initial
        fingerprint = initial['tls']['fingerprint']
        assert re.fullmatch(r'(?:[0-9A-F]{2}:){31}[0-9A-F]{2}', fingerprint), fingerprint
        argv_app = ['nice', '-n', '15', ready['app'], '--e2e', str(output), '--join',
                    f'127.0.0.1:{peer_port}', '--host-port', str(host_port), '--measure-idle',
                    '--expected-peer-fingerprint', fingerprint, '--expected-peer-name', PEER_NAME]
        save(root / 'run-command.json', dict(peer=argv_peer, app=argv_app,
             explicit_environment={k: v for k, v in app_env.items() if k.startswith(('OPENERGY_', 'XDG_'))},
             peer_environment={k: v for k, v in peer_env.items() if k.startswith(('OPENERGY_', 'XDG_'))},
             max_public_requests=128, max_daemon_processes=2, runtime_wall_limit_seconds=60,
             expected_peer_fingerprint=fingerprint, ports=[peer_port, host_port],
             sampling='libproc proc_pid_rusage v0, proc_pidinfo TASKINFO/LISTFDS; ps OS states; compiler inventory'))
        reservations[1].close()
        with (run / 'app.stdout').open('w') as out, (run / 'app.stderr').open('w') as err:
            app = subprocess.Popen(argv_app, cwd=run, env=app_env, stdout=out, stderr=err, start_new_session=True)
        processes.append(app)
        owned['app'] = app.pid
        handled = set()
        phase = 'startup'
        next_sample = time.monotonic()
        steady_start = None

        def sample(label):
            now = time.monotonic()
            data = dict(elapsed_seconds=now-start, phase=label, unix_ms=time.time()*1000,
                        steady_elapsed_seconds=None if steady_start is None else now-steady_start,
                        processes={name: process_sample(pid) for name, pid in owned.items()})
            for name, info in data['processes'].items():
                expected = ready['app'] if name == 'app' else str(daemon)
                assert info['path'] == expected, (name, info['path'], expected)
            data['ps'] = command(['/bin/ps', '-p', ','.join(map(str, owned.values())), '-o',
                                  'pid=,ppid=,ni=,rss=,pcpu=,time=,state=,comm='], run, timeout=2)
            assert data['ps']['exit'] == 0, data['ps']
            data['compiler_inventory'] = compilers()
            data['sample_duration_seconds'] = time.monotonic()-now
            samples.append(data)
            save(root / 'os-samples.json', samples)

        def peer_connected(label, row):
            status = ctl(label)
            assert status['daemon']['pid'] == peer.pid and status['daemon']['platform'] == 'virtual', status
            assert status['name'] == PEER_NAME and status['tls']['fingerprint'] == fingerprint, status
            peers = [p for p in status['peers'] if p['connected']]
            assert len(peers) == 1 and peers[0]['name'] == 'app', status
            assert peers[0]['tls']['enabled'] and peers[0]['tls']['fingerprint'] == row['status']['tls']['fingerprint'], status
            return status

        while app.poll() is None:
            assert time.monotonic() < deadline, '55s controller deadline (60s including teardown)'
            assert peer.poll() is None, 'owned peer exited during app run'
            events = output / 'idle.jsonl'
            rows = [json.loads(line) for line in events.read_text().splitlines()] if events.exists() else []
            for row in rows:
                kind = row['kind']
                if kind in handled or kind == 'check':
                    continue
                handled.add(kind)
                if kind == 'begin':
                    owned['daemon'] = int(row['daemon_pid'])
                    assert owned['daemon'] > 0 and row['app_pid'] == app.pid
                    parents = command(['/bin/ps', '-p', str(owned['daemon']), '-o', 'ppid='], run)
                    assert int(parents['stdout']) == app.pid, parents
                    save(root / 'daemon-parent.json', parents)
                    sample('startup-daemon-identified')
                elif kind == 'ready':
                    peer_connected('peer-connected-before-idle', row)
                    save(root / 'lsof-ready.json', command(['/usr/sbin/lsof', '-w', '-nP', '-a', '-p',
                         ','.join(map(str, owned.values()))], run, timeout=2))
                    (output / 'peer-ready').touch(exist_ok=False)
                elif kind == 'steady_begin':
                    steady_start = time.monotonic()
                    phase = 'steady_idle'
                    sample('steady_begin')
                    next_sample = steady_start + 1
                elif kind == 'steady_end':
                    sample('steady_end')
                    phase = 'recovery'
                    (output / 'idle-sampled').touch(exist_ok=False)
                elif kind == 'rejoined':
                    peer_connected('peer-connected-after-rejoin', row)
                    sample('rejoined')
                    (output / 'rejoin-readback').touch(exist_ok=False)
                elif kind == 'end':
                    phase = 'teardown'
            if time.monotonic() >= next_sample and phase != 'teardown':
                sample(phase)
                next_sample += 1
            time.sleep(.025)
        assert app.returncode == 0, f'app exit {app.returncode}'
        final_rows = [json.loads(line) for line in (output / 'idle.jsonl').read_text().splitlines()]
        assert {'begin', 'ready', 'steady_begin', 'steady_end', 'rejoined', 'end'} <= {r['kind'] for r in final_rows}
        ctl('normal-peer-stop', 'daemon.stop')
        assert peer.wait(timeout=2) == 0
        stdout = (run / 'app.stdout').read_text()
        requests = [json.loads(line.split('IPC → ', 1)[1]) for line in stdout.splitlines() if line.startswith('IPC → ')]
        idle_trace = stdout.split('IDLE steady_begin ', 1)[1].split('IDLE steady_end ', 1)[0]
        idle_requests = sum(line.startswith('IPC → ') for line in idle_trace.splitlines())
        save(root / 'request-accounting.json', dict(app_requests=len(requests), cli_requests=len(cli),
             total_public_requests=len(requests)+len(cli), steady_idle_app_requests=idle_requests,
             methods={m: sum(r['method']==m for r in requests) for m in sorted({r['method'] for r in requests})},
             largest_app_request_utf8_bytes=max(len(json.dumps(r, separators=(',', ':')).encode()) for r in requests)))
        assert len(requests)+len(cli) <= 128
        assert idle_requests == 0, idle_requests
    except BaseException:
        failure = traceback.format_exc()
        (root / 'runtime-first-failure.txt').write_text(failure)
        raise
    finally:
        # This only addresses PIDs/groups started by this invocation.
        for process in reversed(processes):
            stop_group(process)
        for s in reservations:
            s.close()
        save(root / 'os-samples.json', samples)
        cleanup = dict(runtime_elapsed_seconds=time.monotonic()-start, first_failure=failure,
                       owned=owned, groups=[], processes=[], socket_paths=[], ports=[])
        for process in processes:
            try:
                os.killpg(process.pid, 0)
                alive = True
            except ProcessLookupError:
                alive = False
            cleanup['groups'].append(dict(pgid=process.pid, alive=alive, exit=process.returncode))
        for name, pid in owned.items():
            try:
                os.kill(pid, 0)
                alive = True
            except ProcessLookupError:
                alive = False
            cleanup['processes'].append(dict(name=name, pid=pid, alive=alive))
        for path in (peer_dir / 'ipc.sock', output / 'app/ipc.sock'):
            cleanup['socket_paths'].append(dict(path=str(path), exists=path.exists()))
        ports = json.loads((root / 'run-command.json').read_text())['ports'] if (root / 'run-command.json').exists() else []
        for port in ports:
            r = command(['/usr/sbin/lsof', '-w', '-nP', f'-iTCP:{port}'], run, timeout=2)
            with socket.socket() as probe:
                probe.bind(('127.0.0.1', port))
                r['fresh_exclusive_loopback_bind'] = True
            cleanup['ports'].append(r)
        cleanup['postflight_elapsed_seconds'] = time.monotonic()-start
        save(root / 'cleanup.json', cleanup)
    print(root, flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--measure', type=Path, help='run once in an already prepared owned root')
    args = parser.parse_args()
    if args.measure:
        measure(args.measure.resolve(strict=True))
    else:
        prepare()
