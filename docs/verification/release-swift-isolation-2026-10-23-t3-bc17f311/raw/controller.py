"""Owned finite manifest preflight and at most one real package. Never replay."""
import ctypes,hashlib,json,os,signal,socket,subprocess,sys,time,traceback,zipfile,plistlib
from pathlib import Path
from prepare import P,REPO,PYTHON,PIN,SEEDS,sha,save,load,preservation,source_inventory,seed_inventory,meta,inventory

def command(argv,**kwargs):
 return subprocess.check_output(argv,cwd=REPO,env={**os.environ,'GIT_OPTIONAL_LOCKS':'0','GIT_CONFIG_NOSYSTEM':'1','GIT_CONFIG_GLOBAL':'/dev/null'},text=True,**kwargs).strip()
class BsdInfo(ctypes.Structure):
    _fields_ = [(name, ctypes.c_uint32) for name in
        ('flags', 'status', 'xstatus', 'pid', 'ppid', 'uid', 'gid', 'ruid', 'rgid', 'svuid', 'svgid', 'rfu')]
    _fields_ += [('comm', ctypes.c_char * 16), ('name', ctypes.c_char * 32)]
    _fields_ += [(name, ctypes.c_uint32) for name in ('nfiles', 'pgid', 'pjobc', 'tdev', 'tpgid')]
    _fields_ += [('nice', ctypes.c_int32), ('start_sec', ctypes.c_uint64), ('start_usec', ctypes.c_uint64)]


LIBPROC = ctypes.CDLL('/usr/lib/libproc.dylib', use_errno=True)
LIBPROC.proc_pidinfo.argtypes = [ctypes.c_int, ctypes.c_int, ctypes.c_uint64, ctypes.c_void_p, ctypes.c_int]


def identity(pid):
    data = BsdInfo()
    if LIBPROC.proc_pidinfo(pid, 3, 0, ctypes.byref(data), ctypes.sizeof(data)) != ctypes.sizeof(data):
        return None
    return dict(pid=data.pid, ppid=data.ppid, pgid=data.pgid, nice=data.nice,
                start_sec=data.start_sec, start_usec=data.start_usec,
                executable=data.name.decode(errors='replace') or data.comm.decode(errors='replace'))


def same(a, b):
    return b is not None and all(a[k] == b[k] for k in ('pid', 'start_sec', 'start_usec', 'pgid'))


def snapshot(root_pid, root_identity, known):
    rows = command(['/bin/ps', '-axo', 'pid=,ppid=,pgid=,ni=,comm='], timeout=5).splitlines()
    table = {}
    for line in rows:
        fields = line.strip().split(None, 4)
        if len(fields) == 5:
            pid, ppid, pgid, nice = map(int, fields[:4])
            table[pid] = dict(pid=pid, ppid=ppid, pgid=pgid, nice=nice, comm=fields[4])
    owned = {root_pid} if same(root_identity, identity(root_pid)) else set()
    # Keep previously attributed identities, including an observed orphan.
    owned |= {pid for pid, old in known.items() if pid in table and same(old, identity(pid))}
    while True:
        added = owned | {pid for pid, row in table.items() if row['ppid'] in owned}
        if added == owned:
            break
        owned = added
    records = []
    for pid in sorted(owned):
        row = table.get(pid)
        current = identity(pid)
        if row is None or current is None:
            continue
        # Every new descendant must still belong to this command's process group.
        if current['pgid'] != root_identity['pgid']:
            continue
        records.append(dict(**current, comm=row['comm'], ps_parent=row['ppid'],
                            ps_group=row['pgid'], ps_nice=row['nice']))
        known[pid] = current
    return records


def cleanup_owned(known,prefix):
 start=time.monotonic();actions=[]
 for sig,budget in [(signal.SIGTERM,10),(signal.SIGKILL,10)]:
  for pid,old in reversed(list(known.items())):
   if same(old,identity(pid)):
    try:os.kill(pid,sig);actions.append(dict(identity=old,signal=int(sig)))
    except ProcessLookupError:pass
  until=time.monotonic()+budget
  while time.monotonic()<until and any(same(old,identity(pid)) for pid,old in known.items()):time.sleep(.1)
 remaining=[old for pid,old in known.items() if same(old,identity(pid))]
 save(prefix+'-cleanup.json',dict(elapsed_seconds=time.monotonic()-start,budget_seconds=60,remaining=remaining,actions=actions))
 return remaining

