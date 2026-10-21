#!/usr/bin/env python3
"""Drive wire_probe against real Docker peers, in both roles, on WP1 ports.

No host input APIs, raw-wire substitutes, or imported Rust internals are used.
Run from any directory: python3 crates/protocol/examples/interop_probe.py
The caller builds wire_probe first. Every exit stops this lab and only probes
started by this process. Peer logs and TLS keys stay in the lab's ignored state.
"""
import collections
import os
from pathlib import Path
import re
import subprocess
import tempfile
import time


ROOT = Path(__file__).resolve().parents[3]
LAB = ROOT / "tools/interop-lab/lab.sh"
PROBE = ROOT / "target/release/examples/wire_probe"
ENV = {**os.environ, "LAB_NAME": "oe-lab-wp1", "LAB_PORT_RANGE": "24910-24919"}
CHILDREN = []


def lab(*args):
    return subprocess.run(
        [str(LAB), *args], env=ENV, check=True, capture_output=True,
        text=True, timeout=15,
    ).stdout


def start(log, *args):
    handle = log.open("w")
    process = subprocess.Popen([str(PROBE), *args], stdout=handle, stderr=subprocess.STDOUT, env=ENV)
    handle.close()
    CHILDREN.append(process)
    return process


def wait_for(process, log, marker, timeout=8):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if marker in log.read_text():
            return
        if process.poll() is not None:
            raise RuntimeError(f"probe exited before {marker}:\n{log.read_text()}")
        time.sleep(0.05)
    raise RuntimeError(f"probe deadline for {marker}:\n{log.read_text()}")


def finish(process, log):
    if process.wait(timeout=25) != 0:
        raise RuntimeError(log.read_text())
    for line in log.read_text().splitlines():
        if " PASS" in line:
            print(line, flush=True)


def receiving(peer, port, directory):
    print(f"wire_probe client <- {peer} server ({port})", flush=True)
    print(lab("server", peer, "--port", str(port), "--screens", "lab,openergy-mac",
              "--position", "right", "--tls", "off").strip(), flush=True)
    log = directory / f"{peer}-server.log"
    process = start(log, "client", "--server", f"127.0.0.1:{port}",
                    "--name", "openergy-mac", "--duration-secs", "18")
    wait_for(process, log, "RX DSOP")
    lab("clipboard", "set", "WP1 initial clipboard")
    lab("drive", "mousemove", "500", "500", "sleep", ".5", "mousemove", "1919", "540",
        "sleep", ".2", "mousemove_relative", "--", "40", "0")
    wait_for(process, log, "RX CINN")
    lab("drive", "mousemove_relative", "--", "80", "20")
    lab("drive", "click", "1")
    lab("drive", "type", "--clearmodifiers", "abc")
    lab("drive", "click", "4")
    lab("drive", "mousemove_relative", "--", "-2500", "0")
    wait_for(process, log, "RX COUT")
    # The probe announced virtual ownership on entry. The primary now regains
    # ownership when its local clipboard changes: this exercises real CCLP.
    lab("clipboard", "set", "WP1 regain local clipboard")
    lab("drive", "sleep", ".5", "mousemove", "1919", "540", "sleep", ".2",
        "mousemove_relative", "--", "40", "0")
    wait_for(process, log, "RX CCLP")
    lab("drive", "mousemove_relative", "--", "-2500", "0")
    finish(process, log)
    lab("stop", peer)


def sending(peer, port, directory):
    print(f"wire_probe server -> {peer} client ({port})", flush=True)
    log = directory / f"{peer}-client.log"
    args = ["server", "--port", str(port), "--hold-ms", "8000"]
    if peer == "barrier":
        args += ["--product", "Barrier", "--minor", "6"]
    process = start(log, *args)
    wait_for(process, log, "listening")
    lab("events", "start")
    print(lab("client", peer, "--server", f"host.docker.internal:{port}",
              "--name", "lab", "--tls", "off").strip(), flush=True)
    wait_for(process, log, "input sent")
    # Query X11 through the lab's own real interfaces during the input hold.
    lab("drive", "sleep", ".3")
    pointer = dict(re.findall(r"^(\w+)=(\d+)$", lab("pointer"), re.M))
    if pointer.get("X") != "321" or pointer.get("Y") != "234":
        raise RuntimeError(f"wrong injected pointer: {pointer}")
    clipboard = lab("clipboard", "get")
    if clipboard != "Openergy wire probe clipboard":
        raise RuntimeError("injected clipboard byte comparison failed")
    print(f"X11 pointer=321,234; clipboard exact comparison PASS bytes={len(clipboard.encode())}", flush=True)
    finish(process, log)
    lab("events", "stop")
    events = lab("events", "dump")
    counts = collections.Counter(re.findall(r"EVENT type \d+ \(([^)]+)\)", events))
    expected = {"RawButtonPress": 2, "RawButtonRelease": 2, "RawKeyPress": 1, "RawKeyRelease": 1}
    for event, count in expected.items():
        if counts[event] != count:
            raise RuntimeError(f"wrong XInput count {event}: {dict(counts)}")
    raw = re.findall(
        r"EVENT type \d+ \((Raw(?:Button|Key)(?:Press|Release))\)(.*?)(?=EVENT type|\Z)",
        events, re.S,
    )
    # The pinned lab starts Xvfb with its default US XKB map: q is keycode 24.
    # Check that the logical key became that physical key, without logging it.
    expected_order = [
        ("RawButtonPress", 1), ("RawButtonRelease", 1),
        ("RawKeyPress", 24), ("RawKeyRelease", 24),
        ("RawButtonPress", 4), ("RawButtonRelease", 4),
    ]
    actual_order = []
    for event, body in raw:
        detail = re.search(r"detail:\s*(\d+)", body)
        if detail is None:
            raise RuntimeError("missing raw event detail")
        actual_order.append((event, int(detail.group(1))))
    if actual_order != expected_order:
        raise RuntimeError("injected click/key/wheel detail or order differs")
    print(f"XInput exact raw event counts PASS {expected}; raw motion={counts['RawMotion']}", flush=True)
    print("XInput exact click/key/wheel detail and order PASS (key contents not logged)", flush=True)
    lab("stop", peer)


def main():
    try:
        print(lab("up").strip(), flush=True)
        with tempfile.TemporaryDirectory(prefix="oe-wp1-") as temp:
            directory = Path(temp)
            for peer, remote, local in [("deskflow", 24910, 24960), ("barrier", 24911, 24961)]:
                receiving(peer, remote, directory)
                sending(peer, local, directory)
        print("Deskflow 1.27 / Barrier 2.4, both roles PASS", flush=True)
    finally:
        for process in CHILDREN:
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=5)
        print(lab("down").strip(), flush=True)
        print(lab("status").strip(), flush=True)


if __name__ == "__main__":
    main()
