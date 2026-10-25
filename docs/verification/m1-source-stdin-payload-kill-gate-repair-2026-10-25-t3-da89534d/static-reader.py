"""Author-owned static data reader. Never import or execute supplied source."""
import ast
import difflib
import hashlib
import json
import os
from pathlib import Path
import posixpath
import re
import subprocess

FOLDER = Path(__file__).resolve().parent
REPO = FOLDER.parents[2]
REL = FOLDER.relative_to(REPO).as_posix()
BASE = '0dad1c00d0a27f8266ed8e138f780a9d94656cdc'
CANDIDATE = 'fa72a0d865f12ae94b91f84400a31f51026c5b16'
IMPORT = 'dde69c794ae6b547393c81a3de411b6c9af1d179'
OLD = 'docs/verification/m1-source-stdin-payload-template-2026-10-25-t3-65b09134'
REVIEW = 'docs/verification/m1-source-stdin-payload-review-2026-10-25-t3-de175193'
R3 = 'docs/verification/m1-metadata-clock-failure-repair-2026-10-25-t3-d1e829a9'
R7 = 'docs/verification/m1-metadata-r7-repair-2026-10-25-t3-5ce08973'
PREP = 'docs/verification/m1-next-reviewed-ordinary-flow-2026-10-25-t3-a8a4ece5'


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def strict(raw):
    def pairs(items):
        result = {}
        for key, value in items:
            assert key not in result, ('duplicate JSON key', key)
            result[key] = value
        return result
    def invalid(value):
        raise ValueError(value)
    return json.loads(raw, object_pairs_hook=pairs, parse_constant=invalid)


def canon(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'),
                      ensure_ascii=True, allow_nan=False).encode('ascii')


def git(*args):
    env = dict(os.environ, GIT_OPTIONAL_LOCKS='0')
    return subprocess.check_output(['git', '-c', 'core.fsmonitor=false', *args],
                                   cwd=REPO, env=env)


def blob(ref, path):
    return git('show', ref + ':' + posixpath.normpath(path))


def functions(raw):
    lines = raw.splitlines(keepends=True)
    tree = ast.parse(raw, feature_version=(3, 9))
    result = {}
    def walk(nodes, scope=''):
        for node in nodes:
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                name = scope + node.name
                result[name] = (node, b''.join(lines[node.lineno - 1:node.end_lineno]))
                walk(node.body, name + '.')
            elif isinstance(node, ast.ClassDef):
                walk(node.body, scope + node.name + '.')
    walk(tree.body)
    return tree, result


def masked(raw, entries):
    lines = raw.splitlines(keepends=True)
    for name in ('collect', 'observe_exit'):
        node = entries[name][0]
        lines[node.lineno - 1:node.end_lineno] = [('<' + name + '>\n').encode()]
    return b''.join(lines)


def check_schema(value, schema):
    # Closed JSON vocabulary only; no candidate code is called.
    if 'const' in schema:
        assert canon(value) == canon(schema['const'])
    if 'oneOf' in schema:
        matched = 0
        for alternative in schema['oneOf']:
            try:
                check_schema(value, alternative)
                matched += 1
            except (AssertionError, TypeError):
                pass
        assert matched == 1
        return
    kind = schema.get('type')
    if kind == 'object':
        assert type(value) is dict and schema['additionalProperties'] is False
        assert set(value) == set(schema['required']) == set(schema['properties'])
        for key, item in value.items():
            check_schema(item, schema['properties'][key])
    elif kind == 'null':
        assert value is None
    elif kind == 'integer':
        assert type(value) is int and schema['minimum'] <= value <= schema['maximum']
    elif kind == 'string':
        assert type(value) is str and re.fullmatch(schema['pattern'], value)


