#!/usr/bin/env python3
"""Check private identity-file bounds through real CLI, startup and live TLS IPC."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import time

from verify_daemon_audit import Node, require


CAP = 1024 * 1024


def padded(original, size, prefix=False):
    require(size > len(original), "fixture must contain the original PEM")
    padding = b" " * (size - len(original) - 1) + b"\n"
    return padding + original if prefix else original + padding


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--openergy", type=Path, required=True)
    parser.add_argument("--port-range", required=True)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    low, high = map(int, args.port_range.split("-"))
    require(1 <= low and low + 7 <= high <= 65535 and not low <= 24800 <= high, "unsafe port range")
    binary = args.openergy.resolve(strict=True)
    root = Path(tempfile.mkdtemp(prefix="oe-identity-files-", dir="/tmp"))
    root.chmod(0o700)
    results = []

    def fingerprint(node, measured=False):
        command = [str(binary), "fingerprint", "--dir", str(node.dir)]
        if measured and sys.platform == "darwin":
            command = ["/usr/bin/time", "-l", *command]
        started = time.monotonic()
        value = subprocess.run(command, env=dict(node.env, HOSTNAME="openergy-pem-lab"),
                               capture_output=True, timeout=4)
        rss = re.search(rb"(\d+)\s+maximum resident set size", value.stderr) if measured else None
        return value, (time.monotonic() - started) * 1000, int(rss[1]) if rss else None

    seed = Node(root, binary, low)
    value, _, _ = fingerprint(seed)
    require(value.returncode == 0, "could not prepare a private identity")
    original = {name: (seed.dir / "tls" / name).read_bytes() for name in ("cert.pem", "key.pem")}
    expected = value.stdout.splitlines()[0]

    def prepared(port=low):
        node = Node(root, binary, port)
        tls = node.dir / "tls"
        tls.mkdir(mode=0o700)
        for name, body in original.items():
            (tls / name).write_bytes(body)
        return node

    sizes = (CAP - 1, CAP, CAP + 1, 16 * CAP)
    fixtures = [(name, size, name == "key.pem") for name in original for size in sizes]
    fixtures.append(("key.pem", CAP + 1, False))
    for name, size, prefix in fixtures:
        node = prepared()
        path = node.dir / "tls" / name
        body = padded(original[name], size, prefix)
        path.write_bytes(body)
        outcome = {"case": f'offline_{name}_{size}_{"prefix" if prefix else "suffix"}',
                   "file_bytes": size, "expected_acceptance": size <= CAP, "passed": False}
        try:
            value, elapsed, rss = fingerprint(node, measured=True)
            accepted = value.returncode == 0 and value.stdout.splitlines()[0] == expected
            refused = value.returncode != 0 and b"exceeds 1 MiB" in value.stderr
            outcome.update({"exit": value.returncode, "elapsed_ms": elapsed, "max_rss_bytes": rss,
                            "accepted_same_identity": accepted, "size_refusal": refused,
                            "files_preserved": path.read_bytes() == body and all(
                                (node.dir / "tls" / other).read_bytes() == other_body
                                for other, other_body in original.items() if other != name)})
            require(accepted if size <= CAP else refused, "identity size policy was not enforced")
            require(outcome["files_preserved"], "identity refusal changed files")
            path.write_bytes(original[name])
            retry, _, _ = fingerprint(node)
            require(retry.returncode == 0 and retry.stdout.splitlines()[0] == expected, "valid identity did not recover")
            outcome.update(passed=True, retry_succeeded=True)
        except Exception as exc:
            outcome["error_detail"] = f"{type(exc).__name__}: {exc}"
        results.append(outcome)

    for name in original:
        for kind in ("fifo", "symlink", "directory"):
            node = prepared()
            path = node.dir / "tls" / name
            saved = path.with_suffix(".saved")
            path.rename(saved)
            if kind == "fifo":
                os.mkfifo(path, 0o600)
            elif kind == "symlink":
                path.symlink_to(saved.name)
            else:
                path.mkdir()
            before = path.lstat()
            outcome = {"case": f"refuse_{name}_{kind}", "passed": False}
            try:
                value, elapsed, _ = fingerprint(node)
                require(value.returncode != 0 and b"regular file" in value.stderr, "non-regular identity accepted")
                require(path.lstat().st_ino == before.st_ino and saved.read_bytes() == original[name],
                        "refusal changed the fixture or saved identity")
                outcome.update(passed=True, elapsed_ms=elapsed, fixture_preserved=True)
            except Exception as exc:
                outcome["error_detail"] = f"{type(exc).__name__}: {exc}"
            finally:
                path.rmdir() if kind == "directory" else path.unlink()
                saved.rename(path)
            results.append(outcome)

    for index, name in enumerate(original):
        node = prepared(low + index + 5)
        path = node.dir / "tls" / name
        body = padded(original[name], CAP + 1, name == "key.pem")
        path.write_bytes(body)
        node.config.write_text(node.config.read_text().replace("enabled = false", "enabled = true"))
        outcome = {"case": f"startup_{name}", "passed": False}
        try:
            started = time.monotonic()
            node.start()
            try:
                code = node.proc.wait(timeout=.3)
            except subprocess.TimeoutExpired:
                code = None
            elapsed = (time.monotonic() - started) * 1000
            outcome.update({"exit": code, "elapsed_ms": elapsed, "socket_removed": not node.socket.exists(),
                            "size_refusal": "exceeds 1 MiB" in (node.dir / "stderr.log").read_text(),
                            "file_preserved": path.read_bytes() == body})
            require(code is not None and code != 0 and elapsed < 300 and outcome["socket_removed"]
                    and outcome["size_refusal"] and outcome["file_preserved"], "startup did not refuse oversized identity")
            node.stop()
            path.write_bytes(original[name])
            node.start()
            response = node.ready()
            require(response["result"]["tls"]["fingerprint"] == expected.decode(), "startup retry changed identity")
            outcome.update(passed=True, retry_succeeded=True)
        except Exception as exc:
            outcome["passed"] = False
            outcome["error_detail"] = f"{type(exc).__name__}: {exc}"
        finally:
            node.stop()
            path.write_bytes(original[name])
            outcome["cleanup"] = node.proc is not None and node.proc.poll() is not None and not node.socket.exists()
            outcome["passed"] = outcome["passed"] and outcome["cleanup"]
        results.append(outcome)

    node = Node(root, binary, low + 7)
    config = node.config.read_text().replace("enabled = false", "enabled = true")
    node.config.write_text(config.replace('name = "audit"', 'name = "' + "A" * CAP + '"'))
    outcome = {"case": "startup_generated_oversize", "passed": False}
    try:
        node.start()
        try:
            code = node.proc.wait(timeout=2)
        except subprocess.TimeoutExpired:
            code = None
        no_identity = not any((node.dir / "tls" / name).exists() for name in original)
        outcome.update({"exit": code, "no_identity_written": no_identity,
                        "size_refusal": "exceeds 1 MiB" in (node.dir / "stderr.log").read_text()})
        require(code is not None and code != 0 and no_identity and outcome["size_refusal"],
                "generated identity was written before size validation")
        node.stop()
        node.config.write_text(config)
        node.start()
        response = node.ready()
        require(bool(response["result"]["tls"]["fingerprint"]), "normal name did not recover identity creation")
        require(all(0 < (node.dir / "tls" / name).stat().st_size <= CAP for name in original),
                "retry wrote an invalid identity size")
        outcome.update(passed=True, retry_succeeded=True)
    except Exception as exc:
        outcome["passed"] = False
        outcome["error_detail"] = f"{type(exc).__name__}: {exc}"
    finally:
        node.stop()
        outcome["cleanup"] = node.proc is not None and node.proc.poll() is not None and not node.socket.exists()
        outcome["passed"] = outcome["passed"] and outcome["cleanup"]
    results.append(outcome)

    def call(node, method, params=None):
        response = node.call(method, params)
        require("result" in response, f"unexpected {method} failure")
        return response["result"]

    def wait(fn):
        end = time.monotonic() + 4
        while time.monotonic() < end:
            value = fn()
            if value:
                return value
            time.sleep(.01)
        raise AssertionError("real peer did not reach expected state")

    for index, name in enumerate(original):
        server = Node(root, binary, low + index * 2)
        client = Node(root, binary, low + index * 2 + 1)
        server.config.write_text('version=1\nname="server"\nrole="server"\n'
                                 f'[server]\nlisten="127.0.0.1:{server.port}"\nport={server.port}\n'
                                 '[tls]\nenabled=true\nrequire_client_trust=false\n'
                                 '[[screen]]\nname="server"\nrect={x=0.0,y=0.0,w=1920.0,h=1080.0}\n'
                                 '[[screen]]\nname="client"\nrect={x=1920.0,y=0.0,w=1920.0,h=1080.0}\n')
        client.config.write_text('version=1\nname="client"\nrole="client"\n'
                                 f'[client]\nremote="127.0.0.1:{server.port}"\n[tls]\nenabled=true\n')
        outcome = {"case": f"live_{name}", "passed": False}
        path = None
        saved = None
        try:
            server.start()
            server.ready()
            client.start()
            client.ready()
            pending = wait(lambda: call(client, "status.get")["pending_trust"])
            pin = pending[0]["fingerprint"]
            call(client, "trust.approve", {"fingerprint": pin})
            wait(lambda: call(client, "status.get")["state"] == "connected")
            call(server, "trust.approve", {"fingerprint": call(client, "status.get")["tls"]["fingerprint"]})
            call(server, "virtual.inject", {"events": [{"type": "mouse_move_abs", "x": 1919, "y": 540},
                                                       {"type": "mouse_move_rel", "dx": 20, "dy": 0}]})
            wait(lambda: call(server, "status.get")["active_screen"] == "client")
            call(server, "virtual.inject", {"events": [{"type": "key_down", "key": 97, "modifiers": 0, "button": 1}]})
            wait(lambda: call(client, "virtual.recorded")["stats"]["pressed_keys"] == 1)
            target = server if name == "cert.pem" else client
            path = target.dir / "tls" / name
            saved = path.read_bytes()
            body = padded(saved, CAP + 1)
            config = target.config.read_bytes()
            peers = [(p["name"], p["since_unix_ms"], p["connected"]) for p in call(target, "status.get")["peers"]]
            mark = call(client, "virtual.recorded")["stats"]["last_sequence"]
            path.write_bytes(body)
            method = "config.patch" if name == "cert.pem" else "connection.connect"
            params = ({"patch": {"tls": {"require_client_trust": True}}} if name == "cert.pem"
                      else {"host": "127.0.0.1", "port": server.port, "fingerprint": pin})
            started = time.monotonic()
            response = target.call(method, params)
            mutation_ms = (time.monotonic() - started) * 1000
            started = time.monotonic()
            status = call(target, "status.get")
            status_ms = (time.monotonic() - started) * 1000
            call(server, "virtual.inject", {"events": [{"type": "key_down", "key": 98, "modifiers": 0, "button": 2},
                                                       {"type": "key_up", "key": 98, "modifiers": 0, "button": 2}]})
            ledger = call(client, "virtual.recorded", {"since": mark})
            if response.get("error"):
                wait(lambda: any((r.get("event") or {}).get("type") == "key_up"
                                 and (r.get("event") or {}).get("key") == 98
                                 for r in call(client, "virtual.recorded", {"since": mark})["events"]))
                ledger = call(client, "virtual.recorded", {"since": mark})
            input_seen = any((r.get("event") or {}).get("type") == "key_up"
                             and (r.get("event") or {}).get("key") == 98 for r in ledger["events"])
            same_peers = peers == [(p["name"], p["since_unix_ms"], p["connected"]) for p in status["peers"]]
            outcome.update({"error": response.get("error", {}).get("code"), "mutation_ms": mutation_ms,
                            "status_ms": status_ms, "input_seen": input_seen,
                            "config_preserved": target.config.read_bytes() == config,
                            "file_preserved": path.read_bytes() == body, "peers_preserved": same_peers,
                            "held_key_preserved": ledger["stats"]["pressed_keys"] == 1})
            require(bool(response.get("error")) and mutation_ms < 300 and status_ms < 300 and input_seen
                    and outcome["config_preserved"] and outcome["file_preserved"] and same_peers
                    and outcome["held_key_preserved"], "live refusal did not preserve the session")
            path.write_bytes(saved)
            call(target, method, params)
            wait(lambda: call(client, "status.get")["state"] == "connected")
            require(call(client, "virtual.recorded")["stats"]["pressed_keys"] == 0, "retry retained old held input")
            require(path.read_bytes() == saved, "retry changed the valid identity")
            outcome.update(passed=True, retry_succeeded=True)
        except Exception as exc:
            outcome["passed"] = False
            outcome["error_detail"] = f"{type(exc).__name__}: {exc}"
        finally:
            if path is not None and saved is not None:
                path.write_bytes(saved)
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
              "identity_file_limit_bytes": CAP, "cases": results,
              "passed": all(r["passed"] for r in results), "evidence_dir": str(root)}
    args.report.write_text(json.dumps(report, indent=2) + "\n")
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