def managed(prefix,argv,cwd,env,seconds):
 start=time.monotonic();known={};samples=[];timeout=False;error=None
 with (P/(prefix+'.stdout')).open('xb') as out,(P/(prefix+'.stderr')).open('xb') as err:
  process=subprocess.Popen(['/usr/bin/sandbox-exec','-f',str(P/'package.sb'),*argv],cwd=cwd,env=env,stdin=subprocess.DEVNULL,stdout=out,stderr=err,start_new_session=True)
  first=identity(process.pid)
  assert first and first['nice']==15 and first['pgid']==process.pid
  known[process.pid]=first;save(prefix+'-start.json',dict(identity=first,controller=identity(os.getpid()),argv=argv,environment_sha256=hashlib.sha256(json.dumps(env,sort_keys=True).encode()).hexdigest(),profile_sha256=sha(P/'package.sb'),monotonic=start))
  try:
   while process.poll() is None:
    rows=snapshot(process.pid,first,known);samples.append(dict(elapsed_seconds=time.monotonic()-start,owned=rows))
    if any(r['nice']!=15 or r['ps_nice']!=15 for r in rows):raise ValueError('Owned descendant priority differs from NI15')
    if time.monotonic()-start>=seconds:timeout=True;break
    time.sleep(.01)
  except BaseException:error=traceback.format_exc();save(prefix+'-controller-error.json',dict(error=error))
  finally:
   remaining=cleanup_owned(known,prefix)
   code=process.wait(timeout=10)
 save(prefix+'-samples.json',samples)
 result=dict(exit=code,timeout=timeout,controller_error=error,remaining=remaining,elapsed_seconds=time.monotonic()-start,stdout_sha256=sha(P/(prefix+'.stdout')),stderr_sha256=sha(P/(prefix+'.stderr')),root_identity=first)
 save(prefix+'-result.json',result);return result

def seed_check():
 before=load('seed-qualified-base.json');after=seed_inventory();diff={n:[k for k in set(before[n])|set(after[n]) if before[n].get(k)!=after[n].get(k)] for n in SEEDS}
 assert all(set(v)<=({'.git/index'} if n=='checkouts/Sparkle' else set()) for n,v in diff.items()),diff
 c=P/'swift/checkouts/Sparkle'
 assert command(['/usr/bin/git','-C',str(c),'rev-parse','HEAD'])==PIN
 assert not command(['/usr/bin/git','-C',str(c),'status','--porcelain'])
 return dict(differences=diff,pinned_revision=PIN,inventory=after)

def ports(name):
 state=subprocess.run(['/usr/sbin/lsof','-nP','-iTCP:65440-65449'],text=True,capture_output=True)
 record=dict(lsof_before=dict(exit=state.returncode,stdout=state.stdout,stderr=state.stderr),attempts=[]);held=[]
 try:
  for port in range(65440,65450):
   s=socket.socket(socket.AF_INET,socket.SOCK_STREAM);held.append(s)
   try:s.bind(('127.0.0.1',port));s.listen(1);record['attempts'].append(dict(port=port,first_exclusive_bind=True))
   except OSError as e:record['attempts'].append(dict(port=port,first_exclusive_bind=False,error=repr(e)));raise
 finally:
  for s in held:s.close()
  save(name,record)

def preflight(index):
 assert 1<=index<=load('budget.json')['manifest_recipe_limit']
 assert not (P/'package-once.guard').exists()
 for i in range(1,index):assert (P/f'manifest-{i}.guard').exists()
 with (P/f'manifest-{index}.guard').open('x') as f:f.write('Exactly one distinct real manifest recipe invocation; never reset.\n')
 recipe=load(f'manifest-recipe-{index}.json')
 assert recipe['environment']==load('environment.json') and recipe['profile_sha256']==sha(P/'package.sb')
 assert recipe['source']==source_inventory()
 save(f'manifest-{index}-binding.json',dict(recipe_sha256=sha(P/f'manifest-recipe-{index}.json'),controller_sha256=sha(P/'controller.py'),profile_sha256=sha(P/'package.sb'),budget_sha256=sha(P/'budget.json')))
 result=managed(f'manifest-{index}',recipe['argv'],recipe['cwd'],recipe['environment'],180)
 after=preservation();save(f'manifest-{index}-preservation.json',after)
 unchanged=after==load('preservation-initial.json');seed=seed_check();save(f'manifest-{index}-seed-check.json',seed)
 stderr=(P/f'manifest-{index}.stderr').read_text();failure_diagnostics=any(s in stderr for s in ['Operation not permitted',"couldn't create cache",'sandbox_apply'])
 passed=result['exit']==0 and not result['timeout'] and not result['controller_error'] and not result['remaining'] and unchanged and not failure_diagnostics
 if passed:
  description=json.loads((P/f'manifest-{index}.stdout').read_text());assert description['name']=='Openergy'
  save('qualified-manifest.json',dict(passed=True,recipe=index,recipe_sha256=sha(P/f'manifest-recipe-{index}.json'),profile_sha256=sha(P/'package.sb'),source=source_inventory(),real_description=description,old_cache_source_prior_metadata_unchanged=unchanged,seeds_pinned=True,outer_policy_unchanged=True))
 save(f'manifest-{index}-qualification.json',dict(passed=passed,old_cache_source_prior_metadata_unchanged=unchanged,forbidden_diagnostics=failure_diagnostics,result=result))
 print(json.dumps(load(f'manifest-{index}-qualification.json')),flush=True)
 if not passed:raise SystemExit('STOP: manifest recipe unqualified; no package invocation')

