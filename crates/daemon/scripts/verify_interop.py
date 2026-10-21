#!/usr/bin/env python3
"""Openergy against real Deskflow/Barrier in an isolated copy of the lab scripts."""
from verify_virtual import *
import shutil


def main():
    base = Path(tempfile.mkdtemp(prefix='oe-wp6-interop-'))
    labroot = base/'lab'
    shutil.copytree(ROOT/'tools/interop-lab',labroot,ignore=shutil.ignore_patterns('.state','__pycache__'))
    env=dict(os.environ,LAB_NAME='oe-lab-wp6',LAB_PORT_RANGE='24950-24959')
    existing=subprocess.run(['docker','container','inspect','oe-lab-wp6'],capture_output=True,text=True)
    if existing.returncode==0:
        info=json.loads(existing.stdout)[0]
        require(info['Config']['Labels'].get('org.openergy.interop-lab')=='1','assigned container has another owner')
        require(not info['State']['Running'],'assigned lab is already running; leave it untouched')
        # Only the stopped, disposable lab container is removed. Bind-mounted artifacts remain.
        subprocess.run(['docker','rm','oe-lab-wp6'],check=True,capture_output=True)
    def lab(*args):
        result=subprocess.run([str(labroot/'lab.sh'),*args],env=env,capture_output=True,text=True,timeout=30)
        if result.returncode:
            raise RuntimeError(f'lab {args}: {result.stderr.strip()} {result.stdout.strip()}')
        return result.stdout.strip()
    def approve(node):
        for _ in range(80):
            for peer in node.status()['pending_trust']:
                node.call('trust.approve',{'fingerprint':peer['fingerprint']})
            if node.status()['state']=='connected' or any(p['connected'] for p in node.status()['peers']):
                return
            time.sleep(.1)
        raise RuntimeError(f'peer not adopted: {node.status()}')
    print(f'ARTIFACTS {base}',flush=True)
    client=Node(base,'client')
    server=Node(base,'server')
    try:
        print(lab('up'),flush=True)
        for peer in ('deskflow','barrier'):
            for tls in (False,True):
                def as_client(peer=peer,tls=tls):
                    client.call('connection.disconnect')
                    lab('server',peer,'--port','24950','--screens','lab,client','--tls','on' if tls else 'off')
                    client.patch({'name':'client','role':'client','tls':{'enabled':tls},'client':{'remote':'127.0.0.1:24950'}})
                    client.call('connection.connect')
                    approve(client)
                    before=client.ledger()['stats']['last_sequence']
                    lab('drive','mousemove','1000','540')
                    lab('drive','mousemove','1919','540','sleep','.2','mousemove_relative','--','20','0')
                    wait(lambda:client.status()['active_screen']=='client',message='lab edge CINN')
                    lab('drive','type','--clearmodifiers','oe')
                    lab('drive','click','1')
                    wait(lambda:any(r.get('event') and r['event']['type']=='mouse_up' for r in client.call('virtual.recorded',{'since':before})['events']))
                    rows=client.call('virtual.recorded',{'since':before})['events']
                    kinds=[r['event']['type'] for r in rows if r.get('event')]
                    require('key_down' in kinds and 'mouse_down' in kinds, f'missing injected input: {kinds}')
                    client.call('connection.disconnect')
                    lab('stop',peer)
                    return {'events':kinds,'tls':tls}
                case(f'I client against {peer} TLS={tls}',as_client)
        for peer in ('deskflow','barrier'):
            for tls in (False,True):
                def as_server(peer=peer,tls=tls):
                    server.call('connection.disconnect')
                    screens=[{'name':'server','rect':{'x':0,'y':0,'w':1920,'h':1080}}, {'name':'lab','rect':{'x':1920,'y':0,'w':1920,'h':1080}}]
                    server.patch({'name':'server','role':'server','screen':screens,'options':{'barrier':peer=='barrier'},'tls':{'enabled':tls},'server':{'listen':'127.0.0.1:24984','port':24984}})
                    server.call('connection.connect')
                    args=['client',peer,'--server','host.docker.internal:24984','--name','lab','--tls','on' if tls else 'off']
                    if tls:
                        args+=['--trust-fingerprint',server.status()['tls']['fingerprint']]
                    lab(*args)
                    approve(server)
                    lab('events','start')
                    server.inject({'move_abs':[1920,540]})
                    wait(lambda:server.status()['active_screen']=='lab')
                    server.inject({'move_abs':[321,234]},{'mouse_down':1},{'mouse_up':1},{'key_down':'a'},{'key_up':'a'})
                    position=wait(lambda:(p if '321' in (p:=lab('pointer')) and '234' in p else None),message='lab pointer input')
                    dump=lab('events','dump')
                    require('ButtonPress' in dump and 'KeyPress' in dump,'X11 input recorder has no click/key')
                    lab('events','stop')
                    server.call('connection.disconnect')
                    lab('stop',peer)
                    return {'pointer':position,'button_press':True,'key_press':True,'tls':tls}
                case(f'I server against {peer} TLS={tls}',as_server)
    finally:
        for node in reversed(NODES):node.stop()
        try:print(lab('down'),flush=True)
        finally:(base/'report.json').write_text(json.dumps(RESULTS,indent=2))
        print(f'REPORT {base / "report.json"}',flush=True)
    return 0 if RESULTS and all(r['pass'] for r in RESULTS) else 1


if __name__=='__main__':raise SystemExit(main())
