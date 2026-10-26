#!/usr/bin/env python3
"""End-to-end WP2 checks. Runs the real CLI and a real virtual daemon; no mocks.

Writes only synthetic evidence under a fresh private temp directory. Never reads live preferences.
The daemon uses only a private Unix socket and a loopback-only config.
"""
import copy
import argparse
import hashlib
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import socket as net_socket
import tempfile
import time

ROOT = Path(__file__).resolve().parents[2]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--openergy", type=Path, default=ROOT / "target/release/openergy")
parser.add_argument("--port-range", default="25160-25179")
args = parser.parse_args()
first_port, last_port = map(int, args.port_range.split("-"))
if not (1 <= first_port < last_port <= 65535) or first_port <= 24800 <= last_port:
    parser.error("need at least two non-production ports")
reservations = []
for port in range(first_port, last_port + 1):
    candidate = net_socket.socket()
    try:
        candidate.bind(("127.0.0.1", port))
    except OSError:
        candidate.close()
        continue
    reservations.append(candidate)
    if len(reservations) == 2:
        break
if len(reservations) != 2:
    parser.error("need two free ports in --port-range")
PORT, NEXT_PORT = [s.getsockname()[1] for s in reservations]
BIN = args.openergy.resolve()
FIX = ROOT / "crates/config/fixtures"
OUT = Path(tempfile.mkdtemp(prefix="oe-wp2-"))
ENV = dict(os.environ, OPENERGY_CONFIG=str(OUT / "layout.toml"),
           OPENERGY_CONFIG_DIR=str(OUT), OPENERGY_TLS_DIR=str(OUT / "tls"),
           OPENERGY_SOCKET=str(OUT / "oe-wp2.sock"), OPENERGY_DISABLE_DISCOVERY="1",
           XDG_CONFIG_HOME=str(OUT / "xdg-config"), XDG_RUNTIME_DIR=str(OUT / "runtime"))
LOG = (OUT / "verification.log").open("w")
COUNT = 0


def run(*args, ok=True, env=None):
    global COUNT
    command = [str(BIN)] + [str(a) for a in args]
    p = subprocess.run(command, capture_output=True, text=True, cwd=ROOT, timeout=15,
                       env=dict(ENV, **(env or {})))
    LOG.write("$ " + shlex.join(command) + "\n" + p.stdout + p.stderr)
    LOG.flush()
    if (p.returncode == 0) != ok:
        raise RuntimeError("unexpected exit: " + shlex.join(command) + "\n" + p.stdout + p.stderr)
    COUNT += 1
    return p


def imported(source, output):
    p = run("import-synergy", "--from", source, "--out", output, "--json")
    result = json.loads(p.stdout)
    assert output.stat().st_mode & 0o777 == 0o600
    assert "screen" in result["config"] and "screens" not in result["config"]
    return result


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def link(source, edge, sr, target, tr):
    return {"from": source, "edge": edge, "from_range": sr, "to": target, "to_range": tr}


three = imported(FIX / "synergy3", OUT / "config.toml")
one = imported(FIX / "synergy1", OUT / "legacy.toml")
assert three["config"]["role"] == "client"
assert three["config"]["clipboard"]["max_bytes"] == 4096000
assert three["config"]["input"]["scroll_sensitivity"] == 1.25
assert len(three["report"]["trusted_servers"]) == 1
assert three["derived_links"] == [link("studio-pc", "left", [23, 91], "mac-book", [0, 100]), link("mac-book", "right", [0, 100], "studio-pc", [23, 91])]
assert one["config"]["options"]["barrier"]
assert one["config"]["screen"][0]["options"]["half_duplex_num_lock"]
assert one["config"]["screen"][1]["aliases"] == ["laptop"]
assert len(one["report"]["warnings"]) == 5
# Synergy export -> reimport -> export is stable. Barrier's startup dialect is
# an informational comment, not a core setting; import cannot recover that flag.
for stem in ("config", "legacy"):
    first = OUT / (stem + ".conf")
    second = OUT / (stem + "-roundtrip.conf")
    run("export-synergy-conf", "--config", OUT / (stem + ".toml"), "--out", first)
    result = imported(first, OUT / (stem + "-roundtrip.toml"))
    run("export-synergy-conf", "--config", OUT / (stem + "-roundtrip.toml"), "--out", second)
    if stem == "config":
        assert first.read_bytes() == second.read_bytes()
    else:
        assert result["config"]["screen"] == one["config"]["screen"]
        assert result["config"]["switch"] == one["config"]["switch"]
        assert result["config"]["hotkeys"] == one["config"]["hotkeys"]
        assert not result["config"].get("options", {}).get("barrier", False)
    assert result["derived_links"] == (three if stem == "config" else one)["derived_links"]
