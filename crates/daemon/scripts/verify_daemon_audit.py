#!/usr/bin/env python3
"""Real Unix daemon/CLI regression checks; no native input or production paths.

Run with --openergy target/debug/openergy --port-range 25120-25139.
Every process uses a disposable config, socket, TLS directory and virtual backend.
"""
import argparse
import json
import os
from pathlib import Path
import signal
import socket
import struct
import subprocess
import tempfile
import time


CAP = 16 * 1024 * 1024


def require(value, message):
    if not value:
        raise AssertionError(message)


def receive(stream):
    def exact(size):
        data = bytearray()
        while len(data) < size:
            part = stream.recv(size - len(data))
            require(part, "unexpected IPC EOF")
            data.extend(part)
        return data
    size = struct.unpack(">I", exact(4))[0]
    require(size <= CAP, "daemon sent a frame above the IPC cap")
    return json.loads(exact(size))


def frame(request):
    data = json.dumps(request).encode()
    return struct.pack(">I", len(data)) + data


class Node:
    def __init__(self, root, binary, port):
        self.dir = Path(tempfile.mkdtemp(prefix="node-", dir=root))
        self.binary = binary
        self.port = port
        self.config = self.dir / "config.toml"
        self.socket = self.dir / "control.sock"
        self.config.write_text(
            'version = 1\nname = "audit"\nrole = "off"\n'
            f'[server]\nlisten = "127.0.0.1:{port}"\nport = {port}\n'
            f'[client]\nremote = "127.0.0.1:{port}"\n[tls]\nenabled = false\n'
        )
        self.env = dict(os.environ, OPENERGY_CONFIG=str(self.config),
                        OPENERGY_CONFIG_DIR=str(self.dir), OPENERGY_TLS_DIR=str(self.dir / "tls"),
                        OPENERGY_SOCKET=str(self.socket), OPENERGY_DISABLE_DISCOVERY="1",
                        OPENERGY_ALLOW_PORT_24800="0")
        self.proc = None
        self.stderr = None

    def start(self, relative=False):
        self.stderr = open(self.dir / "stderr.log", "ab")
        self.proc = subprocess.Popen(
            [str(self.binary), "daemon", "--platform", "virtual", "--config",
             self.config.name if relative else str(self.config), "--socket",
             self.socket.name if relative else str(self.socket)],
            cwd=self.dir, env=self.env, stdout=self.stderr, stderr=self.stderr)

    def ready(self):
        deadline = time.monotonic() + 4
        while time.monotonic() < deadline:
            if self.proc.poll() is not None:
                raise AssertionError("daemon exited before IPC startup")
            try:
                return self.call("status.get")
            except OSError:
                time.sleep(.02)
        raise AssertionError("daemon did not start IPC")

    def stream(self):
        stream = socket.socket(socket.AF_UNIX)
        stream.settimeout(3)
        stream.connect(str(self.socket))
        return stream

    def call(self, method, params=None):
        with self.stream() as stream:
            stream.sendall(frame(dict(id=1, method=method, params=params or {})))
            return receive(stream)

    def cli(self, *args):
        return subprocess.run([str(self.binary), *args, "--socket", str(self.socket)],
                              cwd=self.dir, env=self.env, capture_output=True, timeout=4)

    def stop(self):
        if self.proc and self.proc.poll() is None:
            self.proc.send_signal(signal.SIGTERM)
            try:
                self.proc.wait(timeout=4)
            except subprocess.TimeoutExpired:
                self.proc.kill()
                self.proc.wait(timeout=4)
        if self.stderr:
            self.stderr.close()


def preserve_socket_file(node):
    marker = b"existing user file\n"
    node.socket.write_bytes(marker)
    node.start()
    try:
        code = node.proc.wait(timeout=2)
    except subprocess.TimeoutExpired:
        raise AssertionError("daemon replaced an existing regular file with its socket") from None
    require(code != 0, "daemon accepted a regular file as its socket path")
    require(node.socket.read_bytes() == marker, "socket bind changed the existing file")


def preserve_socket_symlink(node):
    target = node.dir / "target"
    target.write_text("existing target\n")
    node.socket.symlink_to(target)
    node.start()
    try:
        code = node.proc.wait(timeout=2)
    except subprocess.TimeoutExpired:
        raise AssertionError("daemon replaced an existing symlink with its socket") from None
    require(code != 0 and node.socket.is_symlink(), "socket bind must refuse a symlink")
    require(target.read_text() == "existing target\n", "socket bind changed the symlink target")


