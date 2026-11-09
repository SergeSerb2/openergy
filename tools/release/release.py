#!/usr/bin/env python3
"""Native build, real virtual-daemon checks, archives, and manual draft releases."""
from __future__ import annotations

import argparse
import datetime
import gzip
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import plistlib
import re
import shutil
import signal
import stat
import subprocess
import sys
import tarfile
import tempfile
import time
import zipfile
import zlib

try:
    import tomllib
except ImportError:
    sys.exit("Python 3.11 or later is required. Set PYTHON to that interpreter for bundle.sh.")

ROOT = Path(__file__).resolve().parents[2]
TARGETS = (
    "aarch64-apple-darwin", "x86_64-apple-darwin",
    "x86_64-unknown-linux-gnu", "x86_64-pc-windows-msvc",
)
LEGAL = ("LICENSE-MIT", "LICENSE-APACHE", "THIRD_PARTY_NOTICES.md")
os.environ["CARGO_BUILD_JOBS"] = "2"
os.environ["PATH"] = str(Path.home() / ".cargo/bin") + os.pathsep + os.environ.get("PATH", "")


def run(args, *, capture=False, env=None, check=True, timeout=1800, binary=False):
    args = [str(a) for a in args]
    if not capture:
        print("+ " + " ".join(args), flush=True)
    # Nested release tools must retain absolute nice 15 instead of adding to it.
    if os.name != "nt":
        args = [sys.executable, ROOT / "tools/release/priority.py", "--", *args]
    return subprocess.run(args, cwd=ROOT, env=env, text=not binary, check=check,
                          capture_output=capture, timeout=timeout)


def git(*args):
    return run(["git", *args], capture=True).stdout.strip()


def configuration(read):
    """Validate native metadata supplied as bytes; never execute source files."""
    workspace = tomllib.loads(read("Cargo.toml").decode("utf-8"))["workspace"]
    package = workspace["package"]
    version = package["version"]
    if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", version):
        raise ValueError("Native releases require a three-part numeric workspace version")
    toolchain = tomllib.loads(read("rust-toolchain.toml").decode("utf-8"))["toolchain"]["channel"]
    if package["rust-version"] != toolchain:
        raise ValueError("workspace rust-version must match the pinned Rust toolchain")
    for member in workspace["members"]:
        manifest = tomllib.loads(read(f"{member}/Cargo.toml").decode("utf-8"))["package"]
        for key in ("version", "rust-version"):
            if manifest.get(key) != {"workspace": True}:
                raise ValueError(f"{member}: {key} must inherit workspace.package")
    for variable in ("OPENERGY_VERSION",):
        if os.environ.get(variable, version) != version:
            raise ValueError(f"{variable} must match workspace version {version}")
    template = plistlib.loads(read("apps/macos/Support/Info.plist"))
    if (template["CFBundleShortVersionString"] != "__VERSION__"
            or template["CFBundleVersion"] != "__BUILD__"):
        raise ValueError("Info.plist must use build-time version placeholders")
    swift = read("apps/macos/Package.swift").decode("utf-8")
    minimum = template["LSMinimumSystemVersion"].split(".")[0]
    if f".macOS(.v{minimum})" not in swift:
        raise ValueError("Swift package and Info.plist minimum macOS versions differ")
    return version, toolchain


def check(tag=None):
    version, toolchain = configuration(lambda name: (ROOT / name).read_bytes())
    if tag:
        if tag != f"v{version}":
            raise ValueError(f"tag must be v{version}")
        if git("rev-parse", f"refs/tags/{tag}^{{commit}}") != git("rev-parse", "HEAD"):
            raise ValueError("release tag must point to HEAD")
        if git("status", "--porcelain"):
            raise ValueError("tagged release requires a clean checkout, including untracked source")
        for name in LEGAL:
            if not (ROOT / name).is_file():
                raise ValueError(f"release is missing {name}")
    return version, toolchain


def native():
    rust = run(["rustc", "-Vv"], capture=True).stdout
    _, toolchain = check()
    if f"release: {toolchain}\n" not in rust:
        raise ValueError(f"use the pinned Rust toolchain {toolchain}")
    target = next(line[6:] for line in rust.splitlines() if line.startswith("host: "))
    if target not in TARGETS:
        raise ValueError(f"unsupported native release host: {target}")
    return target


def target_directory():
    metadata = run(["cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"], capture=True)
    return Path(json.loads(metadata.stdout)["target_directory"])


