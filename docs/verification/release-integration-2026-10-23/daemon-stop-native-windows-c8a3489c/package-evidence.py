"""Read back preserved evidence. No compiler, product, transport, or remote command.
Recreate each derived copy from its private original. Retain failures and binaries privately.
"""
import ast
from collections import Counter
import hashlib
import json
import re
import base64
from pathlib import Path
import struct

HERE=Path(__file__).resolve().parent
private=Path(json.loads((HERE/'private-evidence-location.json').read_text())['private_root'])
def sha(b):return hashlib.sha256(b).hexdigest()
def read(path):return json.loads(path.read_text(encoding='utf-8-sig'))
def save(path,value):path.write_text(mask_plain(json.dumps(value,indent=2)+'\n'))
state=read(private/'state.json')
transport=read(Path('/tmp/oe-stop-c8a3489c-transport-selection.json'))
profile=state['profile']
replacements=[(transport['ssh_alias'],'<SSH_TRANSPORT>'),(str(private),'<LOCAL_PRIVATE>'),(transport['r6_private_root'],'<R6_PRIVATE>'),(profile.replace('\\','\\\\'),'<PROFILE_JSON>'),(profile,'<PROFILE>'),(profile.replace('\\','/'),'<PROFILE_FORWARD_SLASH>')]
def mask_plain(s):
 for old,new in replacements:s=s.replace(old,new)
 return s

def mask(s):
 s=mask_plain(s)
 # Nested EncodedCommand is a transport wrapper, not a source change.
 # Retain the exact raw token privately and encode the masked inner script for review.
 def encoded(m):
  token=m.group(1)
  decoded=base64.b64decode(token).decode('utf-16le')
  return '-EncodedCommand '+base64.b64encode(mask(decoded).encode('utf-16le')).decode()
 return re.sub(r'-EncodedCommand ([A-Za-z0-9+/=]{80,})',encoded,s)
ledger=read(HERE/'copy-ledger.json')
# Cleanup-only audit receipts can finish after the main controller export.
# Add each exact original once. Do not replace an existing original binding.
known={e['path'] for e in ledger['files']}
for folder in (private/'raw',private/'downloaded'):
 for path in sorted(folder.rglob('*')):
  if not path.is_file() or path.suffix not in ('.json','.ps1','.cs','.stdout','.stderr','.guard','.toml','.log'):continue
  rel=str(Path(folder.name)/path.relative_to(folder))
  if rel in known:continue
  original=path.read_bytes();omit=path.name in ('shared-before.json','shared-after.json','source-manifest.json')
  ledger['files'].append({'path':rel,'original_bytes':len(original),'original_sha256':sha(original),'committed_copy':not omit})
  known.add(rel)
for e in ledger['files']:
 original=(private/e['path']).read_bytes()
 assert len(original)==e['original_bytes'] and sha(original)==e['original_sha256']
 if e['committed_copy']:
  copied=mask(original.decode('utf-8-sig')).encode()
  dest=HERE/'evidence'/e['path'];dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(copied)
  e.update(copy_bytes=len(copied),copy_sha256=sha(copied),byte_equal=original==copied)
ledger['method']='Exact raw originals retained privately. Derived UTF-8 copies remove BOM and replace known transport/profile/private-root prefixes only; line endings are preserved. Nested EncodedCommand tokens are recursively decoded, masked with the same prefixes, and re-encoded. Full shared snapshots and duplicate source manifest are omitted. No original is changed.'
save(HERE/'copy-ledger.json',ledger)

source=read(HERE/'source-manifest.json')
source_proofs=[]
for name in ('source-before.json','source-after-build.json','source-after.json'):
 p=private/'downloaded'/name
 if not p.exists():continue
 value=read(p);assert value['source_commit']==source['source_commit'] and value['files']==len(source['files'])
 actual={x['path']:x for x in value['actual']};assert len(actual)==len(source['files'])
 for e in source['files']:
  a=actual[e['path']]
  assert all(a[k]==e[k] for k in ('bytes','sha256','crlf_pairs','lf_bytes')) and a['readonly']
 assert value['archive_sha256']==source['archive_sha256'] and value['manifest_sha256']==sha((private/'source-manifest.json').read_bytes())
 source_proofs.append(name)
