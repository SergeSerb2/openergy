"""File/JSON/Git-object checks only. Never import or execute the controller."""
import ast
import difflib
import hashlib
import json
from pathlib import Path
import subprocess
import traceback

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
PREP_COMMIT = '9490a0e46eadf73785901b33d040d873b34e1157'
REVIEW_COMMIT = '48c41fbaca076fb46a028f3900265ac1a41e11ce'
PREP = 'docs/verification/sparkle-running-prep-2026-10-23-t3-aa79bac9/'
REVIEW = 'docs/verification/sparkle-running-prep-review-2026-10-23-t3-2c45371a.md'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def git_bytes(commit, path):
    return subprocess.check_output(['git', 'show', commit + ':' + path], cwd=REPO)


def check_files():
    controller = HERE / 'controller-DO-NOT-RUN.py.txt'
    proposal = HERE / 'operation-DO-NOT-RUN.json'
    text = controller.read_text()
    tree = ast.parse(text, filename=str(controller))
    plan = json.loads(proposal.read_bytes())
    original = json.loads((HERE / 'inputs/operation-DO-NOT-RUN.json').read_bytes())
    inputs = {}
    for p in sorted((HERE / 'inputs').iterdir()):
        data = p.read_bytes()
        expected = git_bytes(REVIEW_COMMIT, REVIEW) if p.name == 'independent-R2-review.md' else git_bytes(PREP_COMMIT, PREP + p.name)
        assert data == expected, p.name
        inputs[p.name] = digest(data)
    assert inputs == plan['original_inputs']
    assert inputs['controller-DO-NOT-RUN.py.txt'] == 'e251cf7477ed9accf7c28e15de4cabfad50306e12c1934c9b37155a1f40b2740'
    assert inputs['operation-DO-NOT-RUN.json'] == '2d23db1f735e607d70f92e29a8e0837425c4908bd935a3950ba60f458dc6f2a6'
    assert plan['controller']['sha256'] == digest(controller.read_bytes())
    assert plan['controller']['path'] == str(controller)
    assert plan['status'] == 'DO-NOT-RUN' and plan['approval_received'] is False
    assert plan['new_independent_review_received'] is False and plan['runtime_accepted'] is False
    assert len(plan['missing_prerequisites']) == 3
    assert plan['previous_approval']['applies_to_this_operation'] is False
    assert not plan['independent_review_receipt_contract']['receipt_created']

    allowed_changes = {'controller', 'approval_scope', 'final_quit', 'limits', 'owned_cleanup',
                       'supervisor_stop_limit', 'missing_prerequisites'}
    changes = [k for k in original if original[k] != plan[k]]
    assert set(changes) == allowed_changes, changes
    unchanged = [k for k in original if k not in allowed_changes]
    assert all(plan[k] == original[k] for k in unchanged)
    assert all(plan['limits'][k] == v for k, v in original['limits'].items())
    assert plan['limits']['control_cutoff_seconds'] + plan['limits']['recovery_reserved_within_whole_seconds'] == 240
    assert plan['marker']['expiresAt'] - plan['marker']['createdAt'] == 21600
    assert plan['starting_root'] == plan['daemon_source_root'] == 'ad280644a96905d9831825bb4f87d1388dbdca73'
    assert plan['marker']['port'] == 56327
    assert plan['repair_of']['preparation_commit'] == PREP_COMMIT
    assert plan['repair_of']['independent_R2_commit'] == REVIEW_COMMIT
    assert plan['repair_of']['original_controller_sha256'] == inputs['controller-DO-NOT-RUN.py.txt']
    assert plan['repair_of']['original_proposal_sha256'] == inputs['operation-DO-NOT-RUN.json']

    evidence = {}
    for name, expected in original['evidence_hashes'].items():
        directory = 'static/' if name == 'audit.json' else 'raw/'
        actual = digest(git_bytes(PREP_COMMIT, PREP + directory + name))
        assert actual == expected, name
        evidence[name] = actual
    assert digest(git_bytes(PREP_COMMIT, PREP + 'evidence-manifest.json')) == plan['repair_of']['original_evidence_manifest_sha256']
    # Read original source blobs as data in one finite Git batch; no checkout/import/build.
    source_records = list(original['source_hashes'].items())
    object_specs = [(commit, path, expected) for commit in [PREP_COMMIT, original['starting_root']]
                    for path, expected in source_records]
    batch = subprocess.check_output(['git', 'cat-file', '--batch'], cwd=REPO,
                                    input=''.join(commit + ':' + path + '\n' for commit, path, _ in object_specs).encode())
    offset = 0
    for commit, path, expected in object_specs:
        end = batch.index(b'\n', offset)
        header = batch[offset:end].split()
        assert len(header) == 3 and header[1] == b'blob', (commit, path)
        size = int(header[2])
        start = end + 1
        assert digest(batch[start:start + size]) == expected, (commit, path)
        assert batch[start + size:start + size + 1] == b'\n'
        offset = start + size + 1
    assert offset == len(batch)

    # Inspect syntax and literal bindings only. No AST evaluation or controller import.
    literals = {}
    for node in tree.body:
        if isinstance(node, ast.Assign) and len(node.targets) == 1 and isinstance(node.targets[0], ast.Name):
            name = node.targets[0].id
            if isinstance(node.value, ast.Constant):
                literals[name] = node.value.value
            elif isinstance(node.value, ast.Call) and isinstance(node.value.func, ast.Name) and node.value.func.id == 'Path':
                if len(node.value.args) == 1 and isinstance(node.value.args[0], ast.Constant):
                    literals[name] = node.value.args[0].value
    assert literals['ROOT'] == plan['marker']['root']
    assert literals['NONCE'] == plan['marker']['nonce']
    assert literals['IDENTIFIER'] == plan['marker']['fixtureID']
    assert literals['PORT'] == plan['marker']['port']
    assert literals['PREP'] == plan['repair_of']['original_preparation_path']
    names = {n.name: n for n in tree.body if isinstance(n, (ast.FunctionDef, ast.ClassDef))}
    calls = lambda node, name: [n for n in ast.walk(node) if isinstance(n, ast.Call) and
                               isinstance(n.func, ast.Name) and n.func.id == name]
    assert len(calls(names['main'], 'launch_once')) == 2
    assert len(calls(tree, 'normal_quit')) == 1
    assert len(calls(names['main'], 'normal_quit')) == 1
    assert not calls(names['recover'], 'normal_quit')
    assert 'VALIDATED_NEW' in ast.get_source_segment(text, names['normal_quit'])
    assert 'independent_review_receipt_sha256' in ast.get_source_segment(text, names['main'])
    assert len(calls(names['exact_read'], 'remaining')) == 1  # inside the recv loop
    assert 'connection_deadline' in ast.get_source_segment(text, names['Feed'])
    assert 'terminal_stop' in ast.get_source_segment(text, names['hard_deadline'])

    for name, diff_name in [('controller-DO-NOT-RUN.py.txt', 'controller.diff'), ('operation-DO-NOT-RUN.json', 'proposal.diff')]:
        patch = ''.join(difflib.unified_diff((HERE / 'inputs' / name).read_text().splitlines(keepends=True),
                         (HERE / name).read_text().splitlines(keepends=True), fromfile='9490a0e/' + name,
                         tofile='repair-r1/' + name, n=3))
        assert (HERE / diff_name).read_text() == patch
    for p in HERE.rglob('*.json'):
        json.loads(p.read_bytes())
    return dict(result='PASS: AST and static file/JSON/Git-object checks only',
                controller_sha256=digest(controller.read_bytes()), proposal_sha256=digest(proposal.read_bytes()),
                original_inputs=inputs, changed_original_fields=changes, unchanged_original_fields=unchanged,
                all_original_limits_unchanged=True, frozen_source_files=len(source_records),
                source_blob_hash_checks=len(object_specs), original_evidence_hashes=evidence,
                independent_review_received=False, approval_received=False, live_preflight_performed=False,
                controller_imported=False, controller_executed=False, simulation_unit_test_or_mock=False,
                private_artifact_state_reaudited=False,
                note='Syntax and metadata consistency only; no proof of controller behavior or runtime acceptance.')


if __name__ == '__main__':
    try:
        result = check_files()
    except BaseException:
        target = HERE / 'raw/first-static-reader-error.txt'
        if not target.exists():
            target.write_text(traceback.format_exc())
            (HERE / 'raw/controller-at-first-static-reader-error.py.txt').write_bytes((HERE / 'controller-DO-NOT-RUN.py.txt').read_bytes())
        raise
    (HERE / 'static-checks.json').write_text(json.dumps(result, indent=2, sort_keys=True) + '\n')
    print(json.dumps({k: result[k] for k in ['result', 'controller_sha256', 'proposal_sha256', 'source_blob_hash_checks']}, indent=2))
