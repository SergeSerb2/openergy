#!/usr/bin/env python3
"""One private whole-app build and one <=15s real caller flow. No retry."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import socket
import subprocess
import tempfile
import time

REPO = Path(__file__).resolve().parents[3]
CACHE = Path('/var/folders/s6/_07cr4w13vsg_myjkv9ldpqc0000gn/T/oe-sparkle-combined-20261023-a_es9dei')


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def save(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def command(root, name, argv, env, timeout):
    start = time.monotonic()
    result = dict(argv=list(map(str, argv)), timeout_seconds=timeout)
    with (root / (name + '.stdout')).open('w') as out, (root / (name + '.stderr')).open('w') as err:
        try:
            process = subprocess.run(argv, cwd=REPO, env=env, stdout=out, stderr=err, timeout=timeout)
            result['exit'] = process.returncode
        except subprocess.TimeoutExpired:
            result['failure'] = 'timeout; no retry'
    result['elapsed_seconds'] = time.monotonic() - start
    save(root / (name + '.json'), result)
    assert result.get('exit') == 0, result
    return result


def cache_hashes():
    # Only dependency source/artifact caches; no identities or signing material.
    return {str(p.relative_to(CACHE / 'swift')): sha(p)
            for part in ('checkouts', 'repositories', 'artifacts')
            for p in (CACHE / 'swift' / part).rglob('*') if p.is_file() and not p.is_symlink()}


def main():
    os.umask(0o077)
    root = Path(tempfile.mkdtemp(prefix='oe-caller-', dir='/tmp')).resolve()
    print(root, flush=True)
    # Do not inherit unrelated app overrides, credentials, or developer settings.
    env = {k: os.environ[k] for k in ('PATH', 'HOME', 'USER', 'LOGNAME', 'SHELL', 'LANG', 'TMPDIR') if k in os.environ}
    env.update(RUSTUP_TOOLCHAIN='1.99.1', CARGO_BUILD_JOBS='2', CARGO_NET_OFFLINE='true',
               MACOSX_DEPLOYMENT_TARGET='15.0', CLANG_MODULE_CACHE_PATH=str(root / 'clang-cache'),
               SWIFTPM_MODULECACHE_OVERRIDE=str(root / 'module-cache'))
    command(root, 'rust-version', ['nice', '-n', '15', '/Users/serge/.cargo/bin/rustc', '--version'], env, 10)
    before = cache_hashes()
    save(root / 'cached-input-hashes.json', before)
    source = root / 'source/apps/macos'
    shutil.copytree(REPO / 'apps/macos', source, ignore=shutil.ignore_patterns('.build', '.swiftpm'))
    scratch = root / 'swift'
    scratch.mkdir()
    for part in ('checkouts', 'repositories', 'artifacts'):
        shutil.copytree(CACHE / 'swift' / part, scratch / part, symlinks=True)
    (scratch / 'workspace-state.json').write_text(
        (CACHE / 'swift/workspace-state.json').read_text().replace(str(CACHE / 'swift'), str(scratch)))
    reference = json.loads((REPO / 'docs/verification/sparkle-integration-2026-10-23/results.json').read_text())['source_sha256']
    harness = 'apps/macos/Sources/Openergy/App/AppEndToEnd.swift'
    inputs = {name: sha(REPO / name) for name in reference
              if name.startswith(('apps/macos/', 'crates/')) or name in ('Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml')}
    differences = [name for name, digest in inputs.items() if digest != reference[name]]
    assert differences == [harness], differences
    save(root / 'source.json', dict(parent=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=REPO, text=True).strip(),
         input_sha256=inputs, differences_from_cached_build=differences, cache=str(CACHE),
         cached_daemon_sha256=sha(CACHE / 'bundle-final/Openergy.app/Contents/Helpers/openergy'),
         build_environment={k: env[k] for k in ('RUSTUP_TOOLCHAIN', 'CARGO_BUILD_JOBS', 'CARGO_NET_OFFLINE',
             'MACOSX_DEPLOYMENT_TARGET', 'CLANG_MODULE_CACHE_PATH', 'SWIFTPM_MODULECACHE_OVERRIDE')}))
    command(root, 'build', ['nice', '-n', '15', 'swift', 'build', '--package-path', str(source),
            '--scratch-path', str(scratch), '--cache-path', str(root / 'cache'),
            '--config-path', str(root / 'config'), '--security-path', str(root / 'security'),
            '--manifest-cache', 'local', '-c', 'release', '--jobs', '2',
            '--disable-automatic-resolution', '--skip-update'], env, 180)
    bundle = root / 'Openergy.app'
    shutil.copytree(CACHE / 'bundle-final/Openergy.app', bundle, symlinks=True)
    app = bundle / 'Contents/MacOS/Openergy'
    shutil.copy2((scratch / 'release/Openergy').resolve(strict=True), app)
    command(root, 'private-sign', ['/usr/bin/codesign', '--force', '--sign', '-', str(bundle)], env, 15)
    assert cache_hashes() == before, 'original dependency caches changed'
    daemon = bundle / 'Contents/Helpers/openergy'
    save(root / 'ready.json', dict(app=str(app), app_sha256=sha(app), daemon=str(daemon),
         daemon_sha256=sha(daemon), original_dependency_files_unchanged=len(before)))

    flow = root / 'flow'
    flow.mkdir(mode=0o700)
    home = root / 'private-home'
    home.mkdir(mode=0o700)
    directory = flow / 'config'
    overrides = dict(OPENERGY_CONFIG=str(directory / 'config.toml'), OPENERGY_CONFIG_DIR=str(directory),
        OPENERGY_TLS_DIR=str(directory / 'tls'), OPENERGY_SOCKET=str(directory / 'ipc.sock'),
        OPENERGY_DAEMON=str(daemon), OPENERGY_DAEMON_ARGS='--platform virtual',
        OPENERGY_DISABLE_DISCOVERY='1', OPENERGY_ALLOW_PORT_24800='0',
        OPENERGY_E2E_NO_RENDER='1', OPENERGY_IPC_TRACE='1', CFFIXED_USER_HOME=str(home),
        XDG_CONFIG_HOME=str(home), XDG_RUNTIME_DIR=str(home), XDG_DATA_HOME=str(home), XDG_CACHE_HOME=str(home))
    runtime_env = dict(env, **overrides)
    reservations = []
    for _ in range(2):
        reservation = socket.socket()
        reservation.bind(('127.0.0.1', 0))
        reservation.listen()
        assert reservation.getsockname()[1] not in (24800, 5353, 25600)
        reservations.append(reservation)
    host_port, unused_join_port = [s.getsockname()[1] for s in reservations]
    argv = ['nice', '-n', '15', str(app), '--e2e', str(flow), '--join',
            f'127.0.0.1:{unused_join_port}', '--host-port', str(host_port), '--caller-cancellation']
    save(root / 'run-command.json', dict(argv=argv, explicit_environment=overrides, wall_limit_seconds=15,
         ports=[host_port, unused_join_port], join_is_unused_in_this_mode=True))
    process = None
    result = {}
    started = time.monotonic()
    try:
        # Neither port is contacted or listened on by this Off-role flow.
        for reservation in reservations:
            reservation.close()
        with (root / 'run.stdout').open('w') as out, (root / 'run.stderr').open('w') as err:
            process = subprocess.Popen(argv, cwd=REPO, env=runtime_env, stdout=out, stderr=err, start_new_session=True)
            result['app_pid'] = process.pid
            try:
                result['exit'] = process.wait(timeout=15)
            except subprocess.TimeoutExpired:
                result['failure'] = '15s wall deadline; first attempt retained; no retry'
        result['elapsed_seconds'] = time.monotonic() - started
        save(root / 'run.json', result)
    finally:
        for reservation in reservations:
            reservation.close()
        # A fresh session contains only this app and its supervised child.
        cleanup = dict(app_pid=process.pid if process else None, actions=[])
        if process is not None:
            inventory = subprocess.run(['/bin/ps', '-axo', 'pid=,ppid=,pgid=,state=,comm='], capture_output=True, text=True, timeout=3)
            owned = [line.strip() for line in inventory.stdout.splitlines()
                     if len(line.split()) >= 4 and line.split()[2] == str(process.pid)]
            cleanup['owned_before_cleanup'] = owned
            for line in owned:
                fields = line.split()
                if fields[1] == str(process.pid) and 'T' in fields[3]:
                    os.kill(int(fields[0]), signal.SIGCONT)
                    cleanup['actions'].append(['SIGCONT', int(fields[0])])
            if owned:
                os.killpg(process.pid, signal.SIGTERM)
                cleanup['actions'].append(['SIGTERM', process.pid])
            if process.poll() is None:
                try:
                    process.wait(timeout=2)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL)
                    cleanup['actions'].append(['SIGKILL', process.pid])
                    process.wait(timeout=2)
            inventory = subprocess.run(['/bin/ps', '-axo', 'pid=,ppid=,pgid=,state=,comm='], capture_output=True, text=True, timeout=3)
            cleanup['owned_after_cleanup'] = [line.strip() for line in inventory.stdout.splitlines()
                if len(line.split()) >= 4 and line.split()[2] == str(process.pid)]
        ipc = directory / 'ipc.sock'
        cleanup['ipc_exists_after_exit'] = ipc.exists()
        listeners = subprocess.run(['/usr/sbin/lsof', '-nP', '-iTCP:' + ','.join(map(str, (host_port, unused_join_port)))],
                                   capture_output=True, text=True, timeout=3)
        cleanup['assigned_port_lsof'] = dict(exit=listeners.returncode, stdout=listeners.stdout, stderr=listeners.stderr)
        cleanup['exclusive_loopback_bind'] = {}
        for port in (host_port, unused_join_port):
            with socket.socket() as probe:
                try:
                    probe.bind(('127.0.0.1', port))
                    cleanup['exclusive_loopback_bind'][port] = True
                except OSError as error:
                    cleanup['exclusive_loopback_bind'][port] = str(error)
        cleanup['original_dependency_files_unchanged'] = cache_hashes() == before
        save(root / 'cleanup.json', cleanup)
    assert result.get('exit') == 0, result
    steps = json.loads((flow / 'report.json').read_text())
    assert steps and all(step['passed'] for step in steps), steps
    assert not cleanup['owned_after_cleanup'] and not cleanup['ipc_exists_after_exit'], cleanup
    assert listeners.returncode == 1 and all(value is True for value in cleanup['exclusive_loopback_bind'].values()), cleanup
    assert cleanup['original_dependency_files_unchanged'], cleanup
    print(json.dumps(dict(root=str(root), checks=len(steps), run=result, cleanup=cleanup)), flush=True)


if __name__ == '__main__':
    main()
