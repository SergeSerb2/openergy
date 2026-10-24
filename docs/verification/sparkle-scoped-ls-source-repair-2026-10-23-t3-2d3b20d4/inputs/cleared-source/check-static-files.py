"""Static file/hash/JSON/AST checks only. Never import or evaluate controller code."""
import ast
import difflib
import hashlib
import json
from pathlib import Path
import stat
import subprocess
import traceback

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
PREP_COMMIT = '9490a0e46eadf73785901b33d040d873b34e1157'
R2_COMMIT = '48c41fbaca076fb46a028f3900265ac1a41e11ce'
REPAIR_COMMIT = '82cb25b6775d69fe1a312bdc1f26486aba0bf287'
R4_COMMIT = 'cf8127d6da7710adfbf04f07ef8bcdd95e782edf'
PREP = 'docs/verification/sparkle-running-prep-2026-10-23-t3-aa79bac9/'
REPAIR = 'docs/verification/sparkle-running-repair-2026-10-23-t3-bd21f531-r1/'
R2 = 'docs/verification/sparkle-running-prep-review-2026-10-23-t3-2c45371a.md'
R4 = 'docs/verification/sparkle-running-repair-R4-review-2026-10-23-t3-37c4f272.md'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def git_bytes(commit, path):
    return subprocess.check_output(['git', 'show', commit + ':' + path], cwd=REPO)


def functions(tree):
    return {n.name: n for n in tree.body if isinstance(n, (ast.FunctionDef, ast.ClassDef))}


def calls(node, name):
    return [n for n in ast.walk(node) if isinstance(n, ast.Call) and
            isinstance(n.func, ast.Name) and n.func.id == name]


