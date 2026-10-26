#!/usr/bin/env python3
"""Run the headless app E2E against real, private virtual daemons on loopback."""
import argparse
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[3]


def require(value, message):
    if not value:
        raise RuntimeError(message)


def wait(condition, message, timeout=10):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if condition():
            return
        time.sleep(.05)
    raise RuntimeError(f'timeout: {message}')


def environment(directory, daemon):
    # Every invocation, including ctl, gets the same explicit private paths.
    return dict(os.environ, OPENERGY_CONFIG=str(directory / 'config.toml'),
                OPENERGY_CONFIG_DIR=str(directory), OPENERGY_TLS_DIR=str(directory / 'tls'),
                OPENERGY_SOCKET=str(directory / 'ipc.sock'), OPENERGY_DAEMON=str(daemon),
                OPENERGY_DAEMON_ARGS='--platform virtual', OPENERGY_ALLOW_PORT_24800='0',
                OPENERGY_DISABLE_DISCOVERY='1', OPENERGY_E2E_NO_RENDER='1',
                XDG_CONFIG_HOME=str(directory), XDG_RUNTIME_DIR=str(directory),
                XDG_DATA_HOME=str(directory), XDG_CACHE_HOME=str(directory))


def call(daemon, env, method):
    result = subprocess.run([str(daemon), 'ctl', method, '--socket', env['OPENERGY_SOCKET'], '--raw'],
                            env=env, capture_output=True, text=True, timeout=5)
    require(result.returncode == 0, result.stdout or result.stderr)
    return json.loads(result.stdout)


def group_exited(process):
    try:
        os.killpg(process.pid, 0)
    except ProcessLookupError:
        return True
    return False


def stop(process):
    # Each process starts a new session. Only its owned group receives signals,
    # including an app child left behind by a failed or timed-out app command.
    for sig in (signal.SIGCONT, signal.SIGTERM):
        try:
            os.killpg(process.pid, sig)
        except ProcessLookupError:
            break
    try:
        process.wait(timeout=5)
    except subprocess.TimeoutExpired:
        pass
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    process.wait(timeout=5)


