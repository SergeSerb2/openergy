#!/usr/bin/env python3
"""Exercise the real 16-peer cap, connection churn, IPC and virtual input.

Only private state, virtual daemons and explicitly assigned loopback ports are
used. The 100 ms p99 gate is a local workload budget, not physical input latency.
"""

import argparse
import concurrent.futures
import hashlib
import json
import math
from pathlib import Path
import socket
import struct
import tempfile
import threading
import time

from verify_daemon_audit import Node, require


SAMPLES = 60
INTERVAL = .02
P99_BUDGET_MS = 100


def wire(payload):
    return struct.pack("!I", len(payload)) + payload


def receive(stream):
    def exact(count):
        data = bytearray()
        while len(data) < count:
            part = stream.recv(count - len(data))
            require(part, "unexpected parked-peer EOF")
            data.extend(part)
        return data
    count = struct.unpack("!I", exact(4))[0]
    require(4 <= count <= 4 * 1024 * 1024, "invalid parked-peer frame")
    return bytes(exact(count))


def expect(stream, code):
    for _ in range(20):
        if receive(stream).startswith(code):
            return
    raise AssertionError("parked peer did not finish the real handshake")


def call(node, method, params=None):
    response = node.call(method, params)
    require("result" in response, f"unexpected {method} failure")
    return response["result"]


def wait(fn, timeout=5):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        value = fn()
        if value:
            return value
        time.sleep(.005)
    raise AssertionError("real peer state timed out")


