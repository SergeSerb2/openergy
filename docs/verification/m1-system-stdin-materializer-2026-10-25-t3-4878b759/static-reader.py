"""Author-owned static data reader. Never run/import the supplied materializer.

Only local tracked public contracts and this owned folder are read. Generated
records are AST/JSON/hash/patch observations, not tests or behavior evidence.
"""
import ast
import hashlib
import json
import pathlib
import re
import sys

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[2]
SOURCE_NAME = 'materializer.py.txt'
SOURCE = (HERE / SOURCE_NAME).read_bytes()
TEXT = SOURCE.decode('ascii')
TREE = ast.parse(TEXT, filename=SOURCE_NAME)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=True,
                      allow_nan=False).encode('ascii')


def strict_json(data):
    def pairs(items):
        result = {}
        for key, value in items:
            if key in result:
                raise ValueError('duplicate JSON key')
            result[key] = value
        return result
    def invalid(_value):
        raise ValueError('nonfinite JSON number')
    return json.loads(data, object_pairs_hook=pairs, parse_constant=invalid)


def save(name, value):
    (HERE / name).write_bytes(json.dumps(value, sort_keys=True, indent=2,
                                       allow_nan=False).encode('ascii') + b'\n')


def literal(name):
    nodes = [n.value for n in TREE.body if isinstance(n, ast.Assign)
             and len(n.targets) == 1 and isinstance(n.targets[0], ast.Name)
             and n.targets[0].id == name]
    if len(nodes) != 1:
        raise ValueError('not one assignment: ' + name)
    return ast.literal_eval(nodes[0])


def schema_bound(spec):
    # Return exact maximum canonical JSON length and number of unescaped quote
    # characters. The closed schema's string alphabets contain no quotes or \.
    if 'const' in spec:
        raw = canonical(spec['const'])
        return len(raw), raw.count(b'"')
    kind = spec['type']
    if kind == 'object':
        pairs = []
        quotes = 0
        for key, child in spec['properties'].items():
            size, q = schema_bound(child)
            key_bytes = canonical(key)
            pairs.append(len(key_bytes) + 1 + size)
            quotes += key_bytes.count(b'"') + q
        return 2 + sum(pairs) + max(0, len(pairs) - 1), quotes
    if kind == 'integer':
        return len(str(spec['maximum'])), 0
    if kind == 'string':
        width = spec.get('maxLength')
        if width is None:
            width = int(re.search(r'\{([0-9]+)\}', spec['pattern']).group(1))
        return width + 2, 2
    raise ValueError('unbounded schema kind')


def full_name(node):
    if isinstance(node, ast.Name):
        return node.id
    if isinstance(node, ast.Attribute):
        return full_name(node.value) + '.' + node.attr
    return '<expression>'


config = strict_json((HERE / 'config.template.json').read_bytes())
schema = strict_json((HERE / 'config.schema.json').read_bytes())
baseline_dir = REPO / 'docs/verification/m1-source-stdin-payload-kill-gate-repair-2026-10-25-t3-da89534d'
baseline = strict_json((baseline_dir / 'config.template.json').read_bytes())
inspection_path = REPO / 'docs/verification/m1-completed-source-parent-inspection-2026-10-25/inspection.json'
inspection = strict_json(inspection_path.read_bytes())
checks = {}


def observe(name, ok):
    checks[name] = bool(ok)
    if not ok:
        raise ValueError('static observation failed: ' + name)


observe('AST_parse_only_no_executable_code_object', isinstance(TREE, ast.Module))
observe('embedded_CONFIG_equals_template', strict_json(literal('CONFIG_TEXT')) == config)
observe('embedded_CONFIG_is_canonical', literal('CONFIG_TEXT').encode('ascii') == canonical(config))
observe('schema_exact_inert_const', schema['oneOf'][0] == {'const': config})
observe('config_execution_false', config['execution'] is False)
observe('all_final_bindings_and_base64_null', config['template_sha256'] is None
        and all(v is None for v in config['caller'].values())
        and all(v is None for v in config['materializer_review'].values())
        and all(v is None for v in config['parent_duties'].values())
        and all(v is None for v in config['inputs'].values()))
