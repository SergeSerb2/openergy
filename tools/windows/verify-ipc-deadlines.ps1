# Real session-0 pipe/CLI deadline checks. No desktop backend or production state.
param(
    [Parameter(Mandatory=$true)][string]$BinaryPath,
    [Parameter(Mandatory=$true)][string]$ReportPath,
    [string]$LabRoot
)
$ErrorActionPreference='Stop'
$ProgressPreference='SilentlyContinue'
if($PSVersionTable.PSVersion.Major -lt 7){throw 'PowerShell 7 is required'}
if([Diagnostics.Process]::GetCurrentProcess().SessionId -ne 0){throw 'run only through session-0 SSH'}
$exe=(Resolve-Path -LiteralPath $BinaryPath).Path
$binaryHash=(Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash.ToLowerInvariant()
$reportFile=[IO.Path]::GetFullPath($ReportPath)
if(Test-Path -LiteralPath $reportFile){throw 'report already exists'}
if(!(Test-Path -LiteralPath ([IO.Path]::GetDirectoryName($reportFile)) -PathType Container)){throw 'report parent must exist'}
$lab=if($LabRoot){[IO.Path]::GetFullPath($LabRoot)}else{Join-Path ([IO.Path]::GetTempPath()) ('oe-pipe-deadline-'+[guid]::NewGuid().ToString('N'))}
if(Test-Path -LiteralPath $lab){throw 'lab root must be new'}
New-Item -ItemType Directory -Path $lab | Out-Null
$sid=[Security.Principal.WindowsIdentity]::GetCurrent().User
$acl=[Security.AccessControl.DirectorySecurity]::new()
$acl.SetAccessRuleProtection($true,$false)
$acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new($sid,'FullControl','ContainerInherit,ObjectInherit','None','Allow'))
Set-Acl -LiteralPath $lab -AclObject $acl
Set-Location -LiteralPath $lab
(Get-Process -Id $PID).PriorityClass='BelowNormal'
$env:TEMP=Join-Path $lab 'tmp';$env:TMP=$env:TEMP
New-Item -ItemType Directory -Path $env:TEMP | Out-Null
$pipeName='oe-deadline-'+[guid]::NewGuid().ToString('N')
$endpoint='\\.\pipe\'+$pipeName
$config=Join-Path $lab 'config.toml'
[IO.File]::WriteAllText($config,"version=1`nname=`"pipe-deadline-lab`"`nrole=`"off`"`n[tls]`nenabled=false`n[server]`nlisten=`"127.0.0.1:25249`"`nport=25249`n")
$childEnv=@{OPENERGY_ALLOW_PORT_24800='0';OPENERGY_DISABLE_DISCOVERY='1';OPENERGY_CONFIG=$config;OPENERGY_CONFIG_DIR=$lab;OPENERGY_TLS_DIR=(Join-Path $lab 'tls');OPENERGY_SOCKET=$endpoint;APPDATA=(Join-Path $lab 'appdata');HOSTNAME='openergy-deadline-lab'}
$report=[ordered]@{binary_sha256=$binaryHash;script_sha256=(Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash.ToLowerInvariant();platform='virtual';transport='private Windows named pipe';session_id=0;checks=@();passed=$false}
$pipes=[Collections.Generic.List[IO.Pipes.NamedPipeClientStream]]::new()
$daemon=$null;$fatal=$null
function Check([string]$Name,[bool]$Passed,[hashtable]$Detail=@{}){
    $item=[ordered]@{name=$Name;passed=$Passed}
    foreach($entry in $Detail.GetEnumerator()){$item[$entry.Key]=$entry.Value}
    $script:report.checks+=$item
    [Console]::WriteLine($(if($Passed){'PASS '}else{'FAIL '})+$Name)
}
function Start-Owned([string[]]$Arguments){
    if((Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash -ne $binaryHash){throw 'frozen binary changed'}
    $info=[Diagnostics.ProcessStartInfo]::new($exe)
    $info.WorkingDirectory=$lab
    $info.UseShellExecute=$false;$info.CreateNoWindow=$true;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
    foreach($argument in $Arguments){$info.ArgumentList.Add($argument)}
    foreach($entry in $childEnv.GetEnumerator()){$info.Environment[$entry.Key]=$entry.Value}
    $process=[Diagnostics.Process]::Start($info)
    $process | Add-Member NoteProperty LabStarted $process.StartTime
    $process | Add-Member NoteProperty OutTask $process.StandardOutput.ReadToEndAsync()
    $process | Add-Member NoteProperty ErrTask $process.StandardError.ReadToEndAsync()
    return $process
}
function Stop-OnlyOwned($Process){
    if(!$Process.HasExited){
        $live=Get-Process -Id $Process.Id -ErrorAction Stop
        if($live.Path -ne $exe -or $live.StartTime -ne $Process.LabStarted -or (Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash -ne $binaryHash){throw 'owned cleanup identity/hash mismatch'}
        $live.Kill();if(!$live.WaitForExit(5000)){throw 'owned cleanup deadline'}
    }
}
function Invoke-DeadlineCli([string]$Method){
    $process=Start-Owned @('ctl',$Method,'--socket',$endpoint,'--raw')
    try{
        if(!$process.WaitForExit(4500)){
            Stop-OnlyOwned $process
            return @{exit=-1;stdout='';stderr='public CLI observation deadline'}
        }
        return @{exit=$process.ExitCode;stdout=$process.OutTask.Result;stderr=$process.ErrTask.Result}
    }finally{$process.Dispose()}
}
Add-Type -TypeDefinition @'
using System;
using System.IO;
using System.IO.Pipes;
using System.Security.Principal;
using System.Threading.Tasks;
public static class DeadlineWire {
 public static NamedPipeClientStream Connect(string name,int timeout) {
  var p=new NamedPipeClientStream(".",name,PipeDirection.InOut,PipeOptions.Asynchronous,TokenImpersonationLevel.Identification);
  try {p.Connect(timeout);return p;} catch {p.Dispose();throw;}
 }
 public static byte[] Frame(string json) {
  var body=System.Text.Encoding.UTF8.GetBytes(json);
  var result=new byte[body.Length+4];var n=BitConverter.GetBytes(body.Length);Array.Reverse(n);
  Buffer.BlockCopy(n,0,result,0,4);Buffer.BlockCopy(body,0,result,4,body.Length);return result;
 }
 public static byte[] Join(byte[] a,byte[] b) {
  var bytes=new byte[a.Length+b.Length];Buffer.BlockCopy(a,0,bytes,0,a.Length);Buffer.BlockCopy(b,0,bytes,a.Length,b.Length);return bytes;
 }
 public static byte[] Slice(byte[] bytes,int offset,int length) {
  var part=new byte[length];Buffer.BlockCopy(bytes,offset,part,0,length);return part;
 }
 public static void Send(PipeStream pipe,byte[] bytes) {pipe.Write(bytes,0,bytes.Length);pipe.Flush();}
 public static byte[] Exact(PipeStream pipe,int count) {
  var bytes=new byte[count];int offset=0;
  while(offset<count){var read=pipe.ReadAsync(bytes,offset,count-offset);if(!read.Wait(3000))throw new IOException("reply deadline");int n=read.Result;if(n==0)throw new EndOfStreamException();offset+=n;}return bytes;
 }
 public static string Response(PipeStream pipe) {
  var prefix=Exact(pipe,4);Array.Reverse(prefix);int count=BitConverter.ToInt32(prefix,0);
  if(count<0||count>16777216)throw new IOException("reply exceeds IPC frame cap");
  return System.Text.Encoding.UTF8.GetString(Exact(pipe,count));
 }
 public static Task<int> AwaitClosure(PipeStream pipe) {return pipe.ReadAsync(new byte[1],0,1);}
 public static bool Closed(Task<int> read) {
  if(!read.IsCompleted)return false;
  if(read.IsFaulted){foreach(var error in read.Exception.Flatten().InnerExceptions)if(!(error is IOException))throw error;return true;}
  return read.Result==0;
 }
}
'@
function Connect-Pipe {
    $stream=[DeadlineWire]::Connect($pipeName,1500);$pipes.Add($stream);return $stream
}
function Frame([string]$Method,[int]$Id){return [DeadlineWire]::Frame((@{id=$Id;method=$Method;params=@{}} | ConvertTo-Json -Compress))}
function Response($Pipe){return ([DeadlineWire]::Response($Pipe) | ConvertFrom-Json)}
try{
    $daemon=Start-Owned @('daemon','--foreground','--platform','virtual','--config',$config,'--socket',$endpoint)
    $ready=[Diagnostics.Stopwatch]::StartNew();$initial=$null
    while($ready.ElapsedMilliseconds -lt 8000 -and !$daemon.HasExited){
        $initial=Invoke-DeadlineCli 'status.get'
        if($initial.exit -eq 0){break};Start-Sleep -Milliseconds 50
    }
    Check 'initial public CLI status' ($null -ne $initial -and $initial.exit -eq 0)
    if(!$initial -or $initial.exit -ne 0){throw 'private daemon did not become ready'}
    $status=$initial.stdout | ConvertFrom-Json
    if($status.daemon.platform -ne 'virtual' -or $status.daemon.pid -ne $daemon.Id){throw 'unexpected daemon identity/platform'}
    $idle=Connect-Pipe
    [DeadlineWire]::Send($idle,(Frame 'status.get' 1))
    Check 'persistent client initial reply' ((Response $idle).id -eq 1)
    Start-Sleep -Milliseconds 100
    $pending=@()
    for($index=0;$index -lt 63;$index++){
        $stream=Connect-Pipe;$watch=[Diagnostics.Stopwatch]::StartNew()
        $kind=@('empty','partial_header','partial_body','drip')[$index % 4]
        if($kind -eq 'partial_header'){[DeadlineWire]::Send($stream,[byte[]]@(0,0))}
        elseif($kind -eq 'partial_body' -or $kind -eq 'drip'){[DeadlineWire]::Send($stream,[byte[]]@(0,0,4,0,123))}
        $pending+=@{kind=$kind;pipe=$stream;watch=$watch;read=[DeadlineWire]::AwaitClosure($stream);closed_ms=$null}
        Start-Sleep -Milliseconds 3
    }
    Check 'all 63 pending clients initially admitted' (@($pending | Where-Object {$_.read.IsCompleted}).Count -eq 0)
    $window=[Diagnostics.Stopwatch]::StartNew();$nextDrip=0
    while($window.ElapsedMilliseconds -lt 11500){
        if($window.ElapsedMilliseconds -ge $nextDrip){
            foreach($item in $pending){if($item.kind -eq 'drip' -and !$item.read.IsCompleted){try{[DeadlineWire]::Send($item.pipe,[byte[]]@(32))}catch [IO.IOException]{}}}
            $nextDrip+=600
        }
        foreach($item in $pending){if($null -eq $item.closed_ms -and $item.read.IsCompleted){$item.closed_ms=$item.watch.ElapsedMilliseconds}}
        Start-Sleep -Milliseconds 25
    }
    foreach($kind in @('empty','partial_header','partial_body','drip')){
        $group=@($pending | Where-Object kind -eq $kind)
        $closed=@($group | Where-Object {[DeadlineWire]::Closed($_.read)}).Count
        $times=@($group | Where-Object {$null -ne $_.closed_ms} | ForEach-Object {$_.closed_ms})
        $minimum=($times | Measure-Object -Minimum).Minimum;$maximum=($times | Measure-Object -Maximum).Maximum
        Check ($kind+' clients expire') ($closed -eq $group.Count -and $times.Count -eq $group.Count -and $minimum -ge 9000 -and $maximum -le 12500) @{closed=$closed;total=$group.Count;min_observed_ms=$minimum;max_observed_ms=$maximum}
    }
    Check 'public CLI recovers while incomplete clients stay open' ((Invoke-DeadlineCli 'status.get').exit -eq 0)
    [DeadlineWire]::Send($idle,(Frame 'status.get' 2))
    Check 'completed-request idle client remains usable' ((Response $idle).id -eq 2)
    $pending[0].pipe.Dispose();Start-Sleep -Milliseconds 100
    Check 'releasing one held client slot restores CLI access' ((Invoke-DeadlineCli 'status.get').exit -eq 0)
    foreach($item in $pending){$item.pipe.Dispose()}
    $fragmented=Connect-Pipe;$request=Frame 'status.get' 3
    foreach($part in @([DeadlineWire]::Slice($request,0,2),[DeadlineWire]::Slice($request,2,7),[DeadlineWire]::Slice($request,9,$request.Length-9))){[DeadlineWire]::Send($fragmented,$part);Start-Sleep -Milliseconds 100}
    Check 'fragmented complete request succeeds' ((Response $fragmented).id -eq 3)
    [DeadlineWire]::Send($fragmented,[DeadlineWire]::Join((Frame 'status.get' 4),(Frame 'status.get' 5)))
    $first=Response $fragmented;$second=Response $fragmented
    Check 'pipelined complete requests preserve reply order' ($first.id -eq 4 -and $second.id -eq 5)
    [DeadlineWire]::Send($fragmented,(Frame 'no.such.method' 6));$invalid=Response $fragmented
    Check 'unknown method returns an error' ($invalid.id -eq 6 -and $null -ne $invalid.error)
    [DeadlineWire]::Send($fragmented,(Frame 'status.get' 7))
    Check 'valid request after method error succeeds' ((Response $fragmented).id -eq 7)
    [DeadlineWire]::Send($fragmented,[byte[]]@(0,0));$partialRead=[DeadlineWire]::AwaitClosure($fragmented)
    Start-Sleep -Milliseconds 11000
    Check 'partial frame after a completed request expires' ([DeadlineWire]::Closed($partialRead))
    $oversize=Connect-Pipe;[DeadlineWire]::Send($oversize,[byte[]]@(1,0,0,1));$oversizeRead=[DeadlineWire]::AwaitClosure($oversize)
    try{[void]$oversizeRead.Wait(1500)}catch [AggregateException]{}
    Check 'oversized header is refused' ([DeadlineWire]::Closed($oversizeRead))
    Check 'public CLI works after refusal' ((Invoke-DeadlineCli 'status.get').exit -eq 0)
    $partial=Connect-Pipe;[DeadlineWire]::Send($partial,[byte[]]@(0))
    $stop=Invoke-DeadlineCli 'daemon.stop';$exited=$daemon.WaitForExit(4500)
    Check 'shutdown with a partial client exits cleanly' ($stop.exit -eq 0 -and $exited -and $daemon.ExitCode -eq 0)
    $endpointClosed=$false
    try{$extra=[DeadlineWire]::Connect($pipeName,250);$extra.Dispose()}catch [TimeoutException]{$endpointClosed=$true}catch [IO.IOException]{$endpointClosed=$true}
    Check 'owned IPC endpoint removed' $endpointClosed
}catch{
    $fatal=$_.Exception.Message
    Check 'verification completed' $false @{error=$fatal}
}finally{
    foreach($stream in $pipes){$stream.Dispose()}
    if($daemon -and !$daemon.HasExited){
        try{$null=Invoke-DeadlineCli 'daemon.stop';if(!$daemon.WaitForExit(4500)){Stop-OnlyOwned $daemon}}catch{Stop-OnlyOwned $daemon}
    }
    if($daemon){
        $report.daemon_exit=$daemon.ExitCode
        [IO.File]::WriteAllText((Join-Path $lab 'daemon.stdout'),$daemon.OutTask.Result)
        [IO.File]::WriteAllText((Join-Path $lab 'daemon.stderr'),$daemon.ErrTask.Result)
    }
    $report.cleanup=$null -ne $daemon -and $daemon.HasExited
    $report.passed=$null -eq $fatal -and $report.cleanup -and $report.daemon_exit -eq 0 -and $report.checks.Count -eq 19 -and @($report.checks | Where-Object {!$_.passed}).Count -eq 0
    $report.evidence_dir=$lab
    $report | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $reportFile -Encoding utf8
    if($daemon){$daemon.Dispose()}
}
if(!$report.passed){exit 1}
[Console]::WriteLine('WINDOWS_IPC_DEADLINES_PASS checks=19')
