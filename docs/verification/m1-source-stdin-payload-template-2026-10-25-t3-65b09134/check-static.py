"""Owned static AST/JSON/hash/patch reader. Never run or import the payload."""
import ast
import difflib
import hashlib
import json
from pathlib import Path
import subprocess


FOLDER = Path(__file__).resolve().parent
REPO = FOLDER.parents[2]
BASE = 'a26871ad9fdae2d55ff53544aed26ae0cdf2620e'
R3 = 'docs/verification/m1-metadata-clock-failure-repair-2026-10-25-t3-d1e829a9'
R4 = 'docs/verification/m1-metadata-clock-failure-review-2026-10-25-t3-e92ea4b9'
PREP = 'docs/verification/m1-next-reviewed-ordinary-flow-2026-10-25-t3-a8a4ece5'
AUDIT = 'docs/verification/m1-canonical-source-file-audit-2026-10-25-t3-ee636b1e'
BRIDGE = 'docs/verification/m1-metadata-basis-bridge-2026-10-25-t3-755a8865'
SIZE = 'docs/verification/m1-source-transport-size-binding-2026-10-25'


def sha(data):
    return hashlib.sha256(data).hexdigest()


def parse_json(raw):
    def pairs(items):
        out = {}
        for key, value in items:
            assert key not in out, ('duplicate key', key)
            out[key] = value
        return out
    def invalid(value):
        raise ValueError(value)
    return json.loads(raw, object_pairs_hook=pairs, parse_constant=invalid)


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=True,
                      allow_nan=False).encode('ascii')


def write_json(name, value):
    (FOLDER / name).write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')


def git(*args):
    return subprocess.check_output(['git', '-c', 'core.fsmonitor=false', *args], cwd=REPO)


def schema_check(value, schema):
    # Validate only the closed schema vocabulary used in this folder.
    if 'const' in schema:
        assert canonical(value) == canonical(schema['const'])
    if 'oneOf' in schema:
        passed = 0
        for choice in schema['oneOf']:
            try:
                schema_check(value, choice)
                passed += 1
            except (AssertionError, TypeError):
                pass
        assert passed == 1
        return
    kind = schema.get('type')
    if kind == 'object':
        assert type(value) is dict
        assert schema['additionalProperties'] is False
        assert set(value) == set(schema['required']) == set(schema['properties'])
        for key, item in value.items():
            schema_check(item, schema['properties'][key])
    elif kind == 'null':
        assert value is None
    elif kind == 'integer':
        assert type(value) is int and schema['minimum'] <= value <= schema['maximum']
    elif kind == 'string':
        assert type(value) is str
        # Only these fixed SHA/Git patterns are permitted in this vocabulary.
        width = {'^[0-9a-f]{64}$': 64, '^[0-9a-f]{40}$': 40}[schema['pattern']]
        assert len(value) == width and all(c in '0123456789abcdef' for c in value)


source = (FOLDER / 'source-payload.py.txt').read_bytes()
text = source.decode('utf-8')
tree = ast.parse(text, filename='source-payload.py.txt', feature_version=(3, 9))
assignments = {n.targets[0].id: n for n in tree.body if isinstance(n, ast.Assign)
               and len(n.targets) == 1 and isinstance(n.targets[0], ast.Name)}
config_node = assignments['CONFIG']
assert isinstance(config_node.value, ast.Call)
assert ast.unparse(config_node.value.func) == 'json.loads'
assert len(config_node.value.args) == 1 and not config_node.value.keywords
config_token = config_node.value.args[0]
assert isinstance(config_token, ast.Constant) and type(config_token.value) is str
config = parse_json(config_token.value)
assert config == parse_json((FOLDER / 'config.template.json').read_bytes())
schema = parse_json((FOLDER / 'config.schema.json').read_bytes())
schema_check(config, schema)
assert config['execution'] is False and config['template_sha256'] is None
assert all(v is None for v in config['caller'].values())
assert all(v is None for v in config['payload_review'].values())
assert all(v is None for v in config['prerequisites'].values())
constants = {key: ast.literal_eval(assignments[key].value) for key in (
    'ROOT', 'SYSTEM', 'PRIVATE_PYTHON', 'SOURCE_ROOT', 'CLOCK', 'COMPLETED',
    'INITIAL_INPUTS', 'PREREQUISITES', 'OUTPUTS')}
