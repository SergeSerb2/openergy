import pathlib,subprocess,json,time,os
root=pathlib.Path(__file__).parent; exe=root/'artifact'/'openergy.exe'; lab=root/'tls-pair-4';lab.mkdir();env=os.environ.copy();env.update(OPENERGY_ALLOW_PORT_24800='0',OPENERGY_DISABLE_DISCOVERY='1',APPDATA=str(lab/'appdata'))
procs={}; logs=[]
def cli(node,args,check=True):
 d=lab/node; e=env|{'OPENERGY_CONFIG':str(d/'config.toml'),'OPENERGY_CONFIG_DIR':str(d),'OPENERGY_TLS_DIR':str(d/'tls'),'OPENERGY_SOCKET':str(d/'control.sock')}
 r=subprocess.run([str(exe)]+args,env=e,capture_output=True,text=True,timeout=12)
 if check and r.returncode:raise Exception(f'{node} {args[0]} exit={r.returncode}: {r.stderr}')
 return r
for n,role in [('host','server'),('peer','client')]:
 d=lab/n;d.mkdir(); config=f'version=1\nname="{n}"\nrole="{role}"\n[tls]\nenabled=true\n'
 config+='[server]\nlisten="127.0.0.1:26270"\nport=26270\n[[screen]]\nname="host"\nrect={x=0,y=0,w=1920,h=1080}\n[[screen]]\nname="peer"\nrect={x=1920,y=0,w=1920,h=1080}\n' if n=='host' else '[client]\nremote="127.0.0.1:26270"\n'
 (d/'config.toml').write_text(config)
fps={n:cli(n,['fingerprint','--config',str(lab/n/'config.toml')]).stdout.splitlines()[0].strip() for n in ['host','peer']}
print('FINGERPRINTS_CREATED',fps,flush=True)
for n,other in [('host','peer'),('peer','host')]:
 cli(n,['trust','approve',fps[other],'--config',str(lab/n/'config.toml')])
def status(n):return json.loads(cli(n,['ctl','status.get','--socket',str(lab/n/'control.sock'),'--raw']).stdout)
def start(n):
 d=lab/n;e=env|{'OPENERGY_CONFIG':str(d/'config.toml'),'OPENERGY_CONFIG_DIR':str(d),'OPENERGY_TLS_DIR':str(d/'tls'),'OPENERGY_SOCKET':str(d/'control.sock')}
 out=open(d/'daemon.stdout','w');err=open(d/'daemon.stderr','w');logs.extend([out,err]);procs[n]=subprocess.Popen([str(exe),'daemon','--platform','virtual','--config',str(d/'config.toml'),'--socket',str(d/'control.sock')],env=e,stdout=out,stderr=err)
def wait_connected():
 deadline=time.monotonic()+15
 while time.monotonic()<deadline:
  try:
   s=status('peer')
   if s['state']=='connected':return s
  except Exception:pass
  time.sleep(.2)
 raise Exception('TLS pair connection deadline')
try:
 for n in ['host','peer']:start(n)
 print('TLS_CONNECTED',json.dumps(wait_connected()),flush=True)
 cli('peer',['disconnect','--socket',str(lab/'peer'/'control.sock')]);print('DISCONNECTED',json.dumps(status('peer')),flush=True)
 cli('peer',['connect','--socket',str(lab/'peer'/'control.sock')]);print('TLS_RECONNECTED',json.dumps(wait_connected()),flush=True)
 print('WINDOWS_EXTRACTED_TLS_PAIR_PASS',flush=True)
finally:
 for n,p in procs.items():
  try:cli(n,['ctl','daemon.stop','--socket',str(lab/n/'control.sock'),'--raw']);p.wait(8)
  except Exception:
   if p.poll() is None:p.kill();p.wait(5)
  print('OWNED_DAEMON_EXIT',n,p.returncode,flush=True)
 for f in logs:f.close()
