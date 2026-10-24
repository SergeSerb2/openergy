"""Private evidence I/O. Importing this module does not inspect or start processes."""
import contextlib
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import time

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
ORIGINAL = '498190feb00f51039f500ae618ac7de5fb113d86'
ORIGINAL_DIR = 'docs/verification/ordinary-app-e2e-2026-10-23-t3-79498fe8/'
SOURCE = '059f052f965bb942d104448b064a9d0e68b1fda1'
PYTHON = '/opt/homebrew/bin/python3'
PORTS = (65431, 65432)  # New literals. No fallback or retry.
LOG_CAP = 8388608
EVIDENCE_CAP = 33554432
FLOW_SECONDS, CLEANUP_SECONDS = 120, 60


class DeadlineError(RuntimeError):
    pass


def remaining(deadline):
    value = deadline - time.monotonic()
    if value <= 0:
        raise DeadlineError('absolute deadline expired')
    return value


@contextlib.contextmanager
def io_until(deadline):
    """Main-thread, nestable absolute deadline, including file and pipe I/O."""
    duration = remaining(deadline)
    previous_handler = signal.getsignal(signal.SIGALRM)
    previous_timer = signal.getitimer(signal.ITIMER_REAL)
    entered = time.monotonic()
    def expired(_sig, _frame):
        raise DeadlineError('I/O absolute deadline expired')
    signal.signal(signal.SIGALRM, expired)
    if previous_timer[0]:
        duration = min(duration, previous_timer[0])
    signal.setitimer(signal.ITIMER_REAL, duration)
    try:
        yield
        remaining(deadline)
    finally:
        signal.setitimer(signal.ITIMER_REAL, 0)
        signal.signal(signal.SIGALRM, previous_handler)
        if previous_timer[0]:
            signal.setitimer(signal.ITIMER_REAL,
                            max(0.000001, previous_timer[0] - (time.monotonic() - entered)),
                            previous_timer[1])


def read(path, deadline, cap=LOG_CAP):
    with io_until(deadline):
        with Path(path).open('rb') as stream:
            value = stream.read(cap + 1)
        if len(value) > cap:
            raise RuntimeError('file byte cap exceeded: ' + str(path))
        return value


def load(path, deadline):
    return json.loads(read(path, deadline))


def exists(path, deadline):
    with io_until(deadline):
        return Path(path).exists()


def save(path, value, deadline):
    """Exclusive, atomic publication; a partial write cannot become an identity."""
    path = Path(path)
    temporary = path.with_name(path.name + '.writing-' + str(os.getpid()) + '-' + str(time.monotonic_ns()))
    data = (json.dumps(value, sort_keys=True, separators=(',', ':')) + '\n').encode()
    with io_until(deadline):
        if len(data) > LOG_CAP:
            raise RuntimeError('evidence record byte cap exceeded')
        if sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file()) + len(data) > EVIDENCE_CAP:
            raise RuntimeError('aggregate evidence cap exceeded')
        with temporary.open('xb') as stream:
            stream.write(data)
            stream.flush()
        try:
            os.link(temporary, path)  # Exclusive. Never replace a first failure/guard.
        finally:
            temporary.unlink()


def first(path, value, deadline):
    try:
        save(path, value, deadline)
    except FileExistsError:
        if not exists(path, deadline):
            raise


def append(path, value, deadline):
    data = (json.dumps(value, sort_keys=True, separators=(',', ':')) + '\n').encode()
    with io_until(deadline):
        if Path(path).exists() and Path(path).stat().st_size + len(data) > LOG_CAP:
            raise RuntimeError('evidence stream byte cap exceeded')
        if sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file()) + len(data) > EVIDENCE_CAP:
            raise RuntimeError('aggregate evidence cap exceeded')
        with Path(path).open('ab', buffering=0) as stream:
            if stream.write(data) != len(data):
                raise RuntimeError('short evidence write')


def sha(path, deadline):
    digest = hashlib.sha256()
    with io_until(deadline):
        with Path(path).open('rb') as stream:
            while chunk := stream.read(262144):
                remaining(deadline)
                digest.update(chunk)
    return digest.hexdigest()


def tree(root, deadline):
    records = []
    with io_until(deadline):
        root = Path(root).resolve()
        for path in sorted(root.rglob('*')):
            remaining(deadline)
            item = dict(path=str(path.relative_to(root)), mode=oct(path.lstat().st_mode & 0o7777))
            if path.is_symlink():
                if not path.resolve().is_relative_to(root):
                    raise RuntimeError('bundle symlink escape')
                item['link'] = os.readlink(path)
            elif path.is_file():
                item.update(bytes=path.stat().st_size, sha256=sha(path, deadline))
            elif path.is_dir():
                item['directory'] = True
            else:
                raise RuntimeError('unexpected bundle node')
            records.append(item)
    return records


def command(argv, deadline, cap=4194304):
    """Read-only tools/probes only. Products always use exec_owned.py + a Popen ledger."""
    with io_until(deadline):
        process = subprocess.Popen(list(map(str, argv)), cwd=REPO,
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            stdout, stderr = process.communicate(timeout=remaining(deadline) * 0.8)
        except BaseException:
            # An unreaped direct tool child cannot have its PID reused.
            if process.poll() is None:
                process.kill()
            # Reserve part of the same absolute budget for killing/reaping our
            # unreaped probe/tool child; never borrow from product cleanup.
            process.communicate(timeout=remaining(deadline))
            raise
        if len(stdout) + len(stderr) > cap:
            raise RuntimeError('tool output byte cap exceeded')
        if process.returncode != 0:
            raise RuntimeError(repr(dict(argv=list(map(str, argv)), code=process.returncode,
                                         stdout=stdout.decode(errors='replace'),
                                         stderr=stderr.decode(errors='replace'))))
        return stdout


def git_file(path, deadline):
    return command(['git', 'show', ORIGINAL + ':' + ORIGINAL_DIR + path], deadline)


def nice15():
    os.setpriority(os.PRIO_PROCESS, 0, 15)
    if os.getpriority(os.PRIO_PROCESS, 0) != 15:
        raise RuntimeError('absolute NI15 assignment failed')
    os.umask(0o077)


def pause(deadline, seconds=0.02):
    time.sleep(min(seconds, remaining(deadline)))
