#!/usr/bin/env python3
"""Validate reviewed Git objects and publish draft data from a trusted checkout.

Run this file from the default-branch workflow SHA, never from the release ref.
Candidate source is read with Git plumbing. It is not checked out or executed.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import stat
import subprocess
import sys

# Also works with python -I: imports come only from this trusted script directory.
sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
import release


def sha(value):
    if not re.fullmatch(r"[0-9a-f]{40}", value):
        raise ValueError("a full lowercase Git commit/object SHA is required")
    return value


def source_blob(commit, name):
    path = PurePosixPath(name)
    if (not isinstance(name, str) or path.is_absolute() or ".." in path.parts
            or path.as_posix() != name or "\\" in name or "\n" in name or ":" in name):
        raise ValueError("source metadata needs a confined repository path")
    entry = release.git("ls-tree", commit, "--", name).split("\t")
    if len(entry) != 2 or entry[1] != name:
        raise ValueError(f"source is missing metadata: {name}")
    mode, kind, object_id = entry[0].split()
    if mode not in ("100644", "100755") or kind != "blob":
        raise ValueError(f"source metadata must be a regular Git blob: {name}")
    if int(release.git("cat-file", "-s", object_id)) > 4 * 1024 * 1024:
        raise ValueError("source metadata exceeds 4 MiB")
    return release.run(["git", "cat-file", "blob", object_id], capture=True, binary=True).stdout


def ancestor(older, newer, message):
    result = release.run(["git", "merge-base", "--is-ancestor", older, newer],
                         capture=True, check=False)
    if result.returncode != 0:
        raise ValueError(message)


def resolve(args):
    if not re.fullmatch(r"v[0-9]+\.[0-9]+\.[0-9]+", args.tag):
        raise ValueError("tag must be a literal version tag such as v0.1.0; branches and expressions are refused")
    trusted = sha(args.trusted_commit)
    if release.git("rev-parse", "HEAD") != trusted or release.git("status", "--porcelain"):
        raise ValueError("use a clean trusted checkout pinned to the workflow commit")
    branch_ref = f"refs/heads/{args.default_branch}"
    if release.run(["git", "check-ref-format", branch_ref], capture=True, check=False).returncode:
        raise ValueError("invalid default branch")
    tag_ref = f"refs/tags/{args.tag}"
    remote = release.git("ls-remote", "--exit-code", "origin", branch_ref, tag_ref, tag_ref + "^{}")
    refs = {}
    for line in remote.splitlines():
        object_id, ref = line.split("\t")
        if ref not in (branch_ref, tag_ref, tag_ref + "^{}") or ref in refs:
            raise ValueError("unexpected remote ref response")
        refs[ref] = sha(object_id)
    if branch_ref not in refs or tag_ref not in refs:
        raise ValueError("remote default branch and literal release tag must exist")
    tag_object = release.git("rev-parse", "--verify", tag_ref)
    source = release.git("rev-parse", "--verify", tag_ref + "^{commit}")
    if tag_object != refs[tag_ref] or source != refs.get(tag_ref + "^{}", refs[tag_ref]):
        raise ValueError("release tag moved or differs from the fetched tag")
    if getattr(args, "source_commit", None) and source != sha(args.source_commit):
        raise ValueError("release tag moved from the frozen source commit")
    if getattr(args, "tag_object", None) and tag_object != sha(args.tag_object):
        raise ValueError("release tag object moved from the frozen validation output")
    ancestor(trusted, refs[branch_ref], "workflow commit is no longer on the remote default branch")
    ancestor(source, trusted, "release source is not a reviewed ancestor of the frozen default-branch workflow commit")
    version, toolchain = release.configuration(lambda name: source_blob(source, name))
    if args.tag != f"v{version}" or (getattr(args, "version", None) and args.version != version):
        raise ValueError("release tag and frozen version must match candidate workspace version")
    for name in release.LEGAL:
        if not source_blob(source, name).strip():
            raise ValueError(f"source is missing legal notice content: {name}")
    inventory = json.loads(source_blob(source, "tools/release/notices/dependency-licenses.json"))
    if (inventory["cargo_lock_sha256"] != hashlib.sha256(source_blob(source, "Cargo.lock")).hexdigest()
            or not inventory["packages"]):
        raise ValueError("candidate notice inventory is stale for Cargo.lock")
    return {"source_commit": source, "tag_object": tag_object, "version": version,
            "tag": args.tag, "trusted_commit": trusted, "rust": toolchain}


def validate(args):
    result = resolve(args)
    if args.output:
        args.output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", newline="\n")
    if args.github_output:
        with args.github_output.open("a") as output:
            for key, value in result.items():
                # All values have been confined to version literals or fixed Git SHAs.
                if not re.fullmatch(r"[A-Za-z0-9.]+", value):
                    raise ValueError("unsafe workflow output")
                output.write(f"{key}={value}\n")
    print(json.dumps(result, sort_keys=True))


def publish(args):
    frozen = resolve(args)
    artifacts = args.artifacts.resolve()
    # Publisher inputs are files only. Do not extract them, add them to sys.path,
    # check out their source, invoke Cargo, or execute an archived program/script.
    for path in artifacts.iterdir():
        if not stat.S_ISREG(path.lstat().st_mode):
            raise ValueError("publisher artifact directory must contain regular files only")
        if path.suffix == ".json" and path.stat().st_size > 1024 * 1024:
            raise ValueError("publisher provenance sidecar exceeds 1 MiB")
    assets = release.validate_artifacts(artifacts, frozen["version"], frozen["source_commit"], frozen["tag"])
    if len(list(artifacts.iterdir())) != len(assets):
        raise ValueError("publisher artifact directory contains unexpected files")
    # Remote refs are read again after archive checks, immediately before mutation.
    if resolve(args) != frozen:
        raise ValueError("release provenance changed during validation")
    if args.check_only:
        print("PASS: frozen source, remote tag, all four targets, checksums, and embedded manifests; no mutation")
        return
    release.create_draft(frozen["tag"], frozen["version"], frozen["source_commit"], assets)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    for name in ("validate", "publish"):
        command = commands.add_parser(name)
        command.add_argument("--tag", required=True)
        command.add_argument("--trusted-commit", required=True)
        command.add_argument("--default-branch", required=True)
        if name == "validate":
            command.add_argument("--output", type=Path)
            command.add_argument("--github-output", type=Path)
        else:
            command.add_argument("--source-commit", required=True)
            command.add_argument("--tag-object", required=True)
            command.add_argument("--version", required=True)
            command.add_argument("--artifacts", type=Path, required=True)
            command.add_argument("--check-only", action="store_true")
    args = parser.parse_args()
    {"validate": validate, "publish": publish}[args.command](args)


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, KeyError, TypeError, UnicodeError, subprocess.SubprocessError) as error:
        sys.exit(f"trusted release: {error}")
