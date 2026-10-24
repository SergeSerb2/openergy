#!/usr/bin/env python3
"""Archive and source-binding checks only. Never starts a product or opens a socket."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
RAW = HERE / 'raw'
REL = str(HERE.relative_to(REPO))
ROOT = 'd2ab17d4e053437d9f7568b0f6dbf96727bfd6c9'


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read(path):
    return json.loads(path.read_text())


def git(*args):
    return subprocess.check_output(['git', *args], cwd=REPO)


def check_manifest():
    for name, entry in read(HERE / 'evidence-manifest.json').items():
        path = HERE / name
        assert path.stat().st_size == entry['bytes'] and digest(path) == entry['sha256'], name


def check_index():
    check_manifest()
    expected = {str(path.relative_to(REPO)) for path in HERE.rglob('*') if path.is_file()}
    expected.add('crates/daemon/src/runtime.rs')
    staged = set(git('diff', '--cached', '--name-only', '-z').decode().strip('\0').split('\0'))
    assert staged == expected, (sorted(expected-staged), sorted(staged-expected))
    for name in expected:
        assert git('show', ':' + name) == (REPO / name).read_bytes(), name
    # Raw compiler stderr and exact unified patches retain original whitespace.
    maintained = ['crates/daemon/src/runtime.rs', str(HERE / 'README.md'),
                  *map(str, HERE.glob('*.py'))]
    subprocess.run(['git', 'diff', '--cached', '--check', '--', *maintained], cwd=REPO, check=True)
    assert git('diff', '--name-only') == b'', 'unstaged tracked edits'
    assert git('ls-files', '--others', '--exclude-standard') == b'', 'untracked files'
    print(f'PASS: {len(expected)} exact staged files; manifest and ownership checked; source/prose whitespace passed')


def validate():
    os.setpriority(os.PRIO_PROCESS, 0, 15)
    report = dict(root=ROOT, controller_pid=os.getpid(), controller_nice=os.getpriority(os.PRIO_PROCESS, 0),
                  monotonic_ns=time.clock_gettime_ns(time.CLOCK_MONOTONIC), flows={}, builds={})
    assert git('rev-parse', 'HEAD').decode().strip() == ROOT
    changes = git('diff', '--name-only').decode().splitlines()
    assert changes == ['crates/daemon/src/runtime.rs'], changes
    unknown = git('ls-files', '--others', '--exclude-standard').decode().splitlines()
    assert all(path.startswith(REL + '/') for path in unknown), unknown
    assert git('diff', '--', 'crates/daemon/src/runtime.rs') == (HERE / 'runtime-fix.patch').read_bytes()
    original = git('show', ROOT + ':crates/daemon/src/runtime.rs')
    current = (REPO / 'crates/daemon/src/runtime.rs').read_bytes()
    tail = b'    log::info!("shutdown: leave and goodbye");'
    assert original[original.index(tail):] == current[current.index(tail):]
    assert git('show', ROOT + ':crates/daemon/src/main.rs') == (REPO / 'crates/daemon/src/main.rs').read_bytes()
    report['shutdown_tail_byte_identical'] = True
    report['cli_overlay_removed_exactly'] = True
    report['final_runtime_sha256'] = digest(REPO / 'crates/daemon/src/runtime.rs')
    source_compares = read(RAW / 'source-comparisons.json')
    assert source_compares['through_1fd5144']['code'] == 0
    assert source_compares['through_1fd5144']['stdout'] == ''
    diagnosis_readme = 'docs/verification/daemon-stop-latency-source-2026-10-23-t3-d1ff2556/README.md'
    assert hashlib.sha256(git('show', 'a64b33dd53a9ba11f2a887086b2615ef949c80fc:' + diagnosis_readme)).hexdigest() == source_compares['diagnosis_readme_sha256']
    report['diagnosis_readme_matches_commit'] = True
    base_bindings = read(RAW / 'root-sources.json')
    assert len(base_bindings) == 397 and all(item['matches_root'] for item in base_bindings.values())
    for label, expected in [('baseline', ['crates/daemon/src/main.rs']),
                            ('candidate', ['crates/daemon/src/main.rs', 'crates/daemon/src/runtime.rs']),
                            ('final-source', ['crates/daemon/src/runtime.rs'])]:
        path = RAW / f'{label}-build'
        bindings = read(path / 'source-bindings.json')
        assert sorted(name for name, item in bindings.items() if not item['matches_root']) == expected
        result = read(path / 'result.json')
        assert result['code'] == 0
        assert result['env']['CARGO_BUILD_JOBS'] == '2' and result['env']['CARGO_NET_OFFLINE'] == 'true'
        assert result['env']['RUSTUP_TOOLCHAIN'] == '1.99.1'
        assert result['env']['MACOSX_DEPLOYMENT_TARGET'] == '15.0'
        binary = read(path / 'binary.json')
        assert digest(Path(binary['path'])) == binary['sha256']
        assert binary['all_actual_rustc_ni15'] and binary['actual_rustc_samples'] > 0
        for item in (path / 'compiler-exec').glob('*.json'):
            receipt = read(item)
            assert receipt['nice'] == 15 and receipt['argv'][0] == result['env']['RUSTC']
        barrier = read(path / 'cargo-exec.json')
        assert barrier['nice'] == 15
        try:
            os.killpg(barrier['pgid'], 0)
        except ProcessLookupError:
            group_absent = True
        else:
            group_absent = False
        assert group_absent, label
        stderr = (path / 'stderr.txt').read_text()
        assert 'stripping debug info with `rust-objcopy` failed' in stderr
        report['builds'][label] = dict(code=0, binary_sha256=binary['sha256'], group_absent=True,
            source_differences_from_root=expected, actual_rustc_samples=binary['actual_rustc_samples'],
            actual_rustc_ni15=True, stripping_warning_preserved=True)
    bsrc = read(RAW / 'baseline-build/source-bindings.json')
    csrc = read(RAW / 'candidate-build/source-bindings.json')
    assert [name for name in bsrc if bsrc[name] != csrc[name]] == ['crates/daemon/src/runtime.rs']
    finalsrc = read(RAW / 'final-source-build/source-bindings.json')
    for name, record in finalsrc.items():
        assert digest(REPO / name) == record['sha256'], name
    b = read(RAW / 'baseline-flow/prelaunch.json')
    c = read(RAW / 'candidate-flow/prelaunch.json')
    assert b['workflow_sha256'] == c['workflow_sha256']
    for name, expected_sha in b['workflow_sha256'].items():
        assert digest(HERE / name) == expected_sha, name
    assert b['trace_overlay_sha256'] == c['trace_overlay_sha256'] == digest(HERE / 'cli-trace-overlay.patch')
    for label in ('baseline', 'candidate'):
        path = RAW / f'{label}-flow'
        timing = read(path / 'stop-timing.json')
        cleanup = read(path / 'cleanup.json')
        stop = read(path / 'stop.json')
        assert timing['normal_exit_observed'] and timing['independent_exit']['code'] == 0
        assert timing['independent_exit']['observed_exit_ns'] <= timing['normal_exit_deadline_ns']
        assert stop['code'] == 0 and stop['stdout'] == '{"stopping":true}\n'
        assert timing['reply']['value'] == {'id': timing['request']['value']['id'], 'result': {'stopping': True}}
        assert cleanup['signals'] == [] and cleanup['group_absent'] and cleanup['ipc_socket_absent']
        assert cleanup['pid_absent']['code'] == 1 and cleanup['pid_absent']['stdout'] == ''
        assert cleanup['tls_directory_absent'] and not (path / 'first-error.txt').exists()
        assert read(path / 'daemon-identity.json')['nice'] == 15
        assert read(path / 'invalid.json')['code'] == 1
        assert json.loads(read(path / 'invalid.json')['stdout'])['error']['code'] == 'unknown_method'
        for phase in ('pre', 'post'):
            assert read(path / f'port-{phase}.json')['first_exclusive_bind'] == 'passed'
        fixture = REPO / 'target/sb' / ('b' if label == 'baseline' else 'c')
        assert not (fixture / 'i.sock').exists() and not (fixture / 'tls').exists()
        report['flows'][label] = dict(normal_exit=True, code=0, stop_requests=1, signals=[],
            request_to_reply_ms=timing['request_to_reply_ms'], request_to_exit_ms=timing['request_to_exit_ms'],
            pid=read(path / 'daemon-identity.json')['pid'], cleanup=True, exclusive_binds=2)
    for item in read(HERE / 'port-warning-redactions.json'):
        assert digest(Path(item['original_private_path'])) == item['original_sha256']
        assert digest(HERE / item['file']) == item['archived_sha256']
    # Verify the source registry remains byte-identical to the copied inventory.
    for name, expected_sha in read(RAW / 'copied-registry.json').items():
        assert digest(Path('/Users/serge/.cargo/registry') / name) == expected_sha, name
        assert digest(REPO / 'target/sb/cargo-home/registry' / name) == expected_sha, name
    report['registry_source_and_copy_bytes_unchanged'] = True
    fmt = subprocess.run(['/Users/serge/.rustup/toolchains/1.99.1-aarch64-apple-darwin/bin/rustfmt',
        '--check', '--edition', '2021', 'crates/daemon/src/runtime.rs'], cwd=REPO, capture_output=True, text=True)
    report['rustfmt'] = dict(code=fmt.returncode, stdout=fmt.stdout, stderr=fmt.stderr)
    assert fmt.returncode == 0, report['rustfmt']
    subprocess.run(['git', 'diff', '--check'], cwd=REPO, check=True)
    (HERE / 'validation.json').write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')
    manifest = {str(path.relative_to(HERE)): dict(bytes=path.stat().st_size, sha256=digest(path))
                for path in sorted(HERE.rglob('*')) if path.is_file() and path.name != 'evidence-manifest.json'}
    (HERE / 'evidence-manifest.json').write_text(json.dumps(manifest, indent=2, sort_keys=True) + '\n')
    check_manifest()
    print(f'PASS: two finite real flows; three builds; {len(manifest)} archived files; final source and cleanup checked')


if __name__ == '__main__':
    if '--check-index' in sys.argv:
        check_index()
    else:
        validate()
