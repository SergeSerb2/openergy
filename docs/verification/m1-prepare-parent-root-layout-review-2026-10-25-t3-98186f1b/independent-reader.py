"""Reviewer-owned public byte/JSON/AST-data reader; never imports supplied code.
Run only this reader. All target programs remain bytes or AST data.
"""
import ast
import copy
import hashlib
import json
import math
import os
from pathlib import Path
import re
import stat
import subprocess

OUT = Path(__file__).parent
PACKET = Path('docs/verification/m1-prepare-parent-root-layout-repair-2026-10-25-t3-1f8914f8')
CANDIDATE = '02f94afbe3e1001b8d3e8fc4d79976475485f661'
PARENT = '7e5de706e7457de5f4880cdef180b27536dc0d5e'
IMPORT = '95181be'
BASIS = '88e4444b8e5604fe9bfde7e768ea31d138ab64be'
BASIS_PATH = 'docs/verification/m1-prepare-parent-admission-cache-repair-2026-10-25-t3-53a3c637/prepare-parent.py.txt'
ENV = dict(os.environ, GIT_OPTIONAL_LOCKS='0')


def git(*args):
    return subprocess.check_output(['git', *args], env=ENV)


def sha(b):
    return hashlib.sha256(b).hexdigest()


def ident(b):
    return dict(bytes=len(b), sha256=sha(b))


def canon(v):
    return json.dumps(v, sort_keys=True, ensure_ascii=True, separators=(',', ':'), allow_nan=False).encode()


def strict(b):
    def pairs(items):
        out = {}
        for k, v in items:
            assert k not in out
            out[k] = v
        return out
    def finite(s):
        v = float(s)
        assert math.isfinite(v)
        return v
    def reject(s):
        raise ValueError('nonfinite JSON ' + s)
    return json.loads(b, object_pairs_hook=pairs, parse_constant=reject, parse_float=finite)


def save(name, v):
    (OUT / name).write_text(json.dumps(v, sort_keys=True, indent=2, allow_nan=False) + '\n')


def tree(commit, path=None):
    args = ['ls-tree', '-r', '-z', commit]
    if path:
        args += ['--', str(path)]
    out = {}
    for row in git(*args).split(b'\0'):
        if row:
            meta, name = row.split(b'\t')
            mode, kind, blob = meta.decode().split()
            out[os.fsdecode(name)] = dict(mode=mode, kind=kind, blob=blob)
    return out


def declarations(b):
    lines = b.splitlines(keepends=True)
    nodes = {}
    def walk(body, prefix=''):
        for node in body:
            if isinstance(node, (ast.ClassDef, ast.FunctionDef, ast.AsyncFunctionDef)):
                start = min([node.lineno] + [d.lineno for d in node.decorator_list])
                name = prefix + node.name
                raw = b''.join(lines[start - 1:node.end_lineno])
                nodes[name] = dict(start_line=start, end_line=node.end_lineno, **ident(raw), raw=raw)
                walk(node.body, name + '.')
    parsed = ast.parse(b, feature_version=(3, 9))
    walk(parsed.body)
    gaps, pos = [], 0
    for node in parsed.body:
        if isinstance(node, (ast.ClassDef, ast.FunctionDef, ast.AsyncFunctionDef)):
            start = min([node.lineno] + [d.lineno for d in node.decorator_list]) - 1
            gaps.append(b''.join(lines[pos:start]))
            pos = node.end_lineno
    gaps.append(b''.join(lines[pos:]))
    return parsed, nodes, gaps


def literals(b):
    out = {}
    for node in ast.parse(b, feature_version=(3, 9)).body:
        if isinstance(node, ast.Assign) and len(node.targets) == 1 and isinstance(node.targets[0], ast.Name):
            try:
                out[node.targets[0].id] = ast.literal_eval(node.value)
            except (ValueError, TypeError):
                pass
    return out


