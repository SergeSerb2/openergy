param([string]$Exe)
$ErrorActionPreference='Stop'
Add-Type -TypeDefinition @'
using System;
using System.IO.Pipes;
using System.Security.Principal;
using System.Threading.Tasks;
public static class PipeProbe {
 public static Task<int> Probe(string name) { return Task.Run(() => {
  using(var pipe = new NamedPipeServerStream(name, PipeDirection.InOut, 1, PipeTransmissionMode.Byte, PipeOptions.Asynchronous)) {
   if(!pipe.WaitForConnectionAsync().Wait(5000)) throw new Exception("connection deadline");
   if(pipe.ReadByte()<0) throw new Exception("missing request");
   int level=-1;
   pipe.RunAsClient(() => { using(var identity=WindowsIdentity.GetCurrent()) level=(int)identity.ImpersonationLevel; });
   return level;
  }
 }); }
}
'@
$lab=Join-Path ([IO.Path]::GetTempPath()) ('oe-pipe-identity-'+[guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $lab | Out-Null
$env:OPENERGY_CONFIG=Join-Path $lab 'probe/config.toml'
$env:OPENERGY_CONFIG_DIR=Join-Path $lab 'probe'
$env:OPENERGY_TLS_DIR=Join-Path $lab 'probe/tls'
$env:OPENERGY_DISABLE_DISCOVERY='1'
$name='oe-audit-probe-'+[guid]::NewGuid().ToString('N')
$task=[PipeProbe]::Probe($name)
Start-Sleep -Milliseconds 100
$info=[Diagnostics.ProcessStartInfo]::new($Exe)
$info.UseShellExecute=$false; $info.RedirectStandardError=$true; $info.RedirectStandardOutput=$true
foreach($a in @('status','--socket',"\\.\pipe\$name")) {$info.ArgumentList.Add($a)}
$p=[Diagnostics.Process]::Start($info)
try {
 if(!$task.Wait(7000)) {throw 'probe deadline'}
 $level=$task.Result
 "PIPE_CLIENT_IMPERSONATION_LEVEL=$level"
 if($level -ne 2) {throw 'CLI must permit identification only, not full token impersonation'}
 'PIPE_IMPERSONATION_PASS'
} finally { if(!$p.WaitForExit(3000)) {$p.Kill();$p.WaitForExit()}; Remove-Item -LiteralPath $lab -Recurse -Force }
