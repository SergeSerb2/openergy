"""Reviewer-authored public text/JSON/hash/AST-DATA reader. No supplied code runs.

Run only with public /usr/bin/python3 -I -S -B. This reader never imports a
candidate module, evaluates a source expression, creates executable code, calls
a supplied checker, opens private paths, or performs runtime verification.
"""
import ast
import hashlib
import json
import math
import os
from pathlib import Path
import re
import stat
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]
OUT = Path(__file__).resolve().parent
BASE = 'cd4fae4155e50e99d033fc78553ad50bbfa7d164'
REPAIR_COMMIT = '88e4444b8e5604fe9bfde7e768ea31d138ab64be'
REPAIR_IMPORT = '161ffce69c9dd588ef44b348c9fa0efe8ac8f0b2'
OLD_COMMIT = '06634b58f1b4795684b9139de716b4fe8f6430e0'
OLD_IMPORT = '9379e2b50b566e3346ec934b322ef142db2ade39'
R2_COMMIT = 'f6e15d0803d52905636f9b916b50e0c0a23ef05a'
R2_IMPORT = '078ced42f232509118e038cd3abbc80225e80929'
NEW = 'docs/verification/m1-prepare-parent-admission-cache-repair-2026-10-25-t3-53a3c637'
OLD = 'docs/verification/m1-prepare-parent-capsule-2026-10-25-t3-764cae00'
R2 = 'docs/verification/m1-prepare-parent-full-review-2026-10-25-t3-1b3fcbf5'
ENV = dict(os.environ, GIT_OPTIONAL_LOCKS='0')


def sha(data):
    return hashlib.sha256(data).hexdigest()


def canonical(data):
    return json.dumps(data, sort_keys=True, separators=(',', ':'), ensure_ascii=True).encode()


def write(name, data):
    (OUT / name).write_text(json.dumps(data, sort_keys=True, indent=2) + '\n')


def git(*args):
    return subprocess.check_output(['git', *args], cwd=str(ROOT), env=ENV)


def unique(items):
    result = {}
    for key, value in items:
        assert key not in result, ('duplicate JSON key', key)
        result[key] = value
    return result


def bad_number(value):
    raise ValueError('nonfinite JSON constant: ' + value)


def finite_float(value):
    result = float(value)
    assert math.isfinite(result)
    return result


def load(path):
    return json.loads((ROOT / path).read_bytes(), object_pairs_hook=unique,
                      parse_constant=bad_number, parse_float=finite_float)


def identity(data):
    return dict(bytes=len(data), sha256=sha(data))


def tree(commit, path=None):
    args = ['ls-tree', '-r', '-z', commit]
    if path:
        args.extend(['--', path])
    rows = []
    for raw in git(*args).split(b'\0'):
        if raw:
            meta, name = raw.split(b'\t', 1)
            mode, kind, blob = meta.decode().split()
            assert kind == 'blob', (kind, name)
            rows.append(dict(path=name.decode(), mode=mode, blob=blob))
    return rows


def blobs(rows):
    # One public Git batch; no filters, hooks, checkout, or supplied executable.
    names = list(dict.fromkeys(row['blob'] for row in rows))
    p = subprocess.run(['git', 'cat-file', '--batch'], cwd=str(ROOT), env=ENV,
                       input=('\n'.join(names) + '\n').encode(), stdout=subprocess.PIPE, check=True)
    data, pos, result = p.stdout, 0, {}
    for name in names:
        end = data.index(b'\n', pos)
        actual, kind, size = data[pos:end].decode().split()
        assert actual == name and kind == 'blob'
        pos, size = end + 1, int(size)
        result[name] = data[pos:pos + size]
        pos += size
        assert data[pos:pos + 1] == b'\n'
        pos += 1
    assert pos == len(data)
    return result


