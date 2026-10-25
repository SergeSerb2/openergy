"""Independent public text/JSON/hash/AST-data reader. Never loads supplied code.

Only this reviewer-owned file executes. ast.parse/literal_eval inspect data;
no supplied import, eval, exec, executable compile, validator, test or child runs.
Git invocations below read public objects only. Outputs stay in this folder.
"""
import ast
import copy
import hashlib
import io
import json
from pathlib import Path
import re
import stat
import subprocess
import tokenize

OUT = Path(__file__).resolve().parent.relative_to(Path.cwd())
NEW = OUT.parent / 'm1-copy-parent-signal-schema-repair-2026-10-25-t3-f3ffc1d7'
OLD = OUT.parent / 'm1-copy-parent-capsule-2026-10-25-t3-f65c3d2f'
PRIOR = OUT.parent / 'm1-copy-parent-full-review-2026-10-25-t3-9edf92e5'
CANDIDATE = 'c82cc12b8a2d709d8ce1e384545475159b24d2a9'
IMPORT = '6ef1663caee7ec8e421daaaf9effca6796ecedf0'
BASIS = '85c668f7e63c07e1f9d3cf9e76af05f21898f80d'
FULL_REVIEW = 'f33a1fd761770cb2a2961f9cf55b59dc4aeb40d5'
observations = []


def git(*args):
    return subprocess.check_output(['git', *args])


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def identity(raw):
    return {'bytes': len(raw), 'sha256': sha(raw)}


def check(name, condition, detail=None):
    observations.append({'name': name, 'equal_or_satisfied': bool(condition), 'detail': detail})
    if not condition:
        raise ValueError(name)


def write(name, obj):
    (OUT / name).write_text(json.dumps(obj, indent=2, ensure_ascii=True) + '\n')


def pairs(items):
    result = {}
    for key, value in items:
        if key in result:
            raise ValueError('Duplicate JSON key: ' + key)
        result[key] = value
    return result


def reject_constant(value):
    raise ValueError('Nonfinite JSON constant: ' + value)


def read_json(path):
    return json.loads(path.read_bytes(), object_pairs_hook=pairs, parse_constant=reject_constant)


