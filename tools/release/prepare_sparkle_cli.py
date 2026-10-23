#!/usr/bin/env python3
"""Sparkle 2.10.0 source/build preflight. Never launch products or use signing keys.

prepare exports the pinned Git object from an existing read-only repository.
Inspect source-review.json and its scripts before preflight. No network fetch is used.
preflight records the build recipe, then exits BLOCKED without invoking Xcode.
Xcode 27.1 was observed to register applications and execution-policy exceptions
under these settings. It has no supported -dry-run flag. Do not repeat that build
on the constrained host. All paths belong to a new mode-0700 /private/tmp directory.
"""

import argparse
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import xml.etree.ElementTree as ET


REVISION = "eef1a539a373c1f1a320624b1130fc5de7b2e100"
PREFIX = "oe-sparkle-cli-build-"
TARGET = "72D9549D1CBB415B006F28BD"
DEVELOPER = Path("/Applications/Xcode.app/Contents/Developer")


def sha(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def write(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def stamp():
    return dt.datetime.now().astimezone().isoformat(timespec="microseconds")


def inventory(path):
    return {str(p.relative_to(path)): {"symlink": os.readlink(p)} if p.is_symlink()
            else {"sha256": sha(p), "bytes": p.stat().st_size}
            for p in sorted(path.rglob("*")) if p.is_symlink() or p.is_file()}


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True).encode()).hexdigest()


def git(repo, *args):
    return subprocess.check_output(
        ["/usr/bin/git", "--no-optional-locks", "-c", "core.hooksPath=/dev/null",
         "-C", str(repo), *args],
        env={**os.environ, "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": "/dev/null"})


def review(source):
    project = json.loads(subprocess.check_output([
        "/usr/bin/plutil", "-convert", "json", "-o", "-",
        str(source / "Sparkle.xcodeproj/project.pbxproj")]))
    objects = project["objects"]
    targets = {}

    def visit(identifier):
        target = objects[identifier]
        name = target["name"]
        if name in targets:
            return
        if "test" in name.lower() or "test" in target.get("productType", "").lower():
            raise ValueError("Test target in build closure: " + name)
        targets[name] = {"product_type": target.get("productType"), "phases": [
            objects[p] for p in target.get("buildPhases", [])]}
        for dependency in target.get("dependencies", []):
            visit(objects[dependency]["target"])

    visit(TARGET)
    scheme = ET.parse(source / "Sparkle.xcodeproj/xcshareddata/xcschemes/sparkle-cli.xcscheme")
    if scheme.findall(".//PreActions") or scheme.findall(".//PostActions"):
        raise ValueError("Unexpected scheme actions")
    return {"targets": targets, "configuration_files": {
        str(p.relative_to(source)): {"sha256": sha(p), "text": p.read_text()}
        for p in sorted((source / "Configurations").iterdir()) if p.is_file()},
        "package_references": [v for v in objects.values()
                               if v.get("isa") == "XCRemoteSwiftPackageReference"],
        "scheme": ET.tostring(scheme.getroot(), encoding="unicode")}


def prepare(repo):
    if git(repo, "rev-parse", REVISION + "^{commit}").decode().strip() != REVISION:
        raise ValueError("Pinned Sparkle commit is unavailable")
    root = Path(tempfile.mkdtemp(prefix=PREFIX, dir="/private/tmp"))
    os.chmod(root, 0o700)
    archive = root / "official-source.tar"
    with archive.open("wb") as output:
        subprocess.run(["/usr/bin/git", "--no-optional-locks", "-c", "core.hooksPath=/dev/null",
                        "-C", str(repo), "archive", "--format=tar", REVISION],
                       stdout=output, check=True,
                       env={**os.environ, "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": "/dev/null"})
    source = root / "source"
    source.mkdir()
    with tarfile.open(archive) as stream:
        stream.extractall(source, filter="data")
    files = inventory(source)
    write(root / "source-files.json", files)
    write(root / "source-review.json", review(source))
    record = {"created_at": stamp(), "source_repository_read_only": str(repo.resolve()),
              "official_repository": "https://github.com/sparkle-project/Sparkle",
              "revision": REVISION, "tree": git(repo, "rev-parse", REVISION + "^{tree}").decode().strip(),
              "archive_sha256": sha(archive), "source_inventory_sha256": digest(files),
              "controller_sha256": sha(Path(__file__)), "root": str(root)}
    write(root / "preparation.json", record)
    print(json.dumps(record, indent=2))


