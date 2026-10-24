#!/usr/bin/env python3
"""Cargo RUSTC_WRAPPER: absolute priority at the actual rustc exec barrier."""
import json
import os
from pathlib import Path
import sys
import time

os.setpriority(os.PRIO_PROCESS, 0, 15)
with (Path(os.environ['STOP_COMPILER_RECEIPTS']) / f'{os.getpid()}.json').open('x') as file:
    json.dump(dict(pid=os.getpid(), ppid=os.getppid(), pgid=os.getpgrp(), uid=os.getuid(),
                   nice=os.getpriority(os.PRIO_PROCESS, 0), monotonic_ns=time.clock_gettime_ns(time.CLOCK_MONOTONIC),
                   argv=sys.argv[1:]), file, indent=2)
    file.write('\n')
os.execv(sys.argv[1], sys.argv[1:])
