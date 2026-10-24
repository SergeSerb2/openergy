"""One format/compile pass. This controller never executes the example."""
import datetime
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parent
REPO = Path.cwd()
RAW = ROOT / "raw"
SOURCE = "06b9a139ec855a6f697ded393fa4c2c894110613"
CANDIDATE = "9f6625c951835295419ed1c53565581a7fa9681a"
ORIGINAL = "docs/verification/release-integration-2026-10-23/idle-tls-deadline-2701060"
TOOL = Path("/Users/serge/.rustup/toolchains/1.99.1-aarch64-apple-darwin/bin")
REGISTRY = Path("/Users/serge/.cargo/registry")


def sha(path):
    h = hashlib.sha256()
    with Path(path).open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def save(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def git(*args):
    return subprocess.check_output(["git", *args], cwd=REPO)


def inventory(root):
    files = []
    for path in sorted(root.rglob("*")):
        if path.is_symlink():
            raise RuntimeError("cache symlink refused")
        if path.is_file():
            files.append({"path": str(path.relative_to(root)), "bytes": path.stat().st_size,
                          "sha256": sha(path)})
    encoded = json.dumps(files, sort_keys=True, separators=(",", ":")).encode()
    return {"files": len(files), "bytes": sum(f["bytes"] for f in files),
            "manifest_sha256": hashlib.sha256(encoded).hexdigest()}


def defaults():
    base = Path("/Users/serge")
    names = [".rustup/settings.toml", ".cargo/config", ".cargo/config.toml",
             ".cargo/.global-cache", ".cargo/.global-cache-wal", ".cargo/.global-cache-shm",
             ".cargo/.package-cache", ".cargo/.package-cache-mutate"]
    return {name: {"exists": (base / name).is_file(),
                   "sha256": sha(base / name) if (base / name).is_file() else None}
            for name in names}


def run(name, command, env, timeout=300):
    record = {"name": name, "command": command, "cwd": str(REPO),
              "start_local": datetime.datetime.now().astimezone().isoformat(),
              "source_commit": SOURCE, "inherited_nice": os.getpriority(os.PRIO_PROCESS, 0),
              "timeout_seconds": timeout,
              "environment": {key: env[key] for key in [
                  "CARGO_HOME", "CARGO_TARGET_DIR", "CARGO_BUILD_JOBS", "CARGO_NET_OFFLINE",
                  "MACOSX_DEPLOYMENT_TARGET", "RUSTC", "RUSTDOC", "RUSTUP_TOOLCHAIN",
                  "RUSTUP_AUTO_INSTALL", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER"]}}
    start = time.monotonic_ns()
    try:
        with (RAW / (name + ".stdout")).open("xb") as out, (RAW / (name + ".stderr")).open("xb") as err:
            child = subprocess.run(command, cwd=REPO, env=env, stdout=out, stderr=err,
                                   timeout=timeout, check=False)
        record["exit_code"] = child.returncode
        record["timed_out"] = False
    except subprocess.TimeoutExpired:
        record["exit_code"] = None
        record["timed_out"] = True
    record["elapsed_ns"] = time.monotonic_ns() - start
    record["end_local"] = datetime.datetime.now().astimezone().isoformat()
    record["stdout_sha256"] = sha(RAW / (name + ".stdout"))
    record["stderr_sha256"] = sha(RAW / (name + ".stderr"))
    save(RAW / (name + ".json"), record)
    print(name, record["exit_code"], flush=True)
    return record


def main():
    os.umask(0o077)
    with (RAW / "build-once.guard").open("x") as guard:
        guard.write(datetime.datetime.now().astimezone().isoformat() + "\n")
    assert git("rev-parse", "HEAD").decode().strip() == SOURCE
    assert not git("diff", "--name-only", SOURCE, "--", "crates", "vendor", "Cargo.toml", "Cargo.lock")
    assert os.getpriority(os.PRIO_PROCESS, 0) == 15
    private = Path(tempfile.mkdtemp(prefix="oe-idle-tls-r2-build-773e34c7-", dir="/private/tmp"))
    cargo_home = private / "cargo-home"
    cargo_home.mkdir(mode=0o700)
    (RAW / "private-build-root.txt").write_text(str(private) + "\n")
    before = {"registry": inventory(REGISTRY), "defaults": defaults()}
    save(ROOT / "shared-inputs-before.json", before)
    subprocess.run(["/bin/cp", "-cR", str(REGISTRY), str(cargo_home / "registry")], check=True)
    copied = inventory(cargo_home / "registry")
    assert copied == before["registry"]
    save(ROOT / "private-cache-copy.json", {"source": str(REGISTRY), "destination": str(cargo_home / "registry"),
                                           "copy": copied, "byte_hashes_match": True,
                                           "credentials_or_global_config_copied": False})
    env = os.environ.copy()
    env.update(CARGO_HOME=str(cargo_home), CARGO_TARGET_DIR=str(private / "target"),
               CARGO_BUILD_JOBS="2", CARGO_NET_OFFLINE="true", MACOSX_DEPLOYMENT_TARGET="15.0",
               RUSTC=str(TOOL / "rustc"), RUSTDOC=str(TOOL / "rustdoc"),
               RUSTUP_TOOLCHAIN="1.99.1-aarch64-apple-darwin", RUSTUP_AUTO_INSTALL="0",
               RUSTC_WRAPPER="", RUSTC_WORKSPACE_WRAPPER="", PATH=str(TOOL) + ":" + env["PATH"])
    steps = [("rustc", [str(TOOL / "rustc"), "-Vv"]),
             ("cargo", [str(TOOL / "cargo"), "-V"]),
             ("host", ["/usr/bin/sw_vers"]),
             ("format-check", [str(TOOL / "rustfmt"), "--edition", "2021", "--check",
                               "crates/net/examples/dns_queue_cancel_c4806a3d.rs"])]
    base = ["--offline", "--locked", "--release", "-j", "2", "-p", "openergy-net",
            "--example", "dns_queue_cancel_c4806a3d"]
    steps += [("clippy", [str(TOOL / "cargo"), "clippy", *base, "--", "-D", "warnings"]),
              ("release", [str(TOOL / "cargo"), "build", *base])]
    results = []
    for name, command in steps:
        result = run(name, command, env)
        results.append(result)
        if result["exit_code"] != 0:
            break
    binary = private / "target/release/examples/dns_queue_cancel_c4806a3d"
    build_pass = len(results) == len(steps) and all(r["exit_code"] == 0 for r in results)
    if build_pass:
        results.append(run("binary-build", ["/usr/bin/xcrun", "vtool", "-show-build", str(binary)], env))
    after = {"registry": inventory(REGISTRY), "defaults": defaults()}
    save(ROOT / "shared-inputs-after.json", after)
    assert before == after
    original_files = git("ls-tree", "-r", "--name-only", CANDIDATE, "--", ORIGINAL).decode().splitlines()
    originals = []
    for name in original_files:
        old = git("show", CANDIDATE + ":" + name)
        current = (REPO / name).read_bytes()
        assert old == current
        originals.append({"path": name, "bytes": len(current), "sha256": hashlib.sha256(current).hexdigest()})
    save(ROOT / "original-evidence-preservation.json", {"candidate_commit": CANDIDATE,
         "files": originals, "all_bytes_identical": True})
    patch = git("diff", SOURCE + "^", SOURCE, "--", "crates/net/examples/dns_queue_cancel_c4806a3d/idle_tls_deadline.rs")
    (ROOT / "source-fix.patch").write_bytes(patch)
    inputs = ["crates/net/examples/dns_queue_cancel_c4806a3d.rs",
              "crates/net/examples/dns_queue_cancel_c4806a3d/idle_tls_deadline.rs",
              "Cargo.toml", "Cargo.lock", "crates/net/Cargo.toml", "rust-toolchain.toml", "rustfmt.toml"]
    save(ROOT / "source.json", {"original_parent": "a121376fe6c1715fa99d72559336ca3b3a73b439",
         "candidate_commit": CANDIDATE, "cherry_pick_commit": git("rev-parse", SOURCE + "^").decode().strip(),
         "source_commit": SOURCE, "source_tree": git("rev-parse", SOURCE + "^{tree}").decode().strip(),
         "fix_patch_sha256": sha(ROOT / "source-fix.patch"),
         "inputs": {name: {"sha256": sha(REPO / name), "git_blob": git("rev-parse", SOURCE + ":" + name).decode().strip()}
                    for name in inputs}, "production_library_diff": git("diff", CANDIDATE, SOURCE, "--", "crates/net/src").decode(),
         "main_example_unchanged_vs_candidate": not bool(git("diff", CANDIDATE, SOURCE, "--", inputs[0])),
         "runtime_attempts_for_fixed_source": 0})
    save(ROOT / "result.json", {"source_commit": SOURCE, "mac_format_clippy_release_pass": build_pass,
         "steps": [{"name": r["name"], "exit_code": r["exit_code"]} for r in results],
         "private_build_root": str(private), "shared_inputs_unchanged": before == after,
         "original_evidence_preserved": True, "original_runtime_overall": "FAIL / exit 1; immediate EADDRINUSE; cause unresolved",
         "runtime_attempts_for_fixed_source": 0, "unit_tests": False, "mocks": False,
         "binary": {"sha256": sha(binary), "bytes": binary.stat().st_size} if build_pass else None,
         "binary_executed": False})
    assert build_pass


if __name__ == "__main__":
    main()
