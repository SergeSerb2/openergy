"""Reviewer-owned bounded data reader. Never runs or imports supplied source.

Only the four explicit private leaves below are readable. Results contain
hashes, shapes, booleans and public command labels, never private payloads.
"""
import ast
import hashlib
import json
import os
from pathlib import Path
import stat
import sys

OUT = Path(__file__).parent
PREP = Path('docs/verification/m1-next-reviewed-ordinary-flow-2026-10-25-t3-a8a4ece5')
CALLER = Path('docs/verification/m1-artifact-metadata-caller-2026-10-25-t3-43cef465')
AUDIT = Path('docs/verification/m1-canonical-source-file-audit-2026-10-25-t3-ee636b1e')
STATE = '/Users/serge/.local/state/openergy-verification/'
OLD = STATE + 'm1-resume-private-20261024/'
ALLOWED = {
    'wrapper': (STATE + 'm1-reviewed-ordinary-canonical-sdk-prepared-20261025-4b20357c1503/expected-source-metadata.json', 142587, '1670da99b768e01de3fc4ecd4d644fba7031ea09e55d97c1395290bfb681c05f'),
    'source': (OLD + 'ordinary-artifact-qualification-8d1e4fee522f/retained/artifact-observations.json', 151715, '6f7531f7dd28ca9672bc206bf1c511fe706005904f1cfd5ffa9694dfa8b8c48a'),
    'supplement': (OLD + 'ordinary-artifact-qualification-8d1e4fee522f/retained/supplement.json', 40212, 'c541d45f40e85e7ec7d652a8b98a34d151e9e8bd133aed58f99ba5090cfad15a'),
    'completion_launcher': (OLD + 'ordinary-flow-inputs-aa9357b7f58a/executed-copy-completion-launcher.py.txt', 178339, '9caba2db9055004565521f41a104e5425193735999213e11947b50e0741cd6ab'),
}


def save(name, value):
    (OUT / name).write_text(json.dumps(value, indent=2, sort_keys=True, ensure_ascii=True) + '\n')


ledger_path = OUT / 'private-read-ledger.json'
ledger = json.loads(ledger_path.read_text()) if ledger_path.exists() else {
    'regular_open_limit': 32, 'content_byte_limit': 8388608,
    'regular_opens': 0, 'content_bytes': 0, 'events': [],
    'private_writes': 0, 'unlisted_private_reads': 0,
}


def need(ok, key):
    if not ok:
        raise ValueError(key)


def signature(s):
    return [s.st_dev, s.st_ino, s.st_mode, s.st_uid, s.st_gid,
            s.st_size, s.st_mtime_ns, s.st_ctime_ns, s.st_nlink]