def reconstruct(old, patch):
    lines = patch.splitlines(keepends=True)
    assert lines[0].startswith(b'--- ') and lines[1].startswith(b'+++ ')
    original = old.splitlines(keepends=True)
    result, pos, i, hunks = [], 0, 2, []
    while i < len(lines):
        m = re.fullmatch(rb'@@ -(\d+),(\d+) \+(\d+),(\d+) @@\n', lines[i])
        assert m, lines[i]
        a, ac, b, bc = map(int, m.groups())
        result += original[pos:a - 1]
        pos = a - 1
        assert len(result) == b - 1
        start, i, oc, nc = i, i + 1, 0, 0
        while i < len(lines) and not lines[i].startswith(b'@@ '):
            row = lines[i]
            assert row[:1] in (b' ', b'-', b'+')
            if row[:1] in (b' ', b'-'):
                assert original[pos] == row[1:]
                pos += 1
                oc += 1
            if row[:1] in (b' ', b'+'):
                result.append(row[1:])
                nc += 1
            i += 1
        assert (oc, nc) == (ac, bc)
        hunks.append(dict(old_start=a, old_count=ac, new_start=b, new_count=bc,
                          **ident(b''.join(lines[start:i]))))
    return b''.join(result + original[pos:]), hunks


packet_files = {p.name: p.read_bytes() for p in PACKET.iterdir()}
assert len(packet_files) == 40
jsons = {n: strict(b) for n, b in packet_files.items() if n.endswith('.json')}
entries = tree(CANDIDATE, PACKET)
assert len(entries) == 40
assert git('rev-parse', CANDIDATE + '^').decode().strip() == PARENT
changed = git('diff-tree', '--no-commit-id', '--name-status', '-r', CANDIDATE).decode().splitlines()
assert len(changed) == 40 and all(x.startswith('A\t' + str(PACKET) + '/') for x in changed)
packet_rows = []
for name, b in sorted(packet_files.items()):
    path = str(PACKET / name)
    ent = entries[path]
    for commit in [CANDIDATE, IMPORT, 'HEAD']:
        assert tree(commit, path)[path] == ent
        assert git('show', commit + ':' + path) == b
    assert ent['mode'] == '100644' and stat.S_IMODE((PACKET / name).stat().st_mode) == 0o644
    packet_rows.append(dict(path=path, **ent, **ident(b), working_mode='0o644'))
manifest = jsons['manifest.json']
assert manifest['payload_count'] == len(manifest['payload']) == 38
assert {x['path'] for x in manifest['payload']} == set(packet_files) - {'manifest.json', 'SHA256SUMS'}
for row in manifest['payload']:
    assert ident(packet_files[row['path']]) == {k: row[k] for k in ['bytes', 'sha256']}
    assert row['git_mode'] == '100644' and row['working_mode'] == '0o644'
sums = [x.split('  ', 1) for x in packet_files['SHA256SUMS'].decode().splitlines()]
assert len(sums) == 39 and {x[1] for x in sums} == set(packet_files) - {'SHA256SUMS'}
assert all(sha(packet_files[n]) == h for h, n in sums)
save('packet-bindings.json', dict(candidate=CANDIDATE, parent=PARENT, imported=git('rev-parse', IMPORT).decode().strip(),
     packet_files=packet_rows, files=40, manifest_payload_entries=38, checksum_entries_including_manifest=39,
     candidate_adds_only_packet=True, execution=False))

verified = []
for row in jsons['input-bindings.json']['public_Git_bindings']:
    b = git('show', row['commit'] + ':' + row['path'])
    ent = tree(row['commit'], row['path'])[row['path']]
    assert ent['blob'] == row['blob'] and ent['mode'] == row['mode']
    assert ident(b) == {k: row[k] for k in ['bytes', 'sha256']}
    assert Path(row['path']).read_bytes() == b
    assert tree('HEAD', row['path'])[row['path']] == ent
    verified.append(row)
assert len(verified) == jsons['input-bindings.json']['count'] == 222
save('public-input-bindings.json', dict(execution=False, count=222, entries=verified))

baseline_inventory = jsons['baseline-tracked-inventory.json']
bt = tree(PARENT)
assert len(bt) == baseline_inventory['tracked_count'] == 4422
assert set(bt) == {r['path'] for r in baseline_inventory['entries']}
for row in baseline_inventory['entries']:
    assert bt[row['path']]['blob'] == row['blob'] and bt[row['path']]['mode'] == row['git_mode']
    b = Path(row['path']).read_bytes()
    assert len(b) == row['working_bytes'] and sha(b) == row['working_sha256']
    assert oct(stat.S_IMODE(Path(row['path']).stat().st_mode)) == row['working_mode']