def relative_paths(node):
    node.start(relative=True)
    require(node.ready()["result"]["role"] == "off", "relative socket/config paths failed")
    result = node.call("config.patch", {"patch": {"clipboard": {"enabled": False}}})
    require("result" in result, "config save failed for a relative config path")


def replaced_socket_shutdown(node):
    node.start()
    node.ready()
    node.socket.unlink()
    node.socket.write_text("replacement belongs to caller\n")
    node.proc.terminate()
    try:
        code = node.proc.wait(timeout=3)
    except subprocess.TimeoutExpired:
        raise AssertionError("daemon shutdown hangs after its socket path is replaced") from None
    require(code == 0, "daemon failed clean shutdown")
    require(node.socket.read_text() == "replacement belongs to caller\n", "shutdown removed a replacement file")


def log_symlink(node):
    target = node.dir / "unrelated-file"
    target.write_text("preserve this\n")
    (node.dir / "openergy.log").symlink_to(target)
    node.start()
    node.ready()
    node.stop()
    require(target.read_text() == "preserve this\n", "logger followed a symlink and changed an unrelated file")


def log_permissions(node):
    log = node.dir / "openergy.log"
    log.write_text("")
    log.chmod(0o644)
    node.start()
    node.ready()
    require(log.stat().st_mode & 0o777 == 0o600, "existing log remains readable by other users")


def ipc_framing(node):
    node.start()
    node.ready()
    with node.stream() as stream:
        first = frame(dict(id=11, method="invalid.method", params={}))
        for part in [first[:1], first[1:3], first[3:9], first[9:]]:
            stream.sendall(part)
            time.sleep(.01)
        require(receive(stream)["error"]["code"] == "unknown_method", "unknown method response changed")
        stream.sendall(frame(dict(id=12, method="status.get")) + frame(dict(id=13, method="config.get")))
        require([receive(stream)["id"], receive(stream)["id"]] == [12, 13], "pipelined request order changed")
    for bad in [struct.pack(">I", CAP + 1), struct.pack(">I", 0), frame({"id": "invalid"})]:
        with node.stream() as stream:
            stream.sendall(bad)
            require(stream.recv(1) == b"", "malformed IPC request was not closed")
    require("result" in node.call("status.get"), "malformed clients stopped the daemon")


def stale_socket_and_single_instance(node):
    with socket.socket(socket.AF_UNIX) as stale:
        stale.bind(str(node.socket))
    node.start()
    before = node.ready()["result"]["daemon"]["pid"]
    duplicate = subprocess.run([str(node.binary), "daemon", "--platform", "virtual", "--config",
                                str(node.config), "--socket", str(node.socket)],
                               cwd=node.dir, env=node.env, capture_output=True, timeout=3)
    require(duplicate.returncode == 1, "duplicate daemon did not refuse the owned socket")
    require(node.call("status.get")["result"]["daemon"]["pid"] == before, "duplicate daemon damaged the listener")


def connecting_idempotence(node):
    peer = Node(node.dir.parent, node.binary, node.port + 1)
    peer.config.write_text(peer.config.read_text().replace('role = "off"', 'role = "client"')
                           .replace(f'remote = "127.0.0.1:{peer.port}"', f'remote = "127.0.0.1:{node.port}"'))
    streams = []
    with socket.socket() as listener:
        listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        listener.bind(("127.0.0.1", node.port))
        listener.listen()
        listener.settimeout(3)
        try:
            peer.start()
            peer.ready()
            first, _ = listener.accept()
            first.close()
            second, _ = listener.accept()
            streams.append(second)
            status = peer.call("status.get")["result"]
            require(status["state"] == "reconnecting" and status["client"]["next_retry_ms"] is None,
                    "did not reach a reconnect handshake")
            for _ in range(3):
                require("result" in peer.call("connection.connect"), "connect command failed")
            listener.settimeout(.2)
            try:
                unexpected, _ = listener.accept()
                streams.append(unexpected)
            except socket.timeout:
                return
            raise AssertionError("connect opened duplicate TCP sessions during a retry handshake")
        finally:
            peer.stop()
            for stream in streams:
                stream.close()


def oversized_response(node):
    node.start()
    node.ready()
    result = node.call("virtual.clipboard", {"dib_bytes": 12 * 1024 * 1024})
    require("result" in result, "virtual clipboard fixture was rejected")
    with node.stream() as stream:
        stream.sendall(frame(dict(id=21, method="virtual.clipboard", params={"get": True})))
        result = receive(stream)
        require(result.get("error", {}).get("code") == "response_too_large", "large response needs a framed error")
        stream.sendall(frame(dict(id=22, method="status.get")))
        require(receive(stream)["id"] == 22, "oversized response broke the control connection")


