#!/usr/bin/env python3
"""Exercise the real config CLI with private paths and synthetic settings only.

No daemon or network connection starts here. C1/C2 cover daemon startup through
the external E2E runner. Artifacts stay in the printed private temp directory.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time


ROOT = Path(__file__).resolve().parents[2]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--openergy", type=Path, default=ROOT / "target/release/openergy")
parser.add_argument("--port", type=int, default=25160)
args = parser.parse_args()
if not 25160 <= args.port <= 25179:
    parser.error("--port must be in the assigned 25160-25179 range")
OUT = Path(tempfile.mkdtemp(prefix="oe-config-cli-"))
BIN = args.openergy.resolve()
ENV = dict(os.environ, OPENERGY_CONFIG=str(OUT / "selected.toml"),
           OPENERGY_CONFIG_DIR=str(OUT), OPENERGY_TLS_DIR=str(OUT / "tls"),
           OPENERGY_SOCKET=str(OUT / "ctl.sock"), OPENERGY_DISABLE_DISCOVERY="1",
           HOME=str(OUT / "home"), XDG_CONFIG_HOME=str(OUT / "xdg-config"),
           XDG_RUNTIME_DIR=str(OUT / "runtime"), APPDATA=str(OUT / "appdata"))
COUNT = 0


def run(*argv, ok=True, env=None, cwd=OUT):
    global COUNT
    result = subprocess.run([str(BIN), *map(str, argv)], env=env or ENV,
                            cwd=cwd, capture_output=True, text=True, timeout=30)
    COUNT += 1
    if (result.returncode == 0) != ok:
        raise AssertionError(f"CLI exit {result.returncode}: {result.stderr}")
    return result


def config(name):
    return (f'version = 1\nrole = "off"\nname = "{name}"\n'
            f'[server]\nlisten = "127.0.0.1:{args.port}"\nport = {args.port}\n'
            f'[[screen]]\nname = "{name}"\n')


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


selected = OUT / "selected.toml"
selected.write_text(config("selected"))
# A synthetic platform default distinguishes a path-selection failure without
# ever reading the user's actual default settings.
for fallback in (OUT / "home/Library/Application Support/Openergy/config.toml",
                 OUT / "xdg-config/openergy/config.toml",
                 OUT / "appdata/Openergy/config.toml"):
    fallback.parent.mkdir(parents=True, exist_ok=True)
    fallback.write_text(config("fallback"))
assert "    selected:" in run("export-synergy-conf").stdout
override = OUT / "override.toml"
override.write_text(config("override"))
assert "    override:" in run("export-synergy-conf", "--config", override).stdout
assert "    override:" in run("export-synergy-conf", "--config", override,
                                 env=dict(ENV, OPENERGY_CONFIG="")).stdout
assert "OPENERGY_CONFIG is empty" in run("export-synergy-conf", ok=False,
                                           env=dict(ENV, OPENERGY_CONFIG="")).stderr

# CLI option failures must leave an existing document untouched.
destination = OUT / "output.conf"
destination.write_text("previous complete file\n")
before = digest(destination)
for argv in (("--out",), ("--out", "--config", str(selected)),
             ("--out=",), ("--out", str(destination), "--out", str(destination)),
             ("--unknown",), ("--json",), ("unexpected-positional",)):
    run("export-synergy-conf", *argv, ok=False)
    assert digest(destination) == before
run("export-synergy-conf", f"--config={selected}", "--out=relative output.conf")
assert (OUT / "relative output.conf").stat().st_mode & 0o777 == 0o600
blocked = OUT / "blocked"
blocked.mkdir()
run("export-synergy-conf", "--out", blocked, ok=False)
assert not list(OUT.glob("blocked.tmp.*"))
run("export-synergy-conf", "--out", destination)
assert destination.stat().st_mode & 0o777 == 0o600
assert "selected:" in destination.read_text()

# Strict TOML defaults and validation run through the real export command.
invalid = OUT / "invalid.toml"
for text in (config("invalid").replace("version = 1", "version = 2"),
             config("invalid") + "[input]\nscroll_sensitivity = nan\n",
             config("invalid") + "[input]\nscroll_sensitivity = inf\n",
             config("invalid") + "[input]\nscroll_sensitivity = 0.0\n",
             config("invalid") + "[client]\nheartbeat_ms = 2147483648\n",
             config("invalid") + "[clipboard]\nmax_bytes = 67108865\n",
             config("invalid") + "[clipboard]\nmax_bytes = 9223372036854775807\n",
             config("invalid") + "[input]\nunknown = true\n"):
    invalid.write_text(text)
    before = digest(destination)
    run("export-synergy-conf", "--config", invalid, "--out", destination, ok=False)
    assert digest(destination) == before
invalid.write_text(config("invalid") + "[clipboard]\nmax_bytes = 268435456\n")
assert "supported maximum of 64 MiB" in run("export-synergy-conf", "--config", invalid,
                                             "--out", destination, ok=False).stderr
assert digest(destination) == before

# Host-only IPv4, DNS and IPv6 settings must retain the separate Synergy port.
# These are offline imports; no endpoint is resolved or contacted.
source = OUT / "import"
source.mkdir()
(source / "synergy.conf").write_text("section: screens\n    client:\nend\n")
for argv in (("--json", "--json"), ("--from=",), ("--json=true",),
             ("--apply",), ("unexpected-positional",)):
    before = digest(selected)
    run("import-synergy", "--from", source, "--out", selected, *argv, ok=False)
    assert digest(selected) == before
(source / "synergy.conf").write_text(
    "section: screens\n    client:\nend\nsection: options\n"
    "clipboardSharingSize = 65537\nend\n")
assert "supported maximum of 64 MiB" in run("import-synergy", "--from", source,
                                             "--out", selected, ok=False).stderr
assert digest(selected) == before
(source / "synergy.conf").write_text("section: screens\n    client:\nend\n")
for host, expected in (("127.0.0.1", f"127.0.0.1:{args.port}"),
                       ("example.invalid", f"example.invalid:{args.port}"),
                       ("::1", f"[::1]:{args.port}"),
                       ("[::1]", f"[::1]:{args.port}"),
                       (f"127.0.0.1:{args.port + 1}", f"127.0.0.1:{args.port + 1}"),
                       (f"[::1]:{args.port + 1}", f"[::1]:{args.port + 1}")):
    (source / "synergy-core.conf").write_text(
        f"[core]\ncoreMode=1\ncomputerName=client\nport={args.port}\n"
        f"[client]\nremoteHost={host}\n")
    result = json.loads(run("import-synergy", "--from", source, "--json").stdout)
    assert result["config"]["client"]["remote"] == expected

# db.json + local.json without INI uses developers.corePort too.
(source / "synergy-core.conf").unlink()
(source / "synergy.conf").unlink()
db = {"version": {"schemaVersion": 17}, "data": {
    "computers": [{"id": "s", "name": "server"}, {"id": "c", "name": "client"}],
    "mainID": {"id": "s"}, "developers": {"corePort": args.port}}}
(source / "db.json").write_text(json.dumps(db))
(source / "local.json").write_text(json.dumps(
    {"version": 7, "myId": "c", "local_computers": {"s": "127.0.0.1"}}))
result = json.loads(run("import-synergy", "--from", source, "--json").stdout)
assert result["config"]["client"]["remote"] == f"127.0.0.1:{args.port}"

# Invalid identity joins and non-document inputs fail before writing output.
preserved = digest(selected)
original_db = json.dumps(db)
for identities in (("same", "same"), ("s", "")):
    for computer, identity in zip(db["data"]["computers"], identities):
        computer["id"] = identity
    (source / "db.json").write_text(json.dumps(db))
    run("import-synergy", "--from", source, "--out", selected, ok=False)
    assert digest(selected) == preserved
(source / "db.json").write_text(original_db)
(source / "local.json").write_text("[]")
assert "expected object" in run("import-synergy", "--from", source,
                                  "--out", selected, ok=False).stderr
assert digest(selected) == preserved
if hasattr(os, "mkfifo"):
    fifo = source / "synergy.conf"
    os.mkfifo(fifo, 0o600)
    assert "expected a regular file" in run("import-synergy", "--from", source,
                                              "--out", selected, ok=False).stderr
    fifo.unlink()
    assert digest(selected) == preserved

# Kill an owned real CLI after it opens its replacement file. Either whole old
# or whole new output is allowed if rename wins the race; partial output is not.
large = OUT / "large.toml"
large.write_text(config("large") + 'aliases = ["' + "a" * (8 * 1024 * 1024) + '"]\n')
complete = OUT / "complete.conf"
run("export-synergy-conf", "--config", large, "--out", complete)
expected_hash = digest(complete)
victim = OUT / "crash.conf"
observed = False
for attempt in range(5):
    victim.write_text("previous complete file\n")
    old_hash = digest(victim)
    with (OUT / f"writer-{attempt}.stderr").open("w") as stderr:
        writer = subprocess.Popen([str(BIN), "export-synergy-conf", "--config", str(large),
                                   "--out", str(victim)], env=ENV, cwd=OUT,
                                  stdout=subprocess.DEVNULL, stderr=stderr)
        try:
            deadline = time.monotonic() + 30
            pattern = f"crash.conf.tmp.{writer.pid}.*"
            while writer.poll() is None and time.monotonic() < deadline:
                if list(OUT.glob(pattern)):
                    writer.send_signal(signal.SIGKILL)
                    writer.wait(timeout=5)
                    observed = writer.returncode == -signal.SIGKILL
                    break
                time.sleep(0.0005)
        finally:
            if writer.poll() is None:
                writer.kill()
            writer.wait(timeout=5)
    assert digest(victim) in (old_hash, expected_hash)
    if observed:
        break
assert observed, "could not observe and kill a live replacement writer"
run("export-synergy-conf", "--config", large, "--out", victim)
assert digest(victim) == expected_hash
assert victim.stat().st_mode & 0o777 == 0o600

print(f"PASS: {COUNT} real CLI invocations; path precedence, CLI failures, ports, atomic replacement and killed-writer recovery")
print(f"Evidence: {OUT}")
