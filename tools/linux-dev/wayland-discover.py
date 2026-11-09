#!/usr/bin/env python3
"""Run M1 discovery against an owned Weston and a private, non-activating bus.

No host display, bus, input devices, clipboard, or daemon is used. Evidence is
kept in a new private directory; all owned process groups are removed on exit.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[2]
BUILD = ['cargo', 'build', '--release', '--locked', '-p',
         'openergy-platform-linux', '--example', 'wayland_probe']
WESTON = ['weston', '-B', 'headless', '--renderer=pixman', '--no-config',
          '--socket=wayland-probe-0', '--idle-time=0', '--width=1280', '--height=720']


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--probe', type=Path, help='use an already built example')
    parser.add_argument('--portal', type=Path, help='also introspect this real portal frontend (no sessions)')
    args = parser.parse_args()
    if args.probe:
        probe = args.probe.resolve()
    else:
        subprocess.run(BUILD, cwd=ROOT, check=True)
        target = Path(os.environ.get('CARGO_TARGET_DIR', ROOT / 'target'))
        probe = (target / 'release/examples/wayland_probe').resolve()
    evidence = Path(tempfile.mkdtemp(prefix='oe-wayland-evidence-'))
    lab = Path(tempfile.mkdtemp(prefix='oe-wayland-session-'))
    env = {'PATH': os.environ['PATH'], 'LANG': 'C.UTF-8',
           'HOME': str(lab / 'home'), 'XDG_RUNTIME_DIR': str(lab / 'runtime'),
           'XDG_CONFIG_HOME': str(lab / 'config'), 'XDG_DATA_HOME': str(lab / 'data'),
           'XDG_STATE_HOME': str(lab / 'state'), 'XDG_CACHE_HOME': str(lab / 'cache'),
           'XDG_DATA_DIRS': str(lab / 'data'), 'XDG_CONFIG_DIRS': str(lab / 'config'),
           'XDG_SESSION_TYPE': 'wayland', 'WAYLAND_DISPLAY': 'wayland-probe-0',
           'DBUS_SESSION_BUS_ADDRESS': f'unix:path={lab}/runtime/bus',
           'DBUS_SYSTEM_BUS_ADDRESS': f'unix:path={lab}/runtime/no-system-bus',
           'OPENERGY_WAYLAND_PROBE_ISOLATED': '1', 'OPENERGY_WAYLAND_PROBE_ROOT': str(lab)}
    for name in ['home', 'runtime', 'config', 'data', 'state', 'cache']:
        (lab / name).mkdir(mode=0o700)
    # No service directories: even accidental activation cannot launch a host portal.
    config = lab / 'bus.conf'
    config.write_text(f'''<busconfig>
<type>session</type><listen>unix:path={lab}/runtime/bus</listen>
<auth>EXTERNAL</auth><policy context="default">
<allow send_destination="*"/><allow receive_sender="*"/><allow own="*"/>
</policy></busconfig>
''')
    processes = []
    handles = []

    def start(command, log):
        handle = (evidence / log).open('w')
        handles.append(handle)
        process = subprocess.Popen(command, env=env, stdout=handle,
                                   stderr=subprocess.STDOUT, start_new_session=True)
        processes.append(process)
        return process

    def stop(process):
        # The group belongs to this Popen; children cannot escape cleanup on normal exit.
        try:
            os.killpg(process.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
        try:
            process.wait(timeout=3)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait(timeout=2)
        # Sweep surviving children (e.g. weston-desktop-shell) after the leader exits.
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        processes.remove(process)

    def ready(path, process):
        deadline = time.monotonic() + 5
        while not path.exists():
            if process.poll() is not None or time.monotonic() >= deadline:
                raise RuntimeError(f'{path.name} did not become ready')
            time.sleep(0.05)

    results = {}

    def run(label, child_env, expected):
        begin = time.monotonic()
        result = subprocess.run([str(probe), 'discover'], env=child_env,
                                capture_output=True, text=True, timeout=40)
        (evidence / f'{label}.json').write_text(result.stdout)
        (evidence / f'{label}.stderr').write_text(result.stderr)
        report = json.loads(result.stdout)
        results[label] = {'exit': result.returncode, 'seconds': round(time.monotonic() - begin, 3)}
        if (result.returncode == 0) != (expected == 0):
            raise RuntimeError(f'{label}: unexpected exit {result.returncode}')
        return report

    def fail_signal(signum, _frame):
        raise InterruptedError(f'lab interrupted by signal {signum}')

    signal.signal(signal.SIGTERM, fail_signal)
    signal.signal(signal.SIGINT, fail_signal)
    try:
        bus = start(['dbus-daemon', '--nofork', '--nopidfile', '--nosyslog',
                     f'--config-file={config}'], 'bus.log')
        ready(lab / 'runtime/bus', bus)
        headless = dict(env)
        headless.pop('WAYLAND_DISPLAY')
        report = run('no-wayland', headless, 1)
        if 'no Wayland session' not in report['errors'][0]:
            raise RuntimeError('no-Wayland case did not explain the missing session')
        compositor = start(WESTON, 'weston.log')
        ready(lab / 'runtime/wayland-probe-0', compositor)
        unguarded = dict(env)
        unguarded.pop('OPENERGY_WAYLAND_PROBE_ISOLATED')
        run('guard-refusal', unguarded, 1)
        wrong_bus = dict(env, DBUS_SESSION_BUS_ADDRESS='unix:path=/not-the-private-bus')
        run('bus-refusal', wrong_bus, 1)
        report = run('weston-discover', env, 0)
        outputs = report['wayland']['outputs']
        if not outputs or not any(o['logical_size'] for o in outputs):
            raise RuntimeError('Weston did not report a logical output')
        if report['portals']['status'] != 'portal_service_missing':
            raise RuntimeError('private bus must report the absent portal service')
        for role in ['server', 'client']:
            if report['capability_matrix'][role]['possible'] != 'unsupported':
                raise RuntimeError(f'{role} must remain unsupported without portals')
        if args.portal:
            portal = start([str(args.portal.resolve())], 'portal.log')
            deadline = time.monotonic() + 10
            while True:
                owner = subprocess.run([
                    'busctl', '--address', env['DBUS_SESSION_BUS_ADDRESS'],
                    '--auto-start=no', '--timeout=1', '--json=short', 'call',
                    'org.freedesktop.DBus', '/org/freedesktop/DBus',
                    'org.freedesktop.DBus', 'NameHasOwner', 's',
                    'org.freedesktop.portal.Desktop'], env=env, capture_output=True,
                    text=True, timeout=2, check=True)
                if json.loads(owner.stdout)['data'] == [True]:
                    break
                if portal.poll() is not None or time.monotonic() >= deadline:
                    raise RuntimeError('real portal frontend did not acquire its bus name')
                time.sleep(0.1)
            report = run('portal-frontend-discover', env, 0)
            if report['portals']['status'] != 'available':
                raise RuntimeError('real portal introspection failed')
            stop(portal)
        # Exercise an actual connected but unresponsive compositor, not a fake server.
        os.killpg(compositor.pid, signal.SIGSTOP)
        try:
            run('compositor-stalled', env, 1)
        finally:
            os.killpg(compositor.pid, signal.SIGCONT)
        stop(compositor)
        run('compositor-removed', env, 1)
        stop(bus)
        run('bus-removed', env, 1)
        print((evidence / 'weston-discover.json').read_text(), end='')
        print('PASS isolated Weston discovery, absent portal, refusals, stalled/removed compositor and cleanup')
    finally:
        for process in reversed(processes[:]):
            try:
                os.killpg(process.pid, signal.SIGCONT)
            except ProcessLookupError:
                pass
            stop(process)
        for handle in handles:
            handle.close()
        metadata = {
            'source_sha': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
            'dirty_worktree': bool(subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT)),
            'toolchain': subprocess.check_output(['rustc', '--version'], text=True).strip(),
            'compositor': subprocess.check_output(['weston', '--version'], text=True).strip(),
            'architecture': os.uname().machine,
            'os_release': Path('/etc/os-release').read_text(),
            'cargo': subprocess.check_output(['cargo', '--version'], text=True).strip(),
            'probe_sha256': hashlib.sha256(probe.read_bytes()).hexdigest(),
            'image_digest': None,
            'image_note': 'native host lane, no container image; see package versions',
            'packages': subprocess.check_output(['dpkg-query', '-W', 'weston', 'libweston-13-0',
                                                'dbus-daemon', 'systemd', 'coreutils']
                                               + (['xdg-desktop-portal'] if args.portal else []), text=True),
            'build_command': BUILD, 'compositor_command': WESTON,
            'probe_command': ['<probe>', 'discover'], 'results': results,
            'portal_command': [str(args.portal)] if args.portal else None,
            'mutter_kwin': 'not installed; GNOME/Plasma portal session not provisioned; not verified',
        }
        (evidence / 'environment.json').write_text(json.dumps(metadata, indent=2) + '\n')
        # Do not archive private socket paths, unique bus identifiers or hostnames.
        for path in evidence.iterdir():
            text = path.read_text().replace(str(lab), '<private-session>')
            text = text.replace(os.uname().nodename, '<host>')
            path.write_text(text)
        import shutil
        shutil.rmtree(lab)
        print(f'Evidence: {evidence}')


if __name__ == '__main__':
    main()
