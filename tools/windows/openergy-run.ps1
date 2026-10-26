# Owned restart supervisor. Called by openergy.ps1 or its per-user logon task.
[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][string]$InstallDir,
    [Parameter(Mandatory=$true)][string]$DataDir,
    [ValidateSet('windows','virtual')][string]$Platform = 'windows'
)
$ErrorActionPreference = 'Stop'
# Used only by unelevated per-user entry points. Never load user code as admin.
$oeIdentity = [Security.Principal.WindowsIdentity]::GetCurrent()
if ($oeIdentity.IsSystem -or ([Security.Principal.WindowsPrincipal]$oeIdentity).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw 'Run this per-user Openergy command from a non-administrator PowerShell. Only the protected migration helper may be elevated.'
}
if (-not ('OpenergyUserIntegrity' -as [type])) {
    Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Security.Principal;
public static class OpenergyUserIntegrity {
    [DllImport("advapi32.dll", SetLastError=true)] static extern bool GetTokenInformation(IntPtr token,int kind,IntPtr data,int length,out int needed);
    public static int Level() {
        using(var identity=WindowsIdentity.GetCurrent()) {
            int needed; GetTokenInformation(identity.Token,25,IntPtr.Zero,0,out needed);
            IntPtr data=Marshal.AllocHGlobal(needed);
            try {
                if(!GetTokenInformation(identity.Token,25,data,needed,out needed)) throw new System.ComponentModel.Win32Exception(Marshal.GetLastWin32Error());
                var sid=new SecurityIdentifier(Marshal.ReadIntPtr(data));
                string[] parts=sid.Value.Split('-'); return int.Parse(parts[parts.Length-1]);
            } finally { Marshal.FreeHGlobal(data); }
        }
    }
}
'@
}
if ([OpenergyUserIntegrity]::Level() -gt 8192) { throw 'Per-user Openergy commands require a medium or lower integrity token.' }

$InstallDir = [IO.Path]::GetFullPath($InstallDir).TrimEnd('\')
$DataDir = [IO.Path]::GetFullPath($DataDir).TrimEnd('\')
$Exe = Join-Path $InstallDir 'openergy.exe'
$Config = Join-Path $DataDir 'config.toml'
$Socket = Join-Path $DataDir 'openergy.sock'
$StopFile = Join-Path $DataDir 'stop-requested'
$RunFile = Join-Path $DataDir 'runner.json'
$lock = $null
try {
    # The handle, rather than a PID file, is the lifetime lock. Never delete it.
    $lock = [IO.File]::Open((Join-Path $DataDir 'runner.lock'), 'OpenOrCreate', 'ReadWrite', 'None')
    if (Test-Path -LiteralPath $StopFile) { exit 0 }
    $env:OPENERGY_CONFIG = $Config
    $env:OPENERGY_CONFIG_DIR = $DataDir
    $env:OPENERGY_TLS_DIR = Join-Path $DataDir 'tls'
    $env:OPENERGY_SOCKET = $Socket
    # The daemon owns port policy. Preserve the caller's explicit override, including 0.
    $self = Get-Process -Id $PID
    $record = @{pid=$PID;started=$self.StartTime.ToUniversalTime().Ticks.ToString()}
    $record | ConvertTo-Json | Set-Content -LiteralPath "$RunFile.new" -Encoding UTF8
    Move-Item -LiteralPath "$RunFile.new" -Destination $RunFile -Force
    function Publish-Identity {
        $record | ConvertTo-Json | Set-Content -LiteralPath "$RunFile.new" -Encoding UTF8
        Move-Item -LiteralPath "$RunFile.new" -Destination $RunFile -Force
    }
    $crashes = 0
    while (!(Test-Path -LiteralPath $StopFile)) {
        $started = Get-Date
        # Start-Process keeps native stderr out of PowerShell's error pipeline.
        $daemon = Start-Process -FilePath $Exe -ArgumentList @('daemon','--platform',$Platform,'--config',('"'+$Config+'"'),'--socket',('"'+$Socket+'"'),'--parent-pid',"$PID") -PassThru -NoNewWindow -RedirectStandardOutput (Join-Path $DataDir 'daemon.stdout.log') -RedirectStandardError (Join-Path $DataDir 'daemon.stderr.log')
        if (!$daemon.HasExited) {
            $record.child_pid = $daemon.Id
            $record.child_started = $daemon.StartTime.ToUniversalTime().Ticks.ToString()
            Publish-Identity
        }
        $daemon.WaitForExit()
        $daemon.Refresh()
        $record.Remove('child_pid')
        $record.Remove('child_started')
        Publish-Identity
        if ($daemon.ExitCode -eq 0 -or (Test-Path -LiteralPath $StopFile)) { break }
        if (((Get-Date) - $started).TotalSeconds -gt 60) { $crashes = 0 }
        $crashes++
        if ($crashes -ge 5) { throw 'Daemon failed five times; restart paused. Inspect daemon.stderr.log.' }
        for ($i=0; $i -lt 20 -and !(Test-Path -LiteralPath $StopFile); $i++) { Start-Sleep -Milliseconds 100 }
    }
} catch {
    $_ | Out-String | Add-Content -LiteralPath (Join-Path $DataDir 'runner.stderr.log')
    exit 1
} finally {
    if ($lock) {
        Remove-Item -LiteralPath $RunFile -ErrorAction SilentlyContinue
        $lock.Dispose()
    }
}
