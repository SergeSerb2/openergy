#!/usr/bin/env python3
"""Exercise real reference peers over TCP/TLS and X11. No host input APIs."""
import datetime
import hashlib
import json
import pathlib
import socket
import ssl
import struct
import subprocess
import threading
import time

ROOT = pathlib.Path(__file__).resolve().parent
LAB = str(ROOT / "lab.sh")
REPORT = {"verified_at": datetime.datetime.now(datetime.timezone.utc).isoformat(), "peers": {}}


def run(*args, check=True):
    result = subprocess.run(args, check=False, capture_output=True, text=True, timeout=25)
    if result.returncode and check:
        raise RuntimeError("command failed: " + repr(args) + "\n" + result.stdout + result.stderr)
    return result


def lab(*args):
    result = run(LAB, *args)
    if result.stdout:
        print(result.stdout.strip(), flush=True)
    return result.stdout


def exact(conn, count):
    data = b""
    while len(data) < count:
        part = conn.recv(count - len(data))
        if not part:
            raise EOFError("peer closed before the complete frame arrived")
        data += part
    return data


def receive(conn):
    head = exact(conn, 4)
    size = struct.unpack("!I", head)[0]
    if size > 65536:
        raise ValueError("unexpected frame length")
    return head + exact(conn, size)


def send(conn, payload):
    conn.sendall(struct.pack("!I", len(payload)) + payload)


def probe(tls=False):
    with socket.create_connection(("127.0.0.1", 24900), timeout=6) as raw:
        if not tls:
            return {"hello": receive(raw).hex()}
        # The inherited peer accept path can race an immediate ClientHello.
        time.sleep(.1)
        # Lab certificate identity is checked by SHA-256 below.
        with ssl._create_unverified_context().wrap_socket(raw, server_hostname="lab") as conn:
            return {"hello": receive(conn).hex(), "tls": conn.version(),
                    "sha256": hashlib.sha256(conn.getpeercert(binary_form=True)).hexdigest()}


def client_round(impl, tls=False, wrong_pin=False):
    """A host protocol listener drives the real Linux peer on Xvfb."""
    result, errors = {}, []
    barrier = impl == "barrier"
    hello = (b"Barrier" if barrier else b"Synergy") + struct.pack("!HH", 1, 6 if barrier else 8)
    pem = ROOT / ".state/deskflow/tls/deskflow.pem"
    context = None
    if tls:
        context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        context.load_cert_chain(str(pem))
    with socket.socket() as listener:
        listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        # OrbStack forwards host.docker.internal to a host interface, not :99.
        listener.bind(("0.0.0.0", 24901))
        listener.listen(1)
        listener.settimeout(12)

        def serve():
            try:
                raw, address = listener.accept()
                with raw:
                    raw.settimeout(8)
                    conn = context.wrap_socket(raw, server_side=True) if context else raw
                    with conn:
                        result["source"] = address[0]
                        if context:
                            result["tls"] = conn.version()
                        send(conn, hello)
                        if wrong_pin:
                            conn.settimeout(2)
                        try:
                            hello_back = receive(conn)
                        except (EOFError, ConnectionResetError, ssl.SSLError, socket.timeout):
                            if wrong_pin:
                                result["pin_rejected"] = True
                                return
                            raise
                        if wrong_pin:
                            raise AssertionError("client accepted a wrong TLS fingerprint")
                        result["hello_back"] = hello_back.hex()
                        assert hello_back == struct.pack("!I", 18) + hello + struct.pack("!I", 3) + b"lab"
                        send(conn, b"QINF")
                        info = receive(conn)
                        assert info[4:8] == b"DINF", info.hex()
                        result["screen_info"] = info.hex()
                        send(conn, b"CIAK")
                        send(conn, b"CROP")
                        send(conn, b"DSOP" + struct.pack("!I", 0))
                        time.sleep(.2)
                        send(conn, b"CINN" + struct.pack("!HHIH", 300, 200, 1, 0))
                        send(conn, b"DMMV" + struct.pack("!HH", 321, 234))
                        send(conn, b"DMDN\x01")
                        send(conn, b"DMUP\x01")
                        time.sleep(.3)
                        result["pointer"] = run(LAB, "pointer").stdout.strip()
                        assert "X=321\nY=234" in result["pointer"], result
                        # Leave the client screen before closing the protocol session.
                        send(conn, b"COUT")
            except Exception as error:
                errors.append(error)

        worker = threading.Thread(target=serve, daemon=True)
        worker.start()
        args = ["client", impl, "--server", "host.docker.internal:24901", "--name", "lab",
                "--tls", "on" if tls else "off"]
        if tls:
            args += ["--trust-fingerprint", "0" * 64 if wrong_pin else REPORT["peers"]["deskflow"]["tls_server"]["sha256"]]
        try:
            lab(*args)
            worker.join(15)
            if worker.is_alive():
                raise TimeoutError("host listener did not finish")
            if errors:
                raise errors[0]
            if wrong_pin:
                log = run(LAB, "logs", impl).stdout.lower()
                assert "fingerprint does not match" in log, log[-2000:]
        finally:
            lab("stop", impl)
        return result


