#!/usr/bin/env python3
"""Measure capture-to-recorded latency through persistent real IPC connections."""
from verify_virtual import *


class IPC:
    def __init__(self,path):
        self.sock=socket.socket(socket.AF_UNIX,socket.SOCK_STREAM)
        self.sock.settimeout(3)
        self.sock.connect(str(path))
        self.seq=0
    def exact(self,n):
        out=b''
        while len(out)<n:
            part=self.sock.recv(n-len(out))
            if not part:raise RuntimeError('IPC closed')
            out+=part
        return out
    def call(self,method,params=None):
        self.seq+=1
        payload=json.dumps({'id':self.seq,'method':method,'params':params or {}}).encode()
        self.sock.sendall(struct.pack('!I',len(payload))+payload)
        while True:
            n=struct.unpack('!I',self.exact(4))[0]
            response=json.loads(self.exact(n))
            if response.get('id')==self.seq:
                if response.get('error'):raise RuntimeError(response['error'])
                return response.get('result')
    def close(self):self.sock.close()


def main():
    base=Path(tempfile.mkdtemp(prefix='oe-wp6-latency-'))
    server=Node(base,'server');client=Node(base,'client')
    try:
        server.patch({'name':'server','role':'server','tls':{'enabled':False},'server':{'port':24986,'listen':'127.0.0.1:24986'},'screen':[{'name':'server','rect':{'x':0,'y':0,'w':1920,'h':1080}},{'name':'client','rect':{'x':1920,'y':0,'w':1920,'h':1080}}]})
        client.patch({'name':'client','role':'client','tls':{'enabled':False},'client':{'remote':'127.0.0.1:24986'}})
        connected(server,client);enter(server,client)
        source=IPC(server.sock);sink=IPC(client.sock)
        try:
            last=sink.call('virtual.recorded')['stats']['last_sequence']
            times=[]
            capture_times=[]
            source_last=source.call('virtual.recorded',{'audit':True})['stats']['last_sequence']
            for index in range(1000):
                started=time.perf_counter_ns()
                source.call('virtual.inject',{'events':[{'move_abs':[200+index%1000,300]}]})
                deadline=time.monotonic()+1
                while True:
                    result=sink.call('virtual.recorded',{'since':last})
                    moves=[r for r in result['events'] if r.get('event') and r['event']['type']=='mouse_move_abs']
                    if moves:
                        last=max(r['sequence'] for r in moves)
                        received=moves[-1]['timestamp_unix_us']
                        break
                    if time.monotonic()>deadline:raise RuntimeError('move was not recorded')
                times.append((time.perf_counter_ns()-started)/1e6)
                captured=source.call('virtual.recorded',{'audit':True,'since':source_last})
                rows=[r for r in captured['events'] if r.get('event') and r['event']['type']=='mouse_move_abs' and r['detail']['kind']=='capture']
                require(rows,'capture audit record missing')
                source_last=captured['stats']['last_sequence']
                capture_times.append(max(0,received-rows[-1]['timestamp_unix_us'])/1000)
            times.sort()
            capture_times.sort()
            result={'samples':len(times),'p50_ms':times[499],'p95_ms':times[949],'p99_ms':times[989],'max_ms':times[-1],'kind':'IPC capture-to-recorded upper bound','capture_to_sink_p99_ms':capture_times[989],'capture_to_sink_p50_ms':capture_times[499]}
            time.sleep(1)
            result['server_rss_bytes']=server.status()['stats']['rss_bytes'];result['client_rss_bytes']=client.status()['stats']['rss_bytes']
            result['process_sample']=subprocess.run(['ps','-p',f'{server.proc.pid},{client.proc.pid}','-o','pid=,%cpu=,rss='],capture_output=True,text=True).stdout.strip()
            (base/'report.json').write_text(json.dumps(result,indent=2));print(json.dumps(result,indent=2));print(f'REPORT {base / "report.json"}')
            require(result['capture_to_sink_p99_ms']<1,'capture-to-sink p99 >=1 ms')
            require(result['p99_ms']<5,'capture-to-recorded p99 >=5 ms')
            require(result['client_rss_bytes']<30*1024*1024 and result['server_rss_bytes']<30*1024*1024,'idle RSS >=30 MiB')
            return 0
        finally:source.close();sink.close()
    finally:
        for node in reversed(NODES):node.stop()


if __name__=='__main__':raise SystemExit(main())