def qualify_package():
 q=load('qualified-manifest.json');env=load('environment.json');assert q['passed'] and q['source']==source_inventory()
 assert q['profile_sha256']==sha(P/'package.sb') and preservation()==load('preservation-initial.json')
 save('package-qualified-seeds.json',seed_check())
 assert not list((P/'cargo-target').iterdir())
 # Manifest caches may exist. No old or new app output/build database is present.
 assert not list((P/'swift').rglob('build.db')) and not list((P/'swift').rglob('Openergy.build'))
 for name in ['SWIFT_SCRATCH_PATH','SWIFT_CACHE_PATH','SWIFT_CONFIG_PATH','SWIFT_SECURITY_PATH','CLANG_MODULE_CACHE_PATH','CARGO_TARGET_DIR','CARGO_HOME','HOME','CFFIXED_USER_HOME','TMPDIR','XDG_CACHE_HOME','XDG_CONFIG_HOME']:
  path=Path(env[name]).resolve(strict=True);assert path.is_relative_to(P)
 assert all(not k.startswith('OPENERGY_SIGN') and k!='OPENERGY_ALLOW_SIGNING' for k in env)
 head=command(['/usr/bin/git','rev-parse','HEAD']);assert not command(['/usr/bin/git','status','--porcelain'])
 # Explicit policy parsing and private Unix IPC under exactly the final profile.
 code='''import socket,tempfile,os
with tempfile.TemporaryDirectory(prefix="oe-release-smoke-",dir="/tmp") as d:
 with socket.socket(socket.AF_UNIX,socket.SOCK_STREAM) as s:
  s.bind(d+"/control.sock");s.listen(1)
  with socket.socket(socket.AF_UNIX,socket.SOCK_STREAM) as c:
   c.connect(d+"/control.sock")
   with s.accept()[0] as a:c.sendall(b"owned");assert a.recv(5)==b"owned"
print("final-policy private Unix IPC; NI",os.getpriority(os.PRIO_PROCESS,0))
'''
 r=subprocess.run(['/usr/bin/sandbox-exec','-f',str(P/'package.sb'),PYTHON,'-c',code],cwd=REPO,env=env,capture_output=True,text=True,timeout=10)
 save('final-policy-unix.json',dict(argv=[PYTHON,'-c',code],exit=r.returncode,stdout=r.stdout,stderr=r.stderr));assert r.returncode==0
 ports('ports-preflight-first.json')
 plan=dict(argv=[PYTHON,str(REPO/'tools/release/release.py'),'package','--port-range','65440-65449','--output',str(P/'output')],environment=env,source_commit=head,source=source_inventory(),manifest_sha256=sha(P/'qualified-manifest.json'),profile_sha256=sha(P/'package.sb'),budget_sha256=sha(P/'budget.json'),controller_sha256=sha(P/'controller.py'),observer_sha256=sha(P/'observe/sitecustomize.py'),fresh_cargo_target=True,fresh_swift_build_database=True,signing='ordinary ad hoc, identity and purpose variables unset',package_limit=1,watchdog_seconds=900,cleanup_seconds=60)
 save('package-plan.json',plan);print('QUALIFIED PACKAGE PLAN',sha(P/'package-plan.json'),flush=True)

def run_package():
 plan=load('package-plan.json');assert plan['environment']==load('environment.json')
 assert plan['source_commit']==command(['/usr/bin/git','rev-parse','HEAD']) and not command(['/usr/bin/git','status','--porcelain'])
 assert preservation()==load('preservation-initial.json')
 for name,key in [('package.sb','profile_sha256'),('controller.py','controller_sha256'),('observe/sitecustomize.py','observer_sha256')]:assert sha(P/name)==plan[key]
 assert source_inventory()==plan['source'];seed_check()
 with (P/'package-once.guard').open('x') as f:f.write('ONE actual release.py package invocation; never reset.\n')
 result=managed('package',plan['argv'],REPO,plan['environment'],900)
 ports('ports-postflight-first.json')
 save('preservation-after-package.json',preservation());save('seeds-after-package.json',seed_check())
 print(json.dumps(result),flush=True)
 assert load('preservation-after-package.json')==load('preservation-initial.json')
 if result['exit']!=0 or result['timeout'] or result['controller_error']:raise SystemExit('STOP: first package failed; no retry')

if __name__=='__main__':
 os.umask(0o077);os.setpriority(os.PRIO_PROCESS,0,15);assert os.getpriority(os.PRIO_PROCESS,0)==15
 try:
  if sys.argv[1]=='manifest':preflight(int(sys.argv[2]))
  elif sys.argv[1]=='qualify':qualify_package()
  elif sys.argv[1]=='package':run_package()
  else:raise ValueError('unknown mode')
 except BaseException:
  name='first-error-'+ '-'.join(sys.argv[1:])+'.txt'
  if not (P/name).exists():
   with (P/name).open('x') as f:f.write(traceback.format_exc())
  raise
