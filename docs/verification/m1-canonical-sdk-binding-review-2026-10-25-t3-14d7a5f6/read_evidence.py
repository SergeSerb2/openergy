"""Reviewer-owned bounded data reader. Never imports or executes retained code."""
import hashlib
import json
import os
import stat
from pathlib import Path

ROOT = "/Users/serge/.local/state/openergy-verification/m1-reviewed-ordinary-operation-prepared-20261025-ed66604cb676"
CANDIDATE = Path("docs/verification/m1-sdk-canonical-path-repair-2026-10-25")
REVIEW = Path(__file__).parent
MAX_OPENS = 64
MAX_BYTES = 8 * 1024 * 1024


def snapshot(names=None):
    records = json.loads((CANDIDATE / "private-evidence-index.json").read_text())["records"]
    if names is not None:
        assert len(names) == len(set(names))
        records = [r for r in records if r["path"] in names]
        assert len(records) == len(names)
    ledgerpath = REVIEW / "private-read-ledger.json"
    ledger = json.loads(ledgerpath.read_text()) if ledgerpath.exists() else {
        "regular_file_opens": 0, "content_bytes": 0}
    assert ledger["regular_file_opens"] + len(records) <= MAX_OPENS
    assert ledger["content_bytes"] + sum(r["bytes"] for r in records) <= MAX_BYTES
    rootfd = os.open("/", os.O_RDONLY | os.O_DIRECTORY)
    try:
        for part in ROOT.split("/")[1:]:
            nextfd = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=rootfd)
            os.close(rootfd)
            rootfd = nextfd
        assert len(records) <= MAX_OPENS
        assert sum(r["bytes"] for r in records) <= MAX_BYTES
        data, observed = {}, []
        consumed = 0
        for row in records:
            parts = row["path"].split("/")
            assert all(p not in ("", ".", "..") for p in parts)
            parentfd = os.dup(rootfd)
            try:
                for part in parts[:-1]:
                    nextfd = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=parentfd)
                    os.close(parentfd)
                    parentfd = nextfd
                ledger["regular_file_opens"] += 1
                ledger["content_bytes"] += row["bytes"]
                ledgerpath.write_text(json.dumps(ledger, indent=2) + "\n")
                fd = os.open(parts[-1], os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=parentfd)
            finally:
                os.close(parentfd)
            try:
                before = os.fstat(fd)
                assert stat.S_ISREG(before.st_mode)
                assert before.st_size == row["bytes"]
                assert consumed + before.st_size <= MAX_BYTES
                chunks, remaining = [], before.st_size
                while remaining:
                    chunk = os.read(fd, min(65536, remaining))
                    assert chunk
                    chunks.append(chunk)
                    remaining -= len(chunk)
                content = b"".join(chunks)
                after = os.fstat(fd)
                assert (before.st_dev, before.st_ino, before.st_mode, before.st_size,
                        before.st_mtime_ns, before.st_ctime_ns) == (
                        after.st_dev, after.st_ino, after.st_mode, after.st_size,
                        after.st_mtime_ns, after.st_ctime_ns)
                digest = hashlib.sha256(content).hexdigest()
                assert digest == row["sha256"]
                consumed += len(content)
                data[row["path"]] = content
                observed.append({**row, "regular": True, "mode": oct(stat.S_IMODE(before.st_mode))})
            finally:
                os.close(fd)
        return data, {"schema": "openergy.bounded-review-snapshot.v1",
                      "regular_file_opens": len(records), "content_bytes": consumed,
                      "symlinks_accepted": 0, "all_index_bindings_match": True,
                      "records": observed}
    finally:
        os.close(rootfd)


def safe(value):
    if isinstance(value, dict):
        return {k: ("[withheld]" if any(t in k.lower() for t in
                ("argv", "pid", "selector", "endpoint", "host", "alias")) else safe(v))
                for k, v in value.items()}
    if isinstance(value, list):
        return [safe(v) for v in value]
    return value


if __name__ == "__main__":
    data, manifest = snapshot()
    (REVIEW / "private-snapshot-initial.json").write_text(json.dumps(manifest, indent=2) + "\n")
    for name, content in data.items():
        print("FILE", name, "BYTES", len(content))
        if name.endswith((".json", ".guard")) or name == "source-files.stdout.bin":
            print(json.dumps(safe(json.loads(content)), indent=2))
        elif name == "retained-source-file-failure/source-files.stderr.bin":
            print(content.decode())
        elif name in ("source-file-phase.py.txt", "source-file-executor.py"):
            lines = content.decode().splitlines()
            print("TEXT_LINES", len(lines))
            for i, line in enumerate(lines, 1):
                if len(line) > 1000:
                    print(i, "[long data line withheld; characters=" + str(len(line)) + "]")
                elif any(s in line.lower() for s in ("argv", "ssh", "host", "alias", "selector", "pid")):
                    print(i, "[sensitive line withheld]")
                else:
                    print(i, line)
