#!/usr/bin/env python3
"""Opt-in entry packaging for an ALREADY PRIVATE, unsigned fixture bundle.

Never builds, launches, signs, registers, installs, creates a root/marker/config,
or changes the production template. Not called by the normal bundle script.
A failed partial preparation is not a runnable fixture and must not be launched.
"""
import argparse
import base64
import json
import os
from pathlib import Path
import plistlib
import re
import shutil
import stat
import struct

PREFIX = 'dev.openergy.updater-fixture.'
UUID = r'[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}'


def require(ok, reason):
    if not ok:
        raise SystemExit('Private entry packaging refused: ' + reason)


def owned(path, kind, mode=None):
    s = path.lstat()
    require(s.st_uid == os.getuid() and kind(s.st_mode), 'owner or file type')
    require(path.resolve(strict=True) == path, 'noncanonical path')
    if mode is not None:
        require(stat.S_IMODE(s.st_mode) == mode, 'mode')
    else:
        require(s.st_mode & 0o7022 == 0, 'writable or special entry')
    if stat.S_ISREG(s.st_mode):
        require(s.st_nlink == 1, 'hard link')
    return s


def libsystem_launcher(path):
    s = owned(path, stat.S_ISREG)
    require(s.st_mode & 0o100 and s.st_size <= 1024 * 1024, 'launcher size or execute bit')
    data = path.read_bytes()
    require(len(data) >= 32, 'launcher header')
    magic, cpu, subtype, kind, count, size, flags, reserved = struct.unpack_from('<8I', data)
    require(magic == 0xfeedfacf and kind == 2 and flags & 0x200000, 'PIE Mach-O executable required')
    require(cpu in (0x100000c, 0x1000007) and 32 + size <= len(data) and count <= 128,
            'launcher architecture or commands')
    offset, libraries, minimum = 32, [], None
    for _ in range(count):
        require(offset + 8 <= 32 + size, 'truncated load command')
        command, length = struct.unpack_from('<2I', data, offset)
        require(length >= 8 and offset + length <= 32 + size, 'invalid load command')
        if command == 0xc:
            require(length >= 24, 'truncated library command')
            name = struct.unpack_from('<I', data, offset + 8)[0]
            require(24 <= name < length, 'invalid library offset')
            name_bytes = data[offset + name:offset + length]
            require(b'\0' in name_bytes, 'unterminated library name')
            libraries.append(name_bytes.split(b'\0', 1)[0])
        # Refuse every other dylib load form, rpath, and dyld environment command.
        require(command not in (0x18, 0x80000018, 0x8000001f, 0x20, 0x80000023, 0x8000001c, 0x27),
                'unexpected dynamic load command')
        if command == 0x32:
            require(length >= 24, 'truncated platform command')
            platform, minimum, sdk, tools = struct.unpack_from('<4I', data, offset + 8)
            require(platform == 1, 'macOS launcher required')
        offset += length
    require(offset == 32 + size and libraries == [b'/usr/lib/libSystem.B.dylib']
            and minimum == 0x000f0000, 'launcher must use only libSystem and deployment 15.0')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--app', type=Path, required=True)
    parser.add_argument('--launcher', type=Path, required=True)
    args = parser.parse_args()
    require(os.getuid() != 0 and os.getuid() == os.geteuid(), 'non-root user required')
    app, launcher = args.app, args.launcher
    owned(app, stat.S_ISDIR)
    require(app.name == 'Openergy.app' and app.parent.name == 'target', 'private target path')
    root = app.parent.parent
    owned(root, stat.S_ISDIR, 0o700)
    owned(app.parent, stat.S_ISDIR, 0o700)
    suffix = root.name.removeprefix('oe-updater-fixture-')
    require(root.name == 'oe-updater-fixture-' + suffix and re.fullmatch(UUID, suffix), 'root UUID')
    require(re.fullmatch(r'/[A-Za-z0-9/_\-.]+', str(root)) and len(str(root).encode()) < 512,
            'root characters or size')
    # This preparer selects the canonical shared-temp option of the launch
    # contract. Python does not expose Darwin's per-user confstr name here.
    require(root.parent == Path('/private/tmp'), '/private/tmp root required')
    require(not (app / 'Contents/_CodeSignature').exists(), 'prepare before bundle signing')
    plist_path = app / 'Contents/Info.plist'
    owned(plist_path, stat.S_ISREG)
    require(plist_path.stat().st_size <= 8192, 'plist bound')
    info = plistlib.loads(plist_path.read_bytes())
    require(isinstance(info, dict), 'plist dictionary required')
    identifier = PREFIX + suffix
    metadata = info.get('OpenergyPrivateUpdateFixture')
    require(info.get('CFBundleIdentifier') == identifier and isinstance(metadata, dict)
            and set(metadata) == {'Schema', 'Root', 'Nonce', 'Port'}, 'existing private metadata required')
    require(type(metadata['Schema']) is int and metadata['Schema'] == 1
            and metadata['Root'] == str(root), 'metadata schema or root')
    nonce, port, build = metadata['Nonce'], metadata['Port'], info.get('CFBundleVersion')
    require(isinstance(nonce, str) and re.fullmatch(UUID, nonce) and nonce != suffix, 'nonce')
    require(type(port) is int and 25000 <= port <= 65535, 'port')
    require(isinstance(build, str) and re.fullmatch(r'[1-9][0-9]{0,8}', build), 'build')
    require(info.get('CFBundleExecutable') == 'Openergy', 'unprepared private image required')
    public_key = info.get('SUPublicEDKey')
    require(isinstance(public_key, str) and re.fullmatch(r'[A-Za-z0-9+/]{42}[AEIMQUYcgkosw048]=', public_key),
            'existing canonical fixture public key required')
    require(len(base64.b64decode(public_key, validate=True)) == 32, '32-byte fixture public key required')
    owned(app / 'Contents', stat.S_ISDIR)
    owned(app / 'Contents/MacOS', stat.S_ISDIR)
    owned(app / 'Contents/MacOS/Openergy', stat.S_ISREG)
    libsystem_launcher(launcher)
    template = Path(__file__).with_name('Info.plist.in').read_text(encoding='ascii')
    values = {'FIXTURE_ID': identifier, 'ROOT': str(root), 'NONCE': nonce, 'PORT': str(port), 'BUILD': build,
              'PUBLIC_ED_KEY': public_key}
    require(set(re.findall(r'@([A-Z_]+)@', template)) == set(values), 'template fields')
    for name, value in values.items():
        require(template.count('@' + name + '@') == 1, 'repeated template field')
        template = template.replace('@' + name + '@', value)
    require('@' not in template and len(template.encode('ascii')) <= 8192, 'rendered plist')
    parsed = plistlib.loads(template.encode('ascii'))
    require(parsed['OpenergyPrivateUpdateFixture'] == metadata, 'rendered metadata mismatch')
    # All input checks precede output. Exclusive files preserve partial failures.
    destination = app / 'Contents/MacOS/OpenergyPrivateLauncher'
    with launcher.open('rb') as source, destination.open('xb') as target:
        os.fchmod(target.fileno(), 0o700)
        shutil.copyfileobj(source, target)
        target.flush()
        os.fsync(target.fileno())
    temporary = app / 'Contents/Info.private-entry.pending'
    with temporary.open('xb') as target:
        os.fchmod(target.fileno(), 0o600)
        target.write(template.encode('ascii'))
        target.flush()
        os.fsync(target.fileno())
    os.replace(temporary, plist_path)
    print(json.dumps({'app': str(app), 'entry': str(destination), 'scope': 'unsigned packaging only'}))


if __name__ == '__main__':
    main()
