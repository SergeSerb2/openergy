"""Read-only final audit and evidence export. Never starts a build or product."""
from collections import Counter, defaultdict
import gzip
import hashlib
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parent
REPO = Path('/Users/serge/.t3/worktrees/openergy/t3-8c51bae6')
DEST = REPO / 'docs/verification/release-priority-isolated-2026-10-23-t3-8c51bae6'
SOURCE = 'd3b30e1719dae5c885ded57cc7523b92d9686adc'
BASE = 'b713d7fbe480a1a3ecadd737ae5de58256169c66'


def read(name):
    return json.loads((ROOT / name).read_text())


def sha(data):
    return hashlib.sha256(data).hexdigest()


def save(name, value):
    with (ROOT / name).open('x') as stream:
        json.dump(value, stream, indent=2, sort_keys=True)
        stream.write('\n')


def git(*args):
    return subprocess.check_output(['/usr/bin/git', *args], cwd=REPO,
        env={**os.environ, 'GIT_OPTIONAL_LOCKS': '0', 'GIT_CONFIG_NOSYSTEM': '1', 'GIT_CONFIG_GLOBAL': '/dev/null'})


assert git('rev-parse', 'HEAD').decode().strip() == BASE
assert not git('status', '--porcelain').strip()
bindings = read('execution-bindings.json')
assert sha((ROOT / 'controller.py').read_bytes()) == bindings['controller_sha256']
assert sha((ROOT / 'observe/sitecustomize.py').read_bytes()) == bindings['observer_sha256']
assert sha((ROOT / 'package.sb').read_bytes()) == bindings['sandbox_sha256']
assert sha((ROOT / 'plan.json').read_bytes()) == bindings['plan_sha256']
assert sha((ROOT / 'preflight-complete.json').read_bytes()) == bindings['preflight_sha256']

# Bind every measured source input to the exact production commit, through Git blobs.
files = read('source-before.json')
tree = {}
for row in git('ls-tree', '-r', '-z', SOURCE).split(b'\0'):
    if row:
        header, name = row.split(b'\t', 1)
        tree[name.decode()] = header.split()[2].decode()
names = sorted(files)
process = subprocess.run(['/usr/bin/git', 'cat-file', '--batch'], cwd=REPO,
    input=('\n'.join(tree[n] for n in names) + '\n').encode(), capture_output=True, check=True,
    env={**os.environ, 'GIT_OPTIONAL_LOCKS': '0', 'GIT_CONFIG_NOSYSTEM': '1', 'GIT_CONFIG_GLOBAL': '/dev/null'})
data, offset, source_bindings = process.stdout, 0, []
for name in names:
    end = data.index(b'\n', offset)
    oid, kind, length = data[offset:end].split()
    assert kind == b'blob'
    offset = end + 1
    payload = data[offset:offset + int(length)]
    offset += int(length) + 1
    local = (REPO / name).read_bytes()
    assert files[name] == sha(local)
    conversion = 'identity'
    if sha(payload) != files[name]:
        assert name == 'tools/windows/openergy-run.cmd'
        attributes = git('check-attr', 'text', 'eol', 'filter', '--', name).decode()
        assert ': eol: crlf\n' in attributes and ': filter: unspecified\n' in attributes
        assert payload.replace(b'\n', b'\r\n') == local
        conversion = '.gitattributes *.cmd text eol=crlf; Git LF to checkout CRLF'
    source_bindings.append(dict(path=name, git_blob=oid.decode(), git_blob_sha256=sha(payload),
                               worktree_sha256=files[name], checkout_conversion=conversion))
assert offset == len(data)
changed = git('diff', '--name-only', SOURCE, BASE).decode().splitlines()
assert all(name.startswith('docs/verification/') for name in changed)
save('production-source-bindings.json', dict(source=SOURCE, build_head=BASE,
    inputs=source_bindings, source_to_head_changed_paths=changed,
    source_to_head_changes_all_verification_evidence=True))

post, package = read('postflight.json'), read('package-result.json')
samples = read('process-samples.json')
observations = [row for sample in samples for row in sample['owned']]
by_command = defaultdict(set)
counts = Counter()
for row in observations:
    name = Path(row['comm']).name.strip('()')
    by_command[name].add(row['pid'])
    counts[name] += 1
selected = {}
for label, commands in dict(rust_compilers=['rustc'], swift_frontends=['swift-frontend'],
    clang_compilers=['clang', 'cc'], linkers=['ld'], codesign=['codesign'], packaged_daemons=['openergy']).items():
    selected[label] = [row for row in observations if Path(row['comm']).name.strip('()') in commands]
save('priority-coverage.json', dict(samples=len(samples), observations=len(observations),
    observed_owned_pid_count=len(post['observed_pids']), observed_nice_values=post['observed_nice_values'],
    controller=bindings['controller_identity'], root=package['root_identity'],
    command_pid_counts={name: len(ids) for name, ids in by_command.items()},
    command_observation_counts=dict(counts), selected_observations=selected,
    first_seconds=post['sample_first_seconds'], last_seconds=post['sample_last_seconds'],
    max_gap_seconds=post['max_sample_gap_seconds'],
    limits='Actual Rust/clang/ld identities observed. No Swift frontend, codesign or packaged daemon was observed. The run stopped before whole-app compilation/signing/IPC. Short processes and unsampled intervals remain gaps.'))

