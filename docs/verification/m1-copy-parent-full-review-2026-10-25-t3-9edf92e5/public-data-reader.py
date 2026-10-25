"""Independent public Git/text/JSON/hash/AST-data reader. No candidate evaluation.

This reader inspects inert declarations. It does not import, execute, compile to
executable code, render, or exercise any supplied implementation or checker.
"""
import ast
import collections
import hashlib
import json
import math
import os
from pathlib import Path
import re
import subprocess

ROOT = Path.cwd()
OUT = Path(__file__).parent
PACKET = Path('docs/verification/m1-copy-parent-capsule-2026-10-25-t3-f65c3d2f')
CANDIDATE = '85c668f7e63c07e1f9d3cf9e76af05f21898f80d'
PARENT = '9379e2b50b566e3346ec934b322ef142db2ade39'
IMPORTED = 'cd4fae4155e50e99d033fc78553ad50bbfa7d164'
checks = []


def git(*args):
    return subprocess.check_output(['git', *args], env=dict(os.environ, GIT_OPTIONAL_LOCKS='0'))


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def check(label, value):
    checks.append(dict(name=label, passed=bool(value)))


def strict(raw):
    def pairs(rows):
        value = {}
        for key, item in rows:
            if key in value:
                raise ValueError('duplicate key')
            value[key] = item
        return value
    def nonfinite(value):
        raise ValueError('nonfinite constant')
    def number(value):
        value = float(value)
        if not math.isfinite(value):
            raise ValueError('nonfinite float')
        return value
    return json.loads(raw, object_pairs_hook=pairs, parse_constant=nonfinite, parse_float=number)


def canonical(value):
    return (json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=True,
                       allow_nan=False) + '\n').encode('ascii')


def save(name, value):
    (OUT / name).write_text(json.dumps(value, indent=2, ensure_ascii=True) + '\n')


def declarations(raw):
    tree = ast.parse(raw, feature_version=(3, 9))
    lines = raw.splitlines(keepends=True)
    found = {}
    def walk(nodes, prefix=''):
        for node in nodes:
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
                name = prefix + node.name
                start = min([node.lineno] + [d.lineno for d in node.decorator_list])
                body = b''.join(lines[start - 1:node.end_lineno])
                found[name] = dict(start=start, end=node.end_lineno, bytes=len(body),
                                   sha256=digest(body), kind='class' if isinstance(node, ast.ClassDef) else 'function')
                walk(node.body, name + '.')
    walk(tree.body)
    return tree, found


def constants(tree):
    found = {}
    for node in tree.body:
        if isinstance(node, ast.Assign) and len(node.targets) == 1 and isinstance(node.targets[0], ast.Name):
            try:
                found[node.targets[0].id] = ast.literal_eval(node.value)
            except (ValueError, TypeError):
                pass
    return found


manifest = []
packet_json = {}
for path in sorted(PACKET.iterdir()):
    raw = path.read_bytes()
    entry = dict(path=str(path), bytes=len(raw), lines=len(raw.splitlines()), sha256=digest(raw),
                 working_mode=oct(path.stat().st_mode & 0o7777))
    for label, revision in [('candidate', CANDIDATE), ('imported', IMPORTED)]:
        stored = git('show', revision + ':' + str(path))
        entry[label + '_raw_equal'] = stored == raw
        entry[label + '_tree_entry'] = git('ls-tree', revision, '--', str(path)).decode().strip()
    check('exact packet file: ' + path.name, entry['candidate_raw_equal'] and entry['imported_raw_equal'])
    if path.suffix == '.json':
        packet_json[path.name] = strict(raw)
        entry['strict_JSON_read'] = True
    manifest.append(entry)
check('exact25 packet files', len(manifest) == 25)
check('all packet Git modes100644 and working0644', all(r['candidate_tree_entry'].startswith('100644 ') and r['imported_tree_entry'].startswith('100644 ') and r['working_mode'] == '0o644' for r in manifest))
for row in (PACKET / 'SHA256SUMS').read_text().splitlines():
    expected, name = row.split('  ', 1)
    check('packet SHA256SUMS: ' + name, digest((PACKET / name).read_bytes()) == expected)
