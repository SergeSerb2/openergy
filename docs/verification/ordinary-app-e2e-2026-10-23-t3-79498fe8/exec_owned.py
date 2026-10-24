"""Same-PID exec receipt after absolute priority assignment. Not a fixture launcher."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

receipt = Path(sys.argv[1])
argv = sys.argv[2:]
os.setpriority(os.PRIO_PROCESS, 0, 15)
assert os.getpriority(os.PRIO_PROCESS, 0) == 15
os.umask(0o077)
assert Path(argv[0]).is_absolute()
value = dict(pid=os.getpid(), ppid=os.getppid(), pgid=os.getpgrp(), uid=os.getuid(), nice=15,
             monotonic_ns=time.clock_gettime_ns(time.CLOCK_MONOTONIC), argv=argv,
             binary_sha256=hashlib.sha256(Path(argv[0]).read_bytes()).hexdigest(),
             environment_allowlist=dict(os.environ),
             identity=subprocess.check_output(['/bin/ps', '-p', str(os.getpid()), '-o',
                 'pid,ppid,pgid,uid,ni,lstart,command'], text=True))
with receipt.open('x') as stream:
    json.dump(value, stream, indent=2, sort_keys=True)
    stream.write('\n')
os.execve(argv[0], argv, dict(os.environ))
