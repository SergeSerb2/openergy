"""Reviewer-owned public dependency/interface data reader; no supplied execution."""
import ast
import hashlib
import json
import os.path
from pathlib import Path
import re
import subprocess

OUT = Path(__file__).resolve().parent.relative_to(Path.cwd())
NEW = OUT.parent / 'm1-copy-parent-signal-schema-repair-2026-10-25-t3-f3ffc1d7'
OLD = OUT.parent / 'm1-copy-parent-capsule-2026-10-25-t3-f65c3d2f'
PRIOR = OUT.parent / 'm1-copy-parent-full-review-2026-10-25-t3-9edf92e5'
rows = []


def check(name, value):
    rows.append({'name': name, 'equal_or_satisfied': bool(value)})
    if not value:
        raise ValueError(name)


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def ident(raw):
    return {'bytes': len(raw), 'sha256': sha(raw)}


def pairs(items):
    result = {}
    for k, v in items:
        if k in result:
            raise ValueError('duplicate key: ' + k)
        result[k] = v
    return result


def bad_constant(value):
    raise ValueError(value)


def read(path):
    return json.loads(path.read_bytes(), object_pairs_hook=pairs, parse_constant=bad_constant)


def canonical(data):
    return json.dumps(data, sort_keys=True, separators=(',', ':'), ensure_ascii=True, allow_nan=False).encode()


def save(name, data):
    (OUT / name).write_text(json.dumps(data, indent=2) + '\n')


def git(*args):
    return subprocess.check_output(['git', *args])


deps = []
for label in ('dependency-bindings.json', 'candidate-review-bindings.json'):
    records = read(NEW / label)
    for r in records:
        path, rev = r['path'], r['commit']
        raw = Path(path).read_bytes()
        tree = git('ls-tree', rev, '--', path).decode().strip()
        meta, actual_path = tree.split('\t')
        mode, kind, blob = meta.split()
        check('frozen Git input ' + rev[:8] + ':' + path,
              actual_path == path and blob == r['blob'] and mode == r.get('mode', r.get('git_mode'))
              and kind == 'blob' and ident(raw) == {k: r[k] for k in ('bytes', 'sha256')}
              and raw == git('show', rev + ':' + path))
        if path.endswith('.json'):
            read(Path(path))
        deps.append({'group': label, 'commit': rev, 'path': path, 'mode': mode, 'blob': blob, **ident(raw)})
save('dependency-bindings.json', deps)

opening = {r['path']: r for r in read(OUT / 'opening-inventory.json')['entries']}
historical = read(NEW / 'baseline-tracked-inventory.json')
tree_rows = git('ls-tree', '-r', '7e5de706e7457de5f4880cdef180b27536dc0d5e').decode().splitlines()
tree = {}
for row in tree_rows:
    meta, path = row.split('\t')
    mode, kind, blob = meta.split()
    tree[path] = (mode, kind, blob)
check('complete 4422-row repair baseline inventory', len(historical) == len(tree) == 4422 and {r['path'] for r in historical} == set(tree))
for r in historical:
    o = opening[r['path']]
    check('baseline inventory exact ' + r['path'], tree[r['path']] == (r['git_mode'], r['git_type'], r['blob'])
          and all(o[k] == r[k] for k in ('git_mode', 'blob', 'working_mode', 'bytes', 'sha256'))
          and r['index'] == {'mode': o['git_mode'], 'blob': o['blob'], 'stage': o['stage']})
check('repair original inventory canonical hash', sha(canonical(historical) + b'\n') == read(NEW / 'preservation-opening.json')['inventory_sha256'])
save('historical-preservation-bindings.json', {'repair_parent': '7e5de706e7457de5f4880cdef180b27536dc0d5e', 'original_entries': len(historical), 'original_entries_equal_opening_checkout': True})