check('exact parent ancestry', git('show', '-s', '--format=%P', CANDIDATE).decode().strip() == PARENT)
delta = git('diff-tree', '--no-commit-id', '--name-status', '-r', PARENT, CANDIDATE).decode().splitlines()
check('candidate adds only packet', len(delta) == 25 and all(row.startswith('A\t' + str(PACKET) + '/') for row in delta))
check('full4291 parent tree text exact', git('ls-tree', '-r', PARENT) == (PACKET / 'unchanged-parent-tree.txt').read_bytes())
save('packet-manifest.json', manifest)

source = (PACKET / 'copy-parent.py.txt').read_bytes()
inventory = packet_json['function-inventory.json']
baseline = Path(inventory['baseline']).read_bytes()
tree, after = declarations(source)
old_tree, before = declarations(baseline)
literal = constants(tree)
old_literal = constants(old_tree)
check('full source hash', digest(source) == 'cfe3be7a67d22160b977a97941f7fa6ed44be322ab6822dd064c66bb93555b47')
check('baseline37775 hash', len(baseline) == 37775 and digest(baseline) == '8d7844a20ce347b7be52c07a516b8d9136bdb5c067e450bea24eb3453441afa6')
check('48 functions and1 class', len(after) == 49 and sum(v['kind'] == 'function' for v in after.values()) == 48)
rows = []
for name in sorted(set(before) | set(after)):
    left, right = before.get(name), after.get(name)
    state = 'ADDED' if left is None else 'REMOVED' if right is None else 'UNCHANGED' if left['sha256'] == right['sha256'] else 'CHANGED'
    rows.append(dict(name=name, before=left, after=right, state=state))
check('full before/after declaration inventory exact', rows == inventory['entries'])
markdown = (PACKET / 'FUNCTIONS.md').read_text()
sections = dict((m.group(1), m.group(2)) for m in re.finditer(r'^## (\S+) [^\n]*\n(.*?)(?=^## |\Z)', markdown, re.M | re.S))
for entry in rows:
    section = sections[entry['name']]
    blocks = re.findall(r'```python\n(.*?)```', section, re.S)
    if entry['state'] == 'UNCHANGED':
        check('unchanged declaration MD: ' + entry['name'], not blocks and entry['before']['sha256'] in section)
    else:
        expected = []
        for key, raw in [('before', baseline), ('after', source)]:
            record = entry[key]
            expected.append('# No declaration in this version.\n' if record is None else b''.join(raw.splitlines(keepends=True)[record['start']-1:record['end']]).decode())
        check('full before/after MD: ' + entry['name'], blocks == expected)
save('function-bindings.json', dict(functions=48, classes=1, states=dict(collections.Counter(r['state'] for r in rows)), entries=rows))

patch = (PACKET / 'from-cleared-source-launcher.patch').read_bytes()
patch_lines = patch.splitlines(keepends=True)
old_lines = baseline.splitlines(keepends=True)
rebuilt, at, i, hunks = [], 0, 2, []
while i < len(patch_lines):
    header = re.fullmatch(rb'@@ -(\d+),(\d+) \+(\d+),(\d+) @@\n', patch_lines[i])
    if not header:
        raise ValueError('unexpected unified patch row')
    old_start, old_count, new_start, new_count = map(int, header.groups())
    rebuilt.extend(old_lines[at:old_start-1])
    at = old_start-1
    check('patch new location ' + str(i+1), len(rebuilt) == new_start-1)
    i += 1
    old_used = new_used = 0
    first = i+1
    while i < len(patch_lines) and not patch_lines[i].startswith(b'@@ '):
        row = patch_lines[i]
        if row[:1] in (b' ', b'-'):
            check('patch old row ' + str(i+1), row[1:] == old_lines[at])
            at += 1
            old_used += 1
        if row[:1] in (b' ', b'+'):
            rebuilt.append(row[1:])
            new_used += 1
        if row[:1] not in (b' ', b'-', b'+'):
            raise ValueError('invalid patch prefix')
        i += 1
    check('patch hunk counts ' + str(first), (old_used, new_used) == (old_count, new_count))
    hunks.append(dict(old_start=old_start, old_count=old_count, new_start=new_start, new_count=new_count,
                      patch_first_body=first, patch_last_body=i))
