#!/usr/bin/env python3
"""Exercise identity/trust through the real offline CLI and concurrent processes."""
import argparse
import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time

parser = argparse.ArgumentParser()
parser.add_argument("--openergy", type=Path, required=True)
args = parser.parse_args()
binary = args.openergy.resolve()
root = Path(tempfile.mkdtemp(prefix="oe-net-release-"))


def environment(directory):
    return dict(os.environ, OPENERGY_CONFIG_DIR=str(directory),
                OPENERGY_TLS_DIR=str(directory / "tls"),
                OPENERGY_SOCKET=str(directory / "ctl.sock"),
                OPENERGY_DISABLE_DISCOVERY="1")


def command(directory, *arguments):
    return [str(binary), *arguments, "--config", str(directory / "config.toml")]


def cli(directory, *arguments, success=True):
    result = subprocess.run(command(directory, *arguments), env=environment(directory),
                            text=True, capture_output=True, timeout=10)
    assert (result.returncode == 0) == success, (arguments, result.returncode, result.stderr)
    return result.stdout.strip()


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


seed = root / "seed"
fingerprint = cli(seed, "fingerprint", "--synergy")
assert fingerprint == cli(seed, "fingerprint", "--synergy")
if os.name == "posix":
    assert (seed / "tls").stat().st_mode & 0o777 == 0o700
    for name in ["cert.pem", "key.pem"]:
        path = seed / "tls" / name
        assert path.stat().st_mode & 0o777 == 0o600
        path.chmod(0o644)
    assert fingerprint == cli(seed, "fingerprint", "--synergy")
    assert (seed / "tls/key.pem").stat().st_mode & 0o777 == 0o600
print("identity stable fingerprint and private permissions=PASS")

fresh = root / "fresh-concurrent"
children = [subprocess.Popen(command(fresh, "fingerprint", "--synergy"),
                            env=environment(fresh), stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            for _ in range(24)]
try:
    fingerprints = set()
    for child in children:
        stdout, stderr = child.communicate(timeout=10)
        assert child.returncode == 0, stderr.decode()
        fingerprints.add(stdout.strip())
    assert len(fingerprints) == 1
finally:
    for child in children:
        if child.poll() is None:
            child.kill()
            child.wait()
print("24 concurrent real fingerprint commands share one identity=PASS")

# Simulate a real writer paused between its two renames while holding the lock.
if os.name == "posix":
    import fcntl
    concurrent = root / "concurrent"
    (concurrent / "tls").mkdir(parents=True)
    with (concurrent / "tls/.identity-lock").open("w") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        shutil.copyfile(seed / "tls/key.pem", concurrent / "tls/key.pem")
        child = subprocess.Popen(command(concurrent, "fingerprint", "--synergy"),
                                 env=environment(concurrent), stdout=subprocess.PIPE,
                                 stderr=subprocess.PIPE, text=True)
        try:
            time.sleep(0.15)
            assert child.poll() is None, "reader did not wait for identity writer"
            shutil.copyfile(seed / "tls/cert.pem", concurrent / "tls/cert.pem")
            fcntl.flock(lock, fcntl.LOCK_UN)
            stdout, stderr = child.communicate(timeout=3)
            assert child.returncode == 0 and stdout.strip() == fingerprint, stderr
        finally:
            if child.poll() is None:
                child.kill()
                child.wait()
    print("identity concurrent reader waits for complete pair=PASS")

partial = root / "partial"
(partial / "tls").mkdir(parents=True)
shutil.copyfile(seed / "tls/key.pem", partial / "tls/key.pem")
before = digest(partial / "tls/key.pem")
cli(partial, "fingerprint", "--synergy", success=False)
assert digest(partial / "tls/key.pem") == before and not (partial / "tls/cert.pem").exists()
if os.name == "posix":
    linked = root / "linked"
    (linked / "tls").mkdir(parents=True)
    # A dangling link also counts as an existing, invalid identity. Never replace it.
    (linked / "tls/key.pem").symlink_to(linked / "missing-key")
    cli(linked, "fingerprint", "--synergy", success=False)
    assert (linked / "tls/key.pem").is_symlink() and not (linked / "tls/cert.pem").exists()
print("partial and linked identities fail without regeneration=PASS")

corrupt = root / "corrupt"
shutil.copytree(seed, corrupt)
(corrupt / "tls/key.pem").write_text("invalid PEM\n")
before = digest(corrupt / "tls/key.pem")
cli(corrupt, "fingerprint", "--synergy", success=False)
assert digest(corrupt / "tls/key.pem") == before
print("corrupt PEM fails without regeneration=PASS")

trust = root / "trust"
pins = ["v2:sha256:" + format(number, "064x") for number in range(1, 25)]
children = [subprocess.Popen(command(trust, "trust", "approve", pin),
                            env=environment(trust), stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            for pin in pins]
try:
    for child in children:
        stdout, stderr = child.communicate(timeout=15)
        assert child.returncode == 0, stderr.decode()
finally:
    for child in children:
        if child.poll() is None:
            child.kill()
            child.wait()
for role in ["trusted-servers", "trusted-clients"]:
    assert set((trust / "tls" / role).read_text().splitlines()) == set(pins)
cli(trust, "trust", "revoke", pins[0])
for role in ["trusted-servers", "trusted-clients"]:
    assert set((trust / "tls" / role).read_text().splitlines()) == set(pins[1:])
print("24 concurrent CLI approvals and revoke persist complete sets=PASS")

source = root / "import-source"
source.mkdir()
(source / "trusted-servers").write_text(pins[0] + "\n" + pins[0] + "\nv1:sha1:legacy\n")
before = ((source / "trusted-servers").stat().st_mode, digest(source / "trusted-servers"))
cli(root / "imported", "trust", "import-synergy", str(source))
assert (root / "imported/tls/trusted-servers").read_text() == pins[0] + "\n"
assert ((source / "trusted-servers").stat().st_mode, digest(source / "trusted-servers")) == before
print("Synergy import deduplicates SHA256 and leaves source unchanged=PASS")

if os.name == "posix":
    # Keep the PID across exec so CREATE_NEW meets a pre-existing temporary file.
    collision = root / "collision"
    (collision / "tls").mkdir(parents=True)
    gate = collision / "go"
    shim = "import os,sys,time\nwhile not os.path.exists(sys.argv[1]): time.sleep(.005)\nos.execv(sys.argv[2],sys.argv[2:])"
    child = subprocess.Popen([sys.executable, "-c", shim, str(gate),
                              *command(collision, "trust", "approve", pins[0])],
                             env=environment(collision), stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    existing = collision / "tls" / (".oe-%d-0.tmp" % child.pid)
    existing.write_text("pre-existing sentinel")
    gate.touch()
    try:
        stdout, stderr = child.communicate(timeout=3)
        assert child.returncode != 0 and existing.read_text() == "pre-existing sentinel"
        assert not (collision / "tls/trusted-servers").exists()
    finally:
        if child.poll() is None:
            child.kill()
            child.wait()
    print("failed atomic temporary-file creation preserves existing file=PASS")
print("OFFLINE_RELEASE=PASS artifacts=" + str(root))