def notices():
    for name in LEGAL:
        if not (ROOT / name).is_file():
            raise ValueError(f"binary packaging/verification is missing {name}")
    generator = ROOT / "tools/release/notices/generate-notices.py"
    if not generator.is_file():
        raise ValueError("integrate the release notice generator before verification or packaging")
    # The generator's offline metadata covers all targets, not just this host.
    run(["cargo", "fetch", "--locked"])
    run([sys.executable, generator, "--check"])


def ports(value):
    match = re.fullmatch(r"([0-9]+)-([0-9]+)", value)
    if not match:
        raise argparse.ArgumentTypeError("expected START-END")
    start, end = map(int, match.groups())
    if not (1024 <= start <= end <= 65535) or start <= 24800 <= end or end - start > 100:
        raise argparse.ArgumentTypeError("use at most 101 unprivileged ports, excluding 24800")
    return value


def temporary(prefix):
    # Keep Unix socket names below sockaddr_un's path limit on macOS.
    return tempfile.TemporaryDirectory(prefix=prefix, dir=None if os.name == "nt" else "/tmp")


def smoke(binary, port_range):
    """Exercise the shipped CLI, real IPC, an error, and graceful daemon exit."""
    version, _ = check()
    with temporary("oe-release-smoke-") as temporary_dir:
        base = Path(temporary_dir)
        config, socket = base / "config.toml", base / "control.sock"
        port = int(port_range.split("-")[0])
        config.write_text(f'name = "release-check"\nrole = "off"\n[server]\n'
                          f'listen = "127.0.0.1:{port}"\nport = {port}\n')
        env = dict(os.environ, OPENERGY_CONFIG=str(config), OPENERGY_CONFIG_DIR=str(base),
                   OPENERGY_TLS_DIR=str(base / "tls"), OPENERGY_SOCKET=str(socket),
                   OPENERGY_DISABLE_DISCOVERY="1", OPENERGY_ALLOW_PORT_24800="0",
                   XDG_CONFIG_HOME=str(base), XDG_DATA_HOME=str(base),
                   XDG_STATE_HOME=str(base), XDG_RUNTIME_DIR=str(base), HOSTNAME="release-check")

        def cli(*args, code=0):
            result = run([binary, *args], env=env, capture=True, check=False, timeout=10)
            if result.returncode != code:
                raise ValueError(f"isolated CLI {args[0]} returned {result.returncode}, expected {code}")
            return result.stdout

        if cli("version").strip() != f"openergy {version}":
            raise ValueError("daemon version differs from workspace")
        cli("status", "--socket", socket, code=1)
        with (base / "daemon.log").open("w") as log:
            command = [str(binary), "daemon", "--platform", "virtual", "--config", str(config),
                       "--socket", str(socket), "--foreground"]
            if os.name != "nt":
                command = [sys.executable, ROOT / "tools/release/priority.py", "--", *command]
            process = subprocess.Popen(command, cwd=ROOT, env=env, stdin=subprocess.DEVNULL,
                                       stdout=log, stderr=log)
            try:
                deadline = time.monotonic() + 20
                while True:
                    result = run([binary, "status", "--socket", socket], env=env,
                                 capture=True, check=False, timeout=5)
                    if result.returncode == 0:
                        status = json.loads(result.stdout)
                        break
                    if process.poll() is not None or time.monotonic() >= deadline:
                        raise ValueError("isolated virtual daemon did not become ready")
                    time.sleep(0.1)
                if (status["daemon"]["platform"] != "virtual" or status["role"] != "off"
                        or status["daemon"]["version"] != version):
                    raise ValueError("isolated virtual daemon returned unexpected status")
                cli("ctl", "release.invalid.method", "--socket", socket, code=1)
                cli("ctl", "daemon.stop", "--socket", socket)
                if process.wait(timeout=10) != 0:
                    raise ValueError("isolated daemon did not exit cleanly")
            finally:
                if process.poll() is None:
                    process.terminate()
                    try:
                        process.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait(timeout=5)
    print("PASS: shipped version, absent-daemon error, virtual IPC status, invalid method, clean stop")


def build(target, workspace=False):
    selection = ["--workspace"] if workspace else ["--bin", "openergy"]
    run(["cargo", "build", "--release", "--locked", "--target", target, *selection])
    return target_directory() / target / "release" / ("openergy.exe" if os.name == "nt" else "openergy")