observe('fixed_root_exact_public_contract', literal('ROOT') == baseline['evidence_root'])
observe('closed_four_input_names', literal('INPUTS') == (
    'metadata-caller.py.txt', 'input-pins.json', 'command-vectors.json',
    'expected-source-metadata.json') and set(config['inputs']) == set(literal('INPUTS')))
observe('fixed_three_raw_hash_length_bindings', literal('FIXED_INPUTS') == {
    k: baseline['initial_inputs'][k] for k in literal('INPUTS')[1:]})
observe('completed_seven_public_bindings_exact', literal('COMPLETED') == baseline['completed_files']
        == inspection['retained_files'])
observe('completed_manifest_hash', digest(canonical(literal('COMPLETED')) + b'\n')
        == literal('COMPLETED_MANIFEST_SHA256') == inspection['manifest_sha256'])
observe('parent_duties_closed_and_null', set(config['parent_duties']) == set(literal('PARENT_DUTIES'))
        and all(v is True for v in literal('PARENT_DUTIES').values()))
live = schema['oneOf'][1]


def closed_objects(value):
    if isinstance(value, dict):
        if value.get('type') == 'object':
            if value.get('additionalProperties') is not False or set(value['required']) != set(value['properties']):
                return False
        return all(closed_objects(v) for v in value.values())
    if isinstance(value, list):
        return all(closed_objects(v) for v in value)
    return True


observe('all_schema_objects_closed', closed_objects(schema))
observe('schema_has_no_other_path_name_argv_slot', set(live['properties']) == set(config)
        and set(live['properties']['inputs']['properties']) == set(literal('INPUTS'))
        and live['properties']['evidence_root'] == {'const': literal('ROOT')})
assign = next(n for n in TREE.body if isinstance(n, ast.Assign)
              and isinstance(n.targets[0], ast.Name) and n.targets[0].id == 'CONFIG_TEXT')
token = assign.value
lines = TEXT.splitlines(keepends=True)
start = sum(map(len, lines[:token.lineno - 1])) + token.col_offset
end = sum(map(len, lines[:token.end_lineno - 1])) + token.end_col_offset
original_token = TEXT[start:end]
observe('single_string_literal_on_one_line', isinstance(token, ast.Constant)
        and type(token.value) is str and token.lineno == token.end_lineno
        and TEXT.count('# BEGIN CLOSED CONFIG') == TEXT.count('# END CLOSED CONFIG') == 1)
observe('exact_null_token_reconstruction', TEXT[:start] + original_token + TEXT[end:] == TEXT
        and original_token == json.dumps(canonical(config).decode('ascii'), ensure_ascii=True))
max_config, config_quotes = schema_bound(live)
max_token = 2 + max_config + config_quotes
rendered_bound = len(SOURCE) - len(original_token.encode('ascii')) + max_token
observe('worst_closed_schema_rendered_payload_within_1MiB', rendered_bound <= literal('PAYLOAD_CAP'))

# Pure JSON size calculation. These descriptors are not a runnable source,
# caller fixture, config binding or mocked syscall/behavior result.
console_descriptor = {'v': 1, 'r': [16384, '0' * 64], 'f': [
    [524288, '0' * 64], [4920, '0' * 64], [12370, '0' * 64], [142587, '0' * 64]]}
console_bound = len(canonical(console_descriptor) + b'\n')
observe('compact_console_maximum_within512', console_bound <= literal('CONSOLE_CAP') == 512)
observe('INITIAL_reservation_arithmetic', 33554432 - 16777216 - 4194304 - 65536 == 12517376)
imports = sorted({n.names[0].name for n in ast.walk(TREE) if isinstance(n, ast.Import)})
observe('standard_library_imports_only', imports == sorted([
    'os', 'sys', 'time', 'base64', 'hashlib', 'json', 'math', 'stat'])
    and not any(isinstance(n, ast.ImportFrom) for n in ast.walk(TREE)))
calls = [full_name(n.func) for n in ast.walk(TREE) if isinstance(n, ast.Call)]
forbidden = {'exec', 'eval', 'compile', '__import__', 'subprocess.Popen', 'os.system',
             'os.popen', 'os.execv', 'os.execve', 'os.spawnv', 'os.fork',
             'os.unlink', 'os.remove', 'os.rename', 'os.replace', 'os.mkdir'}
