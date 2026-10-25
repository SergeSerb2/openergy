"""Reviewer-owned bounded data reader. Never imports or runs supplied text.

Only the exact authorized evidence directory is read. Outputs contain hashes
and sanitized data, never copied private scripts, connection vectors or IDs.
"""
import base64
import codecs
import hashlib
import io
import json
import os
from pathlib import Path
import re
import stat
import shlex
import subprocess
import sys
import tokenize

OUT = Path(__file__).absolute().parent
ROOT = '/Users/serge/.local/state/openergy-verification/m1-reviewed-ordinary-canonical-sdk-prepared-20261025-4b20357c1503'
MAX_OPENS = 96
MAX_BYTES = 8 * 1024 * 1024
LEDGER = OUT / 'private-read-ledger.json'
ledger = json.loads(LEDGER.read_text()) if LEDGER.exists() else {'regular_file_opens': 0, 'cumulative_content_bytes': 0, 'passes': []}

def save(name, value):
    (OUT / name).write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')

def sha(data):
    return hashlib.sha256(data).hexdigest()

def stable(s):
    return (s.st_dev, s.st_ino, s.st_mode, s.st_uid, s.st_gid,
            s.st_size, s.st_mtime_ns, s.st_ctime_ns)

def open_root():
    fd = os.open('/', os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        for part in ROOT.strip('/').split('/'):
            new = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=fd)
            os.close(fd)
            fd = new
        return fd
    except BaseException:
        os.close(fd)
        raise

def inventory(fd, prefix=''):
    rows = []
    for name in sorted(os.listdir(fd)):
        if name in ('.', '..') or '/' in name:
            raise ValueError('invalid entry')
        s = os.stat(name, dir_fd=fd, follow_symlinks=False)
        rel = prefix + name
        if stat.S_ISDIR(s.st_mode):
            child = os.open(name, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=fd)
            try:
                rows.extend(inventory(child, rel + '/'))
            finally:
                os.close(child)
        elif stat.S_ISREG(s.st_mode):
            rows.append((rel, s))
        else:
            raise ValueError('non-regular or symlink entry rejected')
    return rows

def read_leaf(rootfd, rel, expected):
    global ledger
    if ledger['regular_file_opens'] + 1 > MAX_OPENS:
        raise ValueError('open budget exceeded')
    # Reserve expected bytes before open; a failed read still consumes budget.
    if ledger['cumulative_content_bytes'] + expected.st_size > MAX_BYTES:
        raise ValueError('byte budget exceeded')
    ledger['regular_file_opens'] += 1
    ledger['cumulative_content_bytes'] += expected.st_size
    save('private-read-ledger.json', ledger)
    parts = rel.split('/')
    fd = os.dup(rootfd)
    try:
        for part in parts[:-1]:
            new = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=fd)
            os.close(fd)
            fd = new
        leaf = os.open(parts[-1], os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=fd)
        try:
            before = os.fstat(leaf)
            if not stat.S_ISREG(before.st_mode) or stable(before) != stable(expected):
                raise ValueError('leaf changed before read')
            remaining = before.st_size
            chunks = []
            while remaining:
                chunk = os.read(leaf, min(65536, remaining))
                if not chunk:
                    raise ValueError('short read')
                chunks.append(chunk)
                remaining -= len(chunk)
            after = os.fstat(leaf)
            linked = os.stat(parts[-1], dir_fd=fd, follow_symlinks=False)
            if stable(before) != stable(after) or stable(after) != stable(linked):
                raise ValueError('leaf changed during read')
            return b''.join(chunks)
        finally:
            os.close(leaf)
    finally:
        os.close(fd)