def verify(args):
    target = native()
    suite = args.suite
    if suite == "auto":
        suite = "smoke" if os.name == "nt" else "virtual"
    if os.name == "nt" and suite != "smoke":
        raise ValueError("the external scenario runner requires Unix; Windows supports CLI smoke only")
    if set(range(*range_bounds(args.port_range))).intersection(range(*range_bounds(args.lab_port_range))):
        raise ValueError("daemon and Docker port ranges must not overlap")
    if int(args.lab_port_range.split("-")[0]) < 24900:
        raise ValueError("Docker lab ports must start at 24900 or above")
    lock = (ROOT / "Cargo.lock").read_bytes()
    notices()
    run(["cargo", "fmt", "--all", "--check"])
    selection = ["-p", "openergy-daemon"] if os.name == "nt" else ["--workspace"]
    run(["cargo", "clippy", "--locked", "--target", target, *selection, "--all-targets", "--", "-D", "warnings"])
    binary = build(target, workspace=os.name != "nt")
    smoke(binary, args.port_range)
    if sys.platform == "darwin":
        run(["swift", "build", "--package-path", ROOT / "apps/macos", "-c", "release",
             "--jobs", "2", "--disable-automatic-resolution"])
        print("macOS app compiled; app UI/TCC/login-item/native-input acceptance was not run")
    if suite != "smoke":
        report = args.report.resolve()
        report.parent.mkdir(parents=True, exist_ok=True)
        command = [binary.parent / "openergy-e2e", "run", "--openergy", binary,
                   "--port-range", args.port_range, "--lab-port-range", args.lab_port_range,
                   "--report", report]
        if suite == "virtual":
            command.append("--no-docker")
        if args.e2e_slowdown != 1.0:
            command += ["--slowdown", str(args.e2e_slowdown)]
        run(command, timeout=3600)
    if (ROOT / "Cargo.lock").read_bytes() != lock:
        raise ValueError("verification changed Cargo.lock")
    print(f"PASS: {target}; suite={suite}. Native input and external-device acceptance remain manual.")


def range_bounds(value):
    start, end = map(int, value.split("-"))
    return start, end + 1


def staged_files(stage):
    """Include framework links without following them into duplicate payloads."""
    files = []
    root = stage.resolve(strict=True)
    for path in sorted(stage.rglob("*")):
        if path.is_symlink():
            target = Path(os.readlink(path))
            if target.is_absolute() or not path.resolve(strict=True).is_relative_to(root):
                raise ValueError("bundle symlink must resolve inside the staged payload")
            files.append(path)
        elif path.is_file():
            files.append(path)
        elif not path.is_dir():
            raise ValueError("staged payload contains an unsupported file type")
    return files


def archive_tree(stage, archive, epoch):
    files = staged_files(stage)
    if archive.suffix == ".zip":
        stamp = datetime.datetime.fromtimestamp(max(epoch, 315532800), datetime.timezone.utc)
        with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as output:
            for path in files:
                info = zipfile.ZipInfo(path.relative_to(stage).as_posix(), stamp.timetuple()[:6])
                info.create_system = 3
                link = path.is_symlink()
                mode = stat.S_IFLNK | 0o777 if link else stat.S_IFREG | (0o755 if path.stat().st_mode & 0o111 else 0o644)
                info.external_attr = mode << 16
                info.compress_type = zipfile.ZIP_DEFLATED
                if link:
                    output.writestr(info, os.readlink(path).encode("utf-8"))
                else:
                    with path.open("rb") as source, output.open(info, "w") as destination:
                        shutil.copyfileobj(source, destination)
    else:
        with archive.open("wb") as raw, gzip.GzipFile(filename="", fileobj=raw, mode="wb", mtime=epoch) as compressed:
            with tarfile.open(fileobj=compressed, mode="w", format=tarfile.PAX_FORMAT) as output:
                for path in files:
                    if path.is_symlink():
                        raise ValueError("tar release payloads must contain regular files only")
                    info = output.gettarinfo(str(path), path.relative_to(stage).as_posix())
                    info.uid = info.gid = 0
                    info.uname = info.gname = ""
                    info.mtime = epoch
                    info.mode = 0o755 if path.stat().st_mode & 0o111 else 0o644
                    with path.open("rb") as source:
                        output.addfile(info, source)


