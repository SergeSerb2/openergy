import pathlib,json,hashlib,re,datetime
ptr=json.loads(pathlib.Path('/private/tmp/oe-win-dns-8281eb7-private-pointer.json').read_text());p=pathlib.Path(ptr['local']);raw=p/'raw';s=json.loads((p/'state.json').read_text());dest=pathlib.Path.cwd()/'docs/verification/release-integration-2026-10-23/windows-dns-final-8281eb7';dest.mkdir(exist_ok=False)
def sha(f):return hashlib.sha256(pathlib.Path(f).read_bytes()).hexdigest()
def read(f):return json.loads(pathlib.Path(f).read_text(encoding='utf-8-sig'))
def write(rel,v):f=dest/rel;f.parent.mkdir(parents=True,exist_ok=True);f.write_text(json.dumps(v,indent=2)+'\n');return f
roots=[s['root']]+[r['root'] for r in s['prior_fixtures']]
profile=s['root'].split('\\AppData\\')[0]
replacements=[(str(p),'<LOCAL_PRIVATE>'),(ptr['prior'],'<LOCAL_PRIOR_DNS>'),(str(pathlib.Path.cwd()),'<WORKTREE>'),('<LOCAL_PRIOR_OBSERVED>','<LOCAL_PRIOR_OBSERVED>')]
for i,r in enumerate(roots):
 tag='<DNS_FIXTURE>' if i==0 else '<PRIOR_FIXTURE_'+str(i)+'>'
 for a in [r,r.replace('\\','/'),r.replace('\\','\\\\')]:replacements.append((a,tag))
for a in [profile,profile.replace('\\','/'),profile.replace('\\','\\\\')]:replacements.append((a,'<USERPROFILE>'))
replacements.append(('<SSH_ALIAS>','<SSH_ALIAS>'))
replacements.sort(key=lambda r:len(r[0]),reverse=True)
ledger=[]
def copy(origin,rel):
 b=pathlib.Path(origin).read_bytes();text=b.decode('utf-8-sig').replace('\r\n','\n')
 for a,z in replacements:text=text.replace(a,z)
 f=dest/rel;f.parent.mkdir(parents=True,exist_ok=True);f.write_text(text)
 ledger.append({'private_original':str(pathlib.Path(origin).relative_to(p)),'public_copy':rel,'original_sha256':hashlib.sha256(b).hexdigest(),'public_sha256':sha(f),'transforms':'UTF-8 BOM removal if present; CRLF to LF; known private path/user-profile/SSH-alias prefixes only'})
 return f
build=read(raw/'build-result.json');freeze=read(raw/'freeze.json');post=read(raw/'postflight.json');extra=read(raw/'extra-invariance.json');derived=read(p/'derived-runtime.json');runs=read(raw/'run-result.json');manifest=read(p/'source-manifest.json')
assert build['passed'] and post['passed'] and len(runs)==2
assert sha(raw/freeze['binary']['path'])==freeze['binary']['sha256']
for lib in freeze['libraries']:assert sha(raw/lib['path'])==lib['sha256']
for r,d in zip(runs,derived):assert r['name']==d['name'] and r['stdout_sha256']==d['raw_stdout_sha256']
copy(p/'source-inputs.json','source-inputs.json');copy(p/'source-manifest-metadata-clarification.json','source-manifest-metadata-clarification.json');copy(p/'derived-runtime.json','derived-runtime.json')
for name in ['normal-full','owned-conflict-full']:
 for ext in ['stdout','stderr']:copy(raw/'logs'/f'{name}.{ext}',f'outputs/{name}.{ext}')
 copy(raw/f'{name}-result.json',f'execution/{name}.json')
 copy(raw/f'{name}-private-identities-removed.json',f'execution/{name}-identities-removed.json')
for name in ['clippy','release']:
 for ext in ['stdout','stderr']:copy(raw/'logs'/f'{name}.{ext}',f'outputs/{name}.{ext}')
 copy(raw/f'{name}-child-observation.json',f'execution/{name}-children.json')
copy(raw/'owned-conflict.json','execution/owned-conflict.json');copy(raw/'owned-conflict-disposed.json','execution/owned-conflict-disposed.json')
copy(raw/'native-tools-before.json','native-tools-before.json');copy(raw/'native-tools-after.json','native-tools-after.json')
for name in ['common.ps1','audit.ps1','build.ps1','runtime-common.ps1','run.ps1','postflight.ps1','final-audit.ps1','final-input-hashes.json','final-audit-input-hashes.json']:
 copy(raw/name,'workflow/'+name)
for label in ['authenticated-readonly-preflight','setup','cache-discovery','build-attached','fixed-two-runs-attached','final-postflight-attached']:
 for ext in ['transport.json','stdout','stderr']:
  if (p/f'{label}.{ext}').exists():copy(p/f'{label}.{ext}',f'transports/{label}.{ext}')
artifacts={'source_commit':build['source_commit'],'binary':freeze['binary'],'libraries':freeze['libraries'],'artifact_files':freeze['artifact_files'],'artifact_manifest_sha256':freeze['artifact_manifest_sha256'],'full_private_manifest':'raw/artifact-manifest.json','source_freeze':freeze['source']}
write('artifacts.json',artifacts)
child_summary=[]
for name in ['clippy','release']:
 c=read(raw/f'{name}-child-observation.json');child_summary.append({'step':name,'timed_out':c['timed_out'],'samples':len(c['samples']),'observed_name_session_priority':[{'name':n,'session':session,'priority':priority} for n,session,priority in sorted(set((x['name'],x['session'],x['priority']) for x in c['samples']))]})
