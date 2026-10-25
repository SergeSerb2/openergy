"""Parent-owned bounded fixed-ROOT state query. No target writes or child spawn."""
import ast
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

OUT = Path(__file__).resolve().parent
REPO = OUT.parents[2]
BASE = Path('/Users/serge/.local/state/openergy-verification/m1-source-metadata-launch-prepared-20261025-7ae23486940a')
ROOT = '/Users/m1-dev/.local/state/openergy-verification/m1-ordinary-reviewed-flow-20261025-4b20357c1503'


def canonical(value):
    return (json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=True,allow_nan=False)+'\n').encode('ascii')


def put(path, raw, mode):
    fd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW|os.O_NONBLOCK,mode)
    try:
        assert os.write(fd,raw)==len(raw)
        os.fsync(fd)
    finally:
        os.close(fd)
    d=os.open(path.parent,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
    try:os.fsync(d)
    finally:os.close(d)


os.umask(0o077)
os.setpriority(os.PRIO_PROCESS,0,15)
source=(REPO/'docs/verification/m1-prepare-parent-root-layout-repair-2026-10-25-t3-1f8914f8/prepare-parent.py.txt').read_bytes()
assert hashlib.sha256(source).hexdigest()=='b3fbd793a0709c7857e34efca667a081a0486accfd0f3ad13a8068484b7eb8d9'
tree=ast.parse(source)
constants={}
for n in tree.body:
    if isinstance(n,ast.Assign) and len(n.targets)==1 and isinstance(n.targets[0],ast.Name):
        if n.targets[0].id in ['INITIAL_INPUTS','COMPLETED','SOURCE_RAWS','SOURCE_BOUNDARY','EIGHT','ROOT_DATA_ADDITIONS','OUTPUTS']:
            constants[n.targets[0].id]=ast.literal_eval(n.value)
expected=set(constants['INITIAL_INPUTS'])|set(constants['COMPLETED'])|{r['leaf'] for r in constants['SOURCE_RAWS']}|set(constants['SOURCE_BOUNDARY'])|set(constants['EIGHT'])|set(constants['ROOT_DATA_ADDITIONS'])
assert len(expected)==260
fixed=list(constants['OUTPUTS'])+['prepared.json','prepare-guard.json','bytes.json','bytes.lock']
remote = '''import os,sys,time,stat,json,signal
os.umask(0o077)
os.setpriority(os.PRIO_PROCESS,0,15)
assert sys.argv==['-'] and sys.version_info[:3]==(3,9,6)
assert sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode and not sys.flags.optimize
START=time.monotonic();END=START+20.0
def stop(*unused):raise TimeoutError('read-only query deadline')
signal.signal(signal.SIGALRM,stop);signal.setitimer(signal.ITIMER_REAL,20.0)
ROOT=ROOT_VALUE
EXPECTED=EXPECTED_VALUE
FIXED=FIXED_VALUE
fd=os.open('/',os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW|os.O_NONBLOCK)
for part in ROOT[1:].split('/'):
    new=os.open(part,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW|os.O_NONBLOCK,dir_fd=fd);os.close(fd);fd=new
info=os.fstat(fd)
assert stat.S_ISDIR(info.st_mode) and info.st_uid==os.getuid() and stat.S_IMODE(info.st_mode)==0o700
names=[];bad=[];modes={};total=0;too_many=False
with os.scandir(fd) as entries:
    for entry in entries:
        if len(names)>=512:too_many=True;break
        names.append(entry.name)
        s=os.stat(entry.name,dir_fd=fd,follow_symlinks=False)
        mode=oct(stat.S_IMODE(s.st_mode));modes[mode]=modes.get(mode,0)+1;total+=s.st_size
        if not(stat.S_ISREG(s.st_mode) and s.st_uid==os.getuid() and s.st_nlink==1 and s.st_dev==info.st_dev and stat.S_IMODE(s.st_mode)==0o600):
            bad.append(dict(name=entry.name,mode=mode,regular=stat.S_ISREG(s.st_mode),owner_equal=s.st_uid==os.getuid(),nlink=s.st_nlink,same_device=s.st_dev==info.st_dev))
        assert time.monotonic()<END
outputs={}
for name in FIXED:
    try:s=os.stat(name,dir_fd=fd,follow_symlinks=False)
    except FileNotFoundError:outputs[name]=None
    else:outputs[name]=dict(bytes=s.st_size,mode=oct(stat.S_IMODE(s.st_mode)),regular=stat.S_ISREG(s.st_mode),owner_equal=s.st_uid==os.getuid(),nlink=s.st_nlink)
    assert time.monotonic()<END
os.close(fd);assert time.monotonic()<END
out=dict(schema='openergy.prepare-read-only-state.v1',SYSTEM_version=list(sys.version_info[:3]),nice=os.getpriority(os.PRIO_PROCESS,0),entries_seen=len(names),scan_complete=not too_many,expected260=len(EXPECTED),missing_initial_names=sorted(set(EXPECTED)-set(names)),extra_names=sorted(set(names)-set(EXPECTED)),mode_counts=modes,metadata_rejections=bad,listed_total_bytes=total,fixed_outputs=outputs,target_writes=0,children=0,prepare_retried=False,acceptance=False)
raw=(json.dumps(out,sort_keys=True,separators=(',',':'),ensure_ascii=True,allow_nan=False)+'\\n').encode('ascii');assert len(raw)<=32768
assert os.write(1,raw)==len(raw);assert time.monotonic()<END
os._exit(0)
'''.replace('ROOT_VALUE',repr(ROOT)).replace('EXPECTED_VALUE',repr(sorted(expected))).replace('FIXED_VALUE',repr(fixed)).encode('ascii')
ast.parse(remote.decode('ascii'),feature_version=(3,9))
payload=BASE/'prepare-launch/prepare-state-diagnostic.py'
run=BASE/'prepare-state-diagnostic-run'
parent=BASE/'prepare-state-diagnostic-parent-exit.json'
for path in [payload,run,parent]:assert not os.path.lexists(path)
put(payload,remote,0o400)
put(OUT/'remote-read-only-state.py.txt',remote,0o600)
collector=BASE/'tools/transport-direct-signal.py'
raw=collector.read_bytes();assert len(raw)==28099 and hashlib.sha256(raw).hexdigest()=='f1fa00b2ec61f6e4996b5938388f345a8ddcfb205455bc4482ce25711824f371'
S=time.monotonic_ns();A=S+135000000000
argv=['/opt/homebrew/bin/python3.15','-I','-S','-B',str(collector),'--phase','prepare','--profile',str(BASE/'m1-profile.json'),'--profile-sha256','dc9ff86519970ad740961012b0740b1b1a7aab3135a6fe79e47aa0de071acf4f','--payload',str(payload),'--payload-sha256',hashlib.sha256(remote).hexdigest(),'--run-dir',str(run),'--collector-sha256',hashlib.sha256(raw).hexdigest(),'--envelope-start-ns',str(S),'--clock-sha256','67e4f9525b2d1b1496689808d08ab625fc88698bb1e2b049e463868f08e5303d']
p=subprocess.Popen(argv,stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,cwd='/',env={'PATH':'/usr/bin:/bin','LC_ALL':'C','LANG':'C'},shell=False,close_fds=True)
try:code=p.wait(timeout=max(0,(A-time.monotonic_ns())/1e9));T=time.monotonic_ns();returned=True
except subprocess.TimeoutExpired:code=p.poll();T=time.monotonic_ns();returned=code is not None
value=dict(schema='openergy.prepare-diagnostic-parent-exit.v1',operation='read-only fixed ROOT state; not prepare',started_ns=S,absolute_until_ns=A,actual_collector_pid_from_retained_handle=p.pid,actual_collector_exit=code,actual_wait_or_poll_returned=returned,post_actual_exit_observed_ns=T,strictly_before_absolute=returned and T<A,prepare_retried=False,target_writes=0)
put(parent,canonical(value),0o600)
put(OUT/'diagnostic-parent-exit.json',canonical(value),0o600)
print(canonical({k:value[k] for k in ['operation','actual_collector_exit','strictly_before_absolute','prepare_retried','target_writes']}).decode('ascii'),end='')
os._exit(0 if returned and code==0 and T<A else 1)