def sanitize(value):
    # No command vectors, selectors, host endpoints, process IDs or private config.
    sensitive = ('argv', 'command', 'selector', 'pid', 'host', 'alias', 'config')
    if isinstance(value, dict):
        return {k: ('[PRIVATE OMITTED]' if any(x in k.lower() for x in sensitive) else sanitize(v)) for k, v in value.items()}
    if isinstance(value, list):
        return [sanitize(v) for v in value]
    if isinstance(value, str):
        value = re.sub(r'/Users/[^\s\"\']+', '[PRIVATE_PATH]', value)
        value = re.sub(r'\b(?:\d{1,3}\.){3}\d{1,3}\b', '[PRIVATE_ADDRESS]', value)
    return value

def main():
    mode = sys.argv[1]
    if mode not in ('initial', 'final', 'guards', 'analysis', 'expected'):
        raise ValueError('invalid pass')
    fd = open_root()
    try:
        listing = inventory(fd)
        if mode == 'guards':
            listing = [(p, s) for p, s in listing if p.endswith('.guard') or p == 'retained-source-file-facts/source-qualification-guard.json']
        if mode == 'analysis':
            selected = {'source-file-phase.py.txt', 'source-file-executor.py', 'retain-file-facts.py',
                        'operation-disabled.json', 'source-files-once.guard', 'file-facts-retention-once.guard',
                        'file-facts-retention.stdout.bin', 'source-files.stdout.bin'}
            listing = [(p,s) for p,s in listing if p in selected]
        if mode == 'expected':
            listing = [(p,s) for p,s in listing if p == 'expected-source-metadata.json']
        data = {}
        rows = []
        for rel, s in listing:
            content = read_leaf(fd, rel, s)
            data[rel] = content
            rows.append({'path': rel, 'bytes': len(content), 'sha256': sha(content), 'mode': oct(stat.S_IMODE(s.st_mode))})
        if mode in ('initial', 'final'):
            later = inventory(fd)
            if [(p, stable(s)) for p, s in listing] != [(p, stable(s)) for p, s in later]:
                raise ValueError('inventory changed')
    finally:
        os.close(fd)
    save('private-snapshot-' + mode + '.json', {'files': rows, 'regular_files': len(rows), 'bytes': sum(r['bytes'] for r in rows)})
    if mode == 'final':
        original = json.loads((OUT / 'private-snapshot-initial.json').read_text())
        if original['files'] != rows:
            raise ValueError('private snapshot hashes or modes changed')
    if mode == 'guards':
        original = {r['path']:r for r in json.loads((OUT / 'private-snapshot-initial.json').read_text())['files']}
        if not all(r == original[r['path']] for r in rows):
            raise ValueError('guard hashes or modes changed')
    ledger['passes'].append({'pass': mode, 'opens': len(rows), 'bytes': sum(r['bytes'] for r in rows)})
    save('private-read-ledger.json', ledger)
    if mode == 'initial':
        inspect(data)
    if mode == 'final':
        analyze(data)
    if mode == 'analysis':
        original = {r['path']: r for r in json.loads((OUT / 'private-snapshot-initial.json').read_text())['files']}
        assert all(r == original[r['path']] for r in rows)
        for name, value in json.loads((OUT / 'saved-records-sanitized.json').read_text()).items():
            if name not in data:
                data[name] = json.dumps(value).encode()
        for name, row in json.loads(data['file-facts-retention.stdout.bin']).items():
            content = base64.b64decode(row['base64'], validate=True)
            assert sha(content) == original['retained-source-file-facts/' + name]['sha256']
            data['retained-source-file-facts/' + name] = content
        for name in ('source-files.stderr.bin', 'file-facts-retention.stderr.bin'):
            assert original[name]['bytes'] == 0 and original[name]['sha256'] == sha(b'')
            data[name] = b''
        phase = data['source-file-phase.py.txt'].decode()
        encoded = literal(next(x[6:] for x in phase.splitlines() if x.startswith('FILES=')))
        for name, h in encoded.items():
            content = bytes.fromhex(h)
            assert sha(content) == original['inputs/' + name]['sha256'] and len(content) == original['inputs/' + name]['bytes']
            data['inputs/' + name] = content
        analyze(data)
    if mode == 'expected':
        value = json.loads(data['expected-source-metadata.json'])
        digests = {}
        import itertools
        for size in range(1, len(value) + 1):
            for names in itertools.combinations(value, size):
                obj = {n:value[n] for n in names}
                digests[','.join(names)] = sha(json.dumps(obj,sort_keys=True,separators=(',',':'),ensure_ascii=True).encode())
        save('metadata-data-limits.json', {'top_level_noncommands':sanitize({k:v for k,v in value.items() if k != 'commands'}), 'first_command_shape':{k:type(v).__name__ for k,v in value['commands'][0].items()}, 'candidate_subset_digests':digests})
    print(json.dumps({'pass': mode, 'files': len(rows), 'ledger': ledger}))