def baseline_inventory():
    rows = tree(BASE)
    raw = blobs(rows)
    conversions = []
    for row in rows:
        p = ROOT / row['path']
        info = p.lstat()
        working = os.readlink(p).encode() if stat.S_ISLNK(info.st_mode) else p.read_bytes()
        original = raw[row['blob']]
        row.update(git_bytes=len(original), git_sha256=sha(original),
                   filesystem_mode=oct(stat.S_IMODE(info.st_mode)),
                   working_bytes=len(working), working_sha256=sha(working))
        if original != working:
            assert row['path'].endswith('.cmd') and working == original.replace(b'\n', b'\r\n')
            conversions.append(dict(row))
    return dict(base=BASE, tree=git('rev-parse', BASE + '^{tree}').decode().strip(),
                tracked_files=len(rows), canonical_inventory_sha256=sha(canonical(rows)),
                canonical_inventory_bytes=len(canonical(rows)),
                git_bytes=sum(r['git_bytes'] for r in rows),
                working_bytes=sum(r['working_bytes'] for r in rows),
                existing_checkout_conversions=conversions,
                mode_counts={mode: sum(r['mode'] == mode for r in rows) for mode in sorted({r['mode'] for r in rows})},
                COPY_scope='hash preservation only; no semantic COPY review')


def parse_data(raw):
    # PyCF_ONLY_AST through ast.parse: syntax tree DATA only, never a code object.
    return ast.parse(raw.decode(), filename='<public-inert-text>', mode='exec', feature_version=(3, 9))


def literal(node):
    if isinstance(node, ast.Constant):
        assert type(node.value) in (str, bytes, int, float, bool, type(None))
        return node.value
    if isinstance(node, (ast.List, ast.Tuple, ast.Set)):
        values = [literal(n) for n in node.elts]
        return tuple(values) if isinstance(node, ast.Tuple) else set(values) if isinstance(node, ast.Set) else values
    if isinstance(node, ast.Dict):
        keys = [literal(n) for n in node.keys]
        assert len(keys) == len(set(keys))
        return dict(zip(keys, (literal(n) for n in node.values)))
    if isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.USub):
        return -literal(node.operand)
    raise ValueError('nonliteral AST DATA: ' + type(node).__name__)


def assignments(parsed):
    result = {}
    for node in parsed.body:
        if isinstance(node, ast.Assign) and len(node.targets) == 1 and isinstance(node.targets[0], ast.Name):
            try:
                result[node.targets[0].id] = literal(node.value)
            except ValueError:
                pass
    return result


