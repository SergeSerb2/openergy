#!/usr/bin/env python3
"""INERT proposal: one M1 resolution and normal bundle, after parent binding.

Do not import or run for source review. operation.template.json is disabled.
The parent must first verify the interpreter and this source as external data.
"""
from __future__ import annotations

import ctypes
import datetime
import errno
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import platform
import resource
import select
import shutil
import signal
import stat
import subprocess
import sys
import tarfile
import threading
import time
import tomllib

HERE = Path(__file__).resolve().parent
COMMIT = "ed7d96bf53da1a539fa2acde3b7133613221fd4b"
TOTAL = 1800
RESERVE = 15                 # Five seconds TERM grace, then ten seconds reporting.
GRACE = 5
LOG_CAP = 16 * 1024**2
ROOT_CAP = 24 * 1024**3      # Sampled logical bytes, not a filesystem quota.
ENTRY_CAP = 200000


def need(predicate, message):
    if not predicate:
        raise ValueError(message)


def sha(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024**2), b""):
            digest.update(chunk)
    return digest.hexdigest()


def save(path, value):
    with path.open("x") as stream:
        json.dump(value, stream, indent=2, sort_keys=True)
        stream.write("\n")


def unreadable(error):
    raise error


def concrete(value):
    if isinstance(value, dict):
        for item in value.values():
            concrete(item)
    elif isinstance(value, list):
        for item in value:
            concrete(item)
    elif isinstance(value, str):
        need("<" not in value and ">" not in value, "unbound placeholder")


def absolute(value):
    path = Path(value)
    need(path.is_absolute() and ".." not in path.parts, "absolute canonical path required")
    need(path.resolve() == path, "symlink or noncanonical path")
    return path


def prefix_hash(root):
    """SHA256 of sorted compact JSON arrays, each followed by LF; excludes root.

    [path,'d',mode], [path,'f',mode,length,sha256], or [path,'l',link-text].
    Internal symlinks only. This binds Python data, not interpreter safety.
    """
    rows = []
    total = 0
    for base, dirs, files in os.walk(root, followlinks=False, onerror=unreadable):
        for name in dirs + files:
            path = Path(base) / name
            info = path.lstat()
            rel = path.relative_to(root).as_posix()
            if stat.S_ISLNK(info.st_mode):
                need(path.resolve(strict=True).is_relative_to(root), "prefix link escapes")
                row = [rel, "l", os.readlink(path)]
            elif stat.S_ISDIR(info.st_mode):
                row = [rel, "d", stat.S_IMODE(info.st_mode)]
            else:
                need(stat.S_ISREG(info.st_mode), "non-file prefix input")
                total += info.st_size
                need(total <= 8 * 1024**3, "prefix exceeds eight GiB")
                row = [rel, "f", stat.S_IMODE(info.st_mode), info.st_size, sha(path)]
            rows.append(row)
            need(len(rows) <= 100000, "prefix exceeds entry cap")
    digest = hashlib.sha256()
    for row in sorted(rows):
        digest.update((json.dumps(row, separators=(",", ":"), ensure_ascii=True) + "\n").encode())
    return digest.hexdigest()


def unpack(archive, destination, records, prefix=""):
    """Extract exactly bound regular files, no tar links, devices or overwrites.

    Directory records are not needed: parents are generated here. Rust's supplied
    receipt has regular records only. Refuse rather than guess any other format.
    """
    expected = {prefix + row["path"]: row for row in records}
    need(len(expected) == len(records), "duplicate receipt path")
    destination.mkdir(exist_ok=True)
    with tarfile.open(archive, "r:*") as content:
        count = 0
        for item in content:
            count += 1
            need(count <= 10000, "archive entry cap")
            rel = PurePosixPath(item.name)
            need(not rel.is_absolute() and ".." not in rel.parts
                 and str(rel) == item.name.rstrip("/"), "unsafe tar path")
            path = destination / rel
            if item.isdir():
                need(any(name.startswith(str(rel) + "/") for name in expected), "extra tar directory")
                path.mkdir(parents=True, exist_ok=True)
                continue
            row = expected.pop(item.name, None)
            need(row is not None and item.isreg() and not item.sparse, "unexpected tar member")
            need(item.size == row["bytes"] and item.size <= 1024**3, "tar size differs")
            path.parent.mkdir(parents=True, exist_ok=True)
            with content.extractfile(item) as source, path.open("xb") as output:
                shutil.copyfileobj(source, output, 1024**2)
            need(sha(path) == row["sha256"], "extracted file hash differs")
            mode = int(row["mode"], 8) if "mode" in row else (0o755 if item.mode & 0o111 else 0o644)
            need(mode in (0o644, 0o755, 0o444, 0o555), "unexpected input mode")
            path.chmod(mode)
    need(not expected, "archive is incomplete")