def server_edge(impl, magic, minor):
    """Join the real server as a wire probe, then move Xvfb input off its edge."""
    with socket.create_connection(("127.0.0.1", 24900), timeout=3) as conn:
        assert receive(conn)[4:] == magic + struct.pack("!HH", 1, minor)
        name = b"openergy-mac"
        send(conn, magic + struct.pack("!HHI", 1, minor, len(name)) + name)
        messages = []
        for _ in range(12):
            payload = receive(conn)[4:]
            messages.append(payload[:4].decode(errors="replace"))
            if payload == b"QINF":
                send(conn, b"DINF" + struct.pack("!7H", 0, 0, 1920, 1080, 0, 960, 540))
            if payload[:4] == b"DSOP":
                break
        else:
            raise AssertionError("server did not complete option exchange")
        # Absolute XWarpPointer alone does not produce XI2 RawMotion. Finish
        # with XTEST relative motion so both real servers observe the crossing.
        lab("drive", "mousemove", "960", "540", "sleep", ".2", "mousemove", "1919", "540",
            "sleep", ".2", "mousemove_relative", "--", "20", "0")
        for _ in range(12):
            payload = receive(conn)[4:]
            messages.append(payload[:4].decode(errors="replace"))
            if payload == b"CALV":
                send(conn, b"CALV")
            if payload[:4] == b"CINN":
                return {"right_edge_enter": True, "wire_messages": messages}
        raise AssertionError("server did not enter the right client")


