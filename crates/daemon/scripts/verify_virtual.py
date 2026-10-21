#!/usr/bin/env python3
"""Exercise real openergy processes through ctl and raw loopback sockets only."""
import argparse
import atexit
import base64
import json
import os
from pathlib import Path
import signal
import socket
import statistics
import struct
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[3]
BIN = ROOT / 'target/release/openergy'
NODES = []
RESULTS = []


def cleanup():
    for node in reversed(NODES):
        try: node.stop()
        except Exception: pass


atexit.register(cleanup)


def wait(fn, timeout=12, message='condition'):
    end = time.monotonic() + timeout
    last = None
    while time.monotonic() < end:
        try:
            last = fn()
            if last:
                return last
        except (RuntimeError, OSError, KeyError) as exc:
            last = str(exc)
        time.sleep(.03)
    raise RuntimeError(f'timeout: {message}; last={last}')


class Node:
    def __init__(self, base, name):
        self.dir = base / name
        self.dir.mkdir()
        self.sock = self.dir / 'ipc.sock'
        self.config = self.dir / 'config.toml'
        self.log = open(self.dir / 'stderr.log', 'ab')
        self.proc = None
        NODES.append(self)
        self.start()

    def start(self):
        env = dict(os.environ, OPENERGY_ALLOW_PORT_24800='0')
        self.proc = subprocess.Popen([str(BIN), 'daemon', '--platform', 'virtual', '--config', str(self.config), '--socket', str(self.sock), '--foreground'], stdout=self.log, stderr=self.log, env=env)
        wait(lambda: self.call('status.get'), message=f'{self.dir.name} startup')

    def call(self, method, params=None):
        p = subprocess.run([str(BIN), 'ctl', method, '--socket', str(self.sock), '--json', json.dumps(params or {}), '--raw'], text=True, capture_output=True, timeout=15)
        if p.returncode:
            raise RuntimeError(p.stdout.strip() or p.stderr.strip())
        return json.loads(p.stdout)

    def patch(self, patch):
        return self.call('config.patch', {'patch': patch})

    def status(self):
        return self.call('status.get')

    def ledger(self):
        return self.call('virtual.recorded')

    def inject(self, *events):
        return self.call('virtual.inject', {'events': list(events)})

    def stop(self):
        if self.proc and self.proc.poll() is None:
            self.proc.send_signal(signal.SIGCONT)
            try:
                self.call('daemon.stop')
                self.proc.wait(timeout=4)
            except Exception:
                self.proc.kill()
                self.proc.wait(timeout=4)


def case(name, fn):
    started = time.monotonic()
    try:
        detail = fn()
        RESULTS.append({'scenario': name, 'pass': True, 'seconds': round(time.monotonic()-started, 3), 'detail': detail})
        print(f'PASS {name}: {detail}', flush=True)
    except Exception as exc:
        RESULTS.append({'scenario': name, 'pass': False, 'seconds': round(time.monotonic()-started, 3), 'error': str(exc)})
        print(f'FAIL {name}: {exc}', flush=True)


def require(value, message):
    if not value:
        raise RuntimeError(message)
    return value


def events(node):
    return [row['event'] for row in node.ledger()['events'] if row.get('event') and row.get('detail', {}).get('kind') == 'sink']


def enter(server, client):
    server.inject({'move_abs': [1000, 540]})
    server.inject({'move_abs': [1920, 540]})
    wait(lambda: server.status()['active_screen'] == 'client', message='edge entry')
    wait(lambda: client.status()['active_screen'] == 'client', message='client entry')


