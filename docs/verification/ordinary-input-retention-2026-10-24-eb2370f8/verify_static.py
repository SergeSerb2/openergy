"""Own final file/Git/hash/JSON/AST check; never loads supplied modules."""
import ast
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import sys

HERE = Path(__file__).absolute().parent
REPO = HERE.parents[2]
used = 0


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read(path):
    global used
    for parent in reversed(path.parents):
        assert stat.S_ISDIR(parent.lstat().st_mode), "ancestor symlink; stop"
    info = path.lstat()
    assert stat.S_ISREG(info.st_mode)
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW)
    with os.fdopen(fd, "rb") as stream:
        opened = os.fstat(stream.fileno())
        assert (info.st_dev, info.st_ino) == (opened.st_dev, opened.st_ino)
        data = stream.read(info.st_size + 1)
        after = os.fstat(stream.fileno())
    assert len(data) == info.st_size
    assert (info.st_size, info.st_mtime_ns, info.st_ctime_ns) == (
        after.st_size, after.st_mtime_ns, after.st_ctime_ns)
    used += len(data)
    assert used < 64 * 1024 * 1024
    return data


def git(*args, input_bytes=None):
    global used
    assert args[0] in ["cat-file", "ls-tree", "status", "diff"]
    result = subprocess.run(["git", "--no-optional-locks", *args], cwd=REPO,
                            input=input_bytes, capture_output=True, check=True)
    used += len(result.stdout) + len(result.stderr)
    return result.stdout


def protected(relative, commit):
    listing = git("ls-tree", "-rz", commit, "--", relative)
    rows = []
    for entry in listing.split(b"\0"):
        if entry:
            header, name = entry.split(b"\t", 1)
            mode, kind, blob = header.decode().split()
            assert kind == "blob" and mode in ["100644", "100755"]
            rows.append((name.decode(), blob))
    stream = git("cat-file", "--batch", input_bytes=b"".join(x[1].encode()+b"\n" for x in rows))
    offset = 0
    for name, blob in rows:
        end = stream.index(b"\n", offset)
        header = stream[offset:end].decode().split()
        assert header[:2] == [blob, "blob"]
        count = int(header[2])
        historical = stream[end+1:end+1+count]
        assert read(REPO/name) == historical
        offset = end+2+count
    assert offset == len(stream)
    return len(rows)


