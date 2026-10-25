#!/usr/bin/env python3
"""Retain immutable local build inputs as data. No build or remote command."""
import base64
import hashlib
import io
import json
import os
from pathlib import Path
import stat
import subprocess
import tarfile
import time
import tomllib
import uuid

os.setpriority(os.PRIO_PROCESS, 0, 15)
assert os.getpriority(os.PRIO_PROCESS, 0) == 15
started = time.monotonic()
work = Path.cwd()
assert subprocess.check_output(['git', 'status', '--porcelain']).strip() == b''
commit = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
assert commit == 'ed7d96bf53da1a539fa2acde3b7133613221fd4b'
state_base = Path('/Users/serge/.local/state/openergy-verification')
assert state_base.is_dir() and not state_base.is_symlink()
assert state_base.stat().st_uid == os.getuid()
assert stat.S_IMODE(state_base.stat().st_mode) & 0o022 == 0
parent = state_base / 'm1-resume-private-20261024'
parent.mkdir(mode=0o700)
assert stat.S_IMODE(parent.stat().st_mode) == 0o700
out = parent / ('m1-build-inputs-2026-10-24-' + uuid.uuid4().hex[:12])
out.mkdir(mode=0o700)
print(json.dumps({'private_directory_label': str(out.relative_to(state_base)), 'actual_ni': 15}), flush=True)

def check_time():
    if time.monotonic() - started > 180:
        raise RuntimeError('180-second data preparation ceiling exceeded')

def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()

def read_stable(path, limit):
    before = path.lstat()
    if not stat.S_ISREG(before.st_mode) or before.st_size > limit:
        raise RuntimeError('nonregular or overlimit input: ' + str(path))
    data = path.read_bytes()
    after = path.lstat()
    fields = lambda s: (s.st_dev, s.st_ino, s.st_size, s.st_mtime_ns)
    if fields(before) != fields(after) or len(data) != before.st_size:
        raise RuntimeError('input changed during read: ' + str(path))
    check_time()
    return data, stat.S_IMODE(before.st_mode)

def add_bytes(archive, name, data, mode):
    info = tarfile.TarInfo(name)
    info.size = len(data)
    info.mode = mode
    info.mtime = 0
    archive.addfile(info, io.BytesIO(data))

receipt = {'schema': 'openergy-m1-build-data-v1', 'source_commit': commit,
           'new_build': False, 'remote_execution': False, 'actual_ni': 15,
           'old_build_databases_copied': False, 'archives': {}, 'inputs': {}}