source = packet_files['prepare-parent.py.txt']
old = git('show', BASIS + ':' + BASIS_PATH)
assert ident(source) == dict(bytes=109444, sha256='b3fbd793a0709c7857e34efca667a081a0486accfd0f3ad13a8068484b7eb8d9')
assert len(source.splitlines()) == 1927
assert ident(old) == dict(bytes=106001, sha256='fce82ca2775607fd86e0ef8dbb8c44fd3cd5a213815ae77cb9501327160ee57d')
assert sha(packet_files['from-88e4444.patch']) == '228c06f5e57b74016bd19e54168519fea36e06a150a83943a559b9e1c4d9b217'
rebuilt, hunks = reconstruct(old, packet_files['from-88e4444.patch'])
assert rebuilt == source and len(hunks) == 7
assert hunks == jsons['bindings.json']['patch']['hunks']
parsed, after, ag = declarations(source)
_, before, bg = declarations(old)
function_rows = []
for row in jsons['function-inventory.json']['declarations']:
    name = row['name']
    a, b = after.get(name), before.get(name)
    assert {k: v for k, v in a.items() if k != 'raw'} == row['after']
    if b is not None:
        assert {k: v for k, v in b.items() if k != 'raw'} == row['before']
    state = 'ADDED' if b is None else ('UNCHANGED' if a['raw'] == b['raw'] else 'CHANGED')
    assert state == row['state']
    function_rows.append(row)
assert len(after) == 45 and len(before) == 44
assert {r['name'] for r in function_rows} == set(after)
assert [r['name'] for r in function_rows if r['state'] == 'CHANGED'] == ['import_common', 'collect', 'main']
assert sum(r['state'] == 'UNCHANGED' for r in function_rows) == 41
ml = jsons['module-layout.json']
assert [ident(x) for x in bg] == ml['before_module_segments']
assert [ident(x) for x in ag] == ml['after_module_segments']
module_after, module_hunks = reconstruct(b''.join(bg), packet_files['global-diff.patch'])
assert module_after == b''.join(ag)
global_addition = b''.join(source.splitlines(keepends=True)[918:928])
assert ident(global_addition) == ml['global_addition_with_separator']
assert global_addition == packet_files['global-additions.txt'] + b'\n'
restored = source
for name in ['import_common', 'collect', 'main']:
    assert restored.count(after[name]['raw']) == 1
    restored = restored.replace(after[name]['raw'], before[name]['raw'], 1)
restored = restored.replace(global_addition, b'', 1)
assert restored.count(after['closed_root_layout']['raw'] + b'\n\n') == 1
restored = restored.replace(after['closed_root_layout']['raw'] + b'\n\n', b'', 1)
assert restored == old
for which, names, nodes in [('before', ['import_common', 'collect', 'main'], before),
                            ('after', ['closed_root_layout', 'import_common', 'collect', 'main'], after)]:
    transcript = b'\n'.join(b'### ' + n.encode() + b'\n' + nodes[n]['raw'] for n in names)
    assert transcript == packet_files['changed-functions.' + which + '.txt']
globals_old, globals_new = literals(old), literals(source)
assert set(globals_new) - set(globals_old) == {'ROOT_DATA_ADDITIONS'}
assert all(type(globals_new[n]) is type(v) and globals_new[n] == v for n, v in globals_old.items())
global_rows = []
for node in parsed.body:
    if isinstance(node, ast.Assign):
        start, end = node.lineno, node.end_lineno
        raw = b''.join(source.splitlines(keepends=True)[start - 1:end])
        global_rows.append(dict(targets=[ast.unparse(t) for t in node.targets], start_line=start, end_line=end, **ident(raw)))
save('source-bindings.json', dict(execution=False, source=ident(source), basis=ident(old), patch=ident(packet_files['from-88e4444.patch']),
     seven_hunks=hunks, functions=function_rows, globals=global_rows, before_module_segments=[ident(x) for x in bg],
     after_module_segments=[ident(x) for x in ag], module_patch_hunks=module_hunks,
     exact_whole_source_restoration=True, unchanged_declarations=41, new_globals=['ROOT_DATA_ADDITIONS', 'ROOT_INITIAL_NAMES'],
     extra_new_separator_bytes=2, all_prior_literal_globals_equal=True, author_parent_inventory_entries_verified=4422))