def declarations(raw):
    tree = ast.parse(raw.decode(), feature_version=(3, 9))
    lines = raw.splitlines(keepends=True)
    rows = []

    def visit(node, prefix=''):
        for child in ast.iter_child_nodes(node):
            if isinstance(child, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
                name = prefix + child.name
                start = min([child.lineno] + [d.lineno for d in child.decorator_list])
                body = b''.join(lines[start - 1:child.end_lineno])
                rows.append({'name': name, 'kind': type(child).__name__, 'start': start,
                             'end': child.end_lineno, **identity(body), 'raw': body})
                visit(child, name + '.')
            else:
                visit(child, prefix)
    visit(tree)
    return tree, rows


def segments(raw, tree):
    lines = raw.splitlines(keepends=True)
    cursor = 1
    result = []
    for node in tree.body:
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            start = min([node.lineno] + [d.lineno for d in node.decorator_list])
            body = b''.join(lines[cursor - 1:start - 1])
            result.append({'start': cursor, 'end': start - 1, **identity(body), 'raw': body})
            cursor = node.end_lineno + 1
    body = b''.join(lines[cursor - 1:])
    result.append({'start': cursor, 'end': len(lines), **identity(body), 'raw': body})
    return result


def literals(tree):
    result = {}
    for node in tree.body:
        if isinstance(node, ast.Assign) and len(node.targets) == 1 and isinstance(node.targets[0], ast.Name):
            try:
                result[node.targets[0].id] = ast.literal_eval(node.value)
            except (ValueError, TypeError):
                pass
    return result


def patch_sections(raw):
    lines = raw.splitlines(keepends=True)
    sections = []
    i = 0
    while i < len(lines):
        check('patch header ' + str(i), lines[i].startswith(b'--- '))
        old_path = lines[i][4:].rstrip(b'\n').decode()
        new_path = lines[i + 1][4:].rstrip(b'\n').decode()
        old_path = old_path[2:] if old_path.startswith('a/') else old_path
        new_path = new_path[2:] if new_path.startswith('b/') else new_path
        check('patch new header ' + str(i), lines[i + 1].startswith(b'+++ '))
        i += 2
        hunks = []
        while i < len(lines) and not lines[i].startswith(b'--- '):
            match = re.match(rb'@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@', lines[i])
            check('patch hunk header ' + str(i), match is not None)
            old_start, old_count, new_start, new_count = [int(x) if x is not None else 1 for x in match.groups()]
            i += 1
            rows = []
            while i < len(lines) and not lines[i].startswith((b'@@ ', b'--- ')):
                check('patch row prefix ' + str(i), lines[i][:1] in (b' ', b'+', b'-'))
                rows.append(lines[i])
                i += 1
            check('patch old hunk count', sum(x[:1] in (b' ', b'-') for x in rows) == old_count)
            check('patch new hunk count', sum(x[:1] in (b' ', b'+') for x in rows) == new_count)
            hunks.append((old_start, old_count, new_start, new_count, rows))
        sections.append((old_path, new_path, hunks))
    return sections


def reconstruct(raw):
    result = []
    for old_path, new_path, hunks in patch_sections(raw):
        old = Path(old_path).read_bytes().splitlines(keepends=True)
        output = []
        cursor = 0
        for start, count, new_start, new_count, rows in hunks:
            check('ordered patch hunks', start - 1 >= cursor)
            output.extend(old[cursor:start - 1])
            check('patch new position', len(output) == new_start - 1)
            cursor = start - 1
            for row in rows:
                if row[:1] in (b' ', b'-'):
                    check('exact old/context patch row', cursor < len(old) and old[cursor] == row[1:])
                    cursor += 1
                if row[:1] in (b' ', b'+'):
                    output.append(row[1:])
        output.extend(old[cursor:])
        actual = Path(new_path).read_bytes()
        check('exact reconstructed patch bytes ' + new_path, b''.join(output) == actual)
        result.append({'before': old_path, 'after': new_path, 'hunks': len(hunks), **identity(actual)})
    return result


def bound_files(folder, revision):
    records = []
    for path in sorted(folder.iterdir()):
        check('flat packet file ' + str(path), path.is_file() and not path.is_symlink())
        raw = path.read_bytes()
        mode, kind, blob_path = git('ls-tree', revision, '--', str(path)).decode().split(' ', 2)
        blob, name = blob_path.rstrip('\n').split('\t')
        check('public tree bytes ' + str(path), git('show', revision + ':' + str(path)) == raw)
        records.append({'path': str(path), 'git_mode': mode, 'git_type': kind,
                        'blob': blob, 'working_mode': oct(stat.S_IMODE(path.stat().st_mode)), **identity(raw)})
        if path.suffix == '.json':
            read_json(path)
        elif path.suffix == '.py' or path.name.endswith('.py.txt'):
            ast.parse(raw.decode(), feature_version=(3, 9))
    return records


check('opening HEAD', git('rev-parse', 'HEAD').decode().strip() == IMPORT)
check('candidate exact parent', git('rev-parse', CANDIDATE + '^').decode().strip() == '7e5de706e7457de5f4880cdef180b27536dc0d5e')
new_files = bound_files(NEW, CANDIDATE)
check('53 candidate packet files', len(new_files) == 53)
check('candidate/import packet trees', git('ls-tree', '-r', CANDIDATE, '--', str(NEW)) == git('ls-tree', '-r', IMPORT, '--', str(NEW)))
changed = git('diff-tree', '--no-commit-id', '--name-status', '-r', CANDIDATE).decode().splitlines()
check('candidate adds packet only', len(changed) == 53 and all(x.startswith('A\t' + str(NEW) + '/') for x in changed))
old_files = bound_files(OLD, BASIS)
prior_files = bound_files(PRIOR, FULL_REVIEW)
check('baseline source hash', identity((OLD / 'copy-parent.py.txt').read_bytes()) == {'bytes': 120613, 'sha256': 'cfe3be7a67d22160b977a97941f7fa6ed44be322ab6822dd064c66bb93555b47'})
check('new source hash', identity((NEW / 'copy-parent.py.txt').read_bytes()) == {'bytes': 120718, 'sha256': '94efc9b4de615485827e471c3e3f31250b28ea64f35228955b91154e4ba2a22a'})
check('full prior report hash', identity((PRIOR / 'REVIEW.md').read_bytes()) == {'bytes': 21224, 'sha256': '33550d9e4fca61a4425eadfe095b8a8234a825a1978b1a263852e3dac365d53f'})
write('packet-bindings.json', {'candidate': CANDIDATE, 'import': IMPORT, 'new': new_files, 'basis': old_files, 'full_prior_review': prior_files})

for folder in (NEW, OLD, PRIOR):
    sums = (folder / 'SHA256SUMS').read_text().splitlines()
    for line in sums:
        digest, name = line.split('  ', 1)
        check('SHA256SUMS ' + str(folder / name), sha((folder / name).read_bytes()) == digest)
    if not (folder / 'packet-manifest.json').exists():
        check('baseline manifest supplied through full review', folder == OLD)
        continue
    manifest = read_json(folder / 'packet-manifest.json')
    for record in (manifest if isinstance(manifest, list) else manifest['files']):
        name = record['path']
        path = folder / name if '/' not in name else Path(name)
        raw = path.read_bytes()
        check('packet manifest ' + str(path), identity(raw) == {k: record[k] for k in ('bytes', 'sha256')})

patches = []
for name in ('from-85c668f-source.patch', 'from-85c668f-schema.patch', 'from-85c668f.patch'):
    raw = (NEW / name).read_bytes()
    patches.append({'path': name, **identity(raw), 'sections': reconstruct(raw)})
check('combined is exact concatenation', (NEW / 'from-85c668f.patch').read_bytes() == (NEW / 'from-85c668f-source.patch').read_bytes() + (NEW / 'from-85c668f-schema.patch').read_bytes())
check('source patch requested hash', patches[0]['sha256'] == '73442f657c57762239f7d2dbabea0f8a533d83593a258d609bc2bf5bf6f1a12d')
check('combined patch requested hash', patches[2]['sha256'] == 'f901f0faf35b0fe2541137c0c2979b34b589cc487423a698b55adf74c18fc059')
check('historical patch byte exact', (NEW / 'prior-from-cleared-source-launcher.patch').read_bytes() == (OLD / 'from-cleared-source-launcher.patch').read_bytes())
patches.append({'path': 'prior-from-cleared-source-launcher.patch', 'sections': reconstruct((NEW / 'prior-from-cleared-source-launcher.patch').read_bytes())})
write('patch-bindings.json', patches)

before = (OLD / 'copy-parent.py.txt').read_bytes()
after = (NEW / 'copy-parent.py.txt').read_bytes()
before_tree, before_funcs = declarations(before)
after_tree, after_funcs = declarations(after)
check('same declaration names and order', [x['name'] for x in before_funcs] == [x['name'] for x in after_funcs])
bindings = []
for a, b in zip(before_funcs, after_funcs):
    state = 'UNCHANGED' if a['raw'] == b['raw'] else 'CHANGED'
    bindings.append({'name': a['name'], 'state': state,
                     'before': {k: v for k, v in a.items() if k not in ('name', 'raw')},
                     'after': {k: v for k, v in b.items() if k not in ('name', 'raw')}})
check('only collect changes', [r['name'] for r in bindings if r['state'] != 'UNCHANGED'] == ['collect'])
check('48 functions and one class', len(bindings) == 49 and sum(r['after']['kind'] == 'ClassDef' for r in bindings) == 1)
check('author declaration inventory exact', bindings == read_json(NEW / 'function-inventory.json'))
prior_funcs = read_json(PRIOR / 'function-bindings.json')
write('function-bindings.json', {'declarations': bindings, 'prior_full_binding_data': prior_funcs})
before_segs = segments(before, before_tree)
after_segs = segments(after, after_tree)
check('global segment count', len(before_segs) == len(after_segs))
seg_bindings = []
for i, (a, b) in enumerate(zip(before_segs, after_segs)):
    check('unchanged complete global segment ' + str(i), a['raw'] == b['raw'])
    seg_bindings.append({'index': i, 'state': 'UNCHANGED', 'before': {k: v for k, v in a.items() if k != 'raw'}, 'after': {k: v for k, v in b.items() if k != 'raw'}})
check('author global segment inventory exact', seg_bindings == read_json(NEW / 'global-segment-inventory.json'))
write('global-bindings.json', seg_bindings)
before_literals, after_literals = literals(before_tree), literals(after_tree)
check('complete decoded literal globals exact', before_literals == after_literals)
write('literal-bindings.json', after_literals)

schema_before = read_json(OLD / 'prepared-copy-qualification.schema.json')
schema_after = read_json(NEW / 'prepared-copy-qualification.schema.json')
expected = copy.deepcopy(schema_before)
schema_sites = []


def transform(node, pointer=''):
    if isinstance(node, dict):
        props = node.get('properties', {})
        if 'kill_eligibility_exit' in props and 'kill_delivered' in props:
            check('closed parent schema definition', node.get('additionalProperties') is False and len(props) == 37)
            props['kill_syscall_returned'] = props.pop('kill_delivered')
            props['kill_syscall_returned']['description'] = 'True only after os.kill on the retained unreaped direct child returns without exception; not proof of delivery, termination, or reap.'
            node['required'] = ['kill_syscall_returned' if s == 'kill_delivered' else s for s in node['required']]
            for name in ('actual_exit', 'kill_eligibility_exit'):
                check('original unsigned exit domain', props[name] == {'anyOf': [{'type': 'integer', 'minimum': 0, 'maximum': 99999999999999999999}, {'type': 'null'}]})
                props[name] = {'anyOf': [{'type': 'integer'}, {'type': 'null'}]}
                schema_sites.append(pointer + '/properties/' + name)
        for key, value in node.items():
            transform(value, pointer + '/' + key)
    elif isinstance(node, list):
        for i, value in enumerate(node):
            transform(value, pointer + '/' + str(i))


transform(expected)
success = {'properties': {name: {'properties': {'actual_exit': {'const': 0}, 'kill_attempted': {'const': False}, 'kill_syscall_returned': {'const': False}}} for name in ('helper_collection', 'caller_collection')}}
expected['allOf'][0]['then']['allOf'] = [success]
check('exact semantic schema delta only', schema_after == expected)
check('eight signed domain sites', len(schema_sites) == 8)
write('schema-bindings.json', {'signed_integer_or_null_sites': schema_sites, 'semantic_delta': 'Only four parent field renames/descriptions, eight signed exit domains and success actual0/both flags false constraints', 'all_other_schema_data_exact': True, 'negative_exit_domain_reasoning': 'The integer type has no minimum/maximum at these eight sites; negative POSIX signal codes therefore belong to the failure domain. The success branch intersects actual_exit with const0 and both signal flags with const false. No receipt was constructed and no schema validator ran.', 'success_constraints': success})
for name in ('config.template.json', 'config.schema.json', 'copy-parent-stdin.schema.json', 'compact-console.schema.json'):
    check('other schema/template exact ' + name, (NEW / name).read_bytes() == (OLD / name).read_bytes())
config = json.loads(after_literals['CONFIG_TEXT'], object_pairs_hook=pairs, parse_constant=reject_constant)
check('embedded config equals template', config == read_json(NEW / 'config.template.json'))
check('config execution false', config['execution'] is False)
tokens = list(tokenize.generate_tokens(io.StringIO(after.decode()).readline))
config_assign = next(n for n in after_tree.body if isinstance(n, ast.Assign) and any(isinstance(t, ast.Name) and t.id == 'CONFIG_TEXT' for t in n.targets))
token = next(t for t in tokens if t.type == tokenize.STRING and t.start == (config_assign.value.lineno, config_assign.value.col_offset))
lines = after.splitlines(keepends=True)
start = sum(map(len, lines[:token.start[0] - 1])) + token.start[1]
end = sum(map(len, lines[:token.end[0] - 1])) + token.end[1]
check('one CONFIG string token', token.end == (config_assign.value.end_lineno, config_assign.value.end_col_offset))
check('exact current token restoration', after[:start] + after[start:end] + after[end:] == after)
config_assign.value.value = '<ONLY_CONFIG_TEXT_CONSTANT>'
proof = {'source': identity(after), 'token_start': start, 'token_end': end, 'token': identity(after[start:end]), 'prefix': identity(after[:start]), 'suffix': identity(after[end:]), 'config_canonical_without_LF': identity(json.dumps(config, sort_keys=True, separators=(',', ':'), allow_nan=False).encode()), 'normalized_AST_data_sha256': sha(ast.dump(after_tree, annotate_fields=True, include_attributes=False).encode())}
author_proof = read_json(NEW / 'template-proof.json')
check('all exact token/AST bindings', all(author_proof[k] == v for k, v in proof.items()))
proof['actual_future_render'] = None
proof['actual_prepared_or_receipt'] = None
proof['execution'] = False
write('template-bindings.json', proof)
write('static-observations.json', {'execution': False, 'observations': observations, 'count': len(observations), 'failed': [r for r in observations if not r['equal_or_satisfied']]})
print(json.dumps({'observations': len(observations), 'candidate_files': len(new_files), 'basis_files': len(old_files), 'prior_review_files': len(prior_files), 'declarations': len(bindings), 'global_segments': len(seg_bindings), 'signed_sites': len(schema_sites)}))
