#!/usr/bin/env python3
"""Bounded real whole-app delivery measurement. No production source mutation."""
import argparse
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

REPO = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def save(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def command(argv, root, env=None, timeout=180):
    start = time.monotonic()
    result = subprocess.run(argv, cwd=root, env=env, capture_output=True, text=True, timeout=timeout)
    return dict(argv=list(map(str, argv)), cwd=str(root), exit=result.returncode,
                elapsed_seconds=time.monotonic() - start, stdout=result.stdout, stderr=result.stderr)


def environment(directory, daemon):
    return dict(os.environ, OPENERGY_CONFIG=str(directory / 'config.toml'),
                OPENERGY_CONFIG_DIR=str(directory), OPENERGY_TLS_DIR=str(directory / 'tls'),
                OPENERGY_SOCKET=str(directory / 'ipc.sock'), OPENERGY_DAEMON=str(daemon),
                OPENERGY_DAEMON_ARGS='--platform virtual', OPENERGY_ALLOW_PORT_24800='0',
                OPENERGY_DISABLE_DISCOVERY='1', OPENERGY_E2E_NO_RENDER='1', OPENERGY_IPC_TRACE='1',
                XDG_CONFIG_HOME=str(directory), XDG_RUNTIME_DIR=str(directory),
                XDG_DATA_HOME=str(directory), XDG_CACHE_HOME=str(directory))


def compilers():
    output = subprocess.check_output(['/bin/ps', '-axo', 'pid=,ppid=,comm='], text=True)
    return [line.strip() for line in output.splitlines()
            if re.search(r'/(swift(?:c|-frontend|-build)?|clang(?:\+\+)?|rustc|cargo|ld)$', line)]


def prepare():
    os.umask(0o077)
    root = Path(tempfile.mkdtemp(prefix='oe-delivery-', dir='/tmp')).resolve()
    state = json.loads(Path('/tmp/oe-sparkle-combined-20261023-state.json').read_text())
    cache = Path(state['swift_scratch'])
    scratch = root / 'swift'
    scratch.mkdir()
    source = root / 'source'
    shutil.copytree(REPO / 'apps/macos', source / 'apps/macos',
                    ignore=shutil.ignore_patterns('.build', '.swiftpm'))
    original_hashes = {str(p.relative_to(cache)): digest(p) for part in ('checkouts', 'repositories', 'artifacts')
                       for p in (cache / part).rglob('*') if p.is_file() and not p.is_symlink()}
    for part in ('checkouts', 'repositories', 'artifacts'):
        shutil.copytree(cache / part, scratch / part, symlinks=True)
    workspace = (cache / 'workspace-state.json').read_text()
    (scratch / 'workspace-state.json').write_text(workspace.replace(str(cache), str(scratch)))
    overlay = command(['/usr/bin/patch', '-p1', '-i', str(HERE / 'harness-overlay.patch')], source)
    assert overlay['exit'] == 0, overlay
    reference = json.loads((REPO / 'docs/verification/sparkle-integration-2026-10-23/results.json').read_text())
    mismatches = [name for name, sha in reference['source_sha256'].items() if digest(REPO / name) != sha]
    assert not mismatches, mismatches
    cached_bundle = Path(json.loads(Path('/tmp/oe-app-current-flow-20261023-state.json').read_text())['source_app'])
    daemon = root / 'openergy-daemon'
    shutil.copy2(cached_bundle / 'Contents/Helpers/openergy', daemon)
    env = dict(os.environ, MACOSX_DEPLOYMENT_TARGET='15.0',
               CLANG_MODULE_CACHE_PATH=str(root / 'clang-cache'),
               SWIFTPM_MODULECACHE_OVERRIDE=str(root / 'module-cache'))
    argv = ['nice', '-n', '15', 'swift', 'build', '--package-path', str(source / 'apps/macos'),
            '--scratch-path', str(scratch), '--cache-path', str(root / 'cache'),
            '--config-path', str(root / 'config'), '--security-path', str(root / 'security'),
            '--manifest-cache', 'local', '-c', 'release', '--jobs', '2',
            '--disable-automatic-resolution', '--skip-update']
    save(root / 'preparation.json', dict(root=str(root), source_head=subprocess.check_output(
        ['git', 'rev-parse', 'HEAD'], cwd=REPO, text=True).strip(),
        cached_state=state, cached_daemon=str(cached_bundle / 'Contents/Helpers/openergy'),
        daemon_sha256=digest(daemon), matched_source_inputs=len(reference['source_sha256']),
        overlay_sha256=digest(HERE / 'harness-overlay.patch'), overlay_apply=overlay,
        build_environment={k: env[k] for k in ('MACOSX_DEPLOYMENT_TARGET', 'CLANG_MODULE_CACHE_PATH', 'SWIFTPM_MODULECACHE_OVERRIDE')},
        compiler_inventory_before_build=compilers()))
    save(root / 'cached-input-hashes.json', original_hashes)
    print(root, flush=True)
    build = command(argv, REPO, env=env, timeout=240)
    save(root / 'build.json', build)
    assert build['exit'] == 0, build['stderr'] + build['stdout']
    finalize(root)
    return root


def finalize(root):
    scratch = root / 'swift'
    preparation = json.loads((root / 'preparation.json').read_text())
    cache = Path(preparation['cached_state']['swift_scratch'])
    original_hashes = json.loads((root / 'cached-input-hashes.json').read_text())
    shared_unchanged = all(digest(cache / name) == sha for name, sha in original_hashes.items())
    assert shared_unchanged
    app = (scratch / 'release/Openergy').resolve(strict=True)
    framework = scratch / 'artifacts/sparkle/Sparkle/Sparkle.xcframework/macos-arm64_x86_64'
    assert (framework / 'Sparkle.framework').is_dir()
    bundle = root / 'Openergy.app'
    cached_bundle = Path(preparation['cached_daemon']).parents[2]
    shutil.copytree(cached_bundle, bundle, symlinks=True)
    executable = bundle / 'Contents/MacOS/Openergy'
    shutil.copy2(app, executable)
    signed = command(['/usr/bin/codesign', '--force', '--sign', '-', str(bundle)], root)
    assert signed['exit'] == 0, signed
    save(root / 'ready.json', dict(app=str(executable), app_sha256=digest(executable),
         compiled_app=str(app), compiled_app_sha256=digest(app), bundle_sign=signed,
         daemon=str(root / 'openergy-daemon'),
         framework=str(framework), cached_inputs_unchanged_after_build=shared_unchanged,
         compiler_inventory_after_build=compilers()))


def stop_group(process):
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except ProcessLookupError:
        pass
    try:
        process.wait(timeout=3)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        process.wait(timeout=3)


def measure(root):
    os.umask(0o077)
    ready = json.loads((root / 'ready.json').read_text())
    daemon = Path(ready['daemon'])
    run = root / 'run'
    run.mkdir(mode=0o700)
    peer_dir = run / 'peer'
    peer_dir.mkdir(mode=0o700)
    output = run / 'flow'
    output.mkdir(mode=0o700)
    reservations = []
    for _ in range(2):
        sock = socket.socket()
        sock.bind(('127.0.0.1', 0))
        sock.listen()
        assert sock.getsockname()[1] not in (24800, 5353)
        reservations.append(sock)
    peer_port, host_port = [s.getsockname()[1] for s in reservations]
    (peer_dir / 'config.toml').write_text(f'''version = 1
role = "server"
name = "peer"
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
name = "peer"
rect = {{ x = 0, y = 0, w = 1920, h = 1080 }}
[[screen]]
name = "app"
rect = {{ x = 1920, y = 0, w = 1920, h = 1080 }}
''')
    peer_env = environment(peer_dir, daemon)
    app_env = environment(output / 'app', daemon)
    app_env['DYLD_FRAMEWORK_PATH'] = ready['framework']
    records, processes = [], []
    argv_peer = ['nice', '-n', '15', str(daemon), 'daemon', '--platform', 'virtual', '--foreground',
                 '--config', peer_env['OPENERGY_CONFIG'], '--socket', peer_env['OPENERGY_SOCKET'],
                 '--parent-pid', str(os.getpid())]
    argv_app = ['nice', '-n', '15', ready['app'], '--e2e', str(output), '--join',
                f'127.0.0.1:{peer_port}', '--host-port', str(host_port), '--measure-delivery']
    save(root / 'run-command.json', dict(peer=argv_peer, app=argv_app,
         explicit_environment={k: v for k, v in app_env.items() if k.startswith(('OPENERGY_', 'XDG_', 'DYLD_'))},
         peer_environment={k: v for k, v in peer_env.items() if k.startswith(('OPENERGY_', 'XDG_'))},
         max_requests_and_connections=128, wall_seconds_limit=60, ports=[peer_port, host_port]))
    deadline = time.monotonic() + 60
    start = time.monotonic()
    try:
        reservations[0].close()
        with (run / 'peer.log').open('w') as stream:
            peer = subprocess.Popen(argv_peer, env=peer_env, stdout=stream, stderr=stream, start_new_session=True)
        processes.append(peer)
        while not (peer_dir / 'ipc.sock').exists():
            assert peer.poll() is None and time.monotonic() < deadline, 'peer startup failed'
            time.sleep(.025)
        status = command([str(daemon), 'ctl', 'status.get', '--socket', peer_env['OPENERGY_SOCKET'], '--raw'], run, peer_env, 3)
        save(root / 'peer-status.json', status)
        parsed = json.loads(status['stdout'])
        assert parsed['daemon']['pid'] == peer.pid and parsed['daemon']['platform'] == 'virtual', parsed
        reservations[1].close()
        with (run / 'app.stdout').open('w') as out, (run / 'app.stderr').open('w') as err:
            app = subprocess.Popen(argv_app, env=app_env, stdout=out, stderr=err, start_new_session=True)
        processes.append(app)
        fd_phases = set()
        while app.poll() is None:
            assert time.monotonic() < deadline, '60s measurement deadline'
            owned = [app.pid, peer.pid]
            events = output / 'delivery.jsonl'
            phase = 'startup'
            if events.exists():
                lines = events.read_text().splitlines()
                for line in reversed(lines):
                    try:
                        row = json.loads(line)
                        phase = row['phase']
                        break
                    except json.JSONDecodeError:
                        continue
                for line in lines:
                    row = json.loads(line)
                    if row['kind'] == 'begin':
                        owned.append(int(row['daemon_pid']))
                        break
            sample = command(['/bin/ps', '-p', ','.join(map(str, owned)), '-o',
                              'pid=,ppid=,rss=,pcpu=,time=,state=,comm='], run, timeout=3)
            records.append(dict(elapsed_seconds=time.monotonic() - start, phase=phase,
                                ps=sample['stdout'], compiler_inventory=compilers()))
            if phase not in fd_phases and phase in ('setup', 'ordinary_delivery', 'settled'):
                fd_phases.add(phase)
                save(root / f'lsof-{phase}.json', command(['/usr/sbin/lsof', '-w', '-nP', '-a', '-p',
                     ','.join(map(str, owned))], run, timeout=3))
            time.sleep(.1)
        assert app.returncode == 0, f'app exit {app.returncode}'
        stopped = command([str(daemon), 'ctl', 'daemon.stop', '--socket', peer_env['OPENERGY_SOCKET'], '--raw'], run, peer_env, 3)
        save(root / 'peer-stop.json', stopped)
        assert peer.wait(timeout=3) == 0
    finally:
        for process in reversed(processes):
            stop_group(process)
        for sock in reservations:
            sock.close()
        save(root / 'os-samples.json', records)
        cleanup = dict(measurement_elapsed_seconds=time.monotonic() - start,
                       groups=[], socket_paths=[], assigned_ports=[])
        for process in processes:
            try:
                os.killpg(process.pid, 0)
                alive = True
            except ProcessLookupError:
                alive = False
            cleanup['groups'].append(dict(pgid=process.pid, alive=alive, exit=process.returncode))
        for path in (peer_dir / 'ipc.sock', output / 'app/ipc.sock'):
            cleanup['socket_paths'].append(dict(path=str(path), exists=path.exists()))
        for port in (peer_port, host_port):
            cleanup['assigned_ports'].append(command(['/usr/sbin/lsof', '-w', '-nP', f'-iTCP:{port}'], run, timeout=3))
            with socket.socket() as probe:
                probe.bind(('127.0.0.1', port))
                cleanup['assigned_ports'][-1]['fresh_bind_succeeded'] = True
        save(root / 'cleanup.json', cleanup)
    print(root, flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--measure', type=Path, help='measure an already prepared private root')
    parser.add_argument('--finalize', type=Path, help='finish metadata for an existing successful build')
    args = parser.parse_args()
    if args.finalize:
        finalize(args.finalize.resolve())
    elif args.measure:
        measure(args.measure.resolve())
    else:
        prepare()