def private_read(alias):
    path, size, digest = ALLOWED[alias]
    need(ledger['regular_opens'] < 32, 'open-budget')
    need(ledger['content_bytes'] + size + 1 <= 8388608, 'byte-budget')
    handles = []
    leaf = None
    event = {'alias': alias, 'expected_bytes': size, 'expected_sha256': digest,
             'bytes_read': 0, 'complete': False}
    try:
        parent = os.open('/', os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
        handles.append(parent)
        parts = Path(path).parts[1:]
        for component in parts[:-1]:
            parent = os.open(component, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=parent)
            handles.append(parent)
        initial = os.stat(parts[-1], dir_fd=parent, follow_symlinks=False)
        need(stat.S_ISREG(initial.st_mode) and initial.st_size == size, 'regular-size-binding')
        leaf = os.open(parts[-1], os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=parent)
        ledger['regular_opens'] += 1
        ledger['events'].append(event)
        save('private-read-ledger.json', ledger)
        before = os.fstat(leaf)
        need(signature(before) == signature(initial), 'pre-open-stability')
        chunks = []
        remaining = size + 1
        while remaining:
            block = os.read(leaf, min(65536, remaining))
            if not block:
                break
            chunks.append(block)
            remaining -= len(block)
            event['bytes_read'] += len(block)
            ledger['content_bytes'] += len(block)
            save('private-read-ledger.json', ledger)
        data = b''.join(chunks)
        after = os.fstat(leaf)
        linked = os.stat(parts[-1], dir_fd=parent, follow_symlinks=False)
        stable = signature(initial) == signature(before) == signature(after) == signature(linked)
        got = hashlib.sha256(data).hexdigest()
        event.update(actual_sha256=got, stable_metadata=stable,
                     no_symlink_components=True, regular=True,
                     mode=oct(stat.S_IMODE(after.st_mode)))
        need(stable and len(data) == size and got == digest, 'content-binding')
        event['complete'] = True
        save('private-read-ledger.json', ledger)
        return data
    finally:
        if leaf is not None:
            os.close(leaf)
        for handle in reversed(handles):
            os.close(handle)
        save('private-read-ledger.json', ledger)


def load_json(data):
    def unique_pairs(pairs):
        result = {}
        for key, value in pairs:
            need(key not in result, 'duplicate-json-key')
            result[key] = value
        return result
    return json.loads(data.decode('utf-8', errors='strict'), object_pairs_hook=unique_pairs)


def canonical(obj):
    return json.dumps(obj, sort_keys=True, separators=(',', ':'), ensure_ascii=True).encode('utf-8')


def sha(data):
    return hashlib.sha256(data).hexdigest()


def shape(obj):
    if isinstance(obj, dict):
        return {k: {'type': type(v).__name__, 'count': len(v) if isinstance(v, (dict, list)) else None} for k, v in obj.items()}
    return {'type': type(obj).__name__, 'count': len(obj) if isinstance(obj, list) else None}


def static_assignments(text):
    tree = ast.parse(text, mode='exec')  # syntax tree as data only
    assignments = []
    strings = []
    for node in tree.body:
        if isinstance(node, ast.Assign):
            names = [t.id for t in node.targets if isinstance(t, ast.Name)]
            if isinstance(node.value, (ast.Constant, ast.Dict, ast.List, ast.Tuple)):
                try:
                    value = ast.literal_eval(node.value)
                except (ValueError, TypeError):
                    continue
                assignments.append({'names': names, 'line': node.lineno, 'shape': shape(value)})
                if isinstance(value, str):
                    strings.append((names, node.lineno, value))
    return tree, assignments, strings


def inspect(data):
    objs = {k: load_json(data[k]) for k in ('wrapper', 'source', 'supplement')}
    summary = {k: {'shape': shape(v), 'canonical_sha256': sha(canonical(v)),
                   'first_command_shape': shape(v['commands'][0]) if isinstance(v, dict) and 'commands' in v else None}
               for k, v in objs.items()}
    tree, assignments, strings = static_assignments(data['completion_launcher'].decode('utf-8', errors='strict'))
    nested = []
    for names, line, value in strings:
        if 'EXPECTED' in value:
            try:
                inner_tree, inner, _ = static_assignments(value)
                targets = []
                for node in ast.walk(inner_tree):
                    if isinstance(node, ast.Assign) and any(isinstance(t, ast.Name) and t.id == 'EXPECTED' for t in node.targets):
                        target = {'line': node.lineno, 'rhs_type': type(node.value).__name__}
                        if isinstance(node.value, ast.Call):
                            target['function'] = ast.unparse(node.value.func)
                            target['args'] = [{'type': type(a).__name__, 'literal_type': type(a.value).__name__ if isinstance(a, ast.Constant) else None,
                                               'literal_length': len(a.value) if isinstance(a, ast.Constant) and isinstance(a.value, str) else None} for a in node.value.args]
                            if ast.unparse(node.value.func) == 'json.loads' and len(node.value.args) == 1 and not node.value.keywords and isinstance(node.value.args[0], ast.Constant) and isinstance(node.value.args[0].value, str):
                                expected = load_json(node.value.args[0].value.encode('utf-8'))
                                target['decoded_shape'] = shape(expected)
                                target['canonical_sha256'] = sha(canonical(expected))
                                target['children'] = ({k: shape(v[0]) if isinstance(v, list) and v else shape(v) for k, v in expected.items()}
                                                      if isinstance(expected, dict) else [shape(v) for v in expected[:1]])
                        targets.append(target)
                nested.append({'outer_names': names, 'outer_line': line, 'assignments': inner, 'expected_assignments': targets})
            except SyntaxError:
                nested.append({'outer_names': names, 'outer_line': line, 'not_python_syntax': True})
    save('inspection.json', {'objects': summary, 'nested_expected_candidates': nested})


def extract_expected(launcher):
    outer, _, strings = static_assignments(launcher.decode('utf-8', errors='strict'))
    code = [(line, value) for names, line, value in strings if names == ['code']]
    need(len(code) == 1, 'one-code-literal')
    inner = ast.parse(code[0][1], mode='exec')
    assignments = [n for n in ast.walk(inner) if isinstance(n, ast.Assign)
                   and any(isinstance(t, ast.Name) and t.id == 'EXPECTED' for t in n.targets)]
    need(len(assignments) == 1, 'one-expected-assignment')
    node = assignments[0]
    call = node.value
    need(isinstance(call, ast.Call) and ast.unparse(call.func) == 'json.loads'
         and len(call.args) == 1 and not call.keywords
         and isinstance(call.args[0], ast.Constant) and isinstance(call.args[0].value, str), 'expected-literal-json')
    expected = load_json(call.args[0].value.encode('utf-8'))
    need(isinstance(expected, list) and len(expected) == 50, 'expected-list50')
    stores = [n for n in ast.walk(inner) if isinstance(n, ast.Name) and n.id == 'EXPECTED' and isinstance(n.ctx, ast.Store)]
    need(len(stores) == 1, 'expected-one-name-store')
    # Display only the historical comparison functions, with private literals removed.
    class Redact(ast.NodeTransformer):
        def visit_Constant(self, node):
            safe = {'', '-verify', 'label', 'argv', 'exit', 'stdout', 'stderr', 'source_bundle'}
            if isinstance(node.value, str) and node.value not in safe:
                return ast.copy_location(ast.Constant(value='[REDACTED_STRING]'), node)
            return node
    funcs = [n for n in inner.body if isinstance(n, ast.FunctionDef) and 12 <= n.lineno <= 19]
    sanitized = '\n\n'.join(ast.unparse(Redact().visit(n)) for n in funcs) + '\n'
    (OUT / 'historical-comparator-sanitized.txt').write_text(sanitized)
    return expected, {'outer_code_line': code[0][0], 'inner_expected_line': node.lineno,
                      'json_literal_utf8_bytes': len(call.args[0].value.encode('utf-8')),
                      'expected_name_stores': len(stores), 'ast_used_as_data_only': True}


def analyze(data):
    wrapper, source, supplement = [load_json(data[k]) for k in ('wrapper', 'source', 'supplement')]
    recipe = load_json((PREP / 'metadata.recipe.json').read_bytes())
    vectors = load_json((CALLER / 'command-vectors.json').read_bytes())
    pins = load_json((CALLER / 'input-pins.json').read_bytes())
    bindings = load_json((CALLER / 'bindings.json').read_bytes())
    qual = load_json((PREP / 'qualification.recipe.json').read_bytes())
    expected, extraction = extract_expected(data['completion_launcher'])
    checks = {}
    def check(key, value):
        checks[key] = bool(value)
        return bool(value)

    check('historical_expected_declared_canonical_digest', sha(canonical(expected)) == recipe['saved_expected_canonical_json_sha256'] == 'b46d03fec326f76748fb0e181a0bb607ad4cef58198efaa62e762c22336e30e0')
    check('wrapper_canonical_digest_preserved', sha(canonical(wrapper)) == '7fd71953cfbbb71bc194a17412840e4b2420a081e6615e2865cc7abf4103c159')
    check('wrapper_commands_equal_entire_historical_expected', wrapper['commands'] == expected)
    check('wrapper_commands_canonical_equals_historical_digest', sha(canonical(wrapper['commands'])) == sha(canonical(expected)))
    check('distinct_full_wrapper_and_expected_hashes_retained', sha(canonical(wrapper)) != sha(canonical(expected)))
    check('wrapper_closed_keys', set(wrapper) == {'schema', 'bundle', 'commands', 'comparison', 'original_references'})
    check('wrapper_schema', wrapper['schema'] == 'openergy.parent-exact-source-metadata-basis.v1')
    check('wrapper_source_bundle', wrapper['bundle'] == recipe['source_bundle'])
    check('wrapper_comparison_exact_recipe', wrapper['comparison'] == recipe['comparison'])
    check('wrapper_reference_bindings', wrapper['original_references'] == {k: recipe['expected_data'][k] for k in ('source', 'supplement')})
    check('original_source_supplement_hashes_match_recipe', all(recipe['expected_data'][k] == {'path': ALLOWED[k][0], 'bytes': ALLOWED[k][1], 'sha256': ALLOWED[k][2]} for k in ('source', 'supplement')))
    check('completion_launcher_hash_matches_qualification_recipe', qual['saved_algorithms']['copy_comparator'] == {'path': ALLOWED['completion_launcher'][0], 'bytes': ALLOWED['completion_launcher'][1], 'sha256': ALLOWED['completion_launcher'][2]})
    sidecar = {k: v for k, v in wrapper.items() if k != 'commands'}
    check('wrapper_reconstructed_from_expected_and_four_envelope_fields', dict(sidecar, commands=expected) == wrapper)

    ordered = []
    objects = recipe['objects']
    verbs = ['arch', 'verify', 'display', 'entitlements', 'requirements', 'dependencies', 'loadcommands']
    check('declared_verb_order', recipe['verb_order'] == verbs)
    check('declared_object_order', [o['index'] for o in objects] == list(range(7)))
    for obj in objects:
        for verb in verbs:
            replacements = {'${BINARY}': '${BUNDLE}/' + obj['binary'], '${CODE_OBJECT}': '${BUNDLE}' + ('/' + obj['code_object'] if obj['code_object'] else '')}
            ordered.append({'label': 'object-' + str(obj['index']) + '-' + verb,
                            'argv': [replacements.get(token, token) for token in recipe['argv_patterns'][verb]]})
    ordered.append({'label': 'full-bundle-deep-verify', 'argv': recipe['deep_verify']})
    check('independent_expansion_equals_all_closed_vectors_in_order', ordered == vectors['commands'])
    check('closed_50_unique_labels', len(ordered) == len({v['label'] for v in ordered}) == recipe['commands_per_bundle'] == 50)
    check('inert_recipe_and_vectors_and_pins_and_bindings', all(o['execution'] is False for o in (recipe, vectors, pins, bindings)))
    check('supplement_declared_direct_object_mapping', objects[0]['codesign_basis'] == {'file': 'supplement.json', 'first_index': 4, 'verbs': verbs[1:5]}
          and objects[5]['codesign_basis'] == {'file': 'supplement.json', 'first_index': 0, 'verbs': verbs[1:5]})

    source_rows = source['commands']
    source_index = {r['label']: (i, r) for i, r in enumerate(source_rows)}
    check('source_command_labels_unique', len(source_index) == len(source_rows) == 52)
    rows, reconstructed = [], []
    selected_indices = []
    supplement_indices = []
    for row_number, (vector, historical, wrapped) in enumerate(zip(ordered, expected, wrapper['commands'])):
        label = vector['label']
        native_argv = [recipe['source_bundle'] + t[len('${BUNDLE}'):] if t.startswith('${BUNDLE}') else t for t in vector['argv']]
        obj_index = int(label.split('-')[1]) if label.startswith('object-') else None
        verb = label.split('-', 2)[2] if obj_index is not None else None
        if obj_index in (0, 5) and verb in verbs[1:5]:
            index = (4 if obj_index == 0 else 0) + verbs[1:5].index(verb)
            original = supplement['individual_objects'][index]
            supplement_indices.append(index)
            selected = {'label': label, **{k: original[k] for k in ('argv', 'exit', 'stdout', 'stderr')}}
            origin = 'supplement'
            direct_path = recipe['source_bundle'] + '/' + objects[obj_index]['binary']
            direct = original['argv'][-1] == direct_path and original['relative_binary'] == objects[obj_index]['binary']
        else:
            index, original = source_index[label]
            selected_indices.append(index)
            selected = {k: original[k] for k in ('label', 'argv', 'exit', 'stdout', 'stderr')}
            origin = 'source'
            direct = None
        reconstructed.append(selected)
        equal_keys = {k: selected[k] == wrapped[k] == historical[k] for k in ('label', 'argv', 'exit', 'stdout', 'stderr')}
        exits = all(type(r['exit']) is int and r['exit'] == 0 for r in (original, wrapped, historical))
        text_exact = all(type(r[k]) is str and r[k].encode('utf-8', errors='strict') == selected[k].encode('utf-8', errors='strict') for r in (wrapped, historical) for k in ('stdout', 'stderr'))
        rows.append({'row': row_number, 'label': label, 'origin': origin, 'origin_index': index,
                     'whole_argv_matches_expanded_recipe': selected['argv'] == native_argv == historical['argv'] == wrapped['argv'],
                     'actual_saved_exit0_in_origin_wrapper_expected': exits,
                     'all_semantic_fields_exact': all(equal_keys.values()), 'field_match': equal_keys,
                     'raw_utf8_streams_exact': text_exact, 'direct_executable_target': direct,
                     'comparison_rule': 'sorted-complete-utf8-splitlines-per-stream' if label.endswith('-verify') else 'strict-utf8-after-declared-prefix-substitution',
                     'raw_stream_bytes': {k: len(selected[k].encode('utf-8')) for k in ('stdout', 'stderr')},
                     'empty_streams': {k: selected[k] == '' for k in ('stdout', 'stderr')},
                     'difference_keys': [k for k, equal in equal_keys.items() if not equal]})
    check('origin_reconstruction_equals_wrapper_and_expected', reconstructed == wrapper['commands'] == expected)
    check('every_whole_argv_matches', all(r['whole_argv_matches_expanded_recipe'] for r in rows))
    check('every_actual_saved_exit_is_integer_zero', all(r['actual_saved_exit0_in_origin_wrapper_expected'] for r in rows))
    check('every_raw_utf8_stream_exact', all(r['raw_utf8_streams_exact'] for r in rows))
    check('every_semantic_field_exact', all(r['all_semantic_fields_exact'] for r in rows))
    check('eight_supplement_rows_once_each', sorted(supplement_indices) == list(range(8)))
    check('forty_two_source_rows_once_each', len(selected_indices) == len(set(selected_indices)) == 42)
    check('all_eight_supplement_targets_direct_executables', all(r['direct_executable_target'] is True for r in rows if r['origin'] == 'supplement'))
    check('source_asserts_unchanged_and_no_product_or_signing_execution', source['source_and_bundle_unchanged'] is True and source['signing_or_product_execution'] is False)
    check('supplement_no_error_no_product_or_signing_execution', supplement['first_error'] is None and supplement['product_or_signing_execution'] is False)

    pin_results = []
    for name, pin in pins['files'].items():
        payload = Path(pin['repository_path']).read_bytes()  # public repository only
        pin_results.append({'name': name, 'bytes_match': len(payload) == pin['bytes'], 'sha256_match': sha(payload) == pin['sha256'], 'sha256': sha(payload)})
    check('all_public_input_pins_match', all(p['bytes_match'] and p['sha256_match'] for p in pin_results))
    binding_results = []
    for binding in bindings['repository_inputs']:
        payload = Path(binding['path']).read_bytes()
        binding_results.append({'repository_path': binding['path'], 'bytes_match': len(payload) == binding['bytes'], 'sha256_match': sha(payload) == binding['sha256']})
    check('all_caller_public_repository_bindings_match', all(p['bytes_match'] and p['sha256_match'] for p in binding_results))
    private_wrapper_binding = bindings['private_inputs_read_only'][0]
    check('caller_declares_exact_wrapper_bytes_and_hash', private_wrapper_binding['bytes'] == ALLOWED['wrapper'][1] and private_wrapper_binding['sha256'] == ALLOWED['wrapper'][2])
    extraction.update(expected_canonical_sha256=sha(canonical(expected)), expected_shape=shape(expected),
                      wrapper_commands_canonical_sha256=sha(canonical(wrapper['commands'])),
                      wrapper_canonical_sha256=sha(canonical(wrapper)), wrapper_raw_sha256=sha(data['wrapper']))
    result = {'schema': 'openergy.metadata-basis-bridge.v1', 'scope': 'SOURCE_DATA_ONLY',
              'verdict': 'BOUNDED_DATA_CLEAR' if all(checks.values()) else 'HOLD',
              'runtime': 'UNVERIFIED', 'checks': checks, 'extraction': extraction,
              'source_rows_total': len(source_rows), 'source_rows_selected': len(selected_indices),
              'source_rows_unselected': [i for i in range(len(source_rows)) if i not in selected_indices],
              'supplement_rows_total': len(supplement['individual_objects']),
              'supplement_rows_selected': sorted(supplement_indices),
              'supplement_rows_unselected': [i for i in range(len(supplement['individual_objects'])) if i not in supplement_indices],
              'commands': len(rows), 'verify_rows': sum(r['label'].endswith('-verify') for r in rows),
              'strict_rows': sum(not r['label'].endswith('-verify') for r in rows),
              'difference_keys': [k for k, v in checks.items() if not v],
              'caller_logic_review': 'OUT_OF_SCOPE; separate 54bc355 HOLD remains',
              'public_input_pins': pin_results, 'caller_public_bindings': binding_results}
    save('bridge-checks.json', result)
    save('semantic-rows.json', rows)
    print(json.dumps({k: result[k] for k in ('verdict', 'commands', 'source_rows_selected', 'supplement_rows_selected', 'verify_rows', 'strict_rows', 'difference_keys', 'extraction')}, indent=2))


def preservation():
    records = []
    for alias in ALLOWED:
        events = [e for e in ledger['events'] if e['alias'] == alias]
        records.append({'alias': alias, 'regular_opens': len(events),
                        'all_reads_complete': all(e['complete'] for e in events),
                        'all_sizes_equal_bound': all(e['bytes_read'] == ALLOWED[alias][1] for e in events),
                        'all_hashes_equal_bound': all(e['actual_sha256'] == ALLOWED[alias][2] for e in events),
                        'all_modes_equal_first_read': all(e['mode'] == events[0]['mode'] for e in events),
                        'all_reads_stable_no_symlink_regular': all(e['stable_metadata'] and e['no_symlink_components'] and e['regular'] for e in events),
                        'sha256': ALLOWED[alias][2], 'bytes': ALLOWED[alias][1]})
    result = {'records': records, 'regular_opens': ledger['regular_opens'],
              'content_bytes': ledger['content_bytes'], 'regular_open_limit': 32,
              'content_byte_limit': 8388608, 'private_writes': 0,
              'optional_fifth_private_file_read': False,
              'no_further_private_reads_planned': True,
              'preserved_at_all_bounded_reads': all(all(v for k, v in r.items() if k.startswith('all_')) for r in records)}
    save('hash-preservation.json', result)
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    try:
        data = {alias: private_read(alias) for alias in ALLOWED}
        if sys.argv[1:] == ['--preserve']:
            preservation()
        else:
            need(not sys.argv[1:], 'reader-mode')
            inspect(data)
            analyze(data)
    except Exception as error:
        save('reader-errors.json', {'error_type': type(error).__name__, 'payload_published': False,
                                    'ledger_preserved': True})
        print('Reader stopped; ledger retained; no private payload printed.', file=sys.stderr)
        raise SystemExit(1)