rebuilt.extend(old_lines[at:])
check('full exact raw patch reconstruction', b''.join(rebuilt) == source)
check('patch hash', digest(patch) == 'a9608f188227ff7a8ae593dec9258e7046cb24469fe6a9f96de98bc485df2aee')
save('patch-observations.json', dict(hunks=hunks, raw_reconstruction_equal=b''.join(rebuilt) == source,
    baseline_sha256=digest(baseline), patch_sha256=digest(patch), candidate_sha256=digest(source),
    literal_context_only_space_rows=[i+1 for i,row in enumerate(patch_lines) if row == b' \n'],
    unchanged_literal_declarations=[k for k,v in literal.items() if k in old_literal and type(v) is type(old_literal[k]) and v == old_literal[k]]))

inputs = []
for record in packet_json['input-bindings.json']['records']:
    raw = git('show', record['commit'] + ':' + record['path'])
    line = git('ls-tree', record['commit'], '--', record['path']).decode().strip()
    mode, kind, blob_path = line.split(' ', 2)
    blob = blob_path.split('\t', 1)[0]
    valid = len(raw) == record['bytes'] and digest(raw) == record['sha256'] and mode == record['mode'] and blob == record['blob']
    check('frozen Git input: ' + record['path'], valid)
    inputs.append(dict(record, verified=valid, current_working_raw_equal=Path(record['path']).read_bytes() == raw))
save('dependency-bindings.json', inputs)
pins_path = Path('docs/verification/m1-metadata-r9-repair-2026-10-25-t3-9b2c2354/input-pins.json')
pins = strict(pins_path.read_bytes())['files']
for name, row in pins.items():
    raw = Path(row['repository_path']).read_bytes()
    check('17 pin public bytes: ' + name, len(raw) == row['bytes'] and digest(raw) == row['sha256'] and literal['PINS'][name] == {k:row[k] for k in ('bytes','sha256')})
check('exact17 and10 module names', len(pins) == 17 and len([n for n in pins if n.endswith('.py')]) == 10)
source_validation = strict(Path('docs/verification/m1-source-metadata-observed-2026-10-25-t3-52422617/validation.json').read_bytes())
check('all212 SOURCE manifest rows', literal['SOURCE_RAWS'] == source_validation['record_files'])
check('SOURCE manifest canonical hash', digest(canonical(literal['SOURCE_RAWS'])) == source_validation['raw_records_manifest_sha256'])
check('SOURCE manifest213334 bytes', sum(r['bytes'] for r in literal['SOURCE_RAWS']) == 213334)
check('fixed327813 SOURCE result', literal['SOURCE_RESULT'] == dict(bytes=source_validation['raw_result_bytes'], sha256=source_validation['raw_result_sha256']))
artifact_receipt = strict(Path('docs/verification/m1-artifact-data-observed-2026-10-25-t3-52422617/materializer-receipt.json').read_bytes())
check('all8 artifact identities136360', sum(x['bytes'] for x in literal['EIGHT'].values()) == 136360 and all({k:v[k] for k in ('bytes','sha256')} == literal['EIGHT'][n] for n,v in artifact_receipt['files'].items()))
check('all6 SOURCE boundary identities', literal['SOURCE_BOUNDARY_BINDINGS'] == artifact_receipt['source_inputs']['boundary'])
vectors = strict(Path('docs/verification/m1-metadata-r9-repair-2026-10-25-t3-9b2c2354/command-vectors.json').read_bytes())['commands']
check('50 immutable unique vectors', len(vectors) == len({v['label'] for v in vectors}) == 50)
recipe = strict(Path('docs/verification/m1-next-reviewed-ordinary-flow-2026-10-25-t3-a8a4ece5/metadata.recipe.json').read_bytes())
expanded = []
for obj in recipe['objects']:
    tokens = {'${BINARY}': '${BUNDLE}/' + obj['binary'], '${CODE_OBJECT}': '${BUNDLE}' + ('/' + obj['code_object'] if obj['code_object'] else '')}
    for verb in recipe['verb_order']:
        expanded.append(dict(label='object-' + str(obj['index']) + '-' + verb,
                             argv=[tokens.get(s,s) for s in recipe['argv_patterns'][verb]]))
expanded.append(dict(label='full-bundle-deep-verify', argv=recipe['deep_verify']))
check('all50 vectors independently expanded from full recipe', expanded == vectors)
materialization_path = Path('docs/verification/ordinary-combined-context-tuple-repair-2026-10-25/materialization.json')
materialization = strict(materialization_path.read_bytes())
source_checks = strict((materialization_path.parent / 'source-checks.json').read_bytes())['python_files']
for row in materialization['modules']:
    raw = (materialization_path.parent / row['source']).read_bytes()
    check('module/source-check/pin exact: ' + row['destination'], digest(raw) == row['sha256'] == pins[row['destination']]['sha256'] == source_checks[row['destination']]['sha256'] and len(raw) == pins[row['destination']]['bytes'] == source_checks[row['destination']]['bytes'])

