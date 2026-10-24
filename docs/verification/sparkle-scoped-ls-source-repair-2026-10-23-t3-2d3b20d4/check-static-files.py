"""Task-authored file/Git/hash/JSON/AST/patch reader. No supplied code execution.

This script reads source as bytes or syntax trees. It does not import, evaluate,
compile to executable code, simulate or run the supplied controller/helper or
historical checkers. Git subprocesses only read committed objects/status.
"""
import ast
import difflib
import hashlib
import json
from pathlib import Path
import re
import stat
import subprocess

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
BASE = '1cc4d03113d8d78be84ad0fbc161610860455f35'
CLEARED = '95c59a27be0f9c95ed207eac4180193a493bb5f2'
SOURCE = 'docs/verification/sparkle-running-repair-2026-10-23-t3-6133e534-r2'
R6 = '6ad28c7835a09dc40a1a8141caa1551bc91a9979'
R6_PATH = 'docs/verification/sparkle-running-repair-R6-review-2026-10-23-t3-bbbc0189.md'
FAILURE = 'docs/verification/sparkle-running-preflight-failure-2026-10-23'


def require(value, reason):
    if not value:
        raise AssertionError(reason)


def git(*arguments):
    return subprocess.check_output(['git', *arguments], cwd=REPO)


def blob(commit, name):
    return git('show', commit + ':' + name)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def unique(pairs):
    value = {}
    for key, item in pairs:
        require(key not in value, 'duplicate JSON key: ' + key)
        value[key] = item
    return value


def parse_json(data):
    def reject_constant(value):
        raise AssertionError('nonfinite JSON: ' + value)
    return json.loads(data, object_pairs_hook=unique, parse_constant=reject_constant)


def syntax(data, name):
    return ast.parse(data, filename=name)  # syntax only; never evaluate a tree


def dump(node):
    return ast.dump(node, include_attributes=False)


def materialize(original, patch):
    """Apply exact unified hunks in memory as text, without invoking any source."""
    base = original.splitlines(keepends=True)
    lines = patch.splitlines(keepends=True)
    require(lines[0].startswith('--- ') and lines[1].startswith('+++ '), 'patch header')
    position, index, output = 0, 2, []
    while index < len(lines):
        match = re.fullmatch(r'@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@\n', lines[index])
        require(match is not None, 'hunk header')
        old_start, old_count, new_start, new_count = match.groups()
        old_start, old_count = int(old_start), int(old_count or 1)
        new_start, new_count = int(new_start), int(new_count or 1)
        start = old_start - 1 if old_count else old_start
        require(position <= start <= len(base), 'ordered hunk range')
        output.extend(base[position:start])
        require(len(output) == (new_start - 1 if new_count else new_start), 'new hunk offset')
        position, consumed, produced = start, 0, 0
        index += 1
        while index < len(lines) and not lines[index].startswith('@@ '):
            line = lines[index]
            require(line[:1] in (' ', '+', '-'), 'hunk line kind')
            kind, text = line[0], line[1:]
            if kind in (' ', '-'):
                require(position < len(base) and base[position] == text, 'exact hunk context')
                position += 1
                consumed += 1
            if kind in (' ', '+'):
                output.append(text)
                produced += 1
            index += 1
        require((consumed, produced) == (old_count, new_count), 'hunk lengths')
    output.extend(base[position:])
    return ''.join(output)