observe('no_dynamic_code_process_launcher_or_path_repair_calls', not (set(calls) & forbidden))
observe('no_print_buffered_output_or_stdin_reader_calls', not any(
    c in {'print', 'input', 'sys.stdout.write', 'sys.stderr.write', 'sys.stdin.read', 'open'} for c in calls))
write_nodes = [n for n in ast.walk(TREE) if isinstance(n, ast.Call)
               and full_name(n.func) == 'call' and n.args
               and full_name(n.args[0]) == 'os.write']
observe('exactly_two_raw_write_sites_file_and_console', len(write_nodes) == 2)
observe('no_finally_after_failure', not any(isinstance(n, ast.Try) and n.finalbody for n in ast.walk(TREE)))
main = next(n for n in TREE.body if isinstance(n, ast.FunctionDef) and n.name == 'main')
main_text = ''.join(lines[main.lineno - 1:main.end_lineno])
observe('preflight_count_precedes_guard_then_inputs_then_receipt_then_console',
        main_text.index('for name in NEW_NAMES') < main_text.index('initial = inventory')
        < main_text.index('guard_ref = put') < main_text.index('refs = {name: put')
        < main_text.index('receipt_ref = put') < main_text.index('emit_once'))
observe('literal_shared_absolute20s_no_renewal', TEXT.count('DATA_END =') == 1
        and 'DATA_END = DATA_START + 20000000000' in TEXT
        and 'DATA_START <= now < DATA_END' in TEXT)
observe('console_preflight_before_guard', main_text.index('console_admission()')
        < main_text.index('guard_ref = put'))
observe('complete_source_is_nonexecutable_file', (HERE / SOURCE_NAME).stat().st_mode & 0o777 == 0o644)

purposes = {
    'require': 'Stop on a failed source predicate.',
    'check': 'Observe the shared original DATA deadline; equality is late.',
    'call': 'Check immediately before and after one operation; never renew time.',
    'encoded': 'Canonical bounded evidence serialization with deadline checks.',
    'sha': 'Raw data SHA-256 with deadline checks.',
    'strict_json': 'Parse data with duplicate and nonfinite rejection.',
    'strict_json.pairs': 'Reject duplicate object keys.',
    'strict_json.invalid': 'Reject nonfinite JSON numbers.',
    'same_data': 'Compare data without bool/int type equivalence.',
    'hex_string': 'Constrain exact lowercase hash and commit widths.',
    'bind_config': 'Reject null or nonclosed final config before private root access.',
    'decode_inputs': 'Decode only four fixed base64 byte slots; verify hashes and lengths.',
    'stamp': 'Capture stable linked metadata for FD/path identity checks.',
    'open_root': 'Open and authenticate only the exact existing canonical root.',
    'read_leaf': 'Read exact saved completed regular data with bounded nofollow checks.',
    'inspect_completed': 'Authenticate five saved remote records; do not requalify source.',
    'absent': 'Reject existing targets or ledger before any write.',
    'inventory': 'Count bounded INITIAL evidence under the fixed root only.',
    'inventory.walk': 'Enforce finite owned directories and regular-file reservation counting.',
    'put': 'Charge exact bytes and create one exclusive0600 file with barriers.',
    'console_admission': 'Require nonblocking FIFO and sufficient PIPE_BUF before file writes.',
    'emit_once': 'One compact nonblocking raw receipt write; no retry.',
    'main': 'Order admission, fixed preflight, once guard, four writes, receipt and console.',
}
inventory = []


def visit(nodes, prefix=''):
    for node in nodes:
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            name = prefix + node.name
            raw = ''.join(lines[node.lineno - 1:node.end_lineno]).encode('ascii')
            inventory.append({'name': name, 'first_line': node.lineno, 'last_line': node.end_lineno,
                              'bytes': len(raw), 'sha256': digest(raw), 'purpose': purposes[name]})
            visit(node.body, name + '.')


visit(TREE.body)
observe('all_function_purposes_bound', {x['name'] for x in inventory} == set(purposes))
save('function-inventory.json', {'schema': 'openergy.materializer-function-inventory.v1',
     'execution': False, 'range_convention': 'complete physical lines including final LF',
     'source_sha256': digest(SOURCE), 'functions': inventory,
     'module_controls': ['first shared DATA sample and own NI15', 'checked SYSTEM stdlib imports',
                         'single closed CONFIG_TEXT literal', 'fixed public constants',
                         'first-error latch then raw exit; no failure publisher']})

