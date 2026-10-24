"""Freeze file identities and preservation results. No runtime controller import/execution."""
import ast
import hashlib
import json
import os
from pathlib import Path
import struct
import subprocess

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
RAW = HERE / 'raw'
ROOT = Path(json.loads((HERE / 'fixture.json').read_text())['root'])
os.umask(0o077)


def sha(p):
    with p.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def save(p, value):
    with p.open('x') as f:
        json.dump(value, f, indent=2, sort_keys=True)
        f.write('\n')


plan = json.loads((HERE / 'operation-DO-NOT-RUN.json').read_text())
assert plan['approval_received'] is False and plan['status'] == 'DO-NOT-RUN'
assert sha(HERE / 'controller-DO-NOT-RUN.py.txt') == plan['controller']['sha256']
for name, h in plan['source_hashes'].items():
    assert sha(REPO / name) == h, name
for name, h in plan['private_file_hashes'].items():
    assert sha(Path(name)) == h, name
for name, h in plan['evidence_hashes'].items():
    assert sha(HERE / ('static' if name == 'audit.json' else 'raw') / name) == h, name
history = json.loads((RAW / 'history-before.json').read_text())
assert all(sha(REPO / name) == h for name, h in history.items())
assert not (ROOT / 'disposable-seed').exists()
assert not (ROOT / 'public-verifier').exists()
assert not (ROOT / 'state/control.sock').exists()
assert not any((ROOT / 'receipts').iterdir())
assert not any((ROOT / 'runtime-evidence').iterdir())
assert not (ROOT / 'runtime-once.lock').exists()
assert not (ROOT / 'runtime-approval.json').exists()
ast.parse((HERE / 'controller-DO-NOT-RUN.py.txt').read_text())

trees = json.loads((RAW / 'bundle-inventories.json').read_text())
driver = json.loads((RAW / 'driver-inventory.json').read_text())
for label, folder in [('old', 'target'), ('candidate', 'candidate')]:
    bundle = ROOT / folder / 'Openergy.app'
    for path, record in trees[label].items():
        p = bundle / path
        if 'sha256' in record:
            assert sha(p) == record['sha256'], path
        elif 'symlink' in record:
            assert os.readlink(p) == record['symlink'], path
for path, record in driver.items():
    if 'sha256' in record:
        assert sha(ROOT / 'driver/sparkle.app' / path) == record['sha256']

# Binding the executable code of the new driver to its retained repaired copy.
def code_identity(path):
    data = bytearray(path.read_bytes())
    count = struct.unpack_from('<I', data, 16)[0]
    offset = 32
    for _ in range(count):
        command, length = struct.unpack_from('<II', data, offset)
        if command == 0x1d:
            start, size = struct.unpack_from('<II', data, offset + 8)
            data[offset + 8:offset + 16] = bytes(8)
            return hashlib.sha256(data[:start]).hexdigest()
        offset += length
    raise AssertionError('code signature absent')


old_driver = Path('/private/tmp/oe-stopped-update-llil939f/driver/sparkle.app/Contents/MacOS/sparkle')
new_driver = ROOT / 'driver/sparkle.app/Contents/MacOS/sparkle'
driver_code = dict(retained=code_identity(old_driver), copied=code_identity(new_driver))
assert driver_code['retained'] == driver_code['copied']
save(RAW / 'driver-code-identity.json', dict(definition='same normalization as common-code-identity.json; no file change', **driver_code))

samples = json.loads((RAW / 'daemon-build-priority.json').read_text())
known = {p['pid']: p for s in samples for p in s['owned']}
remaining, reused = [], []
for pid, old in known.items():
    p = subprocess.run(['/bin/ps', '-p', str(pid), '-o', 'pid=,ppid=,ni=,comm='], capture_output=True, text=True)
    if p.returncode == 0:
        line = p.stdout.strip().split(None, 3)
        current = dict(pid=int(line[0]), ppid=int(line[1]), nice=int(line[2]), path=line[3])
        (remaining if current['path'] == old['path'] else reused).append(dict(old=old, current=current))
save(RAW / 'compiler-process-postflight.json', dict(remaining_matching=remaining, reused_PID_different_path=reused,
    limit='Compiler sampling saved PID/path, not kernel start times. No signal sent. Not target runtime proof.'))
assert not remaining

report = dict(
    source_root=plan['starting_root'], daemon_source_root=plan['daemon_source_root'],
    private_root=str(ROOT), proposal_sha256=sha(HERE / 'operation-DO-NOT-RUN.json'),
    controller_sha256=plan['controller']['sha256'], approval_received=False,
    runtime_accepted=False, packaging_blockers=plan['packaging_blockers'], missing_prerequisites=plan['missing_prerequisites'],
    public_files=plan['http_files'], driver_executable_sha256=sha(new_driver), driver_unsigned_code_identity=driver_code,
    canonical_tree_identity_definition='SHA256 of compact sorted-key JSON of raw bundle/driver inventory dictionary',
    old_tree_identity_sha256=digest(trees['old']), candidate_tree_identity_sha256=digest(trees['candidate']),
    driver_tree_identity_sha256=digest(driver),
    executables={name: {path: row for path, row in tree.items() if path in [
        'Contents/MacOS/Openergy', 'Contents/MacOS/OpenergyPrivateLauncher', 'Contents/Helpers/openergy']}
        for name, tree in trees.items()},
    preserved_historical_files=len(history), historical_bytes_unchanged=True,
    daemon_compile=dict(result=json.loads((RAW / 'daemon-build-result.json').read_text()), samples=len(samples),
        observed_descendants=sum(len(s['owned']) for s in samples),
        actual_rustc_observations=sum('rustc' in p['path'] for s in samples for p in s['owned']),
        all_observed_nice15=all(p['nice'] == 15 for s in samples for p in s['owned']),
        stripping_substep='SIGABRT/libLLVM failure retained; overall cargo exit0; not retried'),
    static_audit=dict(commands=len(json.loads((HERE / 'static/audit.json').read_text())['records']),
        code_seals='nested and deep/strict all-architectures', no_library_validation_exception=True,
        fixture_packager_invocations=2, fixture_packager_failures=0),
    target_or_daemon_or_cli_or_helper_or_installer_executed=False,
    controller_imported_or_executed=False, server_bound=False, feed_requested=False,
    disposable_key_absent=True, units_or_mocks_written_or_run=False,
    qualification='file-only preparation of DEBUG Swift fixture; no final release or runtime acceptance')
save(HERE / 'artifact-report.json', report)
files = [dict(path=str(p.relative_to(HERE)), bytes=p.stat().st_size, sha256=sha(p))
         for p in sorted(HERE.rglob('*')) if p.is_file() and p.name != 'evidence-manifest.json']
save(HERE / 'evidence-manifest.json', dict(schema=1, records=files, inventory_sha256=digest(files),
    excludes=['evidence-manifest.json'], definition='SHA256 compact sorted-key JSON records array'))
print(json.dumps({k: report[k] for k in ['proposal_sha256', 'controller_sha256', 'driver_executable_sha256',
    'old_tree_identity_sha256', 'candidate_tree_identity_sha256', 'driver_tree_identity_sha256']}))
