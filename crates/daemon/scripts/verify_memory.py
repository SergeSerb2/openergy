#!/usr/bin/env python3
"""Separate idle RSS and retained 8 MiB clipboard RSS checks."""
from verify_virtual import *


def main():
    base=Path(tempfile.mkdtemp(prefix='oe-wp6-memory-'))
    server=Node(base,'server');client=Node(base,'client')
    print(f'ARTIFACTS {base}',flush=True)
    screens=[{'name':'server','rect':{'x':0,'y':0,'w':1920,'h':1080}},{'name':'client','rect':{'x':1920,'y':0,'w':1920,'h':1080}}]
    try:
        server.patch({'role':'server','name':'server','screen':screens,'tls':{'enabled':False},'server':{'listen':'127.0.0.1:24988','port':24988}})
        client.patch({'role':'client','name':'client','tls':{'enabled':False},'client':{'remote':'127.0.0.1:24988'}})
        connected(server,client);enter(server,client)
        baseline={n.dir.name:n.status()['stats']['rss_bytes'] for n in (server,client)}
        require(all(r<30_000_000 for r in baseline.values()),f'idle RSS {baseline}')
        server.call('virtual.clipboard',{'dib_bytes':8*1024*1024})
        wait(lambda:client.call('virtual.clipboard',{'summary':True})['bytes']==8*1024*1024)
        time.sleep(1)
        retained={n.dir.name:n.status()['stats']['rss_bytes'] for n in (server,client)}
        report={'baseline_rss_bytes':baseline,'retained_rss_bytes':retained,'payload_bytes':8*1024*1024,'limit':'baseline + 2 * payload'}
        print(json.dumps(report,indent=2),flush=True)
        (base/'report.json').write_text(json.dumps(report,indent=2))
        for n in (server,client):
            result=subprocess.run(['vmmap','-summary',str(n.proc.pid)],capture_output=True,text=True)
            (n.dir/'vmmap.txt').write_text(result.stdout+result.stderr)
        require(all(retained[k]<=v+2*8*1024*1024 for k,v in baseline.items()),'retained RSS exceeds baseline + 2 payloads')
        print(f'PASS idle and retained memory; REPORT {base / "report.json"}',flush=True)
    finally:
        for n in reversed(NODES):n.stop()
    return 0


if __name__=='__main__':raise SystemExit(main())
