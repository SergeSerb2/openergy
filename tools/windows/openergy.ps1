# Per-user install and lifecycle control. No Synergy, services, or firewall changes.
[CmdletBinding()]
param(
    [Parameter(Mandatory=$true, Position=0)]
    [ValidateSet('Install','Start','Stop','Status','Uninstall')][string]$Action,
    [string]$InstallDir = "$env:LOCALAPPDATA\Programs\Openergy",
    [string]$DataDir = "$env:APPDATA\Openergy",
    [string]$BinaryPath = "$PSScriptRoot\openergy.exe",
    [ValidateSet('windows','virtual')][string]$Platform = 'windows',
    [ValidateRange(1,65535)][int]$Port = 24801,
    [switch]$NoStartup
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version 2
function Full-Path([string]$Path) {
    if (!$Path -or $Path.IndexOfAny([char[]]"`"`r`n`0") -ge 0) { throw 'Invalid path' }
    [IO.Path]::GetFullPath($Path).TrimEnd('\')
}
$InstallDir = Full-Path $InstallDir
$DataDir = Full-Path $DataDir
if ($InstallDir -eq $DataDir -or $InstallDir.StartsWith($DataDir+'\', [StringComparison]::OrdinalIgnoreCase) -or $DataDir.StartsWith($InstallDir+'\', [StringComparison]::OrdinalIgnoreCase)) {
    throw 'InstallDir and DataDir must be separate directories'
}
$Marker = Join-Path $InstallDir 'install.json'
$Exe = Join-Path $InstallDir 'openergy.exe'
$Config = Join-Path $DataDir 'config.toml'
$Socket = Join-Path $DataDir 'openergy.sock'
$RunFile = Join-Path $DataDir 'runner.json'
$StopFile = Join-Path $DataDir 'stop-requested'
$PowerShell = "$env:SystemRoot\System32\WindowsPowerShell\v1.0\powershell.exe"
$Sid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
$env:OPENERGY_CONFIG = $Config
$env:OPENERGY_CONFIG_DIR = $DataDir
$env:OPENERGY_TLS_DIR = Join-Path $DataDir 'tls'
$env:OPENERGY_SOCKET = $Socket
function Read-Install {
    if (!(Test-Path -LiteralPath $Marker -PathType Leaf)) { throw 'This directory has no managed Openergy installation' }
    $saved = Get-Content -LiteralPath $Marker -Raw | ConvertFrom-Json
    if ($saved.version -ne 1 -or $saved.sid -ne $Sid -or $saved.install_dir -ne $InstallDir -or $saved.data_dir -ne $DataDir) {
        throw 'Installation owner or paths do not match; use the original InstallDir and DataDir'
    }
    $saved
}
function Runner {
    if (!(Test-Path -LiteralPath $RunFile)) { return $null }
    $record = Get-Content -LiteralPath $RunFile -Raw | ConvertFrom-Json
    $process = Get-Process -Id $record.pid -ErrorAction SilentlyContinue
    if ($process -and $process.Path -eq $PowerShell -and $process.StartTime.ToUniversalTime().Ticks.ToString() -eq $record.started) { return $process }
    return $null
}
function Ctl([string]$Method) {
    $info = New-Object Diagnostics.ProcessStartInfo
    $info.FileName = $Exe
    $info.Arguments = 'ctl ' + $Method + ' --socket "' + $Socket + '" --raw'
    $info.UseShellExecute = $false
    $info.CreateNoWindow = $true
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    $process = [Diagnostics.Process]::Start($info)
    try {
        $output = $process.StandardOutput.ReadToEndAsync()
        $errors = $process.StandardError.ReadToEndAsync()
        if (!$process.WaitForExit(12000)) { $process.Kill(); $process.WaitForExit(); return $null }
        [IO.File]::WriteAllText((Join-Path $DataDir 'control.stderr.log'), $errors.Result)
        if ($process.ExitCode -ne 0) { return $null }
        $output.Result | ConvertFrom-Json
    } finally { $process.Dispose() }
}
function Task-Owned($Saved) {
    $task = Get-ScheduledTask -TaskName $Saved.task -ErrorAction SilentlyContinue
    if ($task) {
        if ($task.Description -ne "Openergy managed install: $InstallDir | $DataDir" -or $task.Principal.UserId -notin @($Sid, [Security.Principal.WindowsIdentity]::GetCurrent().Name)) {
            throw 'Scheduled task belongs to another installation'
        }
    }
    $task
}
function Stop-Owned($Saved) {
    $task = if (!$Saved.no_startup) { Task-Owned $Saved } else { $null }
    if ($task) { Disable-ScheduledTask -InputObject $task | Out-Null }
    [IO.File]::WriteAllText($StopFile, 'stop')
    $runner = Runner
    if ($runner) {
        $null = Ctl 'daemon.stop'
        if (!$runner.WaitForExit(15000)) { throw 'Runner did not stop; installation preserved for inspection' }
    }
    # Never kill by process name, PID file alone, or a command-line substring.
    if (Runner) { throw 'Runner is still active' }
    if (Test-Path -LiteralPath $RunFile) { Remove-Item -LiteralPath $RunFile }
}
if ($Action -eq 'Install') {
    if (Test-Path -LiteralPath $Marker) { $null = Read-Install; throw 'Already installed; stop and uninstall before replacing the binary. Data is preserved.' }
    if (Test-Path -LiteralPath $InstallDir) {
        if (@(Get-ChildItem -LiteralPath $InstallDir -Force).Count -ne 0) { throw 'Refusing to overwrite an unmanaged directory' }
    }
    $BinaryPath = Full-Path $BinaryPath
    if (!(Test-Path -LiteralPath $BinaryPath -PathType Leaf)) { throw 'BinaryPath must point to openergy.exe' }
    if ($Platform -eq 'virtual' -and !$NoStartup) { throw 'Virtual installs require -NoStartup' }
    New-Item -ItemType Directory -Force -Path $InstallDir, $DataDir | Out-Null
    foreach ($file in @('openergy.ps1','openergy-run.ps1','openergy-run.cmd')) {
        Copy-Item -LiteralPath (Join-Path $PSScriptRoot $file) -Destination (Join-Path $InstallDir $file)
    }
    Copy-Item -LiteralPath $BinaryPath -Destination $Exe
    if (!(Test-Path -LiteralPath $Config)) {
        # A fresh install stays off until the user chooses a role and TLS trust.
        $name = if ($env:COMPUTERNAME) { $env:COMPUTERNAME | ConvertTo-Json -Compress } else { '"windows"' }
        [IO.File]::WriteAllText($Config, "version = 1`nname = $name`nrole = `"off`"`n[server]`nlisten = `"127.0.0.1:$Port`"`nport = $Port`n")
    }
    $sha = [Security.Cryptography.SHA256]::Create()
    try { $hash = [BitConverter]::ToString($sha.ComputeHash([Text.Encoding]::UTF8.GetBytes(($Sid+'|'+$InstallDir+'|'+$DataDir).ToLowerInvariant()))).Replace('-','').Substring(0,16) } finally { $sha.Dispose() }
    $saved = @{version=1;sid=$Sid;install_dir=$InstallDir;data_dir=$DataDir;platform=$Platform;no_startup=[bool]$NoStartup;task="Openergy-$hash"}
    if (!$NoStartup) {
        if (Get-ScheduledTask -TaskName $saved.task -ErrorAction SilentlyContinue) { throw 'Task name already exists; refusing to replace it' }
        $arguments = '-NoLogo -NoProfile -NonInteractive -WindowStyle Hidden -ExecutionPolicy Bypass -File "' + (Join-Path $InstallDir 'openergy-run.ps1') + '" -InstallDir "' + $InstallDir + '" -DataDir "' + $DataDir + '" -Platform windows'
        $taskAction = New-ScheduledTaskAction -Execute $PowerShell -Argument $arguments -WorkingDirectory $InstallDir
        $trigger = New-ScheduledTaskTrigger -AtLogOn -User $Sid
        $principal = New-ScheduledTaskPrincipal -UserId $Sid -LogonType Interactive -RunLevel Limited
        $settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -ExecutionTimeLimit ([TimeSpan]::Zero) -MultipleInstances IgnoreNew
        Register-ScheduledTask -TaskName $saved.task -Description "Openergy managed install: $InstallDir | $DataDir" -Action $taskAction -Trigger $trigger -Principal $principal -Settings $settings | Out-Null
        Disable-ScheduledTask -TaskName $saved.task | Out-Null
    }
    $saved | ConvertTo-Json | Set-Content -LiteralPath $Marker -Encoding UTF8
    Write-Output 'Installed for this user; run Start when configuration is ready. Data is preserved on uninstall.'
    return
}
$saved = Read-Install
if ($Action -eq 'Start') {
    if (Runner) { $status=Ctl 'status.get'; if (!$status) { throw 'Runner exists but control is not ready' }; $status; return }
    Remove-Item -LiteralPath $StopFile -ErrorAction SilentlyContinue
    if ($saved.no_startup) {
        $arguments = '-NoLogo -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "' + (Join-Path $InstallDir 'openergy-run.ps1') + '" -InstallDir "' + $InstallDir + '" -DataDir "' + $DataDir + '" -Platform ' + $saved.platform
        Start-Process -FilePath $PowerShell -ArgumentList $arguments -WorkingDirectory $InstallDir -WindowStyle Hidden | Out-Null
    } else {
        $task = Task-Owned $saved
        if (!$task) { throw 'Startup task is missing; installation preserved' }
        Enable-ScheduledTask -InputObject $task | Out-Null
        Start-ScheduledTask -InputObject $task
    }
    $deadline = (Get-Date).AddSeconds(30)
    while ((Get-Date) -lt $deadline) {
        Start-Sleep -Milliseconds 100
        if (Runner) {
            $status = Ctl 'status.get'
            if ($status) {
                if ($status.last_error) { Stop-Owned $saved; throw "Daemon startup failed: $($status.last_error)" }
                $status; return
            }
        }
    }
    Stop-Owned $saved
    throw 'Daemon did not become ready; see daemon.stderr.log and runner.stderr.log in DataDir'
}
if ($Action -eq 'Status') {
    if (!(Runner)) { throw 'Managed runner is stopped' }
    $status = Ctl 'status.get'
    if (!$status) { throw 'Daemon control is not ready' }
    $status; return
}
Stop-Owned $saved
if ($Action -eq 'Uninstall') {
    if (!$saved.no_startup) {
        $task = Task-Owned $saved
        if ($task) { Unregister-ScheduledTask -InputObject $task -Confirm:$false }
    }
    # Remove only the fixed files installed here. Keep config, identity, trust, and logs.
    foreach ($file in @('openergy.exe','openergy-run.ps1','openergy-run.cmd','openergy.ps1','install.json')) {
        Remove-Item -LiteralPath (Join-Path $InstallDir $file) -ErrorAction Stop
    }
    if (@(Get-ChildItem -LiteralPath $InstallDir -Force).Count -eq 0) { Remove-Item -LiteralPath $InstallDir }
    Write-Output 'Uninstalled. Configuration, TLS identity, trust, and logs remain in DataDir.'
} else { Write-Output 'Stopped. Automatic startup is disabled until Start.' }