def main():
    result = dict(schema=1, kind='static source checks only', runtime_accepted=False,
                  independent_clearance=False, supplied_code_imported_or_executed=False,
                  validation='File/Git/hash/strict JSON/Python AST and in-memory patch materialization only')
    old = parse_json((HERE / 'inputs/cleared-source/operation-DO-NOT-RUN.json').read_bytes())
    plan = parse_json((HERE / 'operation-DO-NOT-RUN.json').read_bytes())
    records = []
    source_names = git('ls-tree', '-r', '--name-only', CLEARED, '--', SOURCE).decode().splitlines()
    require(len(source_names) == 36, 'cleared source file count')
    original_dir = Path('/Users/serge/.t3/worktrees/openergy/t3-6133e534') / SOURCE
    for name in source_names:
        relative = name[len(SOURCE) + 1:]
        expected = blob(CLEARED, name)
        require((HERE / 'inputs/cleared-source' / relative).read_bytes() == expected, 'cleared copy ' + relative)
        require((original_dir / relative).read_bytes() == expected, 'cleared original unchanged ' + relative)
        records.append(dict(path=relative, bytes=len(expected), sha256=digest(expected)))
    require(digest((HERE / 'inputs/cleared-source/controller-DO-NOT-RUN.py.txt').read_bytes()) ==
            'a934611e4bef5d4b355ae11279d6b1aaef0dadf665503efcb788df4869453567', 'cleared controller identity')
    require(digest((HERE / 'inputs/cleared-source/operation-DO-NOT-RUN.json').read_bytes()) ==
            'cb15b124d6c76c77bad90d05bcc4ea1384aa55aaac48c959bb03ff3cfceba41d', 'cleared proposal identity')
    freeze = parse_json((HERE / 'inputs/cleared-source/freeze.json').read_bytes())
    # The original manifest's exact schema and records remain data.
    result['cleared_source_files_byte_exact_and_original_unchanged'] = records
    result['cleared_freeze_sha256'] = digest((HERE / 'inputs/cleared-source/freeze.json').read_bytes())
    result['cleared_freeze_schema_keys'] = sorted(freeze)
    prior_freezes = []
    for manifest in sorted((HERE / 'inputs/cleared-source').rglob('freeze.json')):
        prior = parse_json(manifest.read_bytes())
        actual_names = {path.relative_to(manifest.parent).as_posix() for path in manifest.parent.rglob('*')
                        if path.is_file() and path != manifest}
        require(actual_names == set(prior['files']), 'prior freeze exact file set')
        for name, entry in prior['files'].items():
            path = manifest.parent / name
            data = path.read_bytes()
            require(len(data) == entry['bytes'] and digest(data) == entry['sha256'] and
                    oct(stat.S_IMODE(path.stat().st_mode)) == entry['mode'], 'prior freeze entry ' + name)
        prior_freezes.append(dict(path=manifest.relative_to(HERE).as_posix(), entries=len(prior['files']),
                                 sha256=digest(manifest.read_bytes())))
    result['prior_freezes_verified'] = prior_freezes
    for name, expected in plan['original_inputs'].items():
        data = (HERE / 'inputs' / name).read_bytes()
        require(digest(data) == expected and data == (HERE / 'inputs/cleared-source/inputs' / name).read_bytes(), 'input binding ' + name)
    require((HERE / 'inputs/independent-R6-review.md').read_bytes() == blob(R6, R6_PATH), 'full R6 copy')
    failure_names = git('ls-tree', '-r', '--name-only', BASE, '--', FAILURE).decode().splitlines()
    for name in failure_names:
        require((REPO / name).read_bytes() == blob(BASE, name), 'first failure original unchanged ' + name)
    for path in (HERE / 'inputs/first-failure').rglob('*'):
        if path.is_file():
            relative = path.relative_to(HERE / 'inputs/first-failure').as_posix()
            require(path.read_bytes() == blob(BASE, FAILURE + '/' + relative), 'first failure copy ' + relative)
    require(digest(blob(BASE, FAILURE + '/raw/approval.json')) ==
            'af28e5dfd12c05f9a464bf4180e9e7361d5fb85f5510cae70a1eb9946b8abeee', 'actual closed receipt unchanged')
    result['first_failure_original_files_unchanged'] = len(failure_names)
    result['closed_approval_receipt_reused_or_created'] = False

    changed = sorted(key for key in old if old[key] != plan[key])
    added = sorted(set(plan) - set(old))
    require(changed == ['controller', 'missing_prerequisites', 'proof_limits', 'readonly_pre_post_evidence'], 'narrow proposal changes')
    require(added == ['launchservices_capture', 'scoped_LS_correction'], 'narrow added fields')
    require(old['limits'] == plan['limits'] and len(plan['source_hashes']) == 451, 'frozen limits/source map')
    for key in ('approval_received', 'new_independent_review_received', 'runtime_accepted'):
        require(plan[key] is False, 'false authoring gate ' + key)
    require(plan['fixture']['approval_received'] is False and plan['status'] == 'DO-NOT-RUN', 'fixture/status authoring gates')
    for key in ('runnable', 'clear_for_approval', 'runtime_authorized', 'closed_invocation_reusable'):
        require(plan['scoped_LS_correction'][key] is False, 'false prospective gate ' + key)
    for key, filename in [('controller', 'controller-DO-NOT-RUN.py.txt'), ('launchservices_capture', 'launchservices-DO-NOT-RUN.py.txt')]:
        binding = plan[key] if key == 'controller' else plan[key]['helper']
        require(binding['path'] == str(HERE / filename) and binding['sha256'] == digest((HERE / filename).read_bytes()), 'corrected source binding ' + filename)
    result['proposal_changed_fields'] = changed
    result['proposal_added_fields'] = added
    result['all_other_proposal_fields_equal'] = True
    result['full_limits_object_equal'] = True
    result['source_hash_records_preserved'] = len(plan['source_hashes'])

    before = syntax((HERE / 'inputs/cleared-source/controller-DO-NOT-RUN.py.txt').read_bytes(), 'cleared controller')
    after = syntax((HERE / 'controller-DO-NOT-RUN.py.txt').read_bytes(), 'corrected controller')
    require(len(before.body) == len(after.body), 'same controller top-level nodes')
    same_nodes = []
    for index, (prior, current) in enumerate(zip(before.body, after.body)):
        name = getattr(prior, 'name', None)
        if index == 0:
            require(isinstance(prior, ast.Expr) and isinstance(current, ast.Expr), 'module review docstring')
        elif name == 'os_evidence':
            require(dump(prior.body[0]) == dump(current.body[0]) and
                    dump(prior.body[1]) == dump(current.body[1]), 'unchanged job/listener capture')
        else:
            require(dump(prior) == dump(current), 'unchanged controller AST ' + str(name or index))
            if name:
                same_nodes.append(name)
    result['controller_changed_AST_nodes'] = ['module docstring', 'os_evidence']
    result['controller_unchanged_functions_classes'] = same_nodes
    controller_text = (HERE / 'controller-DO-NOT-RUN.py.txt').read_text()
    require("'-dump'" not in controller_text and '/Support/lsregister' not in controller_text, 'global LS reader removed')
    helper = syntax((HERE / 'launchservices-DO-NOT-RUN.py.txt').read_bytes(), 'scoped helper')
    constants = {node.targets[0].id: node.value for node in helper.body if isinstance(node, ast.Assign)
                 and len(node.targets) == 1 and isinstance(node.targets[0], ast.Name)}
    def tuple_strings(name):
        node = constants[name]
        require(isinstance(node, ast.Tuple) and all(isinstance(item, ast.Constant) and isinstance(item.value, str) for item in node.elts), 'literal tuple ' + name)
        return [item.value for item in node.elts]
    require(tuple_strings('IDS') == plan['launchservices_capture']['bundle_identifiers'] ==
            [plan['fixture']['identifier'], plan['fixture']['driver_id']], 'only frozen IDs')
    require(tuple_strings('OWNED_ROOTS') == plan['launchservices_capture']['owned_roots'] ==
            [plan['fixture']['root']] + plan['owned_cache_roots'], 'only frozen reporting roots')
    native_queries = [node.lineno for node in ast.walk(helper) if isinstance(node, ast.Call) and
                      isinstance(node.func, ast.Attribute) and node.func.attr == 'LSCopyApplicationURLsForBundleIdentifier']
    require(len(native_queries) == 1, 'one native query call site; no fallback')
    result['helper_literal_IDs_and_roots_match'] = True
    result['helper_native_query_call_site_lines'] = native_queries
    result['output_envelope_bytes_by_source_constants'] = 811008
    result['output_publication_cap_bytes'] = 1048576

    patch_records = []
    for filename, patchname, new_file in [('controller-DO-NOT-RUN.py.txt', 'controller.diff', False),
                                          ('operation-DO-NOT-RUN.json', 'proposal.diff', False),
                                          ('launchservices-DO-NOT-RUN.py.txt', 'helper.diff', True)]:
        original = '' if new_file else (HERE / 'inputs/cleared-source' / filename).read_text()
        corrected = (HERE / filename).read_text()
        patch = (HERE / patchname).read_text()
        expected = ''.join(difflib.unified_diff(original.splitlines(keepends=True), corrected.splitlines(keepends=True),
                          fromfile='/dev/null' if new_file else 'a/' + filename, tofile='b/' + filename))
        require(patch == expected and materialize(original, patch) == corrected, 'exact patch materialization ' + patchname)
        patch_records.append(dict(path=patchname, sha256=digest(patch.encode()), materialized_bytes_equal=True))
    result['patches'] = patch_records

    parsed_json, parsed_python = [], []
    for path in sorted(HERE.rglob('*')):
        require(not path.is_symlink(), 'no symlink input/source')
        if not path.is_file():
            continue
        require(stat.S_IMODE(path.stat().st_mode) == 0o644, 'inert mode ' + str(path))
        relative = path.relative_to(HERE).as_posix()
        if path.suffix == '.json' and path.name not in ('static-checks.json',) and path != HERE / 'freeze.json':
            parse_json(path.read_bytes())
            parsed_json.append(relative)
        if path.name.endswith(('.py', '.py.txt')):
            syntax(path.read_bytes(), relative)
            parsed_python.append(relative)
        if not relative.startswith('inputs/') and path.suffix != '.diff':
            require(all(line == line.rstrip() for line in path.read_text().splitlines()), 'authored trailing whitespace ' + relative)
    sdk = parse_json((HERE / 'api/sdk-evidence.json').read_bytes())
    for item in sdk['headers'] + sdk['linkage']:
        data = Path(item['path']).read_bytes()
        require(digest(data) == item['sha256'] and len(data) == item['bytes'], 'SDK header identity')
        lines = data.decode().splitlines()
        for excerpt in item['excerpts']:
            require(excerpt['text'] == '\n'.join(lines[excerpt['first'] - 1:excerpt['last']]) + '\n', 'SDK excerpt')
    result['SDK_headers_checked_as_data'] = len(sdk['headers'])
    result['SDK_linker_stubs_checked_as_text_only'] = len(sdk['linkage'])
    result['JSON_files_parsed_excluding_new_result_and_root_freeze'] = parsed_json
    result['Python_files_AST_parsed_without_evaluation'] = parsed_python
    result['authored_whitespace_clean_excluding_exact_inputs_and_unified_patch_context'] = True
    result['source_sha256'] = {name: digest((HERE / name).read_bytes()) for name in
                              ['controller-DO-NOT-RUN.py.txt', 'launchservices-DO-NOT-RUN.py.txt', 'operation-DO-NOT-RUN.json']}
    result['all_static_assertions_passed'] = True
    output = HERE / 'static-checks.json'
    output.write_text(json.dumps(result, indent=2, sort_keys=True) + '\n')
    output.chmod(0o644)
    parse_json(output.read_bytes())
    print(json.dumps(dict(all_static_assertions_passed=True, unchanged_controller_functions_classes=len(same_nodes),
                         cleared_files=len(records), first_failure_files=len(failure_names),
                         python_AST_files=len(parsed_python), exact_materialized_patches=len(patch_records),
                         result_sha256=digest(output.read_bytes())), sort_keys=True))


if __name__ == '__main__':
    main()
