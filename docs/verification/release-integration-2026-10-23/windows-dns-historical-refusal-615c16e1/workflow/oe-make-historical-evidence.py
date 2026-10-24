import ast,collections,datetime,hashlib,json,pathlib,re
p=pathlib.Path(json.loads(pathlib.Path('/private/tmp/oe-historical-refusal-pointer.json').read_text())['local']);raw=p/'raw';repo=pathlib.Path.cwd();dest=repo/'docs/verification/release-integration-2026-10-23/windows-dns-historical-refusal-615c16e1';dest.mkdir(exist_ok=True)
def sha(f):return hashlib.sha256(pathlib.Path(f).read_bytes()).hexdigest()
def read(f):return json.loads(pathlib.Path(f).read_text(encoding='utf-8-sig'))
state=read(p/'state.json');roots=[state['root']]+[r['root'] for r in state['prior_fixtures']];profile=state['root'].split('\\AppData\\')[0]
tree=ast.parse((p/'transport.py').read_text());alias=next(n.value.elts[5].value for n in ast.walk(tree) if isinstance(n,ast.Assign) and any(isinstance(t,ast.Name) and t.id=='argv' for t in n.targets))
replacements=[(str(p),'<LOCAL_PRIVATE>'),(str(repo),'<WORKTREE>'),(read(p/'provenance.json')['prior_local'],'<LOCAL_R12_PRIVATE>'),(alias,'<SSH_ALIAS>')]
for i,root in enumerate(roots):
 for text in [root,root.replace('\\','/'),root.replace('\\','\\\\')]:replacements.append((text,'<HISTORICAL_FIXTURE>' if i==0 else '<PRIOR_FIXTURE_'+str(i)+'>'))
for text in [profile,profile.replace('\\','/'),profile.replace('\\','\\\\')]:replacements.append((text,'<USERPROFILE>'))
replacements.sort(key=lambda x:len(x[0]),reverse=True)
def sanitize(text):
 text=text.replace('\r\n','\n')
 for a,b in replacements:text=text.replace(a,b)
 return text
ledger=[]
def write(rel,obj):
 f=dest/rel;f.parent.mkdir(parents=True,exist_ok=True);f.write_text(sanitize(json.dumps(obj,indent=2)+'\n'));return f
def copy(origin,rel):
 origin=pathlib.Path(origin);b=origin.read_bytes();f=dest/rel;f.parent.mkdir(parents=True,exist_ok=True);f.write_text(sanitize(b.decode('utf-8-sig')));ledger.append({'private_original':str(origin.relative_to(p)),'public_copy':rel,'original_bytes':len(b),'original_sha256':sha(origin),'public_sha256':sha(f),'transforms':'UTF-8 BOM removal if present, CRLF to LF, exact known private path/profile/SSH-alias prefix replacement only'});return f
build=read(raw/'build-result.json');post=read(raw/'postflight.json');derived=read(p/'derived-runtime.json');manifest=read(p/'source-manifest.json');oldmanifest=read(p/'historical-git-manifest.json');prov=read(p/'provenance.json')
assert post['passed'];assert sha(p/'raw-evidence.tar')==read(p/'bundle.json')['sha256'];assert sha(raw/'source.tar')==manifest['archive_sha256'];assert sha(raw/'source-manifest.json')==sha(p/'source-manifest.json')
base={f['path']:f for f in oldmanifest['files']};actual={f['path']:f for f in manifest['files']};probe=manifest['only_overlay'];assert set(base)==set(actual);assert all(base[f]['sha256']==actual[f]['sha256'] for f in base if f!=probe)
assert actual[probe]['sha256']=='530c880eb74f6c95d24a0612c4fa8773fa21d256765fc3f9f6260801fa303a30'
for f in ['source-manifest.json','historical-git-manifest.json','public-api-review.json']:copy(p/f,'source/'+f)
copy(p/'source-integrity-validation.json','source/source-integrity-validation.json')
copy(p/'overlay'/pathlib.Path(probe).name,'source/'+pathlib.Path(probe).name)
assert sha(dest/'source'/pathlib.Path(probe).name)==actual[probe]['sha256']
freeze=read(raw/'freeze.json') if (raw/'freeze.json').exists() else None
if freeze:
 assert sha(raw/freeze['binary']['path'])==freeze['binary']['sha256']
 for lib in freeze['libraries']:assert sha(raw/lib['path'])==lib['sha256']
 write('artifacts.json',freeze)
