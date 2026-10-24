"""Evidence helpers only; no product callbacks or source changes."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
RAW = HERE / 'raw'
ROOT = '059f052f965bb942d104448b064a9d0e68b1fda1'
APP_SHA = '95b3891e49e479b7caec9a1e18d0e861d7aae57f1bd178b7455b4bc7920aab0c'
DAEMON_SHA = '23e3f9dbee1496b247a5b316811f22c9330b103f979907d1d4fadf2e57399deb'
SWIFT = Path('/private/tmp/oe-bootstrap-r2-build-yl3byo9v/swift')
OLD_APP = REPO / 'docs/verification/private-update-bootstrap-repair-2026-10-23-t3-54de37ad/raw'
OLD_DAEMON = Path('/Users/serge/.t3/worktrees/openergy/t3-b205d42e/docs/verification/daemon-stop-latency-2026-10-23-t3-b205d42e/raw/final-source-build')
PORTS = (65371, 65372)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def now():
    return time.clock_gettime_ns(time.CLOCK_MONOTONIC)


def save(path, value):
    with path.open('x') as stream:
        json.dump(value, stream, indent=2, sort_keys=True)
        stream.write('\n')


def nice15():
    os.setpriority(os.PRIO_PROCESS, 0, 15)
    assert os.getpriority(os.PRIO_PROCESS, 0) == 15
    os.umask(0o077)


def cmd(argv, env=None, timeout=20):
    result = subprocess.run(list(map(str, argv)), cwd=REPO, env=env,
                            capture_output=True, text=True, timeout=timeout)
    return dict(argv=list(map(str, argv)), code=result.returncode,
                stdout=result.stdout, stderr=result.stderr)


def tree(root):
    records = []
    for path in sorted(root.rglob('*')):
        item = dict(path=str(path.relative_to(root)), mode=oct(path.lstat().st_mode & 0o7777))
        if path.is_symlink():
            assert path.resolve().is_relative_to(root.resolve())
            item['link'] = os.readlink(path)
        elif path.is_file():
            item.update(bytes=path.stat().st_size, sha256=sha(path))
        elif path.is_dir():
            item['directory'] = True
        else:
            raise AssertionError(path)
        records.append(item)
    return records
