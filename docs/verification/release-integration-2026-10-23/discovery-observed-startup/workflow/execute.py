import pathlib,subprocess,os,json,time,sys
root=pathlib.Path(__file__).resolve().parent
label=sys.argv[1]
cwd=pathlib.Path(sys.argv[2]).resolve()
args=sys.argv[3:]
env=os.environ.copy()
env.update(PATH="/Users/serge/.cargo/bin:"+env['PATH'],TMPDIR=str(root/'tmp'),MACOSX_DEPLOYMENT_TARGET='15.0',CARGO_BUILD_JOBS='2')
if label.startswith('build-baseline'): env['CARGO_TARGET_DIR']=str(root/'target-baseline')
else: env['CARGO_TARGET_DIR']=str(root/'target-fixed')
if label=='missing-observer': env['OPENERGY_STARTUP_OBSERVE']='0'
start=time.monotonic()
with (root/'reports'/f'{label}.stdout').open('wb') as out,(root/'reports'/f'{label}.stderr').open('wb') as err:
 p=subprocess.run(['/usr/bin/nice','-n','15',*args],cwd=cwd,env=env,stdout=out,stderr=err,timeout=240)
x=dict(label=label,cwd=str(cwd),argv=['/usr/bin/nice','-n','15',*args],env={k:env[k] for k in ('PATH','TMPDIR','MACOSX_DEPLOYMENT_TARGET','CARGO_BUILD_JOBS','CARGO_TARGET_DIR')},exit=p.returncode,duration_ms=round((time.monotonic()-start)*1000))
if label=='missing-observer': x['env']['OPENERGY_STARTUP_OBSERVE']='0'
(root/'reports'/f'{label}.result.json').write_text(json.dumps(x,indent=2)+'\n')
print(json.dumps({k:x[k] for k in ('label','exit','duration_ms')}))
