"""Read-only call/return recorder. No callable replacement or result changes."""
import atexit
import json
import os
from pathlib import Path
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
REPO = '/Users/serge/.t3/worktrees/openergy/t3-8c51bae6'
SCRIPT = str(Path(sys.argv[0]).resolve()) if sys.argv else ''
RELEASE = REPO + '/tools/release/release.py'
SPARKLE = REPO + '/tools/release/sparkle.py'

if SCRIPT in (RELEASE, SPARKLE):
    stream = (ROOT / ('trace-%d.jsonl' % os.getpid())).open('x', buffering=1)

    def clean(value):
        if isinstance(value, Path):
            return str(value)
        if isinstance(value, bytes):
            return {'bytes_utf8': value.decode('utf-8', 'backslashreplace')}
        if isinstance(value, (tuple, list)):
            return [clean(x) for x in value]
        if isinstance(value, dict):
            return {str(k): clean(v) for k, v in value.items()}
        if isinstance(value, (str, int, float, bool)) or value is None:
            return value
        return str(value)

    def record(kind, **fields):
        stream.write(json.dumps(dict(kind=kind, monotonic=time.monotonic(), pid=os.getpid(),
                                    ppid=os.getppid(), pgid=os.getpgrp(), **clean(fields))) + '\n')

    def profile(frame, event, result):
        filename, name = frame.f_code.co_filename, frame.f_code.co_name
        local = frame.f_locals
        if filename in (RELEASE, SPARKLE):
            if name == 'run' and event == 'return':
                item = dict(script=filename, argv=local.get('args'),
                            capture=local.get('capture', True))
                completed = result if hasattr(result, 'returncode') else local.get('result')
                if completed is not None:
                    item.update(exit=completed.returncode, stdout=completed.stdout,
                                stderr=completed.stderr)
                parent = frame.f_back
                if parent and parent.f_code.co_name in ('cli', 'smoke'):
                    smoke = parent.f_back if parent.f_code.co_name == 'cli' else parent
                    item['smoke'] = {k: smoke.f_locals.get(k) for k in ('binary', 'base', 'config', 'socket')}
                    item['runtime_environment'] = smoke.f_locals.get('env')
                record('real-command-return', **item)
            elif name == 'smoke' and event == 'return':
                process = local.get('process')
                record('smoke-return', binary=local.get('binary'), base=local.get('base'),
                       socket=local.get('socket'), daemon_pid=getattr(process, 'pid', None),
                       daemon_exit=getattr(process, 'returncode', None), status=local.get('status'))
        if event == 'return' and name == '__init__' and filename.endswith('/subprocess.py'):
            process = local.get('self')
            if getattr(process, 'pid', None):
                args = local.get('args')
                if args and any(str(a) == 'daemon' for a in args):
                    record('daemon-started', argv=args, daemon_pid=process.pid,
                           environment=local.get('env'))
        if event == 'call' and name == 'cleanup' and filename.endswith('/tempfile.py'):
            directory = Path(local['self'].name)
            if directory.name.startswith(('oe-release-smoke-', 'oe-package-')):
                files = {}
                if directory.name.startswith('oe-release-smoke-'):
                    for name in ('config.toml', 'daemon.log'):
                        path = directory / name
                        if path.is_file():
                            files[name] = path.read_text(errors='backslashreplace')
                record('before-temp-cleanup', directory=directory, files=files)

    record('observer-start', argv=sys.argv, scope='sys.setprofile observation only; real functions unchanged')
    sys.setprofile(profile)
    atexit.register(lambda: (sys.setprofile(None), stream.close()))
