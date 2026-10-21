#!/usr/bin/env python3
"""Verify SIGKILL cleanup through the real runner, daemon, IPC, and Docker."""

import argparse
import json
import os
from pathlib import Path
import socket
import struct
import subprocess
import tempfile
import time

SCENARIO = '''# Requirement: Runner SIGKILL tears down its owned daemon and lab.
name = "guardian-interrupt"
requirement = "SIGKILL cleanup"
timeout_ms = 5000
[[node]]
id = "lab"
kind = "lab"
role = "server"
peer = "deskflow"
screens = "lab,probe"
[[node]]
id = "local"
kind = "openergy"
role = "off"
[[step]]
do = "sleep"
ms = 30000
'''


def read_exact(stream, length):
    data = bytearray()
    while len(data) < length:
        part = stream.recv(length - len(data))
        if not part:
            raise OSError("IPC closed inside a frame")
        data.extend(part)
    return data


def status_pid(path):
    with socket.socket(socket.AF_UNIX) as stream:
        stream.settimeout(0.2)
        stream.connect(str(path))
        request = json.dumps({"id": 1, "method": "status.get", "params": {}}).encode()
        stream.sendall(struct.pack(">I", len(request)) + request)
        length = struct.unpack(">I", read_exact(stream, 4))[0]
        if length > 16 * 1024 * 1024:
            raise OSError("IPC frame exceeded cap")
        return json.loads(read_exact(stream, length))["result"]["daemon"]["pid"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runner", default="target/release/openergy-e2e")
    parser.add_argument("--port-range", default="24998-24999")
    parser.add_argument("--lab-port-range", default="24938-24939")
    args = parser.parse_args()
    directory = Path(tempfile.mkdtemp(prefix="oe7-teardown-"))
    (directory / "guardian.toml").write_text(SCENARIO)
    command = [
        str(Path(args.runner).resolve()), "run", "--scenario-dir", str(directory),
        "--port-range", args.port_range, "--lab-port-range", args.lab_port_range, "--keep",
    ]
    with (directory / "runner.out").open("w") as output:
        child = subprocess.Popen(command, stdout=output, stderr=output)
        try:
            deadline = time.monotonic() + 20
            daemon_pid = None
            while time.monotonic() < deadline:
                sockets = list(Path("/tmp").glob(f"oe7-{child.pid}-*/s0/local/ctl.sock"))
                for path in sorted(sockets, key=lambda p: p.stat().st_mtime, reverse=True):
                    try:
                        daemon_pid = status_pid(path)
                        break
                    except (OSError, ValueError, KeyError):
                        continue
                if daemon_pid:
                    break
                if child.poll() is not None:
                    raise SystemExit(f"runner exited before readiness; artifacts: {directory}")
                time.sleep(0.025)
            if not daemon_pid:
                raise SystemExit(f"readiness timed out; artifacts: {directory}")
            print("Real daemon and Docker lab ready; sending SIGKILL to owned runner", child.pid)
            child.kill()
            child.wait(timeout=5)
            name = f"oe-lab-wp7-{child.pid}-s0-lab"
            deadline = time.monotonic() + 12
            while time.monotonic() < deadline:
                try:
                    os.kill(daemon_pid, 0)
                    alive = True
                except ProcessLookupError:
                    alive = False
                exists = subprocess.run(
                    ["docker", "inspect", name], stdout=subprocess.DEVNULL,
                    stderr=subprocess.DEVNULL, timeout=2,
                ).returncode == 0
                if not alive and not exists:
                    print("PASS: daemon gone; owned lab stopped and removed after runner SIGKILL")
                    return
                time.sleep(0.05)
            raise SystemExit(f"FAIL cleanup: daemon_alive={alive}, container_exists={exists}")
        finally:
            if child.poll() is None:
                child.kill()
                child.wait(timeout=5)


if __name__ == "__main__":
    main()
