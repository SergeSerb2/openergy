# Real public-command E2E. Only an owned temp install, virtual daemons and private pipes.
param([Parameter(Mandatory=$true)][string]$BinaryPath, [ValidateRange(25240,25259)][int]$Port=25240)
$ErrorActionPreference='Stop'
$root=Join-Path ([IO.Path]::GetTempPath()) ('oe-windows-lifecycle-'+[guid]::NewGuid().ToString('N'))
$install=Join-Path $root 'install with spaces'
$data=Join-Path $root 'data with spaces'
$env:OPENERGY_DISABLE_DISCOVERY='1'
$manager=Join-Path $PSScriptRoot 'openergy.ps1'
$passed=$false
function Check($Value,[string]$Name) { if (!$Value) {throw $Name}; Write-Output "PASS $Name" }
function Manage([string]$Action) { & $manager -Action $Action -InstallDir $install -DataDir $data }
try {
    $null=& $manager Install -InstallDir $install -DataDir $data -BinaryPath $BinaryPath -NoStartup -Platform virtual -Port $Port
    Check (Test-Path -LiteralPath "$install\install.json") 'fresh install without Synergy'
    Check ((Get-Content -LiteralPath "$data\config.toml" -Raw).Contains('role = "off"')) 'fresh install starts off'
    $status=Manage Start
    Check ($status.daemon.platform -eq 'virtual' -and $status.state -eq 'idle' -and $status.role -eq 'off') 'start uses isolated virtual config and pipe'
    $status=Manage Start
    Check ($status.state -eq 'idle') 'duplicate start returns existing daemon'
    # Exercise real Windows IPC, configuration persistence, server and failure paths.
    $exe=Join-Path $install 'openergy.exe'
    $socket=Join-Path $data 'openergy.sock'
    $env:OPENERGY_CONFIG=Join-Path $data 'config.toml'
    $env:OPENERGY_CONFIG_DIR=$data
    $env:OPENERGY_TLS_DIR=Join-Path $data 'tls'
    $env:OPENERGY_SOCKET=$socket
    $json=@{patch=@{role='server';tls=@{enabled=$false}}} | ConvertTo-Json -Compress
    # ProcessStartInfo preserves JSON quoting on both Windows PowerShell 5.1 and pwsh 7.
    $info=New-Object Diagnostics.ProcessStartInfo
    $info.FileName=$exe
    $info.Arguments='ctl config.patch --socket "'+$socket+'" --raw --json "'+$json.Replace('"','\"')+'"'
    $info.UseShellExecute=$false
    $info.RedirectStandardOutput=$true
    $info.RedirectStandardError=$true
    $patcher=[Diagnostics.Process]::Start($info)
    $output=$patcher.StandardOutput.ReadToEndAsync()
    $errors=$patcher.StandardError.ReadToEndAsync()
    if (!$patcher.WaitForExit(15000)) { $patcher.Kill(); $patcher.WaitForExit(); throw 'config patch deadline' }
    Check ($patcher.ExitCode -eq 0) 'config patch through real pipe'
    $patcher.Dispose()
    $status=Manage Status
    Check ($status.state -eq 'listening') 'server listens on assigned loopback port'
    $owned=Get-Process -Id $status.daemon.pid
    Check ($owned.Path -eq $exe) 'crash target is the owned temp daemon'
    $owned.Kill()
    $null=$owned.WaitForExit(5000)
    $restarted=$false
    for($i=0;$i -lt 50;$i++) {
        Start-Sleep -Milliseconds 100
        try { $status=Manage Status; if ($status.daemon.pid -ne $owned.Id -and $status.state -eq 'listening') {$restarted=$true;break} } catch {}
    }
    Check $restarted 'supervisor restarts an owned crashed daemon'
    $first=Get-Content -LiteralPath "$data\runner.json" -Raw | ConvertFrom-Json
    $null=Manage Stop
    Check (!(Get-Process -Id $first.pid -ErrorAction SilentlyContinue)) 'stop waits for owned runner exit'
    $null=Manage Stop
    Write-Output 'PASS repeated stop is safe'
    $preserved=(Get-FileHash -LiteralPath "$data\config.toml").Hash
    $status=Manage Start
    Check ($status.state -eq 'listening') 'restart preserves server configuration'
    $null=Manage Uninstall
    Check (!(Test-Path -LiteralPath "$install\openergy.exe")) 'uninstall removes installed binary'
    Check ((Get-FileHash -LiteralPath "$data\config.toml").Hash -eq $preserved) 'uninstall preserves config'
    # A directory not made by this manager must never be overwritten or removed.
    New-Item -ItemType Directory -Path $install -Force | Out-Null
    [IO.File]::WriteAllText("$install\unowned.txt",'preserve')
    $rejected=$false
    try { $null=& $manager Install -InstallDir $install -DataDir $data -BinaryPath $BinaryPath -NoStartup -Platform virtual -Port $Port } catch { $rejected=$true }
    Check ($rejected -and (Test-Path -LiteralPath "$install\unowned.txt")) 'unmanaged install directory refused'
    $rejected=$false
    try { $null=Manage Uninstall } catch { $rejected=$true }
    Check ($rejected -and (Test-Path -LiteralPath "$install\unowned.txt")) 'unmanaged uninstall refused'
    $passed=$true
    Write-Output 'WINDOWS_LIFECYCLE_PASS'
} finally {
    if (Test-Path -LiteralPath "$install\install.json") {
        try { $null=Manage Uninstall } catch { Write-Warning 'Owned lab cleanup needs inspection; directory retained.'; throw }
    }
    if ($passed) { Remove-Item -LiteralPath $root -Recurse -Force } else { Write-Output "Retained lab: $root" }
}