def inspect(data):
    # JSON and literal text only. The two analysis passes remain reviewer code.
    records = {}
    for name, content in data.items():
        if name.endswith('.json') and not name.startswith('inputs/') and name != 'expected-source-metadata.json':
            records[name] = sanitize(json.loads(content))
        if name.endswith('.guard'):
            records[name] = sanitize(json.loads(content))
    save('saved-records-sanitized.json', records)
    expected = json.loads(data['expected-source-metadata.json'])
    save('expected-shape.json', {k: {'type': type(v).__name__, 'count': len(v) if isinstance(v, (dict, list)) else None} for k, v in expected.items()})
    inputs = {p.removeprefix('inputs/'): b for p, b in data.items() if p.startswith('inputs/')}
    save('input-index.json', [{'path': p, 'bytes': len(b), 'sha256': sha(b)} for p, b in sorted(inputs.items())])
    # Print source with hex literals suppressed and connection-bearing lines hidden.
    for name in ('source-file-phase.py.txt', 'source-file-executor.py', 'retain-file-facts.py'):
        print('TEXT ONLY: ' + name)
        for n, line in enumerate(data[name].decode('utf-8').splitlines(), 1):
            if len(line) > 2000:
                line = '[LONG LITERAL OMITTED sha256=' + sha(line.encode()) + ']'
            elif any(x in line.lower() for x in ('ssh', 'host', 'argv', 'selector', 'pid')) or '/Users/' in line:
                line = '[PRIVATE CONNECTION/PATH/IDENTITY LINE OMITTED]'
            print(str(n) + ': ' + line)
    print('SANITIZED SAVED RECORDS:')
    print(json.dumps(records, indent=2, sort_keys=True))
    print('RETAINED FACTS:')
    for name, content in data.items():
        if name.startswith('retained-source-file-facts/'):
            try:
                val = json.loads(content)
            except (ValueError, UnicodeDecodeError):
                val = {'bytes': len(content), 'sha256': sha(content)}
            print(name, json.dumps(sanitize(val), sort_keys=True))

def literal(text):
    """Decode a finite string/list/dict literal using tokens, never AST/eval."""
    ts = [t for t in tokenize.generate_tokens(io.StringIO(text).readline)
          if t.type not in (tokenize.NEWLINE, tokenize.NL, tokenize.ENDMARKER)]
    at = 0
    def take():
        nonlocal at
        t = ts[at]
        at += 1
        if t.type == tokenize.STRING:
            assert t.string[0] in "'\"" and t.string[-1] == t.string[0]
            assert t.string.isascii()
            return codecs.decode(t.string[1:-1], 'unicode_escape')
        if t.type == tokenize.NUMBER and t.string.isdecimal():
            return int(t.string)
        assert t.string in ('[', '{')
        close = ']' if t.string == '[' else '}'
        result = [] if close == ']' else {}
        while ts[at].string != close:
            key = take()
            if close == '}':
                assert ts[at].string == ':' and key not in result
                at += 1
                result[key] = take()
            else:
                result.append(key)
            if ts[at].string != close:
                assert ts[at].string == ','
                at += 1
        at += 1
        return result
    result = take()
    assert at == len(ts)
    return result