def main(private):
    summary = json.loads(read(HERE/"results.json"))
    supplementary = json.loads(read(HERE/"supplemental-bindings.json"))
    raw_receipt = read(private/"copy-receipt.json")
    assert sha(raw_receipt) == summary["private_copy_receipt_sha256"]
    receipt = json.loads(raw_receipt)
    raw_journal = read(private/"audit-journal.json")
    assert sha(raw_journal) == receipt["journal_sha256"] == summary["private_journal_sha256"]
    journal = json.loads(raw_journal)
    assert receipt["copy_rows"] == summary["copies"]
    assert len(summary["copies"]) == 21
    for row in summary["copies"]:
        relative = Path(row["retained_name"])
        assert not relative.is_absolute() and ".." not in relative.parts
        content = read(private/relative)
        assert sha(content) == row["before_sha256"] == row["copied_sha256"]
        assert len(content) == row["bytes"]
    assert sum(row["bytes"] for row in summary["copies"]) == 4850035
    daemon_path = next(Path(row["path"]) for row in journal["plan"] if row["label"] == "compiled-daemon")
    assert sha(read(daemon_path)) == "23e3f9dbee1496b247a5b316811f22c9330b103f979907d1d4fadf2e57399deb"
    assert summary["source_locations"] == len(journal["plan"]) == 21
    assert len({row["path"] for row in journal["plan"]}) == 21
    for row in summary["locations"]:
        source = next(p for p in journal["plan"] if p["label"] == row["location_id"])
        assert sha(source["path"].encode()) == row["path_binding_sha256"]
    assert all(row["archive_match"] and row["original_source_match"] for row in summary["receipts"])
    assert all(row["ordinary_source_match"] for row in summary["source_binding_results"])
    assert [row["path"] for row in summary["source_binding_results"] if row.get("compiled_source_match") is False] == [
        "apps/macos/scripts/bundle.sh", "apps/macos/scripts/bundle.sh"]
    assert all(row.get("match",True) for row in supplementary["checks"])
    protected_counts = {
        "original":protected("docs/verification/ordinary-app-e2e-2026-10-23-t3-79498fe8", summary["original_commit"]),
        "R7":protected("docs/verification/ordinary-controller-repair-r7-2026-10-23-t3-ad9d06e1", "d56c65eb8da5f3d8b4fd58fc0e933360754ba6d6"),
        "R8":protected("docs/verification/ordinary-controller-R8-review-2026-10-23-t3-f51c60d1.md", "172ee851d99bf1c256235b23fb445cae899332ff")}
    assert protected_counts == {"original":87,"R7":16,"R8":1}
    # No tracked changes outside the new owned documentation folder.
    assert not git("diff", "--name-only", "HEAD", "--", ".", ":(exclude)"+str(HERE.relative_to(REPO))).strip()
    status = git("status", "--porcelain=v1", "-z", "--untracked-files=all")
    prefix = str(HERE.relative_to(REPO))+"/"
    for row in status.split(b"\0"):
        if row:
            assert row[3:].decode().startswith(prefix)

    public_rows = []
    for path in sorted(HERE.iterdir()):
        assert path.is_file() and not path.is_symlink()
        content = read(path)
        # Generic system alias literals in the reader are not private paths.
        assert re.search(rb"/Users/[^ /\"'\n]+/", content) is None
        assert re.search(rb"/private/(?:tmp|var)/[^ /\"'\n]+", content) is None
        if path.suffix == ".json":
            json.loads(content)
        if path.suffix == ".py":
            ast.parse(content, filename=path.name)
        public_rows.append({"path":path.name,"bytes":len(content),"sha256":sha(content)})

    private_rows = []

    def inventory(directory):
        assert stat.S_IMODE(directory.lstat().st_mode) == 0o700
        with os.scandir(directory) as entries:
            children = sorted(entries,key=lambda x:x.name)
        assert len(children) <= 2000
        for child in children:
            assert len(private_rows) < 2000
            path = Path(child.path)
            info = path.lstat()
            row = {"path":str(path.relative_to(private)),"mode":oct(stat.S_IMODE(info.st_mode))}
            private_rows.append(row)
            if stat.S_ISDIR(info.st_mode):
                row["directory"] = True
                inventory(path)
            else:
                assert stat.S_ISREG(info.st_mode) and stat.S_IMODE(info.st_mode) == 0o600
                content = read(path)
                row.update(bytes=len(content),sha256=sha(content))

    inventory(private)
    completed = {"kind":"completed private folder inventory, excludes this inventory file itself",
                 "entries":private_rows,"symlinks":0,"nonregular_files":0}
    inventory_bytes = (json.dumps(completed,indent=2,sort_keys=True)+"\n").encode()
    fd = os.open(private/"final-inventory.json",os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    with os.fdopen(fd,"wb") as stream:
        stream.write(inventory_bytes)
    assert read(private/"final-inventory.json") == inventory_bytes
    failed_attempt_allowance = 64 * 1024 * 1024
    total = (summary["conservative_read_total_bytes"] + supplementary["supplemental_content_bytes_read"]
             + failed_attempt_allowance + used)
    assert total <= summary["read_limit_bytes"]
    result = {"kind":"STATIC FILE/GIT/HASH/JSON/AST ONLY; no runtime or source approval",
              "protected_files_match":protected_counts,
              "all_private_copies_match":True,"private_data_files":21,
              "private_input_bytes":4850035,"private_files_including_metadata":len([r for r in private_rows if not r.get("directory")])+1,
              "private_symlinks":0,"private_directories_mode":"0700","private_files_mode":"0600",
              "private_copy_receipt_sha256":summary["private_copy_receipt_sha256"],
              "final_private_inventory_sha256":sha(inventory_bytes),
              "final_verification_content_bytes_read":used,
              "conservative_total_content_bytes_read":total,
              "read_limit_bytes":summary["read_limit_bytes"],
              "preliminary_exploration_allowance_bytes":summary["preliminary_read_allowance_bytes"],
              "failed_own_verifier_attempt_allowance_bytes":failed_attempt_allowance,
              "failed_own_verifier_attempt":"Path scan matched its own literal; replaced literal patterns with general path patterns. Evidence predicates unchanged. Attempt retained as a conservative 64 MiB read charge.",
              "exact_top_level_source_locations":21,"source_location_limit":64,
              "artifact_entry_limit":2000,"final_private_inventory_entries":len(private_rows)+1,
              "public_files_before_validation_record":public_rows,
              "public_private_path_scan":"PASS","outside_owned_docs_diff":"EMPTY",
              "acceptance":"UNVERIFIED; original FAILED / NOT ACCEPTED, 23/25"}
    fd = os.open(HERE/"validation.json",os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
    with os.fdopen(fd,"w") as stream:
        json.dump(result,stream,indent=2,sort_keys=True)
        stream.write("\n")
    print(json.dumps({k:result[k] for k in ["protected_files_match","private_files_including_metadata",
          "conservative_total_content_bytes_read","private_copy_receipt_sha256","public_private_path_scan"]},sort_keys=True))


if __name__ == "__main__":
    assert len(sys.argv) == 2
    main(Path(sys.argv[1]).absolute())
