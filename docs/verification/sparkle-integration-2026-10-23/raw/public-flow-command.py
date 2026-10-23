from pathlib import Path
import json,subprocess,os,plistlib,uuid,time,hashlib
state=json.loads(Path('/tmp/oe-sparkle-combined-20261023-state.json').read_text())
root=Path(state['root']);source=root/'bundle-final/Openergy.app'
records=[]
for label,expected in [('unconfigured','unconfigured'),('cloudflare','current')]:
 output=root/('e2e-final-'+label);output.mkdir(mode=0o700)
 config=output/'config';config.mkdir(mode=0o700)
 app=output/'Openergy.app'
 subprocess.run(['ditto',str(source),str(app)],check=True)
 plist=app/'Contents/Info.plist';info=plistlib.loads(plist.read_bytes())
 info['CFBundleIdentifier']='dev.openergy.updater-e2e.'+uuid.uuid4().hex
 info['SUDefaultsDomain']=str(config.resolve()/'sparkle-preferences')
 feed=info['SUFeedURL']
 if expected=='unconfigured':
  info.pop('SUFeedURL');info.pop('SUPublicEDKey');feed='http://127.0.0.1:25580/unused.xml'
 plist.write_bytes(plistlib.dumps(info))
 subprocess.run(['codesign','--force','--options','0','--timestamp=none','--sign','-',str(app)],capture_output=True,check=True)
 env={k:v for k,v in os.environ.items() if not any(x in k.upper() for x in ['TOKEN','SECRET','PASSWORD','PRIVATE_KEY','API_KEY'])}
 env.update({'CFFIXED_USER_HOME':str(config.resolve()),'OPENERGY_CONFIG':str(config/'config.toml'),'OPENERGY_CONFIG_DIR':str(config),'OPENERGY_TLS_DIR':str(config/'tls'),'OPENERGY_SOCKET':str(config/'unused.sock'),'OPENERGY_ALLOW_PORT_24800':'0','OPENERGY_DISABLE_DISCOVERY':'1','OPENERGY_DAEMON_ARGS':'--platform virtual','OPENERGY_UPDATE_FEED_URL':'http://127.0.0.1:25580/runtime-override-must-be-ignored','OPENERGY_UPDATE_PUBLIC_KEY':'not-a-valid-key'})
 args=['nice','-n','15',str(app/'Contents/MacOS/Openergy'),'--updater-e2e',str(output.resolve()),'--updater-config',str(config.resolve()),'--updater-feed',feed,'--updater-expect',expected,'--updater-timeout','25']
 began=time.monotonic();run=subprocess.run(args,env=env,capture_output=True,text=True,timeout=35)
 (output/'stdout.log').write_text(run.stdout);(output/'stderr.log').write_text(run.stderr)
 record={'name':label,'exit':run.returncode,'elapsed_seconds':time.monotonic()-began,'bundle_identifier':info['CFBundleIdentifier'],'arguments':args,'daemon_started':False}
 result=output/'updater-result.json'
 if result.exists():record['outcome']=json.loads(result.read_text())
 records.append(record);(root/'final-updater-commands.json').write_text(json.dumps(records,indent=2)+'\n')
 print(json.dumps(record),flush=True)
 if run.returncode:
  print(run.stderr[-2000:],flush=True)
  raise SystemExit(run.returncode if run.returncode>0 else 1)
print('Combined unconfigured and production-feed updater commands passed.',flush=True)
