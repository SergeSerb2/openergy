from pathlib import Path
import hashlib,json,os,subprocess,time,xml.etree.ElementTree as ET
state=json.loads(Path('/tmp/oe-sparkle-combined-20261023-state.json').read_text());root=Path(state['root'])
env={k:v for k,v in os.environ.items() if not any(x in k.upper() for x in ['TOKEN','SECRET','PASSWORD','PRIVATE_KEY','API_KEY'])}
env.update(PATH=str(Path.home()/'.cargo/bin')+os.pathsep+env['PATH'],RUSTUP_TOOLCHAIN='1.99.1',CARGO_BUILD_JOBS='2',CARGO_NET_OFFLINE='true',CARGO_TARGET_DIR=state['cargo_target'],SWIFT_SCRATCH_PATH=state['swift_scratch'],PYTHON='/Users/serge/.local/bin/python3.11',MACOSX_DEPLOYMENT_TARGET='15.0',OPENERGY_BUILD='2',OPENERGY_REQUIRE_UPDATES='1',OPENERGY_SIGN_IDENTITY='-',TMPDIR=str(root/'tmp')+'/',UNIVERSAL='1',DIST=str(root/'bundle-final'))
head=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip();records=[]
def run(name,args):
 start=time.monotonic()
 with (root/(name+'.log')).open('w') as log: p=subprocess.run(args,env=env,stdout=log,stderr=subprocess.STDOUT,timeout=900)
 r={'name':name,'source_commit':head,'arguments':args,'exit':p.returncode,'elapsed_seconds':time.monotonic()-start,'log_sha256':hashlib.sha256((root/(name+'.log')).read_bytes()).hexdigest()};records.append(r);(root/'final-commands.json').write_text(json.dumps(records,indent=2)+'\n');print(json.dumps(r),flush=True)
 if p.returncode: print('\n'.join((root/(name+'.log')).read_text().splitlines()[-12:]),flush=True);raise SystemExit(p.returncode)
run('bundle-final',['bash','apps/macos/scripts/bundle.sh'])
run('signed-update-final',['nice','-n','15','/Users/serge/.local/bin/python3.11','tools/release/sparkle.py','archive','--app',str(root/'bundle-final/Openergy.app'),'--output',str(root/'signed-update-final'),'--scratch',state['swift_scratch'],'--private-key-file',str(Path.home()/'.config/openergy-release/sparkle-ed25519.key'),'--version','0.1.0','--build','2','--download-url','https://openergy-updates.sergeserbinenko.com/releases/openergy-0.1.0-2-macos-universal.zip'])
assert ET.parse(root/'signed-update-final/appcast.xml').findtext('channel/title')=='Openergy Updates'
local=Path('/tmp/oe-sparkle-combined-local-fixtures-20261023.py').read_text().replace("bundle-diagnostics/Openergy.app","bundle-final/Openergy.app").replace("signed-update/appcast.xml","signed-update-final/appcast.xml").replace("'fixture-'","'fixture-final-'").replace("combined-local-fixtures.json","final-local-fixtures.json")
p=Path('/tmp/oe-sparkle-final-local-fixtures-20261023.py');p.write_text(local);run('local-flows-final',['/Users/serge/.local/bin/python3.11',str(p)])
public=Path('/tmp/oe-sparkle-diagnostic-updater-20261023.py').read_text().replace("bundle-diagnostics/Openergy.app","bundle-final/Openergy.app").replace("[('cloudflare','current')]","[('unconfigured','unconfigured'),('cloudflare','current')]").replace("e2e-diagnostic-","e2e-final-").replace("diagnostic-updater-commands.json","final-updater-commands.json")
p=Path('/tmp/oe-sparkle-final-public-updater-20261023.py');p.write_text(public);run('public-flows-final',['/Users/serge/.local/bin/python3.11',str(p)])
print('Final universal app, archive signature checks and six real updater command paths passed. Archive remains private.',flush=True)