# Upstream fixed declarations are decoded only as literal DATA.
caller_path = 'docs/verification/m1-metadata-r9-repair-2026-10-25-t3-9b2c2354/metadata-caller.py.txt'
source_path = 'docs/verification/m1-source-stdin-payload-kill-gate-repair-2026-10-25-t3-da89534d/source-payload.py.txt'
four_path = 'docs/verification/m1-system-stdin-materializer-2026-10-25-t3-4878b759/materializer.py.txt'
eight_path = 'docs/verification/m1-artifact-data-materializer-2026-10-25-t3-f0b1e4d1/materializer.py.txt'
caller, launcher, four, eight = [literals(Path(p).read_bytes()) for p in [caller_path, source_path, four_path, eight_path]]
g = globals_new
assert g['INITIAL_INPUTS'] == launcher['INITIAL_INPUTS']
assert g['COMPLETED'] == launcher['COMPLETED'] == four['COMPLETED'] == eight['COMPLETED']
assert {k: (v['bytes'], v['sha256']) for k, v in g['COMPLETED'].items()} == caller['COMPLETED_FILES']
assert g['SOURCE_BOUNDARY'] == launcher['OUTPUTS'] == eight['SOURCE_BOUNDARY']
assert set(g['EIGHT']) == set(eight['INPUTS'])
assert set(four['INPUTS']) - set(g['INITIAL_INPUTS']) == {'metadata-caller.py.txt'}
assert set(g['ROOT_DATA_ADDITIONS']) == {'metadata-caller.py.txt', 'metadata-source-result.json', 'metadata-source-once.json', four['GUARD'], four['RECEIPT'], eight['GUARD'], eight['RECEIPT']}
vectors = strict(Path('docs/verification/m1-metadata-r9-repair-2026-10-25-t3-9b2c2354/command-vectors.json').read_bytes())['commands']
labels = ['entry-template', 'python-version', 'python-architecture'] + [v['label'] for v in vectors]
assert tuple(labels) == eight['LABELS'] and len(labels) == 53
names = [f'metadata-source-{label}{suffix}' for label in labels for suffix in ['.stdout', '.stderr', '-os.json', '-comparison.json']]
assert tuple(names) == eight['RAW_NAMES'] and names == [r['leaf'] for r in g['SOURCE_RAWS']]
validation = strict(Path('docs/verification/m1-source-metadata-observed-2026-10-25-t3-52422617/validation.json').read_bytes())
assert g['SOURCE_RAWS'] == validation['record_files']
assert sum(r['bytes'] for r in g['SOURCE_RAWS']) == 213334
assert sha(canon(g['SOURCE_RAWS']) + b'\n') == validation['raw_records_manifest_sha256'] == '2baea069155b633485b1e8aed8fe72f3bf746f67de83a84f07a2daf0648fe4ee'
receipt = strict(Path('docs/verification/m1-artifact-data-observed-2026-10-25-t3-52422617/materializer-receipt.json').read_bytes())
assert receipt['completed_files'] == g['COMPLETED']
assert receipt['files'] == {n: dict(path=g['ROOT'] + '/' + n, mode='0o600', **v) for n, v in g['EIGHT'].items()}
groups = dict(INITIAL_INPUTS=sorted(g['INITIAL_INPUTS']), COMPLETED=sorted(g['COMPLETED']), SOURCE_RAWS=sorted(names),
              SOURCE_BOUNDARY=sorted(g['SOURCE_BOUNDARY']), EIGHT=sorted(g['EIGHT']), ADDITIONS=sorted(g['ROOT_DATA_ADDITIONS']))
union = set().union(*map(set, groups.values()))
assert sum(map(len, groups.values())) == len(union) == 260
layout = jsons['root-layout.json']
assert groups == {k: sorted(v) for k, v in layout['provenance_groups'].items()}
assert sorted(union) == layout['initial_names']
later = union | set(g['OUTPUTS'][:4])
assert len(later) == 264 and sorted(later) == layout['before_common_and_private_names']
assert layout['entry_additions'] == list(g['OUTPUTS'][:4])
pins = jsons['pin-bindings.json']
assert len(pins['source_pins']) == 17 and {x['name']: x['binding'] for x in pins['source_pins']} == g['PINS']
for pin in pins['source_pins']:
    b = git('show', pin['source']['commit'] + ':' + pin['source']['path'])
    assert ident(b) == pin['binding']