bindings = read(OUT / 'function-bindings.json')['declarations']
prior_bindings = {r['name']: r for r in read(PRIOR / 'function-bindings.json')['entries'] if r['after'] is not None}
prior_assessments = {r['name']: r for r in read(PRIOR / 'function-review.json')['assessments']}
check('complete prior full 48-function and class coverage', len(prior_bindings) == len(prior_assessments) == len(bindings) == 49)
coverage = []
for b in bindings:
    p = prior_bindings[b['name']]['after']
    a = prior_assessments[b['name']]
    check('full prior declaration binding ' + b['name'], all(b['before'][k] == p[k] == a['range'][k] for k in ('start', 'end', 'bytes', 'sha256')))
    assessment = a['assessment']
    if b['name'] == 'collect':
        assessment = 'Fresh full body review: retains original launch floor, helper/private vectors, bounds, clocks, partial stdin, actual exit/reap and independent EOF. Only signal operation/parent flag change. Fresh None poll and immediate sample gate one direct os.kill on the retained unreaped handle, strict original ends, only cap/deadline, old wait remainder, no other waiter. Syscall return is separate from delivery/reap; ESRCH and errors stay failed/unknown. COPY-P2-1 CLOSED_SOURCE_ONLY.'
    elif b['name'] == 'main':
        assessment = 'Full unchanged body re-read: null-closed admissions, durable guard, helper authentication then sole private COPY, actual reap before every postchild publication, single full data-only pending receipt, controller blocked, final root close then compact console and os._exit. Receipt-schema COPY-P2-2 CLOSED_SOURCE_ONLY for the separately changed signed domains; the historical full-review HOLD remains on baseline bytes.'
    coverage.append({'name': b['name'], 'basis': b['before'], 'current': b['after'], 'raw_state': b['state'], 'prior_assessment': a['assessment'], 'current_assessment': assessment, 'current_body_read': True})
save('function-coverage.json', {'functions': 48, 'classes': 1, 'omitted_current_declarations': [], 'coverage': coverage})
for filename, records in [('FUNCTIONS.md', [next(r for r in bindings if r['name'] == 'collect')]), ('GLOBAL-SEGMENTS.md', read(OUT / 'global-bindings.json'))]:
    blocks = re.findall(r'```python\n(.*?)```', (NEW / filename).read_text(), re.S)
    expected_blocks = []
    for record in records:
        for key, folder in [('before', OLD), ('after', NEW)]:
            r = record[key]
            expected_blocks.append(b''.join((folder / 'copy-parent.py.txt').read_bytes().splitlines(keepends=True)[r['start'] - 1:r['end']]).decode())
    check('complete exact documentation code blocks ' + filename, blocks == expected_blocks)

constants = read(OUT / 'literal-bindings.json')
for folder in (OLD, NEW):
    expanded = {}
    tree = ast.parse((folder / 'copy-parent.py.txt').read_text(), feature_version=(3, 9))
    for node in tree.body:
        if isinstance(node, ast.Assign):
            try:
                value = ast.literal_eval(node.value)
            except (ValueError, TypeError):
                continue
            for target in node.targets:
                if isinstance(target, ast.Name):
                    expanded[target.id] = value
                elif isinstance(target, ast.Tuple):
                    check('literal tuple shape', len(target.elts) == len(value) and all(isinstance(t, ast.Name) for t in target.elts))
                    expanded.update({t.id: v for t, v in zip(target.elts, value)})
    if folder == OLD:
        original = expanded
    else:
        check('all 37 literal-bound global names exact including caps', original == expanded and len(expanded) == 37)
save('expanded-literal-bindings.json', expanded)
check('all 31 simple literals equal complete prior review', constants == read(PRIOR / 'literal-bindings.json')['constants'])

pinpath = OUT.parent / 'm1-metadata-r9-repair-2026-10-25-t3-9b2c2354/input-pins.json'
pins = read(pinpath)['files']
check('exact 17 pin keys', len(pins) == 17 and set(pins) == set(constants['PINS']))
pin_bindings = []
for name, r in pins.items():
    public_path = Path(os.path.normpath(r['repository_path']))
    actual = ident(public_path.read_bytes())
    check('exact raw public pin ' + name, actual == constants['PINS'][name] == {k: r[k] for k in ('bytes', 'sha256')})
    pin_bindings.append({'name': name, 'public_path': str(public_path), **actual})
