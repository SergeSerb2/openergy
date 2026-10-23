. (Join-Path $PSScriptRoot 'common.ps1')
if(Test-Path (Join-Path $root 'diagnostic1-process.json')){throw 'first diagnostic already attempted'}
$compile=Get-Content -Raw (Join-Path $root 'compile-result.json')|ConvertFrom-Json
if($compile.exit -ne 0){throw 'diagnostic compile failed'}
$exe=Join-Path $root 'bin/startup_diagnostic.exe';$source=Join-Path $root 'startup_diagnostic.rs'
if(-not (Get-Item $exe).IsReadOnly -or (Hash $exe) -ne $compile.binary.sha256 -or (Hash $source) -ne $compile.source_sha256){throw 'diagnostic inputs changed'}
$acl=Assert-ACL;$before=Inventory 'diagnostic1-before'
if($before.fixture_processes.Count -ne 0 -or $before.assigned_tcp_listeners.Count -ne 0 -or $before.assigned_udp_endpoints.Count -ne 0){throw 'owned process or assigned port occupied'}
Save-Json ([ordered]@{acl=$acl;inventory=$before;tools=(Tools)}) (Join-Path $root 'diagnostic1-before.json')
$arguments=@('startup',(Join-Path $root 'runtime/run1'),'25090','25094')
$info=[Diagnostics.ProcessStartInfo]::new($exe);$info.UseShellExecute=$false;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true;$info.WorkingDirectory=Join-Path $root 'runtime'
foreach($a in $arguments){$info.ArgumentList.Add($a)}
$watch=[Diagnostics.Stopwatch]::StartNew();$started=[DateTime]::UtcNow.ToString('o');$process=[Diagnostics.Process]::Start($info);$identity=$process.StartTime
$stdout=$process.StandardOutput.ReadToEndAsync();$stderr=$process.StandardError.ReadToEndAsync()
$result=[ordered]@{mode='diagnostic1';started_utc=$started;executable=$exe;binary_sha256=$compile.binary.sha256;source_sha256=$compile.source_sha256;arguments=$arguments;session=$process.SessionId;priority=$process.PriorityClass.ToString();pid=$process.Id;timeout_seconds=120;timed_out=$false;forced_cleanup=@();observed_processes=@()}
Save-Json $result (Join-Path $root 'diagnostic1-process.json')
$seen=@{}
while(-not $process.WaitForExit(500)){
 $children=@(Get-CimInstance Win32_Process -ErrorAction Stop|Where-Object {$_.ExecutablePath -and $_.ExecutablePath -eq $exe})
 foreach($c in $children){$key=[string]$c.ProcessId+':'+$c.CreationDate.ToString('o');if(-not $seen.ContainsKey($key)){$p=Get-Process -Id $c.ProcessId -ErrorAction SilentlyContinue;if($p){$seen[$key]=[ordered]@{name=$c.Name;pid=$c.ProcessId;parent_pid=$c.ParentProcessId;created=$c.CreationDate.ToString('o');session=$c.SessionId;priority=$p.PriorityClass.ToString();path=$c.ExecutablePath}}}}
 if($watch.Elapsed.TotalSeconds -ge 120){
  $result.timed_out=$true
  foreach($c in $children){$live=Get-Process -Id $c.ProcessId -ErrorAction SilentlyContinue;if($live -and $live.Path -eq $exe -and $live.StartTime.ToUniversalTime() -eq $c.CreationDate.ToUniversalTime()){$live.Kill();$null=$live.WaitForExit(5000);$result.forced_cleanup+=@($c.ProcessId)}}
  if(-not $process.HasExited){$live=Get-Process -Id $process.Id -ErrorAction Stop;if($live.Path -ne $exe -or $live.StartTime -ne $identity){throw 'refuse cleanup of non-owned process'};$live.Kill();$result.forced_cleanup+=@($live.Id)}
  break
 }
}
$process.WaitForExit();$watch.Stop();$result.exit=$process.ExitCode;$result.elapsed_ms=$watch.ElapsedMilliseconds;$result.ended_utc=[DateTime]::UtcNow.ToString('o');$result.observed_processes=@($seen.Values)
[IO.File]::WriteAllText((Join-Path $root 'logs/diagnostic1.stdout'),$stdout.Result,[Text.UTF8Encoding]::new($false));[IO.File]::WriteAllText((Join-Path $root 'logs/diagnostic1.stderr'),$stderr.Result,[Text.UTF8Encoding]::new($false))
$result.stdout_sha256=Hash (Join-Path $root 'logs/diagnostic1.stdout');$result.stderr_sha256=Hash (Join-Path $root 'logs/diagnostic1.stderr');$result.checks=@($stdout.Result -split '\r?\n'|Where-Object {$_ -match '^CHECK '})
Save-Json $result (Join-Path $root 'diagnostic1-result.json')
$after=Inventory 'diagnostic1-after';Save-Json $after (Join-Path $root 'diagnostic1-after.json')
if($after.fixture_processes.Count -ne 0 -or $after.assigned_tcp_listeners.Count -ne 0 -or $after.assigned_udp_endpoints.Count -ne 0){throw 'owned process or assigned endpoint remains'}
Write-Output ('DIAGNOSTIC1_EXIT='+$result.exit)
exit $result.exit
