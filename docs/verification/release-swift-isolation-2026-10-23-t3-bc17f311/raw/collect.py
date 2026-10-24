"""Read-only artifact/preservation audit and exact evidence export. Never builds or launches."""
from collections import Counter,defaultdict
import gzip,hashlib,json,os,plistlib,re,stat,subprocess,sys,zipfile
from pathlib import Path
from prepare import P,REPO,PYTHON,sha,save,load,preservation,source_inventory,seed_inventory
DEST=REPO/'docs/verification/release-swift-isolation-2026-10-23-t3-bc17f311'
def git(*args):return subprocess.check_output(['/usr/bin/git',*args],cwd=REPO,env={**os.environ,'GIT_OPTIONAL_LOCKS':'0','GIT_CONFIG_NOSYSTEM':'1','GIT_CONFIG_GLOBAL':'/dev/null'})
def digest(data):return hashlib.sha256(data).hexdigest()
def source_bindings():
 build=load('package-plan.json')['source_commit'];before=load('preservation-initial.json')['source'];tree={}
 for row in git('ls-tree','-r','-z',build).split(b'\0'):
  if row:
   header,name=row.split(b'\t',1);tree[name.decode()]=header.split()[2].decode()
 names=sorted(before)
 proc=subprocess.run(['/usr/bin/git','cat-file','--batch'],cwd=REPO,input=('\n'.join(tree[n] for n in names)+'\n').encode(),capture_output=True,check=True)
 data=proc.stdout;offset=0;bindings=[]
 for name in names:
  end=data.index(b'\n',offset);oid,kind,length=data[offset:end].split();assert kind==b'blob'
  offset=end+1;payload=data[offset:offset+int(length)];offset+=int(length)+1
  local=(REPO/name).read_bytes();assert digest(local)==before[name]
  conversion='identity'
  if payload!=local:
   assert name=='tools/windows/openergy-run.cmd'
   assert ': eol: crlf\n' in git('check-attr','eol','--',name).decode()
   assert payload.replace(b'\n',b'\r\n')==local;conversion='committed .gitattributes LF to CRLF checkout conversion'
  bindings.append(dict(path=name,git_blob=oid.decode(),git_sha256=digest(payload),worktree_sha256=digest(local),checkout_conversion=conversion))
 assert offset==len(data)
 changed=git('diff','--name-only','3183a9ed0de957dd471c6e1e8a76d1331f190d22',build).decode().splitlines()
 assert changed==['apps/macos/scripts/bundle.sh','docs/releasing.md']
 return dict(build_commit=build,initial_commit='3183a9ed0de957dd471c6e1e8a76d1331f190d22',changed_paths=changed,inputs=bindings)