def extract_generated_zip(stage, archive, extracted):
    """Check our generated ZIP against staging before restoring framework links.

    This is not a general archive extractor. Names, types, link targets and
    regular-file bytes must match the complete staged payload exactly.
    """
    expected = {path.relative_to(stage).as_posix(): path for path in staged_files(stage)}
    links = []
    with zipfile.ZipFile(archive) as zipped:
        for entry in zipped.infolist():
            original = expected.pop(entry.filename, None)
            if original is None:
                raise ValueError("generated ZIP has an unexpected or duplicate entry")
            destination = extracted / original.relative_to(stage)
            kind = stat.S_IFMT(entry.external_attr >> 16)
            if original.is_symlink():
                target = os.readlink(original)
                encoded = target.encode("utf-8")
                if kind != stat.S_IFLNK or entry.file_size != len(encoded) or zipped.read(entry) != encoded:
                    raise ValueError("generated ZIP symlink differs from staging")
                links.append((destination, target))
            else:
                if kind != stat.S_IFREG or entry.file_size != original.stat().st_size:
                    raise ValueError("generated ZIP regular file differs from staging")
                destination.parent.mkdir(parents=True, exist_ok=True)
                with zipped.open(entry) as source, destination.open("xb") as output:
                    shutil.copyfileobj(source, output)
                with original.open("rb") as source, destination.open("rb") as copy:
                    if hashlib.file_digest(source, "sha256").digest() != hashlib.file_digest(copy, "sha256").digest():
                        raise ValueError("generated ZIP file bytes differ from staging")
                mode = 0o755 if original.stat().st_mode & 0o111 else 0o644
                if (entry.external_attr >> 16) & 0o777 != mode:
                    raise ValueError("generated ZIP executable mode differs from staging")
                destination.chmod(mode)
    if expected:
        raise ValueError("generated ZIP omits staged files")
    for destination, target in links:
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.symlink_to(target)
    root = extracted.resolve(strict=True)
    for destination, _ in links:
        if not destination.resolve(strict=True).is_relative_to(root):
            raise ValueError("generated ZIP symlink escapes the extracted payload")


