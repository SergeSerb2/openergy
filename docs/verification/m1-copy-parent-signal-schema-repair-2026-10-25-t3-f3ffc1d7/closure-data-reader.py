"""Author-owned static closure reader; public data only, no target execution.

Run after construction and documentation. It reads supplied text/JSON/ASTs
as data only. All writes stay within OUT. Git subprocesses are public static
repository reads. This is not a behavior test or a receipt validator.
"""
import ast
import hashlib
import json
import os
from pathlib import Path
import stat
import subprocess

BASE='7e5de706e7457de5f4880cdef180b27536dc0d5e'
OUT=Path('docs/verification/m1-copy-parent-signal-schema-repair-2026-10-25-t3-f3ffc1d7')
OLD=Path('docs/verification/m1-copy-parent-capsule-2026-10-25-t3-f65c3d2f')


def git(*args):
    return subprocess.check_output(['git',*args],env=dict(os.environ,GIT_OPTIONAL_LOCKS='0'))


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def bind(raw):
    return dict(bytes=len(raw),sha256=sha(raw))


def canonical(value):
    return (json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=True,allow_nan=False)+'\n').encode()


def pairs(items):
    value={}
    for key,item in items:
        assert key not in value,key
        value[key]=item
    return value


def load(raw):
    def reject(value):
        raise ValueError(value)
    return json.loads(raw,object_pairs_hook=pairs,parse_constant=reject)


def put(name,value):
    (OUT/name).write_text(json.dumps(value,indent=2,ensure_ascii=True,allow_nan=False)+'\n')
    (OUT/name).chmod(0o644)


def parse(raw):
    return ast.parse(raw.decode(),feature_version=9)


def function(tree,name):
    return next(node for node in ast.walk(tree) if isinstance(node,ast.FunctionDef) and node.name==name)


def text(raw,node):
    return b''.join(raw.splitlines(True)[node.lineno-1:node.end_lineno])


def call_name(node):
    return ast.unparse(node.func) if isinstance(node,ast.Call) else None


raw=(OUT/'copy-parent.py.txt').read_bytes()
tree=parse(raw)
collect=function(tree,'collect')
calls=sorted([node for node in ast.walk(collect) if isinstance(node,ast.Call)],key=lambda node:(node.lineno,node.col_offset))
kills=[node for node in calls if call_name(node)=='os.kill']
assert len(kills)==1
kill=kills[0]
assert [ast.dump(node,include_attributes=False) for node in kill.args]==[
    "Attribute(value=Name(id='proc', ctx=Load()), attr='pid', ctx=Load())",
    "Attribute(value=Name(id='signal', ctx=Load()), attr='SIGKILL', ctx=Load())"]
assert not any(call_name(node) in ('proc.kill','proc.send_signal','os.killpg','os.waitpid','os.getpid') for node in calls)
polls=[node for node in calls if call_name(node)=='proc.poll' and node.lineno<kill.lineno]
last_poll=polls[-1]
observes=[node for node in calls if call_name(node)=='observe_exit' and last_poll.lineno<node.lineno<kill.lineno]
assert len(observes)==1
sample=observes[0]
assert [call_name(node) for node in calls if sample.lineno<node.lineno<=kill.lineno]==['TimeoutError','os.kill']
parents={child:parent for parent in ast.walk(tree) for child in ast.iter_child_nodes(parent)}
ancestors=[]
current=kill
while current in parents:
    current=parents[current]
    ancestors.append(current)
if_tests=[ast.unparse(node.test) for node in ancestors if isinstance(node,ast.If)]
assert "status['overflow'] or status['timed_out']" in if_tests
assert 'code is None' in if_tests
assert "proc is not None and (not status['reaped'])" in if_tests
gate=next(node for node in ast.walk(collect) if isinstance(node,ast.If) and
          ast.unparse(node.test)=='not (observed_ns < reap_end and observed_ns < COPY_END)')
