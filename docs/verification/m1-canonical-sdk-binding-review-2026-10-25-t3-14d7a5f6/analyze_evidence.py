"""Read retained strings as data. No retained Python/AST execution or import."""
import codecs
import base64
import hashlib
import io
import json
import re
import posixpath
import shlex
import subprocess
import tokenize
from pathlib import Path
from read_evidence import CANDIDATE, REVIEW, snapshot


def digest(b):
    return hashlib.sha256(b).hexdigest()


def literal_data(text):
    tokens = list(tokenize.generate_tokens(io.StringIO(text).readline))
    tokens = [t for t in tokens if t.type not in (tokenize.NEWLINE, tokenize.NL, tokenize.ENDMARKER)]
    pos = 0

    def read():
        nonlocal pos
        t = tokens[pos]
        pos += 1
        if t.type == tokenize.STRING:
            assert t.string[0] in "'\"" and t.string[-1] == t.string[0]
            assert t.string.isascii()
            return codecs.decode(t.string[1:-1], "unicode_escape")
        if t.type == tokenize.NUMBER and t.string.isdecimal():
            return int(t.string)
        if t.string in ("[", "{"):
            closing = "]" if t.string == "[" else "}"
            result = [] if closing == "]" else {}
            while tokens[pos].string != closing:
                key = read()
                if closing == "}":
                    assert tokens[pos].string == ":"
                    pos += 1
                    assert key not in result
                    result[key] = read()
                else:
                    result.append(key)
                if tokens[pos].string != closing:
                    assert tokens[pos].string == ","
                    pos += 1
            pos += 1
            return result
        raise ValueError("Nonliteral retained token rejected")

    result = read()
    assert pos == len(tokens)
    return result


data, manifest = snapshot()
(REVIEW / "private-snapshot-final.json").write_text(json.dumps(manifest, indent=2) + "\n")
phase = data["source-file-phase.py.txt"].decode()
encoded = literal_data(next(x[len("FILES="):] for x in phase.splitlines() if x.startswith("FILES=")))
files = {name: bytes.fromhex(text) for name, text in encoded.items()}
print("MATERIALIZED_INPUTS", json.dumps([{"name": n, "bytes": len(v), "sha256": digest(v)} for n, v in files.items()], indent=2))
old = Path("docs/verification/m1-next-reviewed-ordinary-flow-2026-10-25-t3-a8a4ece5")
for n in ("file-qualification.recipe.py.txt", "qualification.recipe.json", "metadata.recipe.json", "artifact-receipt.template.json"):
    assert files[n] == (old / n).read_bytes()
operation = json.loads((old / "next-operation.json").read_bytes())
checks = json.loads(files["source-checks.json"])
materialization = json.loads(Path(operation["materialization"]["manifest"]["path"]).read_bytes())
print("MATERIALIZATION_SHAPE", {k: type(v).__name__ for k, v in materialization.items()})
print("CHECKS_SHAPE", {k: type(v).__name__ for k, v in checks.items()})
for n, value in files.items():
    if n.endswith(".py"):
        print("MODULE", n, len(value), digest(value))
helper = files["file-qualification.recipe.py.txt"].decode().splitlines()
for i in [49, 50, 144, 145, 146, 147]:
    print("HELPER_FAILURE_CONTEXT", i, helper[i - 1])
sourceguard = json.loads(data["source-files-once.guard"])
assert sourceguard["source_sha256"] == digest(data["source-file-phase.py.txt"])
assert sourceguard["source_bytes"] == len(data["source-file-phase.py.txt"])
executor = data["source-file-executor.py"].decode()
for line in executor.splitlines():
    if line.startswith("argv="):
        assert literal_data(line[len("argv="):]) == sourceguard["argv"]
        print("EXECUTOR_GUARD_ARGV_EQUAL", True)