stderr = (ROOT / 'package.stderr').read_text()
assert "couldn't create cache file" in stderr and 'sandbox_apply: Operation not permitted' in stderr
assert package['exit'] == 1 and not package['timeout']
assert post['smoke_completions'] == 0 and read('artifacts.json') == []
assert read('full-smoke.json')['smoke'] == []
assert all(post[k] for k in ('original_cache_bytes_links_unchanged', 'original_selected_metadata_unchanged',
    'source_bytes_unchanged', 'prior_gzip_private_originals_unchanged',
    'private_dependencies_unchanged_except_allowed_git_index'))
assert post['observed_nice_values'] == [15] and not post['observed_owned_remaining']
assert all(post['temporary_paths_absent'].values())
assert all(row['first_exclusive_bind'] for row in read('ports-postflight-first.json')['attempts'])
daemon = ROOT / 'cargo-target/aarch64-apple-darwin/release/openergy'
assert daemon.is_file()
save('failure-limits.json', dict(overall='FAILED', package_invocations=1,
    failure='Swift manifest evaluation failed inside the enclosing sandbox.',
    exact_diagnostics=["swift: error: couldn't create cache file '<original Darwin temp>/xcrun_db-xFzIFMiR' (errno=Operation not permitted)",
                       'sandbox-exec: sandbox_apply: Operation not permitted'],
    first_exact_diagnostics='package.stderr',
    compiled_daemon=dict(path=str(daemon), bytes=daemon.stat().st_size, sha256=sha(daemon.read_bytes()), executed=False),
    rust_build_seconds_as_reported=44.40,
    not_reached=['whole-app compilation', 'bundle assembly', 'nested signatures', 'ZIP roundtrip',
        'staged smoke version/absent-daemon/status/invalid-method/clean-stop',
        'extracted smoke version/absent-daemon/status/invalid-method/clean-stop',
        'extracted signature verification', 'archive/BUILD sidecar/source binding'],
    watchdog_seconds=900, cleanup_seconds=read('owned-cleanup.json')['elapsed_seconds'],
    first_run_status='Original overall FAILED remains immutable.',
    preflight_gap='Policy parser and private Unix IPC setup passed, but preflight did not evaluate a real Swift manifest under the enclosing policy.',
    future_work='A separately authorized attempt must first resolve the Darwin xcrun cache and nested SwiftPM sandbox constraints using private paths/explicit flags. No retry or proposed fix is validated here.',
    metadata_limit='New selected before/after equality only; no full old prebreach metadata, ACL, xattr, atime or ctime claim.',
    local_setup_failures=['read-only copied Git alternates; private chmod + redirected private repository',
        'Seatbelt parser requires localhost rather than numeric IP; policy corrected before package'],
    post_command_assertion='audit() retained its expected-success assertion after recording the failed package and all postflight records; first-error-run.txt is preserved.'))

results = dict(overall='FAILED: one package command exited 1 during Swift manifest evaluation; no retry',
    source=SOURCE, build_head=BASE, package=package, postflight=post,
    scope='Native arm64 whole-app/daemon package attempt; no accepted package artifact',
    original_run='Overall FAILED is unchanged; its 114-removal breach and metadata limits remain.',
    source_inputs=len(files), old_cache_entries={k: len(v) for k,v in read('old-cache-before.json').items()},
    selected_metadata_entries={k: len(v) for k,v in read('old-metadata-before.json').items()},
    prior_raw_records=len(read('prior-before.json')['ledger']),
    priority_observed_pid_count=len(post['observed_pids']), artifacts=[], smoke_flows_completed=0,
    private_root=str(ROOT), failure_limits='raw/failure-limits.json.gz')
save('results.json', results)

# Export exact records, except unrelated mount diagnostics in the two port checks.
# Their unmodified private originals remain retained and hash-bound in the ledger.
DEST.mkdir(mode=0o700)
(DEST / 'raw').mkdir()
inputs = [path for path in sorted(ROOT.iterdir()) if path.is_file()]
inputs += [ROOT / 'observe/sitecustomize.py']
ledger = []
for path in inputs:
    original = path.read_bytes()
    payload = original
    transform = 'exact original bytes'
    if path.name in ('ports-preflight-first.json', 'ports-postflight-first.json'):
        port = json.loads(original)
        port['lsof_before']['stderr'] = ('Unrelated mounted-filesystem stat warning omitted from export. '
            'Original retained privately and SHA-256 bound. lsof warned that output may be incomplete.')
        payload = (json.dumps(port, indent=2, sort_keys=True) + '\n').encode()
        transform = 'lsof unrelated private mount path redacted; original retained privately'
    if path.name == 'results.json':
        destination = DEST / 'results.json'
        encoded = payload
    elif path.name in ('controller.py', 'collect-final.py', 'package.sb') or path.parent.name == 'observe':
        relative = 'observe/sitecustomize.py' if path.parent.name == 'observe' else path.name
        destination = DEST / 'raw' / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        encoded = payload
    else:
        destination = DEST / 'raw' / (path.name + '.gz')
        encoded = gzip.compress(payload, mtime=0)
        transform += '; gzip with mtime 0'
    with destination.open('xb') as stream:
        stream.write(encoded)
    ledger.append(dict(path=str(destination.relative_to(DEST)), private_original=str(path),
        original_bytes=len(original), original_sha256=sha(original),
        payload_bytes=len(payload), payload_sha256=sha(payload),
        exported_bytes=len(encoded), exported_sha256=sha(encoded), transform=transform))
with (DEST / 'copy-ledger.json').open('x') as stream:
    json.dump(ledger, stream, indent=2, sort_keys=True)
    stream.write('\n')
print(json.dumps(dict(evidence=str(DEST), exported_records=len(ledger), package=package,
                     overall=results['overall']), indent=2))