assert gate.lineno<kill.lineno and any(isinstance(node,ast.Raise) for node in gate.body)
# The rejected-late TimeoutError call cannot lie on a signal path: its branch
# always raises, and os.kill is in the following sibling cap/deadline branch.
assert parents[gate] is parents[next(node for node in ancestors if isinstance(node,ast.If) and ast.unparse(node.test)=="status['overflow'] or status['timed_out']")]
wait=next(node for node in calls if call_name(node)=='proc.wait')
assert ast.unparse(wait.keywords[0].value)=='remaining(reap_end)'
observe=function(tree,'observe_exit')
observe_source=text(raw,observe).decode()
assert observe_source.index("status['actual_exit'] = code")<observe_source.index("status['reaped'] = True")<observe_source.index('clock_ns()')
assert "status['kill_syscall_returned'] = True" in text(raw,collect).decode()
assert "status['kill_delivered']" not in text(raw,collect).decode()
assert "kill_syscall_returned" not in text(raw,function(tree,'copy_record_complete')).decode()
assert "'kill_delivered'" in text(raw,function(tree,'copy_record_complete')).decode()

# The entire AST-data diff is one function. No imports/globals/entry drift.
before=(OLD/'copy-parent.py.txt').read_bytes()
old_tree=parse(before)
for name in ['observe_exit','collection_complete','inspect_new_prepared','inspect_copy_result','copy_record_complete',
             'startup','copy_parent_stdin','inspect_entry','bind_config','reserved_put','existing_ledger',
             'system_bootstrap','import_system_common','reject_entry_caches','main']:
    assert text(before,function(old_tree,name))==text(raw,function(tree,name))
module_old=[ast.dump(n,include_attributes=False) for n in old_tree.body if not isinstance(n,ast.FunctionDef) or n.name!='collect']
module_new=[ast.dump(n,include_attributes=False) for n in tree.body if not isinstance(n,ast.FunctionDef) or n.name!='collect']
assert module_old==module_new

schema=load((OUT/'prepared-copy-qualification.schema.json').read_bytes())
objects=[schema['properties'][name]['anyOf'][0] for name in ('helper_collection','caller_collection')]
objects += [schema['allOf'][0]['then']['properties'][name] for name in ('helper_collection','caller_collection')]
assert all(item==objects[0] for item in objects)
status=next(node for node in ast.walk(collect) if isinstance(node,ast.Assign) and any(isinstance(t,ast.Name) and t.id=='status' for t in node.targets))
assert isinstance(status.value,ast.Call) and call_name(status.value)=='dict'
source_keys={k.arg for k in status.value.keywords}|{'retained_bytes'}
assert all(source_keys==set(item['properties'])==set(item['required']) for item in objects)
success=schema['allOf'][0]['then']['allOf'][0]['properties']
assert all(success[name]['properties']=={'actual_exit':{'const':0},'kill_attempted':{'const':False},'kill_syscall_returned':{'const':False}} for name in ('helper_collection','caller_collection'))
assert schema['properties']['entry_authentication']['anyOf'][0]['properties']['actual_exit']=={'const':0}
assert schema['properties']['copy_gate_proof']['anyOf'][0]['properties']['records']['items']['properties']['actual_exit']=={'const':0}
external=schema['properties']['external_parent']['properties']
assert all(external[k]=={'type':'null'} for k in external if k!='own_receipt_final_barriers_rootclose_console_exit')
assert schema['properties']['controller_started']=={'const':False}
assert schema['properties']['runtime_acceptance']=={'const':False}