def declarations(parsed, raw):
    lines = raw.splitlines(keepends=True)
    result = {}
    def walk(node, prefix=''):
        for child in ast.iter_child_nodes(node):
            if isinstance(child, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
                key = prefix + child.name
                start = min([child.lineno] + [d.lineno for d in child.decorator_list])
                end = child.end_lineno
                result[key] = dict(start_line=start, end_line=end, kind=type(child).__name__,
                                   **identity(b''.join(lines[start - 1:end])))
                walk(child, key + '.')
            else:
                walk(child, prefix)
    walk(parsed)
    return result


def global_segments(parsed, raw):
    lines = raw.splitlines(keepends=True)
    decls = [n for n in parsed.body if isinstance(n, (ast.FunctionDef, ast.ClassDef, ast.AsyncFunctionDef))]
    result, start = [], 1
    for node in decls:
        end = min([node.lineno] + [d.lineno for d in node.decorator_list]) - 1
        result.append(dict(start_line=start, end_line=end, **identity(b''.join(lines[start - 1:end]))))
        start = node.end_lineno + 1
    result.append(dict(start_line=start, end_line=len(lines), **identity(b''.join(lines[start - 1:]))))
    return result


def reconstruct(before, patch):
    old = before.splitlines(keepends=True)
    lines = patch.splitlines(keepends=True)
    headers = [i for i, row in enumerate(lines) if row.startswith(b'@@ ')]
    output, at, evidence = [], 0, []
    for number, i in enumerate(headers):
        match = re.fullmatch(rb'@@ -(\d+),(\d+) \+(\d+),(\d+) @@\n', lines[i])
        assert match, lines[i]
        os_, oc, ns, nc = map(int, match.groups())
        body = lines[i + 1:headers[number + 1] if number + 1 < len(headers) else len(lines)]
        assert all(row[:1] in (b' ', b'-', b'+') for row in body)
        left = [row[1:] for row in body if row[:1] in (b' ', b'-')]
        right = [row[1:] for row in body if row[:1] in (b' ', b'+')]
        assert len(left) == oc and len(right) == nc and old[os_ - 1:os_ - 1 + oc] == left
        assert os_ - 1 >= at
        output.extend(old[at:os_ - 1])
        assert len(output) == ns - 1
        output.extend(right)
        at = os_ - 1 + oc
        evidence.append(dict(old_start=os_, old_count=oc, new_start=ns, new_count=nc,
                             old_sha256=sha(b''.join(left)), new_sha256=sha(b''.join(right))))
    output.extend(old[at:])
    return b''.join(output), evidence


def inspect():
    assert git('rev-parse', 'HEAD').decode().strip() == BASE
    assert not git('status', '--porcelain=v1', '--untracked-files=no')
    opening = baseline_inventory()
    if (OUT / 'preservation-opening.json').exists():
        assert opening == load(str((OUT / 'preservation-opening.json').relative_to(ROOT)))
    write('preservation-opening.json', opening)

    packet = []
    for prefix, commits in ((NEW, (REPAIR_COMMIT, REPAIR_IMPORT, BASE)),
                            (OLD, (OLD_COMMIT, OLD_IMPORT, BASE)),
                            (R2, (R2_COMMIT, R2_IMPORT, BASE))):
        sets = [tree(c, prefix) for c in commits]
        assert all(rows == sets[0] for rows in sets)
        data = blobs(sets[0])
        for row in sets[0]:
            assert (ROOT / row['path']).read_bytes() == data[row['blob']]
            assert row['mode'] == '100644'
            packet.append(dict(row, commits=list(commits), **identity(data[row['blob']])))
    assert git('rev-parse', REPAIR_COMMIT + '^').decode().strip() == R2_IMPORT
    assert git('rev-parse', REPAIR_IMPORT + '^').decode().strip() == 'd3c58c507018d1fda32ccd49e359a6973c240a4e'
    assert sha((ROOT / R2 / 'REVIEW.md').read_bytes()) == 'e50e67ef33aa690525f821fb3d77bf7601a1138740e9ee7c4910b1cfc1129981'

    manifest = load(NEW + '/manifest.json')
    assert set(manifest['files']) == {p.name for p in (ROOT / NEW).iterdir()} - {'manifest.json', 'SHA256SUMS'}
    for name, row in manifest['files'].items():
        data = (ROOT / NEW / name).read_bytes()
        assert identity(data) == {k: row[k] for k in ('bytes', 'sha256')}
        assert hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest() == row['Git_blob']
        assert row['mode'] == '100644'
    sums = (ROOT / NEW / 'SHA256SUMS').read_text().splitlines()
    names = []
    for line in sums:
        digest, name = line.split('  ', 1)
        assert sha((ROOT / NEW / name).read_bytes()) == digest
        names.append(name)
    assert len(names) == len(set(names)) and set(names) == set(manifest['files']) | {'manifest.json'}

    # Strictly parse all packet JSON as DATA, including the large baseline.
    parsed_json = {p.name: load(str(p.relative_to(ROOT))) for p in (ROOT / NEW).glob('*.json')}
    author_old = parsed_json['baseline-tracked-inventory.json']
    expected_rows = tree(R2_IMPORT)
    assert len(expected_rows) == len(author_old['entries']) == 4298
    actual_old = {r['path']: r for r in author_old['entries']}
    old_blobs = blobs(expected_rows)
    for row in expected_rows:
        claim = actual_old[row['path']]
        assert all(claim[k] == row[k] for k in ('path', 'blob', 'mode'))
        assert identity(old_blobs[row['blob']]) == {k: claim['git_' + k] for k in ('bytes', 'sha256')}
        data = (ROOT / row['path']).read_bytes()
        assert identity(data) == {k: claim['working_' + k] for k in ('bytes', 'sha256')}
        assert oct(stat.S_IMODE((ROOT / row['path']).lstat().st_mode)) == claim['filesystem_mode']

    inputs = parsed_json['input-bindings.json']['public_Git_bindings']
    assert len(inputs) == 95
    verified_inputs = []
    for row in inputs:
        bound = tree(row['commit'], row['path'])
        assert len(bound) == 1 and all(bound[0][k] == row[k] for k in ('path', 'blob', 'mode'))
        data = git('cat-file', 'blob', row['blob'])
        assert identity(data) == {k: row[k] for k in ('bytes', 'sha256')}
        assert data == (ROOT / row['path']).read_bytes()
        verified_inputs.append({k: row[k] for k in ('commit', 'path', 'blob', 'mode', 'bytes', 'sha256', 'role')})

    before = (ROOT / OLD / 'prepare-parent.py.txt').read_bytes()
    after = (ROOT / NEW / 'prepare-parent.py.txt').read_bytes()
    patch = (ROOT / NEW / 'from-06634b5.patch').read_bytes()
    rebuilt, hunks = reconstruct(before, patch)
    assert rebuilt == after and len(hunks) == 4
    bp, ap = parse_data(before), parse_data(after)
    bd, ad = declarations(bp, before), declarations(ap, after)
    assert set(bd) == set(ad)
    functions = {}
    changed = []
    for name in sorted(bd):
        status_ = 'unchanged' if bd[name]['sha256'] == ad[name]['sha256'] else 'changed'
        functions[name] = dict(before=bd[name], after=ad[name], status=status_)
        if status_ == 'changed':
            changed.append(name)
    assert changed == ['collect', 'import_common', 'main']
    assert functions == parsed_json['function-inventory.json']['functions']
    bg, ag = global_segments(bp, before), global_segments(ap, after)
    assert len(bg) == len(ag) == 40
    segments = []
    for old, new in zip(bg, ag):
        assert old['sha256'] == new['sha256'] and old['bytes'] == new['bytes']
        segments.append(dict(before=old, after=new, unchanged=True))
    assert segments == parsed_json['module-layout.json']['segments']
    ba, aa = assignments(bp), assignments(ap)
    assert ba == aa
    for name in ('config.schema.json', 'config.template.json'):
        assert (ROOT / NEW / name).read_bytes() == (ROOT / OLD / name).read_bytes()
    old_pin_data = load(OLD + '/pin-bindings.json')
    new_pin_data = parsed_json['pin-bindings.json']
    for key in ('completed7', 'fixed8', 'fixed_source_and_review_bindings', 'frozen17',
                'initial20', 'pending_review_inputs', 'source_result', 'source_raw_bytes',
                'source_raw_records', 'source_raw_manifest_sha256'):
        assert old_pin_data[key] == new_pin_data[key]

    # Token and closed false/null DATA. No populated/synthetic render is made.
    config = parsed_json['config.template.json']
    assert json.loads(aa['CONFIG_TEXT'], object_pairs_hook=unique, parse_constant=bad_number) == config
    assert config['execution'] is False and config['template_sha256'] is None
    assert all(v is None for v in config['parent'].values())
    assert all(v is None for row in config['reviews'].values() for v in row.values())
    assert all(v is None for k in ('completed_SOURCE', 'artifact_publication') for v in config[k].values())
    assert all(row['execution'] is False and row['actual_invocations'] == 0 for row in parsed_json['finite-plan.json']['steps'])
    config_assign = [n for n in ap.body if isinstance(n, ast.Assign) and any(isinstance(t, ast.Name) and t.id == 'CONFIG_TEXT' for t in n.targets)]
    assert len(config_assign) == 1 and isinstance(config_assign[0].value, ast.Constant)
    n = config_assign[0].value
    lines = after.splitlines(keepends=True)
    start = sum(map(len, lines[:n.lineno - 1])) + n.col_offset
    end = sum(map(len, lines[:n.end_lineno - 1])) + n.end_col_offset
    token = after[start:end]
    proof = dict(token_start_byte=start, token_end_byte=end, token_sha256=sha(token),
                 prefix_sha256=sha(after[:start]), suffix_sha256=sha(after[end:]),
                 canonical_config_sha256=sha(canonical(config)), literal_equals_template=True,
                 original_token_unchanged=token == before[start:end],
                 source_restores_byte_exact=after[:start] + token + after[end:] == after,
                 template_bytes=len(after), template_le_1MiB=len(after) <= 1048576,
                 actual_future_render=None, actual_render_not_created=True,
                 AST_mode='data only; no code object/eval/supplied import')
    author = parsed_json['template-proof.json']
    for own, theirs in (('token_start_byte', 'config_token_start_byte'), ('token_end_byte', 'config_token_end_byte'),
                        ('token_sha256', 'config_token_sha256'), ('prefix_sha256', 'prefix_sha256'),
                        ('suffix_sha256', 'suffix_sha256'), ('canonical_config_sha256', 'canonical_config_sha256')):
        assert proof[own] == author[theirs]

    write('bindings.json', dict(schema='openergy.independent-prepare-admission-cache-bindings.v1', execution=False,
          reviewer_base=BASE, repair_candidate=REPAIR_COMMIT, repair_import=REPAIR_IMPORT,
          repair_exact_parent=R2_IMPORT, source=dict(path=NEW + '/prepare-parent.py.txt', lines=len(lines), **identity(after)),
          baseline=dict(path=OLD + '/prepare-parent.py.txt', candidate=OLD_COMMIT, import_commit=OLD_IMPORT, **identity(before)),
          full_R2=dict(candidate=R2_COMMIT, import_commit=R2_IMPORT, path=R2 + '/REVIEW.md', **identity((ROOT / R2 / 'REVIEW.md').read_bytes())),
          packet_files=packet, public_input_bindings=verified_inputs, patch=dict(**identity(patch), hunks=hunks, reconstructs_whole_file=True),
          manifest_verified=True, SHA256SUMS_entries=len(sums), author_baseline_entries_verified=4298,
          author_baseline_not_dumped=True, template_data_proof=proof))
    write('function-bindings.json', dict(schema='openergy.independent-prepare-admission-cache-functions.v1', execution=False,
          functions=functions, changed=changed, unchanged_declarations=41, unchanged_module_segments=segments,
          all_literal_globals_unchanged=True, added=[], removed=[]))
    print(json.dumps(dict(source=identity(after), functions=len(functions), changed=changed,
          unchanged_segments=len(segments), public_inputs=len(verified_inputs), packet_files=len(packet),
          checksum_entries=len(sums), opening_preservation=opening), sort_keys=True))


def close():
    closing = baseline_inventory()
    opening = load(str((OUT / 'preservation-opening.json').relative_to(ROOT)))
    assert closing == opening
    write('preservation-closing.json', dict(closing, opening_equals_closing=True,
          tracked_working_bytes_and_modes_preserved=True,
          staged_or_untracked_scope='review folder only; commit scope checked separately'))
    print(json.dumps(dict(preserved=True, tracked_files=closing['tracked_files'],
                         inventory_sha256=closing['canonical_inventory_sha256'])))


def schema_data(value, rule):
    if 'anyOf' in rule:
        assert any(schema_ok(value, option) for option in rule['anyOf'])
    if 'const' in rule:
        assert type(value) is type(rule['const']) and value == rule['const']
    if 'enum' in rule:
        assert any(type(value) is type(v) and value == v for v in rule['enum'])
    if 'type' in rule:
        assert type(value) is {'object': dict, 'array': list, 'string': str,
                               'integer': int, 'null': type(None), 'boolean': bool}[rule['type']]
    if isinstance(value, dict) and 'properties' in rule:
        assert set(rule.get('required', ())) <= set(value)
        if rule.get('additionalProperties') is False:
            assert set(value) <= set(rule['properties'])
        for key, item in value.items():
            schema_data(item, rule['properties'][key])
    if 'pattern' in rule:
        assert re.search(rule['pattern'], value)
    if 'minimum' in rule:
        assert value >= rule['minimum']
    if 'maximum' in rule:
        assert value <= rule['maximum']


def schema_ok(value, rule):
    try:
        schema_data(value, rule)
    except AssertionError:
        return False
    return True


def details():
    raw = (ROOT / NEW / 'prepare-parent.py.txt').read_bytes()
    assert sha(raw) == 'fce82ca2775607fd86e0ef8dbb8c44fd3cd5a213815ae77cb9501327160ee57d'
    parsed = parse_data(raw)
    values = assignments(parsed)
    bindings = load(str((OUT / 'bindings.json').relative_to(ROOT)))
    inputs = bindings['public_input_bindings']
    pin_matches = {}
    for name, value in values['PINS'].items():
        matches = [r for r in inputs if all(r[k] == value[k] for k in ('bytes', 'sha256'))]
        assert matches, name
        pin_matches[name] = dict(value, public_paths=sorted(set(r['path'] for r in matches)))
    assert len(pin_matches) == 17
    config = load(NEW + '/config.template.json')
    schema_data(config, load(NEW + '/config.schema.json'))
    assert set(config['parent']) == set(values['PARENT_DUTIES'])
    assert len(config['parent']) == 17
    materialization_dir = 'docs/verification/ordinary-combined-context-tuple-repair-2026-10-25'
    materialization = load(materialization_dir + '/materialization.json')
    checks = load(materialization_dir + '/source-checks.json')['python_files']
    modules = []
    for module in materialization['modules']:
        path = str((Path(materialization_dir) / module['source']))
        body = (ROOT / path).read_bytes()
        name = module['destination']
        assert sha(body) == module['sha256'] == checks[name]['sha256'] == values['PINS'][name]['sha256']
        assert len(body) == checks[name]['bytes'] == values['PINS'][name]['bytes']
        modules.append(dict(destination=name, public_path=path, **identity(body)))
    assert len(modules) == len(checks) == 10
    source_record = load('docs/verification/m1-source-metadata-observed-2026-10-25-t3-52422617/validation.json')
    assert values['SOURCE_RAWS'] == source_record['record_files']
    assert len(values['SOURCE_RAWS']) == 212
    assert sum(row['bytes'] for row in values['SOURCE_RAWS']) == 213334
    raw_manifest_hash = sha(canonical(values['SOURCE_RAWS']) + b'\n')
    assert raw_manifest_hash == '2baea069155b633485b1e8aed8fe72f3bf746f67de83a84f07a2daf0648fe4ee'
    data_dir = 'docs/verification/m1-artifact-data-observed-2026-10-25-t3-52422617'
    receipt = load(data_dir + '/materializer-receipt.json')
    validation = load(data_dir + '/validation.json')
    data_bindings = load(data_dir + '/bindings.json')
    for name, value in values['EIGHT'].items():
        expected = dict(path=values['ROOT'] + '/' + name, mode='0o600', **value)
        assert receipt['files'][name] == validation['data_files'][name] == expected
        refs = [r for r in data_bindings['private_files'] if r['private_relative_path'] == 'artifact-binding/retained/' + name]
        assert len(refs) == 1 and all(refs[0][k] == value[k] for k in ('bytes', 'sha256'))
    assert len(values['EIGHT']) == 8 and sum(v['bytes'] for v in values['EIGHT'].values()) == 136360
    own_functions = load(str((OUT / 'function-bindings.json').relative_to(ROOT)))['functions']
    for phase, prefix in (('before', OLD), ('after', NEW)):
        body = (ROOT / prefix / 'prepare-parent.py.txt').read_bytes().splitlines(keepends=True)
        transcript = (ROOT / NEW / ('changed-functions.' + phase + '.txt')).read_bytes()
        for name in ('import_common', 'collect', 'main'):
            record = own_functions[name][phase]
            assert b''.join(body[record['start_line'] - 1:record['end_line']]) in transcript
    observer_path = 'docs/verification/m1-prepare-local-exit-observer-review-2026-10-25-t3-c2d0d953/REVIEW.md'
    assert sha((ROOT / observer_path).read_bytes()) == 'bb8ed39275540eabc116cca932a40d42246c939c7245f8c04eaba03c87c97ed1'
    observer_source = 'docs/verification/m1-prepare-local-exit-observer-2026-10-25-t3-52422617/observer.py.txt'
    assert sha((ROOT / observer_source).read_bytes()) == '845d90024ad47f6b76d6d6e646b679d00bb97b8ca0478d0fa86523de3dbec60f'
    library = Path('/Applications/Xcode.app/Contents/Developer/Library/Frameworks/Python3.framework/Versions/3.9/lib/python3.9/importlib/_bootstrap_external.py')
    library_raw = library.read_bytes()
    assert identity(library_raw) == dict(bytes=64569, sha256='ae0b9f573429b63e40942e8d3365c73a7ed37302fff6dfc7eebbd2bb62b6bd85')
    ranges = [(348, 435), (696, 748), (837, 850), (916, 998), (1374, 1428), (1500, 1554), (1634, 1642)]
    ll = library_raw.splitlines(keepends=True)
    library_data = dict(path=str(library), **identity(library_raw), text_only=True, actual_M1_runtime_qualification=False,
                        ranges=[dict(start_line=a, end_line=b, **identity(b''.join(ll[a - 1:b]))) for a, b in ranges])
    # Keep a separately defined reproducible AST-DATA digest; no author normalization convention is inferred.
    for node in parsed.body:
        if isinstance(node, ast.Assign) and any(isinstance(t, ast.Name) and t.id == 'CONFIG_TEXT' for t in node.targets):
            node.value.value = '<REVIEWER_CONFIG_TEXT_DATA>'
    normalized = ast.dump(parsed, annotate_fields=True, include_attributes=False).encode()
    write('static-observations.json', dict(schema='openergy.independent-prepare-admission-cache-observations.v1', execution=False,
          method='Reviewer public raw bytes/strict JSON/parse-only AST DATA; no eval, supplied import, executable code object or target call',
          frozen17=pin_matches, modules=modules, source_raw_manifest=dict(entries=212, bytes=213334, sha256=raw_manifest_hash),
          config=dict(schema_closed_template_traversal=True, execution=False, unresolved_slots='all null', parent_duties=values['PARENT_DUTIES'],
                      null_template_cannot_pass_first_hex_gate=True, actual_future_render=None,
                      reviewer_normalized_AST_data_sha256=sha(normalized), normalization_string='<REVIEWER_CONFIG_TEXT_DATA>',
                      serialization='ast.dump annotate_fields=True include_attributes=False UTF-8 without LF',
                      author_normalized_digest='retained in exact author packet; unspecified marker/serialization not inferred'),
          public_DATA=dict(record_commit='4fbf8f0f32c08fcdfc1f02b3cb2a37b9b79a56ed', eight=values['EIGHT'], total_bytes=136360,
                           receipt=identity((ROOT / data_dir / 'materializer-receipt.json').read_bytes()),
                           guard=identity((ROOT / data_dir / 'materializer-guard.json').read_bytes()),
                           status=validation['status'], initial_tally=receipt['initial_tally'],
                           initial_entries_before_receipt=receipt['initial_entries_before_receipt'],
                           actual_ssh_exit=validation['actual_ssh_exit'], actual_collector_exit=validation['actual_collector_exit'],
                           both_EOF=validation['both_EOF'], D4_before_original_135s=validation['D4_before_original_135s'],
                           current_flat_ROOT_inventory_or_current_shadow_absence_proof=False, private_raw_reopens=0),
          observer=dict(review_path=observer_path, review=identity((ROOT / observer_path).read_bytes()),
                        source_path=observer_source, source=identity((ROOT / observer_source).read_bytes()),
                        dependency_status='bounded SOURCE CLEAR only; no capsule clearance or actual observation'),
          public_library=library_data,
          author_records=dict(source_checks=69, closure_checks=15, public_inputs=95,
              treatment='read as author DATA; no supplied checker run; counts are not module-selection or runtime proof',
              setup_error='Initial raw/working equality assertion rejected existing .cmd CRLF conversion; corrected before author writes',
              whitespace_error='Two trailing transcript separator lines removed after author staged whitespace check; source unchanged'),
          reviewer_reader_errors=[
              dict(error="KeyError: 'unchanged_segments'", cause="Wrong reviewer JSON member name; actual member is 'segments'", effect='only opening preservation record written; no supplied/target action'),
              dict(error='AssertionError', cause='Reviewer over-assumed whole pin-bindings.json byte equality; actual diff is repair/provenance metadata',
                   correction='Verify all original binding subtrees separately; exact diff inspected; no source pin changed', effect='no supplied/target action')],
          scopes=dict(private_state_reads=0, supplied_runs_imports_or_executable_compiles=0, network_remote_native=0, tests_mocks=0,
                      children=0, build_signing_PR_push=0, COPY_review=False, runtime_acceptance=False)))
    print(json.dumps(dict(frozen_pins=17, modules=10, source_rows=212, DATA_files=8,
                         closed_null_template=True, private_reads=0, public_library_sha256=sha(library_raw))))


if __name__ == '__main__':
    if len(sys.argv) == 2 and sys.argv[1] == 'close':
        close()
    elif len(sys.argv) == 2 and sys.argv[1] == 'details':
        details()
    else:
        assert len(sys.argv) == 1
        inspect()
