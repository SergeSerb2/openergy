#!/usr/bin/env python3
"""Build-only evidence controller. Never invokes the product or a bundle script."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import tempfile
import time

REPO = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent
CACHE = Path('/var/folders/s6/_07cr4w13vsg_myjkv9ldpqc0000gn/T/oe-sparkle-combined-20261023-a_es9dei/swift')
PIN = 'eef1a539a373c1f1a320624b1130fc5de7b2e100'
SWIFT = '/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/swift'


def save(name, value):
    with (OUT / name).open('x') as stream:
        json.dump(value, stream, indent=2, sort_keys=True)
        stream.write('\n')


def sha(path):
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(chunk)
    return digest.hexdigest()


def inventory(root, subdirs):
    entries = []
    for subdir in subdirs:
        for path in sorted((root / subdir).rglob('*')):
            relative = str(path.relative_to(root))
            if path.is_symlink():
                entries.append(dict(path=relative, symlink=os.readlink(path)))
            elif path.is_file():
                entries.append(dict(path=relative, bytes=path.stat().st_size, sha256=sha(path)))
    return entries


def nice15():
    # Absolute priority, once. Do not add nice increments to inherited values.
    assert os.getpriority(os.PRIO_PROCESS, 0) <= 15
    os.setpriority(os.PRIO_PROCESS, 0, 15)
    assert os.getpriority(os.PRIO_PROCESS, 0) == 15
    os.umask(0o077)


def main():
    root = Path(tempfile.mkdtemp(prefix='oe-update-bootstrap-build-', dir='/private/tmp'))
    os.chmod(root, 0o700)
    save('preparation.json', dict(private_root=str(root), source_parent=subprocess.check_output(
        ['git', 'rev-parse', 'HEAD'], cwd=REPO, text=True).strip(), historical_cache=str(CACHE),
        sparkle_revision=PIN, scope='SOURCE + BUILD ONLY; no product or installer execution'))
    subdirs = ['artifacts', 'checkouts', 'repositories']
    original = inventory(CACHE, subdirs)
    save('cache-before.json', original)
    scratch = root / 'swift'
    scratch.mkdir(mode=0o700)
    for name in subdirs:
        shutil.copytree(CACHE / name, scratch / name, symlinks=True)
    copied = inventory(scratch, subdirs)
    save('cache-copy.json', copied)
    assert original == copied
    state = json.loads((CACHE / 'workspace-state.json').read_text())
    assert state['object']['dependencies'][0]['state']['checkoutState']['revision'] == PIN
    for artifact in state['object']['artifacts']:
        old = artifact['path']
        assert old.startswith(str(CACHE) + '/')
        artifact['path'] = str(scratch) + old[len(str(CACHE)):]
    (scratch / 'workspace-state.json').write_text(json.dumps(state, indent=2) + '\n')
    for name in ['home', 'tmp', 'module-cache', 'swift-cache', 'swift-config', 'swift-security']:
        (root / name).mkdir(mode=0o700)
    sources = [REPO / 'apps/macos/Package.swift', REPO / 'apps/macos/Package.resolved',
               REPO / 'apps/macos/Support/Info.plist']
    sources += sorted((REPO / 'apps/macos/Sources').rglob('*.swift'))
    save('build-source.json', [dict(path=str(p.relative_to(REPO)), bytes=p.stat().st_size,
                                  sha256=sha(p)) for p in sources])
    environment = dict(os.environ)
    overrides = dict(HOME=str(root / 'home'), CFFIXED_USER_HOME=str(root / 'home'),
                     TMPDIR=str(root / 'tmp') + '/', MACOSX_DEPLOYMENT_TARGET='15.0',
                     CLANG_MODULE_CACHE_PATH=str(root / 'module-cache'),
                     SWIFTPM_MODULECACHE_OVERRIDE=str(root / 'module-cache'))
    environment.update(overrides)
    command = [SWIFT, 'build', '--package-path', str(REPO / 'apps/macos'),
               '--scratch-path', str(scratch), '--cache-path', str(root / 'swift-cache'),
               '--config-path', str(root / 'swift-config'), '--security-path', str(root / 'swift-security'),
               '--manifest-cache', 'local', '--disable-dependency-cache', '--disable-automatic-resolution',
               '--skip-update', '--disable-prefetching', '--disable-netrc', '--disable-keychain',
               '--build-system', 'native', '--jobs', '2', '--disable-index-store', '--product', 'Openergy', '-v']
    save('command.json', dict(argv=command, environment_overrides=overrides,
                              preexec='setpriority(PRIO_PROCESS, 0, 15); verify exactly 15; umask(077)',
                              swift_realpath=str(Path(SWIFT).resolve()), swift_sha256=sha(Path(SWIFT).resolve())))
    start = time.monotonic()
    with (OUT / 'build.stdout').open('xb') as stdout, (OUT / 'build.stderr').open('xb') as stderr:
        process = subprocess.Popen(command, cwd=REPO, env=environment, stdout=stdout, stderr=stderr,
                                   preexec_fn=nice15, start_new_session=True)
        samples = []
        timed_out = False
        while process.poll() is None:
            rows = subprocess.check_output(['/bin/ps', '-axo', 'pid=,ppid=,ni=,comm='], text=True)
            parsed = []
            for line in rows.splitlines():
                fields = line.strip().split(None, 3)
                if len(fields) == 4:
                    parsed.append(dict(pid=int(fields[0]), ppid=int(fields[1]),
                                       nice=int(fields[2]), executable=fields[3]))
            owned = {process.pid}
            for _ in range(16):
                added = {row['pid'] for row in parsed if row['ppid'] in owned}
                if added <= owned:
                    break
                owned |= added
            for row in parsed:
                if row['pid'] in owned:
                    samples.append(dict(elapsed_seconds=round(time.monotonic() - start, 6), **row))
            if time.monotonic() - start > 600:
                timed_out = True
                os.killpg(process.pid, signal.SIGTERM)
                break
            time.sleep(0.10)
        exit_code = process.wait(timeout=30)
    save('priority-samples.json', samples)
    after = inventory(CACHE, subdirs)
    save('cache-after.json', after)
    product = scratch / 'arm64-apple-macosx/debug/Openergy'
    result = dict(exit=exit_code, timed_out=timed_out, elapsed_seconds=round(time.monotonic() - start, 6),
                  original_cache_unchanged=original == after,
                  historical_workspace_state_sha256=sha(CACHE / 'workspace-state.json'),
                  sampled_nice_values=sorted({sample['nice'] for sample in samples}),
                  compiler_samples=sum('swift-frontend' in sample['executable'] for sample in samples),
                  stdout_sha256=sha(OUT / 'build.stdout'), stderr_sha256=sha(OUT / 'build.stderr'),
                  product_path=str(product), product_exists=product.is_file())
    if product.is_file():
        result.update(product_bytes=product.stat().st_size, product_sha256=sha(product))
    save('result.json', result)
    print(json.dumps(result, sort_keys=True), flush=True)
    assert exit_code == 0 and not timed_out and original == after
    assert samples and result['sampled_nice_values'] == [15] and result['compiler_samples'] > 0


if __name__ == '__main__':
    main()
