"""macOS read-only process rows. libproc is loaded only by an explicit call."""
import ctypes as C
import errno
import os
from pathlib import Path


class BSD(C.Structure):
    # Local macOS 15 SDK sys/proc_info.h: proc_bsdinfo / PROC_PIDTBSDINFO=3.
    _fields_ = [(name, C.c_uint32) for name in
                ('flags', 'status', 'xstatus', 'pid', 'ppid', 'uid', 'gid', 'ruid',
                 'rgid', 'svuid', 'svgid', 'reserved')]
    _fields_ += [('comm', C.c_char * 16), ('name', C.c_char * 32)]
    _fields_ += [(name, C.c_uint32) for name in ('nfiles', 'pgid', 'jobc', 'tdev', 'tpgid')]
    _fields_ += [('nice', C.c_int32), ('start_sec', C.c_uint64), ('start_usec', C.c_uint64)]


def api():
    lib = C.CDLL('/usr/lib/libproc.dylib', use_errno=True)
    lib.proc_pidinfo.argtypes = [C.c_int, C.c_int, C.c_uint64, C.c_void_p, C.c_int]
    lib.proc_pidinfo.restype = C.c_int
    lib.proc_pidpath.argtypes = [C.c_int, C.c_void_p, C.c_uint32]
    lib.proc_pidpath.restype = C.c_int
    lib.proc_listallpids.argtypes = [C.c_void_p, C.c_int]
    lib.proc_listallpids.restype = C.c_int
    return lib


def bsd(lib, pid):
    value = BSD()
    C.set_errno(0)
    result = lib.proc_pidinfo(pid, 3, 0, C.byref(value), C.sizeof(value))
    if result != C.sizeof(value):
        code = C.get_errno()
        # Only ESRCH is absence. EPERM, zero errno and short reads are uncertainty.
        if result == 0 and code == errno.ESRCH:
            return None
        raise OSError(code, 'proc_pidinfo short/failed read', str(pid))
    return {key: getattr(value, key) for key in
            ('pid', 'ppid', 'pgid', 'uid', 'ruid', 'nice', 'status', 'start_sec', 'start_usec')}


def key(row):
    return f'{row["pid"]}-{row["start_sec"]}-{row["start_usec"]}'


def expected_starts(tokens):
    result = {}
    for token in tokens:
        pid, sec, usec = map(int, token.split('-'))
        result.setdefault(pid, set()).add(f'{pid}-{sec}-{usec}')
    return result


def argv(pid):
    libc = C.CDLL(None, use_errno=True)
    libc.sysctl.argtypes = [C.POINTER(C.c_int), C.c_uint, C.c_void_p,
                           C.POINTER(C.c_size_t), C.c_void_p, C.c_size_t]
    libc.sysctl.restype = C.c_int
    mib = (C.c_int * 3)(1, 49, pid)  # CTL_KERN, KERN_PROCARGS2, pid
    buffer = C.create_string_buffer(1048576)
    size = C.c_size_t(len(buffer))
    if libc.sysctl(mib, 3, buffer, C.byref(size), None, 0) != 0:
        raise OSError(C.get_errno(), 'KERN_PROCARGS2', str(pid))
    data = buffer.raw[:size.value]
    count = int.from_bytes(data[:4], 'little', signed=True)
    if not 1 <= count <= 128:
        raise RuntimeError('invalid argc')
    end = data.index(b'\0', 4) + 1  # executable path, then alignment NULs
    while end < len(data) and data[end] == 0:
        end += 1
    values = data[end:].split(b'\0')[:count]
    if len(values) != count:
        raise RuntimeError('short argv')
    return [value.decode('utf-8', errors='strict') for value in values]


def executable(lib, pid):
    value = C.create_string_buffer(4096)
    if lib.proc_pidpath(pid, value, len(value)) <= 0:
        raise OSError(C.get_errno(), 'proc_pidpath', str(pid))
    return str(Path(value.value.decode()).resolve())


