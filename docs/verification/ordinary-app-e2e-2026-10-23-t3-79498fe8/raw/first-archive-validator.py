"""Validate retained evidence bytes and ownership. Never starts product code."""
from common import *
import argparse
import ast

EXCLUDED = {'manifest.json', 'validation.json'}


def inventory():
    return [dict(file=str(path.relative_to(HERE)), bytes=path.stat().st_size, sha256=sha(path))
            for path in sorted(HERE.rglob('*')) if path.is_file()
            and str(path.relative_to(HERE)) not in EXCLUDED]


def validate():
    nice15()
    for path in HERE.glob('*.py'): ast.parse(path.read_text(), filename=str(path))
    for item in json.loads((RAW / 'frozen-controller.json').read_text())['files']:
        assert sha(HERE / item['path']) == item['sha256'], item['path']
    expected_manifest = json.loads((HERE / 'manifest.json').read_text())
    assert inventory() == expected_manifest
    outcome = json.loads((RAW / 'outcome.json').read_text())
    assert outcome['result'] == 'FAILED / NOT ACCEPTED'
    assert outcome['app_invocations'] == 1 and outcome['flow_retries'] == 0
    report = json.loads((RAW / 'recovery-report.json').read_text())
    assert len(report) == 25 and sum(i['passed'] for i in report) == 23
    assert [i['name'] for i in report if not i['passed']] == ['real peer connected', 'rejoin same address']
    assert (RAW / 'final-app.log').read_bytes() == (RAW / 'runtime/app.log').read_bytes()
    assert (RAW / 'final-peer.log').read_bytes() == (RAW / 'runtime/peer.log').read_bytes()
    for name in ['runtime/first-runtime-failure.txt', 'runtime/first-cleanup-failure.txt', 'first-controller-failure.txt']:
        assert 'AssertionError' in (RAW / name).read_text()
    app_bindings = json.loads((RAW / 'app-input-bindings.json').read_text())
    target_files = [i for i in app_bindings if i['path'].startswith('apps/macos/Sources/')]
    assert len(target_files) == 23 and all(i['matches_root'] for i in target_files)
    assert {i['path'] for i in target_files} == {str(p.relative_to(REPO)) for p in (REPO / 'apps/macos/Sources').rglob('*.swift')}
    assert [i['path'] for i in app_bindings if not i['matches_root']] == ['apps/macos/scripts/bundle.sh']
    for item in app_bindings: assert sha(REPO / item['path']) == item['actual_sha256']
    daemon = json.loads((RAW / 'daemon-397-input-bindings.json').read_text())
    assert len(daemon) == 397 and all(i['matches_root'] for i in daemon.values())
    for name, item in daemon.items(): assert sha(REPO / name) == item['sha256']
    framework = json.loads((RAW / 'framework-artifact-bindings.json').read_text())
    assert framework['match'] and framework['expected'] == framework['actual'] and len(framework['actual']) == 124
    for name in ['package-0.json', 'package-1.json', 'package-2.json']:
        assert json.loads((RAW / name).read_text())['code'] == 0
    info = json.loads((RAW / 'ordinary-info.json').read_text())
    assert info['CFBundleIdentifier'] == 'dev.openergy.app' and info['CFBundleExecutable'] == 'Openergy'
    assert 'OpenergyPrivateUpdateFixture' not in info
    for item in json.loads((RAW / 'inherited-build-records.json').read_text()):
        assert sha(RAW / item['copy']) == item['sha256']
    assert 'stripping debug info with `rust-objcopy` failed' in (RAW / 'inherited-daemon-build.stderr').read_text()
    assert json.loads((RAW / 'runtime/bundle-after.json').read_text())['unchanged']
    for port in PORTS:
        for path in [RAW / f'runtime/port-{port}-pre.json', RAW / f'recovery-port-{port}-post.json']:
            record = json.loads(path.read_text())
            assert record['success'] and record['no_reuse'] and record['no_retry']
    cleanup = json.loads((RAW / 'final-cleanup-inventory.json').read_text())
    assert not cleanup['remaining_owned_rows'] and not cleanup['remaining_known_group_rows']
    assert all(cleanup['pid_absence'].values()) and all(cleanup['socket_absence'].values())
    assert not json.loads((RAW / 'recovery-cleanup.json').read_text())['signals']
    assert outcome['app_elapsed_upper_bound_to_first_absence_observation_seconds'] < 60
    for path in (RAW / 'runtime').glob('*-exec.json'):
        receipt = json.loads(path.read_text())
        assert receipt['nice'] == 15 and receipt['pgid'] == receipt['pid']
        assert receipt['environment_allowlist']['HOME'].startswith('/private/tmp/oe-ordinary-79498-')
        assert receipt['environment_allowlist']['CFFIXED_USER_HOME'] == receipt['environment_allowlist']['HOME']
        assert receipt['environment_allowlist']['OPENERGY_ALLOW_PORT_24800'] == '0'
        assert receipt['environment_allowlist']['OPENERGY_DISABLE_DISCOVERY'] == '1'
    # Check all original tracked bytes remain unchanged, including old evidence.
    outside = cmd(['git', 'diff', ROOT, '--', '.', ':(exclude)' + str(HERE.relative_to(REPO))])
    assert outside['code'] == 0 and outside['stdout'] == ''
    assert not any(p.name == '__pycache__' for p in HERE.rglob('*'))
    return dict(archive_integrity='PASS', product_result='FAILED / NOT ACCEPTED',
                source_root=ROOT, frozen_controller_unchanged=True, old_tracked_records_unchanged=True,
                manifest_sha256=sha(HERE / 'manifest.json'), manifest_records=len(expected_manifest),
                source_bindings=dict(target_swift=23, daemon=397, framework_artifact_records=124),
                flow_count=1, retries=0, checks=25, passed=23, failed=2,
                no_product_execution_by_validator=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--record', action='store_true')
    parser.add_argument('--index', action='store_true')
    args = parser.parse_args()
    if args.record: save(HERE / 'manifest.json', inventory())
    report = validate()
    if args.record: save(HERE / 'validation.json', report)
    if args.index:
        expected = {str(p.relative_to(REPO)) for p in HERE.rglob('*') if p.is_file()}
        staged = set(cmd(['git', 'diff', '--cached', '--name-only'])['stdout'].splitlines())
        assert staged == expected, (staged - expected, expected - staged)
        for name in expected:
            contents = subprocess.check_output(['git', 'show', ':' + name], cwd=REPO)
            assert contents == (REPO / name).read_bytes(), name
        assert cmd(['git', 'diff', '--name-only'])['stdout'] == ''
        assert cmd(['git', 'ls-files', '--others', '--exclude-standard'])['stdout'] == ''
        report['index_ownership'] = 'all and only exact new evidence files staged'
    print(json.dumps(report, sort_keys=True))