checks = strict(Path('docs/verification/ordinary-combined-context-tuple-repair-2026-10-25/source-checks.json').read_bytes())['python_files']
materialization_path = Path('docs/verification/ordinary-combined-context-tuple-repair-2026-10-25/materialization.json')
modules = strict(materialization_path.read_bytes())['modules']
assert pins['ten_modules'] == modules and len(modules) == len(checks) == 10
for row in modules:
    b = (materialization_path.parent / row['source']).read_bytes()
    check_binding = {k: checks[row['destination']][k] for k in ['bytes', 'sha256']}
    assert sha(b) == row['sha256'] and ident(b) == check_binding == g['PINS'][row['destination']]
save('layout-provenance.json', dict(execution=False, groups=groups, initial_names=sorted(union), before_entry_names=sorted(later),
     counts={k: len(v) for k, v in groups.items()}, source_raw_manifest=ident(canon(g['SOURCE_RAWS']) + b'\n'),
     source_result=g['SOURCE_RESULT'], raw_total_bytes=213334, pins_verified=17, complete_modules_verified=10, historical_receipt=ident(Path('docs/verification/m1-artifact-data-observed-2026-10-25-t3-52422617/materializer-receipt.json').read_bytes()),
     actual_root_layout='UNVERIFIED', name_approval_is_body_authentication=False))

# Compare actual current COPY R3 bodies only for the requested prepare interface.
copy_paths = [p for p in tree('6ef1663') if p.endswith('/copy-parent.py.txt') and 'signal' in p]
assert len(copy_paths) == 1
copy_path = copy_paths[0]
copy_bytes = git('show', '6ef1663:' + copy_path)
copy_original_path = 'docs/verification/m1-copy-parent-capsule-2026-10-25-t3-f65c3d2f/copy-parent.py.txt'
_, copy_current, _ = declarations(copy_bytes)
_, copy_original, _ = declarations(Path(copy_original_path).read_bytes())
copy_bindings = {}
for name in ['inspect_new_prepared', 'collection_complete']:
    assert copy_current[name]['raw'] == copy_original[name]['raw']
    assert b'kill_delivered' not in copy_current[name]['raw'] and b'kill_syscall_returned' not in copy_current[name]['raw']
    copy_bindings[name] = {k: v for k, v in copy_current[name].items() if k != 'raw'}
copy_excerpt = b'\n'.join(b'### ' + n.encode() + b'\n' + copy_current[n]['raw'] for n in ['inspect_new_prepared', 'collection_complete'])
assert copy_excerpt == packet_files['copy-integration-source-excerpts.txt']

# Rehash public stdlib text and compare every retained evidence excerpt.
libraries = []
for name in ['public-stdlib-source-evidence.json', 'public-loader-source-evidence.json']:
    record = jsons[name]
    b = Path(record['path']).read_bytes()
    assert ident(b) == {k: record[k] for k in ['bytes', 'sha256']}
    lines = b.decode().splitlines(keepends=True)
    for r in record['ranges']:
        assert ''.join(lines[r['start'] - 1:r['end']]) == r['text']
    libraries.append(record)

# Template schema traversal is DATA only, without invoking a supplied validator.
def schema_data(v, s):
    if 'anyOf' in s:
        assert any(schema_possible(v, child) for child in s['anyOf'])
    if 'const' in s:
        assert type(v) is type(s['const']) and v == s['const']
    if 'enum' in s:
        assert any(type(v) is type(x) and v == x for x in s['enum'])
    if 'type' in s:
        allowed = s['type'] if type(s['type']) is list else [s['type']]
        types = dict(null=type(None), object=dict, array=list, string=str, integer=int, boolean=bool)
        assert type(v) in [types[t] for t in allowed]
    if type(v) is dict:
        assert set(s.get('required', [])) <= set(v)
        if s.get('additionalProperties') is False:
            assert set(v) <= set(s.get('properties', {}))
        for k, item in v.items():
            if k in s.get('properties', {}):
                schema_data(item, s['properties'][k])
    if type(v) is str and 'pattern' in s:
        assert re.fullmatch(s['pattern'], v)
    if type(v) is int:
        assert s.get('minimum', v) <= v <= s.get('maximum', v)


def schema_possible(v, s):
    try:
        schema_data(v, s)
        return True
    except AssertionError:
        return False


