"""Real bundled CLI/IPC/TLS smoke; virtual peers only, no desktop operations."""
from pathlib import Path
import json
import os
import signal
import socket
import subprocess
import sys
import tempfile
import time

os.umask(0o077)
root = Path(sys.argv[1]).resolve()
assert str(root).startswith('/private/tmp/oe-candidate-20261103.')
assert root.stat().st_uid == os.getuid() and root.stat().st_mode & 0o077 == 0
base = Path(tempfile.mkdtemp(prefix='tls-smoke-', dir=root))
binary = root/'extracted/Openergy.app/Contents/Helpers/openergy'
env = {k: v for k, v in os.environ.items() if not k.startswith('OPENERGY_')
       and not any(w in k.upper() for w in ('TOKEN', 'SECRET', 'PASSWORD', 'KEY'))}
env.update(OPENERGY_DISABLE_DISCOVERY='1', OPENERGY_ALLOW_PORT_24800='0')
processes, logs, commands, checkpoints = {}, [], [], []
port = 25762
with socket.socket() as probe:
    probe.bind(('127.0.0.1', port))


def cli(node, *arguments):
    state = base/node
    private_env = dict(env, OPENERGY_CONFIG=str(state/'config.toml'), OPENERGY_CONFIG_DIR=str(state),
                       OPENERGY_TLS_DIR=str(state/'tls'), OPENERGY_SOCKET=str(state/'control.sock'))
    args = [str(binary), *arguments]
    r = subprocess.run(args, env=private_env, capture_output=True, text=True, timeout=8)
    commands.append(dict(arguments=args, exit=r.returncode, stdout=r.stdout, stderr=r.stderr))
    assert r.returncode == 0, commands[-1]
    return r.stdout


def ctl(node, method, params=None):
    return json.loads(cli(node, 'ctl', method, '--socket', str(base/node/'control.sock'),
                          '--json', json.dumps(params or {})))


def status(node):
    # Polls do not fill the report; retain only checked real status snapshots.
    r = subprocess.run([str(binary), 'status', '--socket', str(base/node/'control.sock')],
                       env=env, capture_output=True, text=True, timeout=5)
    return json.loads(r.stdout) if r.returncode == 0 else None


def wait(node, name, predicate, seconds=15):
    deadline = time.monotonic()+seconds
    while time.monotonic() < deadline:
        s = status(node)
        if s is not None and predicate(s):
            checkpoints.append(dict(name=name, node=node, status=s))
            return s
        assert processes[node].poll() is None
        time.sleep(0.1)
    raise RuntimeError(f'{name} deadline: {s}')


def normalized(fingerprint):
    return fingerprint.removeprefix('v2:sha256:').replace(':', '').lower()


result = dict(platform='virtual', tls_enabled=True, require_client_trust=True, port=port)
try:
    fingerprints = {}
    for node in ('server', 'client'):
        state = base/node
        state.mkdir(mode=0o700)
        (state/'config.toml').write_text(f'''version=1
role="{node}"
name="{node}"
[server]
listen="127.0.0.1:{port}"
port={port}
[client]
remote="127.0.0.1:{port}"
autoconnect=false
[tls]
enabled=true
require_client_trust=true
[[screen]]
name="server"
rect={{x=0,y=0,w=1920,h=1080}}
[[screen]]
name="client"
rect={{x=1920,y=0,w=1920,h=1080}}
''')
        fingerprints[node] = cli(node, 'fingerprint', '--dir', str(state), '--synergy').strip()
        log = (state/'daemon.log').open('w')
        logs.append(log)
        args = [str(binary), 'daemon', '--platform', 'virtual', '--foreground',
                '--config', str(state/'config.toml'), '--socket', str(state/'control.sock')]
        commands.append(dict(arguments=args, kind='spawn'))
        processes[node] = subprocess.Popen(args, env=env, stdout=log, stderr=subprocess.STDOUT,
                                           start_new_session=True)
        wait(node, 'private virtual daemon ready', lambda s: s['daemon']['platform'] == 'virtual'
             and s['daemon']['pid'] == processes[node].pid)
    ctl('client', 'connection.connect', dict(host='127.0.0.1', port=port))
    pending = wait('client', 'unknown server held before approval',
                   lambda s: len(s['pending_trust']) == 1 and not s['peers'])
    assert normalized(pending['pending_trust'][0]['fingerprint']) == normalized(fingerprints['server'])
    empty = ctl('client', 'virtual.recorded')
    checkpoints.append(dict(name='no virtual input before approval', recorded=empty))
    # Lifecycle release_all entries are safe housekeeping, not received input.
    assert all(entry.get('event') is None for entry in empty['events'])
    assert empty['stats']['pressed_keys'] == 0 and empty['stats']['pressed_buttons'] == 0
    ctl('client', 'trust.approve', dict(fingerprint=fingerprints['server']))
    pending = wait('server', 'unknown client held before mutual approval',
                   lambda s: len(s['pending_trust']) == 1 and not s['peers'])
    assert normalized(pending['pending_trust'][0]['fingerprint']) == normalized(fingerprints['client'])
    ctl('server', 'trust.approve', dict(fingerprint=fingerprints['client']))
    for node in ('server', 'client'):
        wait(node, 'mutually trusted TLS peer connected', lambda s: s['state'] == 'connected'
             and bool(s['peers']) and s['peers'][0]['tls']['enabled']
             and s['peers'][0]['tls']['trusted'])
    ctl('client', 'connection.connect', dict(host='127.0.0.1', port=port,
                                           fingerprint='v2:sha256:'+'0'*64))
    wait('client', 'wrong explicit pin refused', lambda s: not s['peers']
         and (s.get('last_error') or {}).get('code') == 'tls_pin_mismatch'
         and s['tls']['enabled'])
    ctl('client', 'connection.connect', dict(host='127.0.0.1', port=port,
                                           fingerprint=fingerprints['server']))
    wait('client', 'verified explicit pin connects', lambda s: s['state'] == 'connected' and bool(s['peers']))
    ctl('client', 'trust.revoke', dict(fingerprint=fingerprints['server']))
    wait('client', 'revocation disconnects and requires approval again',
         lambda s: not s['peers'] and len(s['pending_trust']) == 1)
    trust = ctl('client', 'trust.list')
    assert trust['servers'] == []
    checkpoints.append(dict(name='revoked server absent from trust store', trust=trust))
    result['passed'] = True
except Exception as error:
    result.update(passed=False, error=f'{type(error).__name__}: {error}')
finally:
    for node, process in processes.items():
        if process.poll() is None:
            try:
                ctl(node, 'daemon.stop')
                process.wait(timeout=10)
            except Exception:
                os.killpg(process.pid, signal.SIGTERM)
                process.wait(timeout=5)
    for log in logs:
        log.close()
    result.update(commands=commands, checkpoints=checkpoints,
                  process_exits={node: p.returncode for node, p in processes.items()})
    check = subprocess.run(['/usr/sbin/lsof', '-nP', f'-iTCP:{port}', '-sTCP:LISTEN'],
                           capture_output=True, text=True, timeout=5)
    result['port_postflight'] = dict(port=port, exit=check.returncode, stdout=check.stdout)
    raw = json.dumps(result, indent=2)
    (base/'result.private.json').write_text(raw+'\n')
    print(raw.replace(str(root), '<private-run>').replace(str(Path.home()), '<test-home>'))