auth=function(tree,'inspect_entry')
auth_return=next(n for n in ast.walk(auth) if isinstance(n,ast.Return))
assert len(auth_return.value.keywords)==19
stdin=function(tree,'copy_parent_stdin')
stdin_dict=next(n for n in ast.walk(stdin) if isinstance(n,ast.Call) and call_name(n)=='dict')
assert len(stdin_dict.keywords)==7
startup=function(tree,'startup')
assert len(next(n for n in ast.walk(startup) if isinstance(n,ast.Return)).value.keywords)==10
writers=[n for n in ast.walk(tree) if isinstance(n,ast.Call) and call_name(n)=='common.write_bytes']
assert len(writers)==1 and any(k.arg=='critical' and isinstance(k.value,ast.Constant) and k.value.value is False for k in writers[0].keywords)
put('signal-order-and-interface-observations.json',dict(execution=False,
    os_kill_line=kill.lineno,last_poll_line=last_poll.lineno,immediate_observe_exit_line=sample.lineno,
    strict_unchanged_end_gate_line=gate.lineno,signal_ancestor_if_tests=if_tests,
    no_intervening_OS_query_or_poll=True,late_TimeoutError_branch_exits_before_signal=True,
    exact_retained_proc_pid_SIGKILL=True,signal_sites=1,wait_original_remaining=True,
    actual_exit_and_reap_stored_before_clock=True,no_signal_return_exit_inference=True,
    source_status_and_all_four_closed_schema_keysets_equal=True,parent_status_keys=sorted(source_keys),
    all_four_definitions_equal=True,success_actual_exit0_and_both_signal_flags_false=True,
    other_module_AST_data_unchanged=True,R9_legacy_keys_unchanged=True,
    startup_keys=10,stdin_keys=7,entry_authentication_keys=19,common_writer_sites=1,critical=False,
    unknown_ESRCH_error_path='Existing BaseException handler records kill_or_reap_error and first failure; no fake exit/reap; later writes blocked while unreaped',
    proof_limit='AST/text ordering only; no syscall, receipt, child, clock, EOF, or runtime qualification'))

patterns=[]
for path,expected,context_range in [
    ('docs/verification/m1-metadata-r9-repair-2026-10-25-t3-9b2c2354/metadata-caller.py.txt',
     '01623cd4759f96cd0066a9456a2680bff476bc37b5b1acb61ec40388582a5e9f',(470,493)),
    ('docs/verification/m1-source-stdin-payload-kill-gate-repair-2026-10-25-t3-da89534d/source-payload.py.txt',
     '8d7844a20ce347b7be52c07a516b8d9136bdb5c067e450bea24eb3453441afa6',(500,525)),
    ('docs/verification/m1-transport-phase-budgets-2026-10-25-t3-b31ab18a/transport-collector.py.txt',
     '695dc01461b31e04eb7b8c4a12848350f26a45731f05b0ddca971c05749e4df6',(403,446))]:
    data=Path(path).read_bytes()
    assert sha(data)==expected
    parsed=parse(data)
    for call in ast.walk(parsed):
        if isinstance(call,ast.Call) and call_name(call) in ('proc.kill','self.process.kill'):
            declaration=next(node for node in ast.walk(parsed) if isinstance(node,ast.FunctionDef) and node.lineno<=call.lineno<=node.end_lineno)
            start,end=context_range
            excerpt=b''.join(data.splitlines(True)[start-1:end])
            patterns.append(dict(path=path,source=bind(data),call_line=call.lineno,call=ast.unparse(call),
                declaration=dict(name=declaration.name,start=declaration.lineno,end=declaration.end_lineno,**bind(text(data,declaration))),
                context=dict(start=start,end=end,text=excerpt.decode(),**bind(excerpt)),
                changed=False,new_failure_path_clearance=False,
                limit='Same public Popen.kill wrapper can poll/skip; no dependency patch or full fresh dependency review; historical successful data unchanged'))
assert len(patterns)==3
put('frozen-signal-patterns.json',patterns)

# Read and bind public installed library text; never import it or qualify M1.
lib=load((OUT/'public-stdlib-source-evidence.json').read_bytes())
libraw=Path(lib['path']).read_bytes()
assert bind(libraw)=={k:lib[k] for k in ('bytes','sha256')}
for row in lib['ranges']:
    assert b''.join(libraw.splitlines(True)[row['start']-1:row['end']]).decode()==row['text']
ospath=Path(lib['path']).parent/'os.py'
osraw=ospath.read_bytes()
oslines=osraw.splitlines(True)
index=next(i for i,line in enumerate(oslines) if line.strip()==b'from posix import *')
put('direct-signal-library-audit.json',dict(subprocess_full_identity_rechecked=True,
    subprocess=bind(libraw),os_module=dict(path=str(ospath),**bind(osraw)),
    os_posix_binding_excerpt=dict(start=max(1,index-10),end=index+12,
        text=b''.join(oslines[max(0,index-11):index+12]).decode()),
    old_call_chain='Popen.kill -> send_signal -> poll -> _internal_poll -> waitpid(WNOHANG); can return without os.kill',
    new_selection='os.kill(proc.pid, signal.SIGKILL), the POSIX primitive exported by os; no Popen wrapper/new poll',
    target_interpreter_library_identity_verified=False,syscall_executed=False,native_probe=False,
    C_implementation_body_available_in_this_evidence=False,
    limits='Public Python-library and candidate AST source evidence only. Trusted target POSIX implementation, audit policy, process ownership/unreaped identity and absence of another reaper remain runtime assumptions. No hard scheduling or delivery guarantee.'))

