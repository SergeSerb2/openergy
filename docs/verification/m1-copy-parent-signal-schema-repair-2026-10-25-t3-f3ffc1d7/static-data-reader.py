"""Author-owned public Git/text/JSON/hash/AST-data construction reader.

This reader never imports, evaluates, executes, or bytecode-compiles supplied
source. AST parse results are data only. No target path from supplied data is
opened. Its only writes are under OUT. It is not a runtime checker.
"""
import ast
import copy
import difflib
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess

BASE = '7e5de706e7457de5f4880cdef180b27536dc0d5e'
CANDIDATE = '85c668f7e63c07e1f9d3cf9e76af05f21898f80d'
REVIEW = 'f33a1fd761770cb2a2961f9cf55b59dc4aeb40d5'
OLD = Path('docs/verification/m1-copy-parent-capsule-2026-10-25-t3-f65c3d2f')
FULL = Path('docs/verification/m1-copy-parent-full-review-2026-10-25-t3-9edf92e5')
OUT = Path('docs/verification/m1-copy-parent-signal-schema-repair-2026-10-25-t3-f3ffc1d7')
R4 = Path('docs/verification/m1-prepare-parent-admission-cache-review-2026-10-25-t3-85610967')


def git(*args):
    return subprocess.check_output(['git', *args], env=dict(os.environ, GIT_OPTIONAL_LOCKS='0'))


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def binding(raw):
    return dict(bytes=len(raw), sha256=digest(raw))


def canonical(value):
    return (json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=True, allow_nan=False) + '\n').encode('ascii')


def pairs(items):
    value = {}
    for key, item in items:
        assert key not in value, key
        value[key] = item
    return value


def load(raw):
    def nonfinite(value):
        raise ValueError(value)
    return json.loads(raw, object_pairs_hook=pairs, parse_constant=nonfinite)


def put(name, raw):
    destination = OUT / name
    destination.write_bytes(raw)
    destination.chmod(0o644)


def record(name, value):
    put(name, (json.dumps(value, indent=2, ensure_ascii=True, allow_nan=False) + '\n').encode('ascii'))


def tree(commit):
    rows = []
    for line in git('ls-tree', '-rz', commit).split(b'\0'):
        if not line:
            continue
        head, path = line.split(b'\t', 1)
        mode, kind, blob = head.decode().split()
        rows.append(dict(path=path.decode(), git_mode=mode, git_type=kind, blob=blob))
    return rows


def working(rows):
    index = {}
    for line in git('ls-files', '-s', '-z').split(b'\0'):
        if not line:
            continue
        head, path = line.split(b'\t', 1)
        mode, blob, stage = head.decode().split()
        index[path.decode()] = dict(mode=mode, blob=blob, stage=stage)
    result = []
    for row in rows:
        path = Path(row['path'])
        info = path.lstat()
        raw = os.readlink(path).encode() if stat.S_ISLNK(info.st_mode) else path.read_bytes()
        result.append(dict(row, index=index[row['path']], working_mode=oct(stat.S_IMODE(info.st_mode)),
                           working_kind=stat.S_IFMT(info.st_mode), **binding(raw)))
    return result


def parse(raw):
    return ast.parse(raw.decode('utf-8'), feature_version=9)


