"""One native Windows build-only workflow. No payload is sent to a shell stdin.

Run with nice 15. Supply the existing SSH alias and private historical state
path as arguments. Authentication stays with OpenSSH. There is no retry path.
"""
import argparse
import base64
import datetime
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import subprocess
import tarfile
import tempfile
import time
import traceback

HERE = Path(__file__).resolve().parent
SOURCE = "06b9a139ec855a6f697ded393fa4c2c894110613"
PRIOR = "52cbf8c3c41b15789500ce7be620d203006630a9"
PRIOR_DIR = "docs/verification/release-integration-2026-10-23/idle-tls-deadline-r2-fix-06b9a13-773e34c7"
MODULE = "crates/net/examples/dns_queue_cancel_c4806a3d/idle_tls_deadline.rs"
MODULE_SHA = "9cd3475f9c13c8e28ea54e0f344b55ae795267b22ea1676c45e5659a64f1f1d4"
REMOTE_NAME = "oe-idle-tls-win-r5-04c77b7a-06b9a13-48d2c5e9"
OPTIONS = ["-o", "BatchMode=yes", "-o", "ConnectTimeout=8",
           "-o", "StrictHostKeyChecking=yes", "-o", "UpdateHostKeys=no",
           "-o", "ControlMaster=no", "-o", "ServerAliveInterval=10",
           "-o", "ServerAliveCountMax=2", "-o", "ForwardAgent=no"]


def sha(data):
    return hashlib.sha256(data).hexdigest()


def save(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def git(*args):
    return subprocess.check_output(["git", *args], cwd=HERE)


def freeze(private):
    paths = ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "rustfmt.toml",
             "crates", "tools/e2e", "vendor"]
    archive = git("archive", "--format=tar", SOURCE, "--", *paths)
    entries = {}
    for line in git("ls-tree", "-rz", SOURCE, "--", *paths).split(b"\0"):
        if line:
            meta, path = line.split(b"\t", 1)
            mode, kind, oid = meta.decode().split()
            assert kind == "blob" and mode in ("100644", "100755")
            entries[path.decode()] = oid
    files = []
    with tarfile.open(fileobj=io.BytesIO(archive), mode="r:") as tar:
        for item in tar.getmembers():
            path = PurePosixPath(item.name)
            assert not path.is_absolute() and ".." not in path.parts
            assert item.isdir() or item.isfile()
            assert "installer" not in item.name.lower()
            if item.isfile():
                data = tar.extractfile(item).read()
                oid = entries.pop(item.name)
                blob = git("cat-file", "blob", oid)
                assert blob == data, "Git archive conversion refused: " + item.name
                files.append({"path": item.name, "bytes": len(data),
                              "sha256": sha(data), "git_blob": oid,
                              "crlf_pairs": data.count(b"\r\n"),
                              "lf_bytes": data.count(b"\n"),
                              "archive_equals_git_blob": True})
    assert not entries
    assert next(f["sha256"] for f in files if f["path"] == MODULE) == MODULE_SHA
    manifest = {"schema": "openergy.windows-build-source.v1", "source_commit": SOURCE,
                "source_tree": git("rev-parse", SOURCE + "^{tree}").decode().strip(),
                "module_sha256": MODULE_SHA, "archive_bytes": len(archive),
                "archive_sha256": sha(archive), "export_paths": paths, "files": files,
                "overlays": [], "line_ending_conversion": False}
    (private / "source.tar").write_bytes(archive)
    save(private / "source-manifest.json", manifest)
    save(HERE / "source-manifest.json", manifest)
    prior = []
    for line in git("ls-tree", "-rz", PRIOR, "--", PRIOR_DIR).split(b"\0"):
        if line:
            meta, path = line.split(b"\t", 1)
            oid = meta.decode().split()[2]
            data = git("cat-file", "blob", oid)
            prior.append({"path": path.decode(), "git_blob": oid,
                          "bytes": len(data), "sha256": sha(data)})
    save(HERE / "prior-evidence-binding.json", {"evidence_commit": PRIOR, "files": prior,
         "read_only_git_objects": True, "prior_controller_executed": False})
    return manifest