def preflight(root, reviewed):
    root = root.resolve(strict=True)
    if root.parent != Path("/private/tmp") or not root.name.startswith(PREFIX):
        raise ValueError("Use only a private root created by prepare")
    if root.stat().st_uid != os.getuid() or root.stat().st_mode & 0o077:
        raise ValueError("Private root must be owned and mode 0700")
    prepared = json.loads((root / "preparation.json").read_text())
    if prepared["revision"] != REVISION or reviewed != prepared["source_inventory_sha256"]:
        raise ValueError("Review digest does not match pinned preparation")
    if digest(inventory(root / "source")) != reviewed:
        raise ValueError("Source changed after review")
    if any((root / name).exists() for name in ("build-settings.stdout", "build.stdout", "preflight-result.json")):
        raise ValueError("Use a fresh preparation; existing evidence must remain unchanged")
    for folder in ("tmp", "DerivedData", "products", "objects", "dst", "packages", "cache"):
        (root / folder).mkdir(mode=0o700, exist_ok=True)
    # Do not inherit signing, injection, compiler-wrapper, or user build overrides.
    env = {"PATH": "/usr/bin:/bin:/usr/sbin:/sbin", "LANG": "en_US.UTF-8",
           "DEVELOPER_DIR": str(DEVELOPER), "TMPDIR": str(root / "tmp") + "/",
           "CFFIXED_USER_HOME": str(root / "cache"), "XDG_CACHE_HOME": str(root / "cache"),
           "CLANG_MODULE_CACHE_PATH": str(root / "cache/clang"),
           "SWIFT_MODULECACHE_PATH": str(root / "cache/swift"),
           "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": "/dev/null"}
    write(root / "build-environment.json", env)
    command = ["/usr/bin/nice", "-n", "15", str(DEVELOPER / "usr/bin/xcodebuild"),
               "-project", str(root / "source/Sparkle.xcodeproj"), "-scheme", "sparkle-cli",
               "-configuration", "Release", "-destination", "generic/platform=macOS", "-jobs", "2",
               "-derivedDataPath", str(root / "DerivedData"),
               "-clonedSourcePackagesDirPath", str(root / "packages"),
               "-disableAutomaticPackageResolution", "-skipPackageUpdates",
               "-onlyUsePackageVersionsFromResolvedFile", "-disablePackageRepositoryCache",
               "MACOSX_DEPLOYMENT_TARGET=15.0", "ARCHS=arm64", "ONLY_ACTIVE_ARCH=YES",
               "CODE_SIGNING_ALLOWED=NO", "CODE_SIGNING_REQUIRED=NO", "CODE_SIGN_IDENTITY=",
               "EXPANDED_CODE_SIGN_IDENTITY=", "DEVELOPMENT_TEAM=", "CODE_SIGN_STYLE=Manual",
               "ENABLE_HARDENED_RUNTIME=NO",
               "ENABLE_APP_INTENTS_METADATA_EXTRACTION=NO", "DEPLOYMENT_LOCATION=NO", "SKIP_INSTALL=YES",
               "DSTROOT=" + str(root / "dst"), "SYMROOT=" + str(root / "products"),
               "OBJROOT=" + str(root / "objects"), "CONFIGURATION_BUILD_DIR=" + str(root / "products"),
               "SHARED_PRECOMPS_DIR=" + str(root / "cache/precompiled"),
               "CLANG_MODULE_CACHE_PATH=" + str(root / "cache/clang")]
    write(root / "build-recipe-DO-NOT-RUN.json", {
        "command": command + ["build"], "safe_under_current_limits": False,
        "reason": "Xcode adds LaunchServices and execution-policy registration tasks."})
    unchanged = digest(inventory(root / "source")) == reviewed
    write(root / "preflight-result.json", {
        "status": "blocked", "xcode_invoked": False, "compiled": False, "source_unchanged": unchanged,
        "reason": "Official Xcode application build adds LaunchServices and execution-policy registration.",
        "settings_are_not_a_task_graph": True, "checked_at": stamp(),
        "controller_sha256": sha(Path(__file__))})
    raise RuntimeError("BLOCKED: source/recipe prepared; compilation is disabled because "
                       "Xcode registers products with shared host services. Evidence: " + str(root))


def main():
    os.umask(0o077)
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="action", required=True)
    prep = commands.add_parser("prepare")
    prep.add_argument("--source-repository", required=True, type=Path)
    compile_only = commands.add_parser("preflight")
    compile_only.add_argument("--root", required=True, type=Path)
    compile_only.add_argument("--reviewed-source-sha256", required=True)
    args = parser.parse_args()
    if args.action == "prepare":
        prepare(args.source_repository)
    else:
        preflight(args.root, args.reviewed_source_sha256)


if __name__ == "__main__":
    try:
        main()
    except (ValueError, RuntimeError, subprocess.CalledProcessError) as error:
        sys.exit(str(error))
