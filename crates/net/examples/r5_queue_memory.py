#!/usr/bin/env python3
"""R5: exercise a real virtual client against a non-reading TCP host.

The 64 MiB RSS-growth gate is a local stress regression threshold, not a
whole-daemon memory guarantee. No platform input or clipboard API is used.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import socket
import struct
import subprocess
import tempfile
import threading
import time


def frame(payload):
    return struct.pack("!I", len(payload)) + payload


def exact(stream, length):
    result = bytearray()
    while len(result) < length:
        part = stream.recv(length - len(result))
        if not part:
            raise EOFError("peer closed")
        result.extend(part)
    return bytes(result)


def receive(stream):
    length = struct.unpack("!I", exact(stream, 4))[0]
    if length > 16 * 1024 * 1024:
        raise RuntimeError("unexpected oversized response")
    return exact(stream, length)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--openergy", required=True, type=Path)
    parser.add_argument("--port", required=True, type=int)
    parser.add_argument("--report", required=True, type=Path)
    parser.add_argument("--source", required=True)
    parser.add_argument("--max-growth-mib", type=int, default=64)
    args = parser.parse_args()
    if not 26200 <= args.port <= 26219:
        parser.error("requires assigned loopback port 26200..26219")
    binary = args.openergy.resolve(strict=True)
    root = Path(tempfile.mkdtemp(prefix="oe-r5-queue-", dir="/tmp"))
    root.chmod(0o700)
    config = root / "config.toml"
    control = root / "control.sock"
    config.write_text(f'''version = 1
name = "r5-client"
role = "client"
[client]
remote = "127.0.0.1:{args.port}"
[tls]
enabled = false
''')
    env = dict(os.environ, OPENERGY_CONFIG_DIR=str(root),
               OPENERGY_TLS_DIR=str(root / "tls"), OPENERGY_SOCKET=str(control),
               OPENERGY_DISABLE_DISCOVERY="1", XDG_CONFIG_HOME=str(root),
               XDG_DATA_HOME=str(root), XDG_RUNTIME_DIR=str(root))
    env.pop("OPENERGY_ALLOW_PORT_24800", None)

    def ctl(method):
        with socket.socket(socket.AF_UNIX) as stream:
            stream.settimeout(2)
            stream.connect(str(control))
            stream.sendall(frame(json.dumps({"id": 1, "method": method, "params": {}}).encode()))
            response = json.loads(receive(stream))
            if "error" in response:
                raise RuntimeError(response["error"]["code"])
            return response["result"]

    def rss():
        return int(subprocess.check_output(["ps", "-o", "rss=", "-p", str(child.pid)])) * 1024

    with socket.socket() as listener, (root / "daemon.log").open("w") as log:
        listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        listener.bind(("127.0.0.1", args.port))
        listener.listen()
        listener.settimeout(5)
        child = subprocess.Popen(
            ["nice", "-n", "15", str(binary), "daemon", "--config", str(config),
             "--socket", str(control), "--platform", "virtual", "--foreground"],
            env=env, stdout=log, stderr=subprocess.STDOUT)
        try:
            with listener.accept()[0] as peer:
                peer.settimeout(1)
                peer.sendall(frame(b"Synergy" + struct.pack("!HH", 1, 8)))
                if not receive(peer).startswith(b"Synergy"):
                    raise RuntimeError("missing real hello reply")
                peer.sendall(frame(b"QINF"))
                if not receive(peer).startswith(b"DINF"):
                    raise RuntimeError("missing real screen-info reply")
                peer.sendall(frame(b"CIAK") + frame(b"DSOP" + struct.pack("!I", 0)))
                deadline = time.monotonic() + 3
                while not any(p["connected"] for p in ctl("status.get")["peers"]):
                    if time.monotonic() > deadline:
                        raise RuntimeError("real client did not become ready")
                    time.sleep(.02)
                baseline = rss()
                sent = [0]

                def flood():
                    try:
                        for _ in range(500):
                            peer.sendall(frame(b"CALV") * 8192)
                            sent[0] += 8192
                    except OSError:
                        pass

                sender = threading.Thread(target=flood)
                sender.start()
                samples = []
                start = time.monotonic()
                try:
                    while time.monotonic() - start < 6:
                        resident = rss()
                        before = time.monotonic()
                        status = ctl("status.get")
                        samples.append({"seconds": round(time.monotonic() - start, 3),
                                        "rss_bytes": resident, "sent_frames": sent[0],
                                        "events_out": status["stats"]["events_out"],
                                        "ipc_ms": round((time.monotonic() - before) * 1000, 3),
                                        "error_code": (status.get("last_error") or {}).get("code")})
                        time.sleep(.2)
                finally:
                    try:
                        peer.shutdown(socket.SHUT_RDWR)
                    except OSError:
                        pass  # The slow-peer watchdog may already have closed it.
                    sender.join(timeout=2)
                peak = max(s["rss_bytes"] for s in samples)
                stressed = max(s["events_out"] for s in samples) > 100_000 and any(
                    s["error_code"] == "network_queue_full" for s in samples)
                passed = stressed and peak - baseline <= args.max_growth_mib * 1024 * 1024
                report = {"scenario": "R5-tiny-control-queue-memory", "source": args.source,
                          "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
                          "platform": os.uname().sysname, "baseline_rss_bytes": baseline,
                          "peak_rss_bytes": peak, "growth_bytes": peak - baseline,
                          "growth_limit_bytes": args.max_growth_mib * 1024 * 1024,
                          "stressed": stressed, "passed": passed, "samples": samples}
                args.report.write_text(json.dumps(report, indent=2) + "\n")
                print(json.dumps({key: value for key, value in report.items() if key != "samples"}), flush=True)
                print(f"ARTIFACTS {root}", flush=True)
        finally:
            try:
                ctl("daemon.stop")
            except (OSError, EOFError, RuntimeError):
                child.terminate()
            try:
                child.wait(timeout=5)
            except subprocess.TimeoutExpired:
                child.kill()
                child.wait(timeout=3)
        if child.returncode != 0:
            raise RuntimeError(f"owned virtual daemon exit {child.returncode}")
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