assert (private/'source.tar').stat().st_size==source['archive_bytes'] and sha((private/'source.tar').read_bytes())==source['archive_sha256']
assert (HERE/'source-manifest.json').read_bytes()==(private/'source-manifest.json').read_bytes()

before=private/'downloaded/shared-before.json';after=private/'downloaded/shared-after.json'
shared_equal=before.exists() and after.exists() and before.read_bytes()==after.read_bytes()
controller_sha=sha((HERE/'controller.py').read_bytes())
receipts=[read(p) for p in sorted((private/'raw').glob('*.json'))]
assert all(r['controller_sha256']==controller_sha for r in receipts)
assert (private/('controller-'+controller_sha+'.py')).read_bytes()==(HERE/'controller.py').read_bytes()
for p in sorted((private/'raw').glob('*.json')):
 r=read(p)
 for stream in ('stdout','stderr'):assert sha(p.with_suffix('.'+stream).read_bytes())==r[stream+'_sha256']
 if r['script_sha256']:assert sha(p.with_suffix('.ps1').read_bytes())==r['script_sha256']

build_path=private/'downloaded/build-result.json';build=read(build_path) if build_path.exists() else {}
commands=[];observations=[];races={};chains=[]
for step in build.get('steps',[]):
 name=step['name'];result=read(private/'downloaded'/f'{name}-result.json')
 commands.append(result['command'])
 assert result['command'][1:]==[name if name=='clippy' else 'build','--locked','--offline','--release','--target','x86_64-pc-windows-msvc','--jobs','2','-p','openergy-daemon','--bin','openergy']+(['--','-D','warnings'] if name=='clippy' else [])
 samples=read(private/'downloaded'/f'{name}-processes.json');observations+=samples
 races[name]=len(read(private/'downloaded'/f'{name}-observation-races.json'))
 assert all(s['priority']=='BelowNormal' and s['session']==0 for s in samples)
 assert result['root_identity']['priority']=='BelowNormal' and result['root_identity']['session']==0 and result['root_identity']['retained_handle']
 identities={}
 for item in samples:identities.setdefault(item['pid'],[]).append(item)
 root=result['cargo_pid'];missing=[]
 for sample in samples:
  seen=set();cursor=sample
  while cursor['pid']!=root:
   key=(cursor['pid'],cursor['creation_utc']);assert key not in seen;seen.add(key)
   candidates=[p for p in identities.get(cursor['parent_pid'],[]) if p['creation_utc']<=cursor['creation_utc']]
   if not candidates:missing.append({'pid':sample['pid'],'creation_utc':sample['creation_utc'],'missing_parent':cursor['parent_pid']});break
   cursor=max(candidates,key=lambda p:p['creation_utc'])
 chains.append({'step':name,'samples':len(samples),'missing_sampled_parent_chains':missing,'reused_sampled_pids':[{'pid':pid,'creation_identities':[p['creation_utc'] for p in values]} for pid,values in identities.items() if len(values)>1],'resolution':'Select the latest observed parent generation created no later than its child. Identity keys include PID and creation time.'})
if build.get('artifact'):
 artifact=private/'downloaded/target/x86_64-pc-windows-msvc/release/openergy.exe';data=artifact.read_bytes()
 assert len(data)==build['artifact']['bytes'] and sha(data)==build['artifact']['sha256']
 off=struct.unpack_from('<I',data,60)[0]
 assert data[:2]==b'MZ' and data[off:off+4]==b'PE\0\0' and struct.unpack_from('<H',data,off+4)[0]==0x8664 and struct.unpack_from('<H',data,off+24)[0]==0x20b

runtime_path=private/'downloaded/runtime-result.json';runtime=read(runtime_path) if runtime_path.exists() else {}
product_identities=[]
if runtime:
 for path in sorted((private/'downloaded').glob('runtime-*-identity.json')):
  value=read(path);assert value['priority']=='BelowNormal' and value['session']==0 and value['parent_pid']==runtime['controller']['pid'] and value['below_normal_at_creation'] and value['retained_handle']
  product_identities.append({'file':path.name,'pid':value['pid'],'creation_utc':value['creation_utc'],'priority':value['priority'],'session':value['session'],'primary_thread_suspended_at_identity_sample':value['primary_thread_suspended']})
