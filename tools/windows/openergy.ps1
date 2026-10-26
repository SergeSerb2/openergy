# Per-user install and lifecycle control. No Synergy, services, or firewall changes.
[CmdletBinding()]
param(
    [Parameter(Mandatory=$true, Position=0)]
    [ValidateSet('Install','Start','Stop','Status','Uninstall','Validate')][string]$Action,
    [string]$InstallDir = "$env:LOCALAPPDATA\Programs\Openergy",
    [string]$DataDir = "$env:APPDATA\Openergy",
    [string]$BinaryPath = "$PSScriptRoot\openergy.exe",
    [ValidateSet('windows','virtual')][string]$Platform = 'windows',
    [ValidateRange(1,65535)][int]$Port = 24801,
    [switch]$NoStartup
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
# Authenticate the actual connected pipe before sending any request. A separate
# status CLI followed by a stop CLI would leave a server-replacement race.
if (-not ('OpenergyManagedPipe' -as [type])) {
    Add-Type -TypeDefinition @'
using System;
using System.Diagnostics;
using System.IO;
using System.IO.Pipes;
using System.Runtime.InteropServices;
using System.Security.Principal;
using System.Text;
public static class OpenergyManagedPipe {
    [DllImport("kernel32.dll", SetLastError=true, CharSet=CharSet.Unicode)]
    static extern bool WaitNamedPipeW(string name, uint timeout);
    [DllImport("kernel32.dll", SetLastError=true)]
    static extern bool GetNamedPipeServerProcessId(IntPtr pipe, out uint pid);
    static string Name(string socket) {
        string path=Path.GetFullPath(socket).Replace('/', '\\');
        char[] chars=path.ToCharArray();
        for(int i=0;i<chars.Length;i++) if(chars[i]>='A' && chars[i]<='Z') chars[i]=(char)(chars[i]+32);
        string value=WindowsIdentity.GetCurrent().User.Value+"\0"+new string(chars);
        ulong hash=0xcbf29ce484222325UL;
        unchecked { foreach(byte b in new UTF8Encoding(false,true).GetBytes(value)) hash=(hash^b)*0x100000001b3UL; }
        return "openergy-"+hash.ToString("x16");
    }
    public static bool Occupied(string socket) {
        if(WaitNamedPipeW(@"\\.\pipe\"+Name(socket),1)) return true;
        // A busy, denied, or otherwise inaccessible endpoint is never free.
        return Marshal.GetLastWin32Error()!=2;
    }
    static byte[] Read(PipeStream pipe,int length) {
        byte[] bytes=new byte[length]; int offset=0;
        while(offset<length) {
            var task=pipe.ReadAsync(bytes,offset,length-offset);
            if(!task.Wait(5000)) throw new TimeoutException("Managed control read deadline");
            int n=task.Result; if(n==0) throw new IOException("Managed control pipe closed"); offset+=n;
        }
        return bytes;
    }
    public static string Call(string socket,int childPid,long started,string exe,string request) {
        using(var child=Process.GetProcessById(childPid))
        using(var pipe=new NamedPipeClientStream(".",Name(socket),PipeDirection.InOut,PipeOptions.Asynchronous,TokenImpersonationLevel.Identification)) {
            pipe.Connect(1000);
            uint server;
            if(!GetNamedPipeServerProcessId(pipe.SafePipeHandle.DangerousGetHandle(),out server))
                throw new System.ComponentModel.Win32Exception(Marshal.GetLastWin32Error());
            if(server!=(uint)childPid || child.HasExited || child.StartTime.ToUniversalTime().Ticks!=started ||
                !String.Equals(child.MainModule.FileName,exe,StringComparison.OrdinalIgnoreCase))
                throw new InvalidOperationException("Control endpoint is not the supervised daemon");
            byte[] body=Encoding.UTF8.GetBytes(request);
            byte[] frame=new byte[body.Length+4]; int n=body.Length;
            frame[0]=(byte)(n>>24);frame[1]=(byte)(n>>16);frame[2]=(byte)(n>>8);frame[3]=(byte)n;
            Buffer.BlockCopy(body,0,frame,4,n);
            if(!pipe.WriteAsync(frame,0,frame.Length).Wait(5000)) throw new TimeoutException("Managed control write deadline");
            byte[] header=Read(pipe,4);
            uint size=((uint)header[0]<<24)|((uint)header[1]<<16)|((uint)header[2]<<8)|header[3];
            if(size>16777216) throw new IOException("Oversized managed control response");
            return Encoding.UTF8.GetString(Read(pipe,(int)size));
        }
    }
}
'@
}
function Owned-Child {
    if (!(Runner)) { return $null }
    $record = Get-Content -LiteralPath $RunFile -Raw | ConvertFrom-Json
    if (!$record.PSObject.Properties['child_pid'] -or !$record.PSObject.Properties['child_started']) { return $null }
    $child = Get-Process -Id $record.child_pid -ErrorAction SilentlyContinue
    if ($child -and $child.Path -eq $Exe -and $child.StartTime.ToUniversalTime().Ticks.ToString() -eq $record.child_started) { return $child }
    return $null
}
function Ctl([string]$Method, [Diagnostics.Process]$Child) {
    if (!$Child) { throw 'Managed daemon identity is unavailable' }
    $request = @{id=1;method=$Method;params=@{}} | ConvertTo-Json -Compress
    $response = [OpenergyManagedPipe]::Call($Socket,$Child.Id,$Child.StartTime.ToUniversalTime().Ticks,$Exe,$request) | ConvertFrom-Json
    if ($response.PSObject.Properties['error'] -and $response.error) { throw "Managed control failed: $($response.error.message)" }
    if ($Method -eq 'status.get' -and $response.result.daemon.pid -ne $Child.Id) { throw 'Control endpoint is not the supervised daemon' }
    $response.result
}
function Task-Owned($Saved) {
    $task = Get-ScheduledTask -TaskName $Saved.task -ErrorAction SilentlyContinue
    if ($task) {
        $expectedArguments = '-NoLogo -NoProfile -NonInteractive -WindowStyle Hidden -ExecutionPolicy Bypass -File "' + (Join-Path $InstallDir 'openergy-run.ps1') + '" -InstallDir "' + $InstallDir + '" -DataDir "' + $DataDir + '" -Platform windows'
        if (@($task).Count -ne 1 -or $task.TaskPath -ne '\' -or $task.Description -ne "Openergy managed install: $InstallDir | $DataDir" -or $task.Principal.UserId -notin @($Sid, [Security.Principal.WindowsIdentity]::GetCurrent().Name) -or $task.Principal.RunLevel.ToString() -ne 'Limited' -or @($task.Actions).Count -ne 1 -or $task.Actions[0].Execute -ne $PowerShell -or $task.Actions[0].Arguments -ne $expectedArguments -or $task.Actions[0].WorkingDirectory -ne $InstallDir) {
            throw 'Scheduled task owner, limited token, or action does not match this installation'
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
        $child = Owned-Child
        if ($child) {
            try { $null = Ctl 'daemon.stop' $child } catch { Write-Warning "Owned daemon stop request failed: $($_.Exception.Message)" }
        }
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
function Validate-Install {
    foreach ($path in @($Exe,$Config,(Join-Path $InstallDir 'openergy-run.ps1'))) {
        if (!(Test-Path -LiteralPath $path -PathType Leaf)) { throw 'Managed installation is incomplete' }
    }
    if (!$saved.no_startup -and !(Task-Owned $saved)) { throw 'Startup task is missing; installation preserved' }
    if (Runner) {
        $child = Owned-Child
        if (!$child) { throw 'Managed daemon identity is unavailable' }
        $null = Ctl 'status.get' $child
    } elseif ([OpenergyManagedPipe]::Occupied($Socket)) {
        throw 'Control endpoint is already occupied by another daemon'
    }
}
if ($Action -eq 'Validate') { Validate-Install; $saved; return }
if ($Action -eq 'Start') {
    Validate-Install
    if (Runner) { Ctl 'status.get' (Owned-Child); return }
    Remove-Item -LiteralPath $StopFile -ErrorAction SilentlyContinue
    if ($saved.no_startup) {
        $arguments = '-NoLogo -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "' + (Join-Path $InstallDir 'openergy-run.ps1') + '" -InstallDir "' + $InstallDir + '" -DataDir "' + $DataDir + '" -Platform ' + $saved.platform
        Start-Process -FilePath $PowerShell -ArgumentList $arguments -WorkingDirectory $InstallDir -WindowStyle Hidden | Out-Null
    } else {
        $task = Task-Owned $saved
        Enable-ScheduledTask -InputObject $task | Out-Null
        Start-ScheduledTask -InputObject $task
    }
    try {
        $deadline = (Get-Date).AddSeconds(30)
        while ((Get-Date) -lt $deadline) {
            Start-Sleep -Milliseconds 100
            $child = Owned-Child
            if (!$child) { continue }
            if (![OpenergyManagedPipe]::Occupied($Socket)) { continue }
            try { $status = Ctl 'status.get' $child } catch {
                if ($_.Exception.ToString().Contains('not the supervised daemon')) { throw }
                continue
            }
            if ($status.last_error) { throw "Daemon startup failed: $($status.last_error)" }
            $status; return
        }
        throw 'Daemon did not become ready; see daemon.stderr.log and runner.stderr.log in DataDir'
    } catch {
        $failure = $_
        try { Stop-Owned $saved } catch { Write-Warning "Startup cleanup failed: $($_.Exception.Message)" }
        throw $failure
    }
}
if ($Action -eq 'Status') {
    if (!(Runner)) { throw 'Managed runner is stopped' }
    Ctl 'status.get' (Owned-Child); return
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
