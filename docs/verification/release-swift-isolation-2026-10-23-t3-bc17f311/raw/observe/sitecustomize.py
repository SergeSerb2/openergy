"""Read-only call/return recorder. No callable replacement or result changes."""
import atexit
import ctypes
import hashlib
import stat
import json
import os
from pathlib import Path
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
REPO = '/Users/serge/.t3/worktrees/openergy/t3-bc17f311'
SCRIPT = str(Path(sys.argv[0]).resolve()) if sys.argv else ''
RELEASE = REPO + '/tools/release/release.py'
SPARKLE = REPO + '/tools/release/sparkle.py'

class BsdInfo(ctypes.Structure):
    _fields_ = [(name, ctypes.c_uint32) for name in
        ('flags', 'status', 'xstatus', 'pid', 'ppid', 'uid', 'gid', 'ruid', 'rgid', 'svuid', 'svgid', 'rfu')]
    _fields_ += [('comm', ctypes.c_char * 16), ('name', ctypes.c_char * 32)]
    _fields_ += [(name, ctypes.c_uint32) for name in ('nfiles', 'pgid', 'pjobc', 'tdev', 'tpgid')]
    _fields_ += [('nice', ctypes.c_int32), ('start_sec', ctypes.c_uint64), ('start_usec', ctypes.c_uint64)]


LIBPROC = ctypes.CDLL('/usr/lib/libproc.dylib', use_errno=True)
LIBPROC.proc_pidinfo.argtypes = [ctypes.c_int, ctypes.c_int, ctypes.c_uint64, ctypes.c_void_p, ctypes.c_int]


def identity(pid):
    data = BsdInfo()
    if LIBPROC.proc_pidinfo(pid, 3, 0, ctypes.byref(data), ctypes.sizeof(data)) != ctypes.sizeof(data):
        return None
    return dict(pid=data.pid, ppid=data.ppid, pgid=data.pgid, nice=data.nice,
                start_sec=data.start_sec, start_usec=data.start_usec,
                executable=data.name.decode(errors='replace') or data.comm.decode(errors='replace'))



def tree_inventory(base):
    result = {}
    for p in sorted(base.rglob('*')):
        m = p.lstat().st_mode
        if p.is_symlink():
            result[str(p.relative_to(base))] = dict(mode=stat.S_IMODE(m), link=os.readlink(p))
        elif p.is_file():
            with p.open('rb') as f:
                digest = hashlib.file_digest(f, 'sha256').hexdigest()
            result[str(p.relative_to(base))] = dict(mode=stat.S_IMODE(m), bytes=p.stat().st_size, sha256=digest)
    return result

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
                record('actual-subprocess-start', argv=args, child_pid=process.pid, identity=identity(process.pid))
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
                if directory.name.startswith('oe-package-'):
                    record('actual-payload-trees', staged=tree_inventory(directory / 'payload'), extracted=tree_inventory(directory / 'extracted'))

    record('observer-start', argv=sys.argv, scope='sys.setprofile observation only; real functions unchanged')
    sys.setprofile(profile)
    atexit.register(lambda: (sys.setprofile(None), stream.close()))
