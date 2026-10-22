# Optional migration wrapper. Use openergy.ps1 for installs without Synergy.
# Run elevated as the SAME Windows user who will run Openergy.
[CmdletBinding()]
param(
    [switch]$Install, [switch]$Start, [switch]$Commit, [switch]$Rollback,
    [string]$InstallDir = "$env:LOCALAPPDATA\Programs\Openergy",
    [string]$DataDir = "$env:APPDATA\Openergy",
    [string]$BinaryPath = "$PSScriptRoot\openergy.exe"
)
$ErrorActionPreference = 'Stop'
if (@($Install,$Start,$Commit,$Rollback).Where({$_}).Count -ne 1) { throw 'Choose exactly one of -Install, -Start, -Commit, or -Rollback' }
$State = Join-Path $DataDir 'synergy-state.json'
$Svc = 'Synergy Core Daemon'
$Synergy = 'C:\Program Files\Synergy'
$Approved = 'HKLM:\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run'
$RunName = 'Synergy 3 Service'
$Manager = if ($Install) { Join-Path $PSScriptRoot 'openergy.ps1' } else { Join-Path $InstallDir 'openergy.ps1' }
function Manage([string]$Action) {
    & $Manager -Action $Action -InstallDir $InstallDir -DataDir $DataDir
}
function Restore-Synergy($Saved) {
    Set-Service -Name $Svc -StartupType $Saved.service_start
    if ($null -ne $Saved.delayed) {
        Set-ItemProperty 'HKLM:\SYSTEM\CurrentControlSet\Services\Synergy Core Daemon' -Name DelayedAutostart -Value $Saved.delayed -Type DWord
    } else {
        Remove-ItemProperty 'HKLM:\SYSTEM\CurrentControlSet\Services\Synergy Core Daemon' -Name DelayedAutostart -ErrorAction SilentlyContinue
    }
    if ($Saved.approved) {
        if (!(Test-Path $Approved)) { New-Item $Approved -Force | Out-Null }
        Set-ItemProperty $Approved -Name $RunName -Type Binary -Value ([Convert]::FromBase64String($Saved.approved))
    } else { Remove-ItemProperty $Approved -Name $RunName -ErrorAction SilentlyContinue }
    if ($Saved.was_running) { Start-Service -Name $Svc }
    $ui = @(Get-Process -Name synergy-service -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq (Join-Path $Synergy 'synergy-service.exe') })
    if ($Saved.was_ui_running -and $ui.Count -eq 0) {
        # A unique task cannot replace another install's or user's one-shot task.
        $name = 'OpenergyRollback-'+[guid]::NewGuid().ToString('N')
        try {
            $action = New-ScheduledTaskAction -Execute (Join-Path $Synergy 'synergy-service.exe') -Argument '-d'
            $principal = New-ScheduledTaskPrincipal -UserId $Saved.sid -LogonType Interactive -RunLevel Limited
            Register-ScheduledTask -TaskName $name -Action $action -Principal $principal | Out-Null
            Start-ScheduledTask -TaskName $name
            Start-Sleep -Seconds 3
        } finally { Unregister-ScheduledTask -TaskName $name -Confirm:$false -ErrorAction SilentlyContinue }
    }
}
if ($Install) {
    & $Manager -Action Install -InstallDir $InstallDir -DataDir $DataDir -BinaryPath $BinaryPath
    return
}
if ($Start) {
    # Full owner/path/files/task/endpoint validation must precede service changes.
    $null = Manage 'Validate'
    if (Test-Path -LiteralPath $State) { throw 'A migration is already pending; use -Rollback before retrying' }
    $service = Get-Service -Name $Svc
    $property = Get-ItemProperty $Approved -Name $RunName -ErrorAction SilentlyContinue
    $value = if ($property) { $property.$RunName } else { $null }
    $saved = @{
        version = 1
        sid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
        service_start = $service.StartType.ToString()
        was_running = $service.Status -eq 'Running'
        was_ui_running = @(Get-Process -Name synergy-service -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq (Join-Path $Synergy 'synergy-service.exe') }).Count -gt 0
        delayed = (Get-ItemProperty 'HKLM:\SYSTEM\CurrentControlSet\Services\Synergy Core Daemon' -Name DelayedAutostart -ErrorAction SilentlyContinue).DelayedAutostart
        approved = if ($value) { [Convert]::ToBase64String($value) } else { $null }
    }
    $saved | ConvertTo-Json | Set-Content -LiteralPath $State -Encoding UTF8
    try {
        if ($saved.was_running) { Stop-Service -Name $Svc }
        # Exact executable paths only. Never use a prefix that matches sibling folders.
        foreach ($name in @('synergy-core','synergy-tray','synergy-service')) {
            $path = Join-Path $Synergy "$name.exe"
            Get-Process -Name $name -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $path } | Stop-Process -Force
        }
        $status = Manage 'Start'
        if ($status.state -notin @('listening','connected')) { throw 'Openergy is not ready for migration' }
        $status
    } catch {
        $failure = $_
        $stopFailed = $false
        try { $null = Manage 'Stop' } catch { $stopFailed = $true; Write-Warning "Openergy stop failed; Synergy restoration will still run. $($_.Exception.Message)" }
        try {
            Restore-Synergy $saved
            if (!$stopFailed) { Remove-Item -LiteralPath $State }
        } catch { Write-Warning "Automatic restoration failed; saved migration state remains. Run -Rollback. $($_.Exception.Message)" }
        throw $failure
    }
    return
}
if (!(Test-Path -LiteralPath $State)) { throw 'No saved migration state; refusing to change Synergy or Openergy' }
$saved = Get-Content -LiteralPath $State -Raw | ConvertFrom-Json
if ($saved.version -ne 1 -or $saved.sid -ne [Security.Principal.WindowsIdentity]::GetCurrent().User.Value) { throw 'Migration state belongs to another Windows user or script version' }
if ($Commit) {
    $status = Manage 'Status'
    if ($status.state -notin @('listening','connected')) { throw 'Openergy must be ready before committing migration' }
    Set-Service -Name $Svc -StartupType Manual
    if (!(Test-Path $Approved)) { New-Item $Approved -Force | Out-Null }
    Set-ItemProperty $Approved -Name $RunName -Type Binary -Value ([byte[]](3,0,0,0,0,0,0,0,0,0,0,0))
    Write-Output 'Synergy automatic startup is disabled. Saved state remains available for -Rollback.'
    return
}
$stopFailure = $null
try { $null = Manage 'Stop' } catch { $stopFailure = $_ }
# Restore even when the installation marker or owned-stop check has failed.
Restore-Synergy $saved
if ($stopFailure) { Write-Warning 'Synergy was restored; Openergy stop failed and saved state remains.'; throw $stopFailure }
Remove-Item -LiteralPath $State
Write-Output 'Saved Synergy service and startup state restored.'