class Workflow:
    def __init__(self, args):
        self.alias = args.ssh_alias
        self.private = Path(tempfile.mkdtemp(prefix="oe-idle-tls-windows-r5-"))
        os.chmod(self.private, 0o700)
        self.raw = self.private / "raw"
        self.raw.mkdir(mode=0o700)
        self.state_input = json.loads(Path(args.prior_state).read_text())
        self.root = None
        self.result = {"source_commit": SOURCE, "module_sha256": MODULE_SHA,
                       "prior_evidence_commit": PRIOR, "started_local": self.now(),
                       "controller_nice": os.getpriority(os.PRIO_PROCESS, 0),
                       "transport": "OpenSSH SCP legacy fixed-file protocol (-O)",
                       "upload_attempts": 0, "runtime_invocations": 0, "passed": False}
        assert self.result["controller_nice"] == 15
        save(HERE / "private-evidence-location.json", {"private_root": str(self.private)})

    @staticmethod
    def now():
        return datetime.datetime.now().astimezone().isoformat()

    def command(self, label, argv, timeout, script=None):
        if script is not None:
            (self.raw / (label + ".ps1")).write_text(script)
        started, clock = self.now(), time.monotonic()
        timed_out = False
        try:
            p = subprocess.run(argv, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE, timeout=timeout)
            out, err, code = p.stdout, p.stderr, p.returncode
        except subprocess.TimeoutExpired as e:
            out, err, code, timed_out = e.stdout or b"", e.stderr or b"", None, True
        (self.raw / (label + ".stdout")).write_bytes(out)
        (self.raw / (label + ".stderr")).write_bytes(err)
        shown = [x.replace(self.alias, "<SSH_ALIAS>").replace(str(self.private), "<LOCAL_PRIVATE>") for x in argv]
        if script is not None:
            shown[-1] = "<UTF16LE_BASE64_OF_" + label + ".ps1>"
        record = {"command": shown, "started_local": started, "ended_local": self.now(),
                  "elapsed_seconds": time.monotonic() - clock, "timeout_seconds": timeout,
                  "exit_code": code, "timed_out": timed_out, "stdin": "DEVNULL; immediate EOF",
                  "script_sha256": sha(script.encode()) if script else None,
                  "stdout_sha256": sha(out), "stderr_sha256": sha(err)}
        save(self.raw / (label + ".json"), record)
        print(label, "exit=", code, "timeout=", timed_out, flush=True)
        return record, out

    def remote(self, label, script, timeout=45):
        encoded = base64.b64encode(script.encode("utf-16le")).decode()
        return self.command(label, ["/usr/bin/ssh", *OPTIONS, self.alias, "powershell.exe",
                                   "-NoLogo", "-NoProfile", "-NonInteractive", "-EncodedCommand", encoded], timeout, script)

    def stage(self, name, timeout):
        script = "$ErrorActionPreference='Stop'; (Get-Process -Id $PID).PriorityClass='BelowNormal'; & 'C:\\Program Files\\PowerShell\\7\\pwsh.exe' -NoLogo -NoProfile -NonInteractive -File '" + self.root + "\\" + name + ".ps1'; exit $LASTEXITCODE"
        record, out = self.remote(name, script, timeout)
        if record["timed_out"]:
            self.stop_owned(script)
        return record, out

    def stop_owned(self, exact_script):
        encoded = base64.b64encode(exact_script.encode("utf-16le")).decode()
        script = r"""$ErrorActionPreference='Stop'
$all=@(Get-CimInstance Win32_Process)
$owners=@($all|Where-Object {$_.Name -eq 'powershell.exe' -and $_.CommandLine -and $_.CommandLine.EndsWith('__ENCODED__')})
if($owners.Count -ne 1){throw 'watchdog exact owner not unique; no kill'}
$known=@{};$known[[int]$owners[0].ProcessId]=$owners[0].CreationDate
for($pass=0;$pass -lt 12;$pass++){foreach($p in $all){if($known.ContainsKey([int]$p.ParentProcessId) -and $p.CreationDate -ge $known[[int]$p.ParentProcessId]){$known[[int]$p.ProcessId]=$p.CreationDate}}}
$stopped=@(foreach($p in $all|Sort-Object CreationDate -Descending){if($known.ContainsKey([int]$p.ProcessId)){$live=Get-CimInstance Win32_Process -Filter ('ProcessId='+$p.ProcessId);if($live -and $live.CreationDate -eq $p.CreationDate -and $live.CommandLine -eq $p.CommandLine){Stop-Process -Id $p.ProcessId -ErrorAction Stop;[ordered]@{pid=$p.ProcessId;created=$p.CreationDate;path=$p.ExecutablePath;command_line=$p.CommandLine;exact_owner_or_descendant=$true}}}})
$stopped|ConvertTo-Json -Depth 8 -Compress
""".replace("__ENCODED__", encoded)
        record, _ = self.remote("watchdog-owned-stop", script)
        assert record["exit_code"] == 0, "owned stop unverified"

    def run(self):
        manifest = freeze(self.private)
        self.result["archive_sha256"] = manifest["archive_sha256"]
        setup = r"""$ErrorActionPreference='Stop';$ProgressPreference='SilentlyContinue'
(Get-Process -Id $PID).PriorityClass='BelowNormal'
if((Get-Process -Id $PID).SessionId -ne 0){throw 'session 0 required'}
if(@(Get-Process -Name cargo,rustc,cargo-clippy,clippy-driver -ErrorAction SilentlyContinue).Count){throw 'another native build active'}
foreach($p in @('C:\Program Files\PowerShell\7\pwsh.exe',(Join-Path $env:USERPROFILE '.rustup/toolchains/1.99.1-x86_64-pc-windows-msvc/bin/cargo.exe'),'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64\link.exe','C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\um\x64\kernel32.lib')){if(-not (Test-Path -LiteralPath $p -PathType Leaf)){throw 'installed prerequisite absent'}}
$root=Join-Path $env:TEMP '__NAME__'
if(Test-Path -LiteralPath $root){throw 'fresh owned root exists; no retry'}
if(Test-Path -LiteralPath (Join-Path $env:TEMP 'oe-idle-tls-r2-build-773e34c7-06b9a13')){throw 'old failed root exists; stop'}
New-Item -ItemType Directory -Path $root|Out-Null
$sid=[Security.Principal.WindowsIdentity]::GetCurrent().User
$acl=[Security.AccessControl.DirectorySecurity]::new();$acl.SetAccessRuleProtection($true,$false);$acl.SetOwner($sid)
$acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new($sid,'FullControl','ContainerInherit,ObjectInherit','None','Allow'))
Set-Acl -LiteralPath $root -AclObject $acl
[ordered]@{root=$root;original_temp=$env:TEMP;profile=$env:USERPROFILE;session=(Get-Process -Id $PID).SessionId;priority=(Get-Process -Id $PID).PriorityClass.ToString();created_utc=[DateTime]::UtcNow.ToString('o');root_acl_protected=(Get-Acl -LiteralPath $root).AreAccessRulesProtected}|ConvertTo-Json -Compress
""".replace("__NAME__", REMOTE_NAME)
        rec, out = self.remote("setup", setup)
        assert rec["exit_code"] == 0, "fresh-root setup failed; no alternative transport"
        state = json.loads(out.decode("utf-8-sig"))
        self.root = state["root"]
        state["prior_fixtures"] = self.state_input["prior_fixtures"] + [{"label": "historical-refusal-615c16e1", "root": self.state_input["root"]}]
        save(self.private / "state.json", state)
        self.result["fixture"] = REMOTE_NAME
        files = ["source.tar", "source-manifest.json", "state.json", "common.ps1", "qualify.ps1", "build.ps1", "postflight.ps1"]
        for name in files:
            if (HERE / name).exists() and name.endswith(".ps1"):
                (self.private / name).write_bytes((HERE / name).read_bytes())
        transfer = {"files": [{"path": name, "bytes": (self.private/name).stat().st_size,
                                "sha256": sha((self.private/name).read_bytes())} for name in files]}
        save(self.private / "transfer-manifest.json", transfer)
        self.result["upload_attempts"] += 1
        rec, _ = self.command("scp-upload", ["/usr/bin/scp", "-O", "-q", *OPTIONS,
                 *[str(self.private/name) for name in files + ["transfer-manifest.json"]],
                 self.alias + ":" + self.root.replace("\\", "/") + "/"], 90)
        assert rec["exit_code"] == 0, "SCP fixed-file transfer failed; no resend or fallback"
        rec, _ = self.stage("qualify", 360)
        assert rec["exit_code"] == 0, "source qualification failed; zero compile attempts"
        build_rec, _ = self.stage("build", 1260)
        self.result["build_transport_exit"] = build_rec["exit_code"]
        post_rec, out = self.stage("postflight", 360)
        self.result["postflight_transport_exit"] = post_rec["exit_code"]
        bundle = json.loads(out.decode("utf-8-sig"))
        rec, _ = self.command("scp-evidence", ["/usr/bin/scp", "-O", "-q", *OPTIONS,
                self.alias + ":" + self.root.replace("\\", "/") + "/evidence.tar", str(self.private / "evidence.tar")], 90)
        assert rec["exit_code"] == 0, "evidence download failed; no resend"
        data = (self.private / "evidence.tar").read_bytes()
        assert len(data) == bundle["bytes"] and sha(data) == bundle["sha256"]
        save(self.private / "bundle-confirmed.json", bundle)
        with tarfile.open(fileobj=io.BytesIO(data), mode="r:") as tar:
            for item in tar.getmembers():
                p = PurePosixPath(item.name)
                assert not p.is_absolute() and ".." not in p.parts and item.isfile()
                dest = self.private / "downloaded" / item.name
                dest.parent.mkdir(parents=True, exist_ok=True)
                dest.write_bytes(tar.extractfile(item).read())
        for e in json.loads((self.private / "downloaded/evidence-manifest.json").read_text()):
            data = (self.private / "downloaded" / e["path"]).read_bytes()
            assert len(data) == e["bytes"] and sha(data) == e["sha256"]
        artifact = json.loads((self.private / "downloaded/build-result.json").read_text()).get("artifact")
        if artifact:
            data = (self.private / "downloaded/target/x86_64-pc-windows-msvc/release/examples/dns_queue_cancel_c4806a3d.exe").read_bytes()
            assert len(data) == artifact["bytes"] and sha(data) == artifact["sha256"]
        self.result["bundle"] = bundle
        self.result["passed"] = build_rec["exit_code"] == 0 and post_rec["exit_code"] == 0
        if self.result["passed"]:
            cleanup = r"""$ErrorActionPreference='Stop'
(Get-Process -Id $PID).PriorityClass='BelowNormal'
$root='__ROOT__'
if((Split-Path $root -Leaf) -ne '__NAME__' -or (Split-Path $root -Parent) -ne '__TEMP__'){throw 'owned root identity mismatch'}
$owned=@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -ne $PID -and (($_.ExecutablePath -and $_.ExecutablePath.StartsWith($root+'\',[StringComparison]::OrdinalIgnoreCase)) -or ($_.CommandLine -and $_.CommandLine.Contains($root)))}|Select-Object Name,ProcessId,CreationDate,ExecutablePath,CommandLine)
if($owned.Count){throw 'owned process remains; retain fixture'}
Remove-Item -LiteralPath $root -Recurse -Force
[ordered]@{recorded_utc=[DateTime]::UtcNow.ToString('o');owned_processes_before_removal=$owned;owned_root_removed=(-not (Test-Path -LiteralPath $root));old_failed_root_absent=(-not (Test-Path -LiteralPath (Join-Path '__TEMP__' 'oe-idle-tls-r2-build-773e34c7-06b9a13')));removed_only_fresh_owned_root=$true;runtime_invocations=0}|ConvertTo-Json -Depth 8 -Compress
""".replace("__ROOT__", self.root).replace("__NAME__", REMOTE_NAME).replace("__TEMP__", state["original_temp"])
            rec, out = self.remote("owned-build-only-cleanup", cleanup, 90)
            self.result["cleanup_transport_exit"] = rec["exit_code"]
            self.result["passed"] = rec["exit_code"] == 0
            if rec["exit_code"] == 0:
                self.result["cleanup"] = json.loads(out.decode("utf-8-sig"))
        assert self.result["passed"], "native build or postflight failed; first result retained"

    def export(self):
        # Preserve exact originals privately. Public copies replace known path/alias
        # prefixes only. No original log is edited and no binary is committed.
        substitutions = [(self.alias, "<SSH_ALIAS>"), (str(self.private), "<LOCAL_PRIVATE>")]
        state_path = self.private / "state.json"
        if state_path.exists():
            state = json.loads(state_path.read_text())
            substitutions += [(state["profile"].replace("\\", "\\\\"), "<PROFILE_JSON>"),
                              (state["profile"], "<PROFILE>")]
        ledger = []
        for folder in [self.raw, self.private / "downloaded"]:
            if not folder.exists():
                continue
            for p in sorted(folder.rglob("*")):
                if not p.is_file() or p.suffix not in (".json", ".ps1", ".stdout", ".stderr", ".guard") or p.name == "state.json":
                    continue
                original = p.read_bytes()
                text = original.decode("utf-8-sig")
                for old, replacement in substitutions:
                    text = text.replace(old, replacement)
                copied = text.encode("utf-8")
                rel = Path(folder.name) / p.relative_to(folder)
                dest = HERE / "evidence" / rel
                dest.parent.mkdir(parents=True, exist_ok=True)
                dest.write_bytes(copied)
                ledger.append({"path": str(rel), "original_bytes": len(original), "original_sha256": sha(original),
                               "copy_bytes": len(copied), "copy_sha256": sha(copied), "byte_equal": original == copied})
        save(HERE / "copy-ledger.json", {"method": "UTF-8 BOM removed where present; known profile/local-root/SSH-alias replacements only; line endings preserved", "files": ledger})
        save(HERE / "result.json", self.result)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--ssh-alias", required=True)
    parser.add_argument("--prior-state", required=True)
    args = parser.parse_args()
    with (HERE / "controller-once.guard").open("x") as guard:
        guard.write(Workflow.now() + "\n")
    workflow = Workflow(args)
    code = 0
    try:
        workflow.run()
    except BaseException:
        code = 1
        raw = traceback.format_exc()
        (workflow.raw / "controller-first-failure.stderr").write_text(raw)
        workflow.result["failure"] = raw.splitlines()[-1]
        print(workflow.result["failure"], flush=True)
    finally:
        workflow.result["ended_local"] = workflow.now()
        workflow.result["controller_exit"] = code
        workflow.export()
        print("BUILD_ONLY_RESULT", "PASS" if workflow.result["passed"] else "UNVERIFIED_OR_FAIL", flush=True)
    raise SystemExit(code)


if __name__ == "__main__":
    main()