# No synergy.conf: use db editor ranges, not authoritative overrides.
db_only = OUT / "db-only"
db_only.mkdir(exist_ok=True)
shutil.copyfile(FIX / "synergy3/db.json", db_only / "db.json")
result = imported(db_only, OUT / "db-only.toml")
assert result["config"]["role"] == "off"
assert result["derived_links"] == [link("studio-pc", "left", [22, 91], "mac-book", [0, 100]), link("mac-book", "right", [0, 100], "studio-pc", [22, 91])]
# The import-only service converter retains its historical zero-spacing behavior.
service_db = json.loads((FIX / "synergy3/db.json").read_text())
service_db["data"]["computers"][0]["misc"]["left"] = 200
(db_only / "db.json").write_text(json.dumps(service_db))
service = imported(db_only, OUT / "service-zero-spacing.toml")
assert service["derived_links"] == [link("studio-pc", "left", [22, 91], "mac-book", [0, 100]), link("studio-pc", "right", [22, 91], "mac-book", [0, 100]), link("mac-book", "right", [0, 100], "studio-pc", [22, 91])]
assert any("imported service links can include extra directions" in w for w in service["report"]["warnings"])
(OUT / "service-zero-spacing.json").write_text(json.dumps(service, indent=2))
# Editor object enumeration order must determine the last nonempty corner size.
db = json.loads((FIX / "synergy3/db.json").read_text())
db["data"]["computers"][0]["dead_corners"] = {"top-left": 9, "bottom-right": 17}
(db_only / "db.json").write_text(json.dumps(db))
result = imported(db_only, OUT / "corner-order.toml")
assert result["config"]["screen"][0]["options"]["corner_size_px"] == 17
# Role without INI is obtained from local.json joins.
local_dir = OUT / "local-only"
local_dir.mkdir(exist_ok=True)
for name in ("db.json", "local.json"):
    shutil.copyfile(FIX / "synergy3" / name, local_dir / name)
result = imported(local_dir, OUT / "local-only.toml")
assert result["config"]["role"] == "client" and result["config"]["name"] == "mac-book"
local = json.loads((local_dir / "local.json").read_text())
local["myId"] = "pc"
(local_dir / "local.json").write_text(json.dumps(local))
assert imported(local_dir, OUT / "local-server.toml")["config"]["role"] == "server"
# Parser rejection through the real import command; old output must survive.
valid = (FIX / "synergy1/synergy.conf").read_text()
for label, text in {
    "missing-end": "section: screens\n  studio-pc:\n",
    "unknown-section": "section: unsupported\nend\n",
    "bad-bool": valid.replace("clipboardSharing = true", "clipboardSharing = yes"),
    "bad-range": valid.replace("left(20,80)", "left(80,20)"),
    "unknown-target": valid.replace("= laptop(0,100)", "= absent(0,100)"),
    "overlap": valid.replace("left(20,80) = laptop(0,100)", "left(20,80) = laptop(0,100)\n        left(30,90) = macbook"),
    "duplicate-alias": valid.replace("        laptop\n", "        laptop\n        laptop\n"),
}.items():
    source = OUT / (label + ".conf")
    source.write_text(text)
    before = digest(OUT / "config.toml")
    p = run("import-synergy", "--from", source, "--out", OUT / "config.toml", ok=False)
    assert "line" in p.stderr and digest(OUT / "config.toml") == before
