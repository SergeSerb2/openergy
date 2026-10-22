# Real session-0 CLI/pipe verification. No desktop input or production config.
$ErrorActionPreference = 'Stop'
if ([Diagnostics.Process]::GetCurrentProcess().SessionId -ne 0) { throw 'run only through session-0 SSH' }
$root = "$env:USERPROFILE\openergy-build"
$lab = "$root\lab\checks"
$exe = "$root\target\release\openergy.exe"
New-Item -ItemType Directory -Force $lab | Out-Null
$env:APPDATA = "$lab\appdata"
$env:OPENERGY_DISABLE_DISCOVERY = '1'
$env:OPENERGY_SOCKET = '\\.\pipe\openergy-port-control-lab'
$config = "$lab\config.toml"
@'
version = 1
name = "win-checks"
role = "off"
[server]
listen = "127.0.0.1:24823"
port = 24823
'@ | Set-Content $config
Add-Type -TypeDefinition @'
using System;
using System.IO;
using System.IO.Pipes;
using System.Runtime.InteropServices;
using System.Security.AccessControl;
using System.Security.Principal;
using System.Threading;
using System.Threading.Tasks;
public static class PipeLab {
 [DllImport("advapi32.dll")] static extern uint GetSecurityInfo(IntPtr h, int type, uint info, out IntPtr owner, out IntPtr group, out IntPtr dacl, out IntPtr sacl, out IntPtr sd);
 [DllImport("advapi32.dll", CharSet=CharSet.Unicode)] static extern bool ConvertSecurityDescriptorToStringSecurityDescriptor(IntPtr sd, uint rev, uint info, out IntPtr text, out uint size);
 [DllImport("kernel32.dll")] static extern IntPtr LocalFree(IntPtr p);
 [DllImport("kernel32.dll")] static extern bool AllocConsole();
 [DllImport("kernel32.dll")] static extern bool FreeConsole();
 delegate bool Handler(uint c);
 static Handler ignore = c => true;
 [DllImport("kernel32.dll")] static extern bool SetConsoleCtrlHandler(Handler h, bool add);
 [DllImport("kernel32.dll")] static extern uint GetConsoleProcessList([Out] uint[] ids, uint size);
 public static bool ConsoleContains(uint pid) {var ids=new uint[64]; GetConsoleProcessList(ids,64); return Array.IndexOf(ids,pid)>=0;}
 [DllImport("kernel32.dll")] public static extern bool GenerateConsoleCtrlEvent(uint c, uint group);
 public static void PrivateConsole() { FreeConsole(); if(!AllocConsole()) throw new Exception("AllocConsole"); if(!SetConsoleCtrlHandler(ignore,true)) throw new Exception("handler"); }
 public static void CheckAcl(NamedPipeClientStream stream) {
  IntPtr o,g,d,s,sd,text; uint size;
  if(GetSecurityInfo(stream.SafePipeHandle.DangerousGetHandle(),1,4,out o,out g,out d,out s,out sd)!=0) throw new Exception("GetSecurityInfo");
  try {
   if(!ConvertSecurityDescriptorToStringSecurityDescriptor(sd,1,4,out text,out size)) throw new Exception("SDDL conversion");
   try {
    var acl = new RawSecurityDescriptor(Marshal.PtrToStringUni(text));
    if(acl.DiscretionaryAcl.Count!=1 || (acl.ControlFlags & ControlFlags.DiscretionaryAclProtected)==0) throw new Exception("unexpected DACL shape");
    var ace = (CommonAce)acl.DiscretionaryAcl[0];
    if(ace.AceQualifier!=AceQualifier.AccessAllowed || ace.SecurityIdentifier!=WindowsIdentity.GetCurrent().User) throw new Exception("unexpected allowed SID");
   } finally { LocalFree(text); }
  } finally { LocalFree(sd); }
 }
 public static byte[] ReadExact(PipeStream stream,int length) {
  var b=new byte[length]; int off=0;
  while(off<length) { var task=stream.ReadAsync(b,off,length-off); if(!task.Wait(10000)) throw new Exception("read deadline"); int n=task.Result; if(n==0) throw new Exception("pipe closed"); off+=n; } return b;
 }
 public static Task Silent(string name) { return Task.Run(() => { using(var p=new NamedPipeServerStream(name,PipeDirection.InOut,1,PipeTransmissionMode.Byte,PipeOptions.Asynchronous)) { p.WaitForConnection(); Thread.Sleep(12000); } }); }
}
'@
$ownedProcesses=[Collections.Generic.List[Diagnostics.Process]]::new()
function Check($b,$message) { if(!$b) { throw $message }; Write-Output "PASS $message" }
function Start-Owned($label,$arguments) {
 $i = [System.Diagnostics.ProcessStartInfo]::new($exe)
 $i.UseShellExecute=$false; $i.RedirectStandardError=$true; $i.RedirectStandardOutput=$true
 foreach($a in $arguments) { $i.ArgumentList.Add($a) }
 $p=[System.Diagnostics.Process]::Start($i)
 $ownedProcesses.Add($p)
 # Drain logs asynchronously so no output pipe can block the daemon.
 $p | Add-Member NoteProperty ErrTask $p.StandardError.ReadToEndAsync()
 $p | Add-Member NoteProperty OutTask $p.StandardOutput.ReadToEndAsync()
 "$label $($p.Id) $exe" | Add-Content "$lab\processes.txt"
 return $p
}
function Connect-Pipe {
 $p=[System.IO.Pipes.NamedPipeClientStream]::new('.', 'openergy-port-control-lab', [System.IO.Pipes.PipeDirection]::InOut, [System.IO.Pipes.PipeOptions]::Asynchronous)
 $p.Connect(2000); return $p
}
function Write-Frame($p,$value) {
 $b=[System.Text.Encoding]::UTF8.GetBytes(($value | ConvertTo-Json -Compress -Depth 20))
 $h=[BitConverter]::GetBytes([int]$b.Length); [Array]::Reverse($h)
 $p.Write($h,0,4); $p.Write($b,0,$b.Length); $p.Flush()
}
function Read-Frame($p) {
 $h=[PipeLab]::ReadExact($p,4); [Array]::Reverse($h); $n=[BitConverter]::ToInt32($h,0)
 if($n -gt 16777216) {throw 'oversized response'}
 [System.Text.Encoding]::UTF8.GetString([PipeLab]::ReadExact($p,$n)) | ConvertFrom-Json
}
function Call($method,$params=@{}) {
 $result=& $exe ctl $method --raw --json ($params | ConvertTo-Json -Compress -Depth 20) 2> "$lab\cli.stderr"
 if($LASTEXITCODE) { throw "CLI failed: $method exit=$LASTEXITCODE" }; $result | ConvertFrom-Json
}
function Wait-Ready {
 for($i=0;$i -lt 50;$i++) {
  $probe=Start-Owned "ready-probe" @('status'); $probe.WaitForExit();
  if($probe.ExitCode -eq 0) {return}; Start-Sleep -Milliseconds 100
 }; throw 'startup deadline'
}
function Exit-Clean($p,$label) {
 Check ($p.WaitForExit(6000)) "$label exit deadline"
 Check ($p.ExitCode -eq 0) "$label exit=0"
 $p.ErrTask.Result | Set-Content "$lab\$label.stderr"
}
try {
$p=Start-Owned 'main' @('daemon','--platform','virtual','--config',$config)
Wait-Ready
$null=Call 'status'; Check ($LASTEXITCODE -eq 0) 'ctl status alias'
$null=& $exe status; Check ($LASTEXITCODE -eq 0) 'status command'
$null=& $exe disconnect; Check ($LASTEXITCODE -eq 0) 'disconnect command'
$null=& $exe connect; Check ($LASTEXITCODE -eq 0) 'connect command'
$null=Call 'config.patch' @{patch=@{clipboard=@{enabled=$false}}}
$cfg=Call 'config.get'; Check (!$cfg.config.clipboard.enabled) 'config patch saves and applies'
$dup=Start-Owned 'duplicate' @('daemon','--platform','virtual','--config',$config)
Check ($dup.WaitForExit(5000) -and $dup.ExitCode -eq 1 -and $dup.ErrTask.Result.Contains('another daemon owns')) 'second daemon rejected'
$sub=Connect-Pipe
[PipeLab]::CheckAcl($sub); Write-Output 'PASS kernel DACL: protected, one allow ACE, current user SID only'
Write-Frame $sub @{id=1;method='events.subscribe';params=@{}}
$r=Read-Frame $sub; Check ($r.id -eq 1 -and !$r.error) 'events subscribe response'
$null=Call 'config.patch' @{patch=@{clipboard=@{enabled=$true}}}
$seen=$false
for($i=0;$i -lt 5;$i++) { $r=Read-Frame $sub; if($r.event -eq 'config.changed') {$seen=$true;break} }
Check $seen 'event bus subscription delivery'
$sub.Dispose()
$sub=Connect-Pipe
Write-Frame $sub @{id=2;method='logs.subscribe';params=@{}}
$r=Read-Frame $sub; Check (!$r.error) 'logs subscribe response'
# Starting the server emits a log without any desktop access.
$null=Call 'config.patch' @{patch=@{role='server'}}
$seen=$false
for($i=0;$i -lt 8;$i++) { $r=Read-Frame $sub; if($r.event -eq 'log.line') {$seen=$true;break} }
Check $seen 'logs subscription delivery'
$sub.Dispose()
$null=Call 'config.patch' @{patch=@{role='off'}}
$fragment=Connect-Pipe
$b=[System.Text.Encoding]::UTF8.GetBytes('{"id":3,"method":"status.get","params":{}}'); $h=[BitConverter]::GetBytes([int]$b.Length); [Array]::Reverse($h)
foreach($v in $h) {$fragment.WriteByte($v);Start-Sleep -Milliseconds 60}
$fragment.Write($b,0,$b.Length); $r=Read-Frame $fragment
Check ($r.id -eq 3 -and !$r.error) 'fragmented frame survives read timeouts'
Write-Frame $fragment @{id=4;method='lab.unknown';params=@{}}; $r=Read-Frame $fragment
Check ($r.error.code -eq 'unknown_method') 'unknown method returns typed error'
$fragment.Dispose()
Start-Sleep -Milliseconds 150
$clients=[System.Collections.Generic.List[System.IO.Pipes.NamedPipeClientStream]]::new()
for($i=0;$i -lt 64;$i++) { $c=Connect-Pipe;Write-Frame $c @{id=$i;method='status.get';params=@{}};$r=Read-Frame $c;if($r.error){throw '64-client request'};$clients.Add($c) }
$extra=Connect-Pipe;$rejected=$false
try {Write-Frame $extra @{id=99;method='status.get';params=@{}};$r=Read-Frame $extra} catch {$rejected=$true} finally {$extra.Dispose()}
Check $rejected '65th client rejected'
$clients[0].Dispose(); Start-Sleep -Milliseconds 150; $null=Call 'status.get'; Write-Output 'PASS client slot recovers'
foreach($c in $clients){$c.Dispose()}
# An incomplete request must not keep shutdown waiting on a client thread.
$partial=Connect-Pipe;$partial.WriteByte(0)
$null=Call 'daemon.stop'; Exit-Clean $p 'ipc-stop'; $partial.Dispose()
$null=& $exe status 2> "$lab\absent.stderr"
Check ($LASTEXITCODE -eq 1 -and (Get-Content "$lab\absent.stderr" -Raw).Contains('is the daemon running?')) 'absent daemon error'
# Both watched processes use the owned executable and remain in session 0.
$parent=Start-Owned 'parent' @('daemon','--platform','virtual','--config',$config)
Wait-Ready
$child=Start-Owned 'watched' @('daemon','--platform','virtual','--config',$config,'--socket',"$lab\child.sock",'--parent-pid',"$($parent.Id)")
Start-Sleep -Milliseconds 500
Check (!$child.HasExited) 'watched daemon stays alive with parent'
$null=Call 'daemon.stop';Exit-Clean $parent 'parent';Exit-Clean $child 'parent-watched'
[PipeLab]::PrivateConsole()
$console=Start-Owned 'console' @('daemon','--platform','virtual','--config',$config)
Wait-Ready
Check ([PipeLab]::ConsoleContains([uint32]$console.Id)) 'daemon shares private session-0 console'
Check ([PipeLab]::GenerateConsoleCtrlEvent(0,0)) 'Ctrl+C sent to private session-0 console'
Exit-Clean $console 'ctrl-c'
# A silent local server exercises the actual CLI's 10-second overlapped read deadline.
$task=[PipeLab]::Silent('openergy-port-timeout-lab')
Start-Sleep -Milliseconds 200
$watch=[Diagnostics.Stopwatch]::StartNew()
$probe=Start-Owned "timeout-cli" @('status','--socket','\\.\pipe\openergy-port-timeout-lab'); $probe.WaitForExit()
Check ($probe.ExitCode -eq 1 -and $watch.Elapsed.TotalSeconds -ge 9 -and $watch.Elapsed.TotalSeconds -lt 12) 'CLI read timeout about 10 seconds'
$task.Wait()
Write-Output 'WINDOWS_CONTROL_PASS'

} finally {
 foreach($process in $ownedProcesses) {
  if(!$process.HasExited) {
   $live=Get-Process -Id $process.Id -ErrorAction SilentlyContinue
   if($live -and $live.Path -eq $exe -and $live.StartTime -eq $process.StartTime) {
    $live.Kill(); $null=$live.WaitForExit(5000)
   }
  }
 }
}
