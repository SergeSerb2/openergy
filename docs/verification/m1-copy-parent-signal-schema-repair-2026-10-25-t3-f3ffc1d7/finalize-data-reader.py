"""Author-owned final public static packet reader; no supplied code runs.

Stage the owned folder first. Read actual Git whitespace diagnostics as data,
then write only owned evidence/manifest files. SHA256SUMS excludes itself;
packet-manifest excludes itself and SHA256SUMS to avoid circular identities.
"""
import hashlib
import json
import os
from pathlib import Path
import stat
import subprocess

OUT=Path('docs/verification/m1-copy-parent-signal-schema-repair-2026-10-25-t3-f3ffc1d7')


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def put(name,value):
    (OUT/name).write_text(json.dumps(value,indent=2,ensure_ascii=True,allow_nan=False)+'\n')
    (OUT/name).chmod(0o644)


def pairs(items):
    result={}
    for key,value in items:
        assert key not in result,key
        result[key]=value
    return result


def load(raw):
    def reject(value):
        raise ValueError(value)
    return json.loads(raw,object_pairs_hook=pairs,parse_constant=reject)


def check(pathspecs):
    command=['git','diff','--cached','--check','--',*pathspecs]
    result=subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE,
        env=dict(os.environ,GIT_OPTIONAL_LOCKS='0'))
    return dict(command=command,exit=result.returncode,stdout=result.stdout.decode(),stderr=result.stderr.decode())


full=check([str(OUT)])
no_patches=check([str(OUT),':(exclude)'+str(OUT/'*.patch')])
new_repair_patches=check([str(OUT/name) for name in ['from-85c668f-source.patch','from-85c668f-schema.patch','from-85c668f.patch']])
assert full['exit']==2 and no_patches['exit']==0 and new_repair_patches['exit']==0
assert full['stdout'].count('trailing whitespace.')==42
assert all('prior-from-cleared-source-launcher.patch' in line for line in full['stdout'].splitlines() if 'trailing whitespace.' in line)
put('whitespace-observations.json',dict(full=full,excluding_all_literal_patch_files=no_patches,
    new_repair_patches_only=new_repair_patches,
    explanation='Full exit2 is retained for42 literal single-space blank context rows in the byte-exact historical patch. Other-files and new-repair-patches exit0 are separate results. No patch bytes stripped.',
    original_author_exit2_record='prior-author-whitespace-check.json',
    original_independent_exit2_record='prior-review-whitespace-observations.json'))
delivery=load((OUT/'DELIVERY.json').read_bytes())
inventory=load((OUT/'function-inventory.json').read_bytes())
global_inventory=load((OUT/'global-segment-inventory.json').read_bytes())
assert [row['name'] for row in inventory if row['state']=='CHANGED']==['collect']
assert all(row['state']=='UNCHANGED' for row in global_inventory)
assert len((OUT/'copy-parent.py.txt').read_bytes())==delivery['source']['bytes']
assert sha((OUT/'copy-parent.py.txt').read_bytes())==delivery['source']['sha256']
for key in ['source_patch','schema_patch','combined_patch']:
    raw=(OUT/delivery[key]['path']).read_bytes()
    assert len(raw)==delivery[key]['bytes'] and sha(raw)==delivery[key]['sha256']
assert (OUT/'from-85c668f.patch').read_bytes()==(OUT/'from-85c668f-source.patch').read_bytes()+(OUT/'from-85c668f-schema.patch').read_bytes()
old=Path('docs/verification/m1-copy-parent-capsule-2026-10-25-t3-f65c3d2f')
assert (OUT/'prior-from-cleared-source-launcher.patch').read_bytes()==(old/'from-cleared-source-launcher.patch').read_bytes()
plan=load((OUT/'finite-plan.json').read_bytes())
assert plan['execution'] is False and all(v==0 for v in plan['actual_execution_counts'].values())
json_files=[]
for path in sorted(OUT.glob('*.json')):
    load(path.read_bytes())
    json_files.append(path.name)
put('final-static-checks.json',dict(status='IMPLEMENTED_UNREVIEWED',execution=False,
    full_source=delivery['source'],source_patch=delivery['source_patch'],combined_patch=delivery['combined_patch'],
    changed_functions=['collect'],unchanged_functions=47,unchanged_class=1,
    unchanged_non_declaration_segments=len(global_inventory),strict_JSON_files_read=json_files,
    exact_patch_reconstruction='patch-observations.json',schema_domain_and_closed_keyset_checks='schema-observations.json and signal-order-and-interface-observations.json',
    old_tracked_entries=4422,old_bytes_modes_blobs_index_preservation='preservation-closing.json',
    full_whitespace_exit=2,non_patch_whitespace_exit=0,new_repair_patches_whitespace_exit=0,
    prior_author_reader_errors=4,prior_reviewer_reader_errors=1,new_author_notes='author-reader-notes.json',
    actual_future_render=None,all_nine_prepared_slots='null',future_prepare_source_review=None,
    independent_fresh_R4_review=None,requires='Independent fresh R4 SOURCE/DATA review before use',
    supplied_source_checker_caller_helper_collector_execution=False,private_state_reads=False,
    network_remote_native_runtime_tests_mocks_build_signing_children_PR_push=False,
    runtime_acceptance=False,updater='CLOSED',Windows='SSH255/noRoute; UNKNOWN remote state',whole_flow_NI15='UNVERIFIED',
    note='Static comparisons and identities only. No behavior qualification or held-prepare acceptance. Actual commit/clean status reported outside packet.'))

files=[]
for path in sorted(OUT.iterdir()):
    assert path.is_file() and not path.is_symlink()
    assert stat.S_IMODE(path.stat().st_mode)==0o644
    if path.name in ('packet-manifest.json','SHA256SUMS'):
        continue
    raw=path.read_bytes()
    files.append(dict(path=path.name,bytes=len(raw),sha256=sha(raw),working_mode='0644',git_mode='100644',
        raw_git_blob_sha1=hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()))
put('packet-manifest.json',dict(schema='openergy.copy-parent-signal-schema-packet.v1',execution=False,
    files=files,excluded=['packet-manifest.json','SHA256SUMS'],
    exclusion_reason='No self-reference; SHA256SUMS covers manifest and all other payloads except itself'))
lines=[]
for path in sorted(OUT.iterdir()):
    if path.name!='SHA256SUMS':
        lines.append(sha(path.read_bytes())+'  '+path.name+'\n')
(OUT/'SHA256SUMS').write_text(''.join(lines))
(OUT/'SHA256SUMS').chmod(0o644)
print(json.dumps(dict(full_whitespace_exit=full['exit'],literal_context_rows=42,
    non_patch_exit=no_patches['exit'],new_repair_patch_exit=new_repair_patches['exit'],
    manifest_payloads=len(files),SHA256SUMS_entries=len(lines),execution=False)))