def main():
    lab("up")
    try:
        REPORT["architecture"] = run("docker", "exec", "openergy-interop", "uname", "-m").stdout.strip()
        REPORT["image"] = run("docker", "image", "inspect", "openergy-interop:latest", "--format", "{{.Id}}").stdout.strip()
        for impl, magic, minor in [("deskflow", b"Synergy", 8), ("barrier", b"Barrier", 6)]:
            peer = REPORT["peers"][impl] = {}
            lab("server", impl, "--port", "24900", "--screens", "lab,openergy-mac", "--tls", "off")
            again = lab("server", impl, "--port", "24900", "--screens", "lab,openergy-mac", "--tls", "off")
            assert "already running" in again
            peer["plain_server"] = probe()
            expected = (struct.pack("!I", 11) + magic + struct.pack("!HH", 1, minor)).hex()
            assert peer["plain_server"]["hello"] == expected
            print(impl, "server hello:", expected, flush=True)
            peer["server_edge"] = server_edge(impl, magic, minor)
            lab("server", impl, "--port", "24900", "--tls", "on")
            peer["tls_server"] = probe(tls=True)
            assert peer["tls_server"]["hello"] == expected
            fingerprint = run(LAB, "fingerprint", impl).stdout.splitlines()
            assert fingerprint[0] == "v2:sha256:" + peer["tls_server"]["sha256"]
            peer["fingerprint"] = fingerprint
            # An incomplete hello-only TLS session can leave Barrier 2.4's next
            # SSL_accept retrying. Give the independent OpenSSL check a fresh peer.
            lab("stop", impl)
            lab("server", impl, "--port", "24900", "--tls", "on")
            for attempt in range(3):
                try:
                    openssl = subprocess.run(["openssl", "s_client", "-4", "-tls1_2", "-connect", "localhost:24900"],
                                             input=b"", capture_output=True, timeout=4)
                    break
                except subprocess.TimeoutExpired:
                    if attempt == 2:
                        raise
                    print(impl, "OpenSSL TLS accept stalled; retrying", flush=True)
                    time.sleep(.2)
            peer["openssl_attempts"] = attempt + 1
            transcript = openssl.stdout.decode(errors="replace") + openssl.stderr.decode(errors="replace")
            assert openssl.returncode == 0, transcript
            assert "TLSv1.2" in transcript or "TLSv1.3" in transcript, transcript
            peer["openssl_handshake_exit"] = openssl.returncode
            peer["openssl_tls"] = "TLSv1.2"
            # Retain handshake evidence without session keys or ticket dumps.
            (ROOT / ".state" / (impl + "-s_client.log")).write_text("\n".join(
                line for line in transcript.splitlines()
                if any(marker in line for marker in ["Protocol  :", "Cipher    :", "Verify return code:",
                                                     "handshake has read", "subject=", "issuer=", "DONE"])
            ) + "\n")
            lab("stop", impl)

        # A wildcard host listener cannot share a port with a Docker localhost
        # publication. Reserve 24901 for the host by publishing only 24900 here.
        lab("down")
        print(run("env", "LAB_PORT_RANGE=24900-24900", LAB, "up").stdout.strip(), flush=True)
        lab("events", "start")
        lab("events", "start")
        lab("drive", "mousemove", "120", "220", "click", "1", "type", "--clearmodifiers", "virtual-only")
        assert "X=120\nY=220" in lab("pointer")
        for impl in ["deskflow", "barrier"]:
            peer = REPORT["peers"][impl]
            peer["plain_client"] = client_round(impl)
            peer["tls_client"] = client_round(impl, tls=True)
            peer["wrong_pin"] = client_round(impl, tls=True, wrong_pin=True)
            print(impl, "client hello-back:", peer["plain_client"]["hello_back"], flush=True)
        lab("events", "stop")
        lab("events", "stop")
        event_log = run(LAB, "events", "dump").stdout
        assert "Motion" in event_log and "ButtonPress" in event_log and "KeyPress" in event_log
        REPORT["events"] = {"motion": event_log.count("Motion"), "button_press": event_log.count("ButtonPress"),
                            "key_press": event_log.count("KeyPress"), "wire_pointer": "321,234"}
        lab("clipboard", "set", "Openergy virtual clipboard ✓")
        assert run(LAB, "clipboard", "get").stdout == "Openergy virtual clipboard ✓"
        REPORT["clipboard"] = "UTF-8 set/get passed"
        for args in [("server", "deskflow", "--port", "24800"),
                     ("client", "deskflow", "--server", "192.0.2.10:24901")]:
            assert run(LAB, *args, check=False).returncode != 0
        REPORT["safety_rejections"] = "port 24800 and non-host server address rejected before launch"
        REPORT["passed"] = True
    finally:
        lab("down")
        state = run("docker", "inspect", "-f", "{{.State.Running}}", "openergy-interop").stdout.strip()
        REPORT["container_running_at_end"] = state
        assert state == "false"
        (ROOT / "verification.json").write_text(json.dumps(REPORT, indent=2) + "\n")
    print("All Docker protocol, TLS, host networking, X input, clipboard, and rejection checks passed.")


if __name__ == "__main__":
    main()
