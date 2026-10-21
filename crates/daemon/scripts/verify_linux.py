#!/usr/bin/env python3
"""Real X11 daemon -> virtual daemon; only inside the isolated Xvfb container."""
from verify_virtual import *
import ctypes


class XNode(Node):
    def start(self):
        self.proc=subprocess.Popen([str(BIN),'daemon','--platform','x11','--config',str(self.config),'--socket',str(self.sock)],stdout=self.log,stderr=self.log,env=dict(os.environ,OPENERGY_ALLOW_PORT_24800='0'))
        wait(lambda:self.call('status.get'),message='X11 daemon startup')


def main():
    require(Path('/.dockerenv').exists() and os.environ.get('DISPLAY')==':99' and os.environ.get('OPENERGY_X11_PROBE_ISOLATED')=='1','requires isolated Docker Xvfb')
    base=Path(tempfile.mkdtemp(prefix='oe-wp6-linux-'))
    server=XNode(base,'server');client=Node(base,'client');clip=None
    try:
        screens=[{'name':'server','rect':{'x':0,'y':0,'w':1280,'h':800}},{'name':'client','rect':{'x':1280,'y':0,'w':1920,'h':1080}}]
        server.patch({'role':'server','name':'server','screen':screens,'tls':{'enabled':False},'server':{'listen':'127.0.0.1:24988','port':24988}})
        client.patch({'role':'client','name':'client','tls':{'enabled':False},'client':{'remote':'127.0.0.1:24988'}})
        connected(server,client)
        require(server.status()['daemon']['platform']=='x11','native backend not selected')
        xlib=ctypes.CDLL('libX11.so.6');xtest=ctypes.CDLL('libXtst.so.6')
        xlib.XOpenDisplay.restype=ctypes.c_void_p
        display=xlib.XOpenDisplay(None)
        require(display,'isolated X display unavailable')
        xlib.XFlush.argtypes=[ctypes.c_void_p]
        xtest.XTestFakeMotionEvent.argtypes=[ctypes.c_void_p,ctypes.c_int,ctypes.c_int,ctypes.c_int,ctypes.c_ulong]
        xtest.XTestFakeRelativeMotionEvent.argtypes=[ctypes.c_void_p,ctypes.c_int,ctypes.c_int,ctypes.c_ulong]
        xtest.XTestFakeMotionEvent(display,-1,600,400,0);xlib.XFlush(display);time.sleep(.1)
        xtest.XTestFakeRelativeMotionEvent(display,700,0,0);xlib.XFlush(display)
        print('X11 geometry',server.status()['local_screen'],flush=True)
        wait(lambda:client.status()['active_screen']=='client',message='native X11 edge capture')
        subprocess.run(['xdotool','key','a','click','1'],check=True)
        wait(lambda: {'key_down','key_up','mouse_down','mouse_up'}<=set(e['type'] for e in events(client)),message='X11 capture forwarded')
        clip=subprocess.Popen(['xclip','-selection','clipboard','-in'],stdin=subprocess.PIPE,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
        clip.stdin.write(b'x11-to-virtual');clip.stdin.close()
        wait(lambda:client.call('virtual.clipboard',{'get':True})['data']['formats']==[{'kind':'text_utf8','bytes_base64':base64.b64encode(b'x11-to-virtual').decode()}],message='native clipboard to virtual')
        client.call('virtual.clipboard',{'text':'virtual-to-x11'})
        wait(lambda:subprocess.run(['xclip','-selection','clipboard','-out'],capture_output=True,timeout=2).stdout==b'virtual-to-x11',message='virtual clipboard to X11')
        print('PASS Linux daemon: X11 backend, DSOP, native edge capture, key/click forwarding, clipboard both directions',flush=True)
    finally:
        print('final status',server.status(),flush=True)
        for node in reversed(NODES):node.stop()
        print((server.dir/'stderr.log').read_text(),flush=True)
        if clip and clip.poll() is None:clip.terminate();clip.wait(timeout=3)
    return 0


if __name__=='__main__':raise SystemExit(main())