def check_files(root, rows):
    for row in rows:
        path = root / row["path"]
        need(path.is_file() and not path.is_symlink() and path.stat().st_size == row["bytes"]
             and sha(path) == row["sha256"], "bound file changed: " + row["path"])
        if "mode" in row:
            need(stat.S_IMODE(path.stat().st_mode) == int(row["mode"], 8), "bound file mode changed")


def workspace_closure(repo, source_rows):
    """Read TOML data only, before any Cargo, Swift, or product helper command."""
    bound = {row["path"] for row in source_rows}
    workspace = tomllib.loads((repo / "Cargo.toml").read_text())["workspace"]
    members = workspace["members"]
    need(all(isinstance(member, str) and not any(c in member for c in "*?[") for member in members),
         "this exact source requires literal workspace members")
    member_manifests = [member + "/Cargo.toml" for member in members]
    config_inputs = ["Cargo.toml", "rust-toolchain.toml", "apps/macos/Package.swift",
                     "apps/macos/Support/Info.plist", *member_manifests]
    need(all(name in bound and (repo / name).is_file() for name in config_inputs),
         "release.py configuration input/workspace member missing")
    pending = ["Cargo.toml", *member_manifests]
    visited, edges = set(), []
    while pending:
        relative = pending.pop()
        if relative in visited:
            continue
        path = (repo / relative).resolve(strict=True)
        need(path.is_relative_to(repo) and relative in bound, "local manifest is outside bound source")
        document = tomllib.loads(path.read_text())
        tables = [document, document.get("workspace", {})]
        tables += list(document.get("target", {}).values())
        dependencies = []
        for table in tables:
            for kind in ("dependencies", "dev-dependencies", "build-dependencies"):
                dependencies += list(table.get(kind, {}).values())
        for patches in document.get("patch", {}).values():
            dependencies += list(patches.values())
        dependencies += list(document.get("replace", {}).values())
        for dependency in dependencies:
            if isinstance(dependency, dict) and "path" in dependency:
                target = (path.parent / dependency["path"] / "Cargo.toml").resolve(strict=True)
                need(target.is_relative_to(repo), "local dependency/patch escapes new source")
                name = target.relative_to(repo).as_posix()
                need(name in bound and target.is_file(), "local dependency/patch manifest missing")
                edges.append([relative, name])
                pending.append(name)
        visited.add(relative)
    return {"workspace_members": member_manifests, "release_configuration_inputs": config_inputs,
            "local_manifest_closure": sorted(visited), "local_path_edges": sorted(edges)}