def detail(lib, row):
    pid = row['pid']
    # Four bounded stage reads catch same-start wrapper -> product exec.
    # Never mutate row until BOTH stage and BSD identity checks succeed.
    try:
        exe_before = executable(lib, pid)
        args_before = argv(pid)
        exe_after = executable(lib, pid)
        args_after = argv(pid)
        after = bsd(lib, pid)
        coherent = (exe_before == exe_after and args_before == args_after and
                    str(Path(args_before[0]).resolve()) == exe_before)
    except Exception as error:
        # In particular, Unicode/sysctl errors must not expose staged argv bytes.
        raise RuntimeError('metadata-only detail read uncertain: ' + type(error).__name__) from None
    stable = ('pid', 'ppid', 'pgid', 'uid', 'ruid', 'start_sec', 'start_usec')
    if after is None or any(row[k] != after[k] for k in stable):
        raise RuntimeError('metadata-only race: process identity changed during detail read')
    if not coherent:
        raise RuntimeError('metadata-only pending mixed exec stage')
    return dict(row, executable=exe_before, argv=args_before, nice_after=after['nice'])


def snapshot(private, known, private_candidates=()):
    lib = api()
    known = expected_starts(known)
    private_candidates = expected_starts(private_candidates)
    tracked = known.keys() | private_candidates.keys()
    pids = (C.c_int * 65536)()
    count = lib.proc_listallpids(pids, C.sizeof(pids))
    if count <= 0 or count >= len(pids):
        raise RuntimeError('PID enumeration failed/truncated')
    rows, races = [], []
    for pid in pids[:count]:
        if pid <= 0:
            continue
        try:
            row = bsd(lib, pid)
            if row is None:
                races.append(dict(pid=pid, race='enumerated then ESRCH'))
                continue
            if any(pid in starts and key(row) not in starts[pid] for starts in (known, private_candidates)):
                row['identity_reused'] = True
                rows.append(row)  # Never collect a reused stranger's argv.
                continue
            # Do not capture strangers' argv. Group values are inventory only.
            candidate = (row['uid'] == os.getuid() or pid in tracked)
            if candidate:
                value = C.create_string_buffer(4096)
                size = lib.proc_pidpath(pid, value, len(value))
                path = str(Path(value.value.decode()).resolve()) if size > 0 else None
                if path and Path(path).is_relative_to(private):
                    # Classification only: keep no path/argv from an uncertain stage.
                    row['private_candidate'] = True
                if pid in known or row.get('private_candidate'):
                    try:
                        staged = detail(lib, row)
                        if pid not in known and not Path(staged['executable']).is_relative_to(private):
                            raise RuntimeError('metadata-only private candidate changed exec stage')
                        row = staged
                    except Exception as error:
                        row['read_race'] = repr(error)
            rows.append(row)
        except Exception as error:
            races.append(dict(pid=pid, error=repr(error), known=pid in tracked))
    # Query missing owned AND unresolved PIDs. Enumeration alone is not absence.
    absence = {}
    for pid in tracked:
        if any(r['pid'] == pid for r in rows):
            continue
        try:
            absent = bsd(lib, pid)
            if absent is None:
                absence[str(pid)] = 'ESRCH'
            else:
                if any(pid in starts and key(absent) not in starts[pid] for starts in (known, private_candidates)):
                    absent['identity_reused'] = True
                rows.append(absent)
        except Exception as error:
            races.append(dict(pid=pid, error=repr(error), known=True))
    return dict(rows=rows, races=races, absence=absence)


def targeted(known):
    """Recovery/control verification that does not depend on PID enumeration."""
    lib = api()
    known = expected_starts(known)
    rows, races, absence = [], [], {}
    for pid in sorted(set(known)):
        try:
            row = bsd(lib, pid)
            if row is None:
                absence[str(pid)] = 'ESRCH'
                continue
            if key(row) not in known[pid]:
                row['identity_reused'] = True
                rows.append(row)
                continue
            try:
                row = detail(lib, row)
            except Exception as error:
                row['read_race'] = repr(error)
            rows.append(row)
        except Exception as error:
            races.append(dict(pid=pid, error=repr(error), known=True))
    return dict(rows=rows, races=races, absence=absence, targeted=True)
