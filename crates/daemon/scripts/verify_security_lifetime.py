#!/usr/bin/env python3
"""TLS/duplicate/parent lifetime checks using real daemons and local IPC."""
from verify_virtual import *
import shutil
import sys


def run():
    base = Path(tempfile.mkdtemp(prefix='oe-wp6-security-'))
    print(f'ARTIFACTS {base}', flush=True)
    server = Node(base, 'server')
    client = Node(base, 'client')
    duplicate = Node(base, 'duplicate')
    screens = [{'name':'server','rect':{'x':0,'y':0,'w':1920,'h':1080}}, {'name':'client','rect':{'x':1920,'y':0,'w':1920,'h':1080}}]
    try:
        def tls():
            server.patch({'name':'server','screen':screens,'role':'server','tls':{'enabled':True},'server':{'listen':'127.0.0.1:24983','port':24983}})
            client.patch({'name':'client','role':'client','tls':{'enabled':True},'client':{'remote':'127.0.0.1:24983'}})
            held = wait(lambda: client.status()['pending_trust'])
            require(not client.status()['peers'], 'messages crossed unknown server hold')
            client.call('trust.approve', {'fingerprint':held[0]['fingerprint']})
            held2 = wait(lambda: server.status()['pending_trust'])
            server.call('trust.approve', {'fingerprint':held2[0]['fingerprint']})
            connected(server,client)
            peer = client.status()['peers'][0]
            require(peer['tls']['enabled'] and peer['tls']['trusted'], 'TLS status incorrect')
            return {'client_peer_tls':peer['tls'], 'mutual_trust_files':[str(client.dir/'tls/trusted-servers'),str(server.dir/'tls/trusted-clients')]}
        case('S6 TLS held then mutual approve', tls)

        def wrong_pin():
            fp = client.status()['peers'][0]['tls']['fingerprint']
            client.call('trust.revoke', {'fingerprint':fp})
            wait(lambda: client.status()['pending_trust'])
            require(client.status()['state'] != 'connected', 'revoked pin still connected')
            held = client.status()['pending_trust'][0]
            client.call('trust.reject', {'fingerprint':held['fingerprint']})
            wait(lambda: client.status()['state'] != 'connected')
            client.call('trust.approve', {'fingerprint':fp})
            # Approval may persist a pin before or during the next hold.
            for _ in range(30):
                for p in client.status()['pending_trust']:
                    client.call('trust.approve', {'fingerprint':p['fingerprint']})
                if client.status()['state'] == 'connected':
                    break
                time.sleep(.1)
            connected(server,client)
            return 'revocation closes peer; unknown server held; rejection releases no protocol; pin restored'
        case('S7 revoked pin held without plaintext fallback', wrong_pin)

        def same_fp():
            duplicate.stop()
            shutil.copytree(client.dir/'tls',duplicate.dir/'tls',dirs_exist_ok=True)
            duplicate.start()
            duplicate.patch({'name':'client','role':'client','tls':{'enabled':True},'client':{'remote':'127.0.0.1:24983'}})
            wait(lambda: duplicate.status()['state']=='connected')
            client.call('connection.disconnect')
            require(len([p for p in server.status()['peers'] if p['name']=='client' and p['connected']])==1,'duplicate adoption count')
            return 'healthy incumbent replaced immediately with the same certificate fingerprint'
        case('S17 same TLS fingerprint replacement',same_fp)

        def healthy_plain():
            duplicate.call('connection.disconnect')
            server.patch({'tls':{'enabled':False}})
            client.patch({'tls':{'enabled':False}})
            client.call('connection.connect')
            connected(server,client)
            duplicate.patch({'tls':{'enabled':False}})
            duplicate.call('connection.connect')
            wait(lambda: any('EBSY' in line for line in duplicate.call('logs.tail',{'lines':100})))
            require(client.status()['state']=='connected','healthy incumbent displaced')
            return 'healthy duplicate rejected with EBSY; incumbent stayed connected'
        case('S17 healthy incumbent EBSY',healthy_plain)

        def stale_plain():
            client.proc.send_signal(signal.SIGSTOP)
            try:
                time.sleep(3.2)
                duplicate.call('connection.restart')
                wait(lambda: duplicate.status()['state']=='connected')
                require(len([p for p in server.status()['peers'] if p['connected']])==1,'stale duplicate count')
            finally:
                client.proc.send_signal(signal.SIGCONT)
                client.call('connection.disconnect')
            return 'incumbent silent for one heartbeat replaced before the 3-heartbeat death deadline'
        case('S17 stale incumbent replacement',stale_plain)

        def parent_exit():
            # This parent is ours. It stays alive until an IPC-triggered filesystem marker.
            cfg = base/'parent.toml'
            off = server.call('config.get')['config']
            off['role']='client';off['name']='client';off['tls']['enabled']=False
            # A raw parent launches the real daemon; only JSON/TOML translation uses IPC.
            parent_socket=base/'parent.sock';marker=base/'parent-exit'
            helper = '''import os,subprocess,sys,time,json
p=subprocess.Popen([sys.argv[1],'daemon','--platform','virtual','--config',sys.argv[2],'--socket',sys.argv[3],'--parent-pid',str(os.getpid())])
open(sys.argv[4],'w').write(str(p.pid))
while not os.path.exists(sys.argv[5]): time.sleep(.02)
p.wait(timeout=5) if p.poll() is not None else None
'''
            # The separate config is written by a temporary daemon's config.set API.
            temp=Node(base,'parent-config')
            temp.patch({'name':'parent-client','role':'off','tls':{'enabled':False}})
            cfg.write_bytes(temp.config.read_bytes());temp.stop()
            pidfile=base/'parent.pid'
            parentlog=open(base/'parent.stderr','wb')
            parent=subprocess.Popen([sys.executable,'-c',helper,str(BIN),str(cfg),str(parent_socket),str(pidfile),str(marker)],stdout=parentlog,stderr=parentlog)
            try:
                wait(lambda: pidfile.exists() and parent_socket.exists())
                pid=int(pidfile.read_text())
                subprocess.run([str(BIN),'ctl','status.get','--socket',str(parent_socket),'--raw'],check=True,capture_output=True)
                marker.touch();parent.wait(timeout=4)
                wait(lambda: not parent_socket.exists(),timeout=4,message='parent-watched daemon exit')
                return {'parent_exit':parent.returncode,'child_pid':pid,'socket_removed':True}
            finally:
                if parent.poll() is None:
                    marker.touch();parent.wait(timeout=4)
        case('S20 parent exit watcher',parent_exit)
    finally:
        for node in reversed(NODES):node.stop()
        (base/'report.json').write_text(json.dumps(RESULTS,indent=2))
        print(f'REPORT {base / "report.json"}',flush=True)
    return 0 if RESULTS and all(r['pass'] for r in RESULTS) else 1


if __name__=='__main__':raise SystemExit(run())
