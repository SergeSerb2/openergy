#!/usr/bin/env python3
"""Exercise live trust updates under a real Unix file lock and virtual peer input."""

import argparse
import concurrent.futures
import fcntl
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import threading
import time

from verify_daemon_audit import Node, require


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--openergy", type=Path, required=True)
    parser.add_argument("--port-range", required=True)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    low, high = map(int, args.port_range.split("-"))
    require(1 <= low and low + 9 <= high <= 65535 and not low <= 24800 <= high, "unsafe port range")
    binary = args.openergy.resolve(strict=True)
    root = Path(tempfile.mkdtemp(prefix="oe-trust-contention-", dir="/tmp"))
    root.chmod(0o700)
    results = []

    def wait(fn, timeout=5):
        end = time.monotonic() + timeout
        while time.monotonic() < end:
            value = fn()
            if value:
                return value
            time.sleep(.01)
        raise RuntimeError("real peer did not reach the required state")

    def call(node, method, params=None):
        response = node.call(method, params)
        require("result" in response, f"unexpected {method} failure")
        return response["result"]

    def connected(server, client):
        wait(lambda: call(client, "status.get")["state"] == "connected")
        wait(lambda: any(p["connected"] for p in call(server, "status.get")["peers"]))

    def enter(server):
        call(server, "virtual.inject", {"events": [{"type": "mouse_move_abs", "x": 1919, "y": 540},
                                                   {"type": "mouse_move_rel", "dx": 20, "dy": 0}]})
        wait(lambda: call(server, "status.get")["active_screen"] == "client")

    def transfer(server, client, key, mark):
        started = time.monotonic()
        call(server, "virtual.inject", {"events": [{"type": "key_down", "key": key, "modifiers": 0, "button": 1},
                                                   {"type": "key_up", "key": key, "modifiers": 0, "button": 1}]})
        wait(lambda: any((r.get("event") or {}).get("type") == "key_up"
                         and (r.get("event") or {}).get("key") == key
                         for r in call(client, "virtual.recorded", {"since": mark})["events"]))
        return (time.monotonic() - started) * 1000

    def timed(node, method, params=None):
        started = time.monotonic()
        response = node.call(method, params)
        return response, (time.monotonic() - started) * 1000

    for index, kind in enumerate(("server_approve", "client_approve", "client_revoke",
                                  "held_client_approve", "held_server_approve")):
        directory = root / kind
        directory.mkdir()
        port = low + index * 2
        server = Node(directory, binary, port)
        client = Node(directory, binary, port + 1)
        tls = kind.startswith("held_")
        server.config.write_text(
            'version=1\nname="server"\nrole="server"\n'
            f'[server]\nlisten="127.0.0.1:{port}"\nport={port}\n'
            f'[tls]\nenabled={str(tls).lower()}\nrequire_client_trust={str(kind == "held_server_approve").lower()}\n'
            '[[screen]]\nname="server"\nrect={x=0.0,y=0.0,w=1920.0,h=1080.0}\n'
            '[[screen]]\nname="client"\nrect={x=1920.0,y=0.0,w=1920.0,h=1080.0}\n')
        client.config.write_text('version=1\nname="client"\nrole="client"\n'
                                 f'[client]\nremote="127.0.0.1:{port}"\n'
                                 f'[tls]\nenabled={str(tls).lower()}\n')
        outcome = {"case": kind, "passed": False}
        try:
            server.start()
            server.ready()
            client.start()
            client.ready()
            fingerprint = ":".join(["11"] * 32)
            if tls:
                pending = wait(lambda: call(client, "status.get")["pending_trust"])
                fingerprint = pending[0]["fingerprint"]
                if kind == "held_server_approve":
                    call(client, "trust.approve", {"fingerprint": fingerprint})
                    pending = wait(lambda: call(server, "status.get")["pending_trust"])
                    fingerprint = pending[0]["fingerprint"]
            else:
                connected(server, client)
                enter(server)
            target = server if kind in ("server_approve", "held_server_approve") else client
            method = "trust.revoke" if kind == "client_revoke" else "trust.approve"
            if kind == "client_revoke":
                call(target, "trust.approve", {"fingerprint": fingerprint})
            paths = [target.dir / "tls" / name for name in ("trusted-servers", "trusted-clients")]
            before = [p.read_bytes() if p.exists() else None for p in paths]
            mark = call(client, "virtual.recorded")["stats"]["last_sequence"]
            tls_dir = target.dir / "tls"
            tls_dir.mkdir(mode=0o700, exist_ok=True)
            lock_path = tls_dir / ".trust-lock"
            descriptor = os.open(lock_path, os.O_RDWR | os.O_CREAT, 0o600)
            with os.fdopen(descriptor, "r+b") as lock:
                fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
                control_status, control_status_ms = timed(target, "status.get")
                control_input_ms = None if tls else transfer(server, client, 210 + index, mark)
                control_fast = ("result" in control_status and control_status_ms < 300
                                and (control_input_ms is None or control_input_ms < 300))
                outcome.update({"lock_only_status_ms": control_status_ms,
                                "lock_only_input_ms": control_input_ms, "lock_only_control_passed": control_fast})
                # The owner remains live for 1.2 seconds. A correct daemon must
                # refuse promptly while status and established input still work.
                release = threading.Timer(1.2, lambda: fcntl.flock(lock, fcntl.LOCK_UN))
                release.start()
                try:
                    with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
                        mutation = pool.submit(timed, target, method, {"fingerprint": fingerprint})
                        time.sleep(.1)
                        status = pool.submit(timed, target, "status.get")
                        input_ms = None if tls else transfer(server, client, 110 + index, mark)
                        response, mutation_ms = mutation.result(timeout=4)
                        status_response, status_ms = status.result(timeout=4)
                        after = [p.read_bytes() if p.exists() else None for p in paths]
                        outcome.update({"mutation_ms": mutation_ms, "status_ms": status_ms,
                                        "input_ms": input_ms, "error": response.get("error", {}).get("code"),
                                        "trust_files_unchanged": before == after})
                        outcome["passed"] = (control_fast and bool(response.get("error")) and mutation_ms < 300
                                             and "result" in status_response and status_ms < 300
                                             and (input_ms is None or input_ms < 300) and before == after)
                finally:
                    release.join(timeout=2)
                    fcntl.flock(lock, fcntl.LOCK_UN)
            # Retry through the same public method after releasing the real lock.
            call(target, method, {"fingerprint": fingerprint})
            if tls:
                connected(server, client)
                enter(server)
                transfer(server, client, 120, call(client, "virtual.recorded")["stats"]["last_sequence"])
            outcome["retry_succeeded"] = True
        except Exception as exc:
            outcome["error_detail"] = f"{type(exc).__name__}: {exc}"
        finally:
            client.stop()
            server.stop()
            outcome["cleanup"] = (client.proc is not None and client.proc.poll() is not None
                                  and server.proc is not None and server.proc.poll() is not None
                                  and not client.socket.exists() and not server.socket.exists())
            outcome["passed"] = outcome["passed"] and outcome["cleanup"]
            results.append(outcome)
            print(f'{"PASS" if outcome["passed"] else "FAIL"} {kind}: {json.dumps(outcome)}', flush=True)

    # Offline commands intentionally wait and merge under the same real lock.
    # Concurrent writers must preserve every successful update in both roles.
    offline = {"case": "offline_concurrency", "passed": False}
    try:
        directory = root / "offline"
        directory.mkdir()
        identity_dir = root / "identities"
        identity_dir.mkdir()
        env = dict(os.environ, HOSTNAME="openergy-lock-lab", OPENERGY_ALLOW_PORT_24800="0",
                   OPENERGY_DISABLE_DISCOVERY="1")

        def cli(*arguments):
            value = subprocess.run([str(binary), *arguments], env=env, capture_output=True, text=True, timeout=15)
            require(value.returncode == 0, "concurrent offline CLI command failed")
            return value.stdout

        def approve(number):
            return cli("trust", "approve", f"{number:064x}", "--dir", str(directory))

        def mixed(number):
            if number < 12:
                return cli("trust", "revoke", f"{number:064x}", "--dir", str(directory))
            return approve(number + 12)

        def pins():
            values = {"server": set(), "client": set()}
            for line in cli("trust", "list", "--dir", str(directory)).splitlines():
                role, fingerprint = line.split()
                values[role].add(fingerprint.removeprefix("v2:sha256:"))
            return values

        with concurrent.futures.ThreadPoolExecutor(max_workers=24) as pool:
            list(pool.map(approve, range(24)))
            expected = {f"{n:064x}" for n in range(24)}
            require(all(value == expected for value in pins().values()), "concurrent approvals lost a pin")
            list(pool.map(mixed, range(24)))
            expected = {f"{n:064x}" for n in range(12, 36)}
            require(all(value == expected for value in pins().values()), "mixed updates lost or restored a pin")
            identities = list(pool.map(lambda _: cli("fingerprint", "--dir", str(identity_dir)).splitlines()[0],
                                       range(24)))
            require(len(set(identities)) == 1, "concurrent creators produced different identities")
        require((identity_dir / "tls" / "key.pem").stat().st_mode & 0o777 == 0o600,
                "private key permissions changed")
        offline.update({"passed": True, "initial_approvals": 24, "mixed_updates": 24,
                        "identity_creators": 24, "identical_identity": True, "pins_per_role": 24})
    except Exception as exc:
        offline["error_detail"] = f"{type(exc).__name__}: {exc}"
    results.append(offline)
    print(f'{"PASS" if offline["passed"] else "FAIL"} offline_concurrency: {json.dumps(offline)}', flush=True)
    report = {"binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
              "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
              "held_lock_ms": 1200, "response_budget_ms": 300, "cases": results,
              "passed": all(r["passed"] for r in results), "evidence_dir": str(root)}
    args.report.write_text(json.dumps(report, indent=2) + "\n")
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
