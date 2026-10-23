from pathlib import Path
import hashlib,json,os,signal,socket,subprocess,tempfile,time
state=json.loads(Path('/tmp/oe-sparkle-combined-20261023-state.json').read_text());source=Path(state['root'])/'bundle-final/Openergy.app';binary=source/'Contents/Helpers/openergy'
base=Path(tempfile.mkdtemp(prefix='oe-app-flow-',dir='/tmp'));base.chmod(0o700);peer=base/'peer';peer.mkdir(mode=0o700);home=base/'home';home.mkdir(mode=0o700);output=base/'app';output.mkdir(mode=0o700)
Path('/tmp/oe-app-current-flow-20261023-state.json').write_text(json.dumps({'root':str(base),'source_app':str(source),'source_commit':'043757a6e8c8729081a3d7ba01633be6e72c1ea7'},indent=2)+'\n')
peer_port,host_port=25530,25531
for p in (peer_port,host_port):
 with socket.socket() as s:s.bind(('127.0.0.1',p))
(peer/'config.toml').write_text(f'version=1\nname="e2e-peer"\nrole="server"\n[server]\nlisten="127.0.0.1:{peer_port}"\nport={peer_port}\n[tls]\nenabled=false\n[[screen]]\nname="e2e-peer"\nrect={{x=0,y=0,w=1920,h=1080}}\n[[screen]]\nname="app"\nrect={{x=1920,y=0,w=1920,h=1080}}\n')
env={k:v for k,v in os.environ.items() if not any(x in k.upper() for x in ['TOKEN','SECRET','PASSWORD','PRIVATE_KEY','API_KEY'])}
env.update(OPENERGY_CONFIG=str(peer/'config.toml'),OPENERGY_CONFIG_DIR=str(peer),OPENERGY_TLS_DIR=str(peer/'tls'),OPENERGY_SOCKET=str(peer/'control.sock'),OPENERGY_DISABLE_DISCOVERY='1',OPENERGY_ALLOW_PORT_24800='0',CFFIXED_USER_HOME=str(home),OPENERGY_DAEMON_ARGS='--platform virtual')
commands=[];process=None
try:
 with (base/'peer.log').open('w') as log:
  process=subprocess.Popen(['nice','-n','15',str(binary),'daemon','--platform','virtual','--config',str(peer/'config.toml'),'--socket',str(peer/'control.sock'),'--foreground'],env=env,stdout=log,stderr=subprocess.STDOUT,start_new_session=True)
  began=time.monotonic()
  while True:
   r=subprocess.run([str(binary),'status','--socket',str(peer/'control.sock')],env=env,capture_output=True,text=True,timeout=5)
   if r.returncode==0:
    status=json.loads(r.stdout);assert status['daemon']['pid']==process.pid and status['daemon']['platform']=='virtual' and status['state']=='listening';break
   assert process.poll() is None and time.monotonic()-began<15
   time.sleep(.1)
  appenv=dict(env,OPENERGY_CONFIG=str(output/'config/config.toml'),OPENERGY_CONFIG_DIR=str(output/'config'),OPENERGY_TLS_DIR=str(output/'config/tls'),OPENERGY_SOCKET=str(output/'config/control.sock'),OPENERGY_E2E_NO_RENDER='1')
  args=['nice','-n','15',str(source/'Contents/MacOS/Openergy'),'--e2e',str(output),'--join',f'127.0.0.1:{peer_port}','--host-port',str(host_port),'--require-peer']
  began=time.monotonic()
  with (base/'app-command.log').open('w') as appout:r=subprocess.run(args,env=appenv,stdout=appout,stderr=subprocess.STDOUT,timeout=180)
  record={'arguments':args,'exit':r.returncode,'elapsed_seconds':time.monotonic()-began,'rendered':False,'peer_pid':process.pid,'app_binary_sha256':hashlib.sha256((source/'Contents/MacOS/Openergy').read_bytes()).hexdigest(),'daemon_binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest()}
  (base/'command.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(record),flush=True)
  report=output/'report.json'
  if report.exists():
   steps=json.loads(report.read_text());print(json.dumps({'checks':len(steps),'passed':sum(x['passed'] for x in steps),'failed':[x for x in steps if not x['passed']]}),flush=True)
  if r.returncode:print('\n'.join((base/'app-command.log').read_text().splitlines()[-12:]),flush=True)
finally:
 if process is not None and process.poll() is None:
  subprocess.run([str(binary),'ctl','daemon.stop','--socket',str(peer/'control.sock')],env=env,capture_output=True,timeout=10)
  try:process.wait(timeout=10)
  except subprocess.TimeoutExpired:os.killpg(process.pid,signal.SIGTERM);process.wait(timeout=5)
 print('Private artifacts:',base,flush=True)
