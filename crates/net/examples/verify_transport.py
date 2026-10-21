#!/usr/bin/env python3
"""Run real CLI processes and real TCP/TLS sockets. No unit tests or substitutes.
Artifacts, including encrypted relay captures, remain in the printed temp dir.
"""
import os
from pathlib import Path
import re
import socket
import subprocess
import tempfile
import threading
import time

ROOT = Path(__file__).resolve().parents[3]
BIN = ROOT / "target/release/openergy"
PEER = ROOT / "target/release/examples/tls_peer"
ART = Path(tempfile.mkdtemp(prefix="oe-wp4-"))
children = []

def cli(*args):
    return subprocess.check_output([str(BIN), *map(str, args)], text=True).strip()

def identity(name):
    d = ART / name
    fp = cli("fingerprint", "--dir", d, "--synergy")
    if not re.fullmatch(r"v2:sha256:[0-9a-f]{64}", fp):
        raise RuntimeError("invalid fingerprint output")
    if cli("fingerprint", "--dir", d, "--synergy") != fp:
        raise RuntimeError("identity changed on second run")
    if (d / "tls/key.pem").stat().st_mode & 0o777 != 0o600:
        raise RuntimeError("private key mode is not 0600")
    return d, fp

def start(label, role, d, port, *flags):
    path = ART / f"{label}.log"
    with path.open("w") as stream:
        p = subprocess.Popen([str(PEER), role, "--dir", str(d), "--addr", f"127.0.0.1:{port}",
                              "--seconds", "2", *flags], stdout=stream, stderr=subprocess.STDOUT)
    children.append(p)
    return p, path

def wait_text(path, text, timeout=3):
    end = time.monotonic() + timeout
    while time.monotonic() < end:
        if path.exists() and text in path.read_text():
            return
        time.sleep(.02)
    raise RuntimeError(f"did not observe {text}: {path.read_text()}")

def finish(p, path):
    code = p.wait(timeout=6)
    text = path.read_text()
    if code:
        raise RuntimeError(f"process failed ({code}): {text}")
    return text

def require(condition, label):
    if not condition:
        raise RuntimeError(label)

