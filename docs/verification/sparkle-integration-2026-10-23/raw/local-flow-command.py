from pathlib import Path
import json,subprocess,os,plistlib,uuid,time,threading,http.server,hashlib
s=json.loads(Path('/tmp/oe-sparkle-combined-20261023-state.json').read_text());root=Path(s['root'])
source=root/'bundle-final/Openergy.app'
requests=[]
feeds={'/current.xml':Path('deployment/cloudflare/public/appcast.xml').read_bytes(),'/candidate.xml':(root/'signed-update-final/appcast.xml').read_bytes(),'/broken.xml':b'<rss><channel>'}
class Handler(http.server.BaseHTTPRequestHandler):
 def do_GET(self):
  requests.append(self.path)
  if self.path=='/failure.xml':
   self.send_response(503);self.end_headers();self.wfile.write(b'Service unavailable');return
  payload=feeds.get(self.path)
  if payload is None:self.send_response(404);self.end_headers();return
  self.send_response(200);self.send_header('Content-Type','application/rss+xml');self.send_header('Content-Length',str(len(payload)));self.end_headers();self.wfile.write(payload)
 def log_message(self,*args):pass
server=http.server.ThreadingHTTPServer(('127.0.0.1',0),Handler)
assert server.server_address[1]!=24800
thread=threading.Thread(target=server.serve_forever,daemon=True);thread.start()
records=[]
try:
 for label,path,expected in [('current','/current.xml','current'),('candidate','/candidate.xml','candidate'),('http-failure','/failure.xml','failure'),('malformed','/broken.xml','failure')]:
  output=root/('fixture-final-'+label);output.mkdir(mode=0o700);config=output/'config';config.mkdir(mode=0o700)
  app=output/'Openergy.app';subprocess.run(['ditto',str(source),str(app)],check=True)
  p=app/'Contents/Info.plist';info=plistlib.loads(p.read_bytes());identifier='dev.openergy.updater-e2e.'+uuid.uuid4().hex
  info['CFBundleIdentifier']=identifier;info['CFBundleVersion']='1';info['SUDefaultsDomain']=str(config.resolve()/'sparkle-preferences')
  feed='http://127.0.0.1:'+str(server.server_address[1])+path;info['SUFeedURL']=feed;info['NSAppTransportSecurity']={'NSAllowsLocalNetworking':True}
  p.write_bytes(plistlib.dumps(info))
  subprocess.run(['codesign','--force','--options','0','--timestamp=none','--sign','-',str(app)],capture_output=True,check=True)
  env={k:v for k,v in os.environ.items() if not any(x in k.upper() for x in ['TOKEN','SECRET','PASSWORD','PRIVATE_KEY','API_KEY'])}
  env.update({'CFFIXED_USER_HOME':str(config.resolve()),'OPENERGY_CONFIG':str(config/'unused-config.toml'),'OPENERGY_CONFIG_DIR':str(config),'OPENERGY_TLS_DIR':str(config/'unused-tls'),'OPENERGY_SOCKET':str(config/'unused.sock'),'OPENERGY_ALLOW_PORT_24800':'0','OPENERGY_DISABLE_DISCOVERY':'1','OPENERGY_DAEMON_ARGS':'--platform virtual'})
  args=['nice','-n','15',str(app/'Contents/MacOS/Openergy'),'--updater-e2e',str(output.resolve()),'--updater-config',str(config.resolve()),'--updater-feed',feed,'--updater-expect',expected,'--updater-timeout','15']
  began=time.monotonic();result=subprocess.run(args,env=env,capture_output=True,text=True,timeout=25)
  (output/'stdout.log').write_text(result.stdout);(output/'stderr.log').write_text(result.stderr)
  record={'name':label,'exit':result.returncode,'elapsed_seconds':time.monotonic()-began,'bundle_identifier':identifier,'arguments':args}
  if (output/'updater-result.json').exists():record['outcome']=json.loads((output/'updater-result.json').read_text())
  records.append(record);(root/'final-local-fixtures.json').write_text(json.dumps({'commands':records,'requests':requests},indent=2)+'\n')
  print(json.dumps(record),flush=True)
  assert result.returncode==0,(label,result.stderr[-500:])
  assert not (Path.home()/'Library/Preferences'/(identifier+'.plist')).exists(),'Fixture preference escaped private defaults domain'
 assert requests==['/current.xml','/candidate.xml','/failure.xml','/broken.xml'],requests
finally:
 server.shutdown();server.server_close();thread.join(timeout=3)
assert not thread.is_alive()
print('Four complete real Sparkle fixture flows passed; no archive was requested, no user-domain preference file appeared, server closed.',flush=True)