assert constants['COMPLETED'] == config['completed_files']
assert constants['INITIAL_INPUTS'] == config['initial_inputs']
assert constants['ROOT'] == config['evidence_root']
assert sha(canonical(constants['COMPLETED']) + b'\n') == config['manifest_sha256']
assert len(constants['OUTPUTS']) == 6 and len(set(constants['OUTPUTS'])) == 6
for name, value in {'NS': 1000000000, 'FILE_CAP': 8388608, 'TOTAL_CAP': 33554432,
                    'RAW_CAP': 1048576, 'CONSOLE_CAP': 16384,
                    'ENTRY_CAP': 4096, 'DEPTH_CAP': 4}.items():
    assert ast.literal_eval(assignments[name].value) == value
assert ast.unparse(assignments['INITIAL_CAP'].value) == 'TOTAL_CAP - 16777216 - 4194304 - 65536'
assert 33554432 - 16777216 - 4194304 - 65536 == 12517376

# Read public Git/source data only; no private path declared by inputs is opened.
input_paths = [R3 + '/' + name for name in (
    'metadata-caller.py.txt', 'README.md', 'source-parent.template.json',
    'parent-boundary.schema.json', 'completed-source-bindings.json',
    'input-pins.json', 'command-vectors.json', 'bindings.json', 'closed-cli.json')]
input_paths += [R4 + '/' + name for name in ('REVIEW.md', 'findings.json')]
input_paths += [PREP + '/' + name for name in (
    'README.md', 'next-operation.json', 'metadata.recipe.json',
    'qualification.recipe.json', 'file-qualification.recipe.py.txt')]
input_paths += ['docs/verification/m1-sdk-canonical-path-repair-2026-10-25/qualification.recipe.json']
input_paths += [AUDIT + '/' + name for name in ('REVIEW.md', 'saved-records-sanitized.json')]
input_paths += [BRIDGE + '/' + name for name in ('REVIEW.md', 'projection.json', 'source-bindings.json')]
input_paths += ['docs/verification/m1-ordinary-fresh-flow-plan-2026-10-24-t3-5463ce44/' + name
                for name in ('operator-plan.md', 'receipt-contract.md')]
bindings = []
public = {}
for path in input_paths:
    raw = (REPO / path).read_bytes()
    assert raw == git('show', BASE + ':' + path), path
    public[path] = raw
    bindings.append(dict(path=path, ref=BASE, bytes=len(raw), sha256=sha(raw)))
for name in ('README.md', 'size-bound.json', 'minimum-shape.json'):
    path = SIZE + '/' + name
    raw = git('show', '0b06ca9:' + path)
    public[path] = raw
    bindings.append(dict(path=path, ref='0b06ca9', bytes=len(raw), sha256=sha(raw)))

parent = parse_json(public[R3 + '/source-parent.template.json'])
assert parent['source_file_qualification']['retained_files'] == constants['COMPLETED']
assert parent['clock'] == constants['CLOCK']
pins = parse_json(public[R3 + '/input-pins.json'])['files']
assert len(pins) == 17
for name, pin in pins.items():
    raw = (REPO / pin['repository_path']).read_bytes()
    assert len(raw) == pin['bytes'] and sha(raw) == pin['sha256']
    assert constants['INITIAL_INPUTS'][name] == {k: pin[k] for k in ('bytes', 'sha256')}
    bindings.append(dict(path=pin['repository_path'], ref=BASE, bytes=len(raw), sha256=sha(raw)))
for name in ('input-pins.json', 'command-vectors.json'):
    raw = public[R3 + '/' + name]
    assert constants['INITIAL_INPUTS'][name] == {'bytes': len(raw), 'sha256': sha(raw)}
assert len(constants['INITIAL_INPUTS']) == 20
recipe = parse_json(public[PREP + '/metadata.recipe.json'])
vectors = parse_json(public[R3 + '/command-vectors.json'])['commands']
assert len(vectors) == len({v['label'] for v in vectors}) == 50
next_operation = parse_json(public[PREP + '/next-operation.json'])
startup_node = next(n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name == 'startup')
startup_call = next(n for n in ast.walk(startup_node) if isinstance(n, ast.Return)).value
assert {k.arg for k in startup_call.keywords} == set(next_operation['parent_startup'])
assert len(startup_call.keywords) == 10
def fixed_string(node):
    # Interpret only closed literal string syntax as data, never an AST call.
    if isinstance(node, ast.Constant) and type(node.value) is str:
        return node.value
    if isinstance(node, ast.Name) and node.id in constants:
        assert type(constants[node.id]) is str
        return constants[node.id]
    assert isinstance(node, ast.BinOp) and isinstance(node.op, ast.Add)
    return fixed_string(node.left) + fixed_string(node.right)