def operation(plan, root):
    """Runs only inside the session keeper, with its independent lifeline active."""
    bindings = json.loads((HERE / "source-bindings.json").read_bytes())
    receipt_path = absolute(plan["input_directory"]) / "build-input-receipt.json"
    need(receipt_path.parent.stat().st_uid == os.getuid()
         and stat.S_IMODE(receipt_path.parent.stat().st_mode) == 0o700, "input directory must be owned mode0700")
    need(sha(receipt_path) == bindings["receipt_sha256"], "receipt hash differs")
    receipt = json.loads(receipt_path.read_bytes())
    need(receipt["source_commit"] == COMMIT, "receipt source differs")
    for name, row in bindings["python_prerequisite"]["files"].items():
        path = receipt_path.parent / name
        need(path.is_file() and not path.is_symlink() and path.stat().st_size == row["bytes"]
             and sha(path) == row["sha256"], "staged Python data differs: " + name)
    python = absolute(plan["python_prefix"])
    need(prefix_hash(python) == plan["python_prefix_sha256"], "Python prefix hash differs")
    need(Path(sys.executable).resolve() == (python / plan["python_relative_executable"]).resolve()
         and Path(sys.executable).resolve().is_relative_to(python), "wrong Python executable")
    need(sha(Path(sys.executable).resolve()) == bindings["python_prerequisite"]["interpreter_sha256"],
         "qualified Python executable differs")
    need(sys.version_info >= (3, 11) and sys.flags.isolated and sys.dont_write_bytecode,
         "require parent-verified Python 3.11+ with -I -B")
    need(platform.machine() == "arm64" and platform.mac_ver()[0] == "27.0.1", "wrong M1 baseline")
    for name, row in bindings["archives"].items():
        path = receipt_path.parent / name
        need(path.is_file() and not path.is_symlink() and path.stat().st_size == row["bytes"] and sha(path) == row["sha256"],
             "input archive differs: " + name)
    source_archive = absolute(plan["source_archive_path"])
    source_binding = bindings["source_archive"]
    source_receipt = absolute(plan["source_receipt_path"])
    need(source_receipt.is_file() and not source_receipt.is_symlink()
         and sha(source_receipt) == source_binding["receipt_sha256"], "corrected source receipt differs")
    need(source_archive.is_file() and not source_archive.is_symlink()
         and source_archive.stat().st_size == source_binding["bytes"]
         and sha(source_archive) == source_binding["sha256"], "corrected full source archive differs")
    repo = root / "repo"
    unpack(source_archive, repo, bindings["source_files"])
    unpack(receipt_path.parent / "rust-toolchain.tar.gz", root, receipt["inputs"]["rust"]["records"], "rust/")
    unpack(receipt_path.parent / "cargo-seed.tar.gz", root, receipt["inputs"]["cargo"]["files"])
    source_rows = bindings["source_files"]
    check_files(repo, source_rows)
    save(root / "workspace-closure.json", workspace_closure(repo, source_rows))
    need(sorted(p.relative_to(repo).as_posix() for p in repo.rglob("*") if p.is_file())
         == sorted(row["path"] for row in source_rows), "extra source files")
    for relative in ("target", ".git", "apps/macos/.build", "apps/macos/.swiftpm"):
        need(not (repo / relative).exists(), "reused source state")
    for name in ("home", "tmp", "bin", "dist", "cargo-target", "swift", "swift-cache",
                 "swift-config", "swift-security", "module-cache", "xdg-cache", "xdg-config", "rustup-empty"):
        (root / name).mkdir(mode=0o700)
    entry = root / "bin/swift"
    shutil.copyfile(HERE / "swift-entry.sh", entry)
    entry.chmod(0o700)
    developer = absolute(plan["developer_dir"])
    need(str(developer) == "/Applications/Xcode.app/Contents/Developer", "unexpected developer directory")
    toolbin = developer / "Toolchains/XcodeDefault.xctoolchain/usr/bin"
    sdk = developer / "Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk"
    need(toolbin.is_dir() and sdk.is_dir(), "Xcode data missing")
    env = {
        "PATH": f"{root}/bin:{root}/rust/bin:{toolbin}:/usr/bin:/bin:/usr/sbin:/sbin",
        "HOME": str(root / "home"), "CFFIXED_USER_HOME": str(root / "home"),
        "TMPDIR": str(root / "tmp") + "/", "XDG_CACHE_HOME": str(root / "xdg-cache"),
        "XDG_CONFIG_HOME": str(root / "xdg-config"), "DEVELOPER_DIR": str(developer),
        "SDKROOT": str(sdk), "MACOSX_DEPLOYMENT_TARGET": "15.0",
        "PYTHON": sys.executable, "PYTHONDONTWRITEBYTECODE": "1", "PYTHONNOUSERSITE": "1",
        "CARGO_HOME": str(root / "cargo-home"), "CARGO_TARGET_DIR": str(root / "cargo-target"),
        "CARGO_BUILD_JOBS": "2", "CARGO_NET_OFFLINE": "true", "CARGO_INCREMENTAL": "0",
        "RUSTUP_HOME": str(root / "rustup-empty"),
        "RUSTC": str(root / "rust/bin/rustc"), "RUSTDOC": str(root / "rust/bin/rustdoc"),
        "DIST": str(root / "dist"), "SWIFT_SCRATCH_PATH": str(root / "swift"),
        "CLANG_MODULE_CACHE_PATH": str(root / "module-cache"),
        "SWIFTPM_MODULECACHE_OVERRIDE": str(root / "module-cache"),
        "M1_SWIFT": str(toolbin / "swift"), "M1_SWIFT_CACHE": str(root / "swift-cache"),
        "M1_SWIFT_CONFIG": str(root / "swift-config"), "M1_SWIFT_SECURITY": str(root / "swift-security"),
        "UNIVERSAL": "0", "OPENERGY_BUILD": "1", "OPENERGY_SIGN_IDENTITY": "-",
        "OPENERGY_DISABLE_UPDATES": "1", "OPENERGY_DISABLE_DISCOVERY": "1", "OPENERGY_ALLOW_PORT_24800": "0",
        "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": "/dev/null", "GIT_CONFIG_SYSTEM": "/dev/null",
        "GIT_CONFIG_COUNT": "2", "GIT_CONFIG_KEY_0": "credential.helper", "GIT_CONFIG_VALUE_0": "",
        "GIT_CONFIG_KEY_1": "core.hooksPath", "GIT_CONFIG_VALUE_1": "/dev/null",
        "GIT_TERMINAL_PROMPT": "0", "GIT_SSH_COMMAND": "/usr/bin/false", "LC_ALL": "C", "LANG": "C",
    }
    save(root / "environment.json", env)
    started = time.monotonic()

    def command(label, argv, cap, metadata=False):
        need(os.getpriority(os.PRIO_PROCESS, 0) == 15, "keeper priority changed")
        save(root / (label + "-command.json"), {"argv": list(map(str, argv)), "cwd": str(repo),
                                               "monotonic": time.monotonic()})
        # Metadata files are read only after a successful bounded real command.
        if metadata:
            with (root / (label + ".txt")).open("xb") as output:
                process = subprocess.Popen(argv, cwd=repo, env=env, stdin=subprocess.DEVNULL, stdout=output)
                code = process.wait(timeout=cap)
            need((root / (label + ".txt")).stat().st_size <= 16384, "metadata output cap")
            text = (root / (label + ".txt")).read_text()
        else:
            process = subprocess.Popen(argv, cwd=repo, env=env, stdin=subprocess.DEVNULL)
            code = process.wait(timeout=cap)
            text = ""
        # Timeout raises without killing a child. The keeper then requests the
        # same group TERM/grace/KILL path as every other operation failure.
        save(root / (label + "-exit.json"), {"returncode": code, "monotonic": time.monotonic()})
        need(code == 0, "command failed: " + label)
        return text

    need(command("os-build", ["/usr/sbin/sysctl", "-n", "kern.osversion"], 10, True).strip() == "26A434", "OS build differs")
    need(command("xcode", ["/usr/bin/xcodebuild", "-version"], 15, True).strip()
         == "Xcode 27.1\nBuild version 27A9275", "Xcode version differs")
    command("first-launch", ["/usr/bin/xcodebuild", "-checkFirstLaunchStatus"], 15, True)
    need(command("sdk", ["/usr/bin/xcrun", "--sdk", "macosx", "--show-sdk-version"], 15, True).strip() == "27.0", "SDK differs")
    swift_version = command("swift-version", [toolbin / "swift", "--version"], 15, True)
    need("swiftlang-6.4.0.34.1 clang-2100.3.34.1" in swift_version, "Swift version differs")
    rust_version = command("rust-version", [root / "rust/bin/rustc", "-Vv"], 15, True)
    need("release: 1.99.1\n" in rust_version and "host: aarch64-apple-darwin\n" in rust_version
         and "commit-hash: 58cf8f9c35b355199727627d07ae34a4e2846d67\n" in rust_version
         and "LLVM version: 23.1.1" in rust_version, "Rust differs")
    need("cargo 1.99.1 " in command("cargo-version", [root / "rust/bin/cargo", "-V"], 15, True), "Cargo differs")
    command("resolve", [entry, "package", "--package-path", repo / "apps/macos",
                        "--scratch-path", root / "swift", "--jobs", "2", "--force-resolved-versions", "resolve"], 240)
    check_files(repo, source_rows)
    checkout = root / "swift/checkouts/Sparkle"
    need(command("sparkle-revision", ["/usr/bin/git", "-C", checkout, "rev-parse", "HEAD"], 10, True).strip()
         == bindings["sparkle"]["revision"], "Sparkle revision differs")
    # SwiftPM validates the binary-target checksum from that pinned manifest.
    manifest = (checkout / "Package.swift").read_text()
    need(bindings["sparkle"]["archive_sha256"] in manifest, "Sparkle checksum differs")
    need(not (root / "dist/Openergy.app").exists(), "output appeared before bundle")
    command("bundle", ["/bin/bash", repo / "apps/macos/scripts/bundle.sh"], 900)
    check_files(repo, source_rows)
    check_files(root / "rust", receipt["inputs"]["rust"]["records"])
    check_files(root, receipt["inputs"]["cargo"]["files"])
    need(prefix_hash(python) == plan["python_prefix_sha256"], "Python data changed")
    need(sha(receipt_path) == bindings["receipt_sha256"], "input receipt changed")
    for name, row in bindings["archives"].items():
        path = receipt_path.parent / name
        need(sha(path) == row["sha256"], "original input archive changed")
    need(sha(source_archive) == source_binding["sha256"], "corrected source archive changed")
    need(sha(source_receipt) == source_binding["receipt_sha256"], "corrected source receipt changed")
    for name, row in bindings["python_prerequisite"]["files"].items():
        need(sha(receipt_path.parent / name) == row["sha256"], "original Python data changed")
    app = root / "dist/Openergy.app"
    need(app.is_dir(), "normal bundle output missing")
    files = []
    for path in sorted(app.rglob("*")):
        row = {"path": path.relative_to(app).as_posix(), "mode": oct(stat.S_IMODE(path.lstat().st_mode))}
        if path.is_symlink():
            need(path.resolve(strict=True).is_relative_to(app), "bundle link escapes")
            row["link"] = os.readlink(path)
        elif path.is_file():
            row.update(bytes=path.stat().st_size, sha256=sha(path))
        else:
            need(path.is_dir(), "bundle special file")
            row["directory"] = True
        files.append(row)
    save(root / "bundle-files.json", {"app": str(app), "elapsed_operation_seconds": time.monotonic() - started, "files": files})
    return "BUNDLE_COMPLETED_ACCEPTANCE_UNVERIFIED"