def connected(server, client):
    wait(lambda: any(p['connected'] and p['name'] == 'client' for p in server.status()['peers']), message='server peer adoption')
    wait(lambda: client.status()['state'] == 'connected', message='client DSOP setup')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--out', type=Path)
    opts = parser.parse_args()
    base = opts.out or Path(tempfile.mkdtemp(prefix='oe-wp6-'))
    base.mkdir(exist_ok=True)
    print(f'ARTIFACTS {base}', flush=True)
    server = Node(base, 'server')
    client = Node(base, 'client')
    screens = [
        {'name': 'server', 'rect': {'x': 0, 'y': 0, 'w': 1920, 'h': 1080}},
        {'name': 'client', 'rect': {'x': 1920, 'y': 180, 'w': 1920, 'h': 720}},
    ]
    try:
        def setup():
            server.patch({'name': 'server', 'screen': screens, 'tls': {'enabled': False}, 'server': {'listen': '127.0.0.1:24980', 'port': 24980}, 'role': 'server'})
            client.call('virtual.set_screens', {'rects': [{'x': 0, 'y': 0, 'w': 1920, 'h': 720, 'scale': 1.0}]})
            client.patch({'name': 'client', 'tls': {'enabled': False}, 'client': {'remote': '127.0.0.1:24980'}, 'role': 'client'})
            connected(server, client)
            return 'two real virtual daemons completed DSOP setup'
        case('S1 connect', setup)

        def mapping():
            links = server.call('config.get')['derived_links']
            enter(server, client)
            cursor = client.status()['cursor']
            require(350 <= cursor['y'] <= 370, f'range mapping: {cursor}')
            return {'derived_links': links, 'client_cursor': cursor}
        case('S2 partial range mapping', mapping)

        def typing():
            before = len(events(client))
            server.inject({'key_down': {'key': 'Shift_L', 'modifiers': ['shift']}}, {'key_down': {'key': 'a', 'modifiers': ['shift']}}, {'key_repeat': {'key': 'a', 'modifiers': ['shift'], 'count': 2}}, {'key_up': 'a'}, {'key_up': 'Shift_L'}, {'wheel': [120, -240]})
            wait(lambda: len(events(client)) >= before + 6)
            tail = events(client)[-6:]
            require(tail[-1] == {'type': 'mouse_wheel', 'dx': 120, 'dy': -240}, f'wheel mismatch: {tail}')
            return [e['type'] for e in tail]
        case('S3 typing modifiers repeat wheel', typing)

        def gates():
            server.patch({'switch':{'delay_ms':150}})
            server.inject({'move_abs':[1920,540]})
            require(server.status()['active_screen']=='server','switch delay skipped')
            wait(lambda:server.status()['active_screen']=='client',message='switch delay timer')
            server.patch({'switch':{'delay_ms':0,'double_tap_ms':500}})
            server.inject({'move_abs':[1920,540]})
            require(server.status()['active_screen']=='server','first double tap switched')
            server.inject({'move_abs':[1800,540]})
            server.inject({'move_abs':[1920,540]})
            wait(lambda:server.status()['active_screen']=='client',message='second edge tap')
            server.patch({'switch':{'double_tap_ms':0,'dead_corners':['top_right'],'corner_size_px':64}})
            server.inject({'move_abs':[1920,0]})
            require(server.status()['active_screen']=='server','dead corner allowed switch')
            server.inject({'move_abs':[1920,540]})
            wait(lambda:server.status()['active_screen']=='client')
            server.patch({'switch':{'dead_corners':[],'corner_size_px':0},'options':{'switch_needs_control':True}})
            server.inject({'move_abs':[1920,540]})
            require(server.status()['active_screen']=='server','modifier gate ignored')
            server.inject({'key_down':{'key':'Control_L','modifiers':['control']}},{'move_abs':[1920,540]})
            wait(lambda:server.status()['active_screen']=='client')
            server.inject({'key_up':'Control_L'})
            server.patch({'options':{'switch_needs_control':False},'input':{'relative_mouse':True}})
            enter(server,client)
            before=len(events(client))
            server.inject({'key_down':'Scroll_Lock'},{'key_up':'Scroll_Lock'},{'move_rel':[8,4]})
            wait(lambda:len(events(client))>=before+3)
            require(events(client)[-1]['type']=='mouse_move_rel','locked relative forwarding missing')
            server.patch({'input':{'relative_mouse':False}})
            return 'delay timer, retreat plus second tap, dead corner, modifier gate, and locked relative moves'
        case('S2 switch gates',gates)


        def crash_press(mouse=False):
            enter(server, client)
            server.inject({'mouse_down': 1} if mouse else {'key_down': 'a'})
            field = 'pressed_buttons' if mouse else 'pressed_keys'
            wait(lambda: client.ledger()['stats'][field] > 0)
            server.proc.kill()
            server.proc.wait(timeout=3)
            wait(lambda: client.ledger()['stats']['pressed_keys'] == 0 and client.ledger()['stats']['pressed_buttons'] == 0)
            server.start()
            connected(server, client)
            return 'SIGKILL peer -> release_all and empty pressed ledger; reconnected'
        case('S4 kill mid-keypress', lambda: crash_press(False))
        case('S5 kill mid-drag', lambda: crash_press(True))

        def clipboard_text():
            text = 'Openergy bidirectional λ'
            server.call('virtual.clipboard', {'text': text})
            wait(lambda: client.call('virtual.clipboard')['data']['formats'] == [{'kind': 'text_utf8', 'bytes_base64': base64.b64encode(text.encode()).decode()}])
            text2 = 'Client return λ'
            client.call('virtual.clipboard', {'text': text2})
            wait(lambda: server.call('virtual.clipboard')['data']['formats'] == [{'kind': 'text_utf8', 'bytes_base64': base64.b64encode(text2.encode()).decode()}])
            stamp = server.call('virtual.clipboard')['stamp']
            time.sleep(.7)
            require(server.call('virtual.clipboard')['stamp'] == stamp, 'clipboard loop detected')
            return 'text in both directions; own-write stamp stable for 700 ms'
        case('S8 clipboard both ways and own-write loop', clipboard_text)

        def bulk():
            enter(server, client)
            before = len(events(client))
            server.call('virtual.clipboard', {'dib_bytes': 8*1024*1024, 'summary': True})
            time.sleep(.3)
            require(client.call('virtual.clipboard', {'summary': True})['bytes'] != 8*1024*1024, 'transfer already completed before input')
            server.inject({'key_down': 'b'}, {'key_up': 'b'})
            wait(lambda: len(events(client)) >= before+2)
            require(client.call('virtual.clipboard', {'summary': True})['bytes'] != 8*1024*1024, 'typing completed after clipboard')
            result = wait(lambda: (r if (r := client.call('virtual.clipboard', {'summary': True}))['bytes'] == 8*1024*1024 else None), timeout=15)
            return {'typing_before_complete': True, 'clipboard': result}
        case('S9 8 MiB DIB with typing', bulk)

        def cap():
            client.patch({'clipboard': {'max_bytes': 4*1024*1024}})
            time.sleep(.2)
            server.call('virtual.clipboard', {'dib_bytes': 5*1024*1024, 'summary': True})
            result = wait(lambda: server.status()['clipboard']['last_blocked'])
            require(result['limit'] == 4*1024*1024, f'wrong peer cap: {result}')
            client.patch({'clipboard': {'max_bytes': 64*1024*1024}})
            return result
        case('S10 peer cap visible block', cap)

        def heartbeat():
            enter(server, client)
            server.inject({'key_down': 'a'})
            wait(lambda: client.ledger()['stats']['pressed_keys'] > 0)
            server.proc.send_signal(signal.SIGSTOP)
            try:
                wait(lambda: client.ledger()['stats']['pressed_keys'] == 0 and client.status()['state'] == 'reconnecting', timeout=12, message='heartbeat death')
            finally:
                server.proc.send_signal(signal.SIGCONT)
            connected(server, client)
            return 'SIGSTOP server -> client heartbeat timeout and release within 3 intervals'
        case('S11 heartbeat death', heartbeat)

        def display():
            client.call('virtual.set_screens', {'rects': [{'x': 0, 'y': 0, 'w': 1600, 'h': 900, 'scale': 1.0}]})
            result = wait(lambda: next((p['screen'] for p in server.status()['peers'] if p['name'] == 'client' and p['screen']['w'] == 1600), None))
            return result
        case('S12 display DINF update', display)

        def hostile():
            def recv_frame(sock):
                hdr = sock.recv(4)
                require(len(hdr) == 4, 'missing frame')
                n = struct.unpack('!I', hdr)[0]
                payload = b''
                while len(payload) < n:
                    payload += sock.recv(n-len(payload))
                return payload
            sock = socket.create_connection(('127.0.0.1', 24980), timeout=2)
            require(recv_frame(sock).startswith(b'Synergy'), 'wrong server hello')
            sock.sendall(struct.pack('!I', 7)+b'ZZZZabc')
            name = b'raw-unknown-name'
            payload = b'Synergy'+struct.pack('!HHI', 1, 8, len(name))+name
            sock.sendall(struct.pack('!I', len(payload))+payload)
            require(recv_frame(sock) == b'QINF', 'unknown code disconnected socket')
            sock.close()
            sock = socket.create_connection(('127.0.0.1', 24980), timeout=2)
            recv_frame(sock)
            sock.sendall(struct.pack('!I', 4*1024*1024+1))
            require(sock.recv(1) == b'', 'oversized frame stayed open')
            sock.close()
            require(server.proc.poll() is None, 'daemon crashed')
            return 'unknown frame skipped before QINF; oversized prefix closed only its connection'
        case('S13 hostile framing', hostile)

        def idle():
            time.sleep(.3)
            rss = client.status()['stats']['rss_bytes']
            require(rss is not None and rss < 30*1024*1024, f'RSS {rss}')
            return {'client_rss_bytes': rss, 'server_rss_bytes': server.status()['stats']['rss_bytes']}
        case('S15 idle RSS', idle)

        def permission():
            enter(server, client)
            server.inject({'mouse_down': 1}, {'key_down': 'a'})
            wait(lambda: client.ledger()['stats']['pressed_buttons'] > 0)
            client.call('virtual.set_permissions', {'accessibility': 'not_required', 'input_monitoring': 'not_required', 'post_events': 'denied'})
            wait(lambda: client.ledger()['stats']['pressed_buttons'] == 0 and client.ledger()['stats']['pressed_keys'] == 0)
            client.call('virtual.set_permissions', {'accessibility': 'not_required', 'input_monitoring': 'not_required', 'post_events': 'not_required'})
            connected(server, client)
            return 'revoked post_events -> released key and mouse; connection recovered'
        case('S16 permission revocation', permission)

        def numbering():
            enter(server, client)
            server.inject({'mouse_down': 1}, {'move_rel': [2, 0]}, {'mouse_up': 1}, {'mouse_down': 1}, {'mouse_up': 1})
            wait(lambda: len([r for r in client.ledger()['events'] if (r.get('event') or {}).get('type') == 'mouse_up']) >= 2)
            rows = [r for r in client.ledger()['events'] if r.get('event') and r['event']['type'] in ('mouse_down', 'mouse_up')][-4:]
            numbers = [r['click_number'] for r in rows]
            require(numbers[0] == numbers[1] and numbers[2] == numbers[3] and numbers[2] > numbers[0] > 100, f'numbering: {numbers}')
            return {'numbers': numbers, 'click_states': [r['click_state'] for r in rows]}
        case('S18 click generations', numbering)

        def patch_roundtrip():
            result = server.patch({'server': {'port': 24982}, 'role': 'server'})
            require(result['config']['server']['listen'] == '127.0.0.1:24982', 'listen/port mismatch')
            wait(lambda: server.status()['server']['listen'] == '127.0.0.1:24982')
            before = server.config.read_bytes()
            try:
                server.patch({'server': {'port': 70000}})
                raise RuntimeError('invalid patch accepted')
            except RuntimeError as exc:
                require('invalid_config' in str(exc), str(exc))
            require(server.config.read_bytes() == before, 'invalid patch wrote config')
            server.patch({'server': {'port': 24980}})
            return 'port patch changes listener and persisted TOML; invalid patch leaves file identical'
        case('S19 live config round trip', patch_roundtrip)
    finally:
        for node in reversed(NODES):
            node.stop()
        (base / 'report.json').write_text(json.dumps(RESULTS, indent=2))
        print(f'REPORT {base / "report.json"}', flush=True)
    return 0 if RESULTS and all(r['pass'] for r in RESULTS) else 1


if __name__ == '__main__':
    raise SystemExit(main())