# Supported mouse trigger and fractional percentages retain warnings and core syntax.
extra = valid.replace("keystroke(Control+Alt+l) = lockCursorToScreen(toggle)", "mousebutton(Control+4) = lockCursorToScreen(toggle)")
extra = extra.replace("left(20,80)", "left(20.2,80.2)")
extra = extra.replace("all -bottom-right", "all-bottom-right").replace("switchDelay = 150", "switchDelay = -1")
(OUT / "extra.conf").write_text(extra)
result = imported(OUT / "extra.conf", OUT / "extra.toml")
assert result["config"]["hotkeys"]["lock_to_screen"] == "mousebutton(ctrl+4)"
assert result["config"]["switch"]["delay_ms"] == 0
run("export-synergy-conf", "--config", OUT / "extra.toml", "--out", OUT / "extra-export.conf")
assert "mousebutton(Control+4) = lockCursorToScreen(toggle)" in (OUT / "extra-export.conf").read_text()
# Strict nested TOML diagnostics include source line and unknown field name.
(OUT / "unknown.toml").write_text('[input]\nunknown_setting = true\n')
p = run("export-synergy-conf", "--config", OUT / "unknown.toml", ok=False)
assert "unknown_setting" in p.stderr and "line 2" in p.stderr
empty = OUT / "empty"
empty.mkdir(exist_ok=True)
run("import-synergy", "--from", empty, ok=False)
run("import-synergy", "--unknown", ok=False)

# Output failure removes its own temporary file. Existing destinations survive.
run("export-synergy-conf", "--config", OUT / "config.toml", "--out", empty, ok=False)
assert not list(OUT.glob("empty.tmp.*"))
# Default import location is resolved from HOME without writing live preferences.
home = OUT / "synthetic-home"
preferences = home / "Library/Preferences/Synergy"
preferences.mkdir(parents=True, exist_ok=True)
shutil.copyfile(FIX / "synergy1/synergy.conf", preferences / "synergy.conf")
result = json.loads(run("import-synergy", "--json", env={"HOME": str(home)}).stdout)
assert len(result["config"]["screen"]) == 2

# Step 0 chmods the socket parent; use our own 0700 dir, never /tmp itself.
socket = OUT / "oe-wp2.sock"
live = OUT / "layout.toml"
live.write_text((FIX / "layout/partial.toml").read_text().replace("25020", str(PORT)))
os.chmod(live, 0o600)
if socket.exists():
    raise RuntimeError("refuse to reuse an existing socket")
log = (OUT / "daemon.log").open("w")
reservations[0].close()
daemon = subprocess.Popen([str(BIN), "daemon", "--platform", "virtual", "--socket", str(socket), "--config", str(live), "--foreground"], stdout=log, stderr=log, cwd=ROOT, env=ENV)


def ctl(method, params=None, ok=True):
    args = ["ctl", method, "--socket", socket, "--raw"]
    if params is not None:
        args += ["--json", json.dumps(params, separators=(",", ":"))]
    p = run(*args, ok=ok)
    return json.loads(p.stdout)


