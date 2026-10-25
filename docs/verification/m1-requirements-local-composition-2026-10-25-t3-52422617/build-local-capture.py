"""Parent-authored static transformer. Supplied programs remain source data."""
import ast
import difflib
import hashlib
import json
import os
from pathlib import Path
import stat
import subprocess
import sys

OUT = Path(__file__).resolve().parent
REPO = OUT.parents[2]
OLD = REPO / 'docs/verification/m1-prepare-repaired-local-paths-2026-10-25-t3-52422617'
COLLECTOR = REPO / 'docs/verification/m1-local-direct-signal-composition-2026-10-25-t3-52422617/collector.py.txt'
OLD_HASHES = {
    'observer': 'bc000353179a65a2b1149b26729fa557a2ae93fefb0dcf55bb3d2f1b5c8badb6',
    'assessor': '5d55fb13fbbc541c1c081fe4530a70d904b1b134c8f9f2085018072ae2a38d9e',
    'driver': 'abe9da37c712f68e0ce8863e46b904aea5e664e543112bb8936de0c6d4514c72',
}
COLLECTOR_HASH = 'f1fa00b2ec61f6e4996b5938388f345a8ddcfb205455bc4482ce25711824f371'
NAMES = {
    'prepare-repaired-launch': 'requirements-source-launch',
    'prepare-repaired-run': 'requirements-source-run',
    'prepare.payload.py': 'requirements-source.payload.py',
    'prepare-local-exit-observer.py': 'requirements-source-local-exit-observer.py',
    'prepare-local-parent-driver.py': 'requirements-source-local-parent-driver.py',
    'prepare-repaired-parent-request.json': 'requirements-source-parent-request.json',
    'prepare-repaired-parent-exit.json': 'requirements-source-parent-exit.json',
    'prepare-repaired-transport-assessment.json': 'requirements-source-transport-assessment.json',
    'prepare-repaired-local-parent-once.json': 'requirements-source-local-parent-once.json',
    'prepare-repaired-local-parent-result.json': 'requirements-source-local-parent-result.json',
    'prepare-repaired-observer-handle-source.json': 'requirements-source-observer-handle-source.json',
    'prepare-repaired-local-direct-topology.json': 'requirements-source-local-direct-topology.json',
    'prepare-repaired-transport-assessor.py': 'requirements-source-transport-assessor.py',
    'prepare-repaired-transport-assessor.template.py.txt': 'requirements-source-transport-assessor.template.py.txt',
    'prepare-repaired-assessor-render-method.json': 'requirements-source-assessor-render-method.json',
    'prepare-repaired-assessor-render-proof.json': 'requirements-source-assessor-render-proof.json',
}
VALUES = {
    'prepare': 'metadata-source',
    'one-file-only-prepare': 'one-source-seven-requirements',
    'PREPARED_DATA_COMPLETE': 'COLLECTED_DATA_ONLY',
    'TRANSPORT_ONLY_PENDING_FULL_PREPARE': 'TRANSPORT_ONLY_PENDING_FULL_REQUIREMENTS',
    'LOCAL_TRANSPORT_ONLY_PENDING_FULL_PREPARE': 'LOCAL_TRANSPORT_ONLY_PENDING_FULL_REQUIREMENTS',
    'full prepare receipt and raw retention': 'full requirements receipt, capture, basis and guard retention',
    'full prepare, copy and artifact predicates': 'all nine actual observations and seven full requirements bodies; CURRENT artifact gate',
    'full_prepare_raws_retained_locally': 'full_requirements_raws_retained_locally',
    'full_prepare_receipt_assessed': 'full_requirements_receipt_assessed',
    'prepare_accepted': 'requirements_accepted',
    'prepare-parent.v1': 'requirements-source.v1',
    'prepare-local-parent.v1': 'requirements-source-local-parent.v1',
    'prepare-transport-assessment.v1': 'requirements-source-transport-assessment.v1',
}


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def canonical(value):
    return (json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=True,
                       allow_nan=False) + '\n').encode('ascii')


def unique(pairs):
    value = {}
    for key, item in pairs:
        assert key not in value
        value[key] = item
    return value


def load(raw):
    return json.loads(raw, object_pairs_hook=unique,
                      parse_constant=lambda value: (_ for _ in ()).throw(ValueError(value)))


