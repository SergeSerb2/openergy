"""Real TLS peers through pre-opened SSH tunnels; virtual platforms only.
Run after the catalogue releases 24820/24822. Uses only the fixed disposable labs.
"""
import base64,json,os,pathlib,subprocess,time
ROOT=pathlib.Path('/tmp/oe-win-port-net')
BIN=str(ROOT/'openergy')
REMOTE_ROOT=os.environ.get('OPENERGY_WINDOWS_LAB_ROOT')
if not REMOTE_ROOT:
 raise SystemExit('Set OPENERGY_WINDOWS_LAB_ROOT to the absolute disposable Windows build directory.')
EXE=REMOTE_ROOT+r'\target\release\openergy.exe'
lab=REMOTE_ROOT+r'\lab\network'
owned=[]
def pq(s): return "'"+str(s).replace("'","''")+"'"
def ssh(command,timeout=20):
 p=subprocess.run(['ssh','windows',command],capture_output=True,text=True,timeout=timeout)
 if p.returncode: raise RuntimeError('Windows lab command failed exit='+str(p.returncode)+' '+p.stderr[:180])
 return p.stdout.strip()
def cli(node,method,params=None):
 if node.startswith('win'):
  cmd=f'& {pq(EXE)} ctl {pq(method)} --raw --socket {pq(lab+chr(92)+node+chr(92)+"ctl.sock")} --json {pq(json.dumps(params or {},separators=(",",":")))}; exit $LASTEXITCODE'
  return json.loads(ssh(cmd))
 p=subprocess.run([BIN,'ctl',method,'--socket',str(ROOT/node/'ctl.sock'),'--raw','--json',json.dumps(params or {})],capture_output=True,text=True,timeout=15)
 if p.returncode: raise RuntimeError(f'Mac lab CLI {method} failed exit={p.returncode}')
 return json.loads(p.stdout)
def cfg(name,role,port,remote):
 return f'''version = 1
name = "{name}"
role = "{role}"
[server]
listen = "127.0.0.1:{port}"
port = {port}
heartbeat_ms = 1000
[client]
remote = "127.0.0.1:{remote}"
heartbeat_ms = 1000
[tls]
enabled = true
require_client_trust = true
[[screen]]
name = "{name if role=='server' else remote_name[name]}"
rect = {{x=0,y=0,w=1920,h=1080}}
[[screen]]
name = "{remote_name[name] if role=='server' else name}"
rect = {{x=1920,y=0,w=1920,h=1080}}
'''
remote_name={'win-server':'mac-client','mac-client':'win-server','mac-server':'win-client','win-client':'mac-server'}
def prep(name,role,port,remote):
 text=cfg(name,role,port,remote)
 if name.startswith('win'):
  directory=lab+'\\'+name
  b=base64.b64encode(text.encode()).decode()
  ssh(f'New-Item -ItemType Directory -Force {pq(directory)} | Out-Null; [IO.File]::WriteAllText({pq(directory+chr(92)+"config.toml")},[Text.Encoding]::UTF8.GetString([Convert]::FromBase64String({pq(b)})))')
  return ssh(f'& {pq(EXE)} fingerprint --config {pq(directory+chr(92)+"config.toml")} --synergy; exit $LASTEXITCODE')
 directory=ROOT/name;directory.mkdir(exist_ok=True);(directory/'config.toml').write_text(text)
 return subprocess.check_output([BIN,'fingerprint','--config',str(directory/'config.toml'),'--synergy'],text=True).strip()
def trust(name,fp):
 if name.startswith('win'): ssh(f'& {pq(EXE)} trust approve {pq(fp)} --config {pq(lab+chr(92)+name+chr(92)+"config.toml")}; exit $LASTEXITCODE')
 else:subprocess.run([BIN,'trust','approve',fp,'--config',str(ROOT/name/'config.toml')],check=True,capture_output=True)
def start(name,role,port):
 if name.startswith('win'):
  directory=lab+'\\'+name
  command=f'$env:OPENERGY_DISABLE_DISCOVERY="1"; $p=Start-Process -FilePath {pq(EXE)} -ArgumentList @({pq(role)},"--platform","virtual","--config",{pq(directory+chr(92)+"config.toml")},"--socket",{pq(directory+chr(92)+"ctl.sock")}) -PassThru -RedirectStandardError {pq(directory+chr(92)+"stderr.log")} -RedirectStandardOutput {pq(directory+chr(92)+"stdout.log")}; if($p.SessionId -ne 0){{throw "session must be zero"}}; $p.Id | Set-Content {pq(directory+chr(92)+"pid")}; [Console]::Out.WriteLine($p.Id); [Console]::Out.Flush(); $p.WaitForExit(); exit $p.ExitCode'
  session=subprocess.Popen(['ssh','windows',command],stdout=subprocess.PIPE,stderr=open(ROOT/(name+'-ssh.stderr'),'w'),text=True); pid=int(session.stdout.readline().strip());owned.append((name,(pid,session)));print(f'START {name} pid={pid} session=0',flush=True)
 else:
  directory=ROOT/name;env=dict(os.environ,OPENERGY_DISABLE_DISCOVERY='1')
  p=subprocess.Popen([BIN,role,'--platform','virtual','--config',str(directory/'config.toml'),'--socket',str(directory/'ctl.sock')],stdout=open(directory/'stdout.log','w'),stderr=open(directory/'stderr.log','w'),env=env)
  owned.append((name,p));print(f'START {name} pid={p.pid}',flush=True)