functions = {n.name:n for n in tree.body if isinstance(n, ast.FunctionDef)}
def dict_keys(node):
    return {k.arg for k in node.keywords}
def returned_dict_keys(name):
    returns = [n.value for n in ast.walk(functions[name]) if isinstance(n, ast.Return) and isinstance(n.value, ast.Call) and isinstance(n.value.func, ast.Name) and n.value.func.id == 'dict']
    if returns:
        return dict_keys(returns[0])
    return dict_keys(next(n.value for n in ast.walk(functions[name]) if isinstance(n, ast.Assign) and any(isinstance(t,ast.Name) and t.id == 'value' for t in n.targets) and isinstance(n.value,ast.Call) and isinstance(n.value.func,ast.Name) and n.value.func.id == 'dict'))
stdin_schema = packet_json['copy-parent-stdin.schema.json']
check('seven exact stdin keys', returned_dict_keys('copy_parent_stdin') == set(stdin_schema['required']) == set(stdin_schema['properties']) and len(stdin_schema['required']) == 7)
auth_keys = returned_dict_keys('inspect_entry')
check('nineteen exact auth keys', auth_keys == set(stdin_schema['properties']['entry_authentication']['required']) and len(auth_keys) == 19)
next_operation = strict(Path('docs/verification/m1-next-reviewed-ordinary-flow-2026-10-25-t3-a8a4ece5/next-operation.json').read_bytes())
check('exact ten startup keys', returned_dict_keys('startup') == set(next_operation['parent_startup']) and len(next_operation['parent_startup']) == 10)
receipt_schema = packet_json['prepared-copy-qualification.schema.json']
receipt_call = next(n.value for n in ast.walk(functions['main']) if isinstance(n, ast.Assign) and any(isinstance(t,ast.Name) and t.id == 'receipt' for t in n.targets))
check('all40 exact receipt keys including reserved conversion', dict_keys(receipt_call) | {'common_clock_conversions_including_receipt'} == set(receipt_schema['properties']) == set(receipt_schema['required']))
schema_rows = []
def walk_schema(value, path=''):
    if isinstance(value, dict):
        if '$ref' in value:
            target = receipt_schema
            for part in value['$ref'].split('/')[1:]:
                target = target[part]
        if 'type' in value and value['type'] == 'object':
            schema_rows.append(dict(path=path, keys=list(value.get('properties',{})), required=value.get('required',[]), closed=value.get('additionalProperties') is False))
        for key,item in value.items():
            walk_schema(item,path+'/'+key)
    elif isinstance(value,list):
        for i,item in enumerate(value):
            walk_schema(item,path+'/'+str(i))
walk_schema(receipt_schema)
check('all full receipt variable objects closed', all(r['closed'] for r in schema_rows))
save('schema-observations.json',dict(receipt_objects=schema_rows,
    negative_signal_exit_domain_gap=dict(helper_actual_exit=receipt_schema['properties']['helper_collection']['anyOf'][0]['properties']['actual_exit'],
        caller_actual_exit=receipt_schema['properties']['caller_collection']['anyOf'][0]['properties']['actual_exit'],
        helper_eligibility_exit=receipt_schema['properties']['helper_collection']['anyOf'][0]['properties']['kill_eligibility_exit'],
        caller_eligibility_exit=receipt_schema['properties']['caller_collection']['anyOf'][0]['properties']['kill_eligibility_exit']),
    supplied_schema_validator_run=False, synthetic_receipts_created=False))

stdlib_path = Path('/Applications/Xcode.app/Contents/Developer/Library/Frameworks/Python3.framework/Versions/3.9/lib/python3.9/subprocess.py')
stdlib_raw = stdlib_path.read_bytes()
stdlib_lines = stdlib_raw.splitlines(keepends=True)
stdlib_ranges = [(110,132),(428,437),(1158,1161),(1827,1869),(2040,2080)]
save('public-stdlib-source-evidence.json',dict(path=str(stdlib_path), bytes=len(stdlib_raw), sha256=digest(stdlib_raw),
    read_as_text_only=True, imported_or_executed=False, target_machine_identity_verified=False,
    ranges=[dict(start=a,end=b,text=b''.join(stdlib_lines[a-1:b]).decode()) for a,b in stdlib_ranges],
    observation='POSIX Popen.kill -> send_signal -> self.poll -> _internal_poll -> waitpid(WNOHANG), then os.kill only when returncode stays None. A returned kill call can skip signal.'))