remote = json.loads(data["retained-source-file-failure/source-files-os-exit.json"])
assert remote == json.loads(data["source-files.stdout.bin"])
parent = json.loads(data["source-files-parent-result.json"])
for r in [remote, parent]:
    assert r["first_error"] is None and r["before_deadline"] and all(r["eof"].values())
    assert r["status"] == "FAILED_UNVERIFIED"
    assert r["started_monotonic"] < r["exit_observed_monotonic"] < r["deadline_monotonic"]
assert remote["actual_helper_exit"] == parent["actual_ssh_exit"] == 1
for stream in ("stdout", "stderr"):
    assert parent["streams"][stream] == {"bytes": len(data["source-files." + stream + ".bin"]), "sha256": digest(data["source-files." + stream + ".bin"])}
guard = json.loads(data["sdk-exact-paths-once.guard"])
command = shlex.split(guard["argv"][-1])
print("QUERY_COMMAND_TOKEN_COUNT", len(command))
assert "-c" in command
query = command[command.index("-c") + 1]
assert digest(query.encode()) == guard["query_sha256"]
print("QUERY_SOURCE_SHA256", digest(query.encode()))
print("QUERY_SOURCE_TEXT_BEGIN")
print(query)
print("QUERY_SOURCE_TEXT_END")
raw = data["sdk-exact-paths.stdout.bin"]
assert raw == (CANDIDATE / "sdk-path-observations.json").read_bytes()
observations = json.loads(raw)
recipe = json.loads(files["qualification.recipe.json"])
newrecipe = json.loads((CANDIDATE / "qualification.recipe.json").read_bytes())
assert len(observations["headers"]) == len(recipe["sdk_headers"]) == 10
for before, after, observed in zip(recipe["sdk_headers"], newrecipe["sdk_headers"], observations["headers"]):
    assert before["path"] == observed["requested_path"]
    assert after["path"] == observed["resolved_path"]
    for k in ["sha256", "bytes"]:
        assert before[k] == after[k] == observed[k]
    assert observed["mode"] == "0o644" and observed["uid"] == 0
queryresult = json.loads(data["sdk-exact-paths-result.json"])
assert queryresult["actual_ssh_exit"] == 0 and queryresult["before_deadline"]
assert all(queryresult["eof"].values()) and queryresult["first_error"] is None
assert queryresult["started_monotonic"] == guard["started_monotonic"]
assert queryresult["deadline_monotonic"] == guard["deadline_monotonic"]
assert queryresult["started_monotonic"] < queryresult["exit_observed_monotonic"] < queryresult["deadline_monotonic"]
retention = json.loads(data["failure-retention.stdout.bin"])
print("RETENTION_SHAPE", {k: type(v).__name__ for k, v in retention.items()})
for key, value in retention.items():
    if isinstance(value, list):
        print("RETENTION_RECORD_SHAPES", [{k: type(v).__name__ for k, v in r.items()} for r in value])
print("SNAPSHOT", {k: v for k, v in manifest.items() if k != "records"})

# Final bounded pass: compare all retained input bytes as data and retain only
# hashes and findings. No retained source is imported, compiled or executed.
initial = json.loads((REVIEW / "private-snapshot-initial.json").read_text())
middle = json.loads((REVIEW / "private-snapshot-middle.json").read_text())
assert initial == middle == manifest
modules = []
for module in materialization["modules"]:
    name = module["destination"]
    check = checks["python_files"][name]
    path = posixpath.normpath((Path(operation["materialization"]["manifest"]["path"]).parent / module["source"]).as_posix())
    tracked = subprocess.check_output(["git", "show", operation["materialization"]["commit"] + ":" + str(Path(path))])
    assert files[name] == tracked
    assert len(tracked) == check["bytes"]
    assert len(tracked.splitlines()) == check["lines"]
    assert digest(tracked) == check["sha256"] == module["sha256"]
    modules.append({"name": name, "bytes": len(tracked), "sha256": digest(tracked)})
