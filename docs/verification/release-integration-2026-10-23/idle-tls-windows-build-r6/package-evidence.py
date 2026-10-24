"""Package existing R6 receipts locally. This script never runs a remote command.

Run from any directory. Exact raw files remain private. It fixes the first
packaging check's known forward-slash profile omission in derived copies.
"""
from collections import Counter
import ast
import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent

def sha(data):
    return hashlib.sha256(data).hexdigest()

def save(name, value):
    (HERE / name).write_text(json.dumps(value, indent=2) + "\n")

private = Path(json.loads((HERE / "private-evidence-location.json").read_text())["private_root"])
state = json.loads((private / "state.json").read_text())
profile = state["profile"]
replacements = [(profile.replace("\\", "\\\\"), "<PROFILE_JSON>"),
                (profile, "<PROFILE>"),
                (profile.replace("\\", "/"), "<PROFILE_FORWARD_SLASH>"),
                (str(private), "<LOCAL_PRIVATE>")]
ledger = json.loads((HERE / "copy-ledger.json").read_text())
for item in ledger["files"]:
    original = (private / item["path"]).read_bytes()
    assert len(original) == item["original_bytes"] and sha(original) == item["original_sha256"]
    dest = HERE / "evidence" / item["path"]
    if not item["committed_copy"]:
        assert not dest.exists()
        continue
    copied = dest.read_bytes()
    assert sha(copied) == item["copy_sha256"]
    text = copied.decode("utf-8")
    for old, new in replacements:
        text = text.replace(old, new)
    corrected = text.encode("utf-8")
    if corrected != copied:
        item["first_derived_copy_bytes"] = len(copied)
        item["first_derived_copy_sha256"] = sha(copied)
        dest.write_bytes(corrected)
        item["copy_bytes"] = len(corrected)
        item["copy_sha256"] = sha(corrected)
        item["byte_equal"] = corrected == original
        item["local_packaging_correction"] = "known Windows profile prefix with forward slashes masked"
ledger["method"] += "; package-evidence.py also masks the known profile with forward slashes"
save("copy-ledger.json", ledger)

# Original snapshot hashes give a concise, exact before/after equality binding.
before = (private / "downloaded/shared-before.json").read_bytes()
after = (private / "downloaded/shared-after.json").read_bytes()
assert before == after
summary = json.loads((HERE / "shared-input-summary.json").read_text())
assert summary["original_before_sha256"] == sha(before)
assert summary["original_after_sha256"] == sha(after)
assert summary["original_before_bytes"] == len(before) == len(after)

manifest = json.loads((HERE / "source-manifest.json").read_text())
source_before = json.loads((private / "downloaded/source-before.json").read_text())
source_after = json.loads((private / "downloaded/source-after.json").read_text())
assert source_before == source_after
assert len(manifest["files"]) == source_before["files"] == 558
assert manifest["source_commit"] == "06b9a139ec855a6f697ded393fa4c2c894110613"
assert manifest["module_sha256"] == source_before["module_sha256"] == "9cd3475f9c13c8e28ea54e0f344b55ae795267b22ea1676c45e5659a64f1f1d4"
commands = [json.loads(p.read_text()) for p in (private / "raw").glob("*attempt-*.json")]
assert len(commands) == 7
assert all(c["exit_code"] == 0 and not c["timed_out"] for c in commands)
controller_sha = sha((HERE / "controller.py").read_bytes())
assert all(c["controller_sha256"] == controller_sha for c in commands)
build = json.loads((private / "downloaded/build-result.json").read_text())
postflight = json.loads((private / "downloaded/postflight.json").read_text())
assert build["passed"] and postflight["passed"]
assert [s["name"] for s in build["steps"]] == ["clippy", "release"]
assert all(s["exit_code"] == 0 and not s["timed_out"] and not s["stopped"] for s in build["steps"])
artifact = private / "downloaded/target/x86_64-pc-windows-msvc/release/examples/dns_queue_cancel_c4806a3d.exe"
assert artifact.stat().st_size == build["artifact"]["bytes"]
assert sha(artifact.read_bytes()) == build["artifact"]["sha256"]
samples = []
races = {}
for name in ("clippy", "release"):
    samples += json.loads((private / f"downloaded/{name}-processes.json").read_text())
    races[name] = len(json.loads((private / f"downloaded/{name}-observation-races.json").read_text()))
assert len(samples) == 104
assert all(s["priority"] == "BelowNormal" and s["session"] == 0 for s in samples)
assert all(any(s["name"] == n for s in samples) for n in ("cargo.exe", "rustc.exe", "link.exe"))
result = json.loads((HERE / "result.json").read_text())
assert result["passed"] and result["cleanup"]["owned_root_removed"]
audit = json.loads((HERE / "build-only-stop-audit.json").read_text())
audit["local_post_build_packaging_errors"] = 1
audit["local_post_build_packaging_corrections"] = 1
audit["remote_or_compiler_retries_for_packaging"] = 0
audit["sampled_process_counts"] = dict(Counter(s["name"] for s in samples))
audit["process_observation_races"] = races
audit["observation_limit"] = "sampling records observed processes; it does not claim every short-lived process was observed"
audit["executed_controller_sha256"] = controller_sha
save("build-only-stop-audit.json", audit)

# Scan derived evidence, not the preserved source of the original checking code.
leaks = []
needles = [old for old, _ in replacements]
for path in (HERE / "evidence").rglob("*"):
    if path.is_file() and any(n in path.read_text() for n in needles):
        leaks.append(str(path.relative_to(HERE)))
assert not leaks, leaks
for path in HERE.glob("*.py"):
    ast.parse(path.read_text())
save("packaging-verification.json", {
    "passed": True, "packaging_script_sha256": sha(Path(__file__).read_bytes()),
    "remote_commands": 0, "compiler_commands": 0, "artifact_executed": False,
    "exact_raw_bytes_retained_privately": True, "private_prefix_scan_passed": True,
    "source_before_after_equal": True, "shared_snapshot_bytes_equal": True,
    "original_snapshot_bytes": len(before), "all_committed_copy_hashes_match_ledger": True,
    "actual_controller_identity_matches_all_seven_receipts": True,
    "first_local_packaging_error_preserved": "local-packaging-first-failure/",
})
print("Local evidence package verified; no remote or compiler commands repeated.")
print("Observed process counts:", audit["sampled_process_counts"])
print("Observation races:", races)