children=[]
for name in ['clippy','release']:
 for ext in ['stdout','stderr']:
  f=raw/'logs'/f'{name}.{ext}'
  if f.exists():copy(f,f'outputs/{name}.{ext}')
 f=raw/f'{name}-child-observation.json'
 if f.exists():
  c=read(f);copy(f,f'execution/{name}-children.json');children.append({'step':name,'sample_count':len(c['samples']),'timed_out':c['timed_out'],'observed_name_session_priority':[{'name':n,'session':s,'priority':v} for n,s,v in sorted(set((r['name'],r['session'],r['priority']) for r in c['samples']))]})
run=None
if (raw/'historical-refusal-result.json').exists():
 run=read(raw/'historical-refusal-result.json');assert sha(raw/'logs/historical-refusal.stdout')==run['stdout_sha256']==derived['raw_stdout_sha256']
 for ext in ['stdout','stderr']:copy(raw/'logs'/f'historical-refusal.{ext}',f'outputs/historical-refusal.{ext}')
 copy(raw/'historical-refusal-result.json','execution/historical-refusal.json')
 for f in ['private-identities-removed.json','postflight-identities-removed.json']:
  if (raw/f).exists():copy(raw/f,'execution/'+f)
 write('execution/identity-removal-inventory-status.json',{'separate_files_present':{f:(raw/f).exists() for f in ['private-identities-removed.json','postflight-identities-removed.json']},'postflight_runtime_identity_files_remaining':post['tls_identity_files_remaining'],'note':'No separate file was emitted by the PowerShell empty-array pipeline. Cleanup is verified by postflight.json; no content is synthesized.'})
for f in ['native-tools-before.json','native-tools-after.json','native-metadata-before.json']:copy(raw/f,f)
for f in ['common.ps1','audit.ps1','runtime-common.ps1','build.ps1','run.ps1','postflight.ps1','final-input-hashes.json']:copy(raw/f,'workflow/'+f)
for f in ['oe-prepare-historical-refusal.py','oe-setup-historical-refusal.py','oe-make-historical-workflow.py','oe-launch-historical-build.py','oe-run-historical-refusal.py','oe-finish-historical-refusal.py','oe-analyze-historical-refusal.py','oe-make-historical-evidence.py','transport.py','oe-extract-historical-evidence.py']:
 if (p/f).exists():copy(p/f,'workflow/'+f)
for label in ['authenticated-readonly-preflight','setup','cache-discovery','build-attached','one-historical-refusal-attached','postflight-attached','raw-bundle']:
 for ext in ['transport.json','stdout','stderr']:
  f=p/f'{label}.{ext}'
  if f.exists():copy(f,f'transports/{label}.{ext}')
for f in ['local-extraction-first-failure.stderr','local-extraction-first-failure.json','local-extraction-recovery.json','local-copy-first-failure.json']:copy(p/f,'local-evidence/'+f)
for f in ['transfer-build.json','download.json']:copy(p/f,'transports/'+f)
publicderived=json.loads(json.dumps(derived))
for o in publicderived.get('observations',{}).values():
 calls=o.pop('calls');o['first_call']=calls[0];o['last_call']=calls[-1];o['full_call_transcript']='outputs/historical-refusal.stdout'
