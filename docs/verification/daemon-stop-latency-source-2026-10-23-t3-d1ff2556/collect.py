#!/usr/bin/env python3
"""Read Git source and saved evidence. Never launch a product or open a socket."""
import datetime
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
BASE = "1fd5144628c0494f1ebf561581e0d84fd7231523"
R3 = "b75dfa1b543e4bf6b47858581d4d2cf19aa777e6"
PARENT = "71a40b7c2a5cf22cd0bf18a8b602f9ead51de681"
REVIEW = "ce22c1f37618563dea522c4a04b2248b7fc56e11"
R3_DIR = "docs/verification/macos-idle-resources-r3-2026-10-23/"
REVIEW_PATH = "docs/verification/macos-idle-resources-r3-source-evidence-review-2026-10-23-t3-4021ec78.md"
SOURCE_ROOTS = ["crates/daemon", "crates/net", "crates/session", "crates/ipc",
                "crates/config", "crates/platform", "crates/platform-macos",
                "vendor/mdns-sd", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml"]
SAVED = ["README.md", "measure.py", "evidence-manifest.json", "raw/public-cli.json",
         "raw/cleanup.json", "raw/cleanup-ports-first-observation.json",
         "raw/runtime-first-failure.txt", "raw/cleanup-first-failure.txt",
         "raw/run/peer/openergy.log", "raw/run/peer.log", "raw/exec-barriers.json",
         "raw/runtime-entry.json", "raw/ready.json", "raw/run-command.json"]
EXCERPTS = {
    "crates/daemon/src/runtime.rs": [(48, 54), (77, 89), (1239, 1242), (1392, 1440), (1508, 1664)],
    "crates/daemon/src/ipc_server.rs": [(48, 69), (159, 191), (194, 232), (267, 375)],
    "crates/net/src/reactor.rs": [(1718, 1803)],
    "crates/daemon/src/logger.rs": [(12, 17), (120, 128)],
    "crates/daemon/src/main.rs": [(32, 46), (228, 235)],
    "crates/daemon/src/lifetime.rs": [(1, 6), (41, 47)],
    "crates/session/src/server.rs": [(77, 84)],
    "crates/session/src/client.rs": [(48, 53)],
    "docs/plan/architecture-r2.md": [(587, 600)],
}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def main():
    os.umask(0o077)
    HERE.chmod(0o700)
    started = time.monotonic()
    before_nice = os.getpriority(os.PRIO_PROCESS, 0)
    os.setpriority(os.PRIO_PROCESS, 0, 15)
    calls = []

    def command(argv, output=None):
        result = subprocess.run(argv, cwd=REPO, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=15)
        receipt = dict(argv=argv, exit=result.returncode, stdout_bytes=len(result.stdout),
                       stdout_sha256=sha(result.stdout), stderr=result.stderr.decode())
        if output is not None:
            output.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
            output.write_bytes(result.stdout)
            receipt["stdout_file"] = str(output.relative_to(HERE))
        else:
            receipt["stdout"] = result.stdout.decode()
        calls.append(receipt)
        if result.returncode:
            raise RuntimeError(receipt)
        return result.stdout

    def git(*args, output=None):
        return command(["git", *args], output=output)

    (HERE / "raw").mkdir(mode=0o700)
    head = git("rev-parse", "HEAD").decode().strip()
    if head != BASE:
        raise RuntimeError(("wrong source base", head))
    identity = command(["/bin/ps", "-p", str(os.getpid()), "-o",
                        "pid=,ppid=,pgid=,uid=,ni=,lstart=,comm="]).decode()
    scope = dict(mode="source-and-saved-evidence-only", new_product_processes=0,
                 new_compiler_processes=0, new_daemon_flows=0, daemon_stop_requests=0,
                 socket_operations=0, port_probes=0, builds=0, trace_overlay=None,
                 temporary_runtime_root=None, force_stop_actions=0,
                 product_priority="not applicable: no product or compiler launched",
                 collector=dict(pid=os.getpid(), ppid=os.getppid(), identity_ps=identity,
                                nice_before=before_nice, absolute_nice_requested=15,
                                nice_after=os.getpriority(os.PRIO_PROCESS, 0)),
                 reason="Source establishes fall-through to a poll after stop dispatch; no fresh runtime is needed for that conclusion.")
    write_json(HERE / "scope.json", scope)

    origins = []
    for path in SAVED:
        out = HERE / "raw" / "saved-r3" / path
        content = git("show", R3 + ":" + R3_DIR + path, output=out)
        origins.append(dict(file=str(out.relative_to(HERE)), commit=R3,
                            source_path=R3_DIR + path, bytes=len(content), sha256=sha(content),
                            transform="none"))
    content = git("show", REVIEW + ":" + REVIEW_PATH, output=HERE / "raw" / "saved-review.md")
    origins.append(dict(file="raw/saved-review.md", commit=REVIEW, source_path=REVIEW_PATH,
                        bytes=len(content), sha256=sha(content), transform="none"))
    review_disk = Path("/Users/serge/.t3/worktrees/openergy/t3-4021ec78") / REVIEW_PATH
    review_disk_matches = review_disk.read_bytes() == content

    # This is an archive reader, not a product test. No archived .py file is loaded.
    bindings = []
    tracked = git("ls-files", "-z", "--", *SOURCE_ROOTS).decode().split("\0")
    for path in filter(None, tracked):
        data = (REPO / path).read_bytes()
        bindings.append(dict(path=path, bytes=len(data), sha256=sha(data)))
    comparisons = []
    for ref in [BASE, R3, PARENT]:
        git("diff", "--exit-code", ref, "--", *SOURCE_ROOTS)
        comparisons.append(dict(ref=ref, exact_worktree_diff_exit=0))
    for path, ranges in EXCERPTS.items():
        committed = git("show", BASE + ":" + path)
        if committed != (REPO / path).read_bytes():
            raise RuntimeError(("source differs", path))
        lines = committed.decode().splitlines()
        text = "source=" + path + "\ncommit=" + BASE + "\nsha256=" + sha(committed) + "\n"
        for start, end in ranges:
            text += "\n" + "\n".join(f"{i}: {lines[i-1]}" for i in range(start, end + 1)) + "\n"
        out = HERE / "source" / (path.replace("/", "__") + ".txt")
        out.parent.mkdir(exist_ok=True, mode=0o700)
        out.write_text(text)
    write_json(HERE / "origins.json", origins)
    write_json(HERE / "source-bindings.json", dict(base=BASE, saved_runtime_evidence=R3,
               runtime_source_parent=PARENT, source_review=REVIEW,
               review_worktree_bytes_match_commit=review_disk_matches,
               compared_paths=SOURCE_ROOTS, comparisons=comparisons, files=bindings,
               limitation="Source identity only. No cached executable was opened, hashed, or launched."))

    def saved_json(path):
        return json.loads((HERE / "raw" / "saved-r3" / path).read_text())

    cli = saved_json("raw/public-cli.json")
    stop = [row for row in cli if row["label"] == "normal-peer-stop"]
    cleanup = saved_json("raw/cleanup.json")
    ready = saved_json("raw/ready.json")
    barriers = saved_json("raw/exec-barriers.json")
    peer_log = (HERE / "raw" / "saved-r3" / "raw/run/peer/openergy.log").read_text()
    shutdown = next(int(line.split()[0]) for line in peer_log.splitlines() if "shutdown: leave and goodbye" in line)
    stopped = next(int(line.split()[0]) for line in peer_log.splitlines() if "daemon stopped" in line)
    signal = next(row for row in cleanup["signals"] if row["group"] == 51089)
    ports = saved_json("raw/cleanup-ports-first-observation.json")
    analysis = dict(
        evidence_kind="audited historical records; not independent runtime reproduction",
        stop_cli=stop,
        stop_timestamp_fields=dict(command_start=None, command_end=None, wire_reply=None,
                                   first_stop_store=None, first_runtime_guard_observation=None,
                                   poll_enter=None, poll_return=None, exact_flush_start=None,
                                   exact_process_exit=None),
        saved_wall_time_events_unix_ms=dict(shutdown_entry_log=shutdown,
                                           cleanup_sigterm_receipt=signal["unix_ms"],
                                           post_join_stopped_log=stopped),
        derived_wall_deltas_ms=dict(shutdown_entry_to_sigterm=round(signal["unix_ms"] - shutdown, 3),
                                   shutdown_entry_to_post_join_log=stopped - shutdown,
                                   sigterm_to_post_join_log=round(stopped - signal["unix_ms"], 3)),
        timing_limits=["Daemon log uses integer SystemTime Unix milliseconds, not a monotonic trace.",
                       "CLI receipt has elapsed_seconds only; no absolute start/end timestamps.",
                       "Shutdown entry precedes the flush; the stopped log follows joins and precedes return/drops/exit.",
                       "No clock-adjustment or logging-overhead bound was established.",
                       "The later SIGTERM can affect flush polls; this is not an unsignalled exit-latency measurement."],
        saved_stop_observation=dict(peer_exit=next(p for p in cleanup["processes"] if p["name"] == "peer"),
                                    normal_one_second_wait_failed=True, overall_passed=cleanup["passed"]),
        saved_process_identities=[dict(role=x["role"], accepted=x["accepted"]) for x in barriers],
        recorded_daemon_sha256=ready["daemon_sha256"], live_binary_identity_verified=False,
        port_observations=ports,
        port_causation="Unresolved. lsof ran before bind; the saved netstat ran after bind failed. TIME_WAIT is not a listener or a proved cause.",
        conclusion="Source establishes an avoidable post-dispatch poll opportunity; historical records do not measure the exact wait or prove its sole cause.")
    write_json(HERE / "analysis.json", analysis)
    write_json(HERE / "collection.json", dict(invocation="python3 " + str(Path(__file__).relative_to(REPO)),
               local_time=datetime.datetime.now().astimezone().isoformat(),
               elapsed_seconds=time.monotonic() - started, commands=calls))
    print(json.dumps(dict(source_files=len(bindings), copied_origins=len(origins),
          source_comparisons=comparisons, review_disk_matches=review_disk_matches,
          stop_calls=len(stop), saved_wall_deltas_ms=analysis["derived_wall_deltas_ms"],
          new_flows=0, new_builds=0, scope="source-only", elapsed_seconds=time.monotonic()-started), indent=2))


if __name__ == "__main__":
    main()