full_revisions={name:git('rev-parse',rev).decode().strip() for name,rev in {
    'base':BASE,'candidate':'85c668f','candidate_import':'cd4fae4','full_review':'f33a1fd','full_review_import':'931d2c3',
    'prepare_R4':'c85725b','prepare_R4_import':'a302d4b'}.items()}
put('source-and-review-pins.json',dict(revisions=full_revisions,
    candidate_source=bind(before),candidate_full_report=dict(path='docs/verification/m1-copy-parent-full-review-2026-10-25-t3-9edf92e5/REVIEW.md',
        **bind(Path('docs/verification/m1-copy-parent-full-review-2026-10-25-t3-9edf92e5/REVIEW.md').read_bytes())),
    new_source=bind(raw),new_source_review=None,new_source_status='IMPLEMENTED_UNREVIEWED',required_review='independent fresh R4 before use',
    prepare_R4_report=dict(path='docs/verification/m1-prepare-parent-admission-cache-review-2026-10-25-t3-85610967/REVIEW.md',
        **bind(Path('docs/verification/m1-prepare-parent-admission-cache-review-2026-10-25-t3-85610967/REVIEW.md').read_bytes())),
    prepare_R4_verdict='HOLD_SOURCE PREPARE-P2-3',prepare_R5_binding=None,all_nine_prepared_bindings=None,execution=False))

# Public original evidence closure, including all212 SOURCE rows and vector recipe.
pin_data=load((OUT/'pin-bindings.json').read_bytes())
literal={}
for node in tree.body:
    if isinstance(node,ast.Assign):
        try:value=ast.literal_eval(node.value)
        except (ValueError,TypeError):continue
        for target in node.targets:
            if isinstance(target,ast.Name):literal[target.id]=value
# Complete raw source manifest is independently retained by the full reviewer.
review_literals=load(Path('docs/verification/m1-copy-parent-full-review-2026-10-25-t3-9edf92e5/literal-bindings.json').read_bytes())
assert all(canonical(literal[k])==canonical(v) for k,v in review_literals['constants'].items())
validation=load(Path('docs/verification/m1-source-metadata-observed-2026-10-25-t3-52422617/validation.json').read_bytes())
assert literal['SOURCE_RAWS']==validation['record_files']
assert len(literal['SOURCE_RAWS'])==212 and sum(x['bytes'] for x in literal['SOURCE_RAWS'])==213334
assert sha(canonical(literal['SOURCE_RAWS']))==validation['raw_records_manifest_sha256']==pin_data['source_raw_manifest']['canonical_sha256']
vectors=load(Path('docs/verification/m1-metadata-r9-repair-2026-10-25-t3-9b2c2354/command-vectors.json').read_bytes())['commands']
recipe=load(Path('docs/verification/m1-next-reviewed-ordinary-flow-2026-10-25-t3-a8a4ece5/metadata.recipe.json').read_bytes())
expanded=[]
for item in recipe['objects']:
    for verb in recipe['verb_order']:
        substitutions={'${BINARY}':'${BUNDLE}/'+item['binary'],
                       '${CODE_OBJECT}':'${BUNDLE}'+('/'+item['code_object'] if item['code_object'] else '')}
        expanded.append(dict(label='object-'+str(item['index'])+'-'+verb,
            argv=[substitutions.get(token,token) for token in recipe['argv_patterns'][verb]]))
