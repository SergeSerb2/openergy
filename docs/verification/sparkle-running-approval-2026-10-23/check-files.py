"""Read frozen files only. Never import or execute the supplied operation."""
import hashlib
import json
import os
from pathlib import Path
import stat
import subprocess
from datetime import datetime
from zoneinfo import ZoneInfo

REPO = Path(__file__).resolve().parents[3]
OUT = Path(__file__).resolve().parent
REL = Path('docs/verification/sparkle-running-repair-2026-10-23-t3-6133e534-r2')
CANDIDATE = REPO / REL
ORIGINAL = Path('/Users/serge/.t3/worktrees/openergy/t3-6133e534') / REL
PREP = Path('/Users/serge/.t3/worktrees/openergy/t3-aa79bac9/docs/verification/sparkle-running-prep-2026-10-23-t3-aa79bac9')
REVIEW_REL = Path('docs/verification/sparkle-running-repair-R6-review-2026-10-23-t3-bbbc0189.md')
SOURCE_COMMIT = '95c59a27be0f9c95ed207eac4180193a493bb5f2'
REVIEW_COMMIT = '6ad28c7835a09dc40a1a8141caa1551bc91a9979'
CONTROLLER = 'a934611e4bef5d4b355ae11279d6b1aaef0dadf665503efcb788df4869453567'
PROPOSAL = 'cb15b124d6c76c77bad90d05bcc4ea1384aa55aaac48c959bb03ff3cfceba41d'


def digest(path):
    value = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            value.update(block)
    return value.hexdigest()


def git_blob(commit, path):
    return subprocess.check_output(['git', 'show', f'{commit}:{path.as_posix()}'], cwd=REPO)


def inventory(root):
    result = {}
    total = 0
    for path in sorted(root.rglob('*')):
        assert len(result) < 8192
        info = path.lstat()
        assert info.st_uid == os.getuid()
        key = path.relative_to(root).as_posix()
        if stat.S_ISLNK(info.st_mode):
            assert path.resolve(strict=True).is_relative_to(root.resolve(strict=True))
            result[key] = {'symlink': os.readlink(path)}
        elif stat.S_ISREG(info.st_mode):
            assert info.st_nlink == 1
            total += info.st_size
            assert total <= 256 * 1024 * 1024
            result[key] = {'bytes': info.st_size, 'mode': oct(stat.S_IMODE(info.st_mode)), 'sha256': digest(path)}
        else:
            assert stat.S_ISDIR(info.st_mode)
            result[key] = {'directory': True, 'mode': oct(stat.S_IMODE(info.st_mode))}
    return result


assert digest(CANDIDATE / 'controller-DO-NOT-RUN.py.txt') == CONTROLLER
assert digest(CANDIDATE / 'operation-DO-NOT-RUN.json') == PROPOSAL
freeze = json.loads((CANDIDATE / 'freeze.json').read_text())
assert len(freeze['files']) == 35
expected_paths = set(freeze['files']) | {'freeze.json'}
for directory in [CANDIDATE, ORIGINAL]:
    actual = {p.relative_to(directory).as_posix() for p in directory.rglob('*') if not p.is_dir()}
    assert actual == expected_paths
    for name in sorted(expected_paths):
        path = directory / name
        info = path.lstat()
        assert stat.S_ISREG(info.st_mode)
        assert path.read_bytes() == git_blob(SOURCE_COMMIT, REL / name)
        if name != 'freeze.json':
            expected = freeze['files'][name]
            assert info.st_size == expected['bytes']
            assert oct(stat.S_IMODE(info.st_mode)) == expected['mode']
            assert digest(path) == expected['sha256']
review_path = REPO / REVIEW_REL
assert review_path.read_bytes() == git_blob(REVIEW_COMMIT, REVIEW_REL)
review_sha = digest(review_path)
plan = json.loads((CANDIDATE / 'operation-DO-NOT-RUN.json').read_text())
assert plan['approval_received'] is False and plan['runtime_accepted'] is False
assert not plan['packaging_blockers']
for relative, expected in plan['source_hashes'].items():
    assert digest(PREP.parents[2] / relative) == expected, relative
for name, expected in plan['evidence_hashes'].items():
    assert digest(PREP / ('static' if name == 'audit.json' else 'raw') / name) == expected, name
for name, expected in plan['private_file_hashes'].items():
    assert digest(Path(name)) == expected, name
root = Path(plan['fixture']['root'])
assert root.resolve(strict=True) == root
root_info = root.lstat()
assert root_info.st_uid == os.getuid() and stat.S_IMODE(root_info.st_mode) == 0o700
assert json.loads((root / 'fixture.json').read_text()) == plan['marker']
expected_bundles = json.loads((PREP / 'raw/bundle-inventories.json').read_text())
counts = {}
for role, relative in [('old', 'target/Openergy.app'), ('candidate', 'candidate/Openergy.app')]:
    actual = inventory(root / relative)
    assert actual == expected_bundles[role], role
    counts[role] = len(actual)
driver = inventory(root / 'driver/sparkle.app')
assert driver == json.loads((PREP / 'raw/driver-inventory.json').read_text())
counts['driver'] = len(driver)
receipt = {
    'schema': 1,
    'result': 'clear_for_approval',
    'independent_review_complete': True,
    'unresolved_findings': [],
    'controller_sha256': CONTROLLER,
    'proposal_sha256': PROPOSAL,
    'candidate_commit': SOURCE_COMMIT,
    'independent_review_commit': REVIEW_COMMIT,
    'independent_review_document': str(review_path),
    'independent_review_document_sha256': review_sha,
    'receipt_prepared_by': 'parent; records the separate R6 reviewer result',
    'execution_approval_granted': False,
    'runtime_verified': False,
    'live_preflight_passed': False,
}
report = {
    'checked_at': datetime.now(ZoneInfo('America/Vancouver')).isoformat(),
    'result': 'PASS file-only binding refresh',
    'review_document_sha256': review_sha,
    'freeze_entries_each_copy': 35,
    'git_bound_files_each_copy': 36,
    'source_hashes': len(plan['source_hashes']),
    'evidence_hashes': len(plan['evidence_hashes']),
    'private_file_hashes': len(plan['private_file_hashes']),
    'bundle_inventory_entries': counts,
    'controller_sha256': CONTROLLER,
    'proposal_sha256': PROPOSAL,
    'approval_received': False,
    'live_preflight_passed': False,
    'limits': 'File reads and read-only git show only. No supplied code, process/port/job/cache/OS query, signature verifier, runtime, build, signing, network or private signing key access. This is neither a new independent source review nor a live preflight.',
}
for name, value in [('independent-review-receipt.json', receipt), ('file-bindings.json', report)]:
    with (OUT / name).open('x') as stream:
        stream.write(json.dumps(value, indent=2, sort_keys=True) + '\n')
print(json.dumps({'result': report['result'], 'source_hashes': report['source_hashes'], 'bundle_inventory_entries': counts,
                  'review_receipt_sha256': digest(OUT / 'independent-review-receipt.json')}, sort_keys=True))