def package(args):
    version, toolchain = check(args.tag)
    target = native()
    source_commit = git("rev-parse", "HEAD")
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    signing = "adhoc" if sys.platform == "darwin" else "unsigned"
    name = f"openergy-{version}-{target}-{signing}"
    extension = ".tar.gz" if sys.platform.startswith("linux") else ".zip"
    archive = output / (name + extension)
    destinations = [archive, output / (name + ".json"), output / (archive.name + ".sha256")]
    if any(path.exists() for path in destinations):
        raise ValueError("artifact destination already exists; select a fresh output directory")
    epoch = int(os.environ.get("SOURCE_DATE_EPOCH", git("show", "-s", "--format=%ct", "HEAD")))
    lock = (ROOT / "Cargo.lock").read_bytes()
    notices()
    with temporary("oe-package-") as temporary_dir:
        base = Path(temporary_dir)
        stage = base / "payload"
        stage.mkdir()
        if sys.platform == "darwin":
            env = dict(os.environ, DIST=str(base / "bundle"), PYTHON=sys.executable,
                       OPENERGY_SIGN_IDENTITY="-", UNIVERSAL="0")
            run(["bash", ROOT / "apps/macos/scripts/bundle.sh"], env=env)
            shutil.copytree(base / "bundle/Openergy.app", stage / "Openergy.app", symlinks=True)
            binary = stage / "Openergy.app/Contents/Helpers/openergy"
        else:
            binary = build(target)
            shutil.copy2(binary, stage / binary.name)
            binary = stage / binary.name
        smoke(binary, args.port_range)
        for name in LEGAL:
            shutil.copy2(ROOT / name, stage / name)
        (stage / "docs").mkdir()
        shutil.copy2(ROOT / "docs/releasing.md", stage / "docs/releasing.md")
        (stage / "examples").mkdir()
        for name in ("server.toml", "client.toml"):
            shutil.copy2(ROOT / "examples" / name, stage / "examples" / name)
        if os.name == "nt":
            scripts = sorted(path for path in (ROOT / "tools/windows").iterdir()
                             if path.is_file() and path.suffix.lower() in (".ps1", ".cmd"))
            if not scripts:
                raise ValueError("Windows archive is missing its installer/wrapper scripts")
            for source in scripts:
                shutil.copy2(source, stage / source.name)
        source_url = f"https://github.com/SergeSerb2/openergy/blob/{source_commit}"
        readme = (f"# Openergy {version}\n\nNative target: `{target}`.\n\n"
                  f"Signing: **{signing}; unnotarized**. This is not a trusted distribution build.\n\n"
                  "The macOS payload is Openergy.app. Linux and Windows payloads are the daemon/CLI; "
                  "Windows installation and wrapper scripts sit beside openergy.exe. "
                  "This archive does not install software or register a login task by itself.\n\n"
                  f"Read the [source installation instructions]({source_url}/README.md) for this exact commit. "
                  "See [release limits](docs/releasing.md), [server config](examples/server.toml), "
                  "and [client config](examples/client.toml). Release build commands require a source checkout; "
                  "the archive does not contain the build tools.\n\n"
                  "Keep LICENSE-MIT, LICENSE-APACHE, and THIRD_PARTY_NOTICES.md with these binaries. "
                  "BUILD.json records the source and compiler inputs. The SHA-256 sidecar detects changed "
                  "downloads; it does not authenticate a publisher.\n\n"
                  "Packaged daemon CLI/virtual IPC checks passed. Native desktop input, clipboard, "
                  "TCC, login-item, and real-device acceptance were not exercised by packaging.\n")
        (stage / "README.md").write_text(readme, newline="\n")
        manifest = {"version": version, "tag": args.tag, "commit": source_commit,
                    "dirty": bool(git("status", "--porcelain")),
                    "target": target, "rust": toolchain, "signing": signing, "notarized": False,
                    "source_date_epoch": epoch, "third_party_notices": (ROOT / LEGAL[-1]).is_file(),
                    "verification": "packaged daemon CLI/virtual IPC smoke; native input not tested"}
        manifest["compiler"] = run(["rustc", "-V"], capture=True).stdout.strip()
        if sys.platform == "darwin":
            manifest["swift"] = run(["swift", "--version"], capture=True).stdout.strip()
            manifest["macos_sdk"] = run(["xcrun", "--sdk", "macosx", "--show-sdk-version"], capture=True).stdout.strip()
            manifest["xcode"] = run(["xcodebuild", "-version"], capture=True).stdout.strip()
            manifest["macos_build"] = os.environ.get("OPENERGY_BUILD", "1")
            manifest["swift_package_resolved_sha256"] = hashlib.sha256(
                (ROOT / "apps/macos/Package.resolved").read_bytes()).hexdigest()
        text = json.dumps(manifest, indent=2, sort_keys=True) + "\n"
        (stage / "BUILD.json").write_text(text, newline="\n")
        pending = base / archive.name
        archive_tree(stage, pending, epoch)
        # Exercise the archive extraction path and the archived daemon, not just staging.
        extracted = base / "extracted"
        if extension == ".zip":
            extract_generated_zip(stage, pending, extracted)
        else:
            # This is our freshly generated payload, not an arbitrary archive.
            # Match its regular files to staging instead of relying on tarfile
            # extraction filters, which older supported Python 3.11 lacks.
            expected = {path.relative_to(stage).as_posix(): path
                        for path in stage.rglob("*") if path.is_file()}
            with tarfile.open(pending) as tarred:
                for entry in tarred:
                    original = expected.pop(entry.name, None)
                    if (original is None or entry.type not in (tarfile.REGTYPE, tarfile.AREGTYPE)
                            or entry.sparse or entry.size != original.stat().st_size):
                        raise ValueError("generated archive differs from staged regular files")
                    destination = extracted / original.relative_to(stage)
                    destination.parent.mkdir(parents=True, exist_ok=True)
                    payload = tarred.extractfile(entry)
                    if payload is None:
                        raise ValueError("generated archive is missing file data")
                    with payload, destination.open("xb") as output_file:
                        shutil.copyfileobj(payload, output_file)
                    with original.open("rb") as staged_file, destination.open("rb") as extracted_file:
                        if (hashlib.file_digest(staged_file, "sha256").digest()
                                != hashlib.file_digest(extracted_file, "sha256").digest()):
                            raise ValueError("generated archive file differs from staging")
                    destination.chmod(0o755 if original.stat().st_mode & 0o111 else 0o644)
            if expected:
                raise ValueError("generated archive omits staged files")
        extracted_binary = extracted / binary.relative_to(stage)
        smoke(extracted_binary, args.port_range)
        if sys.platform == "darwin":
            run(["codesign", "--verify", "--deep", "--strict", extracted / "Openergy.app"])
        if (ROOT / "Cargo.lock").read_bytes() != lock:
            raise ValueError("packaging changed Cargo.lock")
        check(args.tag)
        if git("rev-parse", "HEAD") != source_commit:
            raise ValueError("source commit changed while packaging")
        digest = hashlib.sha256(pending.read_bytes()).hexdigest()
        shutil.copyfile(pending, archive)
        destinations[1].write_text(text, newline="\n")
        destinations[2].write_text(f"{digest}  {archive.name}\n", newline="\n")
    print(f"Created {archive.name}; {signing}, unnotarized. Not a trusted distribution build.")


