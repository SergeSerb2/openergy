# Replaces a Synergy 3 server on Windows with Openergy, or puts Synergy back.
# Run elevated, with openergy.exe in C:\Program Files\Openergy and the config in
# %APPDATA%\Openergy (openergy import-synergy --from C:\ProgramData\Synergy\config).
#
#   switch-from-synergy.ps1 -Install    register the Openergy logon task (disabled)
#   switch-from-synergy.ps1 -Start      stop Synergy, start Openergy (reversible)
#   switch-from-synergy.ps1 -Commit     keep Synergy off at the next logon too
#   switch-from-synergy.ps1 -Rollback   stop Openergy, bring Synergy back
param([switch]$Install, [switch]$Start, [switch]$Commit, [switch]$Rollback)
$ErrorActionPreference = 'Stop'
$Dir = 'C:\Program Files\Openergy'
$Exe = "$Dir\openergy.exe"
$State = "$Dir\synergy-state.json"
$Synergy = 'C:\Program Files\Synergy'
$Svc = 'Synergy Core Daemon'
$Approved = 'HKLM:\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run'
$RunName = 'Synergy 3 Service'
$User = [Security.Principal.WindowsIdentity]::GetCurrent().Name
function Log($m) { Write-Output ('{0:HH:mm:ss} {1}' -f (Get-Date), $m) }
function Stop-ByPath($name, $dir) {
    Get-Process -Name $name -ErrorAction SilentlyContinue |
        Where-Object { $_.Path -and $_.Path.StartsWith($dir, [StringComparison]::OrdinalIgnoreCase) } |
        ForEach-Object { Log "stopping $name $($_.Id)"; Stop-Process -Id $_.Id -Force }
}
function Start-Interactive($exe, $arguments) {
    # SSH runs in session 0; a one-shot task starts the program on the user's desktop.
    $name = 'OpenergyOneShot'
    Unregister-ScheduledTask -TaskName $name -Confirm:$false -ErrorAction SilentlyContinue
    $action = New-ScheduledTaskAction -Execute $exe -Argument $arguments
    $principal = New-ScheduledTaskPrincipal -UserId $User -LogonType Interactive -RunLevel Limited
    Register-ScheduledTask -TaskName $name -Action $action -Principal $principal | Out-Null
    Start-ScheduledTask -TaskName $name
    Start-Sleep 3
    Unregister-ScheduledTask -TaskName $name -Confirm:$false
}

if ($Install) {
    # openergy-run.cmd (next to this script) restarts the daemon if it crashes.
    Copy-Item (Join-Path $PSScriptRoot 'openergy-run.cmd') "$Dir\openergy-run.cmd" -ErrorAction SilentlyContinue
    $arguments = '--headless cmd.exe /d /c "' + "$Dir\openergy-run.cmd" + '"'
    $action = New-ScheduledTaskAction -Execute "$env:SystemRoot\System32\conhost.exe" -Argument $arguments
    $trigger = New-ScheduledTaskTrigger -AtLogOn -User $User
    $principal = New-ScheduledTaskPrincipal -UserId $User -LogonType Interactive -RunLevel Highest
    $settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries `
        -ExecutionTimeLimit ([TimeSpan]::Zero) -MultipleInstances IgnoreNew
    Register-ScheduledTask -TaskName 'Openergy' -Action $action -Trigger $trigger -Principal $principal -Settings $settings -Force | Out-Null
    Disable-ScheduledTask -TaskName 'Openergy' | Out-Null
    Log 'Openergy logon task registered (disabled)'
}

if ($Start) {
    if (-not (Test-Path $State)) {
        $value = (Get-ItemProperty $Approved -Name $RunName -ErrorAction SilentlyContinue).$RunName
        @{
            service_start = (Get-Service $Svc).StartType.ToString()
            approved = if ($value) { [Convert]::ToBase64String($value) } else { $null }
        } | ConvertTo-Json | Set-Content -Encoding utf8 $State
    }
    Log 'stopping the Synergy server'
    Stop-Service $Svc
    for ($i = 0; $i -lt 50 -and (Get-Process synergy-core -ErrorAction SilentlyContinue); $i++) { Start-Sleep -Milliseconds 200 }
    Stop-ByPath 'synergy-core' $Synergy
    Stop-ByPath 'synergy-tray' $Synergy
    Stop-ByPath 'synergy-service' $Synergy
    Log 'starting Openergy'
    Enable-ScheduledTask -TaskName 'Openergy' | Out-Null
    Start-ScheduledTask -TaskName 'Openergy'
    for ($i = 0; $i -lt 30; $i++) {
        Start-Sleep -Milliseconds 500
        $status = & $Exe ctl status.get --raw 2>$null | ConvertFrom-Json -ErrorAction SilentlyContinue
        if ($status -and $status.state -in 'listening', 'connected') { break }
    }
    if (-not $status -or $status.state -notin 'listening', 'connected') { throw "Openergy did not start listening: $($status.state) $($status.last_error)" }
    $proc = Get-Process openergy -ErrorAction SilentlyContinue | Where-Object Path -eq $Exe
    Log "Openergy $($status.state) on $($status.server.listen) (pid $($proc.Id), session $($proc.SessionId))"
}

if ($Commit) {
    Set-Service $Svc -StartupType Manual
    if (-not (Test-Path $Approved)) { New-Item $Approved -Force | Out-Null }
    # The value Task Manager writes when a startup app is disabled.
    Set-ItemProperty $Approved -Name $RunName -Type Binary -Value ([byte[]](3,0,0,0,0,0,0,0,0,0,0,0))
    Log 'Synergy stays off at the next logon; Openergy starts at logon'
}

if ($Rollback) {
    Log 'stopping Openergy'
    Disable-ScheduledTask -TaskName 'Openergy' -ErrorAction SilentlyContinue | Out-Null
    Stop-ScheduledTask -TaskName 'Openergy' -ErrorAction SilentlyContinue
    Start-Sleep 1
    # Stop the restart wrapper first, or it starts the daemon again.
    Get-CimInstance Win32_Process -Filter "Name='cmd.exe'" |
        Where-Object CommandLine -like '*openergy-run.cmd*' |
        ForEach-Object { Stop-Process -Id $_.ProcessId -Force }
    Stop-ByPath 'openergy' $Dir
    if (Test-Path $State) {
        $saved = Get-Content $State -Raw | ConvertFrom-Json
        Set-Service $Svc -StartupType $saved.service_start
        if ($saved.approved) {
            Set-ItemProperty $Approved -Name $RunName -Type Binary -Value ([Convert]::FromBase64String($saved.approved))
        } else {
            Remove-ItemProperty $Approved -Name $RunName -ErrorAction SilentlyContinue
        }
        Remove-Item $State
    }
    Log 'starting the Synergy server'
    Start-Service $Svc
    if (-not (Get-Process synergy-service -ErrorAction SilentlyContinue)) {
        Start-Interactive "$Synergy\synergy-service.exe" '-d'
    }
    Start-Sleep 3
    Get-Process synergy-core, synergy-service, synergy-tray -ErrorAction SilentlyContinue | ForEach-Object { Log "$($_.Name) $($_.Id) session $($_.SessionId)" }
}