def wait_connected(name):
 for _ in range(50):
  try:
   s=cli(name,'status.get')
   if any(p.get('connected') for p in s['peers']):
    assert all(p.get('tls',{}).get('enabled') and p.get('tls',{}).get('trusted') for p in s['peers'] if p.get('connected')), 'TLS/trust status'
    print('PASS '+name+' connected with TLS and pinned trust',flush=True);return
  except (OSError,ValueError,RuntimeError):pass
  time.sleep(.2)
 raise RuntimeError('connection deadline '+name)
def wait_for(fn,label):
 for _ in range(25):
  if fn(): print('PASS '+label,flush=True);return
  time.sleep(.15)
 raise RuntimeError('assertion deadline '+label)
def exercise(server,client):
 mark=cli(client,'virtual.recorded',{'since':0})
 last=max([r['sequence'] for r in mark['events']]or[0])
 events=[{'type':'mouse_move_abs','x':1919,'y':540},{'type':'mouse_move_rel','dx':30,'dy':0},{'type':'key_down','key':97,'modifiers':0,'button':1},{'type':'key_up','key':97,'modifiers':0,'button':1},{'type':'mouse_down','button':1},{'type':'mouse_up','button':1},{'type':'mouse_move_abs','x':120,'y':160},{'type':'mouse_move_rel','dx':12,'dy':5},{'type':'mouse_wheel','dx':120,'dy':-120}]
 cli(server,'virtual.inject',{'events':events})
 expected=['key_down','key_up','mouse_down','mouse_up','mouse_move_abs','mouse_wheel']
 def records():
  r=cli(client,'virtual.recorded',{'since':last})['events'];types=[(x.get('event') or {}).get('type') for x in r];return all(t in types for t in expected)
 wait_for(records,server+' input received in '+client)
 data={'formats':[{'kind':'text_utf8','bytes':list(b'Openergy Windows TLS lab')} ]}
 cli(server,'virtual.clipboard',{'set':data})
 def matches(name,data):
  result=cli(name,'virtual.clipboard',{'get':True}); formats=result.get('formats',result.get('data',{}).get('formats',[])); normalized=[{'kind':f['kind'],'bytes':f.get('bytes',list(base64.b64decode(f.get('bytes_base64',''))))} for f in formats]; return normalized==data['formats']
 wait_for(lambda:matches(client,data),server+' clipboard received in '+client)
 data2={'formats':[{'kind':'text_utf8','bytes':list(b'Openergy reverse clipboard lab')} ]}
 cli(client,'virtual.clipboard',{'set':data2})
 wait_for(lambda:matches(server,data2),client+' clipboard received in '+server)
 cli(server,'virtual.inject',{'events':[{'type':'key_down','key':98,'modifiers':0,'button':2},{'type':'mouse_down','button':1}]})
 wait_for(lambda:cli(client,'virtual.recorded',{'since':last})['stats']['pressed_keys']==1 and cli(client,'virtual.recorded',{'since':last})['stats']['pressed_buttons']==1,'held key and mouse button reach '+client)
 cli(client,'connection.disconnect')
 wait_for(lambda:not cli(client,'virtual.recorded',{'since':0})['stats']['pressed_keys'] and not cli(client,'virtual.recorded',{'since':0})['stats']['pressed_buttons'],client+' release ledger empty on disconnect')
def stop(name,process):
 if not name.startswith('win') and process.poll() is None:
  path=subprocess.check_output(['ps','-p',str(process.pid),'-o','comm='],text=True).strip()
  if path!=BIN: raise RuntimeError('owned Mac executable path mismatch')
 if name.startswith('win'):
  pid,session=process
  ssh(f'$p=Get-Process -Id {pid} -ErrorAction SilentlyContinue; if($p -and $p.Path -ne {pq(EXE)}){{throw "owned executable path mismatch"}}; exit 0')
 try:cli(name,'daemon.stop')
 except Exception:pass
 if name.startswith('win'):
  command=f'$p=Get-Process -Id {pid} -ErrorAction SilentlyContinue; if($p) {{if($p.Path -ne {pq(EXE)}){{throw "owned executable path mismatch"}}; if(!$p.WaitForExit(6000)){{throw "owned daemon exit deadline"}}}}; exit 0'
  ssh(command)
  if session.wait(6)!=0: raise RuntimeError("Windows daemon session failed")
 else:process.wait(6)
 print('STOP '+name,flush=True)
try:
 # Both directions use separate identities/configs; fingerprints never reach output.
 for server,client,port in [('win-server','mac-client',24820),('mac-server','win-client',24822)]:
  sfp=prep(server,'server',port,port);cfp=prep(client,'client',port+1,port)
  trust(server,cfp);trust(client,sfp)
  print(f'PREP {server} fp={sfp.split(":")[-1][:8]} {client} fp={cfp.split(":")[-1][:8]}',flush=True)
  start(server,'server',port);start(client,'client',port)
  wait_connected(server);wait_connected(client);exercise(server,client)
  for name,proc in owned[-2:][::-1]:stop(name,proc)
  owned=owned[:-2]
 print('NETWORK_PASS',flush=True)
finally:
 for name,proc in owned[::-1]:
  try:stop(name,proc)
  except Exception as e: print('CLEANUP_REQUIRED '+name+' '+str(e),flush=True)
