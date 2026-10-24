$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
(Get-Process -Id $PID).PriorityClass = 'BelowNormal'
if ((Get-Process -Id $PID).PriorityClass -ne 'BelowNormal') { throw 'priority gate' }
$root = Join-Path $env:TEMP 'oe-appdns-prep-20261023-1ac50255-a447cb28'
if (Test-Path -LiteralPath $root) { throw 'fresh owned root required; no retry' }
$sid = [Security.Principal.WindowsIdentity]::GetCurrent().User
[void](New-Item -ItemType Directory -Path $root)
$acl = [Security.AccessControl.DirectorySecurity]::new()
$acl.SetAccessRuleProtection($true, $false)
$acl.SetOwner($sid)
$acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new($sid, 'FullControl', 'ContainerInherit,ObjectInherit', 'None', 'Allow'))
Set-Acl -LiteralPath $root -AclObject $acl
$envelope = [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String([Console]::In.ReadToEnd())) | ConvertFrom-Json
if (@($envelope.PSObject.Properties).Count -ne 4) { throw 'exact input file count required' }
foreach ($name in @('qualify.c', 'qualify.manifest', 'native-build.ps1', 'source-manifest.json')) {
    $entry = $envelope.$name
    if (-not $entry) { throw 'missing input' }
    $path = Join-Path $root $name
    [IO.File]::WriteAllBytes($path, [Convert]::FromBase64String($entry.base64))
    if ((Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLower() -ne $entry.sha256) { throw 'transferred source hash mismatch' }
}
& 'C:\Program Files\PowerShell\7\pwsh.exe' -NoLogo -NoProfile -NonInteractive -File (Join-Path $root 'native-build.ps1')
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
