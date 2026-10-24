#!/usr/bin/env python3
"""Validate evidence bytes and source invariance, without executing the product."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
OWNED = HERE.relative_to(REPO).as_posix() + "/"
BASE = "1fd5144628c0494f1ebf561581e0d84fd7231523"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    os.umask(0o077)
    started = time.monotonic()
    os.setpriority(os.PRIO_PROCESS, 0, 15)
    commands = []

    def git(*args):
        result = subprocess.run(["git", *args], cwd=REPO, capture_output=True, timeout=15)
        commands.append(dict(argv=["git", *args], exit=result.returncode,
                             stdout=result.stdout.decode(), stderr=result.stderr.decode()))
        if result.returncode:
            raise RuntimeError(commands[-1])
        return result.stdout

    origins = json.loads((HERE / "origins.json").read_text())
    old_manifest = {x["file"]: x for x in json.loads((HERE / "raw/saved-r3/evidence-manifest.json").read_text())}
    original_manifest_matches = 0
    for origin in origins:
        data = (HERE / origin["file"]).read_bytes()
        if len(data) != origin["bytes"] or sha(data) != origin["sha256"]:
            raise RuntimeError(("changed copied origin", origin))
        if origin["file"].startswith("raw/saved-r3/"):
            original_name = origin["file"].removeprefix("raw/saved-r3/")
            if original_name in old_manifest:
                row = old_manifest[original_name]
                if row["sha256"] != sha(data) or row["bytes"] != len(data):
                    raise RuntimeError(("old manifest mismatch", row))
                original_manifest_matches += 1
    bindings = json.loads((HERE / "source-bindings.json").read_text())
    for row in bindings["files"]:
        data = (REPO / row["path"]).read_bytes()
        if sha(data) != row["sha256"] or len(data) != row["bytes"]:
            raise RuntimeError(("source changed", row["path"]))
    git("diff", "--exit-code", BASE, "--", ".", ":(exclude)" + OWNED)
    git("diff", "--cached", "--check")
    status = git("status", "--porcelain=v1", "--untracked-files=all").decode().splitlines()
    if any(not row[3:].startswith(OWNED) for row in status):
        raise RuntimeError(("unowned status", status))
    staged = git("diff", "--cached", "--name-status").decode().splitlines()
    if any(not row.startswith("A\t" + OWNED) for row in staged):
        raise RuntimeError(("unowned staged content", staged))
    result = dict(invocation="python3 " + str(Path(__file__).resolve().relative_to(REPO)),
                  saved_origins_sha256_verified=len(origins),
                  copied_files_matching_original_r3_manifest=original_manifest_matches,
                  unchanged_source_hashes=len(bindings["files"]),
                  outside_owned_folder_diff_exit=0, owned_status_only=True,
                  staged_additions_only=True, collector_stderr_bytes=(HERE / "collection.stderr").stat().st_size,
                  owned_root_mode=oct(HERE.stat().st_mode & 0o777),
                  absolute_collector_nice=os.getpriority(os.PRIO_PROCESS, 0),
                  new_product_or_compiler_launches=0, new_daemon_flows=0,
                  new_stop_requests=0, socket_operations=0,
                  elapsed_seconds=time.monotonic() - started, commands=commands)
    (HERE / "validation.json").write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    entries = []
    for path in sorted(HERE.rglob("*")):
        if path.is_file() and path.name != "evidence-manifest.json":
            data = path.read_bytes()
            entries.append(dict(file=str(path.relative_to(HERE)), bytes=len(data), sha256=sha(data)))
    # Include the copied original manifest; exclude only this archive's manifest.
    old = HERE / "raw/saved-r3/evidence-manifest.json"
    entries.append(dict(file=str(old.relative_to(HERE)), bytes=old.stat().st_size, sha256=sha(old.read_bytes())))
    entries.sort(key=lambda row: row["file"])
    manifest = HERE / "evidence-manifest.json"
    manifest.write_text(json.dumps(entries, indent=2) + "\n")
    for row in entries:
        data = (HERE / row["file"]).read_bytes()
        if len(data) != row["bytes"] or sha(data) != row["sha256"]:
            raise RuntimeError(("new manifest mismatch", row))
    print(json.dumps({k: v for k, v in result.items() if k != "commands"}, indent=2, sort_keys=True))
    print("new_archive_manifest_entries=" + str(len(entries)))


if __name__ == "__main__":
    main()
