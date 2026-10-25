"""Reviewer-owned public Git/bytes/JSON/AST DATA reader. Never loads target code.

Only this new review folder is writable. No target/private paths are opened.
Run with opening, audit, or closing. No supplied checker is called.
"""
import ast
import difflib
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import sys

OUT = Path(__file__).parent
BASE = '64bcde791ea95472b796fab623519a124ee553d0'
CAND = '23cb52f54cf9ef288500d4b09606d42303ff414f'
IMPORTED = 'b6d19d36f3672cce7e9735b371e03ae82e485426'
PACKET = Path('docs/verification/m1-copy-only-caller-direct-signal-2026-10-25-t3-56c87ccf')
R9 = Path('docs/verification/m1-metadata-r9-repair-2026-10-25-t3-9b2c2354')
PARENT = Path('docs/verification/m1-copy-parent-signal-schema-repair-2026-10-25-t3-f3ffc1d7')


def git(*args, data=None):
    return subprocess.check_output(['git', '--no-optional-locks', *args], input=data)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def canonical(value):
    return (json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=True) + '\n').encode()


def strict(data):
    def pairs(rows):
        result = {}
        for k, v in rows:
            assert k not in result, k
            result[k] = v
        return result
    def bad(value):
        raise ValueError(value)
    return json.loads(data, object_pairs_hook=pairs, parse_constant=bad)


def save(name, value):
    assert Path(name).name == name
    (OUT / name).write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')


def entries(commit):
    raw = git('ls-tree', '-rz', '--full-tree', commit)
    result = {}
    for row in raw.split(b'\0'):
        if row:
            meta, path = row.split(b'\t', 1)
            mode, kind, blob = meta.decode().split()
            assert kind == 'blob'
            result[os.fsdecode(path)] = (mode, blob)
    return raw, result


def preservation():
    raw_tree, tree = entries(BASE)
    _, current = entries('HEAD')
    raw_index = git('ls-files', '--stage', '-z')
    index = {}
    index_old = []
    for row in raw_index.split(b'\0'):
        if row:
            meta, path = row.split(b'\t', 1)
            mode, blob, stage = meta.decode().split()
            assert stage == '0'
            path = os.fsdecode(path)
            index[path] = (mode, blob)
            if path in tree:
                index_old.append(row + b'\0')
    rows, mismatches, conversions = [], [], []
    for path, (mode, blob) in tree.items():
        p = Path(path)
        info = p.lstat()
        raw = os.fsencode(os.readlink(p)) if stat.S_ISLNK(info.st_mode) else p.read_bytes()
        expected_kind = 'link' if mode == '120000' else 'file'
        kind = 'link' if stat.S_ISLNK(info.st_mode) else 'file' if stat.S_ISREG(info.st_mode) else 'other'
        work_mode = stat.S_IMODE(info.st_mode)
        expected_mode = {'100644': 0o644, '100755': 0o755, '120000': 0o777}[mode]
        blob_input = raw
        if path == 'tools/windows/openergy-run.cmd':
            assert b'\r\n' in raw and b'\n' not in raw.replace(b'\r\n', b'')
            blob_input = raw.replace(b'\r\n', b'\n')
            conversions.append(dict(path=path, actual_bytes=len(raw), actual_sha256=sha(raw), working_mode=oct(work_mode)))
        actual_blob = hashlib.sha1(b'blob ' + str(len(blob_input)).encode() + b'\0' + blob_input).hexdigest()
        if (current.get(path) != (mode, blob) or index.get(path) != (mode, blob)
                or actual_blob != blob or kind != expected_kind or work_mode != expected_mode):
            mismatches.append(path)
        rows.append([path, mode, blob, index.get(path), kind, oct(work_mode), len(raw), sha(raw)])
    assert not mismatches, mismatches
    result = dict(baseline_commit=BASE, baseline_tree=git('rev-parse', BASE + '^{tree}').decode().strip(),
                  old_entries=len(rows), raw_tree_sha256=sha(raw_tree),
                  old_index_sha256=sha(b''.join(index_old)), canonical_rows_sha256=sha(canonical(rows)),
                  canonical_rows_format='Git ls-tree -rz order; [path, tree_mode, blob, [index_mode, index_blob], working_kind, octal_working_mode, raw_working_bytes, raw_working_sha256]; compact sorted-key ASCII JSON plus LF',
                  per_entry_checks=['HEAD mode/blob', 'index mode/blob/stage0', 'actual raw bytes to Git blob (one declared CRLF conversion)', 'actual file kind and full permission mode'],
                  entries_checked_per_dimension=len(rows), conversions=conversions, mismatches=mismatches)
    return result