source = (FOLDER / 'source-payload.py.txt').read_bytes()
old = blob(CANDIDATE, OLD + '/source-payload.py.txt')
assert len(old) == 37122 and sha(old) == 'dc208d02222c92418046139ecf622c797881af46b3edc1b8a891742b22f9fd9f'
tree, current_functions = functions(source)
old_tree, old_functions = functions(old)
assert set(current_functions) == set(old_functions) and len(current_functions) == 28
changed = [name for name in current_functions if current_functions[name][1] != old_functions[name][1]]
assert changed == ['observe_exit', 'collect']
assert masked(source, current_functions) == masked(old, old_functions)
inventory = strict((FOLDER / 'function-inventory.json').read_bytes())
assert inventory['source_sha256'] == sha(source) and inventory['unchanged_functions'] == 26
assert inventory['changed_functions'] == changed and len(inventory['functions']) == 28
for item in inventory['functions']:
    node, raw = current_functions[item['name']]
    assert (item['first_line'], item['last_line'], item['bytes'], item['sha256']) == (
        node.lineno, node.end_lineno, len(raw), sha(raw))
    assert item['baseline_sha256'] == sha(old_functions[item['name']][1])
    assert item['changed'] is (item['name'] in changed)

# Verify the exact original folder at candidate, import, parent and current tree.
old_names = git('ls-tree', '--name-only', CANDIDATE + ':' + OLD).decode().splitlines()
assert len(old_names) == 11
for name in old_names:
    path = OLD + '/' + name
    raw = blob(CANDIDATE, path)
    assert raw == blob(IMPORT, path) == blob(BASE, path) == (REPO / path).read_bytes()
for line in blob(CANDIDATE, OLD + '/SHA256SUMS').decode().splitlines():
    digest, name = line.split('  ')
    assert digest == sha(blob(CANDIDATE, OLD + '/' + name))
for name in ('config.template.json', 'config.schema.json'):
    assert (FOLDER / name).read_bytes() == blob(CANDIDATE, OLD + '/' + name)
assert (FOLDER / 'inherited-controls.md').read_bytes() == blob(CANDIDATE, OLD + '/README.md')

bindings = strict((FOLDER / 'source-bindings.json').read_bytes())
assert bindings['parent_commit'] == BASE and bindings['source_sha256'] == sha(source)
assert bindings['source_bytes'] == len(source) and bindings['execution'] is False
assert all(bindings[key] is None for key in (
    'final_caller_candidate', 'final_caller_sha256', 'final_caller_review',
    'payload_independent_review', 'actual_rendered_payload_sha256'))
for item in bindings['verified_public_inputs']:
    raw = blob(item['ref'], item['path'])
    assert len(raw) == item['bytes'] and sha(raw) == item['sha256']
    assert git('rev-parse', item['ref'] + ':' + posixpath.normpath(item['path'])).decode().strip() == item['blob']
    assert (REPO / item['path']).read_bytes() == raw
for item in bindings['inherited_public_inputs']:
    raw = blob(item['ref'], item['path'])
    assert len(raw) == item['bytes'] and sha(raw) == item['sha256']
assert len(bindings['inherited_public_inputs']) == 44
review = strict(blob('4b5d756769cdc540abd7f07c92fead31b9530101', REVIEW + '/findings.json'))
assert review['verdict'] == 'HOLD' and review['p2_count'] == 1
assert review['findings'][0]['id'] == 'PAYLOAD-P2-1'

assignments = {node.targets[0].id:node for node in tree.body if isinstance(node, ast.Assign)
               and len(node.targets) == 1 and isinstance(node.targets[0], ast.Name)}
config_call = assignments['CONFIG'].value
assert isinstance(config_call, ast.Call) and ast.unparse(config_call.func) == 'json.loads'
assert len(config_call.args) == 1 and not config_call.keywords
assert isinstance(config_call.args[0], ast.Constant) and type(config_call.args[0].value) is str
config = strict(config_call.args[0].value)
assert canon(config) == canon(strict((FOLDER / 'config.template.json').read_bytes()))
check_schema(config, strict((FOLDER / 'config.schema.json').read_bytes()))
pending = list(config['caller'].values()) + list(config['payload_review'].values())
pending += list(config['prerequisites'].values()) + [config['template_sha256']]
assert len(pending) == 22 and all(value is None for value in pending)
assert config['execution'] is False and sha(canon(config)) == bindings['config_canonical_sha256']
constants = {key:ast.literal_eval(assignments[key].value) for key in (
    'ROOT', 'SYSTEM', 'PRIVATE_PYTHON', 'SOURCE_ROOT', 'CLOCK', 'COMPLETED', 'INITIAL_INPUTS')}