write('derived-runtime.json',publicderived)
cleanup={k:post[k] for k in ['passed','recorded_utc','immutable_input_comparisons','original_global_files_unchanged','private_registry_source_files_unchanged','historical_source','frozen_artifact_files_unchanged','workflow_inputs_unchanged','native_tools_bytes_flags_acls_unchanged','tool_defaults_unchanged','live_sharing_process_identity_unchanged','acl','temp_entries','tls_identity_files_remaining','case_count','case_exits']}
cleanup['inventory_summary']={k:post['inventory'][k] for k in ['recorded_utc','endpoint_query','endpoint_exit','tcp_rows','udp_rows','process_inventory_count']};cleanup['fixture_processes_remaining']=len(post['inventory']['fixture_processes']);cleanup['prior_fixture_processes_remaining']=len(post['prior_fixture_processes']);cleanup['assigned_tcp_non_timewait_remaining']=post['inventory']['assigned_tcp_non_timewait'];cleanup['assigned_tcp_timewait_remaining']=post['inventory']['assigned_tcp_timewait'];cleanup['assigned_udp_endpoints_remaining']=post['inventory']['assigned_udp_endpoints'];write('cleanup.json',cleanup)
record={'recorded_local':datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=-7))).isoformat(),'parent_commit':prov['parent_commit'],'historical_commit':prov['historical_commit'],'scope':'One historical refusal-only negative control; no positive current probe or deadline/full-acceptance rerun.','matching_historical_refusal_gate':derived['matching_historical_refusal_gate'],'historical_fault_witness':derived['historical_fault_witness'],'source':{'historical_file_count':len(base),'export_file_count':len(actual),'unchanged_historical_files':len(base)-1,'only_overlay':probe,'probe_sha256':actual[probe]['sha256'],'original_historical_probe_sha256':base[probe]['sha256'],'archive_sha256':manifest['archive_sha256'],'manifest_sha256':sha(p/'source-manifest.json'),'historical_git_manifest_sha256':sha(p/'historical-git-manifest.json'),'before':build['source_before'],'after_build':build.get('source_after'),'after_runtime':post['historical_source'],'cargo_lock':base['Cargo.lock'],'reactor':base['crates/net/src/reactor.rs'],'line_ending_conversion':False,'library_changes':False,'shims':False,'synthetic_events_callbacks_or_mock_trace':False},'build':{'passed':build['passed'],'target':'x86_64-pc-windows-msvc','rustc':build['tools_before']['rustc'],'cargo':build['tools_before']['cargo'],'session':build['session'],'priority':build['priority'],'jobs':build['jobs'],'steps':build['steps'],'actual_child_observations':children,'private_cache_copy_files':build['private_cache_copy_files'],'private_current_user_only':True},'runtime':{k:run[k] for k in ['name','started_utc','ended_utc','exit','elapsed_ms','timed_out','forced_cleanup','session','priority','binary_sha256','source_commit','stdout_sha256','stderr_sha256']} if run else None,'runtime_command_invocations':1 if run else 0,'runtime_retries':0,'wrong_behavior_summary':{k:derived.get(k) for k in ['fixture_qualification','refusal_matrix_counts','false_connected_count','observation_count','full_six_second_observations','product_trace_count','missing_trace_is_fault_witness','literal_result']},'artifact_binary':freeze['binary'] if freeze else None,'private_raw_bundle':read(p/'bundle.json'),'cleanup_passed':post['passed'],'unit_tests':False,'mocks':False,'computer_use':False,'further_delegation':False,'new_credentials':False,'push_or_publication':False,'remaining_gates':['Strict TCP, fallback, and pending TLS deadline timing remain UNVERIFIED.','Full acceptance remains UNVERIFIED.','Slow DNS, cache/mDNS expiry, original IPC/Synergy3, security/provenance, provider-stopped tasks and broader release gates remain outside scope.']};record['local_copy_first_attempt']={'exit':1,'error':'FileNotFoundError for absent empty identity-removal inventory','recovery':'Preserve absence explicitly and use the successful postflight identity count','remote_or_product_retry':False};record['local_evidence_extraction']={'first_attempt_exit':1,'error':"TypeError: extractall() got an unexpected keyword argument 'filter'",'recovered_from_verified_bundle':True,'remote_or_product_retry':False};write('record.json',record)
# Hash retained private originals, including raw first failures and binary/library copies.
originals=[{'path':str(f.relative_to(p)),'bytes':f.stat().st_size,'sha256':sha(f)} for f in sorted(p.rglob('*')) if f.is_file() and '__pycache__' not in f.parts];write('private-originals-sha256.json',originals);write('copy-ledger.json',ledger)
(p/'public-evidence-location.json').write_text(json.dumps({'destination':str(dest),'report':str(dest.parent/(dest.name+'.md'))},indent=2)+'\n')
print(json.dumps({'gate':record['matching_historical_refusal_gate'],'runtime_commands':record['runtime_command_invocations'],'build_passed':build['passed'],'cleanup_passed':post['passed'],'evidence_copy_count':len(ledger),'wrong_behavior_summary':record['wrong_behavior_summary']},indent=2))
