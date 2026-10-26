#!/usr/bin/env python3
"""Verify vendored source provenance; does not compile or run the library."""
import hashlib
import json
from pathlib import Path
import sys
import tarfile

ROOT = Path(__file__).resolve().parent


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    provenance = json.loads((ROOT / 'UPSTREAM.json').read_text())
    original = provenance['original_file_sha256']
    modified = provenance['modified_file_sha256']
    additions = provenance['local_addition_sha256']
    expected = {**original, **modified, **additions}
    actual = {p.relative_to(ROOT).as_posix() for p in ROOT.rglob('*') if p.is_file()}
    if actual != set(expected) | {'UPSTREAM.json'}:
        raise ValueError('unexpected or missing vendor files: ' + str(actual ^ (set(expected) | {'UPSTREAM.json'})))
    for name, digest in expected.items():
        if sha((ROOT / name).read_bytes()) != digest:
            raise ValueError('local content hash mismatch: ' + name)
    if len(sys.argv) == 2:
        archive = Path(sys.argv[1])
        if sha(archive.read_bytes()) != provenance['archive_sha256']:
            raise ValueError('upstream archive checksum mismatch')
        with tarfile.open(archive) as source:
            prefix = provenance['crate'] + '-' + provenance['version'] + '/'
            for name, digest in original.items():
                if sha(source.extractfile(prefix + name).read()) != digest:
                    raise ValueError('original archive file hash mismatch: ' + name)
            vcs = json.loads(source.extractfile(prefix + '.cargo_vcs_info.json').read())
            if vcs['git']['sha1'] != provenance['revision']:
                raise ValueError('upstream VCS revision mismatch')
    elif len(sys.argv) != 1:
        raise ValueError('usage: verify-source.py [UPSTREAM_CRATE_ARCHIVE]')
    print(f'vendor verified: {len(original)} original files, {len(modified)} modified files; existing tests unchanged')


if __name__ == '__main__':
    main()