public_runs=[]
for r,d in zip(runs,derived):
 public_runs.append({k:r[k] for k in ['name','started_utc','ended_utc','exit','elapsed_ms','timed_out','forced_cleanup','session','priority','binary_sha256','source_commit','stdout_sha256','stderr_sha256']} | {k:d[k] for k in ['literal_result','queue_pass_batches','queue_skipped_batches','obsolete_failure_events_in_queue_transcript','base_frame_exchanges','same_caller_recovery_exchanges','base_event_counts','refusal_matrix_verdict_counts','os_control_groups']} | {'timing_observations':len(d['observations']),'full_six_second_observations':sum(o['full_observation_retained'] for o in d['observations'].values())})
cleanup={k:post[k] for k in ['passed','recorded_utc','tls_identity_files_remaining','immutable_input_comparisons','original_global_files_unchanged','private_registry_source_files_unchanged','candidate_source','frozen_artifact_files_unchanged','workflow_inputs_unchanged','tool_defaults_unchanged','live_sharing_process_identity_unchanged','acl','temp_entries','case_count','case_exits']}
cleanup['extra_invariance']=extra
cleanup['inventory_summary']={k:post['inventory'][k] for k in ['recorded_utc','endpoint_query','endpoint_exit','tcp_rows','udp_rows','process_inventory_count']}
cleanup['fixture_processes_remaining']=len(post['inventory']['fixture_processes']);cleanup['prior_fixture_processes_remaining']=len(post['prior_fixture_processes']);cleanup['assigned_tcp_non_timewait_remaining']=post['inventory']['assigned_tcp_non_timewait'];cleanup['assigned_tcp_timewait_remaining']=post['inventory']['assigned_tcp_timewait'];cleanup['assigned_udp_endpoints_remaining']=post['inventory']['assigned_udp_endpoints']
write('cleanup.json',cleanup)
record={'schema_version':1,'date':datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=-7))).isoformat(),'status':'UNACCEPTED_FULL_ACCEPTANCE_UNVERIFIED','source_commit':build['source_commit'],'reviewed_source_commit':'9a4ac2eed77a8ed8bddd61e060be14a506df570a','exact_reviewed_hashes':read(p/'source-inputs.json')['exact_reviewed_hashes'],'source':{'archive_sha256':manifest['archive_sha256'],'full_manifest_sha256':sha(p/'source-manifest.json'),'regular_files':manifest['regular_files'],'git_exact_byte_matches':manifest['git_exact_byte_matches'],'checkout_conversions':manifest['checkout_conversions'],'before':build['source_before'],'after_build':build['source_after'],'after_commands':post['candidate_source'],'metadata_note':'The frozen source manifest included_parent field denotes the reviewed two-file reference only. See source-manifest-metadata-clarification.json; it is not an ancestry assertion.'},'build':{'target':'x86_64-pc-windows-msvc','rustc':build['tools_before']['rustc'],'cargo':build['tools_before']['cargo'],'wrapper_session':build['session'],'wrapper_priority':build['priority'],'jobs':build['jobs'],'steps':build['steps'],'passed':build['passed'],'actual_child_observations':child_summary,'private_offline_registry_files':build['private_cache_copy_files']},'runtime':public_runs,'artifact_binary':freeze['binary'],'artifact_manifest_sha256':freeze['artifact_manifest_sha256'],'private_raw_bundle':read(p/'bundle.json'),'cleanup_passed':post['passed'],'matching_strict_615c16e1_negative_control':'UNVERIFIED; no attempt in this task','source_or_predicate_changes':False,'runtime_command_invocations':2,'runtime_retries':0,'unit_tests_or_mocks':False,'computer_use':False,'delegation':False,'publication_push_or_GitHub_writes':False,'remaining_gates':['Strict short TCP expiry timing','Strict pending TLS expiry timing','Deadline spanning qualified fallback through expiry','Matching strict historical 615c16e1 negative control','Slow DNS head-of-line regression','Cache/expiry/timer/query/record bytes','Original security/provenance, IPC/Synergy3, mDNS expiry, broader release/native/positive migration gates remain outside this task']}
write('record.json',record)
# Hash all retained private originals. No private paths or file contents are copied by this index.
originals=[]
for f in sorted(p.rglob('*')):
 if f.is_file() and f.name!='private-originals-sha256.json' and '__pycache__' not in f.parts:originals.append({'path':str(f.relative_to(p)),'bytes':f.stat().st_size,'sha256':sha(f)})
write('private-originals-sha256.json',originals)
write('copy-ledger.json',ledger)
summary={'source':record['source'],'build':record['build'],'runtime':public_runs,'artifacts':artifacts,'cleanup':cleanup}
(p/'public-summary.json').write_text(json.dumps(summary,indent=2)+'\n')
print('Evidence copies prepared:',len(ledger));print('Destination:',dest);print(json.dumps(public_runs,indent=2));print('Cleanup:',post['passed'])
