#!/usr/bin/env python3
"""Check user-facing connector failures with real sockets and the OS resolver."""
from verify_virtual import *
import threading


def main():
    base=Path(tempfile.mkdtemp(prefix='oe-wp6-errors-'))
    node=Node(base,'client')
    print(f'ARTIFACTS {base}',flush=True)
    listener=socket.socket();listener.setsockopt(socket.SOL_SOCKET,socket.SO_REUSEADDR,1)
    listener.bind(('127.0.0.1',24987));listener.listen();listener.settimeout(.1)
    done=threading.Event();streams=[]
    def accept():
        while not done.is_set():
            try:streams.append(listener.accept()[0])
            except socket.timeout:pass
    thread=threading.Thread(target=accept);thread.start()
    def check(remote,tls,code,message):
        node.call('connection.disconnect')
        node.patch({'name':'client','role':'client','client':{'remote':remote},'tls':{'enabled':tls}})
        error=wait(lambda:(s['last_error'] if (s:=node.status())['last_error'] and s['last_error']['code']==code else None),timeout=15,message=code)
        require(error['message']==message,f'wrong message {error}')
        require(node.status()['client']['next_retry_ms'] is not None,'missing retry timer')
        return error
    try:
        case('connect refused',lambda:check('127.0.0.1:24989',False,'connect_refused','No server is listening at 127.0.0.1:24989.'))
        case('DNS failed',lambda:check('openergy-not-a-host.invalid:24989',False,'dns_failed',"Can't find host openergy-not-a-host.invalid."))
        case('silent handshake timeout',lambda:check('127.0.0.1:24987',True,'connect_timeout','No answer from 127.0.0.1:24987.'))
    finally:
        node.stop();done.set();thread.join();listener.close()
        for stream in streams:stream.close()
        (base/'report.json').write_text(json.dumps(RESULTS,indent=2))
        print(f'REPORT {base / "report.json"}',flush=True)
    return 0 if RESULTS and all(r['pass'] for r in RESULTS) else 1


if __name__=='__main__':raise SystemExit(main())
