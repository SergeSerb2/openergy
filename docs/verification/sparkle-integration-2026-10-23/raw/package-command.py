from pathlib import Path
import hashlib,json,os,subprocess,time
state=json.loads(Path('/tmp/oe-sparkle-combined-20261023-state.json').read_text());root=Path(state['root'])
env={k:v for k,v in os.environ.items() if not any(x in k.upper() for x in ['TOKEN','SECRET','PASSWORD','PRIVATE_KEY','API_KEY'])}
env.update(PATH=str(Path.home()/'.cargo/bin')+os.pathsep+env['PATH'],RUSTUP_TOOLCHAIN='1.99.1',CARGO_BUILD_JOBS='2',CARGO_NET_OFFLINE='true',CARGO_TARGET_DIR=state['cargo_target'],SWIFT_SCRATCH_PATH=state['swift_scratch'],PYTHON='/Users/serge/.local/bin/python3.11',MACOSX_DEPLOYMENT_TARGET='15.0',OPENERGY_BUILD='2',OPENERGY_REQUIRE_UPDATES='1',OPENERGY_SIGN_IDENTITY='-',TMPDIR=str(root/'tmp')+'/')
head=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()
args=['/Users/serge/.local/bin/python3.11','tools/release/release.py','package','--port-range','25520-25529','--output',str(root/'native-release')]
start=time.monotonic()
with (root/'native-release.log').open('w') as log:
 p=subprocess.run(args,env=env,stdout=log,stderr=subprocess.STDOUT,timeout=900)
record={'source_commit':head,'arguments':args,'exit':p.returncode,'elapsed_seconds':time.monotonic()-start,'log_sha256':hashlib.sha256((root/'native-release.log').read_bytes()).hexdigest()}
(root/'native-release-command.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(record),flush=True)
if p.returncode: print('\n'.join((root/'native-release.log').read_text().splitlines()[-20:]),flush=True)
raise SystemExit(p.returncode)
