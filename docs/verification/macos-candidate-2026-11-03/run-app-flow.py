"""Run the existing packaged app E2E command against its real private virtual peer.

Usage on the Mac: python3 run-app-flow.py <private artifact run directory>
No normal GUI launch, native backend, native clipboard, login or TCC changes.
"""
from pathlib import Path
import hashlib
import json
import os
import signal
import socket
import subprocess
import sys
import time

os.umask(0o077)
root = Path(sys.argv[1]).resolve()
assert str(root).startswith('/private/tmp/oe-candidate-20261103.')
assert root.stat().st_uid == os.getuid() and root.stat().st_mode & 0o077 == 0
app = root/'extracted/Openergy.app'
binary = app/'Contents/Helpers/openergy'
executable = app/'Contents/MacOS/Openergy'
base = root/'app-flow'
base.mkdir(mode=0o700)
peer = base/'peer'
peer.mkdir(mode=0o700)
home = base/'home'
home.mkdir(mode=0o700)
output = base/'app'
output.mkdir(mode=0o700)
peer_port, host_port = 25760, 25761
for port in (peer_port, host_port):
    with socket.socket() as probe:
        probe.bind(('127.0.0.1', port))
(peer/'config.toml').write_text(f'''version=1
name="e2e-peer"
role="server"
[server]
listen="127.0.0.1:{peer_port}"
port={peer_port}
[tls]
enabled=false
[[screen]]
name="e2e-peer"
rect={{x=0,y=0,w=1920,h=1080}}
[[screen]]
name="app"
rect={{x=1920,y=0,w=1920,h=1080}}
''')
env = {key: value for key, value in os.environ.items()
       if not key.startswith('OPENERGY_') and not any(word in key.upper()
       for word in ('TOKEN', 'SECRET', 'PASSWORD', 'PRIVATE_KEY', 'API_KEY'))}
env.update(OPENERGY_CONFIG=str(peer/'config.toml'), OPENERGY_CONFIG_DIR=str(peer),
           OPENERGY_TLS_DIR=str(peer/'tls'), OPENERGY_SOCKET=str(peer/'control.sock'),
           OPENERGY_DISABLE_DISCOVERY='1', OPENERGY_ALLOW_PORT_24800='0',
           CFFIXED_USER_HOME=str(home), OPENERGY_DAEMON_ARGS='--platform virtual')
appenv = dict(env, OPENERGY_CONFIG=str(output/'config/config.toml'),
              OPENERGY_CONFIG_DIR=str(output/'config'),
              OPENERGY_TLS_DIR=str(output/'config/tls'),
              OPENERGY_SOCKET=str(output/'config/control.sock'), OPENERGY_E2E_NO_RENDER='1')
record = dict(peer_port=peer_port, host_port=host_port, tls_enabled=False, platform='virtual',
              app_sha256=hashlib.sha256(executable.read_bytes()).hexdigest(),
              daemon_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),
              environment={key: value for key, value in appenv.items()
                           if key.startswith('OPENERGY_') or key == 'CFFIXED_USER_HOME'})
