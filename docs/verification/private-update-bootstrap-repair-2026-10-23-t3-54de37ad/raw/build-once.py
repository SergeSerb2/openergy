#!/usr/bin/env python3
"""Build only. Does not invoke any app, launcher, daemon or packaging script."""
import ast
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import signal
import subprocess
import tempfile
import time

REPO = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent
CACHE = Path('/var/folders/s6/_07cr4w13vsg_myjkv9ldpqc0000gn/T/oe-sparkle-combined-20261023-a_es9dei/swift')
PIN = 'eef1a539a373c1f1a320624b1130fc5de7b2e100'
TOOLCHAIN = Path('/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin')
SDK = '/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk'
SANDBOX = ['sandbox-exec', '-p', '(version 1)(allow default)(deny network*)']


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
    assert os.getpriority(os.PRIO_PROCESS, 0) <= 15
    os.setpriority(os.PRIO_PROCESS, 0, 15)
    assert os.getpriority(os.PRIO_PROCESS, 0) == 15
    os.umask(0o077)


def compile_once(name, command, environment):
    save(name + '-command.json', dict(argv=command, environment=environment,
         preexec='setpriority(PRIO_PROCESS,0,15); verify 15; umask(077)',
         controller_nice=os.getpriority(os.PRIO_PROCESS, 0)))
    start = time.monotonic()
    samples, timed_out = [], False
    with (OUT / (name + '.stdout')).open('xb') as stdout, (OUT / (name + '.stderr')).open('xb') as stderr:
        process = subprocess.Popen(command, cwd=REPO, env=environment, stdout=stdout, stderr=stderr,
                                   preexec_fn=nice15, start_new_session=True)
        while process.poll() is None:
            rows = subprocess.check_output(['/bin/ps', '-axo', 'pid=,ppid=,ni=,comm='], text=True)
            parsed = []
            for line in rows.splitlines():
                fields = line.strip().split(None, 3)
                if len(fields) == 4:
                    parsed.append(dict(pid=int(fields[0]), ppid=int(fields[1]),
                                       nice=int(fields[2]), executable=fields[3]))
            owned = {process.pid}
            for _ in range(32):
                added = {row['pid'] for row in parsed if row['ppid'] in owned}
                if added <= owned: break
                owned |= added
            samples += [dict(elapsed_seconds=round(time.monotonic() - start, 6), **row)
                        for row in parsed if row['pid'] in owned]
            if time.monotonic() - start > 600:
                timed_out = True
                os.killpg(process.pid, signal.SIGTERM)
                break
            time.sleep(0.05)
        code = process.wait(timeout=30)
    save(name + '-priority.json', samples)
    result = dict(exit=code, timed_out=timed_out, elapsed_seconds=round(time.monotonic() - start, 6),
        sampled_nice_values=sorted({s['nice'] for s in samples}), sampled_pids=sorted({s['pid'] for s in samples}),
        compiler_samples=sum('swift-frontend' in s['executable'] or 'clang' in s['executable'] for s in samples),
        stdout_sha256=sha(OUT / (name + '.stdout')), stderr_sha256=sha(OUT / (name + '.stderr')))
    save(name + '-result.json', result)
    print(name + ': ' + json.dumps(result, sort_keys=True), flush=True)
    assert code == 0 and not timed_out and samples and result['sampled_nice_values'] == [15]
    return result


