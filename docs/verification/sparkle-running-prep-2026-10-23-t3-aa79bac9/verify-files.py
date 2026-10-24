"""One additional file-only metadata audit. Never execute any inspected product."""
import ast
import hashlib
import json
import os
from pathlib import Path
import stat
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = Path(json.loads((HERE / 'fixture.json').read_text())['root'])
OUT = HERE / 'static'
os.umask(0o077)
os.setpriority(os.PRIO_PROCESS, 0, 15)
assert os.getpriority(os.PRIO_PROCESS, 0) == 15
OUT.mkdir(mode=0o700)
env = dict(PATH='/usr/bin:/bin:/usr/sbin:/sbin', LANG='C', HOME=str(ROOT / 'build/home'),
           CFFIXED_USER_HOME=str(ROOT / 'build/home'), TMPDIR=str(ROOT / 'build/tmp') + '/')
records = []


def sha(p):
    with p.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def run(label, args):
    args = list(map(str, args))
    p = subprocess.run(['/usr/bin/sandbox-exec', '-p', '(version 1)(allow default)(deny network*)', *args],
                       env=env, capture_output=True, timeout=30)
    for suffix, data in [('stdout', p.stdout), ('stderr', p.stderr)]:
        with (OUT / (label + '.' + suffix)).open('xb') as f:
            f.write(data)
    records.append(dict(label=label, argv=args, exit=p.returncode,
        stdout_sha256=sha(OUT / (label + '.stdout')), stderr_sha256=sha(OUT / (label + '.stderr'))))
    assert p.returncode == 0, label
    return p.stdout + p.stderr


for label, bundle in [('old', ROOT / 'target/Openergy.app'), ('candidate', ROOT / 'candidate/Openergy.app'),
                      ('driver', ROOT / 'driver/sparkle.app')]:
    framework = bundle / 'Contents/Frameworks/Sparkle.framework'
    version = framework / 'Versions/B'
    binaries = [version / 'Sparkle', version / 'Autoupdate', version / 'Updater.app/Contents/MacOS/Updater',
                version / 'XPCServices/Downloader.xpc/Contents/MacOS/Downloader',
                version / 'XPCServices/Installer.xpc/Contents/MacOS/Installer']
    binaries += ([bundle / 'Contents/MacOS/sparkle'] if label == 'driver' else [
        bundle / 'Contents/MacOS/OpenergyPrivateLauncher', bundle / 'Contents/MacOS/Openergy', bundle / 'Contents/Helpers/openergy'])
    for i, binary in enumerate(binaries):
        base = label + '-' + str(i)
        run(base + '-arch', ['/usr/bin/lipo', '-verify_arch', 'arm64', binary])
        links = run(base + '-links', ['/usr/bin/otool', '-L', binary]).decode()
        for line in links.splitlines():
            if line.startswith('\t'):
                assert line.strip().startswith(('/System/Library/', '/usr/lib/', '@rpath/', '@loader_path/', '@executable_path/'))
        commands = run(base + '-loads', ['/usr/bin/otool', '-l', binary])
        assert b'LC_DYLD_ENVIRONMENT' not in commands
        if binary.name in ['Openergy', 'OpenergyPrivateLauncher', 'openergy', 'sparkle']:
            assert b'minos 15.0' in commands
        entitlements = run(base + '-entitlements', ['/usr/bin/codesign', '-d', '--entitlements', ':-', binary])
        assert b'com.apple.security.cs.disable-library-validation' not in entitlements
    run(label + '-outer-seal', ['/usr/bin/codesign', '--verify', '--all-architectures', '--deep', '--strict', bundle])
    for p in framework.rglob('*'):
        if p.is_symlink():
            assert p.resolve(strict=True).is_relative_to(framework.resolve())

assert not (ROOT / 'disposable-seed').exists() and not (ROOT / 'public-verifier').exists()
for relative in ['fixture.json', 'launch.lock', 'state/config.toml']:
    s = (ROOT / relative).lstat()
    assert stat.S_ISREG(s.st_mode) and s.st_uid == os.getuid() and s.st_nlink == 1 and stat.S_IMODE(s.st_mode) == 0o600
for relative in ['state', 'state/tls', 'home', 'tmp', 'receipts']:
    s = (ROOT / relative).lstat()
    assert stat.S_ISDIR(s.st_mode) and s.st_uid == os.getuid() and stat.S_IMODE(s.st_mode) == 0o700
assert not (ROOT / 'state/control.sock').exists()
assert not any((ROOT / 'receipts').iterdir())
assert not any((ROOT / 'runtime-evidence').iterdir())
# Parser only: never import, compile to bytecode, invoke, or simulate this controller.
ast.parse((HERE / 'controller-DO-NOT-RUN.py.txt').read_text())
with (OUT / 'audit.json').open('x') as f:
    json.dump(dict(records=records, runtime=False, key_absent=True, private_modes=True,
        kernel_receipts=False, controller_text_AST_only=True, DEBUG_Swift_not_release_candidate=True,
        rpath_limit='DEBUG Swift retains recorded Xcode Swift runtime search path; no executable fallback was run'), f, indent=2, sort_keys=True)
    f.write('\n')
print('File metadata, links, entitlements, deep seals and text syntax checked; no product executed.')