public_bootstrap = []
for path, ranges in [
    (Path('/Applications/Xcode.app/Contents/Developer/Library/Frameworks/Python3.framework/Versions/3.9/Headers/patchlevel.h'), [(1,36)]),
    (stdlib_path.parent / 'importlib/_bootstrap_external.py', [(696,748),(916,998),(1065,1084),(1632,1646)])]:
    raw = path.read_bytes()
    lines = raw.splitlines(keepends=True)
    public_bootstrap.append(dict(path=str(path),bytes=len(raw),sha256=digest(raw),
        ranges=[dict(start=a,end=b,text=b''.join(lines[a-1:b]).decode()) for a,b in ranges]))
save('public-bootstrap-source-evidence.json',dict(inputs=public_bootstrap,
    read_as_text_only=True, target_machine_identity_verified=False,
    observation='Local public header declares3.9.6. SourceFileLoader can read bytecode despite -B. Candidate rejects ROOT cache/source/package alternatives before common and private entry. Private normal startup effects remain UNMEASURED.'))

config = strict(literal['CONFIG_TEXT'])
check('complete embedded template exact', config == packet_json['config.template.json'] == packet_json['config.schema.json']['oneOf'][0]['const'])
check('nine prepared slots null', len(config['prepared']) == 9 and all(v is None for v in config['prepared'].values()))
check('execution false and all duties/reviews/hash null', config['execution'] is False and config['template_sha256'] is None and all(v is None for v in config['parent'].values()) and all(v is None for review in config['reviews'].values() for v in review.values()))
config_node = next(n.value for n in tree.body if isinstance(n, ast.Assign) and isinstance(n.targets[0], ast.Name) and n.targets[0].id == 'CONFIG_TEXT')
source_lines = source.splitlines(keepends=True)
start = sum(map(len, source_lines[:config_node.lineno-1])) + config_node.col_offset
end = sum(map(len, source_lines[:config_node.end_lineno-1])) + config_node.end_col_offset
proof = packet_json['template-proof.json']
check('single CONFIG token boundaries and prefix/suffix', start == proof['token_start'] and end == proof['token_end'] and digest(source[:start]) == proof['prefix_sha256'] and digest(source[end:]) == proof['suffix_sha256'] and digest(source[start:end]) == proof['token_sha256'])
check('canonical CONFIG hash', digest(canonical(config)[:-1]) == proof['canonical_CONFIG_without_LF_sha256'])
save('literal-bindings.json', dict(constants=literal, vectors=vectors, config_token=dict(start=start,end=end),
    private_basis_raw_read=False, private_full_SOURCE_raw_read=False, note='Public manifest comparison only; no private/state file opened.'))

whitespace = []
for label, extras in [('full', []), ('non_patch', [':(exclude)' + str(PACKET / 'from-cleared-source-launcher.patch')])]:
    result = subprocess.run(['git','diff','--check',PARENT,CANDIDATE,'--',str(PACKET),*extras],
        env=dict(os.environ,GIT_OPTIONAL_LOCKS='0'), stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    whitespace.append(dict(scope=label,exit=result.returncode,stdout_utf8=result.stdout.decode(),stderr_utf8=result.stderr.decode()))
check('whitespace distinction full2 nonpatch0', [x['exit'] for x in whitespace] == [2,0])
save('whitespace-observations.json', whitespace)
save('static-observations.json', dict(schema='independent-copy-parent-public-static-observations.v1',
    execution=False, candidate=CANDIDATE, exact_parent=PARENT, imported=IMPORTED,
    candidate_lines=len(source_lines), candidate_bytes=len(source), packet_files=len(manifest),
    checks=checks, passed=sum(x['passed'] for x in checks), failed=sum(not x['passed'] for x in checks),
    supplied_checker_executed=False, supplied_code_imported_executed_or_compiled_to_executable=False,
    candidate_predicates_exercised=False, runtime_verified=False))
print(json.dumps(dict(passed=sum(x['passed'] for x in checks), failed=[x for x in checks if not x['passed']]),indent=2))