assert len(modules) == 10
assert {n for n in files if n.endswith(".py")} == {m["name"] for m in modules}
for kind in ("source_checks", "limits", "coverage_policy", "build_source"):
    row = operation["materialization"][kind]
    name = {"source_checks": "source-checks.json", "limits": "limits.json", "coverage_policy": "coverage-policy.json", "build_source": "build-source-bindings.json"}[kind]
    tracked = subprocess.check_output(["git", "show", row.get("commit", operation["materialization"]["commit"]) + ":" + row["path"]])
    assert tracked == files[name] and digest(tracked) == row["sha256"]

for line in executor.splitlines():
    if "=[" in line or "= [" in line:
        right = line.split("=", 1)[1]
        try:
            parsed = literal_data(right)
        except (ValueError, AssertionError, IndexError, tokenize.TokenError):
            continue
        if parsed == sourceguard["argv"]:
            print("EXECUTOR_GUARD_ARGV_EQUAL", True)
            executor_argv_equal = True
            break
else:
    executor_argv_equal = False

def masked_line(line):
    try:
        tokens = list(tokenize.generate_tokens(io.StringIO(line).readline))
    except tokenize.TokenError:
        return "[incomplete text line]"
    spans = [(t.start[1], t.end[1]) for t in tokens if t.type == tokenize.STRING]
    for a, b in reversed(spans):
        line = line[:a] + "'[string withheld]'" + line[b:]
    return line

for n, source, selected in [("executor", executor, [4, 13, 15, 20, 21, 22, 41, 48]),
                            ("phase", phase, [1, 31, 32, 33, 34, 36])]:
    for i, line in enumerate(source.splitlines(), 1):
        if i in selected:
            print("MASKED_TEXT", n, i, masked_line(line))
for name, record in retention.items():
    print("RETENTION_FIELDS", name, list(record))
    rawretained = data["retained-source-file-failure/" + name]
    assert record["bytes"] == len(rawretained) and record["sha256"] == digest(rawretained)
    for key in ("hex", "data_hex", "content_hex"):
        if key in record:
            assert bytes.fromhex(record[key]) == rawretained
    for key in ("b64", "base64", "data_b64", "content_b64", "data_base64"):
        if key in record:
            assert base64.b64decode(record[key], validate=True) == rawretained
assert command[:5] == ["/usr/bin/python3", "-I", "-S", "-B", "-c"]
queryrows = literal_data(next(x[len("rows="):] for x in query.splitlines() if x.startswith("rows=")))
assert queryrows == recipe["sdk_headers"]

result = {
    "schema": "openergy.canonical-sdk-binding-review.v1",
    "candidate_commit": "cfe7721ec08adfda2352d903200b33f921472836",
    "p1_count": 0, "p2_count": 0, "runtime": "UNVERIFIED",
    "materialized_inputs": [{"name": n, "bytes": len(v), "sha256": digest(v)} for n, v in files.items()],
    "modules": modules,
    "executor_guard_argv_equal": executor_argv_equal,
    "final_private_hashes_preserved": True,
    "unique_private_regular_files": len(manifest["records"]),
    "private_regular_file_opens_cumulative": 60,
    "private_content_bytes_cumulative": 2067627,
    "private_max_regular_file_opens": 64, "private_max_content_bytes": 8388608,
    "query_source_sha256": digest(query.encode()),
    "query_raw_stdout_sha256": digest(raw),
    "query_raw_stdout_equal_candidate_observations": True,
    "actual_helper_exit": 1, "actual_source_ssh_exit": 1,
    "actual_query_ssh_exit": 0,
    "parent_and_remote_first_error": None,
    "source_eof_both": True, "query_eof_both": True,
    "source_and_query_deadline_records_hold": True,
    "auxiliary_resolved_paths": [], "raw_header_records": 10,
    "sdk_header_total_bytes": sum(x["bytes"] for x in observations["headers"]),
    "new_execution": False,
}
(REVIEW / "evidence-checks.json").write_text(json.dumps(result, indent=2) + "\n")
print("FINAL_DATA_CHECKS_WRITTEN")