def archive_manifest(archive):
    limit = 1024 * 1024

    def select(entries):
        # Include alternate/nested paths in the count to reject ambiguous manifests.
        matches = [(name, entry) for name, entry in entries
                   if PurePosixPath(name.replace("\\", "/")).name.casefold() == "build.json"]
        if len(matches) != 1 or matches[0][0] != "BUILD.json":
            raise ValueError("archive needs exactly one BUILD.json at its root")
        return matches[0][1]

    def check_entry(size, regular):
        if not regular:
            raise ValueError("archive BUILD.json must be a regular file")
        if not 0 <= size <= limit:
            raise ValueError("archive BUILD.json exceeds 1 MiB")

    try:
        if archive.suffix == ".zip":
            with zipfile.ZipFile(archive) as contents:
                entry = select((item.filename, item) for item in contents.infolist())
                # DOS ZIPs may omit Unix type bits; explicit non-file types fail.
                kind = stat.S_IFMT(entry.external_attr >> 16)
                check_entry(entry.file_size, not entry.is_dir() and not entry.external_attr & 0x10
                            and kind in (0, stat.S_IFREG))
                with contents.open(entry) as source:
                    data = source.read(limit + 1)
        elif archive.name.endswith(".tar.gz"):
            with tarfile.open(archive, "r:gz") as contents:
                entry = select((item.name, item) for item in contents)
                check_entry(entry.size, entry.isfile())
                with contents.extractfile(entry) as source:
                    data = source.read(limit + 1)
        else:
            raise ValueError("release archives must use .zip or .tar.gz")
    except (zipfile.BadZipFile, tarfile.TarError, zlib.error, EOFError, RuntimeError) as error:
        raise ValueError(f"invalid release archive: {error}") from error
    if len(data) > limit:
        raise ValueError("archive BUILD.json exceeds 1 MiB")
    return json.loads(data)


def validate_artifacts(artifacts, version, commit, tag, targets=None):
    """Read artifact data only. This does not extract archives or run a binary.

    Draft creation keeps the default complete target set. Only download-check
    may name the targets a published release actually carries; every named
    target still gets the full checksum and embedded-manifest checks.
    """
    expected_targets = set(TARGETS) if targets is None else set(targets)
    if not expected_targets or not expected_targets.issubset(TARGETS):
        raise ValueError("expected artifact targets must be a nonempty subset of the release targets")
    artifacts = artifacts.resolve()
    assets = []
    seen = set()
    for sidecar in sorted(artifacts.glob("openergy-*.json")):
        manifest = json.loads(sidecar.read_text())
        target = manifest["target"]
        if (manifest["version"] != version or manifest["commit"] != commit
                or manifest["tag"] != tag or manifest["dirty"]
                or not manifest["third_party_notices"] or target not in TARGETS or target in seen
                or manifest["signing"] != ("adhoc" if "apple" in target else "unsigned")
                or manifest["notarized"] is not False):
            raise ValueError("artifact provenance does not match this draft release")
        candidates = [*artifacts.glob(sidecar.stem + ".zip"), *artifacts.glob(sidecar.stem + ".tar.gz")]
        if len(candidates) != 1:
            raise ValueError("each platform needs exactly one archive")
        archive = candidates[0]
        checksum = archive.with_name(archive.name + ".sha256")
        expected = f"{hashlib.sha256(archive.read_bytes()).hexdigest()}  {archive.name}\n"
        if checksum.read_text() != expected:
            raise ValueError("artifact checksum mismatch")
        if archive_manifest(archive) != manifest:
            raise ValueError("archive BUILD.json does not match its provenance sidecar")
        seen.add(target)
        assets.extend([archive, checksum, sidecar])
    if seen != expected_targets:
        if targets is None:
            raise ValueError("draft needs all four native platform archives; no partial release")
        raise ValueError(f"expected release targets {sorted(expected_targets)}, found {sorted(seen)}")
    return assets


def create_draft(tag, version, commit, assets):
    notes = (f"Openergy {version} — DRAFT FOR REVIEW\n\n"
             "macOS archives are ad-hoc signed and unnotarized. Windows and Linux archives are unsigned. "
             "These artifacts are not trusted distribution builds. Do not publish without manual review.\n\n"
             f"Source: {commit}\n\nEach archive has BUILD.json and a SHA-256 sidecar. "
             "Virtual daemon checks do not prove native input, TCC, login-item, or device acceptance.\n")
    with temporary("oe-draft-") as temporary_dir:
        notes_file = Path(temporary_dir) / "notes.txt"
        notes_file.write_text(notes)
        # gh refuses an existing release; do not overwrite assets or publish an existing draft.
        run(["gh", "release", "create", tag, "--draft", "--verify-tag", "--target", commit,
             "--title", f"Openergy {version} (review draft)", "--notes-file", notes_file, *assets])