def collect():
 assert not git('status','--porcelain').strip()
 plan=load('package-plan.json');result=load('package-result.json');before=load('preservation-initial.json')
 assert git('rev-parse','HEAD').decode().strip()==plan['source_commit']
 for filename,key in [('controller.py','controller_sha256'),('observe/sitecustomize.py','observer_sha256'),('package.sb','profile_sha256')]:assert sha(P/filename)==plan[key]
 assert preservation()==before==load('preservation-after-package.json')
 save('source-bindings.json',source_bindings())
 traces=[json.loads(l) for f in sorted(P.glob('trace-*.jsonl')) for l in f.read_text().splitlines()]
 commands=[r for r in traces if r['kind']=='real-command-return'];smokes=[r for r in traces if r['kind']=='smoke-return'];trees=[r for r in traces if r['kind']=='actual-payload-trees'];temp=[r for r in traces if r['kind']=='before-temp-cleanup']
 save('full-smoke.json',dict(smokes=smokes,commands=[r for r in commands if 'smoke' in r],daemon_starts=[r for r in traces if r['kind']=='daemon-started'],temporary_cleanup=temp))
 save('actual-signature-commands.json',[r for r in commands if any(Path(str(a)).name=='codesign' for a in (r.get('argv') or []))])
 save('actual-payload-trees.json',trees)
 artifacts=[dict(path=str(f),bytes=f.stat().st_size,sha256=sha(f)) for f in sorted((P/'output').glob('*')) if f.is_file()];save('artifacts.json',artifacts)
 zip_bindings={};archive=next(iter((P/'output').glob('*.zip')),None)
 if archive:
  with zipfile.ZipFile(archive) as z:
   assert z.testzip() is None
   manifest=z.read('BUILD.json');assert manifest==archive.with_suffix('.json').read_bytes();b=json.loads(manifest)
   assert b['commit']==plan['source_commit'] and not b['dirty'] and b['signing']=='adhoc' and not b['notarized'] and b['target']=='aarch64-apple-darwin' and b['rust']=='1.99.1'
   assert archive.with_name(archive.name+'.sha256').read_text()==sha(archive)+'  '+archive.name+'\n'
   info=plistlib.loads(z.read('Openergy.app/Contents/Info.plist'));template=plistlib.loads((REPO/'apps/macos/Support/Info.plist').read_bytes())
   assert all(info[k]==template[k] for k in ['SUFeedURL','SUPublicEDKey','LSMinimumSystemVersion']) and info['SUEnableAutomaticChecks'] is True
   entries=[]
   for i in z.infolist():
    data=z.read(i);mode=i.external_attr>>16;item=dict(name=i.filename,mode=mode,bytes=i.file_size,sha256=digest(data))
    if stat.S_ISLNK(mode):item['link']=data.decode()
    entries.append(item)
   save('archive-entries.json',entries)
   assert len(trees)==1 and trees[0]['staged'] and trees[0]['staged'].keys()==trees[0]['extracted'].keys()
   mode_diffs=[]
   for n,a in trees[0]['staged'].items():
    btree=trees[0]['extracted'][n]
    assert a.get('sha256')==btree.get('sha256') and a.get('link')==btree.get('link')
    if a['mode']!=btree['mode']:mode_diffs.append(dict(path=n,staged=a['mode'],extracted=btree['mode']))
    assert bool(a['mode']&0o111)==bool(btree['mode']&0o111)
   zip_bindings=dict(manifest=json.loads(manifest),embedded_manifest_equals_sidecar=True,checksum_sidecar_matches=True,production_feed_key_defaults_preserved=True,automatic_checks=True,entries=len(entries),links=sum('link'in i for i in entries),staged_extracted_bytes_links_equal=True,staged_extracted_executable_bits_equal=True,mode_normalizations=mode_diffs,app_sha256=next(i['sha256'] for i in entries if i['name']=='Openergy.app/Contents/MacOS/Openergy'),daemon_sha256=next(i['sha256'] for i in entries if i['name']=='Openergy.app/Contents/Helpers/openergy'))
 save('artifact-bindings.json',zip_bindings)
 samples=load('package-samples.json');observed=[r for s in samples for r in s['owned']];births=[r for r in traces if r['kind']=='actual-subprocess-start'];identities=[r['identity'] for r in births if r.get('identity')]
 assert all(r['nice']==15 and r['ps_nice']==15 for r in observed) and all(r['nice']==15 for r in identities)
 bycommand=defaultdict(set)
 for r in observed:bycommand[Path(r['comm']).name.strip('()')].add(r['pid'])
 gaps=[b['elapsed_seconds']-a['elapsed_seconds'] for a,b in zip(samples,samples[1:])]
 coverage=dict(samples=len(samples),observations=len(observed),observed_pids=len({r['pid'] for r in observed}),nice_values=sorted({r['nice'] for r in observed}|{r['nice'] for r in identities}),command_pid_counts={n:len(v) for n,v in bycommand.items()},selected_observations={label:[r for r in observed if Path(r['comm']).name.strip('()') in names] for label,names in dict(rust_compilers=['rustc'],swift_frontends=['swift-frontend'],clang_compilers=['clang','cc'],linkers=['ld'],codesign=['codesign'],packaged_daemons=['openergy']).items()},actual_subprocess_starts=births,first_seconds=samples[0]['elapsed_seconds'] if samples else None,last_seconds=samples[-1]['elapsed_seconds'] if samples else None,max_gap_seconds=max(gaps,default=None),limits='Actual identity samples and launch observations only. Short processes and all unsampled intervals remain unproven. No unrelated PID was changed.')
 save('priority-coverage.json',coverage)
 success=result['exit']==0 and not result['timeout'] and not result['controller_error'] and not result['remaining']
 if success:
  assert len(smokes)==2 and all(r['daemon_exit']==0 for r in smokes) and archive
  for flow in smokes:
   seen=[r for r in commands if 'smoke' in r and r['smoke']['binary']==flow['binary']]
   assert any(r.get('exit')==0 and any(str(a)=='version' for a in r.get('argv',[])) for r in seen)
   assert any(r.get('exit')==1 and any(str(a)=='status' for a in r.get('argv',[])) for r in seen)
   assert any(r.get('exit')==0 and any(str(a)=='status' for a in r.get('argv',[])) for r in seen)
   assert any(r.get('exit')==1 and any(str(a)=='release.invalid.method' for a in r.get('argv',[])) for r in seen)
   assert any(r.get('exit')==0 and any(str(a)=='daemon.stop' for a in r.get('argv',[])) for r in seen)
 assert all(not Path(r['directory']).exists() for r in temp)
 for n in ['ports-preflight-first.json','ports-postflight-first.json']:assert all(r['first_exclusive_bind'] for r in load(n)['attempts'])
 summary=dict(overall='PASS: one qualified native package and both real staged/extracted CLI virtual IPC flows' if success else 'FAILED: one package; no retry',package_invocations=1,manifest_invocations=len(list(P.glob('manifest-*.guard'))),manifest_budget=3,package=result,source_commit=plan['source_commit'],final_source_unchanged=True,oldcache_entries={n:len(v) for n,v in before['old_cache'].items()},selected_metadata_entries={n:len(v) for n,v in before['old_metadata'].items()},prior_folders=len(before['prior_folders']),prior_private_records=len(before['private_records']),source_inputs=len(before['source']),preservation_all_equal=True,artifacts=artifacts,artifact_bindings=zip_bindings,smoke_completions=len(smokes),smoke_daemon_exits=[r['daemon_exit'] for r in smokes],observed_nice_values=coverage['nice_values'],observed_pids=coverage['observed_pids'],sample_count=coverage['samples'],observation_count=coverage['observations'],max_sample_gap_seconds=coverage['max_gap_seconds'],cleanup=load('package-cleanup.json'),private_root=str(P),limits=['Old failed runs remain FAILED and immutable; original metadata restoration remains UNVERIFIED.','No App UI, public nil-plist entry runtime, private fixture/launcher, updater, installed target or release clearance proof.','New opt-in requires independent source review before general use; normal defaults unchanged.','Rust objcopy and other toolchain warnings retained verbatim; no toolchain substitution.','Read-only xcrun metadata query may emit denied cache-write diagnostics; no outer policy widening.','No GUI, native input/clipboard, production config/TLS/default IPC, live sharing, 24800/5353, service, Keychain identity, keys, notarization/GK/LS, Docker, push/publication or external network fixture.','New metadata baseline includes modes, owner/group, kind, nlink, flags, mtime, ctime, birthtime and symlink targets; read atime excluded. ACLs/xattrs and original prebreach restoration not verified.','No unit tests, mocks, computer use or delegation. Provider-stopped tasks untouched.'])
 save('results.json',summary)
 DEST.mkdir(mode=0o700);(DEST/'raw').mkdir()
 inputs=[f for f in sorted(P.iterdir()) if f.is_file()]+[P/'observe/sitecustomize.py']
 ledger=[]
 for path in inputs:
  original=path.read_bytes();payload=original;transform='exact original bytes'
  if path.name in ['ports-preflight-first.json','ports-postflight-first.json']:
   record=json.loads(payload);record['lsof_before']['stderr']=re.sub(r'(?<=file system )[^\n]+','<unrelated mounted filesystem path redacted>',record['lsof_before']['stderr']);payload=(json.dumps(record,indent=2,sort_keys=True)+'\n').encode();transform='Redact unrelated mount path only; exact private original retained'
  relative=Path('raw')/('observe/sitecustomize.py' if path.parent.name=='observe' else path.name)
  if path.suffix not in ['.py','.sb']:
   exported=gzip.compress(payload,mtime=0);relative=Path(str(relative)+'.gz');transform+='; gzip mtime 0'
  else:exported=payload
  dest=DEST/relative;dest.parent.mkdir(parents=True,exist_ok=True)
  with dest.open('xb') as f:f.write(exported)
  ledger.append(dict(path=str(relative),private_original=str(path),original_sha256=digest(original),original_bytes=len(original),payload_sha256=digest(payload),exported_sha256=digest(exported),exported_bytes=len(exported),transform=transform))
 with (DEST/'copy-ledger.json').open('x') as f:json.dump(ledger,f,indent=2,sort_keys=True);f.write('\n')
 with (DEST/'results.json').open('x') as f:json.dump(summary,f,indent=2,sort_keys=True);f.write('\n')
 print(json.dumps(dict(overall=summary['overall'],exports=len(ledger),destination=str(DEST),package_seconds=result['elapsed_seconds'],smoke_completions=len(smokes))),flush=True)
if __name__=='__main__':
 os.umask(0o077)
 try:collect()
 except BaseException:
  import traceback
  n='first-error-collect.txt'
  if not (P/n).exists():
   with (P/n).open('x') as f:f.write(traceback.format_exc())
  raise