try:
    deadline = time.monotonic() + 5
    while not socket.exists():
        if daemon.poll() is not None or time.monotonic() > deadline:
            raise RuntimeError("daemon did not start")
        time.sleep(0.02)
    initial = ctl("config.get")
    # RFC 7386 object-on-null: deleting a missing nested bool must not set it to null.
    ctl("config.patch", {"patch": {"options": {"barrier": None}}})
    (OUT / "partial-get.json").write_text(json.dumps(initial, indent=2))
    # Native zero-spacing rects produce exactly the two facing links, without warnings.
    expected = [link("macbook", "right", [0, 100], "studio-pc", [21, 89]), link("studio-pc", "left", [21, 89], "macbook", [0, 100])]
    assert initial["derived_links"] == expected
    run("export-synergy-conf", "--config", live, "--out", OUT / "native-partial.conf")
    assert imported(OUT / "native-partial.conf", OUT / "native-partial-roundtrip.toml")["derived_links"] == expected
    screens = copy.deepcopy(initial["config"]["screen"])
    screens[0]["links"] = {"right": ""}
    patched = ctl("config.patch", {"patch": {"screen": screens}})
    assert patched["warnings"] == []
    assert ctl("config.get")["derived_links"] == expected[1:]
    # A full-edge explicit target overrides the native partial destination interval.
    screens[0]["links"]["right"] = "studio-pc"
    ctl("config.patch", {"patch": {"screen": screens}})
    assert ctl("config.get")["derived_links"] == [link("macbook", "right", [0, 100], "studio-pc", [0, 100]), expected[1]]
    screens[0].pop("links")
    # Exactly one point of edge tolerance is included; anything beyond it is excluded.
    screens[1]["rect"]["x"] = 1513
    patched = ctl("config.patch", {"patch": {"screen": screens}})
    assert patched["warnings"] == []
    moved = ctl("config.get")
    assert moved["derived_links"] == expected
    (OUT / "moved-get.json").write_text(json.dumps(moved, indent=2))
    screens[1]["rect"]["x"] = 1513.01
    ctl("config.patch", {"patch": {"screen": screens}})
    assert ctl("config.get")["derived_links"] == []
    # Perpendicular overlap must exceed one point, not merely touch that boundary.
    boundary = [{"name": "studio-pc", "rect": {"x": 0, "y": 0, "w": 100, "h": 100}}, {"name": "macbook", "rect": {"x": 100, "y": 99, "w": 100, "h": 100}}]
    patched = ctl("config.patch", {"patch": {"screen": boundary}})
    assert patched["warnings"] == [] and ctl("config.get")["derived_links"] == []
    boundary[1]["rect"]["y"] = 98
    ctl("config.patch", {"patch": {"screen": boundary}})
    assert ctl("config.get")["derived_links"] == [link("studio-pc", "right", [98, 100], "macbook", [0, 2]), link("macbook", "left", [0, 2], "studio-pc", [98, 100])]
    # A real >1-point overlap can still collapse at the frozen integer precision.
    boundary[0]["rect"]["h"] = 10000
    boundary[1]["rect"].update(y=9998, h=10000)
    rounded = ctl("config.patch", {"patch": {"screen": boundary}})
    assert any("rounds to an empty integer percentage range" in w for w in rounded["warnings"])
    assert ctl("config.get")["derived_links"] == []
    exported = run("export-synergy-conf", "--config", live)
    assert "rounds to an empty integer percentage range" in exported.stderr
    (OUT / "rounded-empty.json").write_text(json.dumps(rounded, indent=2))
    boundary[0]["links"], boundary[1]["links"] = {"right": ""}, {"left": ""}
    assert ctl("config.patch", {"patch": {"screen": boundary}})["warnings"] == []
    # Sub-point widths must not generate a backward edge within the 1-point tolerance.
    narrow = [{"name": "studio-pc", "rect": {"x": 0, "y": 0, "w": 0.25, "h": 100}}, {"name": "macbook", "rect": {"x": 0.25, "y": 0, "w": 0.25, "h": 100}}]
    ctl("config.patch", {"patch": {"screen": narrow}})
    assert ctl("config.get")["derived_links"] == [link("studio-pc", "right", [0, 100], "macbook", [0, 100]), link("macbook", "left", [0, 100], "studio-pc", [0, 100])]
    # Sharing a corner creates no directional link.
    corner = [{"name": "studio-pc", "rect": {"x": 0, "y": 0, "w": 100, "h": 100}}, {"name": "macbook", "rect": {"x": 100, "y": 100, "w": 100, "h": 100}}]
    ctl("config.patch", {"patch": {"screen": corner}})
    assert ctl("config.get")["derived_links"] == []
    screens[1]["rect"]["x"] = 1513
    ctl("config.patch", {"patch": {"screen": screens}})
    # Keep the exact file and live config unchanged for every invalid patch.
    errors = []
    overlap = copy.deepcopy(screens)
    overlap[1]["rect"]["x"] = 1400
    for patch in [{"screen": overlap}, {"server": {"typo": True}}, {"server": {"port": 0}}, {"server": {"port": 70000}}, {"screen": [{"name": "studio-pc", "links": {"left": "absent"}}]}, {"input": {"scroll_sensitivity": 0}}, None]:
        before, state = digest(live), ctl("config.get")
        response = ctl("config.patch", {"patch": patch}, ok=False)
        after = digest(live)
        assert before == after and ctl("config.get") == state
        errors.append({"error": response["error"]["code"], "message": response["error"]["message"], "before": before, "after": after})
    (OUT / "invalid-patches.json").write_text(json.dumps(errors, indent=2))
    # RFC 7386 deletes fields and uses nested defaults; absent null keys are harmless.
    ctl("config.patch", {"patch": {"input": {"relative_mouse": True}, "hotkeys": {"lock_to_screen": "ctrl+l"}}})
    result = ctl("config.patch", {"patch": {"input": {"relative_mouse": None}, "hotkeys": None, "unknown_deleted": None}})
    assert result["config"]["input"]["relative_mouse"] is False and result["config"]["hotkeys"] == {}
    # Server port wins, with a visible warning.
    reservations[1].close()
    result = ctl("config.patch", {"patch": {"server": {"port": NEXT_PORT}}})
    assert "server.port overrides the port in server.listen" in result["warnings"]
    # Three-screen L: one full pair and one half/full pair.
    l_screens = [{"name": "studio-pc", "rect": {"x": 0, "y": 0, "w": 1000, "h": 1000}}, {"name": "macbook", "rect": {"x": 1000, "y": 0, "w": 1000, "h": 1000}}, {"name": "linux-box", "rect": {"x": 0, "y": 1000, "w": 500, "h": 1000}}]
    ctl("config.patch", {"patch": {"screen": l_screens}})
    l_result = ctl("config.get")
    assert l_result["derived_links"] == [link("studio-pc", "right", [0, 100], "macbook", [0, 100]), link("studio-pc", "bottom", [0, 50], "linux-box", [0, 100]), link("macbook", "left", [0, 100], "studio-pc", [0, 100]), link("linux-box", "top", [0, 100], "studio-pc", [0, 50])]
    (OUT / "l-get.json").write_text(json.dumps(l_result, indent=2))
    # Explicit adjacent half-open source intervals succeed; overlapping intervals fail.
    for index, screen in enumerate(l_screens):
        screen.pop("rect")
    l_screens[0]["links"] = {"ranges": [{"edge": "right", "from_range": [0, 50], "to": "macbook", "to_range": [0, 100]}, {"edge": "right", "from_range": [50, 100], "to": "linux-box", "to_range": [0, 100]}]}
    ctl("config.patch", {"patch": {"screen": l_screens}})
    before = digest(live)
    l_screens[0]["links"]["ranges"][1]["from_range"][0] = 49
    ctl("config.patch", {"patch": {"screen": l_screens}}, ok=False)
    assert digest(live) == before
finally:
    if daemon.poll() is None:
        try:
            ctl("daemon.stop")
            daemon.wait(timeout=5)
        except Exception:
            daemon.terminate()  # Only the child this script started.
            daemon.wait(timeout=5)
            raise
    log.close()
# Read the persisted TOML through a fresh real CLI process after daemon exit.
run("export-synergy-conf", "--config", live, "--out", OUT / "persisted.conf")
persisted = imported(OUT / "persisted.conf", OUT / "persisted-roundtrip.toml")
assert persisted["derived_links"] == [link("studio-pc", "right", [0, 50], "macbook", [0, 100]), link("studio-pc", "right", [50, 100], "linux-box", [0, 100])]
assert persisted["config"]["server"]["port"] == NEXT_PORT
print("PASS: CLI import/export round trips; JSON and source precedence; strict failures; native edge geometry, Synergy editor fidelity, L layout and ranged overrides; invalid patches preserve file and live state.")
print("invalid patch sha256 before/after: " + errors[0]["before"] + " / " + errors[0]["after"])
LOG.close()
print("Real CLI invocations: " + str(COUNT))
print("Evidence: " + str(OUT / "verification.log"))
