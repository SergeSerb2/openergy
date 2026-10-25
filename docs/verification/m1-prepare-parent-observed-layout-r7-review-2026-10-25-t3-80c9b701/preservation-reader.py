"""Reviewer-owned byte/mode/index DATA comparison; no supplied source entry.

The default pass records closing evidence in this folder. --read-only performs
the same per-entry checks without changing evidence, including after commit.
"""
import hashlib
import json
import os
from pathlib import Path
import stat
import subprocess
import sys

ROOT = Path.cwd()
OWN = 'docs/verification/m1-prepare-parent-observed-layout-r7-review-2026-10-25-t3-80c9b701'
P = ROOT / OWN / 'preservation.json'
ENV = dict(os.environ, GIT_OPTIONAL_LOCKS='0')


def git(*args):
    return subprocess.check_output(['git', *args], env=ENV)


def sha(b):
    return hashlib.sha256(b).hexdigest()


def enc(v):
    return (json.dumps(v, sort_keys=True, separators=(',', ':'), allow_nan=False) + '\n').encode()


def index_rows(raw):
    out = {}
    for row in raw.rstrip(b'\0').split(b'\0'):
        metadata, path = row.split(b'\t')
        mode, oid, stage = metadata.decode().split()
        out.setdefault(path.decode(), []).append(dict(mode=mode, oid=oid, stage=stage))
    return out


def actual(p):
    info = p.lstat()
    if stat.S_ISREG(info.st_mode):
        kind, b = 'file', p.read_bytes()
    elif stat.S_ISLNK(info.st_mode):
        kind, b = 'symlink', os.readlink(p).encode()
    elif stat.S_ISDIR(info.st_mode):
        kind, b = 'directory', b''
    else:
        raise ValueError(str(p))
    return dict(path=p.relative_to(ROOT).as_posix(), kind=kind,
                mode=format(stat.S_IMODE(info.st_mode), '04o'), bytes=len(b),
                sha256=sha(b) if kind != 'directory' else None)


def main():
    opening = json.loads(P.read_bytes())
    entries = opening['entries']
    current_index = git('ls-files', '--stage', '-z')
    index = index_rows(current_index)
    if 'index_stage_nul_hex' in opening:
        original_index = index_rows(bytes.fromhex(opening.pop('index_stage_nul_hex')))
        for row in entries:
            row['index'] = original_index.get(row['path'])
    else:
        original_index = {r['path']:r['index'] for r in entries if r['index'] is not None}
    observed = []
    for row in entries:
        baseline = {k:row[k] for k in ('path','kind','mode','bytes','sha256')}
        current = actual(ROOT / row['path'])
        if current != baseline:
            raise ValueError('working bytes/mode/type changed: ' + row['path'])
        if index.get(row['path']) != row['index']:
            raise ValueError('old index entry changed: ' + row['path'])
        row['closing_equal'] = True
        observed.append(current)
    actual_paths = set()
    for dp, directories, files in os.walk(ROOT, followlinks=False):
        directories.sort()
        files.sort()
        if Path(dp) == ROOT and '.git' in directories:
            directories.remove('.git')
        for name in directories + files:
            rel = (Path(dp) / name).relative_to(ROOT).as_posix()
            if rel != OWN and not rel.startswith(OWN + '/'):
                actual_paths.add(rel)
        directories[:] = [name for name in directories
                          if (Path(dp) / name).relative_to(ROOT).as_posix() != OWN]
    if actual_paths != {r['path'] for r in entries}:
        raise ValueError('old workspace path set changed outside owned folder')
    projected = b''.join(row + b'\0' for row in current_index.rstrip(b'\0').split(b'\0')
                         if not row.split(b'\t',1)[1].decode().startswith(OWN + '/'))
    if sha(projected) != opening['index_stage_sha256']:
        raise ValueError('old full logical index projection changed')
    if {k:v for k,v in index.items() if not k.startswith(OWN + '/')} != original_index:
        raise ValueError('old index set changed')
    digest = sha(enc(observed))
    original_digest = sha(json.dumps(observed,sort_keys=True,separators=(',',':')).encode())
    summary = dict(execution=False, individually_checked=len(entries),
                   old_tracked_paths=len(original_index), old_index_entries=sum(map(len,original_index.values())),
                   files=sum(r['kind']=='file' for r in entries),
                   directories=sum(r['kind']=='directory' for r in entries),
                   symlinks=sum(r['kind']=='symlink' for r in entries),
                   working_bytes=sum(r['bytes'] for r in entries),
                   canonical_inventory_JSON_LF_sha256=digest,
                   opening_inventory_JSON_noLF_sha256=original_digest,
                   old_index_projection_sha256=sha(projected),
                   old_path_mutations=[], per_entry_bytes_and_full_modes_preserved=True,
                   per_entry_index_identities_preserved=True,
                   extra_paths_outside_owned=[],
                   original_tree=git('rev-parse',opening['head'] + '^{tree}').decode().strip())
    if '--read-only' not in sys.argv:
        index_path = Path(git('rev-parse','--git-path','index').decode().strip())
        if sha(index_path.read_bytes()) != opening['index_bytes_sha256']:
            raise ValueError('pre-staging whole index bytes changed')
        opening['closing'] = summary
        opening['schema'] = 'openergy.r7-independent-per-entry-preservation.v1'
        opening['inventory_encoding'] = 'One inventory only. Sorted path/kind/mode/bytes/sha256 rows, compact sorted-key JSON plus LF; index fields and closing_equal excluded from working digest.'
        opening['original_index_bytes_exact_before_staging'] = True
        opening['post_commit_observation'] = None
        P.write_bytes(enc(opening))
    print(json.dumps(summary,indent=2))


if __name__ == '__main__':
    main()
