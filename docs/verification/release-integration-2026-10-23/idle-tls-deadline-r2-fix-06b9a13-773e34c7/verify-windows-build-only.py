"""Export the committed source and request one native compile, with no runtime."""
import base64
import datetime
import hashlib
import io
import json
from pathlib import Path
import subprocess
import tarfile
import time

ROOT = Path(__file__).resolve().parent
RAW = ROOT / "raw"
SOURCE = "06b9a139ec855a6f697ded393fa4c2c894110613"
REMOTE_NAME = "oe-idle-tls-r2-build-773e34c7-06b9a13"
SSH = ["/usr/bin/ssh", "-o", "BatchMode=yes", "-o", "ConnectTimeout=8",
       "-o", "StrictHostKeyChecking=yes", "-o", "UpdateHostKeys=no", "-o", "ControlMaster=no",
       "sergecode-windows", "powershell.exe", "-NoLogo", "-NoProfile", "-NonInteractive", "-EncodedCommand"]


def digest(data):
    return hashlib.sha256(data).hexdigest()


def remote(label, script, payload=None, timeout=30):
    (ROOT / (label + ".ps1")).write_text(script)
    args = SSH + [base64.b64encode(script.encode("utf-16le")).decode()]
    started = datetime.datetime.now().astimezone().isoformat()
    start = time.monotonic_ns()
    code = None
    try:
        p = subprocess.run(args, input=payload, capture_output=True, timeout=timeout)
        out, err, code = p.stdout, p.stderr, p.returncode
    except subprocess.TimeoutExpired as e:
        out, err = e.stdout or b"", e.stderr or b""
    (RAW / (label + ".stdout")).write_bytes(out)
    (RAW / (label + ".stderr")).write_bytes(err)
    record = {"command": SSH + ["<UTF16LE_BASE64_OF_" + label + ".ps1>"],
              "started_local": started, "exit_code": code, "elapsed_ns": time.monotonic_ns() - start,
              "timeout_seconds": timeout, "script_sha256": digest(script.encode()),
              "stdin_sha256": digest(payload) if payload else None,
              "stdout_sha256": digest(out), "stderr_sha256": digest(err)}
    (RAW / (label + ".json")).write_text(json.dumps(record, indent=2) + "\n")
    print(label, code, flush=True)
    return code, out