def draft(args):
    version, _ = check(args.tag)
    commit = git("rev-parse", "HEAD")
    assets = validate_artifacts(args.artifacts, version, commit, args.tag)
    create_draft(args.tag, version, commit, assets)


def downloaded_release_commit(tag, repo, sidecars):
    """Resolve the expected source commit without trusting a single sidecar."""
    command = ["gh", "release", "view", tag, "--json", "targetCommitish"]
    if repo:
        command += ["--repo", repo]
    result = run(command, capture=True, check=False, timeout=60)
    if result.returncode == 0:
        target = json.loads(result.stdout)["targetCommitish"]
        if re.fullmatch(r"[0-9a-f]{40}", target):
            return target
    result = run(["git", "rev-parse", "--verify", f"refs/tags/{tag}^{{commit}}"],
                 capture=True, check=False, timeout=60)
    if result.returncode == 0 and re.fullmatch(r"[0-9a-f]{40}", result.stdout.strip()):
        return result.stdout.strip()
    commits = {json.loads(sidecar.read_text())["commit"] for sidecar in sidecars}
    if len(commits) != 1:
        raise ValueError("downloaded provenance sidecars disagree on the source commit")
    print("NOTE: release target is not a pinned commit; using the commit all sidecars agree on")
    return commits.pop()


def check_uncovered_checksums(artifacts, assets):
    """Check checksum sidecars validate_artifacts does not own (universal ZIP)."""
    covered = {path.name for path in assets}
    checked = []
    for checksum in sorted(artifacts.glob("*.sha256")):
        if checksum.name in covered:
            continue
        fields = checksum.read_text().split()
        archive_name = checksum.name.removesuffix(".sha256")
        archive = artifacts / archive_name
        if (len(fields) != 2 or fields[1] != archive_name or not archive.is_file()
                or hashlib.sha256(archive.read_bytes()).hexdigest() != fields[0]):
            raise ValueError(f"checksum mismatch for {archive_name}")
        covered.add(archive_name)
        checked.append(archive_name)
    for path in sorted(artifacts.iterdir()):
        if (path.name.endswith((".zip", ".tar.gz")) and path.name not in covered):
            raise ValueError(f"downloaded archive has no validated checksum: {path.name}")
    return checked


def extract_downloaded_linux(archive, extracted):
    """Extract a downloaded tar by hand: confined regular files only, capped size."""
    total = 0
    seen = set()
    try:
        with tarfile.open(archive, "r:gz") as tarred:
            for entry in tarred:
                parts = PurePosixPath(entry.name.replace("\\", "/")).parts
                normalized = "/".join(parts)
                if (not parts or entry.name.startswith("/") or ".." in parts
                        or normalized in seen):
                    raise ValueError("downloaded archive has an unsafe or duplicate entry name")
                seen.add(normalized)
                if entry.isdir():
                    continue
                if entry.type not in (tarfile.REGTYPE, tarfile.AREGTYPE) or entry.sparse:
                    raise ValueError("downloaded archive must contain regular files only")
                total += entry.size
                if entry.size < 0 or total > 512 * 1024 * 1024:
                    raise ValueError("downloaded archive exceeds the 512 MiB extraction cap")
                destination = extracted.joinpath(*parts)
                destination.parent.mkdir(parents=True, exist_ok=True)
                payload = tarred.extractfile(entry)
                if payload is None:
                    raise ValueError("downloaded archive is missing file data")
                with payload, destination.open("xb") as output_file:
                    shutil.copyfileobj(payload, output_file)
                destination.chmod(0o755 if entry.mode & 0o111 else 0o644)
    except (tarfile.TarError, zlib.error, EOFError) as error:
        raise ValueError(f"invalid downloaded archive: {error}") from error
    binary = extracted / "openergy"
    if not binary.is_file() or binary.is_symlink():
        raise ValueError("downloaded Linux archive has no openergy binary at its root")
    return binary