class BSD(ctypes.Structure):
    # SDK27 sys/proc_info.h: proc_bsdinfo; MAXCOMLEN=16. Require the exact size.
    _fields_ = [(name, ctypes.c_uint32) for name in
                ("flags", "status", "xstatus", "pid", "ppid", "uid", "gid", "ruid", "rgid", "svuid", "svgid", "reserved")]
    _fields_ += [("comm", ctypes.c_char * 16), ("name", ctypes.c_char * 32)]
    _fields_ += [(name, ctypes.c_uint32) for name in ("nfiles", "pgid", "jobc", "tdev", "tpgid")]
    _fields_ += [("nice", ctypes.c_int32), ("seconds", ctypes.c_uint64), ("microseconds", ctypes.c_uint64)]


class Processes:
    def __init__(self):
        need(ctypes.sizeof(BSD) == 136, "proc_bsdinfo ABI differs")
        self.lib = ctypes.CDLL("/usr/lib/libproc.dylib", use_errno=True)
        self.lib.proc_pidinfo.argtypes = [ctypes.c_int, ctypes.c_int, ctypes.c_uint64, ctypes.c_void_p, ctypes.c_int]
        self.lib.proc_listpids.argtypes = [ctypes.c_uint32, ctypes.c_uint32, ctypes.c_void_p, ctypes.c_int]

    def one(self, pid):
        info = BSD()
        ctypes.set_errno(0)
        count = self.lib.proc_pidinfo(pid, 3, 0, ctypes.byref(info), ctypes.sizeof(info))
        if count == 0 and ctypes.get_errno() == errno.ESRCH:
            return None
        need(count == ctypes.sizeof(info), "uncertain process identity")
        return {name: int(getattr(info, name)) for name in ("pid", "ppid", "uid", "pgid", "nice", "status", "seconds", "microseconds")}

    def group(self, pgid):
        pids = (ctypes.c_int * 2048)()
        count = self.lib.proc_listpids(2, pgid, pids, ctypes.sizeof(pids))
        need(0 <= count < ctypes.sizeof(pids) and count % 4 == 0, "uncertain/overflow process inventory")
        return [row for pid in pids[:count // 4] if pid > 0 and (row := self.one(pid)) is not None]


def identity(row):
    return tuple(row[key] for key in ("pid", "uid", "seconds", "microseconds"))


def keeper(plan, root, control, status, stop_at):
    os.setsid()
    # Caught handlers reset to default at exec. SIG_IGN would leak into tools
    # and prevent their normal TERM handlers from running.
    signal.signal(signal.SIGTERM, lambda *_: None)
    signal.signal(signal.SIGINT, lambda *_: None)
    os.write(status, b"READY\n")
    ready, _, _ = select.select([control], [], [], 5)
    if not ready or os.read(control, 1) != b"G":
        os._exit(125)  # No payload has started.
    released = threading.Event()

    def lifeline():
        try:
            while time.monotonic() < stop_at:
                ready, _, _ = select.select([control], [], [], min(0.25, max(0, stop_at - time.monotonic())))
                if ready:
                    if os.read(control, 1) == b"R":
                        released.set()
                        return
                    break  # EOF: the parent is gone. Retain this live group anchor.
        except BaseException:
            pass  # An uncertain lifeline also stops this group.
        try:
            save(root / "recovery-term.json", {"monotonic": time.monotonic(), "pgid": os.getpgrp()})
        except OSError:
            pass  # A full volume must not prevent the signal.
        # Signal our own current session group: this live keeper is its anchor.
        # The parent only writes this lifeline pipe, never a reusable PID/PGID.
        os.killpg(os.getpgrp(), signal.SIGTERM)
        until = time.monotonic() + GRACE
        while time.monotonic() < until:
            time.sleep(min(0.05, max(0, until - time.monotonic())))
        try:
            save(root / "recovery-kill.json", {"monotonic": time.monotonic(), "pgid": os.getpgrp()})
        except OSError:
            pass
        os.killpg(os.getpgrp(), signal.SIGKILL)

    threading.Thread(target=lifeline, daemon=True).start()
    resource.setrlimit(resource.RLIMIT_FSIZE, (2 * 1024**3, 2 * 1024**3))
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    try:
        outcome = {"status": operation(plan, root)}
    except BaseException as error:
        outcome = {"status": "FAILED", "error": repr(error)[:2000]}
    try:
        save(root / "worker-result.json", outcome)
    except OSError:
        pass  # Missing result causes parent refusal; keep the live anchor.
    try:
        os.write(status, b"DONE\n")
    except BrokenPipeError:
        pass  # The lifeline will recover on parent EOF.
    released.wait()  # No next operation; stay alive until the parent decides.
    os._exit(0)


def usage(root, expected_root):
    def check_root():
        info = root.lstat()
        need(stat.S_ISDIR(info.st_mode) and info.st_uid == os.getuid()
             and stat.S_IMODE(info.st_mode) == 0o700
             and (info.st_dev, info.st_ino) == expected_root,
             "fresh-root identity or private mode changed")

    def walk_error(error):
        path = Path(error.filename) if isinstance(error.filename, str) else None
        if (isinstance(error, FileNotFoundError) and error.errno == errno.ENOENT
                and path is not None and path.is_absolute() and ".." not in path.parts
                and path != root and path.is_relative_to(root)):
            return  # A listed descendant can disappear during normal build cleanup.
        raise error  # Root loss, unknown paths and all other errors remain fatal.

    check_root()
    total = entries = 0
    for base, dirs, files in os.walk(root, followlinks=False, onerror=walk_error):
        for name in dirs + files:
            try:
                info = (Path(base) / name).lstat()
            except FileNotFoundError:
                continue  # Normal compiler temporary-file churn.
            total += info.st_size
            entries += 1
            need(total <= ROOT_CAP and entries <= ENTRY_CAP, "fresh-root sampled size cap")
    check_root()
    return total, entries


def main():
    need(len(sys.argv) == 2, "usage: parent-verified-python -I -B controller.py BOUND_OPERATION.json")
    raw = Path(sys.argv[1]).read_bytes()
    need(len(raw) <= 65536, "operation JSON cap")
    plan = json.loads(raw)
    need(plan.get("execution") is True, "INERT: operation execution is false")
    concrete(plan)
    need(sys.platform == "darwin" and sys.version_info >= (3, 11), "require macOS Python 3.11+")
    need(plan["source_commit"] == COMMIT and plan["schema"] == "openergy.m1-fresh-bundle.v1", "wrong operation")
    for name in ("controller.py", "swift-entry.sh", "source-bindings.json"):
        need(sha(HERE / name) == plan["recipe_sha256"][name], "recipe hash differs: " + name)
    concrete(json.loads((HERE / "source-bindings.json").read_bytes()))
    deadline = datetime.datetime.fromisoformat(plan["absolute_deadline_utc"].replace("Z", "+00:00"))
    need(deadline.utcoffset() == datetime.timedelta(0), "UTC deadline required")
    remaining = deadline.timestamp() - time.time()
    need(RESERVE + 60 < remaining <= TOTAL, "deadline must be within 1800 seconds, with time for recovery")
    end = time.monotonic() + remaining
    root = absolute(plan["fresh_root"])
    parent = root.parent.stat()
    need(parent.st_uid == os.getuid() and stat.S_IMODE(parent.st_mode) == 0o700, "fresh-root parent must be owned mode0700")
    need(not root.exists() and not root.is_symlink(), "fresh root already exists; no reset/retry")
    os.umask(0o077)
    os.setpriority(os.PRIO_PROCESS, 0, 15)
    need(os.getpriority(os.PRIO_PROCESS, 0) == 15, "cannot set absolute NI15")
    need(shutil.disk_usage(root.parent).free >= 32 * 1024**3, "require 32 GiB free")
    processes = Processes()
    root.mkdir(mode=0o700)
    root_info = root.lstat()
    root_token = (root_info.st_dev, root_info.st_ino)
    save(root / "bound-operation.json", plan)
    save(root / "start.json", {"binding_sha256": hashlib.sha256(raw).hexdigest(), "absolute_deadline_utc": plan["absolute_deadline_utc"],
                               "monotonic_deadline": end, "stop_work_at": end - RESERVE, "execution": True})
    control_read, control_write = os.pipe()
    status_read, status_write = os.pipe()
    log_read, log_write = os.pipe()
    pid = os.fork()
    if pid == 0:
        for fd in (control_write, status_read, log_read):
            os.close(fd)
        os.dup2(log_write, 1)
        os.dup2(log_write, 2)
        os.close(log_write)
        keeper(plan, root, control_read, status_write, end - RESERVE)
        os._exit(125)
    for fd in (control_read, status_write, log_write):
        os.close(fd)
    anchor = None
    reason = "FAILED"
    success = False
    received = b""
    written = 0
    observed = {}
    last_usage = 0
    os.set_blocking(log_read, False)
    log = (root / "output.log").open("xb")
    samples = (root / "processes.jsonl").open("x")
    dropped = 0

    def drain():
        nonlocal written, dropped
        try:
            data = os.read(log_read, 65536)
        except BlockingIOError:
            return False
        keep = data[:max(0, LOG_CAP - written)]
        log.write(keep)
        written += len(keep)
        dropped += len(data) - len(keep)
        return bool(data)

    try:
        while time.monotonic() < end - 10:
            readable, _, _ = select.select([status_read, log_read], [], [], 0.25)
            if log_read in readable:
                drain()
                need(dropped == 0, "combined output exceeded 16 MiB; prefix retained")
            if status_read in readable:
                data = os.read(status_read, 4096)
                need(data, "keeper ended before release")
                received += data
            if b"READY\n" in received and anchor is None:
                anchor = processes.one(pid)
                need(anchor is not None and anchor["pgid"] == pid and anchor["uid"] == os.getuid()
                     and anchor["nice"] == 15, "keeper identity/priority differs")
                save(root / "keeper.json", anchor)
                os.write(control_write, b"G")
            need(time.monotonic() < end - RESERVE, "absolute work deadline reached")
            if anchor is not None:
                current = processes.one(pid)
                need(current is not None and identity(current) == identity(anchor) and current["pgid"] == pid and current["status"] != 5,
                     "lost live keeper identity; no broad signalling")
                rows = processes.group(pid)
                need(any(row["pid"] == pid for row in rows), "keeper absent from group inventory")
                for row in rows:
                    need(row["uid"] == os.getuid() and row["pgid"] == pid and row["nice"] == 15,
                         "owned group priority/identity differs")
                    key = identity(row)
                    observed[key] = row
                    line = json.dumps({"t": time.monotonic(), **row}, separators=(",", ":")) + "\n"
                    need(samples.tell() + len(line.encode()) <= 8 * 1024**2, "process sample cap")
                    samples.write(line)
                if b"DONE\n" in received:
                    result = json.loads((root / "worker-result.json").read_bytes())
                    need(result["status"] == "BUNDLE_COMPLETED_ACCEPTANCE_UNVERIFIED", "worker failed; see worker-result.json")
                    need(len(rows) == 1 and rows[0]["pid"] == pid, "owned group children remain")
                    while drain():
                        need(dropped == 0, "combined output exceeded 16 MiB; prefix retained")
                    usage(root, root_token)
                    os.write(control_write, b"R")
                    success = True
                    reason = result["status"]
                    break
            if time.monotonic() - last_usage >= 2:
                usage(root, root_token)
                last_usage = time.monotonic()
        need(success, "total deadline reached")
    except BaseException as error:
        reason = repr(error)[:2000]
    finally:
        # No signal-by-PID race. Recovery is requested over the keeper's unique
        # pipe; it signals its OWN anchored group. EOF is also a stop request.
        recovery = {"stop_requested": False, "uncertain": False}
        if not success:
            try:
                current = processes.one(pid)
                need(anchor is not None and current is not None and identity(current) == identity(anchor) and current["pgid"] == pid
                     and current["status"] != 5, "no live anchor; recovery UNVERIFIED")
                os.write(control_write, b"T")
                recovery["stop_requested"] = True
            except BaseException as error:
                recovery.update(uncertain=True, error=repr(error)[:2000])
            os.close(control_write)  # Lost identity still gets pipe-EOF recovery, never a guessed signal.
            control_write = None
        # Bounded WNOHANG wait; do not reap early and then reuse the group ID.
        reaped = False
        while time.monotonic() < end:
            drain()  # Preserve recovery diagnostics within the same total output cap.
            child, status = os.waitpid(pid, os.WNOHANG)
            if child == pid:
                reaped = True
                recovery["keeper_wait_status"] = status
                if success and status != 0:
                    recovery["uncertain"] = True
                break
            time.sleep(0.05)
        recovery["keeper_reaped"] = reaped
        while time.monotonic() < end and drain():
            pass
        try:
            remaining_known = []
            for key, row in observed.items():
                need(time.monotonic() < end, "deadline reached during final identity inventory")
                current = processes.one(row["pid"])
                if current is not None and identity(current) == key:
                    remaining_known.append(current)
            recovery["remaining_sampled_identities"] = remaining_known
            recovery["uncertain"] |= bool(remaining_known) or not reaped
        except BaseException as error:
            recovery.update(uncertain=True, inventory_error=repr(error)[:2000])
        recovery["unseen_or_session_escaping_descendants"] = "UNVERIFIED; no universal containment claim"
        for name in ("recovery-term.json", "recovery-kill.json"):
            if (root / name).exists():
                recovery[name] = json.loads((root / name).read_bytes())
        save(root / "result.json", {"status": reason, "bundle_completed": success,
                                    "recovery": recovery, "monotonic_finished": time.monotonic(),
                                    "output_bytes": written, "dropped_output_bytes": dropped,
                                    "before_total_deadline": time.monotonic() <= end,
                                    "acceptance": "UNVERIFIED; no release or host-restoration clearance"})
        log.close()
        samples.close()
        for fd in (control_write, status_read, log_read):
            if fd is not None:
                os.close(fd)
    return 0 if success and not recovery["uncertain"] and time.monotonic() <= end else 1


if __name__ == "__main__":
    sys.exit(main())