if runtime.get('passed'):
 assert runtime['stops_sent']==1 and len(runtime['requests'])==4
 assert runtime['daemon_exit']['exit_code']==0 and runtime['daemon_exit']['independent_retained_os_handle_signalled'] and runtime['daemon_exit']['daemon_pid_absent'] and runtime['daemon_exit']['controller_alive']
 assert runtime['daemon_exit']['exit_first_observed_ticks']<=runtime['daemon_exit']['deadline_ticks']
 assert runtime['pipe_cleanup']['daemon_pipe_absent'] and runtime['pipe_cleanup']['observer_pipe_absent'] and runtime['product_handles_closed'] and not runtime['recovery']
 for req in runtime['requests']:assert req['request']['id']==req['response']['id']
 for path in (private/'downloaded').glob('runtime-*-identity.json'):
  value=read(path);assert value['priority']=='BelowNormal' and value['session']==0 and value['parent_pid']==runtime['controller']['pid'] and value['below_normal_at_creation']
 assert runtime['artifact_sha256_after']==build['artifact']['sha256'] and runtime['config_sha256_after']==runtime['config_sha256']
result=read(HERE/'result.json')
local_equal=read(HERE/'local-prior-preservation.json')['equal']
assert local_equal
for tree in read(private/'local-prior-before.json'):
 root=Path(tree['owner']);actual=[]
 for path in sorted(root.rglob('*')):
  if path.is_file() and '__pycache__' not in path.parts:
   actual.append({'path':str(path.relative_to(root)),'bytes':path.stat().st_size,'sha256':sha(path.read_bytes())})
 assert actual==tree['files'],'prior tree changed after controller completion'
cleanup=read(HERE/'final-owned-cleanup-audit.json')
assert cleanup['passed'] and cleanup['fresh_root_absent'] and not cleanup['private_pipe_names_present'] and all(not i['same_identity_remains'] for i in cleanup['owned_identities'])
assert cleanup['normal_stop_requests_sent']==0 and runtime['stops_sent']==0 and runtime['requests']==[]
for path in HERE.glob('*.py'):ast.parse(path.read_text())
leaks=[]
for path in (HERE/'evidence').rglob('*'):
 if path.is_file():
  text=path.read_text()
  if any(old in text for old,_ in replacements):leaks.append(str(path.relative_to(HERE)))
  for token in re.findall(r'-EncodedCommand ([A-Za-z0-9+/=]{80,})',text):
   inner=base64.b64decode(token).decode('utf-16le')
   if any(old in inner for old,_ in replacements):leaks.append(str(path.relative_to(HERE))+':encoded')
assert not leaks,leaks
save(HERE/'packaging-verification.json',{'passed':True,'script_sha256':sha(Path(__file__).read_bytes()),'evidence_result_passed':result['passed'],'source_proofs':source_proofs,'source_files':len(source['files']),'source_overlays':[],'shared_snapshot_bytes_equal':shared_equal,'local_prior_evidence_equal':local_equal,'controller_identity_matches_all_receipts':True,'transport_receipt_count':len(receipts),'compiler_invocation_count':len(commands),'compiler_commands':mask_plain(json.dumps(commands)),'process_observation_counts':dict(Counter(s['name'] for s in observations)),'process_observation_races':races,'sampled_parent_chains':chains,'linker_observations':[s for s in observations if s['name']=='link.exe'],'priority_limit':'Only observed native compiler identities establish priority. No unsampled final-linker priority is inferred.','runtime_product_identities':product_identities,'final_cleanup_audit_passed':cleanup['passed'],'runtime_attempts':result['runtime_stage_attempts'],'runtime_passed':runtime.get('passed',False),'private_prefix_scan_passed':True,'remote_or_compiler_commands_by_packager':0,'exact_originals_and_artifact_retained_privately':True})
# The validation above is local evidence checking, not independent runtime reproduction.
files=[p for p in HERE.rglob('*') if p.is_file() and p != HERE/'artifacts.json' and p.name != 'private-evidence-location.json' and '__pycache__' not in p.parts]
save(HERE/'artifacts.json',{'files':[{'path':str(p.relative_to(HERE)),'bytes':p.stat().st_size,'sha256':sha(p.read_bytes())} for p in sorted(files)]})
print('LOCAL_EVIDENCE_PACKAGE_VERIFIED','RESULT_PASS' if result['passed'] else 'RESULT_FAIL_OR_UNVERIFIED')
print('Observed compiler processes:',dict(Counter(s['name'] for s in observations)))