startup_values = {k.arg: fixed_string(k.value) for k in startup_call.keywords}
assert startup_values == {k: v.replace('${EVIDENCE}', constants['ROOT'])
                          for k, v in next_operation['parent_startup'].items()}
popen = next(n for n in ast.walk(tree) if isinstance(n, ast.Call)
             and ast.unparse(n.func) == 'subprocess.Popen')
assert len(popen.args) == 1 and ast.unparse(popen.args[0]) == 'argv'
collect_node = next(n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name == 'collect')
argv_node = next(n for n in collect_node.body if isinstance(n, ast.Assign)
                 and isinstance(n.targets[0], ast.Name) and n.targets[0].id == 'argv')
assert isinstance(argv_node.value, ast.List)
argv_nodes = argv_node.value.elts
assert [fixed_string(n) for n in argv_nodes[:7]] == [
    '/usr/bin/python3', '-I', '-S', '-B', constants['ROOT'] + '/metadata-caller.py.txt',
    'source', constants['ROOT']]
assert len(argv_nodes) == 9 and ast.unparse(argv_nodes[7]) == 'str(started)'
assert ast.unparse(argv_nodes[8]) == 'parent_sha'
assert {k.arg: ast.unparse(k.value) for k in popen.keywords} == dict(
    stdin='subprocess.DEVNULL', stdout='subprocess.PIPE', stderr='subprocess.PIPE',
    cwd='ROOT', env='startup()', shell='False', close_fds='True', bufsize='0')

# The corrected saved size is an actual JSON byte count, not a behavioral model.
shape_raw = public[SIZE + '/minimum-shape.json']
shape = parse_json(shape_raw)
size_bound = parse_json(public[SIZE + '/size-bound.json'])
assert len(shape_raw) == len(canonical(shape) + b'\n') == 209116
assert sha(shape_raw) == size_bound['minimum_shape_sha256']
assert size_bound['minimum_excess_bytes'] == 209116 - 131072 == 78044
assert len(shape['native_records']) == 50
assert len(shape['collected_commands']) == 50
for item in shape['collected_commands']:
    assert 'stdout' not in item['collector_status'] and 'stderr' not in item['collector_status']
caller_tree = ast.parse(public[R3 + '/metadata-caller.py.txt'], feature_version=(3, 9))
perform = next(n for n in caller_tree.body if isinstance(n, ast.FunctionDef) and n.name == 'perform')
perform_nodes = list(ast.walk(perform))
# Locate static copy before later publication mutates the original record.
assert any(isinstance(n, ast.keyword) and n.arg == 'collector_status'
           and ast.unparse(n.value) == 'dict(record)' for n in perform_nodes)

functions = []
def inventory_functions(nodes, scope=''):
    for node in nodes:
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            name = scope + node.name
            raw_function = ''.join(text.splitlines(keepends=True)[node.lineno - 1:node.end_lineno]).encode('utf-8')
            functions.append(dict(name=name, first_line=node.lineno, last_line=node.end_lineno,
                                  bytes=len(raw_function), sha256=sha(raw_function),
                                  docstring=ast.get_docstring(node)))
            inventory_functions(node.body, name + '.')
        elif isinstance(node, ast.ClassDef):
            inventory_functions(node.body, scope + node.name + '.')
inventory_functions(tree.body)
write_json('function-inventory.json', dict(schema='openergy.static-function-inventory.v1',
    execution=False, source_sha256=sha(source), functions=functions,
    empty_marker_class='Stop', payload_calls_run=0))

calls = [ast.unparse(n.func) for n in ast.walk(tree) if isinstance(n, ast.Call)]
assert calls.count('subprocess.Popen') == 1
assert calls.count('os.write') == 2  # One file writer; one console attempt.
assert calls.count('proc.kill') == 1
assert not any(call in calls for call in (
    'eval', 'exec', 'compile', '__import__', 'subprocess.run', 'subprocess.call',
    'subprocess.check_output', 'os.system', 'os.unlink', 'os.remove', 'os.rename',
    'os.replace', 'os.makedirs', 'os.mkdir', 'os.kill', 'os.killpg',
    'sys.stdout.buffer.write', 'sys.stdout.buffer.flush', 'proc.communicate'))
imports = [n.module for n in tree.body if isinstance(n, ast.ImportFrom)]
assert imports == []
assert not any(isinstance(n, ast.Import) and any(
    a.name in ('importlib', 'ctypes', 'socket') for a in n.names) for n in tree.body)
