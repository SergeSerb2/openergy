"""Owned one-shot isolation setup; no package or manifest execution."""
import gzip,hashlib,json,os,shutil,stat,subprocess,sys,time
from pathlib import Path
P=Path(__file__).resolve().parent
REPO=Path('/Users/serge/.t3/worktrees/openergy/t3-bc17f311')
R2=Path('/Users/serge/.t3/worktrees/openergy/t3-8c51bae6/docs/verification/release-priority-isolated-2026-10-23-t3-8c51bae6')
OLD=Path('/var/folders/s6/_07cr4w13vsg_myjkv9ldpqc0000gn/T/oe-sparkle-combined-20261023-a_es9dei')
FIRST=REPO/'docs/verification/release-priority-2026-10-23'
SEEDS=['artifacts/sparkle/Sparkle','checkouts/Sparkle','repositories/Sparkle-09d89c53']
PIN='eef1a539a373c1f1a320624b1130fc5de7b2e100'
PYTHON='/Users/serge/.local/bin/python3.11'
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def save(n,v):
 with (P/n).open('x') as f:json.dump(v,f,indent=2,sort_keys=True);f.write('\n')
def load(n):return json.loads((P/n).read_text())
def meta(p):
 s=p.lstat();r=dict(mode=stat.S_IMODE(s.st_mode),kind=stat.S_IFMT(s.st_mode),uid=s.st_uid,gid=s.st_gid,nlink=s.st_nlink,flags=s.st_flags,mtime_ns=s.st_mtime_ns,ctime_ns=s.st_ctime_ns,birthtime=s.st_birthtime)
 if p.is_symlink():r['link']=os.readlink(p)
 return r
def inventory(base):
 return {str(p.relative_to(base)):({'link':os.readlink(p)} if p.is_symlink() else dict(sha256=sha(p),bytes=p.stat().st_size)) for p in sorted(base.rglob('*')) if p.is_symlink() or p.is_file()}
def metadata(base):return {str(p.relative_to(base)):meta(p) for p in [base,*sorted(base.rglob('*'))]}
def git(*args):return subprocess.check_output(['/usr/bin/git',*args],cwd=REPO,env={**os.environ,'GIT_OPTIONAL_LOCKS':'0','GIT_CONFIG_NOSYSTEM':'1','GIT_CONFIG_GLOBAL':'/dev/null'}).decode().strip()
def source_inventory():return {n:sha(REPO/n) for n in git('ls-files','-z').split('\0') if n and not n.startswith('docs/verification/') and (REPO/n).is_file()}
def preservation():
 records={};folders={}
 for folder in [FIRST,R2]:
  folders[str(folder)]=dict(inventory=inventory(folder),metadata=metadata(folder))
  for item in json.loads((folder/'copy-ledger.json').read_text()):
   original=Path(item['private_original']); exported=folder/item['path'];data=exported.read_bytes()
   if 'original_sha256' in item:
    assert sha(original)==item['original_sha256'] and len(original.read_bytes())==item['original_bytes']
    assert hashlib.sha256(data).hexdigest()==item['exported_sha256']
    payload=gzip.decompress(data) if exported.suffix=='.gz' else data
    assert hashlib.sha256(payload).hexdigest()==item['payload_sha256']
   else:
    assert sha(original)==item['sha256'] and sha(exported)==item['compressed_sha256']
    assert gzip.decompress(data)==original.read_bytes()
   records[str(original)]=dict(sha256=sha(original),metadata=meta(original))
 for base in [Path('/private/tmp/oe-priority-package-sr3z_6fy'),Path('/private/tmp/oe-priority-isolated-t3-8c51bae6-nfye7e16')]:
  for p in sorted(base.iterdir()):
   if p.is_file():records[str(p)]=dict(sha256=sha(p),metadata=meta(p))
 return dict(old_cache={n:inventory(OLD/n) for n in ['swift','cargo']},old_metadata={n:metadata(OLD/n) for n in ['swift','cargo']},prior_folders=folders,private_records=records,source=source_inventory())
