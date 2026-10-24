#!/usr/bin/env python3
"""ONE file-only preparation. No target, daemon, launcher, updater or installer execution.

Allowed executions: pinned compiler/build tools, the reviewed Python entry packager,
metadata/archive/codesign tools, a file-only fresh-key generator, official sign_update.
No network, service/LS registration, assessment, help/version product calls or tests.
All output creation is exclusive. A failure is retained and is never auto-retried.
"""
import base64
import gzip
import hashlib
import json
import os
from pathlib import Path
import plistlib
import shutil
import stat
import subprocess
import sys
import time
import traceback
import uuid
import xml.etree.ElementTree as ET
import zipfile

REPO = Path(__file__).resolve().parents[3]
OUT = Path(__file__).resolve().parent
RAW = OUT / 'raw'
BASE = 'ad280644a96905d9831825bb4f87d1388dbdca73'
BUILD = Path('/private/tmp/oe-bootstrap-r2-build-yl3byo9v')
REPAIR = REPO / 'docs/verification/private-update-bootstrap-repair-2026-10-23-t3-54de37ad/raw'
STOPPED = Path('/private/tmp/oe-stopped-update-llil939f/driver/sparkle.app')
REGISTRY = Path('/private/tmp/oe-priority-package-sr3z_6fy/cargo-home/registry')
TOOLS = Path('/Users/serge/.rustup/toolchains/1.99.1-aarch64-apple-darwin/bin')
PYTHON = '/Users/serge/.local/bin/python3.11'
PIN = 'eef1a539a373c1f1a320624b1130fc5de7b2e100'
NS = 'http://www.andymatuschak.org/xml-namespaces/sparkle'


def sha(p):
    with p.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def save(p, value):
    with p.open('x') as f:
        json.dump(value, f, indent=2, sort_keys=True)
        f.write('\n')


def inventory(root):
    result = {}
    for p in sorted(root.rglob('*')):
        s = p.lstat()
        n = p.relative_to(root).as_posix()
        if p.is_symlink():
            result[n] = {'symlink': os.readlink(p)}
        elif stat.S_ISREG(s.st_mode):
            result[n] = {'bytes': s.st_size, 'sha256': sha(p), 'mode': oct(stat.S_IMODE(s.st_mode))}
        elif stat.S_ISDIR(s.st_mode):
            result[n] = {'directory': True, 'mode': oct(stat.S_IMODE(s.st_mode))}
        else:
            raise RuntimeError('special file in input: ' + str(p))
    return result


def public_files(root):
    return {k: {x: y for x, y in v.items() if x != 'mode'}
            for k, v in inventory(root).items() if not v.get('directory')}


def priority():
    assert os.getpriority(os.PRIO_PROCESS, 0) <= 15
    os.setpriority(os.PRIO_PROCESS, 0, 15)
    assert os.getpriority(os.PRIO_PROCESS, 0) == 15