template = jsons['config.template.json']
schema_data(template, jsons['config.schema.json'])
assert template == strict(g['CONFIG_TEXT']) and template['execution'] is False
assert all(x is None for x in template['parent'].values())
assert all(x is None for r in template['reviews'].values() for x in r.values())
assert all(x is None for k in ['completed_SOURCE', 'artifact_publication'] for x in template[k].values())
assert template['template_sha256'] is None
for name in ['config.schema.json', 'config.template.json']:
    assert packet_files[name] == git('show', BASIS + ':' + str(Path(BASIS_PATH).parent / name))
config_nodes = [n for n in parsed.body if isinstance(n, ast.Assign) and any(isinstance(t, ast.Name) and t.id == 'CONFIG_TEXT' for t in n.targets)]
assert len(config_nodes) == 1 and isinstance(config_nodes[0].value, ast.Constant)
cn = config_nodes[0].value
lines = source.splitlines(keepends=True)
start = sum(map(len, lines[:cn.lineno - 1])) + cn.col_offset
end = sum(map(len, lines[:cn.end_lineno - 1])) + cn.end_col_offset
assert source == source[:start] + source[start:end] + source[end:]
proof = jsons['template-proof.json']
assert ident(source[:start]) == proof['prefix'] and ident(source[end:]) == proof['suffix']
assert ident(source[start:end]) == {k: proof['token'][k] for k in ['bytes', 'sha256']}
assert (start, end) == (1288, 3164)
assert sha(canon(template)) == proof['canonical_CONFIG_sha256']
normalized = copy.deepcopy(parsed)
next(n for n in normalized.body if isinstance(n, ast.Assign) and any(isinstance(t, ast.Name) and t.id == 'CONFIG_TEXT' for t in n.targets)).value.value = '<CONFIG_TEXT_TOKEN>'
assert sha(ast.dump(normalized, annotate_fields=True, include_attributes=False).encode()) == proof['normalized_AST']['sha256']
child_schema = jsons['child-status.schema.json']
for key in ['actual_exit', 'kill_eligibility_exit']:
    assert child_schema['properties'][key] == {'type': ['integer', 'null']}
status_node = next(n for n in ast.walk(parsed) if isinstance(n, ast.Assign) and any(isinstance(t, ast.Name) and t.id == 'status' for t in n.targets) and isinstance(n.value, ast.Call) and isinstance(n.value.func, ast.Name) and n.value.func.id == 'dict')
status_keys = [k.arg for k in status_node.value.keywords]
assert set(status_keys) == set(child_schema['required'])
assert set(child_schema['properties']) == set(status_keys) | {'retained_bytes'}
assert 'kill_delivered' not in child_schema['properties'] and child_schema['additionalProperties'] is False
assert all(s['actual_invocations'] == 0 and s['execution'] is False for s in jsons['finite-plan.json']['steps'])
save('interface-and-template.json', dict(execution=False, copy_source_path=copy_path, copy_source=ident(copy_bytes),
     copy_functions=copy_bindings, copy_scope='prepare status source compatibility only; no whole COPY verdict',
     public_library_evidence=libraries, config_token_start=start, config_token_end=end,
     config_token=ident(source[start:end]), prefix=ident(source[:start]), suffix=ident(source[end:]),
     canonical_CONFIG_sha256=sha(canon(template)), normalized_AST_sha256=proof['normalized_AST']['sha256'],
     all_future_slots_null=True, false_plan=True, signed_actual_and_eligibility_exit_domains=True,
     schema_constructor_keys_equal=True, actual_future_render=None))

observations = dict(execution=False, verdict='STATIC_IDENTITIES_AND_STRUCTURE_MATCH', supplied_checker_executed=False,
    source_declarations=45, unchanged_declarations=41, changed_declarations=3, added_declarations=1,
    public_bindings=222, candidate_packet_files=40, packet_manifest_payload=38, packet_checksum_entries=39,
    author_parent_inventory_entries=4422, seven_hunk_reconstruction_exact=True, whole_source_restoration_exact=True,
    global_segments_before=40, global_segments_after=41, pins=17, modules=10, initial_names=260, before_entry_names=264,
    author_checks={'source':430, 'closure':35, 'independently_executed':False},
    notes=['No supplied source/checker is evaluated, imported or run.',
           'These identity comparisons are not runtime tests or acceptance.',
           'All public packet bytes were read, including author checker and error records as text/data.',
           'Original token restoration is the existing token only; no populated future render was synthesized.'])
save('static-observations.json', observations)
print(json.dumps(observations, sort_keys=True, indent=2))
