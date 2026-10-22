param([Parameter(Mandatory=$true)][string]$Openergy)
$ErrorActionPreference = 'Stop'
$Openergy = (Resolve-Path $Openergy).Path
$root = Join-Path ([IO.Path]::GetTempPath()) ('oe-net-acl-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $root | Out-Null
$env:OPENERGY_CONFIG_DIR = $root
$env:OPENERGY_TLS_DIR = Join-Path $root 'tls'
$env:OPENERGY_SOCKET = Join-Path $root 'ctl.sock'
$env:OPENERGY_DISABLE_DISCOVERY = '1'
$config = Join-Path $root 'config.toml'
$sid = [Security.Principal.WindowsIdentity]::GetCurrent().User
function Invoke-Offline([string[]]$CliArgs) {
    $value = & $Openergy @CliArgs --config $config
    if ($LASTEXITCODE -ne 0) { throw "Offline CLI failed: $LASTEXITCODE" }
    return $value
}
function Assert-Private([string]$Path) {
    $acl = Get-Acl -LiteralPath $Path
    if (-not $acl.AreAccessRulesProtected) { throw "Inherited ACL remains on $Path" }
    $rules = @($acl.GetAccessRules($true, $true, [Security.Principal.SecurityIdentifier]))
    if ($rules.Count -ne 1 -or $rules[0].IdentityReference -ne $sid -or
        $rules[0].AccessControlType -ne 'Allow' -or
        ($rules[0].FileSystemRights -band [Security.AccessControl.FileSystemRights]::FullControl) -ne
            [Security.AccessControl.FileSystemRights]::FullControl) {
        throw "ACL does not grant only the current user full control on $Path"
    }
}
$fp = Invoke-Offline -CliArgs @('fingerprint', '--synergy')
if ($fp -notmatch '^v2:sha256:[0-9a-f]{64}$') { throw 'Invalid fingerprint' }
$tls = Join-Path $root 'tls'
foreach ($name in @('', 'cert.pem', 'key.pem', '.identity-lock')) {
    $path = if ($name) { Join-Path $tls $name } else { $tls }
    Assert-Private $path
}
# Deliberately broaden only this disposable key, then verify repair on reload.
$key = Join-Path $tls 'key.pem'
$acl = Get-Acl -LiteralPath $key
$everyone = [Security.Principal.SecurityIdentifier]::new('S-1-1-0')
$acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new($everyone, 'Read', 'Allow'))
Set-Acl -LiteralPath $key -AclObject $acl
if ((Invoke-Offline -CliArgs @('fingerprint', '--synergy')) -ne $fp) { throw 'Identity changed during ACL repair' }
Assert-Private $key
Invoke-Offline -CliArgs @('trust', 'approve', $fp) | Out-Null
foreach ($name in @('trusted-servers', 'trusted-clients', '.trust-lock')) {
    Assert-Private (Join-Path $tls $name)
}
Invoke-Offline -CliArgs @('trust', 'revoke', $fp) | Out-Null
foreach ($name in @('trusted-servers', 'trusted-clients')) {
    $path = Join-Path $tls $name
    Assert-Private $path
    if ((Get-Item -LiteralPath $path).Length -ne 0) { throw 'Revoked pin remains' }
}
Write-Output 'WINDOWS_IDENTITY_TRUST_ACL=PASS'
Write-Output "artifacts=$root"