materialization_dir = OUT.parent / 'ordinary-combined-context-tuple-repair-2026-10-25'
modules = read(materialization_dir / 'materialization.json')['modules']
source_checks = read(materialization_dir / 'source-checks.json')['python_files']
check('10 exact module destinations', len(modules) == len(source_checks) == 10 and {r['destination'] for r in modules} == set(source_checks))
for r in modules:
    name = r['destination']
    raw = (materialization_dir / r['source']).read_bytes()
    check('source-check/materialization/module pin ' + name, ident(raw) == constants['PINS'][name] == {k: source_checks[name][k] for k in ('bytes', 'sha256')} and sha(raw) == r['sha256'])
recipe = read(OUT.parent / 'm1-next-reviewed-ordinary-flow-2026-10-25-t3-a8a4ece5/metadata.recipe.json')
commands = []
for obj in recipe['objects']:
    values = {'${BINARY}': '${BUNDLE}/' + obj['binary'], '${CODE_OBJECT}': '${BUNDLE}' + ('/' + obj['code_object'] if obj['code_object'] else '')}
    for verb in recipe['verb_order']:
        commands.append({'label': 'object-' + str(obj['index']) + '-' + verb, 'argv': [values.get(token, token) for token in recipe['argv_patterns'][verb]]})
commands.append({'label': 'full-bundle-deep-verify', 'argv': recipe['deep_verify']})
check('all 50 ordered full recipe vectors', len(commands) == 50 and commands == read(pinpath.parent / 'command-vectors.json')['commands'])
saved = read(OUT.parent / 'm1-source-metadata-observed-2026-10-25-t3-52422617/validation.json')
check('complete public SOURCE212 manifest', constants['SOURCE_RAWS'] == saved['record_files'] and len(saved['record_files']) == 212)
check('all SOURCE raw counts and original result', sum(r['bytes'] for r in constants['SOURCE_RAWS']) == 213334 == saved['all_raw_status_comparison_bytes'] and constants['SOURCE_RESULT'] == {'bytes': saved['raw_result_bytes'], 'sha256': saved['raw_result_sha256']})
check('SOURCE manifest canonical digest', sha(canonical(constants['SOURCE_RAWS']) + b'\n') == saved['raw_records_manifest_sha256'])
check('historical SOURCE no kill/allzero remains recorded', saved['all_actual_exits'] == 0 and saved['no_timeout_kill_overflow_drop_or_publication_error'] is True and saved['copy_executed'] is False)
save('upstream-static-data.json', {'pins': pin_bindings, 'modules': modules, 'vectors': commands, 'SOURCE212': constants['SOURCE_RAWS'], 'SOURCE_manifest': sha(canonical(constants['SOURCE_RAWS']) + b'\n'), 'SOURCE_result': constants['SOURCE_RESULT'], 'SOURCE_raw_bytes': 213334, 'fresh_COPY52_gate': 'Full source read; unchanged inspect_copy_result and copy_record_complete raw declarations; 52 ordered records, four exact refs each, all208, both full raw native comparisons; not actual COPY evidence', 'runtime_acceptance': False})

copy_map = {'prior-author-first-complete-reader.json': OLD / 'static-checks-first-complete-reader.json', 'prior-author-reader-errors.json': OLD / 'static-reader-errors.json', 'prior-author-static-checks.json': OLD / 'static-checks.json', 'prior-author-final-static-checks.json': OLD / 'final-static-checks.json', 'prior-author-whitespace-check.json': OLD / 'whitespace-check.json', 'prior-review-reader-notes.json': PRIOR / 'reader-notes.json', 'prior-review-whitespace-observations.json': PRIOR / 'whitespace-observations.json', 'public-bootstrap-source-evidence.json': PRIOR / 'public-bootstrap-source-evidence.json', 'public-stdlib-source-evidence.json': PRIOR / 'public-stdlib-source-evidence.json'}
for name, origin in copy_map.items():
    check('historical exact copied record ' + name, (NEW / name).read_bytes() == origin.read_bytes())
