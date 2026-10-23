#!/usr/bin/env python3
"""Exercise legal large trust lists through actual CLI, TLS peers and public IPC."""

import argparse
import concurrent.futures
import hashlib
import json
import os
from pathlib import Path
import re
import statistics
import subprocess
import sys
import tempfile
import time

from verify_daemon_audit import Node, require


CAP = 1024 * 1024
BUDGET_MS = 100
FILES = ("trusted-servers", "trusted-clients")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--openergy", type=Path, required=True)
    parser.add_argument("--port-range", required=True)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    low, high = map(int, args.port_range.split("-"))
    require(1 <= low and low + 5 <= high <= 65535 and not low <= 24800 <= high, "unsafe port range")
    binary = args.openergy.resolve(strict=True)
    root = Path(tempfile.mkdtemp(prefix="oe-trust-scaling-", dir="/tmp"))
    root.chmod(0o700)
    pins = [hashlib.sha256(f"openergy-scaling-{i}".encode()).hexdigest() for i in range(16000)]
    results = []

    def canonical(values):
        return "".join(f"v2:sha256:{p}\n" for p in values).encode()

    def expected_list(servers, clients):
        return b"".join(f"{role} v2:sha256:{p}\n".encode()
                        for role, values in (("server", servers), ("client", clients)) for p in values)

    def write_lists(directory, data):
        directory.mkdir(mode=0o700, exist_ok=True)
        for name, body in zip(FILES, data):
            require(len(body) <= CAP, "fixture exceeds the existing file policy")
            (directory / name).write_bytes(body)

    def cli(node, *arguments):
        command = [str(binary), "trust", *arguments, "--dir", str(node.dir)]
        measured = sys.platform == "darwin"
        if measured:
            command = ["/usr/bin/time", "-l", *command]
        started = time.monotonic()
        value = subprocess.run(command, env=node.env, capture_output=True, timeout=8)
        rss = re.search(rb"(\d+)\s+maximum resident set size", value.stderr) if measured else None
        return value, (time.monotonic() - started) * 1000, int(rss[1]) if rss else None

    # Warm the executable separately from the declared timing samples.
    warm = Node(root, binary, low)
    value, _, _ = cli(warm, "list")
    require(value.returncode == 0, "warm-up command failed")

    for kind in ("small_order_and_legacy", "large_compact_list"):
        node = Node(root, binary, low)
        values = [pins[2], pins[0], pins[1]] if kind.startswith("small") else pins[10000:] + pins[:10000]
        duplicates = values[:2] if kind.startswith("small") else values[:64]
        body = ("\n".join(values + duplicates) + "\nv1:sha1:" + "11" * 20
                + "\ninvalid\n\n" + ":".join([values[0][i:i + 2].upper() for i in range(0, 64, 2)]) + "\n").encode()
        write_lists(node.dir / "tls", (body, body))
        outcome = {"case": kind, "passed": False, "unique_pins_per_role": len(values), "bytes_per_file": len(body)}
        try:
            times, resident = [], []
            for _ in range(3):
                value, elapsed, rss = cli(node, "list")
                require(value.returncode == 0 and value.stdout == expected_list(values, values),
                        "list changed first-seen order, roles or legacy/duplicate handling")
                times.append(elapsed)
                if rss is not None:
                    resident.append(rss)
            outcome.update(elapsed_ms=times, median_ms=statistics.median(times), max_rss_bytes=max(resident, default=None),
                           files_preserved=all((node.dir / "tls" / n).read_bytes() == body for n in FILES),
                           exact_order_and_roles=True)
            require(outcome["files_preserved"] and outcome["median_ms"] < BUDGET_MS, "list exceeded lab budget or changed files")
            outcome["passed"] = True
        except Exception as exc:
            outcome["error_detail"] = f"{type(exc).__name__}: {exc}"
        results.append(outcome)

    for kind in ("bulk_same_lists", "bulk_overlap", "bulk_canonical_expansion_refused", "compact_revoke"):
        node = Node(root, binary, low)
        source = node.dir / "import"
        outcome = {"case": kind, "passed": False}
        if kind == "bulk_same_lists":
            destination, incoming, expected = pins[:13000], pins[:13000], pins[:13000]
        elif kind == "bulk_overlap":
            destination, incoming, expected = pins[:4000], pins[2000:12000], pins[:12000]
        elif kind == "bulk_canonical_expansion_refused":
            destination, incoming, expected = [pins[-1]], pins, [pins[-1]]
        else:
            destination, incoming, expected = pins, [], pins[:8000] + pins[8001:]
        compact = kind in ("compact_revoke", "bulk_canonical_expansion_refused")
        encode = lambda values: ("\n".join(values) + "\n").encode() if compact else canonical(values)
        data = encode(destination)
        write_lists(node.dir / "tls", (data, data))
        write_lists(source, (encode(incoming), encode(incoming)))
        try:
            if kind == "compact_revoke":
                value, elapsed, rss = cli(node, "revoke", pins[8000])
            else:
                value, elapsed, rss = cli(node, "import-synergy", str(source))
            refused = kind == "bulk_canonical_expansion_refused"
            outcome.update(elapsed_ms=elapsed, max_rss_bytes=rss, exit=value.returncode)
            if refused:
                require(value.returncode != 0 and b"exceeds 1 MiB" in value.stderr, "oversized save was accepted")
                require(all((node.dir / "tls" / n).read_bytes() == data for n in FILES), "refused import changed files")
                require(not list((node.dir / "tls").glob(".oe-*.tmp")), "refused import left temporary output")
                outcome["files_preserved"] = True
            else:
                added = 2 * (len(expected) - len(destination))
                require(value.returncode == 0, "valid update failed")
                if kind != "compact_revoke":
                    require(value.stdout == f"imported {added} trust records\n".encode(), "import count changed")
                after = [(node.dir / "tls" / n).read_bytes() for n in FILES]
                require(all(len(body) <= CAP for body in after), "saved list exceeds policy")
                if kind == "bulk_same_lists":
                    require(after == [data, data], "no-op import rewrote files")
                check, _, _ = cli(node, "list")
                require(check.returncode == 0 and check.stdout == expected_list(expected, expected),
                        "update changed first-seen order, roles or pins")
                outcome.update(exact_order_and_roles=True, pins_per_role=len(expected))
            require(elapsed < BUDGET_MS, "bulk command exceeded lab budget")
            outcome["passed"] = True
        except Exception as exc:
            outcome["error_detail"] = f"{type(exc).__name__}: {exc}"
        results.append(outcome)

    def call(node, method, params=None):
        response = node.call(method, params)
        require("result" in response, f"unexpected {method} failure")
        return response["result"]

    def wait(fn):
        end = time.monotonic() + 5
        while time.monotonic() < end:
            value = fn()
            if value:
                return value
            time.sleep(.01)
        raise AssertionError("real peer did not reach expected state")

    def timed(node, method, params=None):
        started = time.monotonic()
        response = node.call(method, params)
        return response, (time.monotonic() - started) * 1000

    for index, kind in enumerate(("live_server_approve", "live_client_approve", "live_server_revoke")):
        server = Node(root, binary, low + index * 2)
        client = Node(root, binary, low + index * 2 + 1)
        server.config.write_text('version=1\nname="server"\nrole="server"\n'
                                 f'[server]\nlisten="127.0.0.1:{server.port}"\nport={server.port}\n'
                                 '[tls]\nenabled=true\nrequire_client_trust=true\n'
                                 '[[screen]]\nname="server"\nrect={x=0.0,y=0.0,w=1920.0,h=1080.0}\n'
                                 '[[screen]]\nname="client"\nrect={x=1920.0,y=0.0,w=1920.0,h=1080.0}\n')
        client.config.write_text('version=1\nname="client"\nrole="client"\n'
                                 f'[client]\nremote="127.0.0.1:{server.port}"\n[tls]\nenabled=true\n')
        outcome = {"case": kind, "passed": False}
        try:
            identities = []
            for node in (server, client):
                value = subprocess.run([str(binary), "fingerprint", "--dir", str(node.dir)],
                                       env=dict(node.env, HOSTNAME="openergy-scale-lab"), capture_output=True, timeout=4)
                require(value.returncode == 0, "private identity creation failed")
                identities.append(value.stdout.splitlines()[0].decode().replace(":", "").lower())
            target = client if kind == "live_client_approve" else server
            server_values = ([identities[0]], [])
            write_lists(client.dir / "tls", tuple(canonical(v) for v in server_values))
            write_lists(server.dir / "tls", (b"", canonical([identities[1]])))
            base = pins[:13000]
            servers = base + ([identities[0]] if target is client else [])
            clients = base + ([identities[1]] if target is server else [])
            write_lists(target.dir / "tls", (canonical(servers), canonical(clients)))
            server.start()
            server.ready()
            client.start()
            client.ready()
            wait(lambda: call(client, "status.get")["state"] == "connected")
            call(server, "virtual.inject", {"events": [{"type": "mouse_move_abs", "x": 1919, "y": 540},
                                                       {"type": "mouse_move_rel", "dx": 20, "dy": 0}]})
            wait(lambda: call(server, "status.get")["active_screen"] == "client")
            call(server, "virtual.inject", {"events": [{"type": "key_down", "key": 97, "modifiers": 0, "button": 1}]})
            wait(lambda: call(client, "virtual.recorded")["stats"]["pressed_keys"] == 1)
            mark = call(client, "virtual.recorded")["stats"]["last_sequence"]
            config = target.config.read_bytes()
            method = "trust.revoke" if kind.endswith("revoke") else "trust.approve"
            pin = base[0] if kind.endswith("revoke") else "a1" * 32
            with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
                mutation = pool.submit(timed, target, method, {"fingerprint": pin})
                time.sleep(.05)
                status = pool.submit(timed, target, "status.get")
                started = time.monotonic()
                call(server, "virtual.inject", {"events": [{"type": "key_down", "key": 98, "modifiers": 0, "button": 2},
                                                           {"type": "key_up", "key": 98, "modifiers": 0, "button": 2}]})
                wait(lambda: any((e.get("event") or {}).get("type") == "key_up"
                                 and (e.get("event") or {}).get("key") == 98
                                 for e in call(client, "virtual.recorded", {"since": mark})["events"]))
                input_ms = (time.monotonic() - started) * 1000
                response, mutation_ms = mutation.result(timeout=5)
                status_response, status_ms = status.result(timeout=5)
            if kind.endswith("revoke"):
                servers, clients = [p for p in servers if p != pin], [p for p in clients if p != pin]
            elif target is server:
                clients = clients + [pin]
            else:
                servers = servers + [pin]
            expected_bytes = (canonical(servers), canonical(clients))
            outcome.update(mutation_ms=mutation_ms, status_ms=status_ms, input_ms=input_ms,
                           mutation_ok="result" in response, status_ok="result" in status_response,
                           exact_role_files=all((target.dir / "tls" / n).read_bytes() == body
                                                for n, body in zip(FILES, expected_bytes)),
                           config_preserved=target.config.read_bytes() == config,
                           held_key_preserved=call(client, "virtual.recorded")["stats"]["pressed_keys"] == 1)
            require(outcome["mutation_ok"] and outcome["status_ok"] and outcome["exact_role_files"]
                    and outcome["config_preserved"] and outcome["held_key_preserved"], "large live update changed valid state")
            require(max(mutation_ms, status_ms, input_ms) < BUDGET_MS, "large live update exceeded lab budget")
            outcome["passed"] = True
        except Exception as exc:
            outcome["passed"] = False
            outcome["error_detail"] = f"{type(exc).__name__}: {exc}"
        finally:
            client.stop()
            server.stop()
            outcome["cleanup"] = (client.proc is not None and client.proc.poll() is not None
                                  and server.proc is not None and server.proc.poll() is not None
                                  and not client.socket.exists() and not server.socket.exists())
            outcome["passed"] = outcome["passed"] and outcome["cleanup"]
        results.append(outcome)

    for outcome in results:
        print(f'{"PASS" if outcome["passed"] else "FAIL"} {outcome["case"]}: {json.dumps(outcome)}', flush=True)
    report = {"binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
              "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
              "response_budget_ms": BUDGET_MS, "max_trust_file_bytes": CAP,
              "cases": results, "passed": all(r["passed"] for r in results), "evidence_dir": str(root)}
    args.report.write_text(json.dumps(report, indent=2) + "\n")
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