def parse(raw):
    return ast.parse(raw, feature_version=(3, 9))


def declarations(raw):
    lines = raw.splitlines(True)
    result = {}
    def walk(nodes, prefix=''):
        for node in nodes:
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
                first = min([node.lineno] + [x.lineno for x in node.decorator_list])
                name = prefix + node.name
                result[name] = (node, b''.join(lines[first - 1:node.end_lineno]), first, node.end_lineno)
                walk(node.body, name + '.')
    walk(parse(raw).body)
    return result


def literals(raw):
    result = {}
    for node in parse(raw).body:
        if isinstance(node, ast.Assign) and len(node.targets) == 1 and isinstance(node.targets[0], ast.Name):
            try:
                result[node.targets[0].id] = ast.literal_eval(node.value)
            except (ValueError, TypeError):
                pass
    return result


def globals_only(raw):
    lines = raw.splitlines(True)
    for node, _, first, last in declarations(raw).values():
        if node.col_offset == 0:
            lines[first - 1:last] = [b''] * (last - first + 1)
    return b''.join(lines)


def apply_hunks(source, patch, reverse=False):
    rows = patch.splitlines(True)
    assert rows[0].startswith(b'--- ') and rows[1].startswith(b'+++ ')
    old = source.splitlines(True)
    output, cursor, i, hunks = [], 0, 2, 0
    while i < len(rows):
        match = re.fullmatch(rb'@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@\n', rows[i])
        assert match
        a, ac, b, bc = [int(v) if v is not None else 1 for v in match.groups()]
        removed, added = [], []
        i += 1
        while i < len(rows) and not rows[i].startswith(b'@@ '):
            assert rows[i][:1] in (b'-', b'+')
            (removed if rows[i][:1] == b'-' else added).append(rows[i][1:])
            i += 1
        assert len(removed) == ac and len(added) == bc
        if reverse:
            a, ac, b, bc, removed, added = b, bc, a, ac, added, removed
        at = a if ac == 0 else a - 1
        assert cursor <= at and old[at:at + ac] == removed
        output.extend(old[cursor:at])
        output.extend(added)
        cursor = at + ac
        hunks += 1
    output.extend(old[cursor:])
    return b''.join(output), hunks