expanded.append(dict(label='full-bundle-deep-verify',argv=recipe['deep_verify']))
assert vectors==review_literals['vectors']==expanded and len(expanded)==50
assert len(pin_data['frozen17'])==17 and len(pin_data['modules10'])==10
module_rows=[]
checks=load(Path('docs/verification/ordinary-combined-context-tuple-repair-2026-10-25/source-checks.json').read_bytes())['python_files']
for row in pin_data['modules10']:
    dest=row['destination']
    pin=pin_data['frozen17'][dest]
    data=Path(pin['repository_path']).read_bytes()
    assert sha(data)==row['sha256']==pin['sha256']==checks[dest]['sha256']
    assert len(data)==pin['bytes']==checks[dest]['bytes']
    module_rows.append(dict(destination=dest,repository_path=pin['repository_path'],**bind(data),unchanged=True))
put('frozen-input-observations.json',dict(all_full_review_literal_constants_equal=True,
    modules10=module_rows,frozen17_count=17,all50_vectors_equal_full_recipe_expansion=True,
    SOURCE212_equal_public_validation_record_files=True,SOURCE_raw_bytes=213334,
    SOURCE_manifest_canonical_sha256=sha(canonical(literal['SOURCE_RAWS'])),
    historical_SOURCE_all_zero=validation['all_actual_exits'],historical_SOURCE_rerun=False,
    future_prepared_tuple=None,execution=False))
put('integration-compatibility.json',dict(execution=False,
    inspect_new_prepared=dict(start=function(tree,'inspect_new_prepared').lineno,end=function(tree,'inspect_new_prepared').end_lineno,
        **bind(text(raw,function(tree,'inspect_new_prepared')))),
    collection_complete=dict(start=function(tree,'collection_complete').lineno,end=function(tree,'collection_complete').end_lineno,
        **bind(text(raw,function(tree,'collection_complete')))),
    closed_prepare_primary_keyset=False,reads_kill_delivered=False,renamed_prepare_parent_key_alone_requires_COPY_change=False,
    actual_gates_unchanged=['actual0/reap/EOF/no errors','receipt primary equals full actual status','full publication/raw bindings',
        'exact prepare source/review identity','original clock enclosure','canonical prepared tuple','external actual D4'],
    no_held_prepare_acceptance=True,prepare_R4='HOLD_SOURCE PREPARE-P2-3',future_R5_identity=None,
    future_prepare_review=None,actual_prepared_tuple=None,all_nine_prepared_slots='null',
    R9_record_validation_unchanged=True,raw52_208_full_comparison_unchanged=True))

# Exact old tracked/index/working preservation, including CRLF working bytes.
baseline=load((OUT/'baseline-tracked-inventory.json').read_bytes())
index={}
for line in git('ls-files','-s','-z').split(b'\0'):
    if line:
        head,path=line.split(b'\t',1)
        mode,blob,stage=head.decode().split()
        index[path.decode()]=dict(mode=mode,blob=blob,stage=stage)
head_tree={}
for line in git('ls-tree','-rz','HEAD').split(b'\0'):
    if line:
        head,path=line.split(b'\t',1)
        mode,kind,blob=head.decode().split()
        head_tree[path.decode()]=dict(git_mode=mode,git_type=kind,blob=blob)
for row in baseline:
    path=Path(row['path'])
    info=path.lstat()
    data=os.readlink(path).encode() if stat.S_ISLNK(info.st_mode) else path.read_bytes()
    assert bind(data)=={k:row[k] for k in ('bytes','sha256')},row['path']
    assert oct(stat.S_IMODE(info.st_mode))==row['working_mode']
    assert stat.S_IFMT(info.st_mode)==row['working_kind']
    assert index[row['path']]==row['index']
    assert head_tree[row['path']]=={k:row[k] for k in ('git_mode','git_type','blob')}
put('preservation-closing.json',dict(parent=BASE,tracked=len(baseline),all_old_tree_blobs_modes_equal=True,
    all_old_index_blobs_modes_equal=True,all_old_working_bytes_modes_equal=True,
    baseline_inventory_canonical_sha256=sha(canonical(baseline)),only_owned_folder_writes=True,
    pending_commit='folder only; hooks/signing disabled; actual final commit and clean status reported outside packet',execution=False))
print(json.dumps(dict(signal_line=kill.lineno,old_tracked_unchanged=len(baseline),parent_schema_definitions=4,
    frozen_dependency_signal_sites=len(patterns),status='STATIC_DATA_CLOSURE_COMPLETE; independent R4 pending',execution=False)))