patch = ('--- /dev/null\n+++ b/' + str(HERE.relative_to(REPO) / SOURCE_NAME) + '\n'
         + '@@ -0,0 +1,' + str(len(lines)) + ' @@\n' + ''.join('+' + line for line in lines)).encode('ascii')
(HERE / 'materializer.patch').write_bytes(patch)
reconstructed = b''.join(line[1:] for line in patch.splitlines(keepends=True)[3:])
observe('exact_add_file_patch_from_empty', reconstructed == SOURCE)

public_paths = [
 ('next_reviewed_flow', 'docs/verification/m1-next-reviewed-ordinary-flow-2026-10-25-t3-a8a4ece5/README.md'),
 ('metadata_recipe', 'docs/verification/m1-next-reviewed-ordinary-flow-2026-10-25-t3-a8a4ece5/metadata.recipe.json'),
 ('operator_plan', 'docs/verification/m1-ordinary-fresh-flow-plan-2026-10-24-t3-5463ce44/operator-plan.md'),
 ('receipt_contract', 'docs/verification/m1-ordinary-fresh-flow-plan-2026-10-24-t3-5463ce44/receipt-contract.md'),
 ('fa72_contract', 'docs/verification/m1-source-stdin-payload-template-2026-10-25-t3-65b09134/README.md'),
 ('fa72_source', 'docs/verification/m1-source-stdin-payload-template-2026-10-25-t3-65b09134/source-payload.py.txt'),
 ('f78_source', str((baseline_dir / 'source-payload.py.txt').relative_to(REPO))),
 ('f78_config', str((baseline_dir / 'config.template.json').relative_to(REPO))),
 ('f78_contract', str((baseline_dir / 'repair-contract.json').relative_to(REPO))),
 ('parent_inspection', str(inspection_path.relative_to(REPO))),
 ('original_18_input_index', 'docs/verification/m1-canonical-source-file-audit-2026-10-25-t3-ee636b1e/input-index.json'),
 ('basis_bridge_projection', 'docs/verification/m1-metadata-basis-bridge-2026-10-25-t3-755a8865/projection.json'),
 ('transport_R4_readme', 'docs/verification/m1-metadata-transport-r4-2026-10-25-t3-53ab1856/README.md'),
 ('transport_R4_profile', 'docs/verification/m1-metadata-transport-r4-2026-10-25-t3-53ab1856/profile.template.json'),
 ('transport_R4_private_template', 'docs/verification/m1-metadata-transport-r4-2026-10-25-t3-53ab1856/bindings.template.json'),
 ('transport_R4_parent_interface', 'docs/verification/m1-metadata-transport-r4-2026-10-25-t3-53ab1856/parent-observer.contract.json'),
 ('transport_independent_clear', 'docs/verification/m1-metadata-transport-r5-review-2026-10-25-t3-2901f30d/REVIEW.md'),
]
input_refs = []
for role, path in public_paths:
    data = (REPO / path).read_bytes()
    input_refs.append({'role': role, 'path': path, 'bytes': len(data), 'sha256': digest(data),
                       'git_blob': hashlib.sha1(
                           b'blob ' + str(len(data)).encode('ascii') + b'\0' + data).hexdigest(),
                       'read_as': 'tracked public source/data only'})
owned_refs = []
for name in ['README.md', SOURCE_NAME, 'config.template.json', 'config.schema.json',
             'finite-plan.json', 'materialization.contract.json', 'function-inventory.json',
             'materializer.patch', 'static-reader.py']:
    data = (HERE / name).read_bytes()
    owned_refs.append({'path': name, 'bytes': len(data), 'sha256': digest(data)})