def put(name, raw):
    fd = os.open(OUT / name, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
    try:
        assert os.write(fd, raw) == len(raw)
        os.fsync(fd)
    finally:
        os.close(fd)


def git(*args):
    return subprocess.check_output(['git', '-C', str(REPO), *args],
                                   env={**os.environ, 'GIT_OPTIONAL_LOCKS': '0'})


def snapshot():
    rows = []
    for entry in git('ls-files', '-s', '-z').split(b'\0'):
        if not entry:
            continue
        meta, name = entry.split(b'\t', 1)
        path = REPO / os.fsdecode(name)
        info = path.lstat()
        raw = os.fsencode(os.readlink(path)) if stat.S_ISLNK(info.st_mode) else path.read_bytes()
        rows.append([meta.decode('ascii'), os.fsdecode(name), stat.S_IMODE(info.st_mode), len(raw), sha(raw)])
    return rows


def span(raw, node):
    lines = raw.splitlines(keepends=True)
    return (sum(map(len, lines[:node.lineno - 1])) + node.col_offset,
            sum(map(len, lines[:node.end_lineno - 1])) + node.end_col_offset)


def rewrite_value(value):
    if type(value) is not str:
        return value
    if value.startswith('{') and '"schema":' in value:
        return canonical(rewrite_json(load(value))).decode('ascii').removesuffix('\n')
    if value in VALUES:
        return VALUES[value]
    value = '/'.join(NAMES.get(part, part) for part in value.split('/'))
    if value.startswith('openergy.prepare-'):
        value = 'openergy.requirements-source-' + value[len('openergy.prepare-'):]
    return value


def rewrite_json(value):
    if type(value) is dict:
        return {rewrite_value(key): rewrite_json(item) for key, item in value.items()}
    if type(value) is list:
        return [rewrite_json(item) for item in value]
    return rewrite_value(value)


def literal_variant(raw):
    tree = ast.parse(raw)
    edits = []
    for node in ast.walk(tree):
        if not isinstance(node, ast.Constant) or type(node.value) is not str:
            continue
        replacement = rewrite_value(node.value)
        if replacement == node.value:
            continue
        start, end = span(raw, node)
        token = repr(replacement).encode('ascii')
        edits.append((start, end, token, raw[start:end]))
        node.value = replacement
    changed = raw
    for start, end, token, original in sorted(edits, reverse=True):
        changed = changed[:start] + token + changed[end:]
    assert ast.dump(ast.parse(changed), include_attributes=False) == ast.dump(tree, include_attributes=False)
    reverse = changed
    offset = 0
    reverse_edits = []
    for start, end, token, original in sorted(edits):
        reverse_edits.append((start + offset, start + offset + len(token), original))
        offset += len(token) - (end - start)
    for start, end, original in sorted(reverse_edits, reverse=True):
        reverse = reverse[:start] + original + reverse[end:]
    assert reverse == raw
    return changed, len(edits)


COMPACT_FUNCTION = '''def requirements_compact(value):
    # Success transport summary only; full capture/basis/body acceptance stays external.
    keys(value, ['v', 'state', 'r', 'c', 'n', 'reaped', 'eof', 'kill', 'loss',
                 'p', 'raws_unpublished', 'f'])
    require(type(value['r']) is list and len(value['r']) == 2)
    integer(value['r'][0], 1, 16384)
    hexhash(value['r'][1])
    equal(value, {'v': 'requirements-source.v1', 'state': 'COLLECTED_DATA_ONLY',
                  'r': value['r'], 'c': [0] * 9, 'n': 9, 'reaped': 511, 'eof': 511,
                  'kill': 0, 'loss': 0, 'p': [2, 2, 2, 2],
                  'raws_unpublished': False, 'f': None})
    return value


'''.encode('ascii')


def add_compact(raw, name):
    tree = ast.parse(raw)
    anchor = next(n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name == name)
    start, end = span(raw, anchor)
    old = raw[start:end]
    if name == 'stream_records':
        begin = old.index(b"    keys(compact, ['v', 'state', 'r'])")
        stop = old.index(b"    equal(canonical(compact), stdout)", begin)
    else:
        begin = old.index(b"    keys(compact, ['v', 'state', 'r'])")
        stop = old.index(b"    equal(canonical(compact), input_raws['stdout'])", begin)
    assert b"integer(compact['r'][0], 1, 262144)" in old[begin:stop]
    replacement = old[:begin] + b'    requirements_compact(compact)\n' + old[stop:]
    raw = raw[:start] + COMPACT_FUNCTION + replacement + raw[end:]
    assert len([n for n in ast.parse(raw).body if isinstance(n, ast.FunctionDef)
                and n.name == 'requirements_compact']) == 1
    return raw, {'function': name, 'old_body_sha256': sha(old), 'new_body_sha256': sha(replacement),
                 'new_complete_predicate_sha256': sha(COMPACT_FUNCTION.rstrip())}


def declarations(raw):
    rows = []
    for node in ast.walk(ast.parse(raw)):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            begin, end = span(raw, node)
            rows.append({'name': node.name, 'line': node.lineno, 'end_line': node.end_lineno,
                         'bytes': end - begin, 'sha256': sha(raw[begin:end])})
    return sorted(rows, key=lambda row: (row['line'], row['end_line']))


def patch(before, after, name):
    return ''.join(difflib.unified_diff(before.decode('ascii').splitlines(keepends=True),
        after.decode('ascii').splitlines(keepends=True), fromfile='a/' + name,
        tofile='b/' + name, n=0)).encode('ascii')


def replace_assignment(raw, name, value):
    tree = ast.parse(raw)
    nodes = [n for n in tree.body if isinstance(n, ast.Assign)
             and any(isinstance(t, ast.Name) and t.id == name for t in n.targets)]
    assert len(nodes) == 1
    start, end = span(raw, nodes[0].value)
    return raw[:start] + repr(value).encode('ascii') + raw[end:]


def main():
    assert len(sys.argv) in (2, 3)
    before = snapshot()
    parent = git('rev-parse', 'HEAD').decode().strip()
    collector = COLLECTOR.read_bytes()
    assert len(collector) == 28099 and sha(collector) == COLLECTOR_HASH
    if sys.argv[1] == 'components':
        outputs = {}
        for name in ['observer', 'assessor']:
            original = (OLD / (name + '.py.txt')).read_bytes()
            assert sha(original) == OLD_HASHES[name]
            result, edits = literal_variant(original)
            predicate = None
            if name == 'assessor':
                result, predicate = add_compact(result, 'stream_records')
            put(name + '.py.txt', result)
            put(name + '-from-repaired.patch', patch(original, result, name + '.py.txt'))
            outputs[name] = {'old': {'bytes': len(original), 'sha256': sha(original)},
                             'new': {'bytes': len(result), 'sha256': sha(result)},
                             'literal_edits': edits, 'compact_predicate_change': predicate,
                             'old_declarations': declarations(original), 'new_declarations': declarations(result)}
        assert snapshot() == before
        put('component-bindings.json', canonical({'schema': 'openergy.requirements-source-local-components.v1',
            'execution': False, 'status': 'IMPLEMENTED_UNREVIEWED', 'baseline_parent': parent,
            'sources': outputs, 'path_mapping': NAMES, 'literal_mapping': VALUES,
            'collector': {'source_candidate': '8bde42c5d18bb548aa435ee8f25321f236150da8',
                          'bytes': len(collector), 'sha256': sha(collector), 'changed': False},
            'prior_tracked_count': len(before), 'prior_identity_sha256': sha(canonical(before)),
            'old_tracked_entries_preserved': True, 'driver_binding': None, 'full_review': None,
            'target_staging': False, 'runtime': 'UNVERIFIED'}))
    else:
        assert sys.argv[1] == 'driver' and len(sys.argv) == 3
        component = sys.argv[2]
        assert len(component) == 40 and git('rev-parse', 'HEAD').decode().strip() == component
        sources = {name: (OUT / (name + '.py.txt')).read_bytes() for name in ['observer', 'assessor']}
        templates = {name: {'bytes': len(raw), 'sha256': sha(raw)} for name, raw in sources.items()}
        templates['collector'] = {'bytes': len(collector), 'sha256': sha(collector)}
        original = (OLD / 'driver.py.txt').read_bytes()
        assert sha(original) == OLD_HASHES['driver']
        result, edits = literal_variant(original)
        result, predicate = add_compact(result, 'qualify_assessment')
        result = replace_assignment(result, 'REQUIRED_COMPOSITION_CANDIDATE', component)
        result = replace_assignment(result, 'REQUIRED_TEMPLATES', templates)
        put('driver.py.txt', result)
        put('driver-from-repaired.patch', patch(original, result, 'driver.py.txt'))
        schema = rewrite_json(load((OLD / 'config.schema.json').read_bytes()))
        refs = {'b94f1c7cba63a15f86aa6dc2f7301158e6bf5b1d': component,
                OLD_HASHES['observer']: sha(sources['observer']), OLD_HASHES['assessor']: sha(sources['assessor'])}
        sizes = {7754: len(sources['observer']), 25827: len(sources['assessor'])}
        def rebind(value):
            if type(value) is dict:
                return {key: rebind(item) for key, item in value.items()}
            if type(value) is list:
                return [rebind(item) for item in value]
            if type(value) is str:
                return refs.get(value, value)
            if type(value) is int:
                return sizes.get(value, value)
            return value
        put('config.schema.json', canonical(rebind(schema)))
        for name in ['config.template.json', 'root-source-association.template.json']:
            put(name, canonical(rewrite_json(load((OLD / name).read_bytes()))))
        assert snapshot() == before
        put('driver-bindings.json', canonical({'schema': 'openergy.requirements-source-local-driver-binding.v1',
            'execution': False, 'status': 'IMPLEMENTED_UNREVIEWED', 'component_candidate': component,
            'old': {'bytes': len(original), 'sha256': sha(original)},
            'new': {'bytes': len(result), 'sha256': sha(result)}, 'required_templates': templates,
            'literal_edits': edits, 'compact_predicate_change': predicate,
            'old_declarations': declarations(original), 'new_declarations': declarations(result),
            'prior_tracked_count': len(before), 'prior_identity_sha256': sha(canonical(before)),
            'old_tracked_entries_preserved': True, 'full_review': None, 'target_staging': False,
            'runtime': 'UNVERIFIED'}))
    print(json.dumps({'phase': sys.argv[1], 'baseline_parent': parent,
                      'old_tracked_entries_preserved': len(before)}))


if __name__ == '__main__':
    main()
