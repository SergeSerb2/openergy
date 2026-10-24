"""LATER file-only preparation. Do not execute during the source-only repair task.

Copies the already validated ordinary signed bundle; never builds, signs, launches,
queries a process/port, uses a credential, or reuses a runtime root.
"""
from common import *
import plistlib
import shutil
import tempfile


def bind_sources(deadline):
    app = json.loads(git_file('raw/app-input-bindings.json', deadline))
    declared = {r['path'] for r in app if r['path'].startswith('apps/macos/Sources/')}
    actual = {str(p.relative_to(REPO)) for p in (REPO / 'apps/macos/Sources').rglob('*.swift')}
    if declared != actual or len(actual) != 22:
        raise RuntimeError('target Swift file set changed')
    inputs = []
    for record in app:
        # bundle.sh differed before the original ordinary packaging. It was
        # neither run nor a Swift compile input. Bind current bytes separately.
        path = record['path']
        digest = sha(REPO / path, deadline)
        if path != 'apps/macos/scripts/bundle.sh' and digest != record['actual_sha256']:
            raise RuntimeError('app source/input changed: ' + path)
        inputs.append(dict(path=path, sha256=digest, bytes=(REPO / path).stat().st_size))
    daemon = json.loads(git_file('raw/daemon-397-input-bindings.json', deadline))
    scopes = ['crates/daemon', 'crates/net', 'crates/session', 'crates/ipc', 'crates/config',
              'crates/platform', 'crates/platform-macos', 'crates/protocol', 'vendor/mdns-sd',
              'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml']
    files = command(['git', 'ls-files', '--cached', '--others', '--exclude-standard', '--', *scopes], deadline).decode().splitlines()
    if len(daemon) != 397 or set(files) != set(daemon):
        raise RuntimeError('daemon declared input set changed')
    for path, record in daemon.items():
        if sha(REPO / path, deadline) != record['sha256'] or (REPO / path).stat().st_size != record['bytes']:
            raise RuntimeError('daemon source/input changed: ' + path)
    framework = json.loads(git_file('raw/framework-artifact-bindings.json', deadline))
    if not framework['match'] or framework['expected'] != framework['actual'] or len(framework['actual']) != 124:
        raise RuntimeError('inherited framework binding failed')
    return dict(app=inputs, daemon=daemon, framework=framework,
                inherited_root=SOURCE, compiled_app='7287e9aa4caf6570fc4116a89908d548a258f302',
                compiled_daemon='2485c1425e4d2f414eb22742dfd73f355d47390c')


def main():
    nice15()
    deadline = time.monotonic() + 120  # File I/O budget only; no flow started.
    with io_until(deadline):
        initialize_bytes(deadline)
        output = HERE / 'prepared.json'
        if output.exists():
            raise RuntimeError('one file-only fixture already prepared; do not retry')
        # A separate exclusive guard survives partial preparation failures.
        save(HERE / 'prepare-guard.json', dict(monotonic=time.monotonic(), runtime_invocations=0), deadline)
        bindings = bind_sources(deadline)
        old = json.loads(git_file('raw/prepared.json', deadline))
        expected_tree = json.loads(git_file('raw/bundle-signed-tree.json', deadline))
        source_bundle = Path(old['bundle']).resolve()
        if tree(source_bundle, deadline) != expected_tree:
            raise RuntimeError('validated source bundle tree changed/missing')
        private = Path(tempfile.mkdtemp(prefix='oe-ordinary-r5-e9c64d62-', dir='/private/tmp')).resolve()
        private.chmod(0o700)
        for name in ('home', 'tmp', 'app', 'peer', 'cache'):
            (private / name).mkdir(mode=0o700)
        if private.stat().st_dev != HERE.stat().st_dev:
            raise RuntimeError('reserved archival requires one filesystem; no copy fallback')
        bundle = private / 'Openergy.app'
        shutil.copytree(source_bundle, bundle, symlinks=True, copy_function=shutil.copy2)
        if tree(bundle, deadline) != expected_tree or tree(source_bundle, deadline) != expected_tree:
            raise RuntimeError('bundle copy/source tree differs')
        info = plistlib.loads(read(bundle / 'Contents/Info.plist', deadline))
        if (info['CFBundleIdentifier'] != 'dev.openergy.app' or info['CFBundleExecutable'] != 'Openergy'
                or 'OpenergyPrivateUpdateFixture' in info or
                (bundle / 'Contents/MacOS/OpenergyPrivateLauncher').exists()):
            raise RuntimeError('not ordinary Info.plist/bundle')
        app_hash = sha(bundle / 'Contents/MacOS/Openergy', deadline)
        daemon_hash = sha(bundle / 'Contents/Helpers/openergy', deadline)
        if app_hash != old['app_sha256'] or daemon_hash != old['daemon_sha256']:
            raise RuntimeError('copied executable differs')
        environment = dict(PATH='/usr/bin:/bin', LANG='C', HOME=str(private / 'home'),
                           CFFIXED_USER_HOME=str(private / 'home'), TMPDIR=str(private / 'tmp') + '/',
                           XDG_CONFIG_HOME=str(private / 'home'), XDG_CACHE_HOME=str(private / 'cache'),
                           MACOSX_DEPLOYMENT_TARGET='15.0', PYTHONDONTWRITEBYTECODE='1')
        inherited_limits = json.loads(git_file('raw/predeclared.json', deadline))
        limit_names = ('flows', 'routine', 'flow_watchdog_seconds', 'owned_cleanup_seconds',
                       'peer_readiness_seconds', 'cli_seconds', 'sample_interval_seconds',
                       'max_process_samples', 'max_sample_rounds', 'max_log_bytes', 'max_evidence_bytes',
                       'ipc_max_frame_bytes', 'routine_stalled_padding_bytes', 'routine_stalled_request_ms',
                       'routine_responsiveness_seconds', 'routine_stalled_bound_seconds',
                       'routine_step_wait_seconds', 'routine_supervisor_stop_seconds', 'exit_contract', 'no_retry', 'scope')
        value = dict(private_root=str(private), bundle=str(bundle), environment=environment,
                     bundle_tree=expected_tree, app_sha256=app_hash, daemon_sha256=daemon_hash,
                     source_bindings=bindings, original_commit=ORIGINAL,
                     source_bundle=str(source_bundle), product_invocations=0, ports=list(PORTS),
                     limits={name: inherited_limits[name] for name in limit_names})
        save(output, value, deadline)


if __name__ == '__main__':
    main()