save('bindings.json', {
 'schema': 'openergy.system-stdin-materializer-bindings.v1', 'execution': False,
 'requested_parent': '81252214f7c391c465510193acf48115ec3d95db',
 'starting_checkout': '263c75859ab643ef0e5b435b3f81bd9ffe6739c4',
 'only_owned_folder': str(HERE.relative_to(REPO)), 'public_input_snapshot_commit': '81252214f7c391c465510193acf48115ec3d95db',
 'source_lineage': {'fa72': 'fa72a0d865f12ae94b91f84400a31f51026c5b16',
                    'f78': 'f78f2be0fe14e5e4cf81990156cb18a1abb5ff62',
                    'parent_inspection': '492f067404649e75ad54e84dc951136220e05a19',
                    'basis_bridge': 'ce7fe4a1af63af88e0b0b2cacab288e4a46a0d13',
                    'transport_R4': '1c46c16b225924d6664d33c06382a3cd05581272',
                    'transport_clear': 'a7e22725cbbf07b5e6ac7e7aa7e7fb0d3dff93b4'},
 'public_inputs': input_refs, 'owned_artifacts': owned_refs,
 'fixed_root': literal('ROOT'), 'fixed_new_inputs': literal('FIXED_INPUTS'),
 'completed_seven_public_bindings': literal('COMPLETED'),
 'unrendered_template_sha256': digest(SOURCE),
 'inert_config_canonical_sha256': digest(canonical(config)),
 'final_caller_binding': None, 'final_caller_review': None, 'materializer_independent_review': None,
 'final_template_binding': None, 'final_parent_duties': None, 'actual_rendered_payload_sha256': None,
 'private_packet_read': False, 'runtime': 'UNVERIFIED'})
json_files = sorted(HERE.glob('*.json'))
for path in json_files:
    strict_json(path.read_bytes())
observe('all_owned_JSON_strict_duplicate_nonfinite_free', True)
save('static-checks.json', {
 'schema': 'openergy.materializer-static-observations.v1', 'execution': False,
 'reader_version': sys.version, 'reader_sha256': digest(pathlib.Path(__file__).read_bytes()),
 'checks': checks, 'check_count': len(checks), 'source_bytes': len(SOURCE),
 'source_lines': len(lines), 'source_sha256': digest(SOURCE), 'functions': len(inventory),
 'public_input_files': len(input_refs), 'imports': imports,
 'reconstruction_literal_offsets_ascii': [start, end],
 'bounds': {'maximum_canonical_live_CONFIG_bytes': max_config,
            'canonical_live_CONFIG_quotes': config_quotes,
            'maximum_single_string_token_bytes': max_token,
            'maximum_full_rendered_payload_bytes': rendered_bound,
            'transport_payload_cap_bytes': 1048576,
            'rendered_spare_bytes': 1048576 - rendered_bound,
            'maximum_console_bytes_including_LF': console_bound,
            'console_cap_bytes': 512,
            'maximum_four_decoded_input_bytes': 524288 + 159877,
            'full_receipt_reserved_bytes': 16384},
 'size_proof': 'Closed schema length arithmetic and pure JSON descriptors only. No live config, rendered source, caller fixture or behavior simulation constructed.',
 'patch_sha256': digest(patch), 'bindings_sha256': digest((HERE / 'bindings.json').read_bytes()),
 'supplied_source_execution_or_import': False, 'runtime_proof': False,
 'independent_review': False, 'end_to_end_not_run': 'Explicit SOURCE/DATA ONLY scope forbids source entry, private access and remote/native execution.',
 'proof_limits': ['No behavior, timing, syscall fault, real pipe delivery, OS exit/EOF, actual NI15 continuity or macOS durability proof.',
                  'Parent must prove actual reviewed bytes and exact one-token reconstruction for the future rendered payload.',
                  'Source ordering scans are static observations, not a control-flow or concurrency proof.',
                  'No native qualification, SOURCE120 start, artifact or release acceptance.']})
for path in HERE.iterdir():
    if path.is_file() and path.suffix == '.json':
        strict_json(path.read_bytes())
sums = ''.join(digest(p.read_bytes()) + '  ' + p.name + '\n'
               for p in sorted(HERE.iterdir()) if p.is_file() and p.name != 'SHA256SUMS')
(HERE / 'SHA256SUMS').write_text(sums)
print(json.dumps({'static_checks': len(checks), 'all_observations_true': all(checks.values()),
                  'source_bytes': len(SOURCE), 'functions': len(inventory),
                  'maximum_rendered_bytes': rendered_bound, 'maximum_console_bytes': console_bound,
                  'supplied_code_entered': False}))