def static_files():
    controller = HERE / 'controller-DO-NOT-RUN.py.txt'
    proposal = HERE / 'operation-DO-NOT-RUN.json'
    text = controller.read_text()
    tree = ast.parse(text, filename=str(controller))
    prior_root = HERE / 'inputs/repair-r1'
    prior_text = (prior_root / controller.name).read_text()
    prior_tree = ast.parse(prior_text)
    plan = json.loads(proposal.read_bytes())
    previous = json.loads((prior_root / proposal.name).read_bytes())
    original = json.loads((HERE / 'inputs' / proposal.name).read_bytes())

    prior_files = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', REPAIR_COMMIT,
                                           '--', REPAIR], cwd=REPO).decode().splitlines()
    assert len(prior_files) == 17
    assert {p.relative_to(prior_root).as_posix() for p in prior_root.rglob('*') if p.is_file()} == {
        p.removeprefix(REPAIR) for p in prior_files}
    for name in prior_files:
        assert (prior_root / name.removeprefix(REPAIR)).read_bytes() == git_bytes(REPAIR_COMMIT, name), name
    prior_freeze = json.loads((prior_root / 'freeze.json').read_bytes())
    assert len(prior_freeze['files']) == 16
    for name, row in prior_freeze['files'].items():
        p = prior_root / name
        assert p.stat().st_size == row['bytes'] and digest(p.read_bytes()) == row['sha256'], name
        assert oct(stat.S_IMODE(p.stat().st_mode)) == row['mode'], name
    assert digest((prior_root / 'freeze.json').read_bytes()) == '2e66b88376fb2594125d6b02331ac42fd5cd991ae86b180533d363a07749a083'
    assert digest((prior_root / controller.name).read_bytes()) == '3d08b11c2dda9256314d6bee0f7da7c7651cd0ea45b3f8dc7f8d3a9b1f0b1066'
    assert digest((prior_root / proposal.name).read_bytes()) == '667c5ef9a362c3319390f0e5e77adc19e7364a34425074edfd08731158774c6b'

    for name, expected in plan['original_inputs'].items():
        p = HERE / 'inputs' / name
        data = git_bytes(R2_COMMIT, R2) if name == 'independent-R2-review.md' else git_bytes(PREP_COMMIT, PREP + name)
        assert p.read_bytes() == data and digest(data) == expected, name
        assert p.read_bytes() == (prior_root / 'inputs' / name).read_bytes()
    assert (HERE / 'inputs/independent-R4-review.md').read_bytes() == git_bytes(R4_COMMIT, R4)
    correction = plan['R4_correction']
    assert correction['repair_commit'] == REPAIR_COMMIT and correction['review_commit'] == R4_COMMIT
    assert correction['review_sha256'] == digest(git_bytes(R4_COMMIT, R4))
    for field, filename in [('previous_controller_sha256', controller.name),
                            ('previous_proposal_sha256', proposal.name), ('previous_freeze_sha256', 'freeze.json')]:
        assert correction[field] == digest((prior_root / filename).read_bytes())

    changed = [k for k, v in previous.items() if plan[k] != v]
    assert set(changed) == {'absolute_deadlines', 'controller', 'identity_ledger', 'missing_prerequisites', 'owned_cleanup'}
    assert set(plan) - set(previous) == {'R4_correction'}
    assert plan['limits'] == previous['limits']
    assert all(plan['limits'][k] == v for k, v in original['limits'].items())
    assert plan['limits']['control_cutoff_seconds'] == 180
    assert plan['limits']['recovery_reserved_within_whole_seconds'] == 60
    assert plan['limits']['whole_runtime_seconds'] == 240
    for k in ['approval_scope', 'final_quit', 'host_effects', 'owned_jobs', 'owned_cache_roots',
              'marker', 'fixture', 'target_environment', 'private_file_hashes', 'source_hashes', 'evidence_hashes',
              'supervisor_stop_limit', 'external_cli_command', 'initial_command', 'signing', 'http_files']:
        assert plan[k] == previous[k], k
    assert plan['starting_root'] == plan['daemon_source_root'] == 'ad280644a96905d9831825bb4f87d1388dbdca73'
    assert plan['marker']['expiresAt'] == 1792828585.1645272
    assert plan['marker']['expiresAt'] - plan['marker']['createdAt'] == 21600
    assert plan['controller']['path'] == str(controller) and plan['controller']['sha256'] == digest(controller.read_bytes())
    assert plan['status'] == 'DO-NOT-RUN' and plan['approval_received'] is False
    assert plan['new_independent_review_received'] is False and plan['runtime_accepted'] is False
    assert correction['independent_review_of_this_candidate_received'] is False
    assert correction['previous_review_is_clearance'] is False
    assert plan['previous_approval']['applies_to_this_operation'] is False
    assert not plan['independent_review_receipt_contract']['receipt_created']
    assert len(plan['missing_prerequisites']) == 3

    # Frozen source/evidence reads use only committed blobs, never private live artifact paths.
    evidence = {}
    for name, expected in original['evidence_hashes'].items():
        path = PREP + ('static/' if name == 'audit.json' else 'raw/') + name
        evidence[name] = digest(git_bytes(PREP_COMMIT, path))
        assert evidence[name] == expected, name
    manifest_hash = digest(git_bytes(PREP_COMMIT, PREP + 'evidence-manifest.json'))
    assert manifest_hash == plan['repair_of']['original_evidence_manifest_sha256']
    records = [(commit, path, expected) for commit in [PREP_COMMIT, original['starting_root']]
               for path, expected in original['source_hashes'].items()]
    blob_data = subprocess.check_output(['git', 'cat-file', '--batch'], cwd=REPO,
                                       input=''.join(c + ':' + p + '\n' for c, p, _ in records).encode())
    offset = 0
    for commit, path, expected in records:
        end = blob_data.index(b'\n', offset)
        header = blob_data[offset:end].split()
        assert len(header) == 3 and header[1] == b'blob', (commit, path)
        size = int(header[2])
        start = end + 1
        assert digest(blob_data[start:start + size]) == expected, (commit, path)
        assert blob_data[start + size:start + size + 1] == b'\n'
        offset = start + size + 1
    assert offset == len(blob_data) and len(records) == 902

    # Syntax/literal inspection only. No exec, eval, import, compile, fake OS, or simulation.
    before, after = functions(prior_tree), functions(tree)
    changed_defs = {'identity', 'IdentityLedger', 'observe_until_stopped', 'fixture_processes', 'recover'}
    unchanged_defs = []
    for name in before:
        if name not in changed_defs:
            assert ast.dump(before[name]) == ast.dump(after[name]), name
            unchanged_defs.append(name)
    assert set(after) - set(before) == {'reconcile_scope', 'require_cache_quiet', 'remove_cache'}
    ledger_before, ledger_after = functions(before['IdentityLedger']), functions(after['IdentityLedger'])
    for name in ['bind_initial', 'validate', 'old_children', 'old_members']:
        assert ast.dump(ledger_before[name]) == ast.dump(ledger_after[name]), name
    raw_sites = calls(tree, 'raw_identity')
    assert len(raw_sites) == 2 and len(calls(after['identity'], 'raw_identity')) == 1
    assert len(calls(ledger_after['observe'], 'raw_identity')) == 1
    assert isinstance(after['identity'].body[0], ast.With)
    assert ast.unparse(after['identity'].body[0].items[0].context_expr) == 'LEDGER.lock if LEDGER is not None else nullcontext()'
    assert isinstance(ledger_after['observe'].body[0], ast.With)
    assert ast.unparse(ledger_after['observe'].body[0].items[0].context_expr) == 'self.lock'
    merges = [n for n in ast.walk(tree) if isinstance(n, ast.Call) and isinstance(n.func, ast.Attribute) and n.func.attr == 'observe']
    assert len(merges) == 1 and merges[0] in list(ast.walk(after['identity']))
    assert 'identity-forward-exec' in ast.get_source_segment(text, ledger_after['add'])
    assert 'unexpected exec' in ast.get_source_segment(text, ledger_after['add'])
    assert len(calls(after['main'], 'launch_once')) == 2
    assert len(calls(tree, 'normal_quit')) == len(calls(after['main'], 'normal_quit')) == 1
    assert not calls(after['recover'], 'normal_quit')
    recovery_text = ast.get_source_segment(text, after['recover'])
    assert recovery_text.index('observer.join') < recovery_text.index("reconcile_scope('final-scope-reconciliation')") < recovery_text.index('RECOVERY_DONE.set()')
    assert 'scoped_exit_outcomes=final_scope' in recovery_text and 'retained_state=bool(cleanup_failure or ERRORS)' in recovery_text
    assert 'LEDGER.scoped()' in ast.get_source_segment(text, after['reconcile_scope'])
    assert 'LEDGER.owned()' not in ast.get_source_segment(text, after['reconcile_scope'])
    removal_text = ast.get_source_segment(text, after['remove_cache'])
    assert 'with LEDGER.lock:' in removal_text and 'require_cache_quiet()' in removal_text
    assert removal_text.index('require_cache_quiet()') < removal_text.index('os.rmdir(')
    assert 'os.O_NOFOLLOW' in removal_text and 'dir_fd=fd' in removal_text
    assert 'shutil.rmtree' not in text
    assert 'cache_started' in recovery_text and 'cache_blocked' in ast.get_source_segment(text, ledger_after['add'])

    for name, diff in [(controller.name, 'controller.diff'), (proposal.name, 'proposal.diff')]:
        expected = ''.join(difflib.unified_diff((prior_root / name).read_text().splitlines(keepends=True),
                          (HERE / name).read_text().splitlines(keepends=True),
                          fromfile='82cb25b/' + name, tofile='repair-r2/' + name, n=3))
        assert (HERE / diff).read_text() == expected, diff
    json_count, ast_count = 0, 0
    for p in HERE.rglob('*'):
        if not p.is_file():
            continue
        if p.suffix == '.json':
            json.loads(p.read_bytes())
            json_count += 1
        if p.name.endswith(('.py', '.py.txt')):
            ast.parse(p.read_text(), filename=str(p))
            ast_count += 1
    return dict(result='PASS: static file/hash/JSON/AST checks only', controller_sha256=digest(controller.read_bytes()),
                proposal_sha256=digest(proposal.read_bytes()), previous_files_exact=17, previous_freeze_entries_verified=16,
                original_inputs_exact=4, R4_review_exact=True, changed_previous_proposal_fields=changed,
                all_limit_values_unchanged=True, frozen_source_files=451, source_blob_hash_checks=len(records),
                original_evidence_hashes=evidence, original_evidence_manifest_sha256=manifest_hash,
                unchanged_controller_AST_definitions=unchanged_defs,
                acquisition_and_merge_sites={'raw_identity': [n.lineno for n in raw_sites], 'observe': [n.lineno for n in merges]},
                static_change_map={name: {'start_line': after[name].lineno, 'end_line': after[name].end_lineno}
                                   for name in ['identity', 'IdentityLedger', 'fixture_processes', 'reconcile_scope', 'require_cache_quiet', 'remove_cache', 'recover']},
                JSON_files_parsed=json_count, AST_files_parsed=ast_count,
                controller_imported_or_executed=False, simulation_test_or_mock=False, live_preflight_performed=False,
                private_artifact_state_reaudited=False, new_independent_review_received=False, approval_received=False,
                note='Syntax and frozen-file consistency only. Static guard assertions are not execution, concurrency, cleanup, timing, or independent review proof.')


if __name__ == '__main__':
    try:
        result = static_files()
    except BaseException:
        p = HERE / 'raw/first-static-reader-error.txt'
        if not p.exists():
            p.write_text(traceback.format_exc())
            for name in ['check-static-files.py', 'controller-DO-NOT-RUN.py.txt', 'operation-DO-NOT-RUN.json']:
                (HERE / 'raw' / ('at-first-static-reader-error-' + name)).write_bytes((HERE / name).read_bytes())
        raise
    (HERE / 'static-checks.json').write_text(json.dumps(result, indent=2, sort_keys=True) + '\n')
    print(json.dumps({k: result[k] for k in ['result', 'controller_sha256', 'proposal_sha256', 'source_blob_hash_checks']}, indent=2))
