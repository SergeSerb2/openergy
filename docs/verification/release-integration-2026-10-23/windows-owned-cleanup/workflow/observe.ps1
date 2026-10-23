. (Join-Path $PSScriptRoot 'common.ps1')
if(Test-Path (Join-Path $root 'observation-start.json')){throw 'normal cases already attempted; inspect existing results'}
$bins=Get-Content -Raw (Join-Path $root 'frozen-binaries.json')|ConvertFrom-Json
foreach($e in $bins){$f=Get-Item (Join-Path $root $e.path);if(-not $f.IsReadOnly -or $f.Length -ne $e.bytes -or (Hash $f.FullName) -ne $e.sha256){throw 'frozen helper/observer changed'}}
Add-Type -Path (Join-Path $root 'bin/OwnedNativeObserver.dll')
# Successful nonempty native tables are required before starting any lifecycle case.
$preNative=[OwnedNativeObserver]::ReadUdp();$pre=Inventory 'pre-observation'
if($pre.fixture_processes.Count -ne 0 -or $pre.assigned_tcp_listeners.Count -ne 0 -or $pre.assigned_udp_endpoints.Count -ne 0){throw 'owned process or assigned endpoint occupied'}
Save-Json ([ordered]@{recorded_utc=[DateTime]::UtcNow.ToString('o');global_udp_count=$preNative.Rows.Count;native_queries=$preNative.Queries;inventory=$pre;tools=(Tools);acl=(Assert-ACL);cleanup_deadline_ms=10000;poll_interval_ms=50;init_deadline_seconds=15;normal_cases_only=$true}) (Join-Path $root 'observation-start.json')
function Transcript($state,$direction,$line){$state.lines.Add([ordered]@{at_utc=[DateTime]::UtcNow.ToString('o');elapsed_ms=$state.clock.Elapsed.TotalMilliseconds;direction=$direction;line=$line})}
function Send-Control($state,$line){Transcript $state 'observer-to-helper' $line;$state.process.StandardInput.WriteLine($line);$state.process.StandardInput.Flush()}
function Read-Until($state,$prefix,$timeoutMs){
 $watch=[Diagnostics.Stopwatch]::StartNew()
 while($watch.ElapsedMilliseconds -lt $timeoutMs){
  if($null -eq $state.pending){$state.pending=$state.process.StandardOutput.ReadLineAsync()}
  if($state.pending.Wait(20)){
   $line=$state.pending.Result;$state.pending=$null
   if($null -eq $line){throw ('helper stdout closed while waiting for '+$prefix)}
   Transcript $state 'helper-to-observer' $line
   if($line -match '^WORKER_LOG native_tid=(\d+) source_line=4058$'){$state.worker=[uint32]$Matches[1]}
   if($line -like 'WORKER_CONFLICT*'){throw 'multiple native worker identities observed'}
   if($line.StartsWith($prefix)){return $line}
  }
  if($state.process.HasExited){throw ('helper exited while waiting for '+$prefix)}
 }
 throw ('helper protocol deadline for '+$prefix)
}
function OS-Snapshot($state,$phase,[bool]$details=$false){
 $p=$state.process;$p.Refresh()
 if($p.HasExited -or $p.StartTime -ne $state.startTime -or $p.Path -ne $state.exe){throw 'helper not alive with its exact original process identity'}
 $threads=@($p.Threads)
 if($threads.Count -eq 0){throw 'native OS process thread inventory is empty'}
 $rows=@(foreach($t in $threads){
  $r=[ordered]@{tid=$t.Id;state=$t.ThreadState.ToString()}
  if($details){
   try {$r.start_address=$t.StartAddress.ToInt64().ToString('x')} catch {$r.start_address_error=$_.Exception.Message}
   try {$r.description=[OwnedNativeObserver]::DescribeThread([uint32]$t.Id)} catch {$r.description_error=$_.Exception.Message}
  }
  $r
 })
 $native=[OwnedNativeObserver]::ReadUdp()
 $owned=@($native.Rows|Where-Object {$_.ProcessId -eq $p.Id})
 $p.Refresh();if($p.HasExited -or $p.StartTime -ne $state.startTime){throw 'helper exited or changed identity during native snapshot'}
 $r=[ordered]@{phase=$phase;recorded_utc=[DateTime]::UtcNow.ToString('o');elapsed_ms=$state.clock.Elapsed.TotalMilliseconds;cleanup_elapsed_ms=$(if($state.cleanupClock){$state.cleanupClock.Elapsed.TotalMilliseconds}else{$null});helper_alive_before_and_after=$true;pid=$p.Id;start_time=$state.startTime.ToUniversalTime().ToString('o');thread_inventory_nonempty=$true;thread_count=$rows.Count;threads=$rows;worker_native_tid=$state.worker;worker_present=(@($rows|Where-Object {$_.tid -eq $state.worker}).Count -ne 0);global_udp_count=$native.Rows.Count;native_udp_queries=$native.Queries;owned_udp=$owned;owned_udp_count=$owned.Count}
 $state.samples.Add($r);return $r
}
$exe=Join-Path $root 'bin/owned_cleanup.exe';$results=@()
foreach($spec in @(@{mode='shutdown';mdns=25090;service=25094},@{mode='drop';mdns=25092;service=25096})){
 $state=@{exe=$exe;clock=[Diagnostics.Stopwatch]::StartNew();worker=[uint32]0;pending=$null;lines=[Collections.Generic.List[object]]::new();samples=[Collections.Generic.List[object]]::new();cleanupClock=$null;process=$null;startTime=$null}
 $record=[ordered]@{mode=$spec.mode;arguments=@($spec.mode,[string]$spec.mdns,[string]$spec.service);started_utc=[DateTime]::UtcNow.ToString('o');passed=$false;forced_cleanup=$false;cleanup_deadline_ms=10000;poll_interval_ms=50}
 try {
  $info=[Diagnostics.ProcessStartInfo]::new($exe);$info.UseShellExecute=$false;$info.RedirectStandardInput=$true;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true;$info.WorkingDirectory=Join-Path $root 'runtime'
  foreach($a in $record.arguments){$info.ArgumentList.Add($a)}
  $state.process=[Diagnostics.Process]::Start($info);$state.startTime=$state.process.StartTime;$state.errTask=$state.process.StandardError.ReadToEndAsync()
  $record.pid=$state.process.Id;$record.session=$state.process.SessionId;$record.priority=$state.process.PriorityClass.ToString()
  Save-Json $record (Join-Path $root ($spec.mode+'-process.json'))
  $ready=Read-Until $state 'READY ' 5000
  if(-not ($ready -match '^READY pid=(\d+) main_tid=(\d+) mode=(shutdown|drop)$')){throw 'invalid helper READY record'}
  if([int]$Matches[1] -ne $state.process.Id -or $Matches[3] -ne $spec.mode){throw 'helper READY identity does not match OS process'}
  $record.main_tid=[uint32]$Matches[2]
  $baseline=OS-Snapshot $state 'baseline-before-any-Discovery-init' $true
  if($baseline.owned_udp_count -ne 0){throw 'uninitialized helper already owns UDP endpoints; baseline must not be warmed'}
  $record.baseline=$baseline
  Send-Control $state 'init'
  $liveLine=Read-Until $state 'LIVE ' 15000
  if(-not ($liveLine -match '^LIVE worker_tid=(\d+) resolved=true init_elapsed_us=(\d+)$')){throw 'invalid LIVE record'}
  if([uint32]$Matches[1] -eq 0 -or [uint32]$Matches[1] -ne $state.worker -or $state.worker -eq $record.main_tid){throw 'LIVE worker identity does not match genuine callback and main thread'}
  $record.publication_resolved=$true;$record.init_publication_elapsed_us=[UInt64]$Matches[2]
  $live=OS-Snapshot $state 'live-before-cleanup' $true
  if(-not $live.worker_present -or $live.owned_udp_count -lt 2 -or @($live.owned_udp|Where-Object {$_.Port -eq $spec.mdns}).Count -eq 0){throw 'real worker and expected custom-port UDP sockets not established'}
  if(@($live.owned_udp|Where-Object {$_.Port -in @(5353,24800)}).Count -ne 0){throw 'forbidden production endpoint in owned helper'}
  $record.worker_native_tid=$state.worker;$record.live=$live
  $state.cleanupClock=[Diagnostics.Stopwatch]::StartNew();Send-Control $state 'cleanup'
  $cleanupLine=Read-Until $state 'CLEANUP_RETURNED ' 5000
  $expectedRetained=if($spec.mode -eq 'shutdown'){'true'}else{'false'}
  if($cleanupLine -ne ('CLEANUP_RETURNED mode='+$spec.mode+' object_retained='+$expectedRetained)){throw 'cleanup control path did not retain/drop the intended Discovery object'}
  $record.cleanup_submission_returned_ms=$state.cleanupClock.Elapsed.TotalMilliseconds
  # Submission/return is recorded only as a boundary. It is never the cleanup verdict.
  $gone=$false
  while($state.cleanupClock.ElapsedMilliseconds -lt 10000){
   $sample=OS-Snapshot $state 'poll-cleanup'
   Save-Json @($state.samples.ToArray()) (Join-Path $root ($spec.mode+'-samples-progress.json'))
   if(-not $sample.worker_present -and $sample.owned_udp_count -eq 0){$gone=$true;$record.resources_absent_observed_ms=$sample.cleanup_elapsed_ms;break}
   Start-Sleep -Milliseconds 50
  }
  if(-not $gone){throw 'actual owned worker or UDP endpoints still present at the fixed 10-second deadline'}
  Send-Control $state 'ping';$alive=Read-Until $state 'ALIVE ' 5000
  if($alive -ne ('ALIVE pid='+$state.process.Id+' cleanup_done=true object_retained='+$expectedRetained)){throw 'helper did not remain responsive with the intended retained-object state'}
  $confirmed=OS-Snapshot $state 'confirmed-after-ping-before-exit' $true
  if($confirmed.worker_present -or $confirmed.owned_udp_count -ne 0){throw 'resource cleanup regressed while helper remained alive'}
  $record.confirmed=$confirmed;$record.helper_ping_after_resource_absence=$true
  Send-Control $state 'exit';$null=Read-Until $state 'EXITING' 5000
  if(-not $state.process.WaitForExit(5000)){throw 'normal helper exit deadline'}
  $record.exit=$state.process.ExitCode;if($record.exit -ne 0){throw 'helper did not exit zero'}
  $record.passed=$true
 } catch {
  $record.error=$_.Exception.Message;$record.error_details=$_.ToString();$record.failed_utc=[DateTime]::UtcNow.ToString('o')
  if($state.process -and -not $state.process.HasExited){$owned=Get-Process -Id $state.process.Id -ErrorAction SilentlyContinue;if($owned -and $owned.Path -eq $exe -and $owned.StartTime -eq $state.startTime){$owned.Kill();$null=$owned.WaitForExit(5000);$record.forced_cleanup=$true}}
 } finally {
  if($state.process -and $state.process.HasExited){$record.exit=$state.process.ExitCode;$record.stderr=$state.errTask.Result}
  $record.ended_utc=[DateTime]::UtcNow.ToString('o');$record.total_elapsed_ms=$state.clock.Elapsed.TotalMilliseconds;$record.transcript=@($state.lines.ToArray());$record.samples=@($state.samples.ToArray())
  if($state.worker -ne 0){$record.worker_native_tid=$state.worker}
  Save-Json $record (Join-Path $root ($spec.mode+'-result.json'))
  if($state.process -and $state.process.HasExited){$state.process.Dispose()}
 }
 $results+=@($record)
 if(-not $record.passed){break} # No repeat, no second case after an observation/access failure.
}
Save-Json $results (Join-Path $root 'observation-result.json')
if($results.Count -ne 2 -or @($results|Where-Object {-not $_.passed}).Count -ne 0){Write-Output 'OWNED_OBSERVATION_FAILED_OR_BLOCKED';exit 1}
Write-Output 'BOTH_NORMAL_OWNED_OBSERVATIONS_PASS'