assert constants['ROOT'].endswith('4b20357c1503') and config['evidence_root'] == constants['ROOT']
assert config['initial_inputs'] == constants['INITIAL_INPUTS'] and len(config['initial_inputs']) == 20
assert config['completed_files'] == constants['COMPLETED'] and len(config['completed_files']) == 7
assert sha(canon(constants['COMPLETED']) + b'\n') == config['manifest_sha256']
pins = strict(blob(BASE, R3 + '/input-pins.json'))['files']
assert len(pins) == 17
for name, item in pins.items():
    raw = blob(BASE, item['repository_path'])
    assert len(raw) == item['bytes'] and sha(raw) == item['sha256']
    assert config['initial_inputs'][name] == {key:item[key] for key in ('bytes', 'sha256')}
modules = sorted(name for name in pins if name.endswith('.py'))
assert modules == sorted(['common.py', 'controller.py', 'coverage.py', 'exec_owned.py',
    'macproc.py', 'ownership.py', 'port_diagnostic.py', 'prepare.py', 'probe.py', 'watchdog.py'])
vectors = strict(blob(BASE, R3 + '/command-vectors.json'))['commands']
assert len(vectors) == len({item['label'] for item in vectors}) == 50
for name in ('input-pins.json', 'command-vectors.json', 'source-parent.template.json', 'parent-boundary.schema.json'):
    assert blob(BASE, R3 + '/' + name) == blob(BASE, R7 + '/' + name)
assert strict(blob(BASE, R7 + '/closed-cli.json'))['source'] == strict(blob(BASE, R3 + '/closed-cli.json'))['source']
assert strict(blob(BASE, R7 + '/startup-contract.json'))['parent_input']['values'] == strict(blob(BASE, PREP + '/next-operation.json'))['parent_startup']

# Inspect statements and AST as data; never call candidate functions.
collect = current_functions['collect'][0]
observe = current_functions['observe_exit'][0]
assert len(observe.body) == 2 and isinstance(observe.body[0], ast.If)
assert ast.unparse(observe.body[0].test) == "code is not None and status['actual_exit'] is None"
assert [ast.unparse(node) for node in observe.body[0].body] == [
    "status['actual_exit'] = code", "status['reaped'] = True",
    "status['actual_exit_observed_ns'] = clock_ns()", "return status['actual_exit_observed_ns']"]
assert ast.unparse(observe.body[1]) == 'return clock_ns()'
settle = [node for node in ast.walk(collect) if isinstance(node, ast.With)
          and ast.unparse(node.items[0].context_expr) == 'until(reap_end)']
assert len(settle) == 1
body = settle[0].body
assert [ast.unparse(node) for node in body[:4]] == [
    'code = proc.poll()', 'observed_ns = observe_exit(proc, status, code)',
    "status['kill_eligibility_exit'] = code", "status['kill_eligibility_observed_ns'] = observed_ns"]
assert len(body) == 5 and isinstance(body[4], ast.If) and ast.unparse(body[4].test) == 'code is None'
branch = body[4].body
assert isinstance(branch[0], ast.If)
assert ast.unparse(branch[0].test) == 'not (observed_ns < reap_end and observed_ns < end)'
assert [ast.unparse(node) for node in branch[0].body] == [
    "status['timed_out'] = True", "status['unread_or_dropped_unknown'] = True",
    "raise TimeoutError('late KILL eligibility poll; no signal')"]
assert [ast.unparse(node) for node in branch[1:]] == [
    "status['kill_attempted'] = True", 'proc.kill()', "status['kill_delivered'] = True",
    'code = proc.wait(timeout=remaining(reap_end))', 'observe_exit(proc, status, code)']
calls = [ast.unparse(node.func) for node in ast.walk(tree) if isinstance(node, ast.Call)]
for name, count in {'subprocess.Popen':1, 'proc.poll':2, 'proc.wait':1, 'proc.kill':1,
                    'os.write':2, 'os.fsync':2}.items():
    assert calls.count(name) == count
assert 'work_end, reap_end = end - 2 * NS, end - 1500000000' in source.decode()
assert 'end = started + 120 * NS' in source.decode()
assert "require(status['reaped'] is True, 'unreaped caller: no subsequent file writes')" in source.decode()
assert not any(name in calls for name in ('eval', 'exec', 'compile', '__import__',
    'os.kill', 'os.killpg', 'os.mkdir', 'os.makedirs', 'os.unlink', 'os.remove',
    'os.rename', 'os.replace', 'subprocess.run', 'proc.communicate'))

