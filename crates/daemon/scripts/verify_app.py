#!/usr/bin/env python3
"""Run the app's own offscreen E2E against real isolated virtual daemons."""
from verify_virtual import *


def main():
    base=Path(tempfile.mkdtemp(prefix='oe-wp6-app-'))
    print(f'ARTIFACTS {base}',flush=True)
    helper=Node(base,'app')
    peer=Node(base,'peer')
    screens=[{'name':'peer','rect':{'x':0,'y':0,'w':1920,'h':1080}}, {'name':'app','rect':{'x':1920,'y':0,'w':1920,'h':1080}}]
    try:
        helper.patch({'role':'off','name':'app','tls':{'enabled':False},'server':{'listen':'127.0.0.1:24985','port':24985},'screen':screens})
        helper.stop()
        peer.patch({'role':'server','name':'peer','tls':{'enabled':False},'server':{'listen':'127.0.0.1:24984','port':24984},'screen':screens})
        app=ROOT/'apps/macos/.build/debug/Openergy'
        if not app.exists():subprocess.run(['swift','build'],cwd=ROOT/'apps/macos',check=True)
        env=dict(os.environ,OPENERGY_SOCKET=str(helper.sock),OPENERGY_CONFIG=str(helper.config),OPENERGY_DAEMON=str(BIN),OPENERGY_DAEMON_ARGS='--platform virtual',OPENERGY_ALLOW_PORT_24800='0')
        result=subprocess.run([str(app),'--e2e',str(base/'output'),'--join','127.0.0.1:24984','--host-port','24985'],env=env,capture_output=True,text=True,timeout=100)
        (base/'app.stdout').write_text(result.stdout)
        (base/'app.stderr').write_text(result.stderr)
        print(result.stdout,flush=True)
        require(result.returncode==0,'app E2E failed; see app.stderr')
        report=json.loads((base/'output/report.json').read_text())
        require(len(report)==8 and all(r['passed'] for r in report),'app did not pass all eight steps')
        wait(lambda:not helper.sock.exists(),message='app child daemon parent-exit cleanup')
        print(f'REPORT {base / "output/report.json"}',flush=True)
    finally:
        if helper.sock.exists():
            try:helper.call('daemon.stop')
            except Exception:pass
        for node in reversed(NODES):node.stop()
    return 0


if __name__=='__main__':raise SystemExit(main())
