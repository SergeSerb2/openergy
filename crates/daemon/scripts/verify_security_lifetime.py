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
            duplicate.call('connection.disconnect')
            client.call('connection.disconnect')
            cfg=base/'parent.toml';cfg.write_bytes(client.config.read_bytes())
            parent_socket=base/'parent.sock';marker=base/'parent-exit'
            helper='import os,sys,time\nwhile not os.path.exists(sys.argv[1]): time.sleep(.02)\n'
            parent=subprocess.Popen([sys.executable,'-c',helper,str(marker)])
            parentlog=open(base/'parent.stderr','wb')
            child=subprocess.Popen([str(BIN),'daemon','--platform','virtual','--config',str(cfg),'--socket',str(parent_socket),'--parent-pid',str(parent.pid)],stdout=parentlog,stderr=parentlog)
            def ctl(method,params=None):
                result=subprocess.run([str(BIN),'ctl',method,'--socket',str(parent_socket),'--json',json.dumps(params or {}),'--raw'],text=True,capture_output=True,timeout=3)
                if result.returncode:raise RuntimeError(result.stderr or result.stdout)
                return json.loads(result.stdout)
            try:
                wait(lambda:ctl('status.get')['state']=='connected')
                server.inject({'move_abs':[1000,540]})
                server.inject({'move_abs':[1920,540]},{'key_down':'a'})
                wait(lambda:ctl('virtual.recorded')['stats']['pressed_keys']>0)
                marker.touch();parent.wait(timeout=4);code=child.wait(timeout=4)
                require(code==0,f'watched daemon exit code {code}')
                wait(lambda:not parent_socket.exists(),timeout=4)
                wait(lambda:not any(p['connected'] for p in server.status()['peers']))
                require(any('peer goodbye' in line for line in server.call('logs.tail',{'lines':100})),'server did not observe CBYE')
                return {'watched_pid':parent.pid,'child_exit':code,'peer_goodbye':True,'socket_removed':True}
            finally:
                if parent.poll() is None:marker.touch();parent.wait(timeout=4)
                if child.poll() is None:child.terminate();child.wait(timeout=4)
        case('S20 parent exit watcher',parent_exit)
    finally:
        for node in reversed(NODES):node.stop()
        (base/'report.json').write_text(json.dumps(RESULTS,indent=2))
        print(f'REPORT {base / "report.json"}',flush=True)
    return 0 if RESULTS and all(r['pass'] for r in RESULTS) else 1


if __name__=='__main__':raise SystemExit(run())