def seed_inventory():return {n:inventory(P/'swift'/n) for n in SEEDS}
def prepare():
 assert git('rev-parse','HEAD')=='3183a9ed0de957dd471c6e1e8a76d1331f190d22'
 assert git('diff','--name-only').splitlines()==['apps/macos/scripts/bundle.sh','docs/releasing.md']
 save('preservation-initial.json',preservation())
 assert load('preservation-initial.json')['old_cache']==json.loads(gzip.decompress((FIRST/'raw/cache-after-recovery.json.gz').read_bytes()))
 for n in ['swift-cache','swift-config','swift-security','swift-sdks','observe']:(P/n).mkdir(mode=0o700)
 shutil.copytree('/Users/serge/.cargo/registry',P/'cargo-home/registry',symlinks=True)
 for n in SEEDS:
  d=P/'swift'/n;d.parent.mkdir(parents=True,exist_ok=True);shutil.copytree(OLD/'swift'/n,d,symlinks=True)
 save('seed-copied-exact.json',seed_inventory())
 assert seed_inventory()=={n:inventory(OLD/'swift'/n) for n in SEEDS}
 c=P/'swift/checkouts/Sparkle';r=P/'swift/repositories/Sparkle-09d89c53'
 subprocess.run(['/usr/bin/git','-C',str(c),'config','--local','remote.origin.url',str(r)],check=True)
 a=c/'.git/objects/info/alternates';a.chmod(0o600);a.write_text(str(r/'objects')+'\n')
 old=json.loads((OLD/'swift/workspace-state.json').read_text());artifact=old['object']['artifacts'][0]
 assert artifact['source']['checksum']=='17e28312b8e18ab7cdbbe09a6fb28cc55a5479ec6c371dbc07cdecd2a14fd959'
 package=dict(identity='sparkle',kind='remoteSourceControl',location='https://github.com/sparkle-project/Sparkle',name='Sparkle')
 state=dict(version=7,object=dict(artifacts=[dict(kind={'xcframework':{}},packageRef=package,path=str(P/'swift/artifacts/sparkle/Sparkle/Sparkle.xcframework'),source=artifact['source'],targetName='Sparkle')],dependencies=[dict(basedOn=None,packageRef=package,state=dict(checkoutState=dict(revision=PIN,version='2.10.0'),name='sourceControlCheckout'),subpath='Sparkle')],prebuilts=[]))
 (P/'swift/workspace-state.json').write_text(json.dumps(state,indent=2)+'\n');save('generated-state.json',state)
 assert json.loads((REPO/'apps/macos/Package.resolved').read_text())['pins'][0]['state']==dict(revision=PIN,version='2.10.0')
 assert subprocess.check_output(['/usr/bin/git','-C',str(c),'rev-parse','HEAD']).decode().strip()==PIN
 assert not subprocess.check_output(['/usr/bin/git','-C',str(c),'status','--porcelain']).strip()
 for n in SEEDS:
  for p in (P/'swift'/n).rglob('*'):
   if p.is_symlink():assert p.resolve(strict=True).is_relative_to(P/'swift')
 save('seed-qualified-base.json',seed_inventory())
 assert sorted(p.name for p in (P/'swift').iterdir())==['artifacts','checkouts','repositories','workspace-state.json']
 assert not list((P/'cargo-target').iterdir())
 env=load('tools-environment.json');tc='/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin'
 env.update(CARGO_HOME=str(P/'cargo-home'),CARGO_TARGET_DIR=str(P/'cargo-target'),CARGO_NET_OFFLINE='true',CARGO_BUILD_JOBS='2',RUSTUP_HOME='/Users/serge/.rustup',RUSTUP_TOOLCHAIN='1.99.1',CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER=tc+'/clang',CC=tc+'/clang',AR=tc+'/ar',SWIFT_SCRATCH_PATH=str(P/'swift'),SWIFT_CACHE_PATH=str(P/'swift-cache'),SWIFT_CONFIG_PATH=str(P/'swift-config'),SWIFT_SECURITY_PATH=str(P/'swift-security'),OPENERGY_SWIFTPM_OUTER_SANDBOX='1',OPENERGY_SWIFT_BIN=tc+'/swift',PYTHON=PYTHON,PYTHONPATH=str(P/'observe'),OPENERGY_BUILD='4',OPENERGY_REQUIRE_UPDATES='1',GIT_OPTIONAL_LOCKS='0',GIT_CONFIG_NOSYSTEM='1',GIT_CONFIG_GLOBAL='/dev/null',GIT_TERMINAL_PROMPT='0',HOSTNAME='release-check')
 save('environment.json',env)
 flags=['--scratch-path',str(P/'swift'),'-c','release','--jobs','2','--disable-automatic-resolution','--arch','arm64','--build-system','native','--disable-sandbox','--cache-path',str(P/'swift-cache'),'--config-path',str(P/'swift-config'),'--security-path',str(P/'swift-security'),'--manifest-cache','local','--sdk',env['SDKROOT'],'--disable-keychain','--disable-netrc']
 recipe=dict(argv=[tc+'/swift','package',*flags,'--verbose','describe','--type','json'],cwd=str(REPO/'apps/macos'),environment=env,profile_sha256=sha(P/'package.sb'),source=source_inventory(),controller_scope='Actual project and pinned Sparkle manifest evaluation under final outer policy; no app build and no package invocation',build_flags=flags)
 save('manifest-recipe-1.json',recipe)
 save('setup-source.patch',{'patch':git('diff','--binary'),'sha256':hashlib.sha256(git('diff','--binary').encode()).hexdigest()})
 save('setup-first-errors.json',dict(source_patch_first_failure="apply_patch verification failed: Failed to find expected lines in docs/releasing.md; entire patch rejected; no source mutation. Corrected exact context before execution.",toolpath='/Applications/Xcode.app/Contents/Developer/usr/bin/xcrun does not exist; genuine /usr/bin/xcrun retained only where metadata tool requires it.',xcrun_no_cache='Read-only installed --no-cache SDK query exited 0 but attempted denied Darwin temp writes; exact stdout/stderr retained. No allowance widened.',non_execution='No manifest or package invoked by setup.'))
 print('SETUP READY',sha(P/'manifest-recipe-1.json'),flush=True)
if __name__=='__main__':
 os.umask(0o077);os.setpriority(os.PRIO_PROCESS,0,15)
 try:prepare()
 except BaseException:
  import traceback
  with (P/'first-error-prepare.txt').open('x') as f:f.write(traceback.format_exc())
  raise