process = None
application = None
try:
    # The safety refusal is a real packaged command. The caller-supplied config
    # directory exists, so the harness must refuse before constructing AppModel.
    refusal = base/'refusal'
    refusal.mkdir(mode=0o700)
    (refusal/'config').mkdir(mode=0o700)
    refusal_env = dict(appenv, OPENERGY_CONFIG=str(refusal/'config/config.toml'),
                       OPENERGY_CONFIG_DIR=str(refusal/'config'),
                       OPENERGY_TLS_DIR=str(refusal/'config/tls'),
                       OPENERGY_SOCKET=str(refusal/'config/control.sock'))
    refusal_args = [str(executable), '--e2e', str(refusal), '--join',
                    f'127.0.0.1:{peer_port}', '--host-port', str(host_port)]
    result = subprocess.run(refusal_args, env=refusal_env, capture_output=True, text=True, timeout=15)
    record['safety_refusal'] = dict(arguments=refusal_args, exit=result.returncode,
                                  stdout=result.stdout, stderr=result.stderr)
    assert result.returncode == 64
    with (base/'peer.log').open('w') as peer_log:
        args = ['nice', '-n', '15', str(binary), 'daemon', '--platform', 'virtual',
                '--config', str(peer/'config.toml'), '--socket', str(peer/'control.sock'), '--foreground']
        record['peer_arguments'] = args
        process = subprocess.Popen(args, env=env, stdout=peer_log, stderr=subprocess.STDOUT,
                                   start_new_session=True)
        began = time.monotonic()
        while True:
            status = subprocess.run([str(binary), 'status', '--socket', str(peer/'control.sock')],
                                    env=env, capture_output=True, text=True, timeout=5)
            if status.returncode == 0:
                status = json.loads(status.stdout)
                assert status['daemon']['pid'] == process.pid
                assert status['daemon']['platform'] == 'virtual' and status['state'] == 'listening'
                record['peer_initial_status'] = status
                break
            assert process.poll() is None and time.monotonic()-began < 15
            time.sleep(0.1)
        args = ['nice', '-n', '15', str(executable), '--e2e', str(output),
                '--join', f'127.0.0.1:{peer_port}', '--host-port', str(host_port), '--require-peer']
        record['app_arguments'] = args
        began = time.monotonic()
        with (base/'app.log').open('w') as app_log:
            application = subprocess.Popen(args, env=appenv, stdout=app_log, stderr=subprocess.STDOUT,
                                           start_new_session=True)
            record['app_exit'] = application.wait(timeout=180)
        record['app_elapsed_seconds'] = round(time.monotonic()-began, 3)
        if (output/'report.json').exists():
            record['steps'] = json.loads((output/'report.json').read_text())
            record['passed'] = sum(step['passed'] for step in record['steps'])
            record['checks'] = len(record['steps'])
except Exception as error:
    record['controller_error'] = f'{type(error).__name__}: {error}'
finally:
    if application is not None and application.poll() is None:
        os.killpg(application.pid, signal.SIGTERM)
        try:
            application.wait(timeout=10)
        except subprocess.TimeoutExpired:
            os.killpg(application.pid, signal.SIGKILL)
            application.wait(timeout=5)
    if process is not None and process.poll() is None:
        stop = subprocess.run([str(binary), 'ctl', 'daemon.stop', '--socket', str(peer/'control.sock')],
                              env=env, capture_output=True, text=True, timeout=10)
        record['peer_stop'] = dict(exit=stop.returncode, stdout=stop.stdout, stderr=stop.stderr)
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGTERM)
            process.wait(timeout=5)
    record['peer_exit'] = process.returncode if process is not None else None
    # Inspect only processes/listeners tied to this private run or our two ports.
    deadline = time.monotonic()+10
    while True:
        ps = subprocess.run(['ps', '-axo', 'pid=,ppid=,args='], capture_output=True, text=True, timeout=5)
        remaining = [line.strip() for line in ps.stdout.splitlines()
                     if str(root) in line and str(os.getpid()) != line.strip().split()[0]]
        if not remaining or time.monotonic() >= deadline:
            break
        time.sleep(0.1)
    record['remaining_private_processes'] = remaining
    record['port_postflight'] = []
    for port in (peer_port, host_port):
        check = subprocess.run(['/usr/sbin/lsof', '-nP', f'-iTCP:{port}', '-sTCP:LISTEN'],
                               capture_output=True, text=True, timeout=5)
        record['port_postflight'].append(dict(port=port, exit=check.returncode, stdout=check.stdout))
    for name in ('peer.log', 'app.log'):
        path = base/name
        if path.exists():
            record[name] = path.read_text()
    (base/'result.private.json').write_text(json.dumps(record, indent=2)+'\n')
    print(json.dumps(record, indent=2).replace(str(root), '<private-run>')
          .replace(str(Path.home()), '<test-home>'))
