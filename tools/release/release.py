#!/usr/bin/env python3
"""Native build, real virtual-daemon checks, archives, and manual draft releases."""
from __future__ import annotations

import argparse
import datetime
import gzip
import hashlib
import json
import os
from pathlib import Path
import plistlib
import re
import shutil
import signal
import subprocess
import sys
import tarfile
import tempfile
import time
import zipfile

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


def run(args, *, capture=False, env=None, check=True, timeout=1800):
    args = [str(a) for a in args]
    if not capture:
        print("+ " + " ".join(args), flush=True)
    # Inherit this priority for every compiler and owned daemon on Unix.
    if os.name != "nt":
        args = ["nice", "-n", "15", *args]
    return subprocess.run(args, cwd=ROOT, env=env, text=True, check=check,
                          capture_output=capture, timeout=timeout)


def git(*args):
    return run(["git", *args], capture=True).stdout.strip()


def check(tag=None):
    workspace = tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]
    package = workspace["package"]
    version = package["version"]
    if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", version):
        raise ValueError("Native releases require a three-part numeric workspace version")
    toolchain = tomllib.loads((ROOT / "rust-toolchain.toml").read_text())["toolchain"]["channel"]
    if package["rust-version"] != toolchain:
        raise ValueError("workspace rust-version must match the pinned Rust toolchain")
    for member in workspace["members"]:
        manifest = tomllib.loads((ROOT / member / "Cargo.toml").read_text())["package"]
        for key in ("version", "rust-version"):
            if manifest.get(key) != {"workspace": True}:
                raise ValueError(f"{member}: {key} must inherit workspace.package")
    for variable in ("OPENERGY_VERSION",):
        if os.environ.get(variable, version) != version:
            raise ValueError(f"{variable} must match workspace version {version}")
    template = plistlib.loads((ROOT / "apps/macos/Support/Info.plist").read_bytes())
    if (template["CFBundleShortVersionString"] != "__VERSION__"
            or template["CFBundleVersion"] != "__BUILD__"):
        raise ValueError("Info.plist must use build-time version placeholders")
    swift = (ROOT / "apps/macos/Package.swift").read_text()
    minimum = template["LSMinimumSystemVersion"].split(".")[0]
    if f".macOS(.v{minimum})" not in swift:
        raise ValueError("Swift package and Info.plist minimum macOS versions differ")
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
    generator = ROOT / "docs/verification/public-release-audit-2026-10-21/generate-notices.py"
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
                command = ["nice", "-n", "15", *command]
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
        run(command, timeout=3600)
    if (ROOT / "Cargo.lock").read_bytes() != lock:
        raise ValueError("verification changed Cargo.lock")
    print(f"PASS: {target}; suite={suite}. Native input and external-device acceptance remain manual.")


def range_bounds(value):
    start, end = map(int, value.split("-"))
    return start, end + 1


def archive_tree(stage, archive, epoch):
    files = sorted(p for p in stage.rglob("*") if p.is_file())
    if archive.suffix == ".zip":
        stamp = datetime.datetime.fromtimestamp(max(epoch, 315532800), datetime.timezone.utc)
        with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as output:
            for path in files:
                info = zipfile.ZipInfo(path.relative_to(stage).as_posix(), stamp.timetuple()[:6])
                info.create_system = 3
                info.external_attr = (0o100755 if path.stat().st_mode & 0o111 else 0o100644) << 16
                info.compress_type = zipfile.ZIP_DEFLATED
                output.writestr(info, path.read_bytes())
    else:
        with archive.open("wb") as raw, gzip.GzipFile(filename="", fileobj=raw, mode="wb", mtime=epoch) as compressed:
            with tarfile.open(fileobj=compressed, mode="w", format=tarfile.PAX_FORMAT) as output:
                for path in files:
                    info = output.gettarinfo(str(path), path.relative_to(stage).as_posix())
                    info.uid = info.gid = 0
                    info.uname = info.gname = ""
                    info.mtime = epoch
                    info.mode = 0o755 if path.stat().st_mode & 0o111 else 0o644
                    with path.open("rb") as source:
                        output.addfile(info, source)


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
            shutil.copytree(base / "bundle/Openergy.app", stage / "Openergy.app")
            binary = stage / "Openergy.app/Contents/Helpers/openergy"
        else:
            binary = build(target)
            shutil.copy2(binary, stage / binary.name)
            binary = stage / binary.name
        smoke(binary, args.port_range)
        for source in [ROOT / "README.md", ROOT / "docs/releasing.md", *(ROOT / name for name in LEGAL)]:
            if source.is_file():
                shutil.copy2(source, stage / source.name)
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
        text = json.dumps(manifest, indent=2, sort_keys=True) + "\n"
        (stage / "BUILD.json").write_text(text, newline="\n")
        pending = base / archive.name
        archive_tree(stage, pending, epoch)
        # Exercise the archive extraction path and the archived daemon, not just staging.
        extracted = base / "extracted"
        if extension == ".zip":
            with zipfile.ZipFile(pending) as zipped:
                zipped.extractall(extracted)
                for entry in zipped.infolist():
                    (extracted / entry.filename).chmod((entry.external_attr >> 16) & 0o777)
        else:
            with tarfile.open(pending) as tarred:
                tarred.extractall(extracted, filter="data")
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


