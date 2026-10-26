#!/usr/bin/env python3
"""Exercise real TLS trust refresh at the 16-connection cap, with private state."""

import argparse
import concurrent.futures
import hashlib
import json
import math
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time

from verify_daemon_audit import Node, require


SAMPLES = 120
INTERVAL = .04
CPU_BUDGET = 5.0
P99_BUDGET_MS = 100


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--openergy", type=Path, required=True)
    parser.add_argument("--port-range", required=True)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    low, high = map(int, args.port_range.split("-"))
    require(1 <= low and low + 77 <= high <= 65535 and not low <= 24800 <= high, "unsafe port range")
    binary = args.openergy.resolve(strict=True)
    root = Path(tempfile.mkdtemp(prefix="oe-trust-refresh-", dir="/tmp"))
    root.chmod(0o700)
    pins = [hashlib.sha256(f"openergy-refresh-{i}".encode()).hexdigest() for i in range(16000)]
    results = []

    def call(node, method, params=None):
        response = node.call(method, params)
        require("result" in response, f"unexpected {method} failure")
        return response["result"]

    def wait(fn, timeout=8, interval=.01):
        end = time.monotonic() + timeout
        while time.monotonic() < end:
            value = fn()
            if value:
                return value
            time.sleep(interval)
        raise AssertionError("real peer did not reach expected state")

    def identity(node):
        value = subprocess.run([str(binary), "fingerprint", "--dir", str(node.dir)],
                               env=dict(node.env, HOSTNAME="openergy-refresh-lab"), capture_output=True, timeout=4)
        require(value.returncode == 0, "private identity preparation failed")
        return value.stdout.splitlines()[0].decode().replace(":", "").lower()

    def client_config(node, name, port, role="client"):
        node.config.write_text(f'version=1\nname="{name}"\nrole="{role}"\n'
                               f'[client]\nremote="127.0.0.1:{port}"\n[tls]\nenabled=true\n')

    def server_config(node, mutual=True):
        text = ('version=1\nname="server"\nrole="server"\n'
                f'[server]\nlisten="127.0.0.1:{node.port}"\nport={node.port}\n'
                f'[tls]\nenabled=true\nrequire_client_trust={str(mutual).lower()}\n')
        for i, name in enumerate(["server", "client"] + [f"held-{i}" for i in range(15)]):
            text += f'[[screen]]\nname="{name}"\nrect={{x={i * 1920}.0,y=0.0,w=1920.0,h=1080.0}}\n'
        node.config.write_text(text)

    def write_pins(node, name, values):
        (node.dir / "tls" / name).write_text("".join(p + "\n" for p in values))

    def enter(server):
        call(server, "virtual.inject", {"events": [{"type": "mouse_move_abs", "x": 1919, "y": 540},
                                                   {"type": "mouse_move_rel", "dx": 20, "dy": 0}]})
        wait(lambda: call(server, "status.get")["active_screen"] == "client")

    def hold(server, client):
        call(server, "virtual.inject", {"events": [{"type": "key_down", "key": 97, "modifiers": 0, "button": 1}]})
        wait(lambda: call(client, "virtual.recorded")["stats"]["pressed_keys"] == 1)

    def transfer(server, client, mark):
        started = time.monotonic()
        call(server, "virtual.inject", {"events": [{"type": "key_down", "key": 98, "modifiers": 0, "button": 2},
                                                   {"type": "key_up", "key": 98, "modifiers": 0, "button": 2}]})
        def recorded():
            result = call(client, "virtual.recorded", {"since": mark})
            return result if any((e.get("event") or {}).get("type") == "key_up"
                                 and (e.get("event") or {}).get("key") == 98 for e in result["events"]) else None
        ledger = wait(recorded, interval=.001)
        return (time.monotonic() - started) * 1000, ledger["stats"]["last_sequence"]

    def cpu_seconds(pid):
        if sys.platform == "darwin":
            value = subprocess.check_output(["ps", "-p", str(pid), "-o", "time="], text=True).strip()
            return sum(float(part) * 60 ** i for i, part in enumerate(reversed(value.split(":"))))
        if sys.platform.startswith("linux"):
            fields = Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()
            return (int(fields[11]) + int(fields[12])) / os.sysconf("SC_CLK_TCK")
        raise RuntimeError("CPU measurement supports macOS and Linux")

    def distribution(values):
        values = sorted(values)
        return {"samples": len(values), **{f"p{q}_ms": values[math.ceil(q / 100 * len(values)) - 1]
                                          for q in (50, 95, 99)}, "max_ms": max(values)}

    def cleanup(nodes):
        for node in reversed(nodes):
            node.stop()
        return all((node.proc is None or node.proc.poll() is not None) and not node.socket.exists() for node in nodes)

    for index, (kind, count, large) in enumerate((("large_store_alone", 0, True),
                                                 ("small_store_15_held", 15, False),
                                                 ("large_store_15_held", 15, True))):
        directory = root / kind
        directory.mkdir()
        server = Node(directory, binary, low + index * 20)
        client = Node(directory, binary, server.port + 1)
        nodes, held = [server, client], []
        outcome = {"case": kind, "held_peers": count, "extra_pins_per_role": len(pins) if large else 0, "passed": False}
        try:
            server_config(server)
            client_config(client, "client", server.port)
            server_pin, client_pin = identity(server), identity(client)
            values = pins if large else []
            write_pins(server, "trusted-servers", values)
            write_pins(server, "trusted-clients", values + [client_pin])
            write_pins(client, "trusted-servers", [server_pin])
            server.start()
            server.ready()
            client.start()
            client.ready()
            wait(lambda: call(client, "status.get")["state"] == "connected")
            enter(server)
            hold(server, client)
            for i in range(count):
                node = Node(directory, binary, server.port + 2 + i)
                nodes.append(node)
                client_config(node, f"held-{i}", server.port)
                pin = identity(node)
                write_pins(node, "trusted-servers", [server_pin])
                node.start()
                node.ready()
                held.append((node, pin))
            wait(lambda: len(call(server, "status.get")["pending_trust"]) == count)
            config = server.config.read_bytes()
            before_files = [(server.dir / "tls" / name).read_bytes() for name in ("trusted-servers", "trusted-clients")]
            started, before_cpu = time.monotonic(), cpu_seconds(server.proc.pid)
            def status_samples():
                times = []
                for _ in range(SAMPLES):
                    before = time.monotonic()
                    call(server, "status.get")
                    times.append((time.monotonic() - before) * 1000)
                    time.sleep(INTERVAL)
                return times
            input_times = []
            with concurrent.futures.ThreadPoolExecutor(max_workers=1) as pool:
                statuses = pool.submit(status_samples)
                mark = call(client, "virtual.recorded")["stats"]["last_sequence"]
                for _ in range(SAMPLES):
                    elapsed, mark = transfer(server, client, mark)
                    input_times.append(elapsed)
                    time.sleep(INTERVAL)
                status_times = statuses.result(timeout=8)
            window = time.monotonic() - started
            outcome.update(input=distribution(input_times), status=distribution(status_times), window_ms=window * 1000,
                           server_cpu_percent=(cpu_seconds(server.proc.pid) - before_cpu) / window * 100,
                           pending_preserved=len(call(server, "status.get")["pending_trust"]) == count,
                           held_key_preserved=call(client, "virtual.recorded")["stats"]["pressed_keys"] == 1,
                           config_preserved=server.config.read_bytes() == config,
                           files_preserved=before_files == [(server.dir / "tls" / name).read_bytes()
                                                          for name in ("trusted-servers", "trusted-clients")])
            perf_passed = (outcome["input"]["p99_ms"] < P99_BUDGET_MS
                           and outcome["status"]["p99_ms"] < P99_BUDGET_MS
                           and outcome["server_cpu_percent"] < CPU_BUDGET)
            require(all(outcome[k] for k in ("pending_preserved", "held_key_preserved", "config_preserved", "files_preserved")),
                    "cohort workload changed valid state")
            if kind == "large_store_15_held":
                # A wrong-role pin must stay held. A correct-role file change must
                # be observed by the next refresh without an IPC approval.
                write_pins(server, "trusted-servers", [held[1][1]])
                write_pins(server, "trusted-clients", [client_pin, held[0][1]])
                wait(lambda: call(held[0][0], "status.get")["state"] == "connected")
                pending = wait(lambda: call(server, "status.get")["pending_trust"]
                               if len(call(server, "status.get")["pending_trust"]) == 14 else None)
                wrong = held[1][1].upper()
                require(any(p["fingerprint"].replace(":", "") == wrong for p in pending), "wrong-role pin was accepted")
                require(call(client, "virtual.recorded")["stats"]["pressed_keys"] == 1, "unrelated approval released held input")
                outcome.update(fresh_file_approval=True, wrong_role_stays_held=True)
                (server.dir / "tls" / "trusted-clients").write_bytes(b"\xff")
                wait(lambda: call(client, "virtual.recorded")["stats"]["pressed_keys"] == 0)
                wait(lambda: not any(p["connected"] for p in call(server, "status.get")["peers"]))
                outcome["read_failure_closed_required_peers"] = True
                write_pins(server, "trusted-clients", [client_pin, held[0][1]])
                wait(lambda: call(client, "status.get")["state"] == "connected")
                wait(lambda: call(held[0][0], "status.get")["state"] == "connected")
                outcome["read_failure_recovered"] = True
            outcome["within_budgets"] = perf_passed
            require(perf_passed, "cohort exceeded declared workload CPU or p99 budget")
            outcome["passed"] = True
        except Exception as exc:
            outcome["passed"] = False
            outcome["error_detail"] = f"{type(exc).__name__}: {exc}"
        finally:
            outcome["cleanup"] = cleanup(nodes)
            outcome["passed"] = outcome["passed"] and outcome["cleanup"]
        results.append(outcome)
        print(f'{"PASS" if outcome["passed"] else "FAIL"} {kind}: {json.dumps(outcome)}', flush=True)

    # Pin and optional-client-auth guards must keep bypassing store reads, even
    # when a dirty refresh has a real filesystem/encoding error.
    directory = root / "bypass_read_failure"
    directory.mkdir()
    server, client = Node(directory, binary, low + 60), Node(directory, binary, low + 61)
    outcome = {"case": "bypass_read_failure", "passed": False}
    try:
        server_config(server, mutual=False)
        client_config(client, "client", server.port, role="off")
        server_pin = identity(server)
        identity(client)
        server.start()
        server.ready()
        client.start()
        client.ready()
        call(client, "connection.connect", {"host": "127.0.0.1", "port": server.port, "fingerprint": server_pin})
        wait(lambda: call(client, "status.get")["state"] == "connected")
        enter(server)
        hold(server, client)
        for node, name in ((server, "trusted-clients"), (client, "trusted-servers")):
            (node.dir / "tls" / name).write_bytes(b"\xff")
            response = node.call("trust.approve", {"fingerprint": "ab" * 32})
            require(response.get("error", {}).get("code") == "trust_failed", "corrupt store mutation was accepted")
        for node in (server, client):
            wait(lambda: "transport error peer=None:" in (node.dir / "stderr.log").read_text())
        elapsed, _ = transfer(server, client, call(client, "virtual.recorded")["stats"]["last_sequence"])
        require(call(client, "status.get")["state"] == "connected"
                and call(client, "virtual.recorded")["stats"]["pressed_keys"] == 1, "bypass peer disconnected on store error")
        outcome.update(explicit_pin_preserved=True, optional_client_trust_preserved=True,
                       real_refresh_error_seen=True, input_ms=elapsed)
        write_pins(server, "trusted-clients", [])
        write_pins(client, "trusted-servers", [])
        call(server, "trust.approve", {"fingerprint": "ab" * 32})
        call(client, "trust.approve", {"fingerprint": "ab" * 32})
        outcome.update(passed=True, valid_store_retry_succeeded=True)
    except Exception as exc:
        outcome["passed"] = False
        outcome["error_detail"] = f"{type(exc).__name__}: {exc}"
    finally:
        outcome["cleanup"] = cleanup([server, client])
        outcome["passed"] = outcome["passed"] and outcome["cleanup"]
    results.append(outcome)
    print(f'{"PASS" if outcome["passed"] else "FAIL"} bypass_read_failure: {json.dumps(outcome)}', flush=True)
    report = {"binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
              "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
              "connection_cap": 16, "cpu_budget_percent": CPU_BUDGET, "p99_budget_ms": P99_BUDGET_MS,
              "cases": results, "passed": all(r["passed"] for r in results), "evidence_dir": str(root)}
    args.report.write_text(json.dumps(report, indent=2) + "\n")
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
