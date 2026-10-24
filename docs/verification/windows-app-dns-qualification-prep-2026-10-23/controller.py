"""Bounded native SOURCE/BUILD ONLY transport. Never run the generated executable."""
import base64
import datetime
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parent
RAW = ROOT / "evidence"
SSH = ["/usr/bin/ssh", "-o", "BatchMode=yes", "-o", "ConnectTimeout=8",
       "-o", "StrictHostKeyChecking=yes", "-o", "UpdateHostKeys=no",
       "-o", "ControlMaster=no", "sergecode-windows", "powershell.exe",
       "-NoLogo", "-NoProfile", "-NonInteractive", "-EncodedCommand"]


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    action = sys.argv[1] if len(sys.argv) == 2 else ""
    if action not in ("inspect-sdk", "build-only"):
        raise SystemExit("only inspect-sdk or build-only; no executable-run action")
    current = os.getpriority(os.PRIO_PROCESS, 0)
    if current < 15:
        os.nice(15 - current)
    if os.getpriority(os.PRIO_PROCESS, 0) != 15:
        raise SystemExit("effective nice must equal 15")
    RAW.mkdir(exist_ok=True)
    with (RAW / (action + ".guard")).open("x") as guard:
        guard.write(datetime.datetime.now().astimezone().isoformat() + "\n")
    payload = None
    if action == "inspect-sdk":
        script = (ROOT / "native-preflight.ps1").read_bytes().decode()
    else:
        preflight = json.loads((RAW / "inspect-sdk.stdout").read_text(encoding="utf-8-sig"))
        if not preflight["all_required_available"] or not preflight["pwsh7_available"]:
            raise SystemExit("native prerequisites unavailable; no build attempt")
        files = {name: (ROOT / name).read_bytes() for name in
                 ("qualify.c", "qualify.manifest", "native-build.ps1", "source-manifest.json")}
        envelope = {name: {"base64": base64.b64encode(data).decode(), "sha256": sha(data)}
                    for name, data in files.items()}
        payload = base64.b64encode(json.dumps(envelope).encode())
        script = (ROOT / "native-transfer.ps1").read_text()
    encoded = base64.b64encode(script.encode("utf-16le")).decode()
    start = time.monotonic_ns()
    started = datetime.datetime.now().astimezone().isoformat()
    code = None
    timed_out = False
    try:
        result = subprocess.run(SSH + [encoded], input=payload, capture_output=True, timeout=55)
        out, err, code = result.stdout, result.stderr, result.returncode
    except subprocess.TimeoutExpired as exc:
        out, err = exc.stdout or b"", exc.stderr or b""
        timed_out = True
    (RAW / (action + ".stdout")).write_bytes(out)
    (RAW / (action + ".stderr")).write_bytes(err)
    record = {"started_local": started, "exit_code": code, "timed_out": timed_out,
              "elapsed_ns": time.monotonic_ns() - start, "transport_timeout_seconds": 55,
              "local_effective_nice": os.getpriority(os.PRIO_PROCESS, 0),
              "command": SSH + ["<encoded exact script>"], "script_sha256": sha(script.encode()),
              "stdin_sha256": sha(payload) if payload else None,
              "stdout_sha256": sha(out), "stderr_sha256": sha(err),
              "generated_executable_runs": 0, "transport_attempts": 1}
    (RAW / (action + ".json")).write_text(json.dumps(record, indent=2) + "\n")
    if action == "build-only" and code == 0:
        data = json.loads(out.decode("utf-8-sig"))
        for item in data.pop("evidence_files", []):
            name = item["name"]
            if name not in ("build-result.json", "compile.stdout", "compile.stderr",
                            "link.stdout", "link.stderr", "imports.stdout", "imports.stderr"):
                raise SystemExit("unexpected evidence file")
            (RAW / name).write_bytes(base64.b64decode(item["base64"]))
        (RAW / "transfer-result.json").write_text(json.dumps(data, indent=2) + "\n")
    print(json.dumps(record, indent=2))
    raise SystemExit(0 if code == 0 else 1)


if __name__ == "__main__":
    main()