def draft(args):
    version, _ = check(args.tag)
    commit = git("rev-parse", "HEAD")
    args.artifacts = args.artifacts.resolve()
    assets = []
    seen = set()
    for sidecar in sorted(args.artifacts.glob("openergy-*.json")):
        manifest = json.loads(sidecar.read_text())
        target = manifest["target"]
        if (manifest["version"] != version or manifest["commit"] != commit
                or manifest["tag"] != args.tag or manifest["dirty"]
                or not manifest["third_party_notices"] or target not in TARGETS or target in seen
                or manifest["signing"] != ("adhoc" if "apple" in target else "unsigned")
                or manifest["notarized"] is not False):
            raise ValueError("artifact provenance does not match this draft release")
        candidates = [*args.artifacts.glob(sidecar.stem + ".zip"), *args.artifacts.glob(sidecar.stem + ".tar.gz")]
        if len(candidates) != 1:
            raise ValueError("each platform needs exactly one archive")
        archive = candidates[0]
        checksum = archive.with_name(archive.name + ".sha256")
        expected = f"{hashlib.sha256(archive.read_bytes()).hexdigest()}  {archive.name}\n"
        if checksum.read_text() != expected:
            raise ValueError("artifact checksum mismatch")
        seen.add(target)
        assets.extend([archive, checksum, sidecar])
    if seen != set(TARGETS):
        raise ValueError("draft needs all four native platform archives; no partial release")
    notes = (f"Openergy {version} — DRAFT FOR REVIEW\n\n"
             "macOS archives are ad-hoc signed and unnotarized. Windows and Linux archives are unsigned. "
             "These artifacts are not trusted distribution builds. Do not publish without manual review.\n\n"
             f"Source: {commit}\n\nEach archive has BUILD.json and a SHA-256 sidecar. "
             "Virtual daemon checks do not prove native input, TCC, login-item, or device acceptance.\n")
    with temporary("oe-draft-") as temporary_dir:
        notes_file = Path(temporary_dir) / "notes.txt"
        notes_file.write_text(notes)
        # gh refuses an existing release; do not overwrite assets or publish an existing draft.
        run(["gh", "release", "create", args.tag, "--draft", "--verify-tag", "--target", commit,
             "--title", f"Openergy {version} (review draft)", "--notes-file", notes_file, *assets])


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
    command = commands.add_parser("package")
    command.add_argument("--tag")
    command.add_argument("--port-range", type=ports, required=True)
    command.add_argument("--output", type=Path, default=ROOT / "dist/artifacts")
    command = commands.add_parser("draft")
    command.add_argument("--tag", required=True)
    command.add_argument("--artifacts", type=Path, required=True)
    args = parser.parse_args()
    if args.command in ("check", "version"):
        version, toolchain = check(args.tag)
        print(version if args.command == "version" else f"PASS: version {version}; Rust {toolchain}; workspace and macOS metadata consistent")
    else:
        {"verify": verify, "package": package, "draft": draft}[args.command](args)


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
