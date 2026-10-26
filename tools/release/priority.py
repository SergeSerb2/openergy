#!/usr/bin/env python3
"""Run an owned Unix release command at absolute nice 15, including nested calls."""
import os
import sys


if len(sys.argv) < 3 or sys.argv[1] != "--":
    sys.exit("usage: priority.py -- COMMAND [ARG ...]")

try:
    if os.getpriority(os.PRIO_PROCESS, 0) != 15:
        os.setpriority(os.PRIO_PROCESS, 0, 15)
    if os.getpriority(os.PRIO_PROCESS, 0) != 15:
        raise OSError("effective nice is not 15")
    os.execvp(sys.argv[2], sys.argv[2:])
except OSError as error:
    sys.exit(f"release command could not start at nice 15: {error}")