# Reconstruct forward and reverse patch data, without applying it to files.
patch = (FOLDER / 'payload-from-fa72.patch').read_bytes()
expected = ''.join(difflib.unified_diff(old.decode().splitlines(keepends=True),
    source.decode().splitlines(keepends=True), fromfile='a/' + OLD + '/source-payload.py.txt',
    tofile='b/' + REL + '/source-payload.py.txt', n=3)).encode()
assert patch == expected
lines = patch.splitlines(keepends=True)
old_lines, new_lines = old.splitlines(keepends=True), source.splitlines(keepends=True)
forward, reverse, oi, ni, hunks = [], [], 0, 0, 0
for line in lines[2:]:
    if line.startswith(b'@@ '):
        match = re.fullmatch(rb'@@ -(\d+),(\d+) \+(\d+),(\d+) @@\n', line)
        assert match
        os_, oc, ns, nc = map(int, match.groups())
        forward.extend(old_lines[oi:os_ - 1]); oi = os_ - 1
        reverse.extend(new_lines[ni:ns - 1]); ni = ns - 1
        old_count, new_count = 0, 0
        hunks += 1
    else:
        assert line[:1] in (b' ', b'+', b'-')
        raw = line[1:]
        if line[:1] in (b' ', b'-'):
            assert old_lines[oi] == raw
            oi += 1; old_count += 1
            reverse.append(raw)
        if line[:1] in (b' ', b'+'):
            assert new_lines[ni] == raw
            ni += 1; new_count += 1
            forward.append(raw)
        assert old_count <= oc and new_count <= nc
forward.extend(old_lines[oi:]); reverse.extend(new_lines[ni:])
assert b''.join(forward) == source and b''.join(reverse) == old and hunks == 3
assert bindings['patch_sha256'] == sha(patch)
plan = strict((FOLDER / 'finite-plan.json').read_bytes())
assert plan['execution'] is False and not any(plan['execution_counts'].values())
assert plan['source_clear'] is False and plan['payload_clear_review'] is None
assert plan['independent_review'] is None and plan['human_gate_created'] is False
json_files = sorted(path.name for path in FOLDER.glob('*.json') if path.name != 'static-checks.json')
for name in json_files:
    strict((FOLDER / name).read_bytes())
assert not any(path.stat().st_mode & 0o111 for path in FOLDER.iterdir())
assert not git('diff', '--name-only', BASE, '--', '.', ':(exclude)' + REL).strip()
print(json.dumps(dict(
    schema='openergy.source-payload-kill-gate-static-checks.v1', execution=False,
    source_bytes=len(source), source_lines=source.count(b'\n'), source_sha256=sha(source),
    patch_bytes=len(patch), patch_sha256=sha(patch), patch_hunks=hunks,
    changed_functions=changed, unchanged_functions_byte_exact=26,
    outside_changed_functions_byte_exact=True, python39_ast_as_data=True,
    baseline_files_exact_across_candidate_import_parent_current=11,
    baseline_manifest_entries_verified=10, verified_public_inputs=len(bindings['verified_public_inputs']),
    inherited_public_declarations_verified=44, pins_verified=17, module_names_verified=10,
    ordered_vector_identity_count=50, r7_parent_schema_cli_env_data_identity=True,
    r7_behavior_or_interface_acceptance=False, null_config_slots=22,
    closed_json_schema_and_embedded_config_equal=True, duplicate_and_nonfinite_rejected=True,
    eligibility_poll_sample_retained=True, strict_both_deadlines_before_kill=True,
    late_none_failure_unknown_branch_before_kill=True, real_exit_preserved_before_clock=True,
    wait_and_fixed_deadlines_unchanged=True, unreaped_file_write_gate_unchanged=True,
    forward_and_reverse_patch_reconstruction=True, supplied_source_or_checker_calls=0,
    tests=0, mocks=0, native_commands=0, remote_connections=0, private_state_access=0,
    children=0, build=0, signing=0, json_data_files_verified=json_files,
    independent_review=None, source_clear=False, runtime='UNVERIFIED',
    limits=['Static data checks only; no independent review or runtime acceptance.',
            'No hard OS preemption, timely signal/reap, delivery or durability proof.',
            'Parent causal inspection and exclusive temporal ownership remain external.',
            'Caller R7 compact interface and transport repair remain unreviewed.']),
    indent=2, sort_keys=True, allow_nan=False))