def analyze(data):
    """Saved-data consistency analysis; no supplied source is run or imported."""
    checks = {}
    def check(name, condition):
        checks[name] = bool(condition)
    def js(name):
        return json.loads(data[name])
    def git(ref, path):
        return subprocess.check_output(['git', 'show', ref + ':' + path])
    def pinned(ref, path):
        b = git(ref, path)
        check('working_tree_equals_git:' + path, Path(path).read_bytes() == b)
        return b
    olddir = 'docs/verification/m1-next-reviewed-ordinary-flow-2026-10-25-t3-a8a4ece5/'
    sdkdir = 'docs/verification/m1-sdk-canonical-path-repair-2026-10-25/'
    prep = '3a7335ceaa761af2c6530351658db0ba1787b40a'
    sdk = 'cfe7721ec08adfda2352d903200b33f921472836'
    controller = '2b5fc1f505b1f0426b8f1e8b00537a0b6e805fc7'
    phase = data['source-file-phase.py.txt'].decode()
    lines = phase.splitlines()
    encoded = literal(next(x[6:] for x in lines if x.startswith('FILES=')))
    embedded = {n: bytes.fromhex(h) for n, h in encoded.items()}
    files = {n[len('inputs/'):]: b for n, b in data.items() if n.startswith('inputs/')}
    check('exact_18_embedded_and_materialized_inputs', len(files) == len(embedded) == 18 and files == embedded)
    plan = js('operation-disabled.json')
    check('plan_input_names_exact', set(plan['file_inputs']) == set(files))
    check('plan_disabled_and_no_failed_root_reuse', plan['execution'] is False and plan['no_old_guard_reuse'] is True and plan['old_failed_root_reused'] is False)
    check('plan_controller_and_review', plan['candidate'] == controller and plan['source_review'] == 'b8a422145a5ab5dc197304f6420b85654b5f14ea')
    for n, b in files.items():
        check('plan_input_binding:' + n, plan['file_inputs'][n] == {'bytes': len(b), 'sha256': sha(b)})
    for n in ('file-qualification.recipe.py.txt', 'metadata.recipe.json', 'artifact-receipt.template.json'):
        check('preparation_bytes:' + n, files[n] == pinned(prep, olddir + n))
    check('canonical_recipe_exact_git', files['qualification.recipe.json'] == pinned(sdk, sdkdir + 'qualification.recipe.json'))
    operation = json.loads(pinned(prep, olddir + 'next-operation.json'))
    spec = operation['materialization']
    materialization = json.loads(pinned(spec['commit'], spec['manifest']['path']))
    sourcechecks = json.loads(files['source-checks.json'])['python_files']
    modules = []
    for row in materialization['modules']:
        import posixpath
        n = row['destination']
        path = posixpath.normpath(str(Path(spec['manifest']['path']).parent / row['source']))
        b = pinned(spec['commit'], path)
        binding = sourcechecks[n]
        check('module_bytes_and_declared_binding:' + n,
              b == files[n] and len(b) == binding['bytes'] and len(b.splitlines()) == binding['lines']
              and sha(b) == binding['sha256'] == row['sha256'])
        modules.append({'name': n, 'path': path, 'ref': spec['commit'], 'bytes': len(b), 'lines': len(b.splitlines()), 'sha256': sha(b)})
    check('exact_ten_modules', len(modules) == 10 and set(sourcechecks) == {m['name'] for m in modules} == {n for n in files if n.endswith('.py')})
    for kind, n in [('source_checks','source-checks.json'), ('limits','limits.json'), ('coverage_policy','coverage-policy.json'), ('build_source','build-source-bindings.json')]:
        row = spec[kind]
        b = pinned(row.get('commit', spec['commit']), row['path'])
        check('pinned_binding:' + n, files[n] == b and sha(b) == row['sha256'])
    before = json.loads(pinned(prep, olddir + 'qualification.recipe.json'))
    after = json.loads(files['qualification.recipe.json'])
    changed = []
    def diff(a, b, path=''):
        if isinstance(a, dict) and isinstance(b, dict) and set(a) == set(b):
            for k in a: diff(a[k], b[k], path + '/' + k)
        elif isinstance(a, list) and isinstance(b, list) and len(a) == len(b):
            for i, (x, y) in enumerate(zip(a, b)): diff(x, y, path + '/' + str(i))
        elif a != b: changed.append(path)
    diff(before, after)
    check('only_ten_sdk_path_values_changed', changed == ['/sdk_headers/' + str(i) + '/path' for i in range(10)])
    headers = []
    for a, b in zip(before['sdk_headers'], after['sdk_headers']):
        check('SDK_path_only:' + b['path'], a['path'].replace('/MacOSX27.0.sdk/', '/MacOSX.sdk/') == b['path'] and a['bytes'] == b['bytes'] and a['sha256'] == b['sha256'])
        headers.append({'path': b['path'], 'bytes': b['bytes'], 'sha256': b['sha256']})
    check('source_bundle_prefix_roots_unchanged', before['source_basis'] == after['source_basis'] and before['bundle_basis'] == after['bundle_basis'] and before['prefix_basis'] == after['prefix_basis'])
    for ref, path in [('c490566f5306d79f2537e9bbce3c75db79a279d3','docs/verification/m1-canonical-sdk-binding-review-2026-10-25-t3-14d7a5f6/REVIEW.md'), ('20187fe1b286851f1416437c40cc27108269b542','docs/verification/m1-file-qualification-review-2026-10-25-t3-a6524d66/REVIEW.md'), ('b8a422145a5ab5dc197304f6420b85654b5f14ea','docs/verification/ordinary-combined-context-tuple-review-2026-10-25-t3-4d328640/REVIEW.md')]:
        pinned(ref, path)
    guard = js('source-files-once.guard')
    review = js('source-helper-review.json')
    check('source_guard_payload_and_review', guard['source_bytes'] == len(data['source-file-phase.py.txt']) and guard['source_sha256'] == sha(data['source-file-phase.py.txt']) and guard['source_review'] == review)
    check('review_hashes', review['helper_sha256'] == sha(files['file-qualification.recipe.py.txt']) and review['qualification_recipe_sha256'] == sha(files['qualification.recipe.json']) and review['sdk_data_review_sha256'] == sha(git(review['sdk_data_clear_review'], 'docs/verification/m1-canonical-sdk-binding-review-2026-10-25-t3-14d7a5f6/REVIEW.md')))
    check('review_identity_and_scope', review['source_clear'] is True and review['review_commit'] == '20187fe1b286851f1416437c40cc27108269b542' and review['sdk_data_clear_review'] == 'c490566f5306d79f2537e9bbce3c75db79a279d3' and review['old_root_or_guard_reused'] is False and review['runtime'] == 'UNVERIFIED')
    old_index = json.loads(pinned(sdk, sdkdir + 'private-evidence-index.json'))['records']
    check('executor_unchanged_from_old_saved_binding', sha(data['source-file-executor.py']) == next(r['sha256'] for r in old_index if r['path'] == 'source-file-executor.py'))
    executor = data['source-file-executor.py'].decode()
    def literal_vectors(text):
        ts = list(tokenize.generate_tokens(io.StringIO(text).readline))
        found = []
        for i,t in enumerate(ts):
            if t.string != '=' or i + 1 >= len(ts) or ts[i+1].string != '[':
                continue
            depth = 0
            for j in range(i+1, len(ts)):
                if ts[j].string == '[': depth += 1
                if ts[j].string == ']': depth -= 1
                if depth == 0:
                    try:
                        found.append(literal(tokenize.untokenize([(x.type,x.string) for x in ts[i+1:j+1]])))
                    except (AssertionError,IndexError,ValueError):
                        pass
                    break
        return found
    check('executor_guard_vector_equal', guard['argv'] in literal_vectors(executor))
    remote_guard = js('retained-source-file-facts/source-qualification-guard.json')
    root_literal = literal(next(x[5:] for x in lines if x.startswith('ROOT=')))
    # The helper vector contains ROOT expressions. Its actual guard is checked
    # below from exact literal suffixes rather than evaluating those expressions.
    check('fresh_remote_root_binding', root_literal == plan['remote_evidence'] and root_literal.endswith('m1-ordinary-reviewed-flow-20261025-4b20357c1503'))
    helper_line = next(x for x in lines if x.startswith('argv='))
    expected_helper_vector = ['/usr/bin/python3', '-I', '-S', '-B', plan['remote_evidence'] + '/file-qualification.recipe.py.txt', plan['remote_evidence'], 'source']
    check('helper_guard_isolated_system_python_source', remote_guard['argv'] == expected_helper_vector)
    # Read the finite vector expression by tokens. Only these exact two ROOT
    # constructions are permitted; replacement produces data, never code.
    rhs = helper_line.split('=',1)[1]
    rhs = rhs.replace("str(root/'file-qualification.recipe.py.txt')", repr(plan['remote_evidence'] + '/file-qualification.recipe.py.txt'))
    rhs = rhs.replace('str(root)', repr(plan['remote_evidence']))
    try:
        vector_data = literal(rhs)
    except (AssertionError, ValueError, IndexError):
        vector_data = None
    check('phase_helper_vector_exact_guard', vector_data == remote_guard['argv'])
    check('phase_single_source_helper', phase.count('subprocess.Popen(') == 1 and "'source'" in helper_line and remote_guard['one_attempt'] is True)
    parent = js('source-files-parent-result.json')
    remote_bytes = data['retained-source-file-facts/source-files-os-exit.json']
    remote = json.loads(remote_bytes)
    facts_bytes = data['retained-source-file-facts/qualification-source-files.json']
    facts = json.loads(facts_bytes)
    check('outer_raw_stdout_remote_JSON_equal', json.loads(data['source-files.stdout.bin']) == remote == parent['remote'])
    check('remote_raw_facts_byte_equal', facts_bytes == data['retained-source-file-facts/source-files.stdout.bin'])
    check('facts_identical_in_both_receipts', facts == remote['facts'] == parent['remote']['facts'])
    required = dict(bundle_records=167, mode='source', modules=10, prefix_entries=2006,
                    prefix_regular_bytes=46504040, prefix_root_mode='0o700',
                    prefix_sha256='c46c9234c07785e90b66774ca161f20c7ef72c0a87323f52583fa4edb1186e67',
                    product_invocations=0, runtime='UNVERIFIED', sdk_headers=10, source_files=702,
                    schema='openergy.file-qualification-observation.v1')
    check('exact_fact_keys_and_values', set(facts) == set(required) | {'elapsed_seconds'} and all(facts.get(k) == v for k,v in required.items()) and facts['elapsed_seconds'] >= 0)
    check('facts_SHA256_matches_brief', sha(facts_bytes) == 'ebd466312ed346991a9cdd72c64535dbc0696af82b4061b9d79d91fcc937d192')
    check('actual_source_exits_zero', parent['actual_ssh_exit'] == remote['actual_helper_exit'] == 0)
    check('source_statuses_exact', parent['status'] == remote['status'] == 'SOURCE_FILES_QUALIFIED_RUNTIME_UNVERIFIED')
    timings = {}
    for label, result, g, seconds in [('source_local', parent, guard, 150), ('source_remote', remote, remote_guard, 120)]:
        check(label + '_deadlines_EOF_no_wrapper_error', result['first_error'] is None and result['before_deadline'] is True and result['eof'] == {'stdout': True,'stderr': True} and result['started_monotonic'] == g['started_monotonic'] and result['deadline_monotonic'] == g['deadline_monotonic'] and result['started_monotonic'] < result['exit_observed_monotonic'] < result['deadline_monotonic'] and abs(result['deadline_monotonic'] - result['started_monotonic'] - seconds) < 1e-8)
        timings[label] = {'deadline_seconds': seconds, 'observed_elapsed_seconds': result['exit_observed_monotonic'] - result['started_monotonic']}
    check('no_source_kills_or_retries', parent['local_ssh_kill_called'] is False and remote['local_helper_kill_called'] is False and parent['retries'] == 0)
    check('remote_wrapper_reports_own_NI15', remote['own_nice'] == 15)
    for stream in ('stdout','stderr'):
        b = data['source-files.' + stream + '.bin']
        check('local_stream_binding:' + stream, parent['streams'][stream] == {'bytes': len(b),'sha256': sha(b)})
    check('all_three_stderr_streams_empty', data['source-files.stderr.bin'] == data['retained-source-file-facts/source-files.stderr.bin'] == data['file-facts-retention.stderr.bin'] == b'')
    retention_guard = js('file-facts-retention-once.guard')
    retained = js('file-facts-retention.stdout.bin')
    retention_result = js('file-facts-retention-result.json')
    names = {'qualification-source-files.json','source-files.stdout.bin','source-files.stderr.bin','source-files-os-exit.json','source-qualification-guard.json'}
    check('retention_exact_five_names', set(retained) == names)
    for n, record in retained.items():
        b = data['retained-source-file-facts/' + n]
        check('retention_raw_base64_and_hash:' + n, base64.b64decode(record['base64'], validate=True) == b and record['bytes'] == len(b) and record['sha256'] == sha(b))
    query_vector = shlex.split(retention_guard['argv'][-1])
    query = query_vector[-1]
    check('retention_system_python_isolated', query_vector[:5] == ['/usr/bin/python3', '-I','-S','-B','-c'])
    check('retention_query_and_collector_hashes', sha(query.encode()) == retention_guard['query_sha256'] and sha(data['retain-file-facts.py']) == retention_guard['collector_sha256'])
    check('retention_same_connection_binding', retention_guard['argv'][:-1] == guard['argv'][:-1])
    # Strings embedded in the query are read as tokens; no query execution.
    query_strings = [literal(t.string) for t in tokenize.generate_tokens(io.StringIO(query).readline) if t.type == tokenize.STRING and t.string[0] in "'\""]
    check('retention_query_root_and_five_names', plan['remote_evidence'] in query_strings and names.issubset(set(query_strings)))
    collector = data['retain-file-facts.py'].decode()
    check('collector_exact_five_names', literal(collector.split('NAMES = ',1)[1].split('\n\n',1)[0]) == ['qualification-source-files.json','source-files.stdout.bin','source-files.stderr.bin','source-files-os-exit.json','source-qualification-guard.json'])
    # Reconstruct only the static query string and two literal substitutions.
    marker = "remote = '''"
    template = collector.split(marker,1)[1].split("'''",1)[0]
    reconstructed = template.replace('REMOTE', repr(plan['remote_evidence'])).replace('NAMES', repr(['qualification-source-files.json','source-files.stdout.bin','source-files.stderr.bin','source-files-os-exit.json','source-qualification-guard.json']))
    check('collector_query_literal_exact', reconstructed == query)
    rr = retention_result
    check('retention_actual_exit_status', rr['actual_ssh_exit'] == 0 and rr['status'] == 'FIVE_FILE_FACTS_RETAINED' and rr['files'] == 5 and rr['facts_sha256'] == sha(facts_bytes))
    check('retention_deadline_EOF_no_error', rr['first_error'] is None and rr['eof'] == {'stdout':True,'stderr':True} and rr['before_deadline'] is True and rr['final_check_before_write'] is True and rr['local_kill_called'] is False and rr['retries'] == 0 and rr['started_monotonic'] == retention_guard['started_monotonic'] and rr['deadline_monotonic'] == retention_guard['deadline_monotonic'] and rr['started_monotonic'] < rr['exit_observed_monotonic'] < rr['deadline_monotonic'] and abs(rr['deadline_monotonic'] - rr['started_monotonic'] - 20) < 1e-8)
    timings['retention_local'] = {'deadline_seconds':20, 'observed_elapsed_seconds': rr['exit_observed_monotonic'] - rr['started_monotonic']}
    metadata_recipe = json.loads(files['metadata.recipe.json'])
    # The auxiliary wrapper is not the full historical EXPECTED object pinned
    # by the metadata recipe. Preserve the unequal digest as a proof limit.
    prior = json.loads((OUT / 'audit-checks.json').read_text()) if (OUT / 'audit-checks.json').exists() else {}
    expected_shape = json.loads((OUT / 'expected-shape.json').read_text())
    if 'expected-source-metadata.json' in data:
        raw_expected = data['expected-source-metadata.json']
        canonical_expected_hash = sha(json.dumps(json.loads(raw_expected),sort_keys=True,separators=(',', ':'),ensure_ascii=True).encode())
        raw_expected_hash = sha(raw_expected)
    else:
        canonical_expected_hash = prior['metadata_expected_canonical_sha256']
        raw_expected_hash = prior['metadata_expected_raw_sha256']
    check('metadata_inputs_inert', metadata_recipe['execution'] is False and expected_shape['commands']['count'] == 50)
    receipt = json.loads(files['artifact-receipt.template.json'])
    check('all_ten_artifact_receipt_gates_false', len(receipt['qualification_gates']) == 10 and all(v is False for v in receipt['qualification_gates'].values()))
    save('receipt-shape.json', sanitize({k: v for k,v in receipt.items() if k in ('qualification_gates','schema','execution')}))
    save('audit-checks.json', {'schema':'openergy.canonical-source-file-saved-audit.v1', 'checks': checks, 'failed_checks': [n for n,v in checks.items() if not v], 'timings': timings, 'facts': facts, 'facts_sha256': sha(facts_bytes), 'metadata_expected_raw_sha256': raw_expected_hash, 'metadata_expected_canonical_sha256':canonical_expected_hash, 'informational_comparisons':{'auxiliary_metadata_wrapper_equals_full_recipe_EXPECTED_digest':canonical_expected_hash == metadata_recipe['saved_expected_canonical_json_sha256'], 'full_EXPECTED_digest_declared_only':metadata_recipe['saved_expected_canonical_json_sha256'], 'scope':'Different objects. Auxiliary wrapper binding to full historical EXPECTED remains UNVERIFIED. No metadata command ran or gate was asserted.'}, 'materialized_inputs_bytes':sum(map(len,files.values())), 'modules': modules, 'canonical_sdk_headers':headers, 'changed_recipe_values':changed})
    # Show code structure with all string literals withheld from the transcript.
    for name, text in [('phase',phase),('executor',executor),('retention_query',query)]:
        print('MASKED TEXT ONLY:',name)
        for line in text.splitlines():
            if line.startswith('FILES=') or line.startswith('argv='):
                continue
            try:
                spans = [(t.start[1],t.end[1]) for t in tokenize.generate_tokens(io.StringIO(line).readline) if t.type == tokenize.STRING]
                for a,b in reversed(spans): line = line[:a] + "'[string withheld]'" + line[b:]
                print(line)
            except tokenize.TokenError:
                print('[incomplete line]')
    print(json.dumps({'checks':len(checks),'failed_checks':[n for n,v in checks.items() if not v],'timings':timings}))

if __name__ == '__main__':
    main()
