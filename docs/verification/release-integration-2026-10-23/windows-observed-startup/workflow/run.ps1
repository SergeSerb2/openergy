. (Join-Path $PSScriptRoot 'audit.ps1')
if(Test-Path (Join-Path $root 'run-start.json')){throw 'fixed run sequence already attempted; no retry'}
$build=Get-Content -Raw (Join-Path $root 'build-result.json')|ConvertFrom-Json
$freeze=Get-Content -Raw (Join-Path $root 'freeze.json')|ConvertFrom-Json
if(-not $build.passed){throw 'candidate build did not pass'}
$exe=Join-Path $root 'bin/discovery_recovery.exe'
if(-not (Get-Item $exe).IsReadOnly -or (Hash $exe) -ne $freeze.binary.sha256){throw 'frozen executable mismatch'}
$null=Assert-Source
$parentEnv=@{OPENERGY_STARTUP_OBSERVE=$env:OPENERGY_STARTUP_OBSERVE;OPENERGY_STARTUP_CLEANUP_OBSERVE=$env:OPENERGY_STARTUP_CLEANUP_OBSERVE}
Save-Json ([ordered]@{started_utc=[DateTime]::UtcNow.ToString('o');binary_sha256=$freeze.binary.sha256;source_commit=$build.source_commit;parent_control_env=$parentEnv;tools=(Tools);startup_timeout_seconds=360;retained_timeout_seconds=120;sequence_fixed=$true}) (Join-Path $root 'run-start.json')
$specs=@(
 @{name='startup-1';mode='startup';mdns='25090';service='25094';timeout=360;control=$null},
 @{name='startup-2';mode='startup';mdns='25092';service='25096';timeout=360;control=$null},
 @{name='retained-800';mode='probe';mdns='25090';service='25094';timeout=120;control=$null},
 @{name='missing-startup-observer';mode='startup';mdns='25092';service='25096';timeout=360;control='OPENERGY_STARTUP_OBSERVE'},
 @{name='missing-cleanup-observer';mode='startup';mdns='25090';service='25094';timeout=360;control='OPENERGY_STARTUP_CLEANUP_OBSERVE'}
)
$results=@()
foreach($spec in $specs){
 $before=Inventory ($spec.name+'-before');Save-Json $before (Join-Path $root ($spec.name+'-before.json'))
 if($before.fixture_processes.Count -ne 0 -or $before.assigned_tcp_listeners.Count -ne 0 -or $before.assigned_udp_endpoints.Count -ne 0){throw 'runtime prerequisite: owned process or assigned endpoint occupied'}
 if((Hash $exe) -ne $freeze.binary.sha256){throw 'frozen binary changed before case'}
 $arguments=@($spec.mode,(Join-Path $root ('runtime/'+$spec.name)),$spec.mdns,$spec.service)
 $info=[Diagnostics.ProcessStartInfo]::new($exe);$info.UseShellExecute=$false;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true;$info.WorkingDirectory=Join-Path $root 'runtime'
 foreach($a in $arguments){$info.ArgumentList.Add($a)}
 foreach($key in @('OPENERGY_STARTUP_OBSERVE','OPENERGY_STARTUP_CLEANUP_OBSERVE')){$null=$info.Environment.Remove($key)}
 $overrides=@{};if($spec.control){$info.Environment[$spec.control]='0';$overrides[$spec.control]='0'}
 $started=[DateTime]::UtcNow.ToString('o');$watch=[Diagnostics.Stopwatch]::StartNew()
 $process=[Diagnostics.Process]::Start($info);$startTime=$process.StartTime;$stdout=$process.StandardOutput.ReadToEndAsync();$stderr=$process.StandardError.ReadToEndAsync()
 $record=[ordered]@{name=$spec.name;mode=$spec.mode;arguments=$arguments;started_utc=$started;binary_sha256=$freeze.binary.sha256;source_commit=$build.source_commit;environment_overrides=$overrides;negative_control=[bool]$spec.control;expected_exit=$(if($spec.control){1}else{0});pid=$process.Id;session=$process.SessionId;priority=$process.PriorityClass.ToString();timeout_seconds=$spec.timeout;timed_out=$false;forced_cleanup=@();observed_processes=@()}
 Save-Json $record (Join-Path $root ($spec.name+'-process.json'))
 $seen=@{}
 while(-not $process.WaitForExit(500)){
  $children=@(Get-CimInstance Win32_Process -ErrorAction Stop|Where-Object {$_.ExecutablePath -and $_.ExecutablePath -eq $exe})
  foreach($c in $children){$key=[string]$c.ProcessId+':'+$c.CreationDate.ToString('o');if(-not $seen.ContainsKey($key)){$p=Get-Process -Id $c.ProcessId -ErrorAction SilentlyContinue;if($p){$seen[$key]=[ordered]@{name=$c.Name;pid=$c.ProcessId;parent_pid=$c.ParentProcessId;path=$c.ExecutablePath;created=$c.CreationDate.ToString('o');session=$c.SessionId;priority=$p.PriorityClass.ToString();command_line=$c.CommandLine}}}}
  if($watch.Elapsed.TotalSeconds -ge $spec.timeout){
   $record.timed_out=$true
   foreach($c in $children){$owned=Get-Process -Id $c.ProcessId -ErrorAction SilentlyContinue;if($owned -and $owned.Path -eq $exe -and $owned.StartTime.ToUniversalTime() -eq $c.CreationDate.ToUniversalTime()){$owned.Kill();$null=$owned.WaitForExit(5000);$record.forced_cleanup+=@([ordered]@{pid=$owned.Id;path=$owned.Path})}}
   if(-not $process.HasExited){$owned=Get-Process -Id $process.Id -ErrorAction Stop;if($owned.Path -ne $exe -or $owned.StartTime -ne $startTime){throw 'refuse timeout cleanup of non-owned process'};$owned.Kill();$record.forced_cleanup+=@([ordered]@{pid=$owned.Id;path=$owned.Path})}
   break
  }
 }
 $process.WaitForExit();$watch.Stop();$record.exit=$process.ExitCode;$record.ended_utc=[DateTime]::UtcNow.ToString('o');$record.elapsed_ms=$watch.ElapsedMilliseconds;$record.observed_processes=@($seen.Values)
 [IO.File]::WriteAllText((Join-Path $root ('logs/'+$spec.name+'.stdout')),$stdout.Result,[Text.UTF8Encoding]::new($false))
 [IO.File]::WriteAllText((Join-Path $root ('logs/'+$spec.name+'.stderr')),$stderr.Result,[Text.UTF8Encoding]::new($false))
 $record.stdout_sha256=Hash (Join-Path $root ('logs/'+$spec.name+'.stdout'));$record.stderr_sha256=Hash (Join-Path $root ('logs/'+$spec.name+'.stderr'))
 $record.checks=@($stdout.Result -split '\r?\n'|Where-Object {$_ -match '^CHECK '});$record.check_count=$record.checks.Count;$record.passed_checks=@($record.checks|Where-Object {$_.EndsWith('=true')}).Count;$record.failed_checks=@($record.checks|Where-Object {$_.EndsWith('=false')})
 foreach($key in @('OPENERGY_STARTUP_OBSERVE','OPENERGY_STARTUP_CLEANUP_OBSERVE')){$null=$info.Environment.Remove($key)}
 $record.child_override_map_cleared=$true;$record.parent_control_env_unchanged=$env:OPENERGY_STARTUP_OBSERVE -eq $parentEnv.OPENERGY_STARTUP_OBSERVE -and $env:OPENERGY_STARTUP_CLEANUP_OBSERVE -eq $parentEnv.OPENERGY_STARTUP_CLEANUP_OBSERVE
 Save-Json $record (Join-Path $root ($spec.name+'-result.json'));$results+=@($record)
 $after=Inventory ($spec.name+'-after');Save-Json $after (Join-Path $root ($spec.name+'-after.json'))
 if($after.fixture_processes.Count -ne 0 -or $after.assigned_tcp_listeners.Count -ne 0 -or $after.assigned_udp_endpoints.Count -ne 0){throw 'runtime cleanup prerequisite failed; stop dependent cases'}
}
Save-Json $results (Join-Path $root 'run-result.json')
$wrong=@($results|Where-Object {$_.exit -ne $_.expected_exit -or $_.timed_out})
if($wrong.Count -gt 0){Write-Output 'FIXED_SEQUENCE_HAS_ACCEPTANCE_OR_CONTROL_FAILURE';exit 1}
Write-Output 'FIXED_SEQUENCE_COMPLETE_EXPECTED_EXITS'
