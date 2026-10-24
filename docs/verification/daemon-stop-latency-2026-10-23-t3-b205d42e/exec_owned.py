#!/usr/bin/env python3
"""Set absolute NI 15, record an exec barrier, then keep the same PID at exec."""
import json
import os
from pathlib import Path
import subprocess
import sys
import time

os.setpriority(os.PRIO_PROCESS, 0, 15)
record = dict(pid=os.getpid(), ppid=os.getppid(), pgid=os.getpgrp(), uid=os.getuid(),
              nice=os.getpriority(os.PRIO_PROCESS, 0), monotonic_ns=time.clock_gettime_ns(time.CLOCK_MONOTONIC),
              argv=sys.argv[2:], ps=subprocess.check_output(
                  ['/bin/ps', '-p', str(os.getpid()), '-o', 'pid,ppid,pgid,uid,ni,lstart,command'],
                  text=True))
with Path(sys.argv[1]).open('x') as file:
    json.dump(record, file, indent=2)
    file.write('\n')
assert record['nice'] == 15
os.execv(sys.argv[2], sys.argv[2:])