def cli_validation(node):
    node.start()
    node.ready()
    for args in [("status", "--foreground"), ("status", "extra"),
                 ("ctl", "status.get", "--foreground"), ("ctl", "status.get", "extra"),
                 ("ctl", "status.get", "--json", "--raw"),
                 ("ctl", "status.get", "--raw", "--raw"),
                 ("ctl", "status.get", "--json", "{}", "--json", "{}"),
                 ("status", "--socket", str(node.socket))]:
        result = node.cli(*args)
        require(result.returncode == 2, f"CLI did not reject invalid arguments: {args}")
    require(node.cli("status").returncode == 0, "normal status CLI failed")


def daemon_argument_validation(node):
    for command, extra in [("daemon", ["extra"]), ("daemon", ["--platform", "virtual"]),
                           ("client", ["--remote", f"127.0.0.1:{node.port} "])]:
        args = [str(node.binary), command, "--platform", "virtual", "--config", str(node.config),
                "--socket", str(node.socket), *extra]
        try:
            result = subprocess.run(args, cwd=node.dir, env=node.env, capture_output=True, timeout=1)
        except subprocess.TimeoutExpired:
            raise AssertionError("daemon accepted invalid or duplicate startup arguments") from None
        require(result.returncode != 0, "daemon accepted invalid startup overrides")
        require(not node.socket.exists(), "invalid startup arguments created an IPC listener")


def cli_connect_host(node):
    node.start()
    node.ready()
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", node.port + 1))
        listener.listen()
        listener.settimeout(3)
        remote = f"127.0.0.1:{node.port + 1}"
        result = node.cli("connect", remote)
        require(result.returncode == 0, "connect HOST rejected the documented host argument")
        connection, _ = listener.accept()
        with connection:
            config = node.call("config.get")["result"]["config"]
            require(config["role"] == "client" and config["client"]["remote"] == remote,
                    "connect HOST did not apply the requested destination")
            require(remote in node.config.read_text(), "connect HOST did not persist its destination")
            require(node.cli("connect", remote, "extra").returncode == 2, "connect accepted an extra argument")


