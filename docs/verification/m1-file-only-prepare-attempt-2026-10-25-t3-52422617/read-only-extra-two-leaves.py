"""Read only the two concrete names from the completed layout diagnostic."""
from pathlib import Path
import ast
import hashlib
import json
import os
import subprocess
import time

OUT = Path(__file__).resolve().parent
BASE = Path('/Users/serge/.local/state/openergy-verification/m1-source-metadata-launch-prepared-20261025-7ae23486940a')
REMOTE = r'''import os,sys,time,stat,json,hashlib,base64,signal
os.umask(0o077);os.setpriority(os.PRIO_PROCESS,0,15)
assert sys.version_info[:3]==(3,9,6) and sys.argv==['-'] and sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode and not sys.flags.optimize
END=time.monotonic()+20.0
def late(*args):raise TimeoutError('fixed two-leaf read deadline')
signal.signal(signal.SIGALRM,late);signal.setitimer(signal.ITIMER_REAL,20.0)
ROOT='/Users/m1-dev/.local/state/openergy-verification/m1-ordinary-reviewed-flow-20261025-4b20357c1503'
d=os.open('/',os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW|os.O_NONBLOCK)
for part in ROOT[1:].split('/'):
 n=os.open(part,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW|os.O_NONBLOCK,dir_fd=d);os.close(d);d=n
root=os.fstat(d);assert root.st_uid==os.getuid() and stat.S_IMODE(root.st_mode)==0o700
rows=[]
for name in [':-','coverage-policy.json']:
 f=os.open(name,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK,dir_fd=d)
 try:
  old=os.fstat(f);assert stat.S_ISREG(old.st_mode) and old.st_uid==os.getuid() and old.st_nlink==1 and old.st_dev==root.st_dev and stat.S_IMODE(old.st_mode)==0o600 and 0<=old.st_size<=8192
  raw=os.read(f,8193);assert len(raw)==old.st_size and os.read(f,1)==b''
  now=os.fstat(f);linked=os.stat(name,dir_fd=d,follow_symlinks=False)
  fields=lambda s:(s.st_dev,s.st_ino,s.st_mode,s.st_uid,s.st_nlink,s.st_size,s.st_mtime_ns,s.st_ctime_ns)
  assert fields(old)==fields(now)==fields(linked)
  rows.append(dict(name=name,bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest(),mode='0o600',base64=base64.b64encode(raw).decode('ascii')))
 finally:os.close(f)
 assert time.monotonic()<END
os.close(d);assert time.monotonic()<END
raw=(json.dumps(dict(schema='openergy.prepare-extra-two-leaf-read.v1',files=rows,target_writes=0,native_commands=0,prepare_retries=0),sort_keys=True,separators=(',',':'))+'\n').encode('ascii');assert len(raw)<=32768
assert os.write(1,raw)==len(raw);assert time.monotonic()<END
os._exit(0)
'''.encode('ascii')


def canonical(value):
    return (json.dumps(value,sort_keys=True,separators=(',',':'))+'\n').encode('ascii')


def put(path, raw, mode=0o600):
    f=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,mode)
    try:assert os.write(f,raw)==len(raw);os.fsync(f)
    finally:os.close(f)
    d=os.open(path.parent,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
    try:os.fsync(d)
    finally:os.close(d)


os.umask(0o077)
os.setpriority(os.PRIO_PROCESS,0,15)
ast.parse(REMOTE.decode('ascii'),feature_version=(3,9))
payload=BASE/'prepare-launch/prepare-extra-two-leaf-read.py'
run=BASE/'prepare-extra-two-leaf-read-run'
parent=BASE/'prepare-extra-two-leaf-read-parent-exit.json'
for path in [payload,run,parent,OUT/'remote-extra-two-leaf-read.py.txt']:
    assert not os.path.lexists(path)
put(payload,REMOTE,0o400)
put(OUT/'remote-extra-two-leaf-read.py.txt',REMOTE)
collector=BASE/'tools/transport-direct-signal.py'
sha=lambda raw:hashlib.sha256(raw).hexdigest()
assert sha(collector.read_bytes())=='f1fa00b2ec61f6e4996b5938388f345a8ddcfb205455bc4482ce25711824f371'
S=time.monotonic_ns()
A=S+135000000000
argv=['/opt/homebrew/bin/python3.15','-I','-S','-B',str(collector),
      '--phase','prepare','--profile',str(BASE/'m1-profile.json'),
      '--profile-sha256','dc9ff86519970ad740961012b0740b1b1a7aab3135a6fe79e47aa0de071acf4f',
      '--payload',str(payload),'--payload-sha256',sha(REMOTE),'--run-dir',str(run),
      '--collector-sha256',sha(collector.read_bytes()),'--envelope-start-ns',str(S),
      '--clock-sha256','67e4f9525b2d1b1496689808d08ab625fc88698bb1e2b049e463868f08e5303d']
PROCESS=subprocess.Popen(argv,stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,
    stderr=subprocess.DEVNULL,cwd='/',env={'PATH':'/usr/bin:/bin','LC_ALL':'C','LANG':'C'},
    shell=False,close_fds=True)
try:
    code=PROCESS.wait(timeout=max(0,(A-time.monotonic_ns())/1e9))
    T=time.monotonic_ns()
    returned=True
except subprocess.TimeoutExpired:
    code=PROCESS.poll()
    T=time.monotonic_ns()
    returned=code is not None
value=dict(schema='openergy.prepare-extra-two-leaf-parent-exit.v1',started_ns=S,
    absolute_until_ns=A,actual_collector_pid_from_retained_handle=PROCESS.pid,
    actual_collector_exit=code,actual_wait_or_poll_returned=returned,
    post_actual_exit_observed_ns=T,strictly_before_absolute=returned and T<A,
    target_writes=0,prepare_retries=0)
put(parent,canonical(value))
put(OUT/'extra-two-leaf-parent-exit.json',canonical(value))
assert os.write(1,canonical({k:value[k] for k in ['actual_collector_exit',
    'strictly_before_absolute','target_writes','prepare_retries']}))>0
os._exit(0 if code==0 and returned and T<A else 1)