def audit():
    _, ct = entries(CAND)
    _, it = entries(IMPORTED)
    packet_rows = []
    for p in sorted(PACKET.iterdir()):
        path = p.as_posix()
        raw = p.read_bytes()
        assert ct[path] == it[path]
        assert git('show', CAND + ':' + path) == raw
        assert git('show', IMPORTED + ':' + path) == raw
        assert ct[path][0] == '100644' and stat.S_IMODE(p.lstat().st_mode) == 0o644
        if p.suffix == '.json':
            strict(raw)
        packet_rows.append(dict(path=path, mode=ct[path][0], blob=ct[path][1], bytes=len(raw), sha256=sha(raw)))
    assert len(packet_rows) == 20
    delta = git('diff-tree', '--no-commit-id', '--name-status', '-r', CAND).decode().splitlines()
    assert len(delta) == 20 and all(row.startswith('A\t' + PACKET.as_posix() + '/') for row in delta)
    checksums = (PACKET / 'SHA256SUMS').read_text().splitlines()
    assert len(checksums) == 19
    for line in checksums:
        digest, name = line.split('  ')
        assert sha((PACKET / name).read_bytes()) == digest
    old = (R9 / 'metadata-caller.py.txt').read_bytes()
    new = (PACKET / 'metadata-copy-caller-direct-signal.py.txt').read_bytes()
    assert len(old) == 46908 and sha(old) == '01623cd4759f96cd0066a9456a2680bff476bc37b5b1acb61ec40388582a5e9f'
    assert len(new) == 47491 and sha(new) == '43415cd864f2e163eb053ec9a6999ce849a477cb4bc980a00591ec3ddb9ba0a1'
    patch = (PACKET / 'from-frozen-r9.patch').read_bytes()
    forward, count = apply_hunks(old, patch)
    reverse, reverse_count = apply_hunks(new, patch, True)
    assert forward == new and reverse == old and count == reverse_count == 7
    recomputed = ''.join(difflib.unified_diff(old.decode().splitlines(True), new.decode().splitlines(True),
                       fromfile='a/' + (R9 / 'metadata-caller.py.txt').as_posix(),
                       tofile='b/' + PACKET.name + '/metadata-copy-caller-direct-signal.py.txt', n=0)).encode()
    assert recomputed == patch
    before, after = declarations(old), declarations(new)
    assert before.keys() == after.keys()
    changes = [name for name in before if before[name][1] != after[name][1]]
    assert changes == ['collect', 'main']
    function_rows = []
    inventory = strict((PACKET / 'function-inventory.json').read_bytes())
    assert len(inventory) == len(before) == 36
    for row in inventory:
        name = row['name']
        for side, table in [('before', before), ('after', after)]:
            _, data, first, last = table[name]
            assert row[side] == dict(bytes=len(data), lines=[first, last], sha256=sha(data))
        assert row['state'] == ('CHANGED' if name in changes else 'UNCHANGED')
        function_rows.append(dict(name=name, lines=[after[name][2], after[name][3]], sha256=sha(after[name][1]), changed=name in changes))
    for side, table in [('before', before), ('after', after)]:
        assert (PACKET / ('changed-functions.' + side + '.txt')).read_bytes() == b'\n\n'.join(table[n][1] for n in changes)
    og, ng = globals_only(old), globals_only(new)
    gp = (PACKET / 'global-diff.patch').read_bytes()
    assert apply_hunks(og, gp)[0] == ng and apply_hunks(ng, gp, True)[0] == og
    def unaffected(raw):
        lines = raw.splitlines(True)
        return [b''.join(lines[n.lineno - 1:n.end_lineno]) for n in parse(raw).body
                if not (isinstance(n, ast.Expr) and isinstance(n.value, ast.Constant) and isinstance(n.value.value, str))
                and not (isinstance(n, ast.Assign) and any(isinstance(t, ast.Name) and t.id == 'COPY_CALLER_LEAF' for t in n.targets))
                and not (isinstance(n, ast.FunctionDef) and n.name in changes)]
    assert unaffected(old) == unaffected(new)
    bound_rows = []
    for row in strict((PACKET / 'input-bindings.json').read_bytes()):
        raw = git('show', row['commit'] + ':' + row['path'])
        entry = git('ls-tree', row['commit'], '--', row['path']).split(b'\t')[0].decode().split()
        assert entry == [row['mode'], 'blob', row['blob']]
        assert len(raw) == row['bytes'] and sha(raw) == row['sha256'] and Path(row['path']).read_bytes() == raw
        bound_rows.append(row)
    extra_specs = [
        ('b56dcdc450d2152c06bff384f2760af2a82ceff0', 'docs/verification/m1-copy-parent-signal-schema-review-2026-10-25-t3-03e26ccf/REVIEW.md'),
        ('2b5fc1f505b1f0426b8f1e8b00537a0b6e805fc7', 'docs/verification/ordinary-combined-context-tuple-repair-2026-10-25/materialization.json'),
    ]
    for commit, path in extra_specs:
        raw = git('show', commit + ':' + path)
        mode, kind, blob = git('ls-tree', commit, '--', path).split(b'\t')[0].decode().split()
        assert kind == 'blob' and Path(path).read_bytes() == raw
        bound_rows.append(dict(commit=commit, path=path, mode=mode, blob=blob, bytes=len(raw), sha256=sha(raw), working_bytes_equal=True))
    pins = strict((R9 / 'input-pins.json').read_bytes())['files']
    for row in pins.values():
        path = Path(os.path.normpath(row['repository_path']))
        raw = path.read_bytes()
        assert git('show', BASE + ':' + path.as_posix()) == raw
        assert len(raw) == row['bytes'] and sha(raw) == row['sha256']
    assert len(pins) == 17
    modules = strict(Path('docs/verification/ordinary-combined-context-tuple-repair-2026-10-25/source-checks.json').read_bytes())['python_files']
    assert len(modules) == 10
    for name, row in modules.items():
        assert row['bytes'] == pins[name]['bytes'] and row['sha256'] == pins[name]['sha256']
    materialization_path = Path(extra_specs[1][1])
    materialization = strict(materialization_path.read_bytes())
    assert len(materialization['modules']) == 10
    for row in materialization['modules']:
        name = row['destination']
        path = Path(os.path.normpath(materialization_path.parent / row['source']))
        raw = git('show', extra_specs[1][0] + ':' + path.as_posix())
        assert raw == path.read_bytes() and len(raw) == pins[name]['bytes']
        assert sha(raw) == row['sha256'] == pins[name]['sha256']
    vectors = strict((R9 / 'command-vectors.json').read_bytes())['commands']
    recipe = strict(Path('docs/verification/m1-next-reviewed-ordinary-flow-2026-10-25-t3-a8a4ece5/metadata.recipe.json').read_bytes())
    expanded = []
    for obj in recipe['objects']:
        for verb in recipe['verb_order']:
            replacements = {'${BINARY}': '${BUNDLE}/' + obj['binary'],
                            '${CODE_OBJECT}': '${BUNDLE}' + ('/' + obj['code_object'] if obj['code_object'] else '')}
            expanded.append(dict(label='object-' + str(obj['index']) + '-' + verb,
                                 argv=[replacements.get(t, t) for t in recipe['argv_patterns'][verb]]))
    expanded.append(dict(label='full-bundle-deep-verify', argv=recipe['deep_verify']))
    assert vectors == expanded
    parent = (PARENT / 'copy-parent.py.txt').read_bytes()
    pl = literals(parent)
    validation_path = Path('docs/verification/m1-source-metadata-observed-2026-10-25-t3-52422617/validation.json')
    validation = strict(validation_path.read_bytes())
    assert len(vectors) == len({v['label'] for v in vectors}) == 50
    assert pl['SOURCE_RAWS'] == validation['record_files'] and len(pl['SOURCE_RAWS']) == 212
    assert pl['SOURCE_RESULT'] == dict(bytes=327813, sha256='7482a637159e0389c5cd4c505778919bb49354437bc9feb1cf008d9db395712d')
    assert len(validation['records']) == 53 and all(type(r['actual_exit']) is int and r['actual_exit'] == 0 for r in validation['records'])
    assert validation['no_timeout_kill_overflow_drop_or_publication_error'] is True
    schema = strict((PACKET / 'child-status-v3.schema.json').read_bytes())
    for name in ['collector_status', 'published_status', 'retained_status']:
        definition = schema['$defs'][name]
        assert definition['additionalProperties'] is False
        for key in ['actual_exit', 'kill_eligibility_exit']:
            assert definition['properties'][key] == dict(type=['integer', 'null'])
        assert 'kill_delivered' not in definition['properties']
    core = next(n.value for n in ast.walk(after['collect'][0]) if isinstance(n, ast.Assign)
                and any(isinstance(t, ast.Name) and t.id == 'record' for t in n.targets))
    keys = {k.arg for k in core.keywords}
    assert keys == set(schema['$defs']['collector_status']['required'])
    assert keys | {'stdout', 'stderr'} == set(schema['$defs']['published_status']['required'])
    assert keys == set(schema['$defs']['retained_status']['required'])
    assert schema['$defs']['direct_child']['properties']['actual_exit'] == dict(type=['integer', 'null'])
    for name in ['collector_status', 'published_status', 'retained_status']:
        props = schema['$defs'][name]['properties']
        for key in ['pid', 'started_ns', 'finished_ns', 'command_deadline_ns', 'kill_eligibility_observed_ns', 'observed_stream_bytes', 'dropped_observed_bytes']:
            assert props[key]['minimum'] >= 0
    success = schema['$defs']['success_published_status']['allOf'][1]['properties']
    assert success['actual_exit'] == {'const': 0} and success['kill_syscall_returned'] == {'const': False}
    cli = strict((PACKET / 'closed-cli.json').read_bytes())
    assert cli['observations'] == ['copy-files-before'] + [v['label'] for v in vectors] + ['copy-files-after']
    plan = strict((PACKET / 'finite-plan.json').read_bytes())
    assert plan['execution'] is False and all(v is None for v in plan['future'].values())
    parent_schema = strict((R9 / 'parent-boundary.schema.json').read_bytes())
    assert len(parent_schema['required']) == 7
    assert len(parent_schema['oneOf'][1]['properties']['entry_authentication']['required']) == 19
    startup_call = after['startup'][0].body[0].value
    assert isinstance(startup_call, ast.Call) and len(startup_call.keywords) == 10
    collect_calls = []
    for node in ast.walk(after['collect'][0]):
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute) and node.func.attr in ('poll', 'wait', 'kill', 'killpg', 'waitpid'):
            collect_calls.append(dict(line=node.lineno, call=ast.unparse(node)))
    collect_calls.sort(key=lambda r: r['line'])
    assert [r['call'] for r in collect_calls] == ['proc.poll()', 'proc.wait(timeout=remaining(work_end))',
        'proc.poll()', 'os.kill(proc.pid, signal.SIGKILL)', 'proc.wait(timeout=remaining(deadline))']
    save('observations.json', dict(candidate_bytes=len(new), candidate_sha256=sha(new), candidate_lines=len(new.splitlines()),
         packet_files=20, packet_checksum_entries=19, original_import_worktree_equal=True,
         patch_hunks=count, forward_every_byte_equal=True, reverse_every_byte_equal=True,
         declarations=function_rows, changed_declarations=changes, unchanged_declarations=34,
         all_other_module_nodes_and_entry_byte_exact=True, globals_forward_reverse_equal=True,
         full_changed_function_excerpts_equal=True, pins=17, modules=10, vectors=50,
         all50_vectors_equal_independent_recipe_expansion=True, materialization_all10_equal=True,
         original_candidate_delta='20 additions in the candidate folder only',
         copy_stdin_keys=7, entry_authentication_keys=19, copy_startup_keys=10,
         collect_process_call_sites=collect_calls,
         source_public_records=53, source_public_raw_refs=212,
         source_raw_manifest_sha256=sha(canonical(pl['SOURCE_RAWS'])),
         source_boundary_bindings_sha256=sha(canonical(pl['SOURCE_BOUNDARY_BINDINGS'])),
         source_result_binding=pl['SOURCE_RESULT'], schema_actual_keys_equal=True,
         supplied_readers_executed=False, supplied_source_executed_imported_evaluated_compiled=False,
         exact_parent_composition='BLOCKED', private_raw_reads=0))
    save('inputs.json', dict(candidate=CAND, imported=IMPORTED, packet=packet_rows, frozen_inputs=bound_rows))


mode = sys.argv[1]
assert mode in ('opening', 'audit', 'closing')
if mode == 'audit':
    audit()
else:
    value = preservation()
    if mode == 'opening':
        assert git('rev-parse', 'HEAD').decode().strip() == BASE
        save('baseline.json', value)
    else:
        assert value == strict((OUT / 'baseline.json').read_bytes())
        save('preservation.json', dict(**value, stage='closing before commit; post-commit comparison reported externally'))
    print(json.dumps(dict(stage=mode, old_entries=value['old_entries'], mismatches=value['mismatches'], digest=value['canonical_rows_sha256'])))