def main():
    os.umask(0o077)
    priority()
    RAW.mkdir(mode=0o700)
    assert subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=REPO, text=True).strip() == BASE
    identifier_uuid, nonce = str(uuid.uuid4()), str(uuid.uuid4())
    root = Path('/private/tmp/oe-updater-fixture-' + identifier_uuid)
    root.mkdir(mode=0o700)
    fixture = 'dev.openergy.updater-fixture.' + identifier_uuid
    # Read-only port observation only. No socket creation, connect, feed request or bind.
    port = 56327
    rows = subprocess.run(['/usr/sbin/lsof', '-nP', '-iTCP:' + str(port)], capture_output=True)
    assert rows.returncode == 1 and not rows.stdout
    save(RAW / 'port-observation.json', {'port': port, 'argv': ['/usr/sbin/lsof', '-nP', '-iTCP:' + str(port)],
         'exit': rows.returncode, 'stdout': rows.stdout.decode(), 'stderr': rows.stderr.decode(),
         'limit': 'read-only absence observation; no reservation or future ownership proof'})
    for n in ['target', 'candidate', 'old', 'state', 'state/tls', 'home', 'tmp', 'receipts', 'public',
              'driver', 'build', 'build/home', 'build/tmp', 'build/modules', 'build/cargo-home', 'build/target',
              'build/xdg', 'driver-home', 'driver-tmp', 'runtime-evidence']:
        (root / n).mkdir(mode=0o700)
    created = time.time()
    marker = dict(schema=1, fixtureID=fixture, root=str(root), nonce=nonce, port=port,
                  builds=['1', '2'], createdAt=created, expiresAt=created + 21600)
    save(root / 'fixture.json', marker)
    (root / 'launch.lock').open('xb').close()
    config = (f'version = 1\nrole = "off"\nname = "updater-fixture-{nonce}"\n'
              f'[server]\nlisten = "127.0.0.1:{port}"\nport = {port}\n'
              '[client]\nremote = ""\nautoconnect = false\n[tls]\nenabled = false\n'
              '[clipboard]\nenabled = false\n')
    (root / 'state/config.toml').write_text(config)
    save(OUT / 'fixture.json', {'root': str(root), 'identifier': fixture, 'driver_id': fixture + '.driver',
         'marker': marker, 'config': config, 'port': port, 'approval_received': False,
         'status': 'DO-NOT-RUN', 'scope': 'file-only preparation; runtime remains unverified'})
    env = {'PATH': str(TOOLS) + ':/Applications/Xcode.app/Contents/Developer/usr/bin:/usr/bin:/bin:/usr/sbin:/sbin',
           'LANG': 'C', 'HOME': str(root / 'build/home'), 'CFFIXED_USER_HOME': str(root / 'build/home'),
           'TMPDIR': str(root / 'build/tmp') + '/', 'XDG_CACHE_HOME': str(root / 'build/xdg'),
           'XDG_CONFIG_HOME': str(root / 'build/xdg'), 'CARGO_HOME': str(root / 'build/cargo-home'),
           'CARGO_TARGET_DIR': str(root / 'build/target'), 'CARGO_BUILD_JOBS': '2', 'CARGO_NET_OFFLINE': 'true',
           'RUSTC': str(TOOLS / 'rustc'), 'MACOSX_DEPLOYMENT_TARGET': '15.0',
           'CLANG_MODULE_CACHE_PATH': str(root / 'build/modules'),
           'SWIFT_MODULECACHE_PATH': str(root / 'build/modules'), 'GIT_CONFIG_NOSYSTEM': '1',
           'GIT_CONFIG_GLOBAL': '/dev/null', 'GIT_OPTIONAL_LOCKS': '0', 'PYTHONDONTWRITEBYTECODE': '1'}
    records = []

    def run(label, args, timeout=120, compiler=False):
        args = list(map(str, args))
        save(RAW / (label + '-command.json'), {'argv': args, 'env': env, 'timeout_seconds': timeout,
             'absolute_nice': 15, 'scope': 'file-only', 'controller_sha256': sha(Path(__file__))})
        samples = []
        start = time.monotonic()
        with (RAW / (label + '.stdout')).open('xb') as out, (RAW / (label + '.stderr')).open('xb') as err:
            p = subprocess.Popen(['/usr/bin/sandbox-exec', '-p', profile, *args], cwd=REPO, env=env,
                                 stdin=subprocess.DEVNULL, stdout=out, stderr=err)
            while p.poll() is None:
                if compiler:
                    table = {}
                    for row in subprocess.check_output(['/bin/ps', '-axo', 'pid=,ppid=,ni=,comm='], text=True).splitlines():
                        a = row.split(None, 3)
                        if len(a) == 4:
                            table[int(a[0])] = dict(pid=int(a[0]), ppid=int(a[1]), nice=int(a[2]), path=a[3])
                    owned = {p.pid}
                    while True:
                        expanded = owned | {pid for pid, row in table.items() if row['ppid'] in owned}
                        if expanded == owned:
                            break
                        owned = expanded
                    samples.append({'elapsed': time.monotonic() - start,
                                    'owned': [table[pid] for pid in sorted(owned) if pid in table]})
                if time.monotonic() - start > timeout:
                    # Compiler-only timeout: never a target/daemon/helper process.
                    p.terminate()
                    p.wait(timeout=10)
                    raise RuntimeError(label + ': timeout; no retry')
                time.sleep(.1 if compiler else .02)
            result = {'label': label, 'exit': p.returncode, 'elapsed_seconds': time.monotonic() - start,
                      'stdout_sha256': sha(RAW / (label + '.stdout')), 'stderr_sha256': sha(RAW / (label + '.stderr'))}
        save(RAW / (label + '-result.json'), result)
        records.append(result)
        if compiler:
            save(RAW / (label + '-priority.json'), samples)
            assert samples and all(x['nice'] == 15 for s in samples for x in s['owned'])
        assert p.returncode == 0, label + ': first failure retained; no retry'
        return (RAW / (label + '.stdout')).read_text()

    profile = '(version 1)(allow default)(deny network*)' + ''.join(
        '(deny file-write* (subpath ' + json.dumps(str(p)) + '))'
        for p in [REPO, BUILD, STOPPED.parent.parent, REGISTRY, root / 'build/cargo-home/registry'])
    # Evidence is written by this parent outside the sandbox. Children may not edit the repository.
    save(RAW / 'sandbox.json', {'profile': profile})
    source = json.loads((REPAIR / 'build-source.json').read_text())
    for row in source:
        assert (REPO / row['path']).stat().st_size == row['bytes'] and sha(REPO / row['path']) == row['sha256']
    assert {str(p.relative_to(REPO)) for p in (REPO / 'apps/macos/Sources').rglob('*.swift')} == {
        r['path'] for r in source if r['path'].endswith('.swift')}
    save(RAW / 'app-source-binding.json', source)
    for row in json.loads((REPAIR / 'artifact-audit.json').read_text())['products']:
        p = Path(row['path'])
        assert sha(p) == row['sha256'] and p.stat().st_size == row['bytes']
    selected = json.loads((REPAIR / 'cache-copy-after.json').read_text())
    for row in selected:
        p = BUILD / 'swift' / row['path']
        assert os.readlink(p) == row['symlink'] if 'symlink' in row else sha(p) == row['sha256'] and p.stat().st_size == row['bytes']
    save(RAW / 'dependency-binding.json', selected)
    # Preserve historical byte evidence through a new hash inventory; do not rewrite it.
    history = {}
    for folder in ['private-update-bootstrap-2026-10-23', 'private-update-bootstrap-repair-2026-10-23-t3-54de37ad',
                   'sparkle-stopped-fixtures-2026-10-23', 'sparkle-stopped-runtime-2026-10-23',
                   'sparkle-cli-build-preparation-2026-10-23']:
        for p in (REPO / 'docs/verification' / folder).rglob('*'):
            if p.is_file():
                history[str(p.relative_to(REPO))] = sha(p)
    save(RAW / 'history-before.json', history)
    # The old daemon's broad saved source inventory differs. Do not reuse its product.
    old = json.loads(Path('/private/tmp/oe-priority-package-sr3z_6fy/source-inputs.json').read_text())
    cargo_paths = subprocess.check_output(['git', 'ls-files', '-z', 'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml',
                            'crates', 'vendor', '.cargo'], cwd=REPO).decode().split('\0')
    rust_source = {n: sha(REPO / n) for n in cargo_paths if n and (REPO / n).is_file()}
    save(RAW / 'daemon-source-binding.json', rust_source)
    differences = [n for n, h in rust_source.items() if old.get(n) != h]
    save(RAW / 'daemon-reuse-decision.json', {'reuse': False, 'different_or_undeclared_inputs': differences,
         'decision': 'fresh scoped native compile; no old Cargo or Swift build-state clone'})
    registry_before = public_files(REGISTRY)
    save(RAW / 'registry-before.json', registry_before)
    shutil.copytree(REGISTRY, root / 'build/cargo-home/registry', symlinks=True)
    assert public_files(root / 'build/cargo-home/registry') == registry_before
    run('rust-version', [TOOLS / 'rustc', '--version', '--verbose'])
    assert 'rustc 1.99.1 ' in (RAW / 'rust-version.stdout').read_text()
    save(RAW / 'compiler-inputs.json', {str(p): sha(p) for p in [TOOLS / 'cargo', TOOLS / 'rustc',
         Path('/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/clang')]})
    run('daemon-build', [TOOLS / 'cargo', 'build', '--release', '--locked', '--offline', '--jobs', '2',
                         '--package', 'openergy-daemon', '--bin', 'openergy', '--target', 'aarch64-apple-darwin'],
        timeout=600, compiler=True)
    daemon = root / 'build/target/aarch64-apple-darwin/release/openergy'
    run('daemon-load', ['/usr/bin/otool', '-l', daemon])
    assert 'minos 15.0' in (RAW / 'daemon-load.stdout').read_text()
    save(RAW / 'daemon-artifact.json', {'path': str(daemon), 'bytes': daemon.stat().st_size, 'sha256': sha(daemon),
         'kind': 'fresh native arm64 release daemon; never executed'})
    assert public_files(REGISTRY) == registry_before
    assert public_files(root / 'build/cargo-home/registry') == registry_before
    assert all(sha(REPO / n) == h for n, h in rust_source.items())
    save(RAW / 'build-postflight.json', {'original_and_copy_registry_byte_identity': True,
         'rust_source_identity': True, 'cargo_target_was_fresh': True, 'product_executed': False})
    # Bind the complete old repaired driver before copying it; source export must match the saved official pin.
    saved_driver = json.loads(gzip.decompress((REPO / 'docs/verification/sparkle-stopped-fixtures-2026-10-23/raw/prepared-driver-tree.json.gz').read_bytes()))
    observed = inventory(STOPPED)
    assert {k: v for k, v in observed.items() if not v.get('directory')} == saved_driver
    cli_source = Path('/private/tmp/oe-sparkle-cli-build-njko0i6t/source')
    expected_source = json.loads(gzip.decompress((REPO / 'docs/verification/sparkle-cli-build-preparation-2026-10-23/raw/source-files.json.gz').read_bytes()))
    assert public_files(cli_source) == expected_source
    save(RAW / 'driver-source-binding.json', {'pin': PIN, 'source_inventory': expected_source,
         'retained_driver_tree': observed, 'retained_driver_unchanged_required': True})
    run('retained-driver-seal', ['/usr/bin/codesign', '--verify', '--all-architectures', '--deep', '--strict', STOPPED])
    run('copy-driver', ['/usr/bin/ditto', STOPPED, root / 'driver/sparkle.app'])
    driver = root / 'driver/sparkle.app'
    info_path = driver / 'Contents/Info.plist'
    info = plistlib.loads(info_path.read_bytes())
    info.update(CFBundleIdentifier=fixture + '.driver', SUDefaultsDomain=str(root / 'driver-home/preferences'))
    info_path.write_bytes(plistlib.dumps(info))
    run('sign-driver', ['/usr/bin/codesign', '--force', '--options', '0', '--timestamp=none', '--sign', '-', driver])
    run('driver-seal', ['/usr/bin/codesign', '--verify', '--all-architectures', '--deep', '--strict', driver])
    run('driver-link', ['/usr/bin/otool', '-L', driver / 'Contents/MacOS/sparkle'])
    # A new identity/payload needs a fresh public key. Old public signed bytes cannot bind this archive.
    generator = root / 'build/keygen.swift'
    generator.write_text('import Foundation\nimport CryptoKit\n'
        'let root = URL(fileURLWithPath: CommandLine.arguments[1])\n'
        'let key = Curve25519.Signing.PrivateKey()\n'
        'try Data(key.rawRepresentation.base64EncodedString().utf8).write(to: root.appendingPathComponent("disposable-seed"), options: .withoutOverwriting)\n'
        'try Data(key.publicKey.rawRepresentation.base64EncodedString().utf8).write(to: root.appendingPathComponent("public-key.txt"), options: .withoutOverwriting)\n')
    key = root / 'disposable-seed'
    verifier = root / 'public-verifier'
    try:
        run('fresh-file-key', ['/usr/bin/swift', '-module-cache-path', root / 'build/modules', generator, root], compiler=True)
        assert stat.S_IMODE(key.stat().st_mode) == 0o600
        public = (root / 'public-key.txt').read_text()
        pub = base64.b64decode(public, validate=True)
        assert len(pub) == 32
        verifier.write_bytes(base64.b64encode(bytes(64) + pub))
        framework_root = BUILD / 'swift/artifacts/sparkle/Sparkle'
        framework = framework_root / 'Sparkle.xcframework/macos-arm64_x86_64/Sparkle.framework'
        swift_image = BUILD / 'swift/arm64-apple-macosx/debug/Openergy'
        launcher = BUILD / 'launcher/OpenergyPrivateLauncher'
        assert (REPO / 'apps/macos/PrivateUpdateBootstrap/Info.plist.in').read_bytes() + b'\0' in launcher.read_bytes()
        snapshots = {}
        for build, label in [('2', 'candidate'), ('1', 'old')]:
            app = root / 'target/Openergy.app'
            for n in ['', 'Contents', 'Contents/MacOS', 'Contents/Helpers', 'Contents/Frameworks', 'Contents/Resources']:
                (app / n).mkdir(mode=0o700)
            shutil.copy2(swift_image, app / 'Contents/MacOS/Openergy')
            shutil.copy2(daemon, app / 'Contents/Helpers/openergy')
            for p in [app / 'Contents/MacOS/Openergy', app / 'Contents/Helpers/openergy']:
                p.chmod(0o700)
            run(label + '-framework-copy', ['/usr/bin/ditto', framework, app / 'Contents/Frameworks/Sparkle.framework'])
            shutil.copy2(framework_root / 'LICENSE', app / 'Contents/Resources/Sparkle-LICENSE.txt')
            for n in ['LICENSE-MIT', 'LICENSE-APACHE', 'THIRD_PARTY_NOTICES.md']:
                shutil.copy2(REPO / n, app / 'Contents/Resources' / n)
            metadata = dict(Schema=1, Root=str(root), Nonce=nonce, Port=port)
            initial = dict(CFBundleIdentifier=fixture, CFBundleExecutable='Openergy', CFBundleVersion=build,
                           SUPublicEDKey=public, OpenergyPrivateUpdateFixture=metadata)
            (app / 'Contents/Info.plist').write_bytes(plistlib.dumps(initial))
            # First real use of reviewed packager; any failure aborts without source fix or retry.
            run(label + '-package-entry', [PYTHON, REPO / 'apps/macos/PrivateUpdateBootstrap/package-entry.py',
                                           '--app', app, '--launcher', launcher])
            (app / 'Contents/PkgInfo').write_bytes(b'APPL????')
            fw = app / 'Contents/Frameworks/Sparkle.framework'
            version = fw / 'Versions/B'
            nested = [version / 'Autoupdate', version / 'Updater.app', version / 'XPCServices/Downloader.xpc',
                      version / 'XPCServices/Installer.xpc', fw]
            for i, p in enumerate(nested):
                run(label + '-sign-nested-' + str(i), ['/usr/bin/codesign', '--force', '--options', '0',
                    '--timestamp=none', '--sign', '-', '--preserve-metadata=identifier,entitlements', p])
            for name, relative in [('swift', 'MacOS/Openergy'), ('daemon', 'Helpers/openergy')]:
                run(label + '-sign-' + name, ['/usr/bin/codesign', '--force', '--options', '0',
                    '--timestamp=none', '--sign', '-', '--identifier', fixture + '.' + name, app / 'Contents' / relative])
            run(label + '-sign-outer', ['/usr/bin/codesign', '--force', '--options', '0', '--timestamp=none', '--sign', '-', app])
            for i, p in enumerate(nested + [app / 'Contents/MacOS/Openergy', app / 'Contents/Helpers/openergy', app]):
                run(label + '-seal-' + str(i), ['/usr/bin/codesign', '--verify', '--all-architectures', '--strict', p])
            run(label + '-deep-seal', ['/usr/bin/codesign', '--verify', '--all-architectures', '--deep', '--strict', app])
            for name in ['Openergy', 'OpenergyPrivateLauncher']:
                run(label + '-entitlements-' + name, ['/usr/bin/codesign', '-d', '--entitlements', ':-', app / 'Contents/MacOS' / name])
                assert b'com.apple.security.cs.disable-library-validation' not in (RAW / (label + '-entitlements-' + name + '.stdout')).read_bytes()
            expected = (REPO / 'apps/macos/PrivateUpdateBootstrap/Info.plist.in').read_text()
            for k, v in dict(FIXTURE_ID=fixture, ROOT=str(root), NONCE=nonce, PORT=str(port), BUILD=build, PUBLIC_ED_KEY=public).items():
                expected = expected.replace('@' + k + '@', v)
            assert (app / 'Contents/Info.plist').read_text() == expected
            snapshots[label] = inventory(app)
            if label == 'candidate':
                app.rename(root / 'candidate/Openergy.app')
            else:
                shutil.copytree(app, root / 'old/Openergy.app', symlinks=True)
        assert snapshots['old'] == inventory(root / 'old/Openergy.app')
        save(RAW / 'bundle-inventories.json', snapshots)
        save(RAW / 'driver-inventory.json', inventory(driver))
        payload = root / 'public/openergy-fixture.zip'
        original = root / 'build/candidate-ditto.zip'
        run('archive', ['/usr/bin/ditto', '-c', '-k', '--sequesterRsrc', '--keepParent', root / 'candidate/Openergy.app', original])
        with zipfile.ZipFile(original) as z, zipfile.ZipFile(payload, 'x', compression=zipfile.ZIP_DEFLATED, compresslevel=9) as out:
            for entry in sorted(z.infolist(), key=lambda e: e.filename):
                item = zipfile.ZipInfo(entry.filename, (1980, 1, 1, 0, 0, 0))
                item.create_system = 3
                item.external_attr = entry.external_attr
                item.compress_type = zipfile.ZIP_DEFLATED
                out.writestr(item, z.read(entry))
        run('roundtrip-extract', ['/usr/bin/ditto', '-x', '-k', payload, root / 'roundtrip'])
        assert inventory(root / 'roundtrip/Openergy.app') == snapshots['candidate']
        run('roundtrip-seal', ['/usr/bin/codesign', '--verify', '--all-architectures', '--deep', '--strict', root / 'roundtrip/Openergy.app'])
        signer = framework_root / 'bin/sign_update'
        run('official-signer-seal', ['/usr/bin/codesign', '--verify', '--all-architectures', '--strict', signer])
        signature = run('sign-archive', [signer, '--ed-key-file', key, '-p', payload]).strip()
        assert len(base64.b64decode(signature, validate=True)) == 64
        run('verify-archive', [signer, '--verify', '--ed-key-file', verifier, payload, signature])
        ET.register_namespace('sparkle', NS)
        feed_url = f'http://127.0.0.1:{port}/appcast.xml'
        download_url = f'http://127.0.0.1:{port}/openergy-fixture.zip'
        rss = ET.Element('rss', {'version': '2.0'})
        channel = ET.SubElement(rss, 'channel')
        ET.SubElement(channel, 'title').text = 'Openergy private running-target fixture'
        ET.SubElement(channel, 'link').text = feed_url
        item = ET.SubElement(channel, 'item')
        ET.SubElement(item, 'title').text = 'Private build 2'
        for k, v in [('version', '2'), ('shortVersionString', '0.1.0'), ('minimumSystemVersion', '15.0')]:
            ET.SubElement(item, '{' + NS + '}' + k).text = v
        ET.SubElement(item, 'enclosure', {'url': download_url, 'length': str(payload.stat().st_size),
            'type': 'application/octet-stream', '{' + NS + '}edSignature': signature})
        ET.indent(rss)
        feed = root / 'public/appcast.xml'
        feed.write_bytes(ET.tostring(rss, encoding='utf-8', xml_declaration=True) + b'\n')
        run('sign-feed', [signer, '--ed-key-file', key, feed])
        run('verify-feed', [signer, '--verify', '--ed-key-file', verifier, feed])
        assert ET.parse(feed).find('channel/item/enclosure').attrib['length'] == str(payload.stat().st_size)
        save(RAW / 'public-fixtures.json', {'public_key': public, 'archive_signature': signature,
            'feed_url': feed_url, 'download_url': download_url, 'signer_sha256': sha(signer),
            'files': {p.name: {'bytes': p.stat().st_size, 'sha256': sha(p)} for p in [payload, feed]},
            'normal_signature_checks': True, 'server_bound': False, 'feed_requested': False,
            'production_sparkle_py_unchanged': sha(REPO / 'tools/release/sparkle.py')})
        shutil.copy2(feed, RAW / 'appcast.xml')
        assert inventory(STOPPED) == observed
        assert all(sha(REPO / n) == h for n, h in history.items())
        save(RAW / 'final-file-checks.json', {'history_byte_exact': True, 'retained_driver_byte_mode_link_exact': True,
             'archive_roundtrip_exact': True, 'official_archive_and_full_feed_verified': True,
             'target_launched': False, 'daemon_executed': False, 'cli_installer_helper_executed': False,
             'runtime_approved': False, 'kind': 'DEBUG Swift fixture; not a final release candidate'})
    finally:
        key.unlink(missing_ok=True)
        verifier.unlink(missing_ok=True)
        save(RAW / 'key-deletion.json', {'disposable_key_absent': not key.exists(),
             'public_verifier_absent': not verifier.exists(), 'preexisting_key_or_keychain_access': False,
             'key_material_logged_or_committed': False})
    save(RAW / 'preparation-complete.json', {'root': str(root), 'commands': len(records),
         'scope': 'file operations and scoped native compile only; no runtime acceptance', 'approval_received': False})
    print(json.dumps({'root': str(root), 'status': 'file-only preparation complete; DO-NOT-RUN'}))


if __name__ == '__main__':
    try:
        main()
    except BaseException:
        # Includes the first failed assertion/tool traceback, never secrets.
        RAW.mkdir(mode=0o700, exist_ok=True)
        with (RAW / 'first-preparation-failure.txt').open('x') as f:
            traceback.print_exc(file=f)
        raise
