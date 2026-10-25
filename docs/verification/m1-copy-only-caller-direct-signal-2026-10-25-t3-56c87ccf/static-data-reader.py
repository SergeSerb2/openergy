"""Author-owned Git/text/JSON/hash/AST-DATA reader. No supplied code is run.

Writes only this packet. Reads only public tracked files and public Git objects.
Modes: construct (deterministic source/contract data), verify (static data only).
No private path literal in supplied source is opened. No target imports occur.
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

ROOT = Path.cwd()
OUT = Path(__file__).resolve().parent
BASE = '4ea32a7538aa1b996e60fe7dd185497cbba75356'
R9 = 'docs/verification/m1-metadata-r9-repair-2026-10-25-t3-9b2c2354/'
PARENT = 'docs/verification/m1-copy-parent-signal-schema-repair-2026-10-25-t3-f3ffc1d7/'
LEAF = 'metadata-copy-caller-direct-signal.py.txt'


def git(*args, input=None):
    return subprocess.check_output(['git', '--no-optional-locks', *args], input=input)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def canonical(value):
    return (json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=True) + '\n').encode()


def strict(data):
    def unique(pairs):
        value = {}
        for key, item in pairs:
            assert key not in value, key
            value[key] = item
        return value
    def nonfinite(value):
        raise ValueError(value)
    return json.loads(data, object_pairs_hook=unique, parse_constant=nonfinite)


def put(name, data):
    assert Path(name).name == name
    path = OUT / name
    assert path.parent == OUT and not path.is_symlink()
    path.write_bytes(data if isinstance(data, bytes) else data.encode())


def save(name, value):
    put(name, json.dumps(value, indent=2, sort_keys=True, ensure_ascii=True) + '\n')


def parse(data):
    return ast.parse(data, feature_version=(3, 9))


def functions(data):
    lines = data.splitlines(keepends=True)
    result = {}
    def visit(nodes, prefix=''):
        for node in nodes:
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
                name = prefix + node.name
                first = min([node.lineno] + [n.lineno for n in node.decorator_list])
                raw = b''.join(lines[first - 1:node.end_lineno])
                result[name] = (node, raw, first, node.end_lineno)
                visit(node.body, name + '.')
    visit(parse(data).body)
    return result


def globals_only(data):
    lines = data.splitlines(keepends=True)
    for node, _, first, end in functions(data).values():
        if isinstance(node, (ast.FunctionDef, ast.ClassDef)) and node.col_offset == 0:
            for i in range(first - 1, end):
                lines[i] = b''
    return b''.join(lines)


def assignments(data):
    result = {}
    for node in parse(data).body:
        if isinstance(node, ast.Assign) and len(node.targets) == 1 and isinstance(node.targets[0], ast.Name):
            try:
                result[node.targets[0].id] = ast.literal_eval(node.value)
            except (ValueError, TypeError):
                pass
    return result


def candidate(old):
    edits = [
        (b'CLI: metadata-caller.py.txt source|copy EVIDENCE START_CLOCK_MONOTONIC_NS PARENT_RECORD_SHA256\nSource parent record: EVIDENCE/metadata-source-parent.json. Copy: bounded stdin.\nSee README.md for the parent boundary and outer collection contract.',
         b'CLI: metadata-copy-caller-direct-signal.py.txt copy EVIDENCE START_CLOCK_MONOTONIC_NS PARENT_RECORD_SHA256\nCOPY ONLY. Bounded stdin; saved SOURCE v2 stays in the SAME existing ROOT.\nPublish this new alias only AFTER prepare through existing SYSTEM/common R9\nregular reservations; pin its exact bytes before private entry. Separate exact\nparent/materializer composition and independent review remain BLOCKED.\nSee README.md for the parent boundary and outer collection contract.'),
        (b"SYSTEM = '/usr/bin/python3'\n", b"SYSTEM = '/usr/bin/python3'\nCOPY_CALLER_LEAF = 'metadata-copy-caller-direct-signal.py.txt'\n"),
        (b'kill_attempted, kill_delivered = False, False', b'kill_attempted, kill_syscall_returned = False, False'),
        (b'                            kill_attempted = True\n                            proc.kill()  # Only this retained, unreaped direct child.\n                            kill_delivered = True',
         b'                            if overflow or timed_out:\n                                kill_attempted = True\n                                # Only this retained, still-unreaped child; no extra poll/OS query.\n                                os.kill(proc.pid, signal.SIGKILL)\n                                kill_syscall_returned = True  # Return is not delivery, exit or reap.'),
        (b'kill_attempted=kill_attempted, kill_delivered=kill_delivered,', b'kill_attempted=kill_attempted, kill_syscall_returned=kill_syscall_returned,'),
        (b"    require(len(sys.argv) == 5 and sys.argv[1] in ('source', 'copy'), 'fixed CLI')",
         b"    require(len(sys.argv) == 5 and sys.argv[1] == 'copy', 'fixed COPY-only CLI')\n    require(Path(__file__).name == COPY_CALLER_LEAF, 'fixed COPY caller leaf')"),
        (b"    result = dict(schema='openergy.metadata-observations.v2', clock=CLOCK, mode=mode, expected_basis_sha256=EXPECTED_HASH,",
         b"    result = dict(schema='openergy.metadata-observations.v3', clock=CLOCK, mode=mode, expected_basis_sha256=EXPECTED_HASH,"),
    ]
    new = old
    for before, after in edits:
        assert new.count(before) == 1, before
        new = new.replace(before, after, 1)
    restored = new
    for before, after in reversed(edits):
        assert restored.count(after) == 1
        restored = restored.replace(after, before, 1)
    assert restored == old
    return new


def patch(old, new, a, b):
    return ''.join(difflib.unified_diff(old.decode().splitlines(True), new.decode().splitlines(True),
                                      fromfile=a, tofile=b, n=0))


def reconstruct(old, delta):
    # Apply zero-context hunk DATA to bytes in memory; never touch a source path.
    source = old.splitlines(True)
    lines = delta.encode().splitlines(True)
    cursor, pos, output = 0, 2, []
    while pos < len(lines):
        match = re.fullmatch(rb'@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@\n', lines[pos])
        assert match
        begin = int(match[1]) - (0 if match[2] == b'0' else 1)
        old_count = int(match[2]) if match[2] is not None else 1
        new_count = int(match[4]) if match[4] is not None else 1
        output += source[cursor:begin]
        cursor = begin
        removed, added = [], []
        pos += 1
        while pos < len(lines) and not lines[pos].startswith(b'@@ '):
            line = lines[pos]
            assert line[:1] in (b'-', b'+')
            (removed if line[:1] == b'-' else added).append(line[1:])
            pos += 1
        assert len(removed) == old_count and len(added) == new_count
        assert removed == source[cursor:cursor+old_count]
        cursor += old_count
        output += added
    return b''.join(output + source[cursor:])


def bind(commit, path):
    path = os.path.normpath(path)
    commit = git('rev-parse', commit).decode().strip()
    raw = git('show', commit + ':' + path)
    mode, kind, blob = git('ls-tree', commit, '--', path).split(b'\t')[0].split()
    assert kind == b'blob' and (ROOT / path).read_bytes() == raw
    return dict(commit=commit, path=path, blob=blob.decode(), mode=mode.decode(),
                bytes=len(raw), sha256=sha(raw), working_bytes_equal=True)


def preserve():
    baseline = strict((OUT / 'baseline.json').read_bytes())
    raw_tree = git('ls-tree', '-rz', '--full-tree', BASE)
    entries = []
    for row in raw_tree.split(b'\0'):
        if row:
            meta, name = row.split(b'\t', 1)
            mode, kind, blob = meta.split()
            assert kind == b'blob'
            entries.append((mode, blob, name))
    batch = git('cat-file', '--batch', input=b''.join(blob + b'\n' for _, blob, _ in entries))
    index = {}
    for row in git('ls-files', '--stage', '-z').split(b'\0'):
        if row:
            meta, name = row.split(b'\t', 1)
            mode, blob, stage = meta.split()
            assert stage == b'0'
            index[name] = (mode, blob)
    rows, errors, offset = [], [], 0
    for mode, blob, name in entries:
        end = batch.index(b'\n', offset)
        returned, kind, size = batch[offset:end].split()
        assert returned == blob and kind == b'blob'
        raw = batch[end + 1:end + 1 + int(size)]
        offset = end + int(size) + 2
        path = ROOT / os.fsdecode(name)
        info = path.lstat()
        data = os.readlink(path).encode() if stat.S_ISLNK(info.st_mode) else path.read_bytes()
        # This sole existing checkout conversion was measured before any old edit.
        expected = raw.replace(b'\n', b'\r\n') if name == b'tools/windows/openergy-run.cmd' else raw
        expected_mode = {b'100644': 0o644, b'100755': 0o755, b'120000': 0o777}[mode]
        if data != expected or stat.S_IMODE(info.st_mode) != expected_mode or index.get(name) != (mode, blob):
            errors.append(os.fsdecode(name))
        rows.append(dict(path=os.fsdecode(name), mode=mode.decode(), blob=blob.decode(),
                         index_mode=index[name][0].decode(), index_blob=index[name][1].decode(),
                         working_kind='link' if stat.S_ISLNK(info.st_mode) else 'file',
                         working_mode=oct(stat.S_IMODE(info.st_mode)), working_bytes=len(data),
                         working_sha256=sha(data)))
    assert not errors, errors
    assert len(rows) == baseline['tracked_count']
    assert sha(canonical(rows)) == baseline['canonical_inventory_sha256']
    assert sha(raw_tree) == baseline['raw_git_tree_sha256']
    # Use index order for the index-entry digest, with new packet entries excluded.
    old_names = {n for _, _, n in entries}
    old_index = b''.join(row + b'\0' for row in git('ls-files', '--stage', '-z').split(b'\0')
                         if row and row.split(b'\t', 1)[1] in old_names)
    assert sha(old_index) == baseline['raw_index_entries_sha256']
    return dict(baseline_parent=BASE, old_entries_compared=len(rows),
                per_file_git_blob_and_index_mode_blob_and_working_bytes_mode_checks=len(rows),
                mismatches=errors, canonical_inventory_sha256=sha(canonical(rows)),
                old_index_entries_sha256=sha(old_index), old_working_conversion='tools/windows/openergy-run.cmd: original CRLF retained',
                old_blob_and_tree_identity='all parent entries retained by exact object ID; no old entry changed',
                stage='pre-commit; post-commit checks reported outside packet to avoid self-reference')


def construct():
    old = git('show', '9b50dd7e3a672eafe319be33141cb875d54a471e:' + R9 + 'metadata-caller.py.txt')
    assert len(old) == 46908 and sha(old) == '01623cd4759f96cd0066a9456a2680bff476bc37b5b1acb61ec40388582a5e9f'
    new = candidate(old)
    put(LEAF, new)
    put('from-frozen-r9.patch', patch(old, new, 'a/' + R9 + 'metadata-caller.py.txt', 'b/' + OUT.name + '/' + LEAF))
    before, after = functions(old), functions(new)
    inventory = []
    changed = []
    for name in before:
        left, right = before[name], after[name]
        state = 'UNCHANGED' if left[1] == right[1] else 'CHANGED'
        inventory.append(dict(name=name, state=state, before=dict(lines=[left[2], left[3]], bytes=len(left[1]), sha256=sha(left[1])),
                              after=dict(lines=[right[2], right[3]], bytes=len(right[1]), sha256=sha(right[1]))))
        if state == 'CHANGED':
            changed.append(name)
    assert set(before) == set(after) and changed == ['collect', 'main']
    save('function-inventory.json', inventory)
    for side, table in [('before', before), ('after', after)]:
        put('changed-functions.' + side + '.txt', b'\n\n'.join(table[n][1] for n in changed))
    left, right = globals_only(old), globals_only(new)
    put('global-diff.patch', patch(left, right, 'a/R9-globals', 'b/COPY-alias-globals'))
    save('global-inventory.json', dict(before_sha256=sha(left), after_sha256=sha(right),
         changes=['module docstring', 'COPY_CALLER_LEAF constant'],
         all_other_module_nodes_byte_equal=True, entry_block_byte_equal=True))
    specs = [
        ('9b50dd7e3a672eafe319be33141cb875d54a471e', R9 + 'metadata-caller.py.txt'),
        ('1c0e4bcdc2dddcb64a3277db34d5ccd5e872fab2', 'docs/verification/m1-metadata-r10-review-2026-10-25-t3-a1a624d3/REVIEW.md'),
        ('f33a1fd761770cb2a2961f9cf55b59dc4aeb40d5', 'docs/verification/m1-copy-parent-full-review-2026-10-25-t3-9edf92e5/REVIEW.md'),
        ('931d2c3', 'docs/verification/m1-copy-parent-full-review-2026-10-25-t3-9edf92e5/REVIEW.md'),
        ('c82cc12b', PARENT + 'copy-parent.py.txt'), ('6ef1663', PARENT + 'copy-parent.py.txt'),
        (BASE, PARENT + 'direct-signal-library-audit.json'),
        (BASE, 'docs/verification/m1-copy-parent-full-review-2026-10-25-t3-9edf92e5/public-stdlib-source-evidence.json'),
    ]
    paths = [R9 + name for name in ['input-pins.json', 'command-vectors.json', 'startup-contract.json',
             'parent-boundary.schema.json', 'compact-success.schema.json', 'compact-success-contract.md', 'repair-contract.md']]
    paths += ['docs/verification/m1-source-metadata-observed-2026-10-25-t3-52422617/' + n for n in ['README.md', 'validation.json']]
    paths += ['docs/verification/m1-prepare-parent-root-layout-repair-2026-10-25-t3-1f8914f8/' + n for n in ['prepare-parent.py.txt', 'README.md', 'root-layout.json']]
    paths += ['docs/verification/m1-ordinary-fresh-flow-plan-2026-10-24-t3-5463ce44/' + n for n in ['operator-plan.md', 'receipt-contract.md', 'receipt.schema.json']]
    paths += ['docs/verification/m1-next-reviewed-ordinary-flow-2026-10-25-t3-a8a4ece5/' + n for n in ['README.md', 'next-operation.json', 'file-qualification.recipe.py.txt']]
    pins = strict((ROOT / R9 / 'input-pins.json').read_bytes())['files']
    paths += [v['repository_path'] for v in pins.values()]
    specs += [(BASE, p) for p in dict.fromkeys(os.path.normpath(p) for p in paths)]
    bindings = [bind(c, p) for c, p in specs]
    save('input-bindings.json', bindings)
    parent_raw = (ROOT / PARENT / 'copy-parent.py.txt').read_bytes()
    fixed = assignments(parent_raw)
    validation = strict((ROOT / 'docs/verification/m1-source-metadata-observed-2026-10-25-t3-52422617/validation.json').read_bytes())
    assert fixed['SOURCE_RAWS'] == validation['record_files'] and len(fixed['SOURCE_RAWS']) == 212
    assert fixed['SOURCE_RESULT'] == dict(bytes=validation['raw_result_bytes'], sha256=validation['raw_result_sha256'])
    assert len(validation['records']) == 53 and all(type(r['actual_exit']) is int and r['actual_exit'] == 0 for r in validation['records'])
    assert validation['no_timeout_kill_overflow_drop_or_publication_error'] is True
    pf = functions(parent_raw)
    save('compatibility-observations.json', dict(
         current_copy_parent=dict(commit='c82cc12b8a2d709d8ce1e384545475159b24d2a9', import_commit='6ef1663caee7ec8e421daaaf9effca6796ecedf0',
             bytes=len(parent_raw), sha256=sha(parent_raw), execution_in_this_task=False,
             functions={n: dict(sha256=sha(pf[n][1]), lines=[pf[n][2], pf[n][3]])
                        for n in ['collect', 'inspect_fixed_inputs', 'copy_record_complete', 'inspect_copy_result', 'inspect_saved_source', 'main']},
             own_collect_direct_signal=True, still_launches_old_leaf=True, still_requires_copy_full_v2=True,
             still_requires_copy_kill_delivered=True, can_launch_new_alias_as_is=False),
         saved_SOURCE_public_data=dict(result=fixed['SOURCE_RESULT'], raw_rows=212,
             raw_manifest_canonical_sha256=sha(canonical(fixed['SOURCE_RAWS'])),
             boundary_files=len(fixed['SOURCE_BOUNDARY_BINDINGS']), boundary_canonical_sha256=sha(canonical(fixed['SOURCE_BOUNDARY_BINDINGS'])),
             records=53, all_actual_exit_zero=True, no_kill_reported=True,
             validation_status=validation['status'], validation_caller_sha256=validation['caller_sha256'],
             raw_private_files_reopened=False, source_repeated=False,
             proof_scope='Exact public recorded data and parent literal identities only; no new private evidence inspection'),
         old_source_fresh_source_and_complete_byte_exact=True,
         new_copy_full_result_schema='openergy.metadata-observations.v3',
         new_copy_record_key='kill_syscall_returned', old_source_schema='openergy.metadata-observations.v2',
         exact_parent_materializer_composition='BLOCKED, separate work and fresh independent review required'))
    vectors = strict((ROOT / R9 / 'command-vectors.json').read_bytes())['commands']
    labels = ['copy-files-before'] + [v['label'] for v in vectors] + ['copy-files-after']
    save('closed-cli.json', dict(execution=False, script=LEAF, argv_count=5, allowed_modes=['copy'], source_rejected_before_root_or_clock=True,
         exact_leaf_required=True, cwd='ROOT', script_parent='ROOT', root='SAME existing ROOT as old caller and saved SOURCE v2',
         future_private_vector=['PRIVATE', '-B', 'ROOT/' + LEAF, 'copy', 'ROOT', 'originalStart', 'stdinHash'],
         parent_input='exact bounded stdin <=1048576 bytes, original SHA argv[4]',
         start='original canonical ASCII decimal argv[3], <=20 digits; original start+120s',
         parent_keys=7, authentication_keys=19, source_file_qualification=None,
         observations=labels, observation_count=52, raw_status_comparator_refs=208,
         produced_full_result_schema='openergy.metadata-observations.v3', saved_source_schema='openergy.metadata-observations.v2',
         compact_schema_unchanged='openergy.metadata-success.v1', compact_success_max_bytes=512,
         materialization='BLOCKED pending separate exact parent/materializer composition review; AFTER prepare only',
         current_copy_parent_compatible=False, startup='byte-exact R9 ten-key COPY map; no bootstrap exception'))
    status = dict(label={'enum': labels}, argv={'type': 'array', 'items': {'type': 'string'}, 'minItems': 1},
                  pid={'type': ['integer', 'null'], 'minimum': 1})
    booleans = ['handle_retained', 'reaped', 'stdout_eof', 'stderr_eof', 'unread_or_dropped_unknown',
                'overflow', 'timed_out', 'deadline_loss', 'kill_attempted', 'kill_syscall_returned']
    status.update({n: {'type': 'boolean'} for n in booleans})
    status.update({n: {'type': ['integer', 'null']} for n in ['actual_exit', 'kill_eligibility_exit']})
    status.update({n: {'type': 'integer', 'minimum': 0} for n in ['observed_stream_bytes', 'dropped_observed_bytes', 'started_ns', 'command_deadline_ns', 'finished_ns']})
    status['kill_eligibility_observed_ns'] = {'type': ['integer', 'null'], 'minimum': 0}
    status.update({n: {'type': ['string', 'null']} for n in ['error', 'kill_or_reap_error']})
    clock = strict((ROOT / R9 / 'closed-cli.json').read_bytes())['clock']
    status['clock'] = {'const': clock}
    reference = dict(type='object', additionalProperties=False, required=['path', 'bytes', 'sha256'],
                     properties=dict(path={'type': 'string'}, bytes={'type': 'integer', 'minimum': 0, 'maximum': 8388608},
                                     sha256={'type': 'string', 'pattern': '^[0-9a-f]{64}$'}))
    def obj(props):
        return dict(type='object', additionalProperties=False, required=list(props), properties=props)
    published = dict(status, stdout={'$ref': '#/$defs/reference'}, stderr={'$ref': '#/$defs/reference'})
    defs = dict(reference=reference, collector_status=obj(status), published_status=obj(published))
    defs['retained_status'] = dict(type='object', additionalProperties=False,
                                  required=list(status), properties=published)
    defs['success_published_status'] = dict(allOf=[{'$ref': '#/$defs/published_status'},
        {'properties': dict(actual_exit={'const': 0}, kill_eligibility_exit={'const': 0},
            kill_attempted={'const': False}, kill_syscall_returned={'const': False}, reaped={'const': True},
            handle_retained={'const': True}, pid={'type': 'integer', 'minimum': 1},
            kill_eligibility_observed_ns={'type': 'integer', 'minimum': 0},
            stdout_eof={'const': True}, stderr_eof={'const': True}, overflow={'const': False},
            timed_out={'const': False}, deadline_loss={'const': False},
            unread_or_dropped_unknown={'const': False}, dropped_observed_bytes={'const': 0},
            error={'const': None}, kill_or_reap_error={'const': None})}])
    defs['direct_child'] = obj(dict(label={'enum': labels}, argv=status['argv'], pid=status['pid'],
         actual_exit={'type': ['integer', 'null']}, handle_retained={'const': True},
         collector_status={'anyOf': [{'$ref': '#/$defs/retained_status'}, {'type': 'null'}]}))
    save('child-status-v3.schema.json', {'$schema': 'https://json-schema.org/draft/2020-12/schema',
        '$id': 'urn:openergy:metadata-copy-child-status:v3', '$ref': '#/$defs/published_status', '$defs': defs,
        '$comment': 'Author DATA contract for new full COPY openergy.metadata-observations.v3 only. Raw status has no schema key. Signed exits/null are failure evidence; success requires 0. Clock/PID/byte constraints do not prove causal order or semantics. Generic complete and old SOURCE v2 remain unchanged.'})


def verify():
    old = git('show', '9b50dd7e3a672eafe319be33141cb875d54a471e:' + R9 + 'metadata-caller.py.txt')
    new = (OUT / LEAF).read_bytes()
    assert new == candidate(old)
    before, after = functions(old), functions(new)
    assert {n for n in before if before[n][1] != after[n][1]} == {'collect', 'main'}
    for name in ['fresh_source', 'complete', 'Writer', 'inspect_parent_helper', 'parent_reference', 'perform', 'direct_children', 'emit_summary']:
        assert before[name][1] == after[name][1]
    # All module nodes other than the docstring/new leaf constant and these functions are byte-exact.
    def module_nodes(data):
        lines = data.splitlines(True)
        return [b''.join(lines[n.lineno-1:n.end_lineno]) for n in parse(data).body
                if not (isinstance(n, ast.Expr) and isinstance(n.value, ast.Constant) and isinstance(n.value.value, str))
                and not (isinstance(n, ast.FunctionDef) and n.name in ('main', 'collect'))
                and not (isinstance(n, ast.Assign) and any(isinstance(t, ast.Name) and t.id == 'COPY_CALLER_LEAF' for t in n.targets))]
    assert module_nodes(old) == module_nodes(new)
    collect = after['collect'][1].decode()
    assert 'kill_delivered' not in new.decode()
    assert collect.count('os.kill(proc.pid, signal.SIGKILL)') == 1 and 'proc.kill(' not in collect
    assert 'if overflow or timed_out:' in collect
    assert 'kill_eligibility_exit = proc.poll()\n                    try:\n                        kill_eligibility_observed_ns = clock_ns()' in collect
    assert 'kill_eligibility_observed_ns < deadline and kill_eligibility_observed_ns < phase_deadline' in collect
    assert "status = proc.returncode if proc is not None else None" in collect
    main = after['main'][1].decode()
    assert main.index("sys.argv[1] == 'copy'") < main.index('canonical(sys.argv[2])') < main.index('clock_ns()')
    assert main.count('openergy.metadata-observations.v3') == 1
    assert b'openergy.metadata-observations.v2' in after['fresh_source'][1]
    assert b'kill_delivered' not in after['complete'][1]
    assert (OUT / 'from-frozen-r9.patch').read_text() == patch(old, new, 'a/' + R9 + 'metadata-caller.py.txt', 'b/' + OUT.name + '/' + LEAF)
    assert reconstruct(old, (OUT / 'from-frozen-r9.patch').read_text()) == new
    for side, table in [('before', before), ('after', after)]:
        assert (OUT / ('changed-functions.' + side + '.txt')).read_bytes() == b'\n\n'.join(table[n][1] for n in ['collect', 'main'])
    inventory = strict((OUT / 'function-inventory.json').read_bytes())
    assert {r['name'] for r in inventory} == set(before)
    for row in inventory:
        for side, table in [('before', before), ('after', after)]:
            item = table[row['name']]
            assert row[side] == dict(lines=[item[2], item[3]], bytes=len(item[1]), sha256=sha(item[1]))
    assert (OUT / 'global-diff.patch').read_text() == patch(globals_only(old), globals_only(new), 'a/R9-globals', 'b/COPY-alias-globals')
    assert reconstruct(globals_only(old), (OUT / 'global-diff.patch').read_text()) == globals_only(new)
    pins = strict((ROOT / R9 / 'input-pins.json').read_bytes())['files']
    assert len(pins) == 17
    for row in pins.values():
        raw = (ROOT / row['repository_path']).read_bytes()
        assert len(raw) == row['bytes'] and sha(raw) == row['sha256']
    modules = strict((ROOT / 'docs/verification/ordinary-combined-context-tuple-repair-2026-10-25/source-checks.json').read_bytes())['python_files']
    assert len(modules) == 10
    assert all(modules[n]['sha256'] == pins[n]['sha256'] and modules[n]['bytes'] == pins[n]['bytes'] for n in modules)
    vectors = strict((ROOT / R9 / 'command-vectors.json').read_bytes())['commands']
    assert len(vectors) == len({v['label'] for v in vectors}) == 50
    schema = strict((OUT / 'child-status-v3.schema.json').read_bytes())
    for key in ['collector_status', 'published_status']:
        props = schema['$defs'][key]['properties']
        for n in ['actual_exit', 'kill_eligibility_exit']:
            assert props[n] == {'type': ['integer', 'null']}
        for n in ['pid', 'started_ns', 'finished_ns', 'command_deadline_ns', 'kill_eligibility_observed_ns', 'observed_stream_bytes', 'dropped_observed_bytes']:
            assert props[n]['minimum'] >= 0
        assert 'kill_delivered' not in props and 'kill_syscall_returned' in props
    assert schema['$defs']['direct_child']['properties']['actual_exit'] == {'type': ['integer', 'null']}
    assert schema['$defs']['success_published_status']['allOf'][1]['properties']['actual_exit'] == {'const': 0}
    # Closed status keys are derived from the actual dict construction, not an example receipt.
    collector_node = after['collect'][0]
    record_calls = [n.value for n in ast.walk(collector_node) if isinstance(n, ast.Assign)
                    and any(isinstance(t, ast.Name) and t.id == 'record' for t in n.targets)]
    assert len(record_calls) == 1 and isinstance(record_calls[0], ast.Call)
    core_keys = {k.arg for k in record_calls[0].keywords}
    assert core_keys == set(schema['$defs']['collector_status']['required'])
    assert core_keys | {'stdout', 'stderr'} == set(schema['$defs']['published_status']['required'])
    assert schema['$defs']['retained_status']['required'] == schema['$defs']['collector_status']['required']
    bindings = strict((OUT / 'input-bindings.json').read_bytes())
    for row in bindings:
        assert bind(row['commit'], row['path']) == row
    for path in OUT.glob('*.json'):
        strict(path.read_bytes())
    save('preservation.json', preserve())
    save('static-observations.json', dict(execution=False, supplied_source_execution_import_compile=False,
         supplied_checker_execution=False, tests_or_mocks=False, private_state_access=False,
         candidate_bytes=len(new), candidate_sha256=sha(new), candidate_lines=len(new.splitlines()),
         python39_ast_data_parsed=True, changed_functions=['collect', 'main'],
         unchanged_declarations=len(before)-2, declarations_including_class_and_nested=len(before),
         reverse_exact_edits_restore_all_r9_bytes=True, other_module_nodes_and_entry_byte_exact=True,
         exact_patch_recomputed=True, patch_hunks_reconstruct_all_new_bytes=True,
         full_changed_function_excerpts_and_every_inventory_hash_checked=True,
         global_patch_reconstructed=True, fresh_source_v2_and_complete_byte_exact=True,
         closed_copy_only_cli_and_fixed_leaf_static=True, no_source_mode_reachable_from_main=True,
         one_direct_os_kill_site_cap_or_deadline_only=True, original_poll_clock_strict_ends_wait_and_actual_returncode=True,
         schema_domains_inspected_as_data_only=True, signed_exit_domains_no_minimum=True,
         successful_exit_const_zero=True, schema_is_no_runtime_receipt=True,
         input_bindings_rehashed=len(bindings), pins_rehashed=17, module_pins_checked=10, vectors_count=50,
         copy_observation_count=52, durable_raw_status_comparator_refs=208,
         new_parent_materializer_not_supplied=True, composition='BLOCKED', independent_fresh_review='REQUIRED',
         reader_notes=[
             'Initial opening-inventory stdout exceeded tool output cap; JSON parse failed after baseline summary write. No supplied source or old file changed. Per-file Git/raw comparison found only pre-existing CRLF and no mode differences; canonical closing digest exactly reproduces opening actual inventory.',
             'First construct reader used literal repository_path containing ../ as a Git tree path; Git refused that spelling. Reader now normalizes only its public Git lookup paths. Frozen pin strings and all supplied source bytes stay unchanged.',
             'Initial staged whitespace check exited2 for added blank EOF separators in changed-function excerpts. Author excerpt formatter corrected; function body bytes stay unchanged.']))
    files = sorted(p for p in OUT.iterdir() if p.name != 'SHA256SUMS')
    assert all(p.is_file() and not p.is_symlink() for p in files)
    put('SHA256SUMS', ''.join(sha(p.read_bytes()) + '  ' + p.name + '\n' for p in files))
    print(json.dumps(dict(candidate_sha256=sha(new), candidate_bytes=len(new), declarations=len(before),
                         unchanged=len(before)-2, packet_files=len(files)+1, old_preserved=4566)))


assert OUT == ROOT / 'docs/verification/m1-copy-only-caller-direct-signal-2026-10-25-t3-56c87ccf'
assert sys.argv[1:] in [['construct'], ['verify']]
if sys.argv[1] == 'construct':
    construct()
else:
    verify()
