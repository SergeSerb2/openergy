#!/usr/bin/env python3
"""Verify request-read recovery through a real virtual daemon, CLI and Unix sockets."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import signal
import socket
import struct
import subprocess
import tempfile
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--openergy", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    binary = args.openergy.resolve(strict=True)
    root = Path(tempfile.mkdtemp(prefix="oe-ipc-deadline-"))
    root.chmod(0o700)
    endpoint = root / "ipc.sock"
    config = root / "config.toml"
    config.write_text('version = 1\nname = "ipc-deadline-lab"\nrole = "off"\n[tls]\nenabled = false\n')
    env = dict(os.environ)
    env.update({
        "OPENERGY_ALLOW_PORT_24800": "0", "OPENERGY_DISABLE_DISCOVERY": "1",
        "OPENERGY_CONFIG": str(config), "OPENERGY_CONFIG_DIR": str(root),
        "OPENERGY_SOCKET": str(endpoint), "OPENERGY_TLS_DIR": str(root / "tls"),
        "XDG_CONFIG_HOME": str(root / "xdg-config"), "XDG_DATA_HOME": str(root / "xdg-data"),
        "XDG_RUNTIME_DIR": str(root),
    })
    report = {"binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
              "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
              "platform": "virtual", "transport": "private Unix socket", "checks": []}
    sockets = []
    proc = None
    error = None

    def check(name, passed, **detail):
        report["checks"].append({"name": name, "passed": bool(passed), **detail})
        print(f'{"PASS" if passed else "FAIL"} {name}', flush=True)

    def connect():
        stream = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        stream.settimeout(2)
        stream.connect(str(endpoint))
        sockets.append(stream)
        return stream

    def frame(method, request_id):
        payload = json.dumps({"id": request_id, "method": method, "params": {}}).encode()
        return struct.pack(">I", len(payload)) + payload

    def read_exact(stream, count):
        data = bytearray()
        while len(data) < count:
            chunk = stream.recv(count - len(data))
            if not chunk:
                raise RuntimeError("IPC stream closed before the reply")
            data.extend(chunk)
        return data

    def response(stream):
        count = struct.unpack(">I", read_exact(stream, 4))[0]
        if count > 16 * 1024 * 1024:
            raise RuntimeError("reply exceeds IPC frame limit")
        return json.loads(read_exact(stream, count))

    def cli(method):
        return subprocess.run([str(binary), "ctl", method, "--socket", str(endpoint), "--raw"],
                              env=env, capture_output=True, text=True, timeout=4)

    def closed(stream):
        stream.setblocking(False)
        try:
            return stream.recv(1, socket.MSG_PEEK) == b""
        except (BlockingIOError, socket.timeout):
            return False
        except (ConnectionResetError, BrokenPipeError):
            return True
        finally:
            stream.settimeout(2)

    try:
        with (root / "daemon.log").open("wb") as log:
            proc = subprocess.Popen(["nice", "-n", "15", str(binary), "daemon", "--foreground",
                                     "--platform", "virtual", "--config", str(config), "--socket", str(endpoint)],
                                    env=env, stdout=log, stderr=log, start_new_session=True)
            deadline = time.monotonic() + 8
            while not endpoint.exists() and proc.poll() is None and time.monotonic() < deadline:
                time.sleep(0.02)
            initial = cli("status.get")
            check("initial public CLI status", initial.returncode == 0)
            if initial.returncode:
                raise RuntimeError("private daemon did not become ready")

            # One completed request may remain idle, as a persistent UI client does.
            idle = connect()
            idle.sendall(frame("status.get", 1))
            check("persistent client initial reply", response(idle).get("id") == 1)
            time.sleep(0.1)  # Let the short-lived startup CLI leave its client slot.
            pending = []
            for index in range(63):
                stream = connect()
                kind = ("empty", "partial_header", "partial_body", "drip")[index % 4]
                if kind == "partial_header":
                    stream.sendall(b"\x00\x00")
                elif kind in ("partial_body", "drip"):
                    stream.sendall(struct.pack(">I", 1024) + b"{")
                pending.append((kind, stream))
                time.sleep(0.003)
            check("all 63 pending clients initially admitted", all(not closed(s) for _, s in pending))
            started = time.monotonic()
            next_drip = started
            # Longer than the 10-second request deadline. Drips must not reset it.
            while time.monotonic() - started < 11.5:
                if time.monotonic() >= next_drip:
                    for kind, stream in pending:
                        if kind == "drip":
                            try:
                                stream.sendall(b" ")
                            except (BrokenPipeError, ConnectionResetError):
                                pass
                    next_drip += 0.6
                time.sleep(0.025)
            for kind in ("empty", "partial_header", "partial_body", "drip"):
                group = [stream for name, stream in pending if name == kind]
                count = sum(closed(stream) for stream in group)
                check(f"{kind} clients expire", count == len(group), closed=count, total=len(group))
            recovered = cli("status.get")
            check("public CLI recovers while incomplete clients stay open", recovered.returncode == 0,
                  returncode=recovered.returncode)
            idle.sendall(frame("status.get", 2))
            check("completed-request idle client remains usable", response(idle).get("id") == 2)
            pending[0][1].close()
            time.sleep(0.1)
            check("releasing one held client slot restores CLI access", cli("status.get").returncode == 0)
            for _, stream in pending:
                stream.close()

            # Completing a fragmented frame in time must still work.
            fragmented = connect()
            request = frame("status.get", 3)
            for part in (request[:2], request[2:9], request[9:]):
                fragmented.sendall(part)
                time.sleep(0.1)
            check("fragmented complete request succeeds", response(fragmented).get("id") == 3)
            fragmented.sendall(frame("status.get", 4) + frame("status.get", 5))
            check("pipelined complete requests preserve reply order",
                  [response(fragmented).get("id"), response(fragmented).get("id")] == [4, 5])
            fragmented.sendall(frame("no.such.method", 6))
            invalid = response(fragmented)
            check("unknown method returns an error", invalid.get("id") == 6 and bool(invalid.get("error")))
            fragmented.sendall(frame("status.get", 7))
            check("valid request after method error succeeds", response(fragmented).get("id") == 7)
            fragmented.sendall(b"\x00\x00")
            time.sleep(11.0)
            check("partial frame after a completed request expires", closed(fragmented))

            oversize = connect()
            oversize.sendall(struct.pack(">I", 16 * 1024 * 1024 + 1))
            time.sleep(0.15)
            check("oversized header is refused", closed(oversize))
            check("public CLI works after refusal", cli("status.get").returncode == 0)
            # Leave a fresh partial request present during shutdown.
            connect().sendall(b"\x00")
            stop = cli("daemon.stop")
            proc.wait(timeout=4)
            check("shutdown with a partial client exits cleanly", stop.returncode == 0 and proc.returncode == 0)
            check("owned IPC socket removed", not endpoint.exists())
    except Exception as exc:
        error = f"{type(exc).__name__}: {exc}"
        check("verification completed", False, error=error)
    finally:
        for stream in sockets:
            stream.close()
        if proc is not None and proc.poll() is None:
            try:
                cli("daemon.stop")
                proc.wait(timeout=4)
            except (OSError, subprocess.SubprocessError):
                os.killpg(proc.pid, signal.SIGTERM)
                try:
                    proc.wait(timeout=4)
                except subprocess.TimeoutExpired:
                    os.killpg(proc.pid, signal.SIGKILL)
                    proc.wait(timeout=4)
        report["daemon_exit"] = proc.returncode if proc is not None else None
        report["cleanup"] = proc is not None and proc.poll() is not None and not endpoint.exists()
        report["passed"] = error is None and report["cleanup"] and all(item["passed"] for item in report["checks"])
        report["evidence_dir"] = str(root)
        args.report.write_text(json.dumps(report, indent=2) + "\n")
        print(f'Report: {args.report}; passed={report["passed"]}', flush=True)
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
