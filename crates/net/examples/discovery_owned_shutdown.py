#!/usr/bin/env python3
"""Inspect a real, still-live child's owned worker and UDP sockets on macOS."""
import argparse
import json
import pathlib
import select
import subprocess
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=pathlib.Path)
    parser.add_argument("new_report_dir", type=pathlib.Path)
    parser.add_argument("mdns_port", type=int)
    parser.add_argument("service_port", type=int)
    args = parser.parse_args()
    if not all(25080 <= p <= 25099 for p in (args.mdns_port, args.service_port)):
        parser.error("requires assigned ports 25080..=25099")
    if args.mdns_port == args.service_port:
        parser.error("ports must differ")
    args.new_report_dir.mkdir()
    reports = []
    for case, mode, exhaust in (("normal-drop", "drop", False),
                                ("normal-shutdown", "shutdown", False),
                                ("exhausted-drop", "drop", True),
                                ("exhausted-shutdown", "shutdown", True)):
        with (args.new_report_dir / f"{case}.child.log").open("w") as child_log:
            child = subprocess.Popen(
                [str(args.binary.resolve()), mode, str(args.mdns_port), str(args.service_port)],
                stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=child_log,
            )
            received = bytearray()

            def until(marker):
                deadline = time.monotonic() + 12
                while marker.encode() not in received:
                    if time.monotonic() >= deadline or child.poll() is not None:
                        raise RuntimeError(f"{case}: missing {marker}; child status={child.poll()}")
                    readable, _, _ = select.select([child.stdout], [], [], 0.2)
                    if readable:
                        data = child.stdout.read1(4096)
                        if not data:
                            raise RuntimeError(f"{case}: child closed stdout")
                        received.extend(data)
                        child_log.write(data.decode())
                        child_log.flush()

            def command(value, marker):
                child.stdin.write((value + "\n").encode())
                child.stdin.flush()
                until(marker)

            def inspect(label):
                ps = subprocess.run(["/bin/ps", "-M", "-p", str(child.pid)],
                                    capture_output=True, text=True, check=True)
                sockets = subprocess.run(["/usr/sbin/lsof", "-w", "-nP", "-a", "-p",
                                          str(child.pid), "-iUDP", "-Fpn"],
                                         capture_output=True, text=True)
                if sockets.returncode not in (0, 1):
                    raise RuntimeError(sockets.stderr)
                raw = ps.stdout + "\nUDP:\n" + sockets.stdout
                (args.new_report_dir / f"{case}.{label}.txt").write_text(raw)
                names = [line[1:] for line in sockets.stdout.splitlines() if line.startswith("n")]
                return {"threads": len(ps.stdout.strip().splitlines()) - 1, "udp": names}

            try:
                until("READY")
                baseline = inspect("baseline")
                command("init", "LIVE")
                live = inspect("live")
                if live["threads"] != baseline["threads"] + 1 or not live["udp"]:
                    raise RuntimeError(f"{case}: no owned worker/socket established")
                if exhaust:
                    command("exhaust", "EXHAUSTED")
                command("cleanup", "DROPPED")
                command("release", "RELEASED")
                start = time.monotonic()
                while True:
                    after = inspect("after-release")
                    cleaned = after == baseline
                    if cleaned or time.monotonic() - start >= 7:
                        break
                    time.sleep(0.2)
                report = {"case": case, "pid": child.pid, "baseline": baseline, "live": live,
                          "after_release": after, "cleanup_ms": int(1000 * (time.monotonic() - start)),
                          "child_still_alive": child.poll() is None,
                          "owned_worker_and_sockets_cleaned": cleaned}
                reports.append(report)
                print(json.dumps(report), flush=True)
                child.stdin.write(b"exit\n")
                child.stdin.flush()
                if child.wait(timeout=5) != 0:
                    raise RuntimeError(f"{case}: child failed")
            finally:
                if child.poll() is None:
                    child.kill()
                    child.wait()
    (args.new_report_dir / "report.json").write_text(json.dumps(reports, indent=2) + "\n")
    return 0 if all(r["owned_worker_and_sockets_cleaned"] and r["child_still_alive"] for r in reports) else 1


if __name__ == "__main__":
    raise SystemExit(main())