assert calls.index('os.setpriority') < calls.index('subprocess.Popen')
assert 'os._exit(exit_code)' in text and 'sys.exit(' not in text
assert 'O_EXCL' in text and 'O_NOFOLLOW' in text and 'bufsize=0' in text
assert 'end - 2 * NS' in text and 'end - 1500000000' in text

# Exact full add-file patch, statically reconstructed in memory only.
relative = str(FOLDER.relative_to(REPO) / 'source-payload.py.txt')
patch = ''.join(difflib.unified_diff([], text.splitlines(keepends=True),
                                   fromfile='/dev/null', tofile=relative, n=0)).encode('utf-8')
(FOLDER / 'payload.patch').write_bytes(patch)
patch_lines = patch.splitlines(keepends=True)
assert patch_lines[0].startswith(b'--- ') and patch_lines[1].startswith(b'+++ ')
assert patch_lines[2].startswith(b'@@ -0,0 +1,')
assert all(line.startswith(b'+') for line in patch_lines[3:])
assert b''.join(line[1:] for line in patch_lines[3:]) == source

write_json('source-bindings.json', dict(schema='openergy.source-payload-bindings.v1',
    execution=False, base_commit=BASE, public_inputs=bindings,
    private_inputs_read=False, source_sha256=sha(source), source_bytes=len(source),
    config_canonical_sha256=sha(canonical(config)), final_caller_candidate=None,
    final_caller_sha256=None, final_caller_review=None, payload_independent_review=None,
    actual_rendered_payload_sha256=None,
    identity_rule='reviewed unrendered template hash + canonical CONFIG hash; local transport separately binds full rendered raw hash'))

# The source enforces the16KiB encoded cap before its one console write.
# This is a structural check, not a measured delivery or all-failures size proof.
# No stdout/stderr/input data or full caller JSON is embedded in the envelope.
checks = dict(
    python39_ast_parsed_as_data=True, candidate_imported_or_executed=False,
    supplied_ast_evaluated_or_compiled=False, json_duplicate_keys_rejected_by_static_reader=True,
    config_closed_schema_matches_embedded_template=True,
    unresolved_binding_fields=sum(v is None for v in config['caller'].values())
        + sum(v is None for v in config['payload_review'].values())
        + sum(v is None for v in config['prerequisites'].values()) + 1,
    final_nulls_not_runtime_admissible=True, completed_manifest_hash_exact=True,
    fixed_initial_inputs=20, public_pins_verified=17, vectors=50,
    exact_startup_key_count=10, all_startup_values_equal_next_operation=True,
    exact_system_source_launch_vector=True, console_runtime_guard_bytes=16384,
    console_delivery_or_all_failure_size_proven=False, popen_sites=1, direct_kill_sites=1,
    os_write_sites=2, file_writer_exclusive_unbuffered=True,
    no_copy_transport_reservation_claim=True, unchanged_existing_files=True,
    corrected_retained_minimum_json_bytes=209116, minimum_excess_bytes=78044,
    corrected_collector_copy_has_no_raw_refs=True,
    add_file_patch_reconstructs_source=True, source_lines=text.count('\n'),
    source_bytes=len(source), source_sha256=sha(source),
    patch_sha256=sha(patch), function_count=len(functions),
    runtime='UNVERIFIED', independent_review='REQUIRED_BEFORE_USE')
write_json('static-checks.json', dict(schema='openergy.source-payload-static-checks.v1',
    execution=False, checks=checks, limits=[
        'AST/JSON/hash/patch only; no runtime/behavior/deadline proof',
        'no tests/mocks, native/remote/process query or private reads/writes',
        'parent causal inspection/exclusive ownership/substitution proof external',
        'R5 caller and payload independent CLEAR reviews pending',
        'COPY/prepare/controller/product/artifact/route integration absent']))

for path in FOLDER.glob('*.json'):
    parse_json(path.read_bytes())
assert not any(p.stat().st_mode & 0o111 for p in FOLDER.iterdir())
manifest = []
for path in sorted(FOLDER.iterdir()):
    if path.name != 'SHA256SUMS':
        manifest.append(sha(path.read_bytes()) + '  ' + path.name + '\n')
(FOLDER / 'SHA256SUMS').write_text(''.join(manifest))
print(json.dumps({'source_bytes': len(source), 'source_sha256': sha(source),
                  'functions': len(functions), 'public_pins': 17,
                  'static_checks': 'PASS', 'execution': False,
                  'independent_review': 'REQUIRED_BEFORE_USE'}, sort_keys=True))
