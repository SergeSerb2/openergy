#!/usr/bin/env python3
"""Drive real examples through sockets and child-process interfaces."""
from pathlib import Path
import subprocess
import time

ROOT = Path(__file__).resolve().parents[3]
BIN = ROOT / "target/release/openergy"
EXAMPLES = ROOT / "target/release/examples"
ART = Path("/tmp/oe-wp4")
ART.mkdir(parents=True,exist_ok=True)

def cli(*args):
    return subprocess.check_output([str(BIN),*map(str,args)],text=True).strip()
def pair(label, server_args, client_args, wait_listen=False):
    children=[]
    paths=[]
    try:
        # Start the listener/publisher and observe actual readiness before its client.
        for name,args in [("server",server_args),("client",client_args)]:
            path=ART/f"{label}-{name}.log"
            paths.append(path)
            with path.open("w") as stream:
                p=subprocess.Popen([str(EXAMPLES/args[0]),*map(str,args[1:])],stdout=stream,stderr=subprocess.STDOUT)
            children.append(p)
            if name=="server" and wait_listen:
                deadline=time.monotonic()+3
                while "LISTENING" not in path.read_text() and p.poll() is None and time.monotonic()<deadline:
                    time.sleep(.01)
                if "LISTENING" not in path.read_text():
                    raise RuntimeError(f"listener not ready: {path.read_text()}")
        codes=[p.wait(timeout=25) for p in children]
        for path in paths:
            print(path.read_text(),end="",flush=True)
        if codes != [0,0]:
            raise RuntimeError(f"{label} processes exited {codes}")
    finally:
        for p in children:
            if p.poll() is None:
                p.terminate()
                p.wait(timeout=3)

for name,other in [("a","b"),("b","a")]:
    fp=cli("fingerprint","--dir",ART/name,"--synergy")
    cli("trust","approve",fp,"--dir",ART/other)

for example in ["guardrails","lifecycle"]:
    result=subprocess.run([str(EXAMPLES/example),str(ART/example)],text=True,capture_output=True,timeout=20)
    (ART/f"{example}.log").write_text(result.stdout+result.stderr)
    print(result.stdout+result.stderr,end="",flush=True)
    if result.returncode:
        raise RuntimeError(f"{example} exited {result.returncode}")

pair("priority",["priority_peer","server",ART/"a",24973],["priority_peer","client",ART/"b",24973],wait_listen=True)
pair("mdns",["discovery","advertise",ART/"a"],["discovery","browse",ART/"a"])
print(f"REAL_EXAMPLES=PASS artifacts={ART}",flush=True)