def pinned_connect_rollback(node, revoke=False):
    peer = Node(node.dir.parent, node.binary, node.port + 1)
    node.config.write_text(node.config.read_text().replace('role = "off"', 'role = "server"')
                           .replace('name = "audit"', 'name = "server"')
                           .replace('enabled = false', 'enabled = true\nrequire_client_trust = false')
                           + '\n[[screen]]\nname = "server"\nrect = {x=0.0,y=0.0,w=1920.0,h=1080.0}\n'
                             '[[screen]]\nname = "client"\nrect = {x=1920.0,y=0.0,w=1920.0,h=1080.0}\n')
    peer.config.write_text(peer.config.read_text().replace('role = "off"', 'role = "client"')
                           .replace('name = "audit"', 'name = "client"')
                           .replace(f'remote = "127.0.0.1:{peer.port}"', f'remote = "127.0.0.1:{node.port}"')
                           .replace('enabled = false', 'enabled = true'))
    def wait_status(target, predicate):
        deadline = time.monotonic() + 4
        while time.monotonic() < deadline:
            status = target.call("status.get")["result"]
            if predicate(status):
                return status
            time.sleep(.02)
        raise AssertionError("TLS pair did not reach expected status")
    try:
        node.start()
        node.ready()
        peer.start()
        peer.ready()
        pending = wait_status(peer, lambda s: len(s["pending_trust"]) == 1)
        fingerprint = pending["pending_trust"][0]["fingerprint"]
        require("result" in peer.call("trust.approve", {"fingerprint": fingerprint}), "TLS approval failed")
        before = wait_status(peer, lambda s: s["state"] == "connected")
        if revoke:
            require("result" in peer.call("connection.connect", {"host": "127.0.0.1", "port": node.port,
                                                                  "fingerprint": fingerprint}), "pinned connect failed")
            wait_status(peer, lambda s: s["state"] == "connected")
            wait_status(node, lambda s: any(p["connected"] for p in s["peers"]))
            node.call("virtual.inject", {"events": [{"type": "mouse_move_abs", "x": 1919, "y": 540},
                                                    {"type": "mouse_move_rel", "dx": 20, "dy": 0}]})
            wait_status(node, lambda s: s["active_screen"] == "client")
            node.call("virtual.inject", {"events": [{"type": "key_down", "key": 97, "modifiers": 0, "button": 1}]})
            deadline = time.monotonic() + 3
            while time.monotonic() < deadline:
                if peer.call("virtual.recorded")["result"]["stats"]["pressed_keys"] == 1:
                    break
                time.sleep(.01)
            else:
                raise AssertionError("key was not held before revocation")

            files = [peer.dir / "tls" / name for name in ("trusted-servers", "trusted-clients")]
            original = [p.read_bytes() if p.exists() else None for p in files]

            def valid_peer_preserved():
                require(peer.call("status.get")["result"]["state"] == "connected", "valid peer was disconnected")
                require(peer.call("virtual.recorded")["result"]["stats"]["pressed_keys"] == 1,
                        "valid peer's held input was released")
                require([p.read_bytes() if p.exists() else None for p in files] == original,
                        "failed or unrelated revocation changed trust files")

            require("result" in peer.call("trust.revoke", {"fingerprint": ":".join(["00"] * 32)}),
                    "unrelated fingerprint revoke failed")
            valid_peer_preserved()
            # A real nonempty lock directory forces persistence refusal without
            # changing pins or using a mock. Every path belongs to this lab node.
            lock = peer.dir / "tls" / ".trust-lock"
            saved = lock.with_name(".trust-lock.saved")
            lock.rename(saved)
            lock.mkdir()
            marker = lock / "occupied"
            marker.write_text("owned refusal fixture")
            try:
                response = peer.call("trust.revoke", {"fingerprint": fingerprint})
                require(response.get("error", {}).get("code") == "trust_failed", "save refusal was not reported")
                valid_peer_preserved()
            finally:
                marker.unlink()
                lock.rmdir()
                saved.rename(lock)
            require("result" in peer.call("trust.revoke", {"fingerprint": fingerprint}), "trust revoke failed")
            pending = wait_status(peer, lambda s: len(s["pending_trust"]) == 1)
            require(pending["pending_trust"][0]["fingerprint"] == fingerprint, "revoked identity did not need approval")
            require(not any(p["connected"] for p in pending["peers"]), "revoked identity remained connected")
            require(not peer.call("trust.list")["result"]["servers"], "revoked server remained in the trust store")
            require(peer.call("virtual.recorded")["result"]["stats"]["pressed_keys"] == 0,
                    "successful revocation retained held input")
            return
        original = peer.config.read_bytes()
        (peer.dir / f'config.toml.{before["daemon"]["pid"]}.tmp').write_text("occupied temporary file")
        response = peer.call("connection.connect", {"host": "127.0.0.1", "port": node.port,
                                                    "fingerprint": fingerprint})
        require(response.get("error", {}).get("code") == "connect_failed", "config-save failure was not reported")
        after = peer.call("status.get")["result"]
        require(after["state"] == "connected" and after["peers"] == before["peers"],
                "failed pinned connect destroyed the existing TLS session")
        require(peer.config.read_bytes() == original, "failed pinned connect changed the config")
    finally:
        peer.stop()


def pinned_trust_revoke(node):
    pinned_connect_rollback(node, revoke=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--openergy", type=Path, required=True)
    parser.add_argument("--port-range", required=True)
    parser.add_argument("--report", type=Path)
    args = parser.parse_args()
    low, high = map(int, args.port_range.split("-"))
    require(1 <= low < high <= 65535 and not low <= 24800 <= high, "unsafe port range; at least two ports required")
    binary = args.openergy.resolve(strict=True)
    results = []
    with tempfile.TemporaryDirectory(prefix="oe-daemon-audit-", dir="/tmp") as root:
        for check in [preserve_socket_file, preserve_socket_symlink, relative_paths, replaced_socket_shutdown,
                      log_symlink, log_permissions, ipc_framing, oversized_response, cli_validation,
                      pinned_connect_rollback, stale_socket_and_single_instance, connecting_idempotence,
                      cli_connect_host, pinned_trust_revoke, daemon_argument_validation]:
            node = Node(root, binary, low)
            try:
                check(node)
                results.append(dict(check=check.__name__, passed=True))
                print(f"PASS {check.__name__}", flush=True)
            except Exception as error:
                # Error messages are fixed descriptions; no private paths or payloads.
                results.append(dict(check=check.__name__, passed=False, error=str(error)))
                print(f"FAIL {check.__name__}: {error}", flush=True)
            finally:
                node.stop()
    if args.report:
        args.report.write_text(json.dumps(results, indent=2) + "\n")
    return 0 if all(result["passed"] for result in results) else 1


if __name__ == "__main__":
    raise SystemExit(main())
