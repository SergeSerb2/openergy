#!/usr/bin/env python3
"""Measure the actual isolated X11 clipboard probe, excluding Cargo's RSS."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--probe", required=True, type=Path)
parser.add_argument("--source", required=True)
parser.add_argument("--report", required=True, type=Path)
args = parser.parse_args()
if sys.platform != "linux" or os.environ.get("DISPLAY") != ":99" or os.environ.get("OPENERGY_X11_PROBE_ISOLATED") != "1":
    parser.error("requires the isolated Linux/Xvfb container")
probe = args.probe.resolve(strict=True)
start = time.monotonic()
child = subprocess.Popen([str(probe), "r5-clipboard-expansion"])
# wait4 reports this child's usage. getrusage(RUSAGE_CHILDREN) can include a
# compiler that ran before a shell exec'ed this Python process.
_, status, usage = os.wait4(child.pid, 0)
child.returncode = os.waitstatus_to_exitcode(status)
report = {"scenario": "R5-x11-clipboard-expansion", "source": args.source,
          "binary_sha256": hashlib.sha256(probe.read_bytes()).hexdigest(),
          "peak_rss_bytes": usage.ru_maxrss * 1024,
          "elapsed_s": round(time.monotonic() - start, 6),
          "exit_code": child.returncode, "passed": child.returncode == 0}
args.report.write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps(report), flush=True)
raise SystemExit(child.returncode)
