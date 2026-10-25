"""Reviewer-authored PUBLIC DATA reader. Never imports or executes supplied code.

Only public Git reads, workspace text/strict JSON/hash/AST/literal data reads,
and writes of this one owned review folder are performed. No target access.
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

ROOT = Path.cwd()
OWN = ROOT / 'docs/verification/m1-prepare-parent-observed-layout-r7-review-2026-10-25-t3-80c9b701'
R7 = 'docs/verification/m1-prepare-parent-observed-layout-r7-2026-10-25-t3-00056e6d'
R5 = 'docs/verification/m1-prepare-parent-root-layout-repair-2026-10-25-t3-1f8914f8'
R6 = 'docs/verification/m1-prepare-parent-root-layout-review-2026-10-25-t3-98186f1b'
FAILED = 'docs/verification/m1-file-only-prepare-attempt-2026-10-25-t3-52422617'
AUDIT = 'docs/verification/m1-file-only-prepare-attempt-review-2026-10-25-t3-cc7061fd'
CANDIDATE = '44a6f8dfc7b8cb9597caec7b2fa34e98677cb705'
IMPORT = '5a587faa701da00d6e16f2e5ff9f2f43f553caf5'
BASIS = '02f94afbe3e1001b8d3e8fc4d79976475485f661'
REVIEW6 = '63b6a3b564d6a557f60e0e2b0e4a1943ffa2b19f'
AUDIT_COMMIT = 'b90044c48727e7d0e96e685b1b6c1d08cff81111'
AUDIT_IMPORT = 'e3611068e1e2ad3766e79eff87be6f38c7f4c531'
FAILED_COMMIT = '6dea2532eeae763150d7efb6701632f8ebeb9163'
ENV = dict(os.environ, GIT_OPTIONAL_LOCKS='0')
CHECKS = []
BINDINGS = {}


def check(ok, label):
    CHECKS.append({'name': label, 'pass': bool(ok)})
    if not ok:
        raise ValueError(label)


def git(*args):
    return subprocess.check_output(['git', *args], env=ENV)


def sha(b):
    return hashlib.sha256(b).hexdigest()


def enc(v):
    return (json.dumps(v, sort_keys=True, separators=(',', ':'), ensure_ascii=True,
                       allow_nan=False) + '\n').encode('ascii')


def identity(b):
    return {'bytes': len(b), 'sha256': sha(b)}


def pairs(rows):
    out = {}
    for k, v in rows:
        if k in out:
            raise ValueError('duplicate JSON key')
        out[k] = v
    return out


def nonfinite(v):
    raise ValueError('nonfinite JSON number: ' + v)


def floating(v):
    result = float(v)
    if not math.isfinite(result):
        raise ValueError('nonfinite JSON float')
    return result


def strict(b):
    return json.loads(b, object_pairs_hook=pairs, parse_constant=nonfinite,
                      parse_float=floating)


def read(path):
    p = ROOT / path
    check(p.is_relative_to(ROOT) and not p.is_symlink(), 'public regular path ' + path)
    return p.read_bytes()


def data(path):
    return strict(read(path))


def bind(commit, path):
    key = commit + ':' + path
    if key in BINDINGS:
        return git('show', key)
    row = git('ls-tree', '-z', commit, '--', path).rstrip(b'\0')
    metadata, actual_path = row.split(b'\t')
    mode, kind, blob = metadata.decode().split()
    check(actual_path.decode() == path and kind == 'blob', 'Git blob ' + key)
    b = git('show', key)
    p = ROOT / path
    working = read(path)
    working_mode = format(stat.S_IMODE(p.lstat().st_mode), '04o')
    check(working == b, 'working exact ' + key)
    check(working_mode == ('0755' if mode == '100755' else '0644'), 'mode exact ' + key)
    BINDINGS[key] = dict(commit=commit, path=path, blob=blob, mode=mode,
                         working_mode=working_mode, **identity(b))
    if path.endswith('.json'):
        strict(b)
    return b


def packet(commit, folder):
    raw = git('ls-tree', '-r', '-z', commit, '--', folder)
    rows = []
    for entry in raw.rstrip(b'\0').split(b'\0'):
        metadata, p = entry.split(b'\t')
        path = p.decode()
        b = bind(commit, path)
        rows.append(dict(path=path.removeprefix(folder + '/'),
                         mode=metadata.decode().split()[0], **identity(b)))
    rows.sort(key=lambda r: r['path'])
    sums = read(folder + '/SHA256SUMS').decode().splitlines()
    names = []
    for line in sums:
        digest, name = line.split('  ', 1)
        check(name in {r['path'] for r in rows}, 'checksum path in exact Git packet ' + name)
        check(not name.startswith('/') and '..' not in Path(name).parts,
              'checksum public path ' + name)
        check(sha(read(folder + '/' + name)) == digest, 'checksum ' + folder + '/' + name)
        names.append(name)
    check(len(set(names)) == len(names), 'unique checksum rows ' + folder)
    check(set(names) == {r['path'] for r in rows} - {'SHA256SUMS'},
          'complete checksum coverage ' + folder)
    return dict(commit=commit, folder=folder,
                tree=git('rev-parse', commit + ':' + folder).decode().strip(),
                count=len(rows), bytes=sum(r['bytes'] for r in rows),
                entries_digest=identity(enc(rows)), checksum_rows=len(names))


def module(b):
    return ast.parse(b, feature_version=(3, 9))


def physical(b, n):
    lines = b.splitlines(keepends=True)
    first = min([n.lineno] + [d.lineno for d in getattr(n, 'decorator_list', [])])
    return b''.join(lines[first - 1:n.end_lineno])


def declarations(b):
    out = {}
    def walk(n, prefix=''):
        for child in ast.iter_child_nodes(n):
            if isinstance(child, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
                name = prefix + child.name
                check(name not in out, 'unique declaration ' + name)
                out[name] = (child, physical(b, child))
                walk(child, name + '.')
            else:
                walk(child, prefix)
    walk(module(b))
    return out


def assignments(b):
    return {n.targets[0].id: n for n in module(b).body
            if isinstance(n, ast.Assign) and len(n.targets) == 1 and isinstance(n.targets[0], ast.Name)}


def literal_globals(b):
    out = {}
    for name, n in assignments(b).items():
        try:
            out[name] = ast.literal_eval(n.value)
        except (ValueError, TypeError):
            pass
    return out


def patch_reconstruction(b, patch, reverse=False):
    lines = patch.splitlines(keepends=True)
    check(lines[:2] == [b'--- a/prepare-parent.py.txt\n', b'+++ b/prepare-parent.py.txt\n'],
          'exact patch headers')
    original = b.splitlines(keepends=True)
    result, pos, index, count = [], 0, 2, 0
    while index < len(lines):
        match = re.fullmatch(rb'@@ -(\d+),(\d+) \+(\d+),(\d+) @@\n', lines[index])
        check(match is not None, 'strict patch hunk header')
        oldstart, oldcount, newstart, newcount = map(int, match.groups())
        index += 1
        old, new = [], []
        while index < len(lines) and not lines[index].startswith(b'@@ '):
            line = lines[index]
            check(line[:1] in (b' ', b'-', b'+'), 'strict patch line')
            if line[:1] != b'+': old.append(line[1:])
            if line[:1] != b'-': new.append(line[1:])
            index += 1
        check((len(old), len(new)) == (oldcount, newcount), 'exact patch hunk counts')
        if reverse:
            old, new = new, old
            oldstart, newstart = newstart, oldstart
        start = oldstart - 1
        check(start >= pos and original[start:start + len(old)] == old,
              'exact patch context at declared offset')
        result.extend(original[pos:start])
        check(len(result) == newstart - 1, 'exact patch new offset')
        result.extend(new)
        pos = start + len(old)
        count += 1
    result.extend(original[pos:])
    check(count == 2, 'exact two patch hunks')
    return b''.join(result)


def schema(v, s):
    if 'anyOf' in s:
        if not any(schema(v, option) for option in s['anyOf']): return False
    kinds = s.get('type')
    if kinds is not None:
        if isinstance(kinds, str): kinds = [kinds]
        actual = {dict:'object', list:'array', str:'string', int:'integer',
                  float:'number', bool:'boolean', type(None):'null'}[type(v)]
        if actual not in kinds and not (actual == 'integer' and 'number' in kinds): return False
    if 'const' in s and (type(v) != type(s['const']) or v != s['const']): return False
    if 'enum' in s and not any(type(v) == type(a) and v == a for a in s['enum']): return False
    if type(v) is dict:
        if not set(s.get('required', [])).issubset(v): return False
        props = s.get('properties', {})
        if s.get('additionalProperties') is False and not set(v).issubset(props): return False
        if not all(schema(val, props[k]) for k, val in v.items() if k in props): return False
    if type(v) is list and 'items' in s:
        if not all(schema(a, s['items']) for a in v): return False
    if type(v) is str and 'pattern' in s and not re.search(s['pattern'], v): return False
    if type(v) in (int, float):
        if 'minimum' in s and v < s['minimum']: return False
        if 'maximum' in s and v > s['maximum']: return False
    return True


def main():
    packets = [packet(CANDIDATE, R7), packet(IMPORT, R7), packet(BASIS, R5),
               packet(REVIEW6, R6), packet(FAILED_COMMIT, FAILED),
               packet(AUDIT_COMMIT, AUDIT), packet(AUDIT_IMPORT, AUDIT)]
    check(packets[0]['count'] == 18 and packets[0]['tree'] == packets[1]['tree'],
          'candidate/import entire eighteen-file subtree equality')
    check(packets[-2]['tree'] == packets[-1]['tree'], 'failed-audit original/import subtree equality')
    manifest = data(R7 + '/manifest.json')
    check(len(manifest['payload']) == 16, 'sixteen R7 manifest payload entries')
    for row in manifest['payload']:
        actual = BINDINGS[CANDIDATE + ':' + R7 + '/' + row['path']]
        check(all(actual[k] == row[k] for k in ('bytes','sha256','working_mode'))
              and actual['mode'] == row['git_mode'], 'exact manifest payload ' + row['path'])
    for commit in (CANDIDATE, IMPORT):
        changed = git('diff-tree', '--no-commit-id', '--name-only', '-r', commit + '^', commit).decode().splitlines()
        check(len(changed) == 18 and all(p.startswith(R7 + '/') for p in changed),
              'only candidate folder added ' + commit)
    old = bind(BASIS, R5 + '/prepare-parent.py.txt')
    new = bind(CANDIDATE, R7 + '/prepare-parent.py.txt')
    patch = read(R7 + '/from-R5.patch')
    check(identity(old) == {'bytes':109444, 'sha256':'b3fbd793a0709c7857e34efca667a081a0486accfd0f3ad13a8068484b7eb8d9'}, 'exact original source')
    check(identity(new) == {'bytes':110497, 'sha256':'8fb377d4e9d0f1fc45a261bf26b8b2df06111fa780e8d3932d9fbc3029e4d67c'}, 'exact R7 source')
    check(identity(patch) == {'bytes':2114, 'sha256':'432e651d52ab33f650e13c61a094167ff066adcf137eb7d2f594c0b0b18948dd'}, 'exact R7 patch')
    check(patch_reconstruction(old, patch) == new, 'strict full forward reconstruction')
    check(patch_reconstruction(new, patch, True) == old, 'strict full reverse reconstruction')
    check(sha(bind(REVIEW6, R6 + '/REVIEW.md')) == 'dc8e986c6a3e914b65a13686fc9f1a7e15702e8548e545075d6967f340541c4e', 'exact historical R6 report')
    check(sha(bind(AUDIT_COMMIT, AUDIT + '/REVIEW.md')) == '604fec5c9c11b6ca34706d8752c8db5332b9c78163af8338aabf291fef485d7f', 'exact latest failed-data audit report')
    od, nd = declarations(old), declarations(new)
    check(set(od) == set(nd) and len(nd) == 45, 'all forty-five physical declarations')
    changed = [name for name in od if od[name][1] != nd[name][1]]
    check(changed == ['closed_root_layout'], 'only closed_root_layout declaration changes')
    rows = []
    author = data(R7 + '/function-inventory.json')['declarations']
    author_by_name = {row['name']:row for row in author}
    for name, (n, b) in nd.items():
        first = min([n.lineno] + [a.lineno for a in getattr(n, 'decorator_list', [])])
        row = dict(name=name, kind=type(n).__name__, lines=[first,n.end_lineno],
                   before=identity(od[name][1]), after=identity(b), unchanged=od[name][1] == b,
                   source_reviewed=True)
        check(author_by_name[name]['after']['sha256'] == sha(b)
              and author_by_name[name]['before']['sha256'] == sha(od[name][1]),
              'complete author declaration range binding ' + name)
        rows.append(row)
    check(read(R7 + '/changed-function.before.txt') == od['closed_root_layout'][1]
          and read(R7 + '/changed-function.after.txt') == nd['closed_root_layout'][1],
          'complete before/after function transcripts')
    og, ng = assignments(old), assignments(new)
    check(len(og) == 41 and set(ng) - set(og) == {'ROOT_PARENT_ONLY','ROOT_OBSERVED_DATA'},
          'exact two new top-level assignments')
    check([k for k in og if physical(old, og[k]) != physical(new, ng[k])] == ['ROOT_INITIAL_NAMES'],
          'all old assignments exact except name-set expression')
    before_block = old[old.index(b'ROOT_INITIAL_NAMES ='):old.index(b'\n\nHANDLES =')]
    after_block = new[new.index(b'# These two COMPLETED originals'):new.index(b'\n\nHANDLES =')]
    restored = new.replace(after_block, before_block, 1).replace(nd['closed_root_layout'][1], od['closed_root_layout'][1], 1)
    check(restored == old, 'all bytes outside changed global block and body unchanged')
    g = literal_globals(new)
    check(len(g['COMPLETED']) == 7 and sha(enc(g['COMPLETED'])) == g['MANIFEST_HASH'],
          'unchanged complete-seven canonical provenance')
    check(g['ROOT_PARENT_ONLY'] == ('source-files-once.guard','source-files-parent-result.json'),
          'exact parent-only pair')
    remote = set(g['COMPLETED']) - set(g['ROOT_PARENT_ONLY'])
    groups = [set(g['INITIAL_INPUTS']), remote, {r['leaf'] for r in g['SOURCE_RAWS']},
              set(g['SOURCE_BOUNDARY']),set(g['EIGHT']),set(g['ROOT_DATA_ADDITIONS']),set(g['ROOT_OBSERVED_DATA'])]
    names = set().union(*groups)
    check([len(s) for s in groups] == [20,5,212,6,8,7,2]
          and sum(map(len,groups)) == len(names) == 260, 'disjoint actual remote260 set')
    before_private = names | set(g['OUTPUTS'][:4])
    check(len(before_private) == 264, 'four additional durable publications make264')
    check(sorted(n for n in names if n.endswith('.py')) == sorted(n for n in g['PINS'] if n.endswith('.py'))
          and len([n for n in names if n.endswith('.py')]) == 10,
          'exact ten pinned Python leaves; observed leaves not import alternatives')
    check(g['INITIAL_CAP'] == 12517376 and g['ENTRY_CAP'] == 4096 and g['RAW_CAP'] == 1048576,
          'original actual INITIAL/entry/raw caps')
    check(g['INITIAL_CAP'] == 33554432 - 16777216 - 4194304 - 65536,
          'original32MiB minus16MiB minus4MiB minus64KiB arithmetic')
    supplied = data(R7 + '/source-bindings.json')
    for r in supplied['original17_pins'] + supplied['public_refs']:
        b = bind(r['commit'], r['path'])
        check(identity(b) == {'bytes':r['bytes'],'sha256':r['sha256']}
              and BINDINGS[r['commit'] + ':' + r['path']]['blob'] == r['blob'], 'declared public binding ' + r['path'])
    check(len(supplied['original17_pins']) == len(g['PINS']) == 17, 'all seventeen original pins')
    for r in supplied['original17_pins']:
        check(g['PINS'][r['name']] == {'bytes':r['bytes'],'sha256':r['sha256']}, 'full pin binding ' + r['name'])
    for r in data(R6 + '/public-input-bindings.json')['entries']:
        b = bind(r['commit'], r['path'])
        check(identity(b) == {'bytes':r['bytes'],'sha256':r['sha256']}, 'full R6 public input binding ' + r['path'])
    for name, public in [('coverage-policy.json','observed-coverage-policy.json'),(':-','observed-colon-dash.bin')]:
        b = read(FAILED + '/' + public)
        check(identity(b) == g['ROOT_OBSERVED_DATA'][name], 'actual observed raw binding ' + name)
    colon = read(FAILED + '/observed-colon-dash.bin')
    check(colon == b'# designated => cdhash H"07f9138c67bb59ebdc8c04536fc0c9cb108b0fb0"\n'
          and colon.endswith(b'\n') and not colon.endswith(b'\n\n'), 'colon-dash exactly one LF')
    check(read(FAILED + '/observed-coverage-policy.json') == read('docs/verification/ordinary-combined-context-tuple-repair-2026-10-25/coverage-policy.json'), 'full original frozen policy equality')
    diag = data(FAILED + '/diagnostic-state.json')
    check(diag['entries_seen'] == 260 and diag['listed_total_bytes'] == 1242294
          and set(diag['missing_initial_names']) == set(g['ROOT_PARENT_ONLY'])
          and set(diag['extra_names']) == set(g['ROOT_OBSERVED_DATA'])
          and diag['metadata_rejections'] == [], 'saved layout facts; no new target observation')
    oldnames = set(g['INITIAL_INPUTS']) | set(g['COMPLETED']) | set().union(*groups[2:6])
    check((oldnames - set(diag['missing_initial_names'])) | set(diag['extra_names']) == names,
          'revised set exactly saved differences applied to old data set')
    layout = dict(groups=[len(s) for s in groups], initial_names=sorted(names),
                  initial_names_digest=identity(enc(sorted(names))),
                  before_private_names_digest=identity(enc(sorted(before_private))),
                  parent_only=sorted(g['ROOT_PARENT_ONLY']), remote_completed=sorted(remote),
                  observed_data=g['ROOT_OBSERVED_DATA'], saved_entries=260, saved_bytes=1242294,
                  current_target_observation=None, execution=False)
    for n in ('PARENT-CONTRACT.md','config.schema.json','config.template.json','child-status.schema.json'):
        check(read(R7 + '/' + n) == read(R5 + '/' + n), 'byte-exact historical R5 contract ' + n)
    check(read(R7 + '/cli-contract.R5.json') == read(R5 + '/cli-contract.json'), 'byte-exact historical R5 CLI contract')
    config = strict(g['CONFIG_TEXT'])
    check(config == data(R7 + '/config.template.json') and schema(config, data(R7 + '/config.schema.json')),
          'full config template equals embedded JSON and closed schema data')
    check(config['execution'] is False and config['template_sha256'] is None
          and all(v is None for v in config['parent'].values())
          and all(v is None for row in config['reviews'].values() for v in row.values())
          and all(v is None for k in ('completed_SOURCE','artifact_publication') for v in config[k].values()),
          'all actual CONFIG review/ref/inspection slots null; non-admitting')
    status_schema = data(R7 + '/child-status.schema.json')
    constructor = next(n for n in ast.walk(nd['collect'][0]) if isinstance(n,ast.Assign)
                       and any(isinstance(t,ast.Name) and t.id == 'status' for t in n.targets))
    status_keys = {k.arg for k in constructor.value.keywords}
    check(status_schema['additionalProperties'] is False
          and set(status_schema['required']) == status_keys
          and set(status_schema['properties']) == status_keys | {'retained_bytes'},
          'closed status schema exact constructor plus retained_bytes')
    check(all(status_schema['properties'][k] == {'type':['integer','null']}
              for k in ('actual_exit','kill_eligibility_exit'))
          and 'kill_syscall_returned' in status_keys and 'kill_delivered' not in status_keys,
          'genuine signed exits and truthful direct-syscall-return flag')
    startup_return = nd['startup'][0].body[0].value
    check(isinstance(startup_return,ast.Call) and len(startup_return.keywords) == 10, 'exact startup ten keys')
    startup_keys = {a.arg for a in startup_return.keywords}
    nextop = data('docs/verification/m1-next-reviewed-ordinary-flow-2026-10-25-t3-a8a4ece5/next-operation.json')
    check(startup_keys == set(nextop['parent_startup']), 'original next-operation startup keys')
    source_checks = data('docs/verification/ordinary-combined-context-tuple-repair-2026-10-25/source-checks.json')
    for name, row in source_checks['python_files'].items():
        p = (ROOT / 'docs/verification/ordinary-combined-context-tuple-repair-2026-10-25' / row['package_path']).resolve()
        check(p.is_relative_to(ROOT) and identity(p.read_bytes()) == g['PINS'][name], 'ten actual materialized module bytes ' + name)
    vectors = data('docs/verification/m1-metadata-r9-repair-2026-10-25-t3-9b2c2354/command-vectors.json')
    recipe = data('docs/verification/m1-next-reviewed-ordinary-flow-2026-10-25-t3-a8a4ece5/metadata.recipe.json')
    expanded = []
    for obj in recipe['objects']:
        for verb in recipe['verb_order']:
            substitutions = {'${BINARY}':'${BUNDLE}/' + obj['binary'],
                             '${CODE_OBJECT}':'${BUNDLE}' + ('/' + obj['code_object'] if obj['code_object'] else '')}
            expanded.append(dict(label='object-' + str(obj['index']) + '-' + verb,
                                 argv=[substitutions.get(a,a) for a in recipe['argv_patterns'][verb]]))
    expanded.append(dict(label='full-bundle-deep-verify',argv=recipe['deep_verify']))
    check(len(vectors['commands']) == 50 and vectors['commands'] == expanded,
          'all fifty actual vectors equal complete recipe data expansion')
    labels = ['entry-template','python-version','python-architecture'] + [r['label'] for r in expanded]
    expected_raws = ['metadata-source-' + label + suffix for label in labels
                     for suffix in ('.stdout','.stderr','-os.json','-comparison.json')]
    check([r['leaf'] for r in g['SOURCE_RAWS']] == expected_raws, 'all212 actual53-label raw names')
    validation = data('docs/verification/m1-source-metadata-observed-2026-10-25-t3-52422617/validation.json')
    check(g['SOURCE_RAWS'] == validation['record_files']
          and sha(enc(g['SOURCE_RAWS'])) == validation['raw_records_manifest_sha256']
          and sum(r['bytes'] for r in g['SOURCE_RAWS']) == 213334,
          'all212 length/hash rows equal full historical saved SOURCE validation')
    check(g['SOURCE_RESULT'] == {'bytes':validation['raw_result_bytes'],'sha256':validation['raw_result_sha256']},
          'complete saved SOURCE result identity')
    saved_requirements = []
    for i in range(7):
        label = 'object-' + str(i) + '-requirements'
        selected = next(r for r in expanded if r['label'] == label)
        check(selected['argv'][-2] == ':-', 'inherited unsafe requirements destination ' + label)
        base = FAILED + '/saved-requirements/metadata-source-' + label
        stdout, stderr = read(base + '.stdout'), read(base + '.stderr')
        actual, comparison = data(base + '-os.json'), data(base + '-comparison.json')
        for suffix in ('.stdout','.stderr','-os.json','-comparison.json'):
            b = read(base + suffix)
            source_ref = next(r for r in g['SOURCE_RAWS'] if r['leaf'] == 'metadata-source-' + label + suffix)
            check(identity(b) == {k:source_ref[k] for k in ('bytes','sha256')}, 'actual saved requirement raw binding ' + label + suffix)
        check(stdout == b'' and stderr.startswith(b'Executable=') and stderr.count(b'\n') == 1
              and actual['actual_exit'] == 0 and actual['reaped'] is True
              and actual['stdout_eof'] is True and actual['stderr_eof'] is True
              and actual['argv'] == [a.replace('${BUNDLE}',recipe['source_bundle']) for a in selected['argv']],
              'actual saved0 complete pipes with no requirement body ' + label)
        saved_requirements.append(dict(object=i, actual_saved_exit=actual['actual_exit'],
                                       stdout_bytes=len(stdout), stderr_bytes=len(stderr),
                                       requirement_body_proof=False))
    producers = [
        ('docs/verification/m1-source-stdin-payload-kill-gate-repair-2026-10-25-t3-da89534d/source-payload.py.txt','inspect_remote'),
        ('docs/verification/m1-system-stdin-materializer-2026-10-25-t3-4878b759/materializer.py.txt','inspect_completed'),
        ('docs/verification/m1-artifact-data-materializer-2026-10-25-t3-f0b1e4d1/materializer.py.txt','inspect_completed')]
    provenance = []
    for path, function in producers:
        b = read(path)
        check(literal_globals(b)['COMPLETED'] == g['COMPLETED'], 'producer full-seven provenance ' + path)
        node, body = declarations(b)[function]
        raw_read = next(n for n in ast.walk(node) if isinstance(n,ast.DictComp))
        actual_five = set(ast.literal_eval(raw_read.generators[0].iter))
        check(actual_five == remote, 'actual producer reads five remote originals ' + path)
        provenance.append(dict(path=path, function=function, lines=[node.lineno,node.end_lineno],
                               function_bytes=identity(body),remote_names=sorted(actual_five)))
    published = data('docs/verification/m1-artifact-data-observed-2026-10-25-t3-52422617/materializer-receipt.json')
    check(published['remote_completed_files_authenticated'] == 5, 'actual public publisher receipt authenticates five')
    config_node = ng['CONFIG_TEXT'].value
    lines = new.splitlines(keepends=True)
    token_start = sum(map(len,lines[:config_node.lineno - 1])) + config_node.col_offset
    token_end = sum(map(len,lines[:config_node.end_lineno - 1])) + config_node.end_col_offset
    token = new[token_start:token_end]
    check(ast.literal_eval(token.decode()) == g['CONFIG_TEXT'], 'unique exact CONFIG token data')
    normalized = module(new)
    next(n for n in normalized.body if isinstance(n,ast.Assign)
         and isinstance(n.targets[0],ast.Name) and n.targets[0].id == 'CONFIG_TEXT').value.value = '<CONFIG_TEXT_TOKEN>'
    current_template = dict(token_range=[token_start,token_end], token=identity(token),
                            prefix=identity(new[:token_start]),suffix=identity(new[token_end:]),
                            normalized_AST_sha256=sha(ast.dump(normalized,annotate_fields=True,include_attributes=False).encode()),
                            canonical_CONFIG=identity(enc(config)[:-1]), actual_future_render=None)
    # Only this unrendered template is parsed. No future CONFIG or receipt is built.
    output = dict(schema='openergy.r7-independent-source-data-review.v1', execution=False,
                  source=identity(new), basis=identity(old), patch=identity(patch), patch_hunks=2,
                  packet_summaries=packets, declarations=45, unchanged_declarations=44,
                  original_assignments=41, added_assignments=['ROOT_PARENT_ONLY','ROOT_OBSERVED_DATA'],
                  whole_file_restoration_exact=True, original_caps={k:g[k] for k in
                  ('FILE_CAP','TOTAL_CAP','INITIAL_CAP','RAW_CAP','ENTRY_CAP','CONSOLE_CAP','STATUS_CAP','RECEIPT_CAP')},
                  config_non_admitting=True, startup_keys=sorted(startup_keys),
                  actual_future_render=None, actual_future_local_composition=None,
                  actual_operation=None, execution_authority=None,
                  vector_count=50, source_raw_rows=212, source_raw_bytes=213334,
                  saved_requirements=saved_requirements, actual_remote_reader_provenance=provenance,
                  current_unrendered_template=current_template,
                  checks=dict(total=len(CHECKS), passed=sum(r['pass'] for r in CHECKS),
                              failed=[r for r in CHECKS if not r['pass']]))
    (OWN / 'source-data-evidence.json').write_bytes(enc(output))
    (OWN / 'declaration-review.json').write_bytes(enc(dict(execution=False, entries=rows)))
    (OWN / 'layout-evidence.json').write_bytes(enc(layout))
    (OWN / 'input-bindings.json').write_bytes(enc(dict(execution=False, entries=sorted(BINDINGS.values(),key=lambda r:(r['commit'],r['path'])))))
    print(json.dumps({'checks':output['checks'],'bindings':len(BINDINGS),
                      'declarations':len(rows),'layout_digest':layout['initial_names_digest'],
                      'vector_count':50,'saved_requirements':saved_requirements},indent=2))


if __name__ == '__main__':
    main()
