. (Join-Path $PSScriptRoot 'common.ps1')
if(Test-Path -LiteralPath (Join-Path $root 'runtime-once.guard')){throw 'runtime already attempted; no retry'}
[IO.File]::WriteAllText((Join-Path $root 'runtime-once.guard'),[DateTime]::UtcNow.ToString('o'))
$build=Get-Content -Raw -LiteralPath (Join-Path $root 'build-result.json')|ConvertFrom-Json
if(-not $build.passed){throw 'successful native build required'}
$binary=Join-Path $root 'target/x86_64-pc-windows-msvc/release/openergy.exe'
if((Hash $binary) -ne $build.artifact.sha256 -or (Get-Item -LiteralPath $binary).Length -ne $build.artifact.bytes){throw 'runtime artifact binding failed'}
$run=Join-Path $root 'runtime';New-Item -ItemType Directory -Path $run|Out-Null
$originalProfile=$env:USERPROFILE
foreach($n in @('profile','appdata','localappdata','config','data','cache','ipc','temp')){New-Item -ItemType Directory -Path (Join-Path $run $n)|Out-Null}
$daemonPipeName='oe-stop-c8a3489c-daemon-__NONCE__'
$relayPipeName='oe-stop-c8a3489c-observer-__NONCE__'
$daemonPipe='\\.\pipe\'+$daemonPipeName;$relayPipe='\\.\pipe\'+$relayPipeName
$config=Join-Path $run 'config/config.toml'
$env:USERPROFILE=Join-Path $run 'profile';$env:APPDATA=Join-Path $run 'appdata';$env:LOCALAPPDATA=Join-Path $run 'localappdata'
$env:XDG_CONFIG_HOME=Join-Path $run 'config';$env:XDG_DATA_HOME=Join-Path $run 'data';$env:XDG_CACHE_HOME=Join-Path $run 'cache';$env:XDG_RUNTIME_DIR=Join-Path $run 'ipc'
$env:TEMP=Join-Path $run 'temp';$env:TMP=$env:TEMP
$env:OPENERGY_CONFIG=$config;$env:OPENERGY_SOCKET=$daemonPipe;$env:OPENERGY_DISABLE_DISCOVERY='1';$env:OPENERGY_ALLOW_PORT_24800='0';$env:OPENERGY_LOG='info'
$configText=@'
version = 1
role = "off"
name = "native-stop-c8a3489c"
[server]
listen = "127.0.0.1:59367"
port = 59367
[client]
remote = ""
autoconnect = false
[tls]
enabled = false
[clipboard]
enabled = false
send = false
receive = false
'@
[IO.File]::WriteAllText($config,$configText,[Text.UTF8Encoding]::new($false))
Add-Type -Path (Join-Path $root 'observer.cs')
$controller=Get-Process -Id $PID;$controllerIdentity=Identity $controller
if($controllerIdentity.priority -ne 'BelowNormal' -or $controllerIdentity.session -ne 0){throw 'controller priority/session mismatch'}
Save 'runtime-controller.json' (Ancestors $controller)
$script:children=[Collections.Generic.List[object]]::new();$script:exchanges=[Collections.Generic.List[object]]::new();$script:daemon=$null
$script:stopDeadline=0L;$script:daemonExitTicks=0L;$script:stopCalls=0
$result=[ordered]@{passed=$false;source_commit=$build.source_commit;artifact=$build.artifact;controller=$controllerIdentity;bounds=[ordered]@{readiness_seconds=15;cli_seconds=15;normal_exit_seconds=10;runtime_stage_seconds=150;owned_recovery_seconds=15};frequency=[Diagnostics.Stopwatch]::Frequency;clock='Windows QPC / Stopwatch.GetTimestamp in the same native PowerShell controller and observer';timing_limit='Includes transparent pipe observer and scheduling. Request forwarding is bounded by before/after ticks. Response read is controller receipt, not daemon send. OS exit is first retained-handle observation, not kernel exit timestamp. No cross-platform or speedup claim.';daemon_pipe=$daemonPipe;observer_pipe=$relayPipe;config_sha256=(Hash $config);environment=[ordered]@{USERPROFILE=$env:USERPROFILE;APPDATA=$env:APPDATA;LOCALAPPDATA=$env:LOCALAPPDATA;XDG_CONFIG_HOME=$env:XDG_CONFIG_HOME;XDG_DATA_HOME=$env:XDG_DATA_HOME;XDG_CACHE_HOME=$env:XDG_CACHE_HOME;XDG_RUNTIME_DIR=$env:XDG_RUNTIME_DIR;TEMP=$env:TEMP;TMP=$env:TMP;OPENERGY_CONFIG=$env:OPENERGY_CONFIG;OPENERGY_SOCKET=$env:OPENERGY_SOCKET;OPENERGY_DISABLE_DISCOVERY=$env:OPENERGY_DISABLE_DISCOVERY;OPENERGY_ALLOW_PORT_24800=$env:OPENERGY_ALLOW_PORT_24800};requests=@();stops_sent=0;recovery=@();jobs_created=0;process_groups_created=0;desktop_or_windowstation_changed=$false}
Save 'runtime-start.json' $result
function Pipe-Exists($name){@([IO.Directory]::EnumerateFiles('\\.\pipe\')|Where-Object {[IO.Path]::GetFileName($_) -eq $name}).Count -gt 0}
function Port-State($phase){
 $sockets=[Collections.Generic.List[object]]::new()
 try{
  foreach($family in @([Net.Sockets.AddressFamily]::InterNetwork,[Net.Sockets.AddressFamily]::InterNetworkV6)){
   $s=[Net.Sockets.Socket]::new($family,[Net.Sockets.SocketType]::Stream,[Net.Sockets.ProtocolType]::Tcp);$sockets.Add($s);$s.ExclusiveAddressUse=$true
   if($family -eq [Net.Sockets.AddressFamily]::InterNetworkV6){$s.DualMode=$false;$address=[Net.IPAddress]::IPv6Loopback}else{$address=[Net.IPAddress]::Loopback}
   $s.Bind([Net.IPEndPoint]::new($address,59367))
  }
  $value=[ordered]@{phase=$phase;port=59367;ipv4_and_ipv6_exclusive_bind_passed=$true;listen_called=$false;connect_called=$false;recorded_utc=[DateTime]::UtcNow.ToString('o')}
  Save ('runtime-port-'+$phase+'.json') $value
 }finally{foreach($s in $sockets){$s.Dispose()}}
}
function New-Child($label,[string[]]$arguments){
 $child=[WinOwned]::Start($binary,$arguments,$run,(Join-Path $root ($label+'.stdout')),(Join-Path $root ($label+'.stderr')))
 $script:children.Add($child)
 $id=Identity $child.Process
 $id.native_process_handle_retained=$true;$id.primary_thread_suspended=$true;$id.in_job=$child.InJob;$id.below_normal_at_creation=$true;$id.explicit_inherited_handles=@('private stdout','private stderr','NUL stdin');$id.command=$arguments
 Save ($label+'-identity.json') $id
 if($id.priority -ne 'BelowNormal' -or $id.session -ne 0 -or $id.parent_pid -ne $PID){throw ('owned product identity/priority/session mismatch '+$label)}
 return $child
}
function Invoke-Cli($label,[string[]]$arguments,$expected,$method){
 $exchange=$null
 if($method){$exchange=[WinOwned+Exchange]::new($relayPipeName,$daemonPipeName);$script:exchanges.Add($exchange)}
 $child=New-Child $label $arguments
 $startTicks=[Diagnostics.Stopwatch]::GetTimestamp();$child.Resume()
 $isStop=$method -eq 'daemon.stop'
 if($isStop){$script:stopCalls++}
 $cliExitTicks=0L;$timedOut=$false
 while($true){
  $now=[Diagnostics.Stopwatch]::GetTimestamp()
  if($child.Process.WaitForExit(0) -and $cliExitTicks -eq 0){$cliExitTicks=[Diagnostics.Stopwatch]::GetTimestamp()}
  if($isStop){
   if($exchange.ForwardBeforeTicks -gt 0){$script:stopDeadline=$exchange.ForwardBeforeTicks+10L*[Diagnostics.Stopwatch]::Frequency}
   if($script:daemon.Process.WaitForExit(0) -and $script:daemonExitTicks -eq 0){$script:daemonExitTicks=[Diagnostics.Stopwatch]::GetTimestamp()}
   if($script:stopDeadline -gt 0 -and $now -ge $script:stopDeadline -and $script:daemonExitTicks -eq 0){$timedOut=$true;break}
   if($cliExitTicks -gt 0 -and $script:daemonExitTicks -gt 0){break}
  }elseif($cliExitTicks -gt 0){break}
  if(($now-$startTicks)/[Diagnostics.Stopwatch]::Frequency -ge 15){$timedOut=$true;break}
  Start-Sleep -Milliseconds 20
 }
 $record=[ordered]@{label=$label;arguments=$arguments;expected_exit_code=$expected;exit_code=$(if($cliExitTicks){$child.Process.ExitCode}else{$null});timed_out=$timedOut;before_create_ticks=$child.BeforeCreateTicks;after_create_ticks=$child.AfterCreateTicks;resume_ticks=$child.ResumeTicks;cli_exit_observed_ticks=$cliExitTicks;stdin='NUL';retained_process_handles=$true;stdout_sha256=(Hash (Join-Path $root ($label+'.stdout')));stderr_sha256=(Hash (Join-Path $root ($label+'.stderr')))}
 if($exchange){
  if(-not $timedOut){if(-not $exchange.Work.Wait(2000)){throw 'observer receipt exceeds 2 seconds'}}
  $record.observer=[ordered]@{request_json=$exchange.Request;response_json=$exchange.Response;client_pid=$exchange.ClientPid;daemon_pid=$exchange.ServerPid;request_read_ticks=$exchange.RequestReadTicks;forward_before_ticks=$exchange.ForwardBeforeTicks;forward_after_ticks=$exchange.ForwardAfterTicks;response_read_ticks=$exchange.ResponseReadTicks;client_write_ticks=$exchange.ClientWriteTicks;completed=$exchange.Work.IsCompletedSuccessfully}
 }
 Save ($label+'-result.json') $record
 if($timedOut){throw ('finite CLI/normal-exit bound failed: '+$label)}
 if($record.exit_code -ne $expected){throw ('unexpected CLI exit: '+$label)}
 if($exchange){
  $req=$exchange.Request|ConvertFrom-Json;$resp=$exchange.Response|ConvertFrom-Json
  if($exchange.ClientPid -ne $child.Process.Id -or $exchange.ServerPid -ne $script:daemon.Process.Id -or $req.method -ne $method -or $req.id -ne $resp.id){throw 'request/response/process identity mismatch'}
  $result.requests+=@([ordered]@{label=$label;request=$req;response=$resp;cli_pid=$child.Process.Id;daemon_pid=$exchange.ServerPid})
  $exchange.Dispose()
 }else{$resp=$null}
 return [ordered]@{record=$record;response=$resp;stdout=[IO.File]::ReadAllText((Join-Path $root ($label+'.stdout')));stderr=[IO.File]::ReadAllText((Join-Path $root ($label+'.stderr')))}
}
try{
 if((Pipe-Exists $daemonPipeName) -or (Pipe-Exists $relayPipeName)){throw 'fresh private pipe name already exists'}
 Port-State 'before'
 $v=Invoke-Cli 'runtime-version' @('version') 0 $null
 if($v.stdout.Trim() -ne 'openergy 0.1.0'){throw 'version mismatch'}
 $absent=Invoke-Cli 'runtime-absent' @('status','--socket',$daemonPipe) 1 $null
 if($absent.stderr -notmatch 'is the daemon running\?' -or $absent.stdout.Length -ne 0){throw 'absent daemon failure mismatch'}
 $script:daemon=New-Child 'runtime-daemon' @('daemon','--config',$config,'--platform','virtual','--socket',$daemonPipe,'--parent-pid',[string]$PID,'--foreground')
 $daemon.Resume();$ready=[Diagnostics.Stopwatch]::StartNew()
 $log=Join-Path $run 'config/openergy.log'
 while($true){
  if($daemon.Process.HasExited){throw 'daemon exited before readiness'}
  if((Test-Path -LiteralPath $log) -and ([IO.File]::ReadAllText($log)).Contains('daemon listening on '+$daemonPipe)){break}
  if($ready.Elapsed.TotalSeconds -ge 15){throw 'readiness exceeded 15 seconds'}
  Start-Sleep -Milliseconds 50
 }
 Save 'runtime-readiness.json' ([ordered]@{elapsed_ms=$ready.ElapsedMilliseconds;marker='daemon listening on private named pipe';controller_alive=(-not $controller.HasExited);daemon_identity=(Identity $daemon.Process);pipe_present=(Pipe-Exists $daemonPipeName)})
 $status=Invoke-Cli 'runtime-status' @('status','--socket',$relayPipe) 0 'status.get'
 $s=$status.response.result
 if($s.daemon.pid -ne $daemon.Process.Id -or $s.daemon.platform -ne 'virtual' -or $s.daemon.version -ne '0.1.0' -or $s.role -ne 'off' -or $s.state -ne 'idle' -or $null -ne $s.last_error -or $s.peers.Count -ne 0){throw 'healthy virtual Off status mismatch'}
 $cfg=Invoke-Cli 'runtime-config' @('ctl','config.get','--socket',$relayPipe,'--raw') 0 'config.get'
 $c=$cfg.response.result.config
 if($c.role -ne 'off' -or $c.server.listen -ne '127.0.0.1:59367' -or $c.server.port -ne 59367 -or $c.client.autoconnect -or $c.tls.enabled -or $c.clipboard.enabled){throw 'private Off config mismatch'}
 $bad=Invoke-Cli 'runtime-invalid' @('ctl','fixture.invalid.c8a3489c','--socket',$relayPipe,'--raw') 1 'fixture.invalid.c8a3489c'
 if($bad.response.error.code -ne 'unknown_method' -or $bad.response.error.message -ne 'unknown method fixture.invalid.c8a3489c'){throw 'invalid method error mismatch'}
 Port-State 'while-off'
 $tcp=@(Get-NetTCPConnection -OwningProcess $daemon.Process.Id -ErrorAction SilentlyContinue)
 $udp=@(Get-NetUDPEndpoint -OwningProcess $daemon.Process.Id -ErrorAction SilentlyContinue)
 Save 'runtime-owned-network.json' ([ordered]@{daemon_pid=$daemon.Process.Id;tcp=$tcp;udp=$udp;sampled_utc=[DateTime]::UtcNow.ToString('o');network_inventory_queries_only=$true})
 if($tcp.Count -or $udp.Count){throw 'Off daemon unexpectedly owns a network socket'}
 $stop=Invoke-Cli 'runtime-stop' @('ctl','daemon.stop','--socket',$relayPipe,'--raw') 0 'daemon.stop'
 if($stop.stdout.Trim() -ne '{"stopping":true}' -or $stop.stderr.Length -ne 0 -or -not $stop.response.result.stopping){throw 'exact daemon.stop acknowledgement mismatch'}
 $result.stops_sent=$script:stopCalls
 $result.daemon_exit=[ordered]@{exit_code=$daemon.Process.ExitCode;independent_retained_os_handle_signalled=$daemon.Process.WaitForExit(0);exit_first_observed_ticks=$script:daemonExitTicks;deadline_ticks=$script:stopDeadline;controller_alive=(-not $controller.HasExited);controller_start_unchanged=($controller.StartTime.ToUniversalTime().ToString('o') -eq $controllerIdentity.start_utc);daemon_pid_absent=($null -eq (Get-CimInstance Win32_Process -Filter ('ProcessId='+$daemon.Process.Id)));stop_to_exit_observation_ms=1000.0*($script:daemonExitTicks-$stop.record.observer.forward_before_ticks)/[Diagnostics.Stopwatch]::Frequency;response_receipt_to_exit_observation_ms=1000.0*($script:daemonExitTicks-$stop.record.observer.response_read_ticks)/[Diagnostics.Stopwatch]::Frequency}
 if($daemon.Process.ExitCode -ne 0 -or -not $result.daemon_exit.daemon_pid_absent -or -not $result.daemon_exit.controller_alive -or $script:stopCalls -ne 1){throw 'normal daemon OS exit mismatch'}
 Port-State 'after'
 $result.pipe_cleanup=[ordered]@{daemon_pipe_absent=(-not (Pipe-Exists $daemonPipeName));observer_pipe_absent=(-not (Pipe-Exists $relayPipeName))}
 if(-not $result.pipe_cleanup.daemon_pipe_absent -or -not $result.pipe_cleanup.observer_pipe_absent){throw 'private pipe remains'}
 $result.passed=$true
}catch{
 $result.failure=$_.Exception.Message;$result.error_record=($_|Out-String)
 [IO.File]::WriteAllText((Join-Path $root 'runtime-first-failure.stderr'),($_|Out-String))
}finally{
 $result.stops_sent=$script:stopCalls
 foreach($exchange in $script:exchanges){try{$exchange.Dispose()}catch{}}
 foreach($child in $script:children){
  if(-not $child.Process.HasExited){
   if($script:stopCalls -gt 0 -and $script:stopDeadline -gt 0){$remaining=[Math]::Max(0,[Math]::Ceiling(1000.0*($script:stopDeadline-[Diagnostics.Stopwatch]::GetTimestamp())/[Diagnostics.Stopwatch]::Frequency));if($remaining -gt 0){$null=$child.Process.WaitForExit([int]$remaining)}}
   if(-not $child.Process.HasExited){
    $cim=Get-CimInstance Win32_Process -Filter ('ProcessId='+$child.Process.Id)
    if(-not $cim -or $cim.ExecutablePath -ne $binary -or [Math]::Abs(($cim.CreationDate.ToUniversalTime()-$child.Process.StartTime.ToUniversalTime()).TotalMilliseconds) -ge 2 -or $cim.ParentProcessId -ne $PID){$result.recovery+=@('identity mismatch: no kill');continue}
    $result.recovery+=@([ordered]@{identity=(Identity $child.Process);kill_ticks=[Diagnostics.Stopwatch]::GetTimestamp();only_after_declared_deadline=$true})
    $child.Process.Kill();if(-not $child.Process.WaitForExit(10000)){$result.recovery+=@('owned recovery exceeded 10 seconds')}
   }
  }
  if($child.Process.HasExited){$child.Dispose()}
 }
 $result.product_handles_closed=@($script:children|Where-Object {-not $_.Closed}).Count -eq 0
 $result.controller_alive_at_finally=(-not $controller.HasExited)
 $result.remaining_owned_processes=Owned-Processes
 $result.artifact_sha256_after=Hash $binary
 $result.config_sha256_after=Hash $config
 $result.finished_utc=[DateTime]::UtcNow.ToString('o')
 $result.passed=($result.passed -and $result.product_handles_closed -and $result.remaining_owned_processes.Count -eq 0 -and $result.artifact_sha256_after -eq $build.artifact.sha256 -and $result.config_sha256_after -eq $result.config_sha256)
 Save 'runtime-result.json' $result
 $env:USERPROFILE=$originalProfile
}
if(-not $result.passed){exit 1}
Write-Output 'ONE_NATIVE_VIRTUAL_OFF_FLOW_PASS'