try:
    tool = Path('/Users/serge/.rustup/toolchains/1.99.1-aarch64-apple-darwin')
    channel = tomllib.loads((tool / 'lib/rustlib/multirust-channel-manifest.toml').read_text())
    assert channel['pkg']['rust']['version'].startswith('1.99.1 ')
    tool_rows = []
    total = 0
    archive_path = out / 'rust-toolchain.tar.gz'
    with tarfile.open(archive_path, 'x:gz', compresslevel=1) as archive:
        for path in sorted(tool.rglob('*')):
            if len(tool_rows) >= 12000:
                raise RuntimeError('toolchain record ceiling exceeded')
            rel = path.relative_to(tool).as_posix()
            st = path.lstat()
            if stat.S_ISDIR(st.st_mode):
                continue
            if stat.S_ISLNK(st.st_mode):
                link = os.readlink(path)
                resolved = path.resolve(strict=True)
                resolved.relative_to(tool)
                if link.startswith('/'):
                    raise RuntimeError('absolute toolchain link')
                info = tarfile.TarInfo('rust/' + rel)
                info.type = tarfile.SYMTYPE
                info.linkname = link
                info.mode = 0o777
                archive.addfile(info)
                tool_rows.append({'path': rel, 'link': link})
            else:
                data, mode = read_stable(path, 256 * 1024 * 1024)
                total += len(data)
                if total > 1024 * 1024 * 1024:
                    raise RuntimeError('toolchain byte ceiling exceeded')
                add_bytes(archive, 'rust/' + rel, data, mode)
                tool_rows.append({'path': rel, 'bytes': len(data), 'mode': oct(mode),
                                  'sha256': hashlib.sha256(data).hexdigest()})
    archive_path.chmod(0o600)
    receipt['inputs']['rust'] = {'version': channel['pkg']['rust']['version'],
                                'records': tool_rows, 'regular_bytes': total}
    receipt['archives']['rust-toolchain.tar.gz'] = {'bytes': archive_path.stat().st_size, 'sha256': sha(archive_path)}
    print(json.dumps({'rust_records': len(tool_rows), 'rust_regular_bytes': total}), flush=True)

    lock = tomllib.loads((work / 'Cargo.lock').read_text())
    packages = [p for p in lock['package'] if p.get('source', '').startswith('registry+')]
    assert all(p['source'] == 'registry+https://github.com/rust-lang/crates.io-index' for p in packages)
    registry = Path('/Users/serge/.cargo/registry')
    cargo_files = {}
    cargo_rows = []
    for package in packages:
        name, version = package['name'], package['version']
        candidates = list((registry / 'cache').glob('*/' + name + '-' + version + '.crate'))
        if len(candidates) != 1:
            raise RuntimeError('crate archive missing or ambiguous: ' + name)
        path = candidates[0]
        data, mode = read_stable(path, 64 * 1024 * 1024)
        digest = hashlib.sha256(data).hexdigest()
        assert digest == package['checksum'], name
        cargo_files['cargo-home/registry/cache/' + path.parent.name + '/' + path.name] = data
        lower = name.lower()
        index_rel = ('1/' + lower if len(lower) == 1 else
                     '2/' + lower if len(lower) == 2 else
                     '3/' + lower[0] + '/' + lower if len(lower) == 3 else
                     lower[:2] + '/' + lower[2:4] + '/' + lower)
        index_root = registry / 'index' / path.parent.name
        index_path = index_root / '.cache' / index_rel
        index_bytes, _ = read_stable(index_path, 32 * 1024 * 1024)
        cargo_files['cargo-home/registry/index/' + path.parent.name + '/.cache/' + index_rel] = index_bytes
        config_bytes, _ = read_stable(index_root / 'config.json', 16384)
        cargo_files['cargo-home/registry/index/' + path.parent.name + '/config.json'] = config_bytes
        cargo_rows.append({'name': name, 'version': version, 'crate_bytes': len(data),
                           'checksum': digest, 'index_sha256': hashlib.sha256(index_bytes).hexdigest()})
    if sum(map(len, cargo_files.values())) > 128 * 1024 * 1024:
        raise RuntimeError('cargo seed byte ceiling exceeded')
    archive_path = out / 'cargo-seed.tar.gz'
    with tarfile.open(archive_path, 'x:gz', compresslevel=1) as archive:
        for name, data in sorted(cargo_files.items()):
            add_bytes(archive, name, data, 0o600)
    archive_path.chmod(0o600)
    receipt['inputs']['cargo'] = {'packages': cargo_rows,
                                 'files': [{'path': name, 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()}
                                           for name, data in sorted(cargo_files.items())]}
    receipt['archives']['cargo-seed.tar.gz'] = {'bytes': archive_path.stat().st_size, 'sha256': sha(archive_path)}
    print(json.dumps({'cargo_packages': len(packages), 'cargo_seed_files': len(cargo_files)}), flush=True)

    selected = ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'crates', 'apps/macos',
                'tools/release', 'LICENSE-MIT', 'LICENSE-APACHE', 'THIRD_PARTY_NOTICES.md']
    source_path = out / 'source.tar'
    with source_path.open('xb') as stream:
        proc = subprocess.run(['git', '-c', 'core.hooksPath=/dev/null', 'archive', '--format=tar',
                               commit, '--', *selected], stdout=stream, stderr=subprocess.PIPE, timeout=30)
    if proc.returncode:
        raise RuntimeError(proc.stderr.decode(errors='replace'))
    source_path.chmod(0o600)
    if source_path.stat().st_size > 128 * 1024 * 1024:
        raise RuntimeError('source archive ceiling exceeded')
    source_rows = []
    with tarfile.open(source_path, 'r:') as archive:
        for member in archive:
            if member.isfile():
                data = archive.extractfile(member).read()
                source_rows.append({'path': member.name, 'bytes': len(data),
                                    'sha256': hashlib.sha256(data).hexdigest()})
    receipt['inputs']['source'] = {'selection': selected, 'files': source_rows,
                                 'git_archive': True, 'complete_repository': False}
    receipt['archives']['source.tar'] = {'bytes': source_path.stat().st_size, 'sha256': sha(source_path)}
    receipt['elapsed_seconds'] = time.monotonic() - started
    receipt['status'] = 'DATA_PREPARED_NO_BUILD'
except BaseException as exc:
    receipt['status'] = 'FAILED'
    receipt['first_error'] = repr(exc)
    receipt['elapsed_seconds'] = time.monotonic() - started
    raise
finally:
    path = out / 'receipt.json'
    path.write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    path.chmod(0o600)
    pointer = work / 'target/m1-connection-recovery-2026-10-24/build-data-pointer.json'
    pointer.write_text(json.dumps({'private_directory_label': str(out.relative_to(state_base)),
                                   'receipt_sha256': sha(path), 'status': receipt['status']}, indent=2) + '\n')
    pointer.chmod(0o600)
    print(json.dumps({'status': receipt['status'], 'elapsed_seconds': receipt['elapsed_seconds'],
                      'receipt_sha256': sha(path), 'archives': receipt['archives']}), flush=True)
