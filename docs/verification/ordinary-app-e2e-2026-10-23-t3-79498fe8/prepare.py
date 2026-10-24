"""Read exact bindings; assemble one new ordinary bundle. No product execution."""
from common import *
import platform
import plistlib
import shutil
import tempfile
import traceback


def main():
    nice15()
    RAW.mkdir(mode=0o700)
    assert cmd(['git', 'rev-parse', 'HEAD'])['stdout'].strip() == ROOT
    assert platform.machine() == 'arm64'
    private = Path(tempfile.mkdtemp(prefix='oe-ordinary-79498-', dir='/private/tmp'))
    private.chmod(0o700)
    for name in ['home', 'tmp', 'peer', 'app', 'cache']:
        (private / name).mkdir(mode=0o700)
    bundle = private / 'Openergy.app'
    env = dict(PATH='/usr/bin:/bin', LANG='C', HOME=str(private / 'home'),
               CFFIXED_USER_HOME=str(private / 'home'), TMPDIR=str(private / 'tmp') + '/',
               XDG_CONFIG_HOME=str(private / 'home'), XDG_CACHE_HOME=str(private / 'cache'),
               MACOSX_DEPLOYMENT_TARGET='15.0', PYTHONDONTWRITEBYTECODE='1',
               OPENERGY_SIGN_IDENTITY='-')
    save(RAW / 'predeclared.json', dict(root=ROOT, private_root=str(private), bundle=str(bundle),
         setup_environment_allowlist=env, controller_pid=os.getpid(), controller_nice=15,
         flows=1, routine='unchanged default AppEndToEnd with --require-peer, no optional mode',
         ports=list(PORTS), flow_watchdog_seconds=120, owned_cleanup_seconds=60,
         peer_readiness_seconds=15, cli_seconds=5, sample_interval_seconds=0.02,
         max_process_samples=24000, max_sample_rounds=6000, max_log_bytes=8388608,
         max_evidence_bytes=33554432, ipc_max_frame_bytes=16777216,
         routine_stalled_padding_bytes=1000000, routine_stalled_request_ms=300,
         routine_responsiveness_seconds=1, routine_stalled_bound_seconds=2,
         routine_step_wait_seconds=[3, 5, 10, 15], routine_supervisor_stop_seconds=1.5,
         exit_contract='No new one-second process-exit bound', no_retry=True,
         scope='ordinary packaged model/IPC only; no rendering, fixture launcher, updater, native input or trusted release'))
    app_records = json.loads((OLD_APP / 'build-source.json').read_text())
    actual_swift = {str(p.relative_to(REPO)) for p in (REPO / 'apps/macos/Sources').rglob('*.swift')}
    assert actual_swift == {i['path'] for i in app_records if i['path'].startswith('apps/macos/Sources/')}
    app_bindings = []
    for item in app_records:
        path = REPO / item['path']
        app_bindings.append(dict(**item, actual_sha256=sha(path), actual_bytes=path.stat().st_size,
                                 matches_root=sha(path) == item['sha256']))
    save(RAW / 'app-input-bindings.json', app_bindings)
    assert [i['path'] for i in app_bindings if not i['matches_root']] == ['apps/macos/scripts/bundle.sh']
    for name in ['apps/macos/Package.swift', 'apps/macos/Package.resolved', 'apps/macos/Support/Info.plist', *sorted(actual_swift)]:
        assert next(i for i in app_bindings if i['path'] == name)['matches_root']
    scope = ['crates/daemon', 'crates/net', 'crates/session', 'crates/ipc', 'crates/config',
             'crates/platform', 'crates/platform-macos', 'crates/protocol', 'vendor/mdns-sd',
             'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml']
    source = json.loads((OLD_DAEMON / 'source-bindings.json').read_text())
    names = cmd(['git', 'ls-files', '--', *scope])['stdout'].splitlines()
    assert len(names) == len(source) == 397 and set(names) == set(source)
    bindings = {}
    for name, record in source.items():
        path = REPO / name
        bindings[name] = dict(expected_sha256=record['sha256'], sha256=sha(path),
                              bytes=path.stat().st_size, matches_root=sha(path) == record['sha256'])
    save(RAW / 'daemon-397-input-bindings.json', bindings)
    assert all(i['matches_root'] for i in bindings.values())
    # Entire pinned binary artifact input tree, including all helpers and licenses.
    cache = json.loads((OLD_APP / 'cache-copy-after.json').read_text())
    artifact_records = [i for i in cache if i['path'].startswith('artifacts/')]
    current = []
    for path in sorted((SWIFT / 'artifacts').rglob('*')):
        item = dict(path=str(path.relative_to(SWIFT)))
        if path.is_symlink(): item['symlink'] = os.readlink(path)
        elif path.is_file(): item.update(bytes=path.stat().st_size, sha256=sha(path))
        else: continue
        current.append(item)
    save(RAW / 'framework-artifact-bindings.json', dict(expected=artifact_records, actual=current,
         match=artifact_records == current))
    assert artifact_records == current and artifact_records
    state = json.loads((SWIFT / 'workspace-state.json').read_text())
    resolved = json.loads((REPO / 'apps/macos/Package.resolved').read_text())
    pin = resolved['pins'][0]['state']
    assert state['object']['dependencies'][0]['state']['checkoutState'] == pin
    assert pin == dict(revision='eef1a539a373c1f1a320624b1130fc5de7b2e100', version='2.10.0')
    assert state['object']['artifacts'][0]['source']['checksum'] == '17e28312b8e18ab7cdbbe09a6fb28cc55a5479ec6c371dbc07cdecd2a14fd959'
    save(RAW / 'framework-resolution.json', dict(workspace_state=state, package_resolved=resolved,
         workspace_state_sha256=sha(SWIFT / 'workspace-state.json')))
    app_source = SWIFT / 'arm64-apple-macosx/debug/Openergy'
    daemon_source = Path(json.loads((OLD_DAEMON / 'binary.json').read_text())['path'])
    assert sha(app_source) == APP_SHA and sha(daemon_source) == DAEMON_SHA
    for label, path in [('app', app_source), ('daemon', daemon_source)]:
        result = cmd(['/usr/bin/lipo', '-archs', path])
        save(RAW / (label + '-source-architecture.json'), result)
        assert result['code'] == 0 and result['stdout'].strip() == 'arm64'
    inherited = [
        (OLD_APP / 'app-result.json', 'inherited-app-build-result.json'),
        (OLD_APP / 'app-command.json', 'inherited-app-build-command.json'),
        (OLD_APP / 'app.stderr', 'inherited-app-build.stderr'),
        (OLD_DAEMON / 'result.json', 'inherited-daemon-build-result.json'),
        (OLD_DAEMON / 'stderr.txt', 'inherited-daemon-build.stderr'),
        (OLD_DAEMON / 'binary.json', 'inherited-daemon-binary.json')]
    references = []
    for source_path, name in inherited:
        shutil.copyfile(source_path, RAW / name)
        references.append(dict(source=str(source_path), copy=name, sha256=sha(source_path), transform='none'))
    save(RAW / 'inherited-build-records.json', references)
    for name in ['MacOS', 'Helpers', 'Resources']:
        (bundle / 'Contents' / name).mkdir(mode=0o700, parents=True, exist_ok=True)
    shutil.copyfile(app_source, bundle / 'Contents/MacOS/Openergy')
    shutil.copyfile(daemon_source, bundle / 'Contents/Helpers/openergy')
    for path in [bundle / 'Contents/MacOS/Openergy', bundle / 'Contents/Helpers/openergy']:
        path.chmod(0o700)
    for name in ['LICENSE-MIT', 'LICENSE-APACHE', 'THIRD_PARTY_NOTICES.md']:
        shutil.copyfile(REPO / name, bundle / 'Contents/Resources' / name)
    icon = Path('/var/folders/s6/_07cr4w13vsg_myjkv9ldpqc0000gn/T/oe-sparkle-combined-20261023-a_es9dei/bundle-final/Openergy.app/Contents/Resources/AppIcon.icns')
    shutil.copyfile(icon, bundle / 'Contents/Resources/AppIcon.icns')
    save(RAW / 'icon-copy.json', dict(source=str(icon), sha256=sha(icon), compiled_here=False))
    (bundle / 'Contents/PkgInfo').write_bytes(b'APPL????')
    tool = REPO / 'tools/release/sparkle.py'
    commands = [
        ['configure-plist', '--template', REPO / 'apps/macos/Support/Info.plist', '--output', bundle / 'Contents/Info.plist', '--version', '0.1.0', '--build', '1'],
        ['embed', '--app', bundle, '--scratch', SWIFT],
        ['sign', '--app', bundle, '--arch', 'arm64']]
    save(RAW / 'packaging-tool-binding.json', dict(path=str(tool), sha256=sha(tool),
         priority_helper_sha256=sha(REPO / 'tools/release/priority.py'), no_build=True,
         input_app_sha256=sha(app_source), input_daemon_sha256=sha(daemon_source)))
    for i, args in enumerate(commands):
        result = cmd(['/opt/homebrew/bin/python3', tool, *args], env, timeout=90)
        save(RAW / f'package-{i}.json', result)
        assert result['code'] == 0, result
    info = plistlib.loads((bundle / 'Contents/Info.plist').read_bytes())
    assert info['CFBundleIdentifier'] == 'dev.openergy.app' and info['CFBundleExecutable'] == 'Openergy'
    assert 'OpenergyPrivateUpdateFixture' not in info
    assert not (bundle / 'Contents/MacOS/OpenergyPrivateLauncher').exists()
    save(RAW / 'ordinary-info.json', info)
    save(RAW / 'bundle-signed-tree.json', tree(bundle))
    shutil.copyfile(bundle / 'Contents/Info.plist', RAW / 'ordinary-Info.plist')
    save(RAW / 'prepared.json', dict(private_root=str(private), bundle=str(bundle), environment=env,
         input_app_sha256=APP_SHA, input_daemon_sha256=DAEMON_SHA,
         app_sha256=sha(bundle / 'Contents/MacOS/Openergy'),
         daemon_sha256=sha(bundle / 'Contents/Helpers/openergy'), arch='arm64', swift_files=23,
         daemon_input_count=397, framework_artifact_records=len(current), product_invocations=0))


if __name__ == '__main__':
    try:
        main()
    except BaseException:
        (RAW / 'first-setup-failure.txt').write_text(traceback.format_exc())
        raise
