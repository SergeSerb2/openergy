#!/usr/bin/env python3
"""Real Deskflow 1.27 TLS handshakes in the assigned isolated Docker lab."""
import os
from pathlib import Path
import subprocess
import time

ROOT = Path(__file__).resolve().parents[3]
os.chdir(ROOT)
if os.environ.get("LAB_NAME") != "oe-lab-wp4" or os.environ.get("LAB_PORT_RANGE") != "24920-24929":
    raise SystemExit("export LAB_NAME=oe-lab-wp4 LAB_PORT_RANGE=24920-24929 first")
LAB = ROOT / "tools/interop-lab/lab.sh"
BIN = ROOT / "target/release/openergy"
PEER = ROOT / "target/release/examples/tls_peer"
ART = Path("/tmp/oe-wp4/interop-final")
ART.mkdir(parents=True, exist_ok=True)

def lab(*args):
    return subprocess.check_output([str(LAB), *args], text=True)
def fingerprint(directory):
    return subprocess.check_output([str(BIN),"fingerprint","--dir",str(directory),"--synergy"],text=True).strip()
def approve(fp, directory):
    subprocess.run([str(BIN),"trust","approve",fp,"--dir",str(directory)],check=True)
def run_peer(label,*args):
    result=subprocess.run([str(PEER),*map(str,args)],text=True,capture_output=True,timeout=10)
    text=result.stdout+result.stderr
    (ART/f"{label}.log").write_text(text)
    if result.returncode:
        raise RuntimeError(text)
    return text

server_process = None
try:
    print(lab("up"),end="",flush=True)
    client=ART/"client"
    our_client_fp=fingerprint(client)
    print(lab("server","deskflow","--port","24920","--screens","lab,openergy-mac","--tls","on","--trust-fingerprint",our_client_fp),end="",flush=True)
    lab_fp=lab("fingerprint","deskflow").splitlines()[0]
    approve(lab_fp,client)
    # The real server must request/validate our client certificate before its hello.
    text=run_peer("outbound","client","--dir",client,"--addr","127.0.0.1:24920","--seconds","4","--hello-only")
    if "HELLO 0000000b53796e6572677900010008" not in text or "CONNECTED" not in text:
        raise RuntimeError(f"Deskflow hello missing: {text}")
    print("OUR_CLIENT",text,flush=True)
    logs=lab("logs","deskflow")
    tls_lines=[line for line in logs.splitlines() if any(token in line.lower() for token in ["fingerprint matches","client tls certificate info","openssl cipher:"])]
    (ART/"deskflow-server-tls.log").write_text("\n".join(tls_lines)+"\n")
    print("\n".join(tls_lines),flush=True)
    text=run_peer("wrong-pin","client","--dir",client,"--addr","127.0.0.1:24920","--seconds","3","--pin","00"*32)
    if "CONNECT_FAILED TlsFailure" not in text or "delivered=0" not in text:
        raise RuntimeError(f"wrong lab pin accepted: {text}")
    print("WRONG_PIN",text,flush=True)

    server=ART/"server"
    our_server_fp=fingerprint(server)
    approve(lab_fp,server)
    path=ART/"inbound.log"
    with path.open("w") as stream:
        server_process=subprocess.Popen([str(PEER),"server","--dir",str(server),"--addr","127.0.0.1:24974","--seconds","4"],stdout=stream,stderr=subprocess.STDOUT)
    deadline=time.monotonic()+3
    while "LISTENING" not in path.read_text() and time.monotonic()<deadline:
        time.sleep(.02)
    if "LISTENING" not in path.read_text():
        raise RuntimeError("our server did not listen")
    print(lab("client","deskflow","--server","host.docker.internal:24974","--name","lab","--tls","on","--trust-fingerprint",our_server_fp),end="",flush=True)
    if server_process.wait(timeout=7):
        raise RuntimeError(path.read_text())
    text=path.read_text()
    if "CONNECTED" not in text or "FRAME conn=2 bytes=18" not in text:
        raise RuntimeError(f"Deskflow hello-back missing: {text}")
    print("OUR_SERVER",text,flush=True)
    logs=lab("logs","deskflow")
    tls_lines=[line for line in logs.splitlines() if any(token in line.lower() for token in ["fingerprint matches","server tls certificate info","openssl cipher:"])]
    (ART/"deskflow-client-tls.log").write_text("\n".join(tls_lines)+"\n")
    print("\n".join(tls_lines),flush=True)
    print(f"DESKFLOW_TLS_BOTH_DIRECTIONS=PASS artifacts={ART}",flush=True)
finally:
    if server_process is not None and server_process.poll() is None:
        server_process.terminate()
        server_process.wait(timeout=3)
    print(lab("down"),end="",flush=True)