def main():
    nice15()
    root = Path(tempfile.mkdtemp(prefix='oe-bootstrap-r2-build-', dir='/private/tmp'))
    os.chmod(root, 0o700)
    save('preparation.json', dict(private_root=str(root), source_parent=subprocess.check_output(
        ['git', 'rev-parse', 'HEAD'], cwd=REPO, text=True).strip(), historical_cache=str(CACHE),
        sparkle_revision=PIN, scope='SOURCE + WHOLE-APP BUILD ONLY; launcher never executed',
        controller_pid=os.getpid(), controller_nice=os.getpriority(os.PRIO_PROCESS, 0)))
    subdirs = ['artifacts', 'checkouts', 'repositories']
    original = inventory(CACHE, subdirs)
    save('cache-before.json', original)
    # Confirm the available cache still matches the complete held cache record.
    held = json.loads((REPO / 'docs/verification/private-update-bootstrap-2026-10-23/raw/cache-before.json').read_text())
    assert original == held
    scratch = root / 'swift'
    scratch.mkdir(mode=0o700)
    for name in subdirs: shutil.copytree(CACHE / name, scratch / name, symlinks=True)
    copied = inventory(scratch, subdirs)
    save('cache-copy.json', copied)
    assert original == copied
    assert subprocess.check_output(['git', '-C', str(scratch / 'checkouts/Sparkle'), 'rev-parse', 'HEAD'],
                                   text=True).strip() == PIN
    state = json.loads((CACHE / 'workspace-state.json').read_text())
    assert state['object']['dependencies'][0]['state']['checkoutState']['revision'] == PIN
    original_state_hash = sha(CACHE / 'workspace-state.json')
    for artifact in state['object']['artifacts']:
        old = artifact['path']
        assert old.startswith(str(CACHE) + '/')
        artifact['path'] = str(scratch) + old[len(str(CACHE)):]
    (scratch / 'workspace-state.json').write_text(json.dumps(state, indent=2) + '\n')
    for name in ['home', 'tmp', 'module-cache', 'swift-cache', 'swift-config', 'swift-security', 'launcher']:
        (root / name).mkdir(mode=0o700)
    source_dir = REPO / 'apps/macos/PrivateUpdateBootstrap'
    template = (source_dir / 'Info.plist.in').read_text(encoding='ascii')
    tokens = re.findall(r'@([A-Z_]+)@', template)
    assert sorted(tokens) == sorted(['FIXTURE_ID', 'ROOT', 'NONCE', 'PORT', 'BUILD', 'PUBLIC_ED_KEY'])
    header = root / 'launcher/private-info-template.h'
    header.write_text('static const char private_info_template[] = ' + json.dumps(template) + ';\n')
    shutil.copyfile(header, OUT / 'private-info-template.h')
    # Syntax-read only: never import or execute the packaging implementation.
    ast.parse((source_dir / 'package-entry.py').read_text())
    sources = [REPO / 'apps/macos/Package.swift', REPO / 'apps/macos/Package.resolved',
               REPO / 'apps/macos/Support/Info.plist', REPO / 'apps/macos/scripts/bundle.sh']
    sources += sorted((REPO / 'apps/macos/Sources').rglob('*.swift')) + sorted(source_dir.iterdir())
    save('build-source.json', [dict(path=str(p.relative_to(REPO)), bytes=p.stat().st_size,
                                  sha256=sha(p)) for p in sources])
    environment = dict(PATH='/usr/bin:/bin', LANG='C', HOME=str(root / 'home'),
        CFFIXED_USER_HOME=str(root / 'home'), TMPDIR=str(root / 'tmp') + '/',
        MACOSX_DEPLOYMENT_TARGET='15.0', SDKROOT=SDK,
        CLANG_MODULE_CACHE_PATH=str(root / 'module-cache'),
        SWIFTPM_MODULECACHE_OVERRIDE=str(root / 'module-cache'),
        XDG_CONFIG_HOME=str(root / 'swift-config'), XDG_CACHE_HOME=str(root / 'swift-cache'))
    launcher = root / 'launcher/OpenergyPrivateLauncher'
    clang = SANDBOX + [str(TOOLCHAIN / 'clang'), '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror',
        '-target', 'arm64-apple-macosx15.0', '-isysroot', SDK, '-I', str(root / 'launcher'),
        str(source_dir / 'launcher.c'), '-o', str(launcher), '-v']
    compile_once('launcher', clang, environment)
    swift = SANDBOX + [str(TOOLCHAIN / 'swift'), 'build', '--package-path', str(REPO / 'apps/macos'),
        '--scratch-path', str(scratch), '--cache-path', str(root / 'swift-cache'),
        '--config-path', str(root / 'swift-config'), '--security-path', str(root / 'swift-security'),
        '--manifest-cache', 'local', '--disable-dependency-cache', '--disable-automatic-resolution',
        '--skip-update', '--disable-prefetching', '--disable-netrc', '--disable-keychain', '--disable-sandbox',
        '--build-system', 'native', '--jobs', '2', '--disable-index-store', '--product', 'Openergy', '-v']
    compile_once('app', swift, environment)
    after = inventory(CACHE, subdirs)
    used = inventory(scratch, subdirs)
    save('cache-after.json', after)
    save('cache-copy-after.json', used)
    assert original == after == copied == used and original_state_hash == sha(CACHE / 'workspace-state.json')
    products = [launcher, scratch / 'arm64-apple-macosx/debug/Openergy']
    audit = []
    for product in products:
        for tool, args in [('otool', ['-l']), ('otool', ['-L']), ('file', [])]:
            name = product.name + '-' + tool + ''.join(args).replace('-', '')
            command = ['/usr/bin/' + tool] + args + [str(product)]
            result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
            (OUT / (name + '.stdout')).write_bytes(result.stdout)
            (OUT / (name + '.stderr')).write_bytes(result.stderr)
            audit.append(dict(argv=command, exit=result.returncode,
                stdout_sha256=sha(OUT / (name + '.stdout')), stderr_sha256=sha(OUT / (name + '.stderr'))))
            assert result.returncode == 0
    libraries = (OUT / 'OpenergyPrivateLauncher-otoolL.stdout').read_text()
    assert '\t/usr/lib/libSystem.B.dylib ' in libraries and len(libraries.strip().splitlines()) == 2
    macho = (OUT / 'OpenergyPrivateLauncher-otooll.stdout').read_text()
    assert 'minos 15.0' in macho and 'LC_RPATH' not in macho and 'LC_DYLD_ENVIRONMENT' not in macho
    save('artifact-audit.json', dict(kind='read-only load-command/file audit; neither executable invoked',
        commands=audit, launcher_libraries=['/usr/lib/libSystem.B.dylib'],
        launcher_minimum_os='15.0', cache_original_and_copies_unchanged=True,
        historical_workspace_state_sha256=original_state_hash,
        template_header_sha256=sha(header),
        products=[dict(path=str(p), bytes=p.stat().st_size, sha256=sha(p)) for p in products]))
    print('artifact and unchanged-cache audit complete', flush=True)


if __name__ == '__main__':
    main()