def download_check(args):
    """Download a published release, reuse the artifact gate, smoke its binary.

    Unlike trusted_release.py this deliberately executes the downloaded Linux
    binary, with --help/--version only, on the maintainer's own machine.
    """
    if not re.fullmatch(r"v[0-9]+\.[0-9]+\.[0-9]+", args.tag):
        raise ValueError("tag must be a literal version tag such as v0.1.0")
    version = args.tag[1:]
    output = args.output.resolve()
    if output.exists() and (not output.is_dir() or any(output.iterdir())):
        raise ValueError("download destination must be a fresh or empty directory")
    output.mkdir(parents=True, exist_ok=True)
    command = ["gh", "release", "download", args.tag, "--dir", output]
    if args.repo:
        command += ["--repo", args.repo]
    run(command, timeout=600)
    for path in output.iterdir():
        if path.is_symlink() or not path.is_file():
            raise ValueError("downloaded release assets must be regular files only")
    sidecars = sorted(output.glob("openergy-*.json"))
    if not sidecars:
        raise ValueError("downloaded release has no provenance sidecars")
    present = {json.loads(sidecar.read_text())["target"] for sidecar in sidecars}
    commit = downloaded_release_commit(args.tag, args.repo, sidecars)
    assets = validate_artifacts(output, version, commit, args.tag, targets=present)
    extra = check_uncovered_checksums(output, assets)
    print(f"PASS: checksums and embedded BUILD.json for {sorted(present)}"
          + (f"; extra checksum sidecars for {extra}" if extra else ""))
    if args.skip_smoke:
        print("SKIP: downloaded Linux binary smoke not run (--skip-smoke)")
        return
    linux = [path for path in assets if path.name.endswith(".tar.gz")
             and "x86_64-unknown-linux-gnu" in path.name]
    if len(linux) != 1:
        raise ValueError("download-check smoke needs the x86_64 Linux archive; use --skip-smoke to only validate")
    with temporary("oe-download-check-") as temporary_dir:
        binary = extract_downloaded_linux(linux[0], Path(temporary_dir) / "extracted")
        for flag, expected in (("--version", f"openergy {version}"), ("--help", "usage: openergy")):
            result = run([binary, flag], capture=True, check=False, timeout=10)
            if result.returncode != 0:
                detail = (result.stderr or result.stdout).strip().splitlines()
                raise ValueError(f"downloaded binary {flag} returned {result.returncode}"
                                 + (f": {detail[0]}" if detail else "")
                                 + "; run on x86_64 Linux or pass --skip-smoke to only validate")
            if not result.stdout.strip().startswith(expected):
                raise ValueError(f"downloaded binary {flag} output does not start with {expected!r}")
    print(f"PASS: downloaded Linux binary --version is openergy {version} and --help runs")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    for name in ("check", "version"):
        command = commands.add_parser(name)
        command.add_argument("--tag")
    command = commands.add_parser("verify")
    command.add_argument("--suite", choices=("auto", "smoke", "virtual", "full"), default="auto")
    command.add_argument("--port-range", type=ports, required=True)
    command.add_argument("--lab-port-range", type=ports, required=True)
    command.add_argument("--report", type=Path, default=ROOT / "dist/verification.json")
    command.add_argument("--e2e-slowdown", type=float, default=1.0,
                         help="stretch scenario deadlines and latency/CPU budgets on slower machines (1-10)")
    command = commands.add_parser("package")
    command.add_argument("--tag")
    command.add_argument("--port-range", type=ports, required=True)
    command.add_argument("--output", type=Path, default=ROOT / "dist/artifacts")
    command = commands.add_parser("draft")
    command.add_argument("--tag", required=True)
    command.add_argument("--artifacts", type=Path, required=True)
    command = commands.add_parser("download-check",
                                    help="download a published release, validate its assets, and smoke the Linux binary")
    command.add_argument("--tag", required=True)
    command.add_argument("--output", type=Path, required=True,
                         help="fresh or empty directory the release assets are downloaded into")
    command.add_argument("--repo", help="GitHub OWNER/REPO to pass to gh (default: this checkout's remote)")
    command.add_argument("--skip-smoke", action="store_true",
                         help="validate checksums/manifests only; do not run the downloaded Linux binary")
    args = parser.parse_args()
    if args.command in ("check", "version"):
        version, toolchain = check(args.tag)
        print(version if args.command == "version" else f"PASS: version {version}; Rust {toolchain}; workspace and macOS metadata consistent")
    else:
        {"verify": verify, "package": package, "draft": draft,
         "download-check": download_check}[args.command](args)


def interrupted(signum, _frame):
    # Let context managers and the owned-daemon finally block run on termination.
    raise SystemExit(128 + signum)


if __name__ == "__main__":
    signal.signal(signal.SIGTERM, interrupted)
    try:
        main()
    except KeyboardInterrupt:
        sys.exit(130)
    except (ValueError, OSError, KeyError, subprocess.SubprocessError) as error:
        sys.exit(f"release: {error}")
