from pathlib import Path
import datetime,hashlib,json
r=Path('docs/verification/release-integration-2026-10-23/idle-tls-windows-build-r6');p=Path(json.loads((r/'private-evidence-location.json').read_text())['private_root'])
def digest(b):return hashlib.sha256(b).hexdigest()
def save(n,v):(r/n).write_text(json.dumps(v,indent=2)+'\n')
a=(p/'downloaded/shared-before.json').read_bytes();b=(p/'downloaded/shared-after.json').read_bytes();assert a==b
shared=json.loads((r/'evidence/downloaded/shared-before.json').read_text())
summary={'original_before_bytes':len(a),'original_before_sha256':digest(a),'original_after_bytes':len(b),'original_after_sha256':digest(b),'exact_snapshot_bytes_equal':True,'trees':[{k:v for k,v in t.items() if k!='entries'}|{'entries_sha256':digest(json.dumps(t['entries'],separators=(',',':')).encode())} for t in shared['trees']],'defaults':shared['defaults'],'tools':shared['tools'],'sharing':shared['sharing'],'old_failed_root_absent':shared['old_failed_root_absent'],'full_original_snapshots_retained_privately':True}
save('shared-input-summary.json',summary)
ledger=json.loads((r/'copy-ledger.json').read_text())
omitted={'downloaded/shared-before.json','downloaded/shared-after.json','downloaded/source-manifest.json'}
for item in ledger['files']:
 item['committed_copy']=item['path'] not in omitted
 if not item['committed_copy']:
  item['reason']='Concise evidence: full original retained privately; source manifest is also present at the directory root' if item['path'].endswith('source-manifest.json') else 'Concise evidence: full original retained privately; exact before/after equality and hashes in shared-input-summary.json'
  (r/'evidence'/item['path']).unlink()
save('copy-ledger.json',ledger)
prior=Path('/Users/serge/.t3/worktrees/openergy/t3-04c77b7a/docs/verification/release-integration-2026-10-23/idle-tls-windows-build-r5')
before=json.loads((r/'r5-preserved-before.json').read_text())
actual=[{'path':str(f.relative_to(prior)),'bytes':f.stat().st_size,'sha256':digest(f.read_bytes())} for f in sorted(prior.rglob('*')) if f.is_file()]
assert actual==before['files']
save('r5-preservation-result.json',{'prior_commit':before['commit'],'entire_tree_file_count':len(actual),'entire_tree_paths_bytes_and_sha256_equal':True,'original_guard_sha256':next(x['sha256'] for x in actual if x['path']=='controller-once.guard'),'r5_writes':0,'r5_executions':0,'old_guard_reset':False,'r5_first_error':'git archive exit 128: fatal: pathspec Cargo.toml did not match any files; failed locally before SSH','r5_amended_controller_previous_execution_state':'UNEXECUTED; copied and reviewed as R6 starting point only'})
commands=[json.loads(f.read_text()) for f in (p/'raw').glob('*attempt-*.json')]
assert len(commands)==7 and all(c['exit_code']==0 and not c['timed_out'] for c in commands)
clippy=json.loads((p/'downloaded/clippy-result.json').read_text());release=json.loads((p/'downloaded/release-result.json').read_text())
post=json.loads((p/'downloaded/postflight.json').read_text());build=json.loads((p/'downloaded/build-result.json').read_text())
assert [clippy['name'],release['name']]==['clippy','release']
assert all(s['exit_code']==0 and not s['timed_out'] and not s['stopped'] for s in (clippy,release))
artifact=p/'downloaded/target/x86_64-pc-windows-msvc/release/examples/dns_queue_cancel_c4806a3d.exe'
assert artifact.stat().st_size==build['artifact']['bytes'] and digest(artifact.read_bytes())==build['artifact']['sha256']
samples=[]
for n in ['clippy','release']:
 samples+=json.loads((p/f'downloaded/{n}-processes.json').read_text())
assert len(samples)==104 and all(s['priority']=='BelowNormal' and s['session']==0 for s in samples)
assert all(any(s['name']==n for s in samples) for n in ['cargo.exe','rustc.exe','link.exe'])
save('build-only-stop-audit.json',{'source_commit':build['source_commit'],'runtime_invocations':0,'app_daemon_example_invocations':0,'artifact_executed':False,'strict_release_clippy_invocations':1,'release_build_invocations':1,'clippy_exit':clippy['exit_code'],'release_exit':release['exit_code'],'control_and_transfer_commands':len(commands),'failed_commands':0,'setup_corrections_or_resumes':0,'watchdog_events':0,'processes_stopped':0,'compiler_process_observations':len(samples),'all_observed_build_processes_below_normal_session_0':True,'parent_chain_identity_samples_retained':True,'shared_input_snapshots_byte_equal':True,'postflight_passed':post['passed'],'artifact_download_bytes_sha256_verified_before_cleanup':True,'artifact_bytes':build['artifact']['bytes'],'artifact_sha256':build['artifact']['sha256'],'new_root_removed':True,'removed_other_roots':False,'product_source_edits':False,'r5_tree_and_guard_preserved':True,'runtime_or_strict_deadline_acceptance_claimed':False,'no_test_or_mock_added':True,'no_delegation':True})
# Check for accidental private prefixes without printing their values.
state=json.loads((p/'state.json').read_text());needles=[state['profile'],state['profile'].replace('\\','\\\\'),state['profile'].replace('\\','/'),str(p),'sergecode-windows']
leaks=[]
for f in r.rglob('*'):
 if f.is_file() and f.name!='private-evidence-location.json':
  txt=f.read_text()
  if any(n in txt for n in needles):leaks.append(str(f.relative_to(r)))
assert not leaks,leaks
print('Verified: 7 successful control/transfer receipts, 2 compiler commands, 104 priority/session observations, exact artifact hash, R5 byte preservation, no private prefix leaks.')
print('Concise evidence bytes:',sum(f.stat().st_size for f in r.rglob('*') if f.is_file() and f.name!='private-evidence-location.json'))