def main():
    with (RAW / "windows-controller-once.guard").open("x") as guard:
        guard.write(datetime.datetime.now().astimezone().isoformat() + "\n")
    private = Path((RAW / "private-build-root.txt").read_text().strip())
    paths = ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "rustfmt.toml", "crates", "tools/e2e", "vendor"]
    archive = subprocess.check_output(["git", "archive", "--format=tar", SOURCE, "--", *paths])
    files = []
    with tarfile.open(fileobj=io.BytesIO(archive), mode="r:") as tar:
        for item in tar.getmembers():
            assert not item.issym() and not item.islnk()
            assert not item.name.startswith("/") and ".." not in Path(item.name).parts
            assert "installer" not in item.name.lower()
            if item.isfile():
                data = tar.extractfile(item).read()
                files.append({"path": item.name, "bytes": len(data), "sha256": digest(data)})
    manifest = {"source_commit": SOURCE, "archive_sha256": digest(archive),
                "archive_bytes": len(archive), "export_paths": paths, "files": files,
                "library_changes_or_platform_shims": False}
    manifest_bytes = (json.dumps(manifest, indent=2) + "\n").encode()
    (ROOT / "windows-source-manifest.json").write_bytes(manifest_bytes)
    (private / "windows-source.tar").write_bytes(archive)
    payload_file = io.BytesIO()
    with tarfile.open(fileobj=payload_file, mode="w:") as tar:
        for name, data in [("source.tar", archive), ("source-manifest.json", manifest_bytes),
                           ("build.ps1", (ROOT / "windows-build.ps1").read_bytes())]:
            entry = tarfile.TarInfo(name)
            entry.size = len(data)
            entry.mode = 0o600
            tar.addfile(entry, io.BytesIO(data))
    payload = payload_file.getvalue()
    (private / "windows-payload.tar").write_bytes(payload)
    setup = r'''$ErrorActionPreference='Stop';$ProgressPreference='SilentlyContinue'
(Get-Process -Id $PID).PriorityClass='BelowNormal'
$root=Join-Path $env:TEMP '__REMOTE_NAME__'
if(Test-Path -LiteralPath $root){throw 'owned root exists; no retry'}
$sid=[Security.Principal.WindowsIdentity]::GetCurrent().User
New-Item -ItemType Directory -Path $root|Out-Null
$acl=[Security.AccessControl.DirectorySecurity]::new();$acl.SetAccessRuleProtection($true,$false);$acl.SetOwner($sid)
$rule=[Security.AccessControl.FileSystemAccessRule]::new($sid,'FullControl','ContainerInherit,ObjectInherit','None','Allow');$acl.AddAccessRule($rule)
Set-Acl -LiteralPath $root -AclObject $acl
$bytes=[Convert]::FromBase64String([Console]::In.ReadToEnd())
$payload=Join-Path $root 'payload.tar';[IO.File]::WriteAllBytes($payload,$bytes)
if((Get-FileHash -LiteralPath $payload -Algorithm SHA256).Hash.ToLower() -ne '__PAYLOAD_SHA__'){throw 'payload hash mismatch'}
& (Join-Path $env:WINDIR 'System32/tar.exe') -xf $payload -C $root
if($LASTEXITCODE -ne 0){throw 'payload extraction failed'}
& 'C:\Program Files\PowerShell\7\pwsh.exe' -NoLogo -NoProfile -NonInteractive -File (Join-Path $root 'build.ps1')
$buildExit=$LASTEXITCODE
Write-Output ('NATIVE_BUILD_EXIT='+$buildExit)
exit $buildExit
'''.replace("__REMOTE_NAME__", REMOTE_NAME).replace("__PAYLOAD_SHA__", digest(payload))
    code, _ = remote("windows-setup-and-build", setup, base64.b64encode(payload), timeout=900)
    fetch = r'''$ErrorActionPreference='Stop';$ProgressPreference='SilentlyContinue'
$root=Join-Path $env:TEMP '__REMOTE_NAME__'
$names=@('build-once.guard','build-start.json','build-result.json','build.stdout','build.stderr','shared-inputs-before.json','shared-inputs-after.json')
$files=@(foreach($n in $names){$p=Join-Path $root $n;[ordered]@{name=$n;exists=(Test-Path -LiteralPath $p -PathType Leaf);base64=$(if(Test-Path -LiteralPath $p -PathType Leaf){[Convert]::ToBase64String([IO.File]::ReadAllBytes($p))}else{$null})}})
$a=Get-Acl -LiteralPath $root
[ordered]@{files=$files;root_acl_protected=$a.AreAccessRulesProtected;root_acl_rule_count=@($a.Access).Count;root_acl_current_user_only=(@($a.Access).Count -eq 1 -and $a.Access[0].IdentityReference.Translate([Security.Principal.SecurityIdentifier]).Value -eq [Security.Principal.WindowsIdentity]::GetCurrent().User.Value);binary_execution_requested=$false}|ConvertTo-Json -Depth 8 -Compress
'''.replace("__REMOTE_NAME__", REMOTE_NAME)
    fetch_code, out = remote("windows-fetch-build-evidence", fetch, timeout=45)
    if fetch_code == 0:
        fetched = json.loads(out.decode("utf-8-sig"))
        for item in fetched.pop("files"):
            if item["exists"]:
                (RAW / ("windows-" + item["name"])).write_bytes(base64.b64decode(item["base64"]))
        (ROOT / "windows-build-acl.json").write_text(json.dumps(fetched, indent=2) + "\n")
    assert code == 0 and fetch_code == 0


if __name__ == "__main__":
    main()
