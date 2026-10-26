#!/usr/bin/env python3
"""Verify identity-lock contention through real startup, TLS peers and live IPC."""

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
    require(1 <= low and low + 7 <= high <= 65535 and not low <= 24800 <= high, "unsafe port range")
    binary = args.openergy.resolve(strict=True)
    root = Path(tempfile.mkdtemp(prefix="oe-identity-contention-", dir="/tmp"))
    root.chmod(0o700)
    results = []

    def wait(fn, timeout=5):
        end = time.monotonic() + timeout
        while time.monotonic() < end:
            value = fn()
            if value:
                return value
            time.sleep(.01)
        raise RuntimeError("real peer did not reach the expected state")

    def call(node, method, params=None):
        response = node.call(method, params)
        require("result" in response, f"unexpected {method} failure")
        return response["result"]

    def timed(node, method, params=None):
        started = time.monotonic()
        response = node.call(method, params)
        return response, (time.monotonic() - started) * 1000

    def pair_ready(server, client):
        wait(lambda: call(client, "status.get")["state"] == "connected")
        wait(lambda: any(p["connected"] for p in call(server, "status.get")["peers"]))

    def enter(server):
        call(server, "virtual.inject", {"events": [{"type": "mouse_move_abs", "x": 1919, "y": 540},
                                                   {"type": "mouse_move_rel", "dx": 20, "dy": 0}]})
        wait(lambda: call(server, "status.get")["active_screen"] == "client")

    def input_probe(server, client, mark):
        started = time.monotonic()
        call(server, "virtual.inject", {"events": [{"type": "key_down", "key": 98, "modifiers": 0, "button": 2},
                                                   {"type": "key_up", "key": 98, "modifiers": 0, "button": 2}]})
        end = time.monotonic() + .5
        while time.monotonic() < end:
            ledger = call(client, "virtual.recorded", {"since": mark})
            if any((r.get("event") or {}).get("type") == "key_up"
                   and (r.get("event") or {}).get("key") == 98 for r in ledger["events"]):
                return True, (time.monotonic() - started) * 1000
            time.sleep(.01)
        return False, (time.monotonic() - started) * 1000

    def identity_bytes(node):
        return [(node.dir / "tls" / name).read_bytes() if (node.dir / "tls" / name).exists() else None
                for name in ("cert.pem", "key.pem")]

    for index, kind in enumerate(("server_patch", "client_pinned_connect")):
        directory = root / kind
        directory.mkdir()
        server = Node(directory, binary, low + index * 2)
        client = Node(directory, binary, low + index * 2 + 1)
        server.config.write_text('version=1\nname="server"\nrole="server"\n'
                                 f'[server]\nlisten="127.0.0.1:{server.port}"\nport={server.port}\n'
                                 '[tls]\nenabled=true\nrequire_client_trust=false\n'
                                 '[[screen]]\nname="server"\nrect={x=0.0,y=0.0,w=1920.0,h=1080.0}\n'
                                 '[[screen]]\nname="client"\nrect={x=1920.0,y=0.0,w=1920.0,h=1080.0}\n')
        client.config.write_text('version=1\nname="client"\nrole="client"\n'
                                 f'[client]\nremote="127.0.0.1:{server.port}"\n[tls]\nenabled=true\n')
        outcome = {"case": kind, "passed": False}
        try:
            server.start()
            server.ready()
            client.start()
            client.ready()
            pending = wait(lambda: call(client, "status.get")["pending_trust"])
            server_pin = pending[0]["fingerprint"]
            call(client, "trust.approve", {"fingerprint": server_pin})
            pair_ready(server, client)
            client_pin = call(client, "status.get")["tls"]["fingerprint"]
            call(server, "trust.approve", {"fingerprint": client_pin})
            enter(server)
            call(server, "virtual.inject", {"events": [{"type": "key_down", "key": 97, "modifiers": 0, "button": 1}]})
            wait(lambda: call(client, "virtual.recorded")["stats"]["pressed_keys"] == 1)
            target = server if kind == "server_patch" else client
            method = "config.patch" if kind == "server_patch" else "connection.connect"
            params = ({"patch": {"tls": {"require_client_trust": True}}} if kind == "server_patch"
                      else {"host": "127.0.0.1", "port": server.port, "fingerprint": server_pin})
            before_config = target.config.read_bytes()
            before_identity = identity_bytes(target)
            before_peers = [(p["name"], p["since_unix_ms"], p["connected"])
                            for p in call(target, "status.get")["peers"]]
            mark = call(client, "virtual.recorded")["stats"]["last_sequence"]
            with (target.dir / "tls" / ".identity-lock").open("r+b") as lock:
                fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
                control, control_ms = timed(target, "status.get")
                release = threading.Timer(1.2, lambda: fcntl.flock(lock, fcntl.LOCK_UN))
                release.start()
                try:
                    with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
                        mutation = pool.submit(timed, target, method, params)
                        time.sleep(.1)
                        status = pool.submit(timed, target, "status.get")
                        input_seen, input_ms = input_probe(server, client, mark)
                        response, mutation_ms = mutation.result(timeout=4)
                        status_response, status_ms = status.result(timeout=4)
                        after_peers = [(p["name"], p["since_unix_ms"], p["connected"])
                                       for p in call(target, "status.get")["peers"]]
                        held = call(client, "virtual.recorded")["stats"]["pressed_keys"]
                        outcome.update({"lock_only_status_ms": control_ms, "mutation_ms": mutation_ms,
                                        "status_ms": status_ms, "input_seen": input_seen, "input_ms": input_ms,
                                        "error": response.get("error", {}).get("code"),
                                        "config_unchanged": target.config.read_bytes() == before_config,
                                        "identity_unchanged": identity_bytes(target) == before_identity,
                                        "peers_unchanged": after_peers == before_peers, "held_key_preserved": held == 1})
                        outcome["passed"] = ("result" in control and control_ms < 300 and bool(response.get("error"))
                                             and mutation_ms < 300 and "result" in status_response and status_ms < 300
                                             and input_seen and input_ms < 300 and outcome["config_unchanged"]
                                             and outcome["identity_unchanged"] and outcome["peers_unchanged"] and held == 1)
                finally:
                    release.join(timeout=2)
                    fcntl.flock(lock, fcntl.LOCK_UN)
            call(target, method, params)
            pair_ready(server, client)
            require(identity_bytes(target) == before_identity, "retry changed the identity")
            require(call(client, "virtual.recorded")["stats"]["pressed_keys"] == 0, "reconnect retained held input")
            outcome["retry_succeeded"] = True
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
            print(f'{"PASS" if outcome["passed"] else "FAIL"} {kind}: {json.dumps(outcome)}', flush=True)

    for index, kind in enumerate(("startup_existing", "startup_missing")):
        directory = root / kind
        directory.mkdir()
        node = Node(directory, binary, low + 4 + index * 2)
        node.config.write_text(node.config.read_text().replace("enabled = false", "enabled = true"))
        outcome = {"case": kind, "passed": False}
        saved = None
        cert = node.dir / "tls" / "cert.pem"
        try:
            if kind == "startup_existing":
                value = subprocess.run([str(binary), "fingerprint", "--dir", str(node.dir)],
                                       env=dict(node.env, HOSTNAME="openergy-identity-lab"),
                                       capture_output=True, timeout=4)
                require(value.returncode == 0, "identity preparation failed")
            tls = node.dir / "tls"
            tls.mkdir(mode=0o700, exist_ok=True)
            before = identity_bytes(node)
            with (tls / ".identity-lock").open("a+b") as lock:
                fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
                if kind == "startup_existing":
                    saved = cert.with_name("cert.saved")
                    cert.rename(saved)

                def unlock():
                    if saved is not None and saved.exists():
                        saved.rename(cert)
                    fcntl.flock(lock, fcntl.LOCK_UN)

                release = threading.Timer(1.2, unlock)
                release.start()
                started = time.monotonic()
                try:
                    node.start()
                    try:
                        code = node.proc.wait(timeout=.3)
                    except subprocess.TimeoutExpired:
                        code = None
                    elapsed = (time.monotonic() - started) * 1000
                    outcome.update({"first_exit": code, "first_wait_ms": elapsed,
                                    "socket_removed": not node.socket.exists(),
                                    "busy_reported": "busy" in (node.dir / "stderr.log").read_text().lower()})
                    if kind == "startup_missing":
                        outcome["no_partial_identity"] = identity_bytes(node) == [None, None]
                    else:
                        outcome["no_partial_identity"] = not cert.exists() and identity_bytes(node)[1] == before[1]
                    outcome["passed"] = (code is not None and code != 0 and elapsed < 300
                                         and outcome["socket_removed"] and outcome["busy_reported"]
                                         and outcome["no_partial_identity"])
                finally:
                    release.join(timeout=2)
                    unlock()
            node.stop()
            node.start()
            node.ready()
            after = identity_bytes(node)
            require(all(after), "retry did not load a complete identity")
            if kind == "startup_existing":
                require(after == before, "retry replaced the original identity")
            outcome["retry_succeeded"] = True
            if kind == "startup_existing":
                node.stop()
                saved = cert.with_name("cert.partial-fixture")
                cert.rename(saved)
                node.start()
                code = node.proc.wait(timeout=2)
                outcome["partial_pair_refused"] = (code != 0 and not cert.exists()
                                                   and identity_bytes(node)[1] == before[1])
                require(outcome["partial_pair_refused"], "partial identity was replaced or accepted")
                saved.rename(cert)
        except Exception as exc:
            outcome["passed"] = False
            outcome["error_detail"] = f"{type(exc).__name__}: {exc}"
        finally:
            node.stop()
            if saved is not None and saved.exists():
                saved.rename(cert)
            outcome["cleanup"] = node.proc is not None and node.proc.poll() is not None and not node.socket.exists()
            outcome["passed"] = outcome["passed"] and outcome["cleanup"]
            results.append(outcome)
            print(f'{"PASS" if outcome["passed"] else "FAIL"} {kind}: {json.dumps(outcome)}', flush=True)
    report = {"binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
              "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
              "held_lock_ms": 1200, "response_budget_ms": 300, "cases": results,
              "passed": all(r["passed"] for r in results), "evidence_dir": str(root)}
    args.report.write_text(json.dumps(report, indent=2) + "\n")
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