save('historical-record-bindings.json', [{'new': str(NEW / name), 'origin': str(origin), **ident(origin.read_bytes())} for name, origin in copy_map.items()])

pattern_rows = []
for r in read(NEW / 'frozen-signal-patterns.json'):
    raw = Path(r['path']).read_bytes()
    lines = raw.splitlines(keepends=True)
    context = b''.join(lines[r['context']['start'] - 1:r['context']['end']])
    declaration = b''.join(lines[r['declaration']['start'] - 1:r['declaration']['end']])
    check('frozen signal full raw declaration/context ' + r['path'], ident(raw) == r['source'] and ident(context) == {k: r['context'][k] for k in ('bytes', 'sha256')} and context.decode() == r['context']['text'] and ident(declaration) == {k: r['declaration'][k] for k in ('bytes', 'sha256')} and r['call'] in lines[r['call_line'] - 1].decode())
    pattern_rows.append(r)
save('frozen-signal-boundaries.json', {'execution': False, 'patterns': pattern_rows, 'composition_status': 'PENDING; frozen matching Popen.kill failure patterns are not repaired or newly cleared by this COPY review', 'root_local_CPython315_audit': 'Reported in task brief; no new local collector source or runtime clearance inferred'})

config = json.loads(constants['CONFIG_TEXT'])
check('all 14 duties null', len(config['parent']) == 14 and all(v is None for v in config['parent'].values()))
check('all9 prepared slots null', len(config['prepared']) == 9 and all(v is None for v in config['prepared'].values()))
check('all review slots and template hash null', config['template_sha256'] is None and all(v is None for r in config['reviews'].values() for v in r.values()))
check('CONFIG exact inert schema constant', read(NEW / 'config.schema.json')['oneOf'][0]['const'] == config)
plan = read(NEW / 'finite-plan.json')
save('plan-and-config-observations.json', {'CONFIG': config, 'finite_plan': plan, 'all_actual_future_bindings_null': True, 'no_execution_authority': True, 'no_new_human_gate': True})

prep = OUT.parent / 'm1-prepare-parent-root-layout-repair-2026-10-25-t3-1f8914f8/prepare-parent.py.txt'
prep_raw = prep.read_bytes()
check('separate prepare R5/import source exact', prep_raw == git('show', '02f94afbe3e1001b8d3e8fc4d79976475485f661:' + str(prep)) == git('show', '95181be:' + str(prep)))
save('prepare-field-compatibility.json', {'separate_R5': '02f94afbe3e1001b8d3e8fc4d79976475485f661', 'import': '95181be', 'path': str(prep), **ident(prep_raw), 'R6': 'Separate independent review pending per commission; no verdict assigned here', 'COPY_inspect_new_prepared': next(b['after'] for b in bindings if b['name'] == 'inspect_new_prepared'), 'COPY_collection_complete': next(b['after'] for b in bindings if b['name'] == 'collection_complete'), 'observation': 'Read actual generic collection predicate and prepare receipt equality paths. No prepare-primary closed-key set or kill_delivered read exists there. R5 parent kill_syscall_returned field alone requires no COPY relaxation. Actual identity, raw status equality, actual0/reap/EOF, errors, clocks, source/artifact/tuple/publication and external D4 remain mandatory.', 'prepare_clearance': False, 'future_prepared': None})
save('interface-observations.json', {'execution': False, 'count': len(rows), 'observations': rows, 'failed': [r for r in rows if not r['equal_or_satisfied']]})
print(json.dumps({'observations': len(rows), 'frozen_dependencies': 43, 'candidate_and_review_bindings': 57, 'coverage': len(coverage), 'pins': len(pins), 'modules': len(modules), 'vectors': len(commands), 'SOURCE_rows': len(constants['SOURCE_RAWS'])}))