def distribution(values):
    ordered = sorted(values)
    return {"samples": len(values), "p50_ms": ordered[math.ceil(.5 * len(values)) - 1],
            "p99_ms": ordered[math.ceil(.99 * len(values)) - 1], "max_ms": max(values)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--openergy", type=Path, required=True)
    parser.add_argument("--port-range", required=True)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    low, high = map(int, args.port_range.split("-"))
    require(1 <= low and low + 7 <= high <= 65535 and not low <= 24800 <= high, "unsafe port range")
    binary = args.openergy.resolve(strict=True)
    root = Path(tempfile.mkdtemp(prefix="oe-connection-churn-", dir="/tmp"))
    root.chmod(0o700)
    results = []
    for index, workers in enumerate((0, 4, 16, 32)):
        directory = root / f"workers-{workers}"
        directory.mkdir(mode=0o700)
        server, client = (Node(directory, binary, low + index * 2 + offset) for offset in (0, 1))
        server.config.write_text(
            'version=1\nname="server"\nrole="server"\n'
            f'[server]\nlisten="127.0.0.1:{server.port}"\nport={server.port}\nheartbeat_ms=300000\n'
            '[tls]\nenabled=false\n' + ''.join(
                f'[[screen]]\nname="{name}"\nrect={{x={i * 1920}.0,y=0.0,w=1920.0,h=1080.0}}\n'
                for i, name in enumerate(["server", "client"] + [f"parked-{j}" for j in range(15)])))
        client.config.write_text('version=1\nname="client"\nrole="client"\n'
                                 f'[client]\nremote="127.0.0.1:{server.port}"\n[tls]\nenabled=false\n')
        stop = threading.Event()
        parked = []
        outcome = {"workers": workers, "passed": False}

        def churn():
            rejected, admitted, errors = 0, 0, 0
            while not stop.is_set():
                try:
                    with socket.create_connection(("127.0.0.1", server.port), timeout=.15) as peer:
                        if peer.recv(1):
                            admitted += 1
                        else:
                            rejected += 1
                except OSError:
                    errors += 1
            return {"rejected_at_cap": rejected, "unexpected_admissions": admitted, "socket_errors": errors}

        try:
            server.start()
            server.ready()
            client.start()
            client.ready()
            wait(lambda: call(client, "status.get")["state"] == "connected")
            call(server, "virtual.inject", {"events": [{"type": "mouse_move_abs", "x": 1919, "y": 540},
                                                      {"type": "mouse_move_rel", "dx": 20, "dy": 0}]})
            wait(lambda: call(server, "status.get")["active_screen"] == "client")
            call(server, "virtual.inject", {"events": [{"type": "key_down", "key": 97,
                                                        "modifiers": 0, "button": 1}]})
            wait(lambda: call(client, "virtual.recorded")["stats"]["pressed_keys"] == 1)
            for i in range(15):
                peer = socket.create_connection(("127.0.0.1", server.port), timeout=2)
                parked.append(peer)
                expect(peer, b"Synergy")
                name = f"parked-{i}".encode()
                peer.sendall(wire(b"Synergy" + struct.pack("!HHI", 1, 8, len(name)) + name))
                expect(peer, b"QINF")
                peer.sendall(wire(b"DINF" + struct.pack("!hhhhhhh", 0, 0, 1920, 1080, 0, 0, 0)))
                expect(peer, b"DSOP")
            wait(lambda: len(call(server, "status.get")["peers"]) == 16
                 and all(p["connected"] for p in call(server, "status.get")["peers"]))
            # One real overflow connection must close before the first protocol byte.
            with socket.create_connection(("127.0.0.1", server.port), timeout=1) as overflow:
                require(not overflow.recv(1), "17th peer was admitted")
            config_bytes = [node.config.read_bytes() for node in (server, client)]
            mark = call(client, "virtual.recorded")["stats"]["last_sequence"]

            def statuses():
                samples = []
                for _ in range(SAMPLES):
                    started = time.monotonic()
                    call(server, "status.get")
                    samples.append((time.monotonic() - started) * 1000)
                    time.sleep(INTERVAL)
                return samples

            with concurrent.futures.ThreadPoolExecutor(max_workers=workers + 1) as pool:
                churn_jobs = [pool.submit(churn) for _ in range(workers)]
                status_job = pool.submit(statuses)
                input_samples = []
                started = time.monotonic()
                try:
                    for _ in range(SAMPLES):
                        before = time.monotonic()
                        call(server, "virtual.inject", {"events": [
                            {"type": "key_down", "key": 98, "modifiers": 0, "button": 2},
                            {"type": "key_up", "key": 98, "modifiers": 0, "button": 2}]})

                        def delivered():
                            ledger = call(client, "virtual.recorded", {"since": mark})
                            return ledger if any((e.get("event") or {}).get("type") == "key_up"
                                                 and (e.get("event") or {}).get("key") == 98
                                                 for e in ledger["events"]) else None

                        ledger = wait(delivered)
                        input_samples.append((time.monotonic() - before) * 1000)
                        mark = ledger["stats"]["last_sequence"]
                        time.sleep(INTERVAL)
                    status_samples = status_job.result(timeout=5)
                finally:
                    stop.set()
                counts = [job.result(timeout=2) for job in churn_jobs]
                outcome.update(window_ms=(time.monotonic() - started) * 1000,
                               input=distribution(input_samples), status=distribution(status_samples),
                               **{key: sum(c[key] for c in counts) for key in
                                  ("rejected_at_cap", "unexpected_admissions", "socket_errors")})
            state = call(server, "status.get")
            outcome.update(peers_preserved=len(state["peers"]) == 16 and all(p["connected"] for p in state["peers"]),
                           held_key_preserved=call(client, "virtual.recorded", {"since": mark})["stats"]["pressed_keys"] == 1,
                           config_preserved=config_bytes == [node.config.read_bytes() for node in (server, client)],
                           active_screen_preserved=state["active_screen"] == "client", overflow_refused=True)
            require(all(outcome[k] for k in ("peers_preserved", "held_key_preserved", "config_preserved",
                                             "active_screen_preserved")), "churn changed valid session state")
            require(outcome["unexpected_admissions"] == 0, "churn admitted a 17th peer")
            require(workers == 0 or outcome["rejected_at_cap"] >= 1000, "churn workload was not reached")
            require(outcome["input"]["p99_ms"] < P99_BUDGET_MS
                    and outcome["status"]["p99_ms"] < P99_BUDGET_MS, "churn exceeded local p99 budget")
            call(server, "virtual.inject", {"events": [{"type": "key_up", "key": 97, "modifiers": 0, "button": 1}]})
            wait(lambda: call(client, "virtual.recorded", {"since": mark})["stats"]["pressed_keys"] == 0)
            outcome["release_delivered"] = True
            outcome["passed"] = True
        except Exception as error:
            outcome["error"] = f"{type(error).__name__}: {error}"
        finally:
            stop.set()
            for peer in parked:
                peer.close()
            for node in (client, server):
                node.stop()
            outcome["cleanup"] = all((node.proc is None or node.proc.poll() is not None)
                                     and not node.socket.exists() for node in (server, client))
            outcome["passed"] &= outcome["cleanup"]
        results.append(outcome)
        print(json.dumps(outcome), flush=True)
    report = {"binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(), "workers": [0, 4, 16, 32],
              "samples_per_probe_per_case": SAMPLES, "p99_budget_ms": P99_BUDGET_MS,
              "tls": False, "platform": "virtual", "cases": results,
              "passed": all(case["passed"] for case in results)}
    args.report.write_text(json.dumps(report, indent=2) + "\n")
    print(f"ARTIFACTS {root}", flush=True)
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