try:
    a, afp = identity("a")
    b, bfp = identity("b")
    print("S21 fingerprint regex=PASS mode=0600 stable=PASS", flush=True)
    require(cli("trust", "list", "--dir", a) == "", "new trust list is not empty")
    cli("trust", "approve", bfp, "--dir", a)
    cli("trust", "approve", afp, "--dir", b)
    s, sl = start("trusted-server", "server", a, 24970)
    wait_text(sl, "LISTENING")
    c, cl = start("trusted-client", "client", b, 24970)
    st, ct = finish(s, sl), finish(c, cl)
    require("delivered=1" in st and "delivered=1" in ct, "two-way TLS frames missing")
    require("HELD" not in st + ct, "pre-seeded peer held")
    print("TLS trusted two-way frames=PASS", flush=True)

    cli("trust", "revoke", afp, "--dir", b)
    s, sl = start("held-api-server", "server", a, 24971)
    wait_text(sl, "LISTENING")
    c, cl = start("held-api-client", "client", b, 24971, "--approve-after-ms", "700")
    wait_text(cl, "HELD")
    require("FRAME" not in cl.read_text(), "held peer delivered a frame")
    st, ct = finish(s, sl), finish(c, cl)
    require("APPROVE" in ct and "delivered=1" in ct and "delivered=1" in st, "API approval failed")
    print("S6 HELD no frames; API approve -> two-way frames=PASS", flush=True)

    cli("trust", "revoke", bfp, "--dir", a)
    s, sl = start("server-held-api-server", "server", a, 24971, "--approve-after-ms", "700")
    wait_text(sl, "LISTENING")
    c, cl = start("server-held-api-client", "client", b, 24971)
    wait_text(sl, "HELD")
    require("FRAME" not in sl.read_text(), "server-held peer delivered before approval")
    st, ct = finish(s, sl), finish(c, cl)
    require("APPROVE" in st and "delivered=1" in ct and "delivered=1" in st, "server-side API approval failed")
    require(bfp in (a/"tls/trusted-clients").read_text() and bfp not in (a/"tls/trusted-servers").read_text(), "server approval wrote the wrong role")
    print("server-side HELD -> API approve; only trusted-clients updated=PASS", flush=True)

    cli("trust", "revoke", afp, "--dir", b)
    s, sl = start("held-file-server", "server", a, 24971)
    wait_text(sl, "LISTENING")
    c, cl = start("held-file-client", "client", b, 24971)
    wait_text(cl, "HELD")
    time.sleep(.6)
    require("FRAME" not in cl.read_text(), "held peer delivered before file approval")
    cli("trust", "approve", afp, "--dir", b)
    st, ct = finish(s, sl), finish(c, cl)
    require("delivered=1" in ct and "delivered=1" in st, "file approval failed")
    print("HELD trust file reload -> two-way frames=PASS", flush=True)

    s, sl = start("wrong-pin-server", "server", a, 24970)
    wait_text(sl, "LISTENING")
    capture = bytearray()
    ready = threading.Event()
    def relay():
        with socket.socket() as listener:
            listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            listener.bind(("127.0.0.1", 24972))
            listener.listen(1)
            ready.set()
            with listener.accept()[0] as down, socket.create_connection(("127.0.0.1",24970)) as up:
                def pump(source, target):
                    try:
                        while True:
                            data = source.recv(65536)
                            if not data:
                                break
                            capture.extend(data)
                            target.sendall(data)
                    except OSError:
                        pass
                    try:
                        target.shutdown(socket.SHUT_WR)
                    except OSError:
                        pass
                first = threading.Thread(target=pump,args=(down,up))
                first.start()
                pump(up,down)
                first.join(timeout=3)
    worker = threading.Thread(target=relay)
    worker.start()
    ready.wait(timeout=2)
    c, cl = start("wrong-pin-client", "client", b, 24972, "--pin", "00" * 32)
    ct = finish(c,cl)
    worker.join(timeout=3)
    # The server terminates after the failed handshake, so no long-lived listener remains.
    st = finish(s,sl)
    require("CONNECT_FAILED TlsFailure" in ct and "delivered=0" in ct, "wrong pin not refused")
    require(b"Synergy\x00\x01\x00\x08" not in capture, "plaintext application hello leaked")
    require(len(capture) > 300, "empty TLS capture")
    (ART/"wrong-pin.tls").write_bytes(capture)
    print(f"S7 wrong pin refused; captured_tls_bytes={len(capture)} plaintext_hello=absent", flush=True)

    cli("trust", "revoke", afp, "--dir", b)
    s, sl = start("expiry-server", "server", a, 24970)
    wait_text(sl, "LISTENING")
    c, cl = start("expiry-client", "client", b, 24970, "--hold-ms", "500")
    st, ct = finish(s, sl), finish(c, cl)
    require("HELD" in ct and "DISCONNECTED TlsFailure delivered=0" in ct, "held deadline failed")
    print("HELD deadline -> close with no frame=PASS", flush=True)

    s, sl = start("plain-server", "server", a, 24970, "--plain")
    wait_text(sl, "LISTENING")
    c, cl = start("plain-client", "client", b, 24970, "--plain")
    st, ct = finish(s,sl), finish(c,cl)
    require("tls=false" in ct and "delivered=1" in st and "delivered=1" in ct, "explicit plaintext failed")
    print("explicit plaintext two-way frames=PASS", flush=True)

    s, sl = start("hostile-server", "server", a, 24970, "--plain")
    wait_text(sl, "LISTENING")
    with socket.create_connection(("127.0.0.1",24970)) as sock:
        sock.sendall((4 * 1024 * 1024 + 1).to_bytes(4,"big"))
        sock.settimeout(2)
        # The legitimate server hello can already be in flight before the bad prefix.
        while sock.recv(4096):
            pass
    st = finish(s,sl)
    require("DISCONNECTED BadFrame" in st, "oversized prefix did not close")
    print("hostile oversized prefix -> BadFrame=PASS", flush=True)

    src = ART/"synergy-tls"
    src.mkdir()
    (src/"trusted-servers").write_text(afp + "\n" + afp[10:] + "\n" + "invalid\n" + "v2:sha1:" + "11"*20 + "\n")
    (src/"trusted-clients").write_text(bfp + "\n")
    imported = ART/"imported"
    result = cli("trust", "import-synergy", src, "--dir", imported)
    listing = cli("trust", "list", "--dir", imported)
    require("imported 2" in result and len(listing.splitlines()) == 2, "Synergy import/dedup failed")
    bad = subprocess.run([str(BIN),"trust","approve","invalid","--dir",str(imported)],capture_output=True)
    require(bad.returncode != 0, "invalid fingerprint accepted")
    unknown = subprocess.run([str(BIN),"fingerprint","--unknown","--dir",str(imported)],capture_output=True)
    require(unknown.returncode == 2, "unknown flag did not return usage status 2")
    print("trust import/dedup/SHA1 ignored/invalid CLI/usage status 2=PASS", flush=True)
    print(f"ARTIFACTS {ART}", flush=True)
finally:
    for p in children:
        if p.poll() is None:
            p.terminate()
            p.wait(timeout=4)