def declarations(raw):
    lines = raw.splitlines(keepends=True)
    result = {}
    def visit(node, parent=''):
        for item in ast.iter_child_nodes(node):
            if isinstance(item, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
                name = parent + item.name
                start = min([item.lineno] + [d.lineno for d in getattr(item, 'decorator_list', [])])
                data = b''.join(lines[start - 1:item.end_lineno])
                result[name] = dict(kind=type(item).__name__, start=start, end=item.end_lineno,
                                    raw=data, **binding(data))
                visit(item, name + '.')
            else:
                visit(item, parent)
    visit(parse(raw))
    return result


def modules(raw):
    lines = raw.splitlines(keepends=True)
    spans = []
    cursor = 1
    for node in parse(raw).body:
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            start = min([node.lineno] + [d.lineno for d in getattr(node, 'decorator_list', [])])
            if cursor < start:
                data = b''.join(lines[cursor - 1:start - 1])
                spans.append(dict(start=cursor, end=start - 1, raw=data, **binding(data)))
            cursor = node.end_lineno + 1
    if cursor <= len(lines):
        data = b''.join(lines[cursor - 1:])
        spans.append(dict(start=cursor, end=len(lines), raw=data, **binding(data)))
    return spans


def literals(raw):
    output = {}
    for node in parse(raw).body:
        if isinstance(node, ast.Assign):
            try:
                value = ast.literal_eval(node.value)
            except (ValueError, TypeError):
                continue
            names = [x.id for x in node.targets if isinstance(x, ast.Name)]
            for name in names:
                output[name] = value
    return output


def patch(before, after, name):
    return ''.join(difflib.unified_diff(before.decode().splitlines(True), after.decode().splitlines(True),
        fromfile='a/' + str(OLD / name), tofile='b/' + str(OUT / name), n=3)).encode()


def reconstruct(before, diff):
    """Read each unified row as data; do not call patch or alter a supplied file."""
    source, output, cursor = before.splitlines(True), [], 0
    rows = diff.splitlines(True)
    index = 2
    hunks = []
    while index < len(rows):
        match = re.fullmatch(rb'@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@\n', rows[index])
        assert match, rows[index]
        old_start, old_count, new_start, new_count = [int(x) if x is not None else 1 for x in match.groups()]
        start_index = old_start - 1 if old_count else old_start
        output.extend(source[cursor:start_index])
        cursor = start_index
        assert len(output) == (new_start - 1 if new_count else new_start)
        index += 1
        consumed = added = 0
        first = index
        while index < len(rows) and not rows[index].startswith(b'@@ '):
            row = rows[index]
            assert row[:1] in (b' ', b'+', b'-')
            if row[:1] in (b' ', b'-'):
                assert source[cursor] == row[1:]
                cursor += 1
                consumed += 1
            if row[:1] in (b' ', b'+'):
                output.append(row[1:])
                added += 1
            index += 1
        assert consumed == old_count and added == new_count
        hunks.append(dict(old_start=old_start, old_count=old_count, new_start=new_start,
                          new_count=new_count, rows=index-first, sha256=digest(b''.join(rows[first:index]))))
    output.extend(source[cursor:])
    return b''.join(output), hunks


def generate():
    assert git('rev-parse', 'HEAD').decode().strip() == BASE
    rows = tree(BASE)
    baseline = working(rows)
    record('baseline-tracked-inventory.json', baseline)
    record('preservation-opening.json', dict(parent=BASE, tracked=len(rows),
        inventory_sha256=digest(canonical(baseline)), status_before_owned_writes='clean (actual opening git status --short was empty)',
        index_matches_parent=all(r['index'] == dict(mode=r['git_mode'], blob=r['blob'], stage='0') for r in baseline)))

    # Authenticate every file of the exact candidate and review against this import.
    origin_records = []
    for revision, folder in [(CANDIDATE, OLD), (REVIEW, FULL), ('c85725b', R4)]:
        for row in tree(revision):
            if not row['path'].startswith(str(folder) + '/'):
                continue
            raw = git('show', revision + ':' + row['path'])
            assert Path(row['path']).read_bytes() == raw
            assert next(r for r in rows if r['path'] == row['path']) == row
            origin_records.append(dict(commit=revision, **row, **binding(raw), working_equal=True))
    record('candidate-review-bindings.json', origin_records)
    before = git('show', CANDIDATE + ':' + str(OLD / 'copy-parent.py.txt'))
    assert binding(before) == dict(bytes=120613, sha256='cfe3be7a67d22160b977a97941f7fa6ed44be322ab6822dd064c66bb93555b47')
    assert binding((FULL/'REVIEW.md').read_bytes()) == dict(bytes=21224, sha256='33550d9e4fca61a4425eadfe095b8a8234a825a1978b1a263852e3dac365d53f')
    old_call = b"                        proc.kill()  # Exact retained LIVE handle; no OS call since sample.\n                        status['kill_delivered'] = True\n"
    new_call = b"                        # Only this retained, still-unreaped child; no new poll/OS query.\n                        os.kill(proc.pid, signal.SIGKILL)\n                        status['kill_syscall_returned'] = True  # Return is not delivery or exit.\n"
    assert before.count(old_call) == 1
    after = before.replace(b'kill_attempted=False, kill_delivered=False, kill_or_reap_error=None,',
                           b'kill_attempted=False, kill_syscall_returned=False, kill_or_reap_error=None,', 1).replace(old_call, new_call, 1)
    put('copy-parent.py.txt', after)

    schema_before = (OLD/'prepared-copy-qualification.schema.json').read_bytes()
    schema = load(schema_before)
    changed_schema = []
    def repair(value, pointer=''):
        if isinstance(value, dict):
            props = value.get('properties', {})
            if props.get('schema') == {'const':'openergy.copy-parent-child.v1'}:
                assert value['additionalProperties'] is False
                props = {('kill_syscall_returned' if k == 'kill_delivered' else k): v for k,v in props.items()}
                props['kill_syscall_returned']['description'] = 'True only after os.kill on the retained unreaped direct child returns without exception; not proof of delivery, termination, or reap.'
                for field in ('actual_exit','kill_eligibility_exit'):
                    assert props[field] == {'anyOf':[{'type':'integer','minimum':0,'maximum':99999999999999999999},{'type':'null'}]}
                    props[field] = {'anyOf':[{'type':'integer'},{'type':'null'}]}
                value['properties'] = props
                value['required'] = ['kill_syscall_returned' if k == 'kill_delivered' else k for k in value['required']]
                changed_schema.append(pointer)
            for key,item in value.items():
                repair(item, pointer+'/'+key)
        elif isinstance(value,list):
            for index,item in enumerate(value):
                repair(item,pointer+'/'+str(index))
    repair(schema)
    assert len(changed_schema) == 4
    # Separate success-only constraints. All four underlying exit domains agree.
    success = schema['allOf'][0]['then']
    success['allOf'] = [{'properties': {name: {'properties': {'actual_exit': {'const':0},
        'kill_attempted': {'const':False}, 'kill_syscall_returned': {'const':False}}}
        for name in ('helper_collection','caller_collection')}}]
    schema_after = (json.dumps(schema, indent=2, ensure_ascii=True) + '\n').encode()
    put('prepared-copy-qualification.schema.json', schema_after)
    source_patch = patch(before, after, 'copy-parent.py.txt')
    schema_patch = patch(schema_before, schema_after, 'prepared-copy-qualification.schema.json')
    put('from-85c668f-source.patch',source_patch)
    put('from-85c668f-schema.patch',schema_patch)
    put('from-85c668f.patch', source_patch + schema_patch)
    patch_rows = {}
    for name, raw, delta, expected in [('source',before,source_patch,after),('schema',schema_before,schema_patch,schema_after)]:
        rebuilt, hunks = reconstruct(raw, delta)
        assert rebuilt == expected
        patch_rows[name] = dict(before=binding(raw),after=binding(expected),patch=binding(delta),hunks=hunks,exact_reconstruction=True)
    record('patch-observations.json',patch_rows)

    a,b = declarations(before),declarations(after)
    assert set(a) == set(b)
    names = [name for name in a if a[name]['raw'] != b[name]['raw']]
    assert names == ['collect']
    inventory = [dict(name=name,state=('CHANGED' if name in names else 'UNCHANGED'),
        before={k:v for k,v in a[name].items() if k!='raw'},
        after={k:v for k,v in b[name].items() if k!='raw'}) for name in a]
    assert len([x for x in inventory if x['after']['kind']=='FunctionDef']) == 48
    record('function-inventory.json',inventory)
    full = '# Full changed declaration before and after\n\nExact physical source slices, including final LF. No supplied source ran.\n'
    for name in names:
        for label, data in [('before',a[name]),('after',b[name])]:
            full += '\n## '+name+' '+label+'\n\nLines '+str(data['start'])+'..'+str(data['end'])+'; SHA256 '+data['sha256']+'.\n\n```python\n'+data['raw'].decode()+'```\n'
    put('FUNCTIONS.md',full.encode())
    ma,mb = modules(before),modules(after)
    assert len(ma)==len(mb) and all(x['raw']==y['raw'] for x,y in zip(ma,mb))
    record('global-segment-inventory.json',[dict(index=i,state='UNCHANGED',before={k:v for k,v in x.items() if k!='raw'},
        after={k:v for k,v in y.items() if k!='raw'}) for i,(x,y) in enumerate(zip(ma,mb))])
    put('GLOBAL-SEGMENTS.md',('# Complete non-declaration segments\n\nEvery before/after pair is byte identical; there are no changed globals.\n'+''.join(
        '\n## Segment '+str(i)+' '+label+'\n\n```python\n'+v['raw'].decode()+'```\n'
        for i,pair in enumerate(zip(ma,mb)) for label,v in zip(('before','after'),pair))).encode())
    la,lb = literals(before),literals(after)
    assert canonical(la)==canonical(lb)
    record('literal-global-bindings.json',[dict(name=k,**binding(canonical(v)),unchanged=True) for k,v in la.items()])

    # Full closed configurations and schemas; no populated future data.
    for name in ['config.template.json','config.schema.json','copy-parent-stdin.schema.json','compact-console.schema.json','pin-bindings.json']:
        put(name,(OLD/name).read_bytes())
    plan=load((OLD/'finite-plan.json').read_bytes())
    plan.update(parent=BASE,owned_folder=str(OUT),prepare_basis='R4 c85725b/import a302d4b HOLD_SOURCE PREPARE-P2-3; separate R5 unreviewed/pending; all future slots null')
    record('finite-plan.json',plan)
    proof_ast=parse(after)
    config_node=next(n.value for n in proof_ast.body if isinstance(n,ast.Assign) and any(isinstance(t,ast.Name) and t.id=='CONFIG_TEXT' for t in n.targets))
    chunks=after.splitlines(True)
    start=sum(map(len,chunks[:config_node.lineno-1]))+config_node.col_offset
    end=sum(map(len,chunks[:config_node.end_lineno-1]))+config_node.end_col_offset
    token=after[start:end]
    assert ast.literal_eval(token.decode())==config_node.value
    config=load(config_node.value)
    assert config==load((OUT/'config.template.json').read_bytes()) and config['execution'] is False
    assert all(v is None for v in config['prepared'].values()) and len(config['prepared'])==9
    assert all(v is None for v in config['parent'].values()) and len(config['parent'])==14
    assert all(v is None for row in config['reviews'].values() for v in row.values())
    config_node.value='<ONLY_CONFIG_TEXT_CONSTANT>'
    ast_data=ast.dump(proof_ast,annotate_fields=True,include_attributes=False).encode()
    record('template-proof.json',dict(source=binding(after), token_start=start,token_end=end,token=binding(token),
        prefix=binding(after[:start]),suffix=binding(after[end:]),config_canonical_without_LF=binding(canonical(config)[:-1]),
        normalized_AST_data_sha256=digest(ast_data),normalization="Replace only CONFIG_TEXT Constant.value by '<ONLY_CONFIG_TEXT_CONSTANT>'; ast.dump(annotate_fields=True, include_attributes=False), UTF8; AST data only",
        exact_original_token_restoration=after[:start]+token+after[end:]==after,
        actual_future_render=None,actual_future_review=None,all_nine_prepared_slots_null=True,all_review_slots_null=True,execution=False))
    bound=load((OLD/'byte-bound-proof.json').read_bytes())
    assert bound['full_render_bound']==len(before)-len(token)+bound['Python_string_token_bound']
    bound['full_render_bound']=len(after)-len(token)+bound['Python_string_token_bound']
    bound['basis']='Unchanged closed CONFIG schema width bound retained; only fixed source byte delta added; no populated render'
    record('byte-bound-proof.json',bound)
    for name,src in [('prior-author-reader-errors.json',OLD/'static-reader-errors.json'),
                     ('prior-from-cleared-source-launcher.patch',OLD/'from-cleared-source-launcher.patch'),
                     ('prior-review-reader-notes.json',FULL/'reader-notes.json'),
                     ('prior-author-static-checks.json',OLD/'static-checks.json'),
                     ('prior-author-first-complete-reader.json',OLD/'static-checks-first-complete-reader.json'),
                     ('prior-author-final-static-checks.json',OLD/'final-static-checks.json'),
                     ('prior-author-whitespace-check.json',OLD/'whitespace-check.json'),
                     ('prior-review-whitespace-observations.json',FULL/'whitespace-observations.json'),
                     ('public-stdlib-source-evidence.json',FULL/'public-stdlib-source-evidence.json'),
                     ('public-bootstrap-source-evidence.json',FULL/'public-bootstrap-source-evidence.json')]:
        put(name,src.read_bytes())
    record('author-reader-notes.json',dict(errors=[dict(stage='Read current prepare R4 by public Git',
        error="fatal: path 'docs/verification/m1-prepare-admission-cache-review-2026-10-25-t3-7e619cc4/REVIEW.md' does not exist in 'c85725b'",
        cause='Author guessed a folder spelling before listing exact commit paths.',
        correction='Used git diff-tree name inventory and read the actual m1-prepare-parent-admission-cache-review folder.',
        source_changed=False,target_execution=False)],prior_records_copied_byte_exact=True))

    # Public dependency identities and original 17 pins, never target private paths.
    dependencies=load((FULL/'dependency-bindings.json').read_bytes())
    for row in dependencies:
        raw=git('show',row['commit']+':'+row['path'])
        assert binding(raw)=={k:row[k] for k in ('bytes','sha256')}
        assert Path(row['path']).read_bytes()==raw
        exact=next(r for r in tree(row['commit']) if r['path']==row['path'])
        assert exact['blob']==row['blob'] and exact['git_mode']==row['mode']
    record('dependency-bindings.json',dependencies)
    pins=load((OLD/'pin-bindings.json').read_bytes())['frozen17']
    for name,row in pins.items():
        raw=Path(row['repository_path']).read_bytes()
        assert binding(raw)=={k:row[k] for k in ('bytes','sha256')}
        assert {k:row[k] for k in ('bytes','sha256')}==lb['PINS'][name]
    vectors=load(Path('docs/verification/m1-metadata-r9-repair-2026-10-25-t3-9b2c2354/command-vectors.json').read_bytes())
    assert len(vectors['commands'])==50 and len(lb['SOURCE_RAWS'])==212
    assert sum(x['bytes'] for x in lb['SOURCE_RAWS'])==213334
    schema_rows=[]
    def inspect(v,p=''):
        if isinstance(v,dict):
            if v.get('properties',{}).get('schema')=={'const':'openergy.copy-parent-child.v1'}:
                props=v['properties']
                assert 'kill_delivered' not in props and 'kill_syscall_returned' in props
                assert set(props)==set(v['required']) and v['additionalProperties'] is False
                assert props['actual_exit']==props['kill_eligibility_exit']=={'anyOf':[{'type':'integer'},{'type':'null'}]}
                schema_rows.append(dict(pointer=p,signed_nullable_fields=['actual_exit','kill_eligibility_exit'],closed=True,
                    key_count=len(props),property_keys=list(props),required=list(v['required'])))
            for k,w in v.items():inspect(w,p+'/'+k)
        elif isinstance(v,list):
            for i,w in enumerate(v):inspect(w,p+'/'+str(i))
    inspect(schema)
    assert len(schema_rows)==4
    # Reverse only the declared repair fields and separate success constraint;
    # every remaining full schema byte-semantic value must equal the candidate.
    inverse=copy.deepcopy(schema)
    del inverse['allOf'][0]['then']['allOf']
    def undo(v):
        if isinstance(v,dict):
            if v.get('properties',{}).get('schema')=={'const':'openergy.copy-parent-child.v1'}:
                props={('kill_delivered' if k=='kill_syscall_returned' else k):w for k,w in v['properties'].items()}
                del props['kill_delivered']['description']
                for field in ('actual_exit','kill_eligibility_exit'):
                    props[field]={'anyOf':[{'type':'integer','minimum':0,'maximum':99999999999999999999},{'type':'null'}]}
                v['properties']=props
                v['required']=['kill_delivered' if k=='kill_syscall_returned' else k for k in v['required']]
            for w in v.values():undo(w)
        elif isinstance(v,list):
            for w in v:undo(w)
    undo(inverse)
    assert inverse==load(schema_before)
    record('schema-observations.json',dict(parent_records=schema_rows,all_other_full_schema_semantics_unchanged=True,
        non_exit_integer_minima_unchanged=True,success_actual_exit=0,success_kill_attempted=False,success_kill_syscall_returned=False,
        method='Whole schema traversal and declared-delta reversal; no receipt examples, mocks, signal, runtime validator, or supplied code execution'))
    record('static-observations.json',dict(execution=False,source_functions=48,class_declarations=1,
        changed_functions=names,unchanged_functions=47,unchanged_class=True,unchanged_non_declaration_segments=len(ma),
        all_literal_globals_unchanged=True,all17_pins_unchanged=True,modules=10,vectors=50,SOURCE_raw_rows=212,
        SOURCE_raw_bytes=213334,SOURCE_manifest=binding(canonical(lb['SOURCE_RAWS'])),
        full_candidate_files_bound=sum(r['commit']==CANDIDATE for r in origin_records),
        full_review_files_bound=sum(r['commit']==REVIEW for r in origin_records),dependencies=len(dependencies),
        receipt_parent_definitions=4,signed_nullable_exit_domains=8,
        config_schema_template_stdin_console_byte_exact=True,full52_208_gate_unchanged=True,
        generic_collection_complete_unchanged=True,inspect_new_prepared_unchanged=True,
        no_prepare_primary_closed_keyset_added=True,no_held_prepare_clearance=True,
        signal_authority='one os.kill(proc.pid, signal.SIGKILL) inside unchanged None/strict-end/cap-or-deadline gates; no signal-return exit inference',
        actual_future_render=None,actual_prepared_tuple=None,independent_fresh_R4_review=None,runtime_acceptance=False))
    record('DELIVERY.json',dict(status='IMPLEMENTED_UNREVIEWED',execution=False,parent=BASE,
        source=dict(path='copy-parent.py.txt',**binding(after)),source_patch=dict(path='from-85c668f-source.patch',**binding(source_patch)),
        combined_patch=dict(path='from-85c668f.patch',**binding(source_patch+schema_patch)),schema_patch=dict(path='from-85c668f-schema.patch',**binding(schema_patch)),
        source_changed_functions=['collect'],unchanged_functions=47,unchanged_class_declarations=1,
        future_actual_render=None,future_prepared_tuple=None,independent_fresh_R4_review=None,
        requires='Independent fresh R4 SOURCE/DATA review of this exact repair before use; no execution authority',
        runtime_acceptance=False,private_COPY_startup='UNMEASURED'))
    print(json.dumps(dict(source=binding(after),changed_functions=names,old_tracked=len(rows),dependency_count=len(dependencies),new_folder=str(OUT))))


if __name__=='__main__':
    generate()