def run_app(app, output, env, peer_port, host_port, processes, label):
    process = subprocess.Popen([str(app), '--e2e', str(output), '--join',
                                f'127.0.0.1:{peer_port}', '--host-port', str(host_port),
                                '--require-peer'], env=env, stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE, text=True, start_new_session=True)
    processes.append(process)
    try:
        stdout, stderr = process.communicate(timeout=120)
    except subprocess.TimeoutExpired:
        stop(process)
        stdout, stderr = process.communicate()
        (output / f'{label}.stdout').write_text(stdout)
        (output / f'{label}.stderr').write_text(stderr)
        raise RuntimeError(f'{label}: app timed out')
    (output / f'{label}.stdout').write_text(stdout)
    (output / f'{label}.stderr').write_text(stderr)
    wait(lambda: group_exited(process), 'app and supervised daemon process group exit')
    processes.remove(process)
    return process.returncode, stdout, stderr


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--app', type=Path, default=ROOT / 'apps/macos/.build/release/Openergy')
    parser.add_argument('--daemon', type=Path, default=ROOT / 'target/release/openergy')
    parser.add_argument('--port-base', type=int, choices=range(25760, 25778), default=25760)
    args = parser.parse_args()
    app, daemon = args.app.resolve(), args.daemon.resolve()
    require(app.is_file() and os.access(app, os.X_OK), 'build the release app with swift build -c release --jobs 2')
    require(daemon.is_file() and os.access(daemon, os.X_OK), 'build the release daemon first')
    os.umask(0o077)
    base = Path(tempfile.mkdtemp(prefix='oe-app-', dir='/tmp')).resolve()
    print(f'ARTIFACTS {base}', flush=True)
    processes, reservations, failures = [], [], []
    try:
        # Reserve the complete assigned loopback range before starting either daemon.
        for port in range(args.port_base, args.port_base + 3):
            reservation = socket.socket()
            reservations.append(reservation)
            reservation.bind(('127.0.0.1', port))
            reservation.listen()
        peer_port, host_port, trap_port = range(args.port_base, args.port_base + 3)

        # A caller-supplied client config must fail before the model starts. Its
        # remote is a reserved local trap, so even a regression cannot reach LAN.
        unsafe = base / 'existing-config'
        unsafe.mkdir(mode=0o700)
        stale = unsafe / 'app'
        stale.mkdir(mode=0o700)
        config = stale / 'config.toml'
        original = (f'"role" = "client"\nname = "stale"\n[client]\n'
                    f'"remote" = "127.0.0.1:{trap_port}"\nautoconnect = true\n')
        config.write_text(original)
        code, _, stderr = run_app(app, unsafe, environment(stale, daemon), peer_port,
                                  host_port, processes, 'rejected')
        require(code == 64 and 'config directory must not already exist' in stderr,
                'existing client config was not refused before startup')
        require(config.read_text() == original and sorted(p.name for p in stale.iterdir()) == ['config.toml'],
                'failed preflight changed config or created daemon artifacts')
        reservations[2].settimeout(.2)
        try:
            connection, _ = reservations[2].accept()
        except socket.timeout:
            pass
        else:
            connection.close()
            raise RuntimeError('preflight connected to the stale config remote')
        failures.append({'case': 'existing client config', 'exit': code, 'passed': True})

        # Exercise each explicit isolation input through the real app entry point.
        for label, overrides in [
            ('public-output', {}),
            ('config-escape', {'OPENERGY_CONFIG': str(base / 'outside.toml')}),
            ('config-dir-escape', {'OPENERGY_CONFIG_DIR': str(base / 'outside')}),
            ('tls-escape', {'OPENERGY_TLS_DIR': str(base / 'tls')}),
            ('socket-escape', {'OPENERGY_SOCKET': str(base / 'outside.sock')}),
            ('extra-daemon-args', {'OPENERGY_DAEMON_ARGS': '--platform virtual --foreground'}),
        ]:
            output = base / label
            output.mkdir(mode=0o700)
            if label == 'public-output':
                output.chmod(0o755)
            env = environment(output / 'app', daemon)
            env.update(overrides)
            code, _, stderr = run_app(app, output, env, peer_port, host_port, processes, 'rejected')
            require(code == 64 and 'Unsafe E2E settings:' in stderr, f'{label}: preflight did not fail closed')
            require(not (output / 'app').exists(), f'{label}: preflight created config state')
            output.chmod(0o700)
            failures.append({'case': label, 'exit': code, 'passed': True})

        output = base / 'flow'
        output.mkdir(mode=0o700)
        peer = base / 'peer'
        peer.mkdir(mode=0o700)
        peer_env = environment(peer, daemon)
        (peer / 'config.toml').write_text(f'''version = 1
role = "server"
name = "peer"
[server]
listen = "127.0.0.1:{peer_port}"
port = {peer_port}
[client]
remote = ""
autoconnect = false
[tls]
enabled = false
[[screen]]
name = "peer"
rect = {{ x = 0, y = 0, w = 1920, h = 1080 }}
[[screen]]
name = "app"
rect = {{ x = 1920, y = 0, w = 1920, h = 1080 }}
''')
        reservations[0].close()
        with (peer / 'stderr.log').open('w') as log:
            peer_process = subprocess.Popen([str(daemon), 'daemon', '--config', peer_env['OPENERGY_CONFIG'],
                                             '--socket', peer_env['OPENERGY_SOCKET'], '--platform', 'virtual',
                                             '--parent-pid', str(os.getpid()), '--foreground'],
                                            env=peer_env, stdout=log, stderr=log, start_new_session=True)
        processes.append(peer_process)
        wait(lambda: (peer / 'ipc.sock').exists(), 'private peer startup')
        require(call(daemon, peer_env, 'status.get')['daemon']['platform'] == 'virtual', 'peer is not virtual')
        reservations[1].close()
        code, stdout, _ = run_app(app, output, environment(output / 'app', daemon), peer_port,
                                  host_port, processes, 'app')
        print(stdout, flush=True)
        require(code == 0, 'app E2E failed; see flow/app.stderr')
        report = json.loads((output / 'report.json').read_text())
        required = {'starts from safe off config', 'real peer connected', 'disconnect peer',
                    'rejoin same address', 'owned daemon crash recovers', 'empty import reports error'}
        require(required <= {row['name'] for row in report} and all(row['passed'] for row in report),
                'app did not pass every required E2E step')
        wait(lambda: not (output / 'app/ipc.sock').exists(), 'app child daemon cleanup')
        call(daemon, peer_env, 'daemon.stop')
        require(peer_process.wait(timeout=5) == 0, 'peer shutdown failed')
        wait(lambda: not (peer / 'ipc.sock').exists(), 'peer socket cleanup')
        wait(lambda: group_exited(peer_process), 'peer process group exit')
        processes.remove(peer_process)
        for directory in (base, output, output / 'app', peer):
            require(directory.stat().st_mode & 0o077 == 0, 'artifact directory is not private')
        (base / 'preflight.json').write_text(json.dumps(failures, indent=2))
        (base / 'cleanup.json').write_text(json.dumps({'app_socket_removed': True,
                                                       'peer_socket_removed': True,
                                                       'owned_process_groups_exited': True,
                                                       'private_directories': True,
                                                       'peer_exit': peer_process.returncode}, indent=2))
        print(f'PASS {len(report)} app steps; {len(failures)} preflight refusals; owned daemon cleanup', flush=True)
        print(f'REPORT {output / "report.json"}', flush=True)
    finally:
        for process in reversed(processes):
            stop(process)
        for reservation in reservations:
            reservation.close()
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
