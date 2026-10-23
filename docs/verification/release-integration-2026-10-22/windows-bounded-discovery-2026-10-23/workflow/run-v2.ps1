. (Join-Path $PSScriptRoot 'common.ps1')
if(Test-Path -LiteralPath (Join-Path $root 'run-start.json')){throw 'runtime already attempted; inspect durable state'}
$build=Get-Content -Raw -LiteralPath (Join-Path $root 'build-result.json')|ConvertFrom-Json
$freeze=Get-Content -Raw -LiteralPath (Join-Path $root 'freeze.json')|ConvertFrom-Json
if(-not $build.passed){throw 'build did not pass'}
foreach($e in $freeze.frozen_bin){$f=Get-Item -LiteralPath (Join-Path $root $e.path);if(-not $f.IsReadOnly -or (Hash $f.FullName) -ne $e.sha256){throw 'frozen input mismatch'}}
$null=Assert-Source
Save-Json ([ordered]@{started_utc=[DateTime]::UtcNow.ToString('o');session=(Get-Process -Id $PID).SessionId;priority=(Get-Process -Id $PID).PriorityClass.ToString();temp=$env:TEMP;tmp=$env:TMP;discovery_disabled_for_virtual_control=$env:OPENERGY_DISABLE_DISCOVERY;production_port_allowed=$env:OPENERGY_ALLOW_PORT_24800}) (Join-Path $root 'run-start.json')
$bin=Join-Path $root 'bin/discovery_recovery.exe';$daemon=Join-Path $root 'bin/openergy.exe'
$specs=@(@{name='startup';exe=$bin;arguments=@('startup',(Join-Path $root 'runtime/startup'),'25090','25094');timeout=120},@{name='retained';exe=$bin;arguments=@('probe',(Join-Path $root 'runtime/retained'),'25092','25096');timeout=120},@{name='control';exe=(Get-Command pwsh.exe).Source;arguments=@('-NoLogo','-NoProfile','-NonInteractive','-File',(Join-Path $root 'source/tools/windows/verify-control.ps1'),'-BinaryPath',$daemon,'-Port','25241');timeout=120})
$results=@()
foreach($spec in $specs){
 $before=Inventory ($spec.name+'-before');Save-Json $before (Join-Path $root ($spec.name+'-before.json'))
 if($before.fixture_processes.Count -ne 0 -or $before.assigned_tcp_listeners.Count -ne 0 -or $before.assigned_udp_endpoints.Count -ne 0){throw 'owned process or assigned endpoint remains before run'}
 $start=[DateTime]::UtcNow.ToString('o');$watch=[Diagnostics.Stopwatch]::StartNew()
 $info=[Diagnostics.ProcessStartInfo]::new($spec.exe);$info.UseShellExecute=$false;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true;$info.WorkingDirectory=Join-Path $root 'runtime'
 foreach($a in $spec.arguments){$info.ArgumentList.Add($a)}
 $process=[Diagnostics.Process]::Start($info);$identity=$process.StartTime;$stdout=$process.StandardOutput.ReadToEndAsync();$stderr=$process.StandardError.ReadToEndAsync()
 $record=[ordered]@{name=$spec.name;executable=$spec.exe;executable_sha256=(Hash $spec.exe);arguments=$spec.arguments;started_utc=$start;pid=$process.Id;session=$process.SessionId;priority=$process.PriorityClass.ToString();timeout_seconds=$spec.timeout;timed_out=$false;forced_cleanup=@();observed_processes=@()}
 Save-Json $record (Join-Path $root ($spec.name+'-process.json'))
 $observed=@{}
 while(-not $process.WaitForExit(500)){
  $children=@(Get-CimInstance Win32_Process -ErrorAction Stop|Where-Object {$_.ExecutablePath -and $_.ExecutablePath.StartsWith((Join-Path $root 'bin')+'\',[StringComparison]::OrdinalIgnoreCase)})
  foreach($c in $children){$key=[string]$c.ProcessId+':'+$c.CreationDate.ToString('o');if(-not $observed.ContainsKey($key)){$native=Get-Process -Id $c.ProcessId -ErrorAction SilentlyContinue;if($native){$observed[$key]=[ordered]@{name=$c.Name;pid=$c.ProcessId;parent_pid=$c.ParentProcessId;path=$c.ExecutablePath;created=$c.CreationDate.ToString('o');session=$c.SessionId;priority=$native.PriorityClass.ToString();command_line=$c.CommandLine}}}}
  if($watch.Elapsed.TotalSeconds -ge $spec.timeout){
   $record.timed_out=$true
   foreach($c in $children){$live=Get-Process -Id $c.ProcessId -ErrorAction SilentlyContinue;if($live -and $live.Path -eq $c.ExecutablePath -and $live.StartTime.ToUniversalTime() -eq $c.CreationDate.ToUniversalTime()){$live.Kill();$null=$live.WaitForExit(5000);$record.forced_cleanup+=@([ordered]@{pid=$c.ProcessId;path=$c.ExecutablePath})}}
   if(-not $process.HasExited){$live=Get-Process -Id $process.Id -ErrorAction Stop;if($live.Path -ne $spec.exe -or $live.StartTime -ne $identity){throw 'refuse cleanup of non-owned wrapper'};$live.Kill();$record.forced_cleanup+=@([ordered]@{pid=$live.Id;path=$live.Path})}
   break
  }
 }
 $process.WaitForExit();$watch.Stop()
 [IO.File]::WriteAllText((Join-Path $root ('logs/'+$spec.name+'.stdout')),$stdout.Result,[Text.UTF8Encoding]::new($false))
 [IO.File]::WriteAllText((Join-Path $root ('logs/'+$spec.name+'.stderr')),$stderr.Result,[Text.UTF8Encoding]::new($false))
 $record.exit=$process.ExitCode;$record.elapsed_ms=$watch.ElapsedMilliseconds;$record.ended_utc=[DateTime]::UtcNow.ToString('o');$record.observed_processes=@($observed.Values)
 $record.stdout_sha256=Hash (Join-Path $root ('logs/'+$spec.name+'.stdout'));$record.stderr_sha256=Hash (Join-Path $root ('logs/'+$spec.name+'.stderr'))
 $record.checks=@($stdout.Result -split '\r?\n'|Where-Object {$_ -match '^(CHECK |PASS )'});$record.check_count=$record.checks.Count
 Save-Json $record (Join-Path $root ($spec.name+'-result.json'));$results+=@($record)
 $after=Inventory ($spec.name+'-after');Save-Json $after (Join-Path $root ($spec.name+'-after.json'))
 if($after.fixture_processes.Count -ne 0 -or $after.assigned_tcp_listeners.Count -ne 0 -or $after.assigned_udp_endpoints.Count -ne 0){throw 'owned process or assigned endpoint remains after run'}
}
Save-Json $results (Join-Path $root 'run-result.json')
if(@($results|Where-Object {$_.exit -ne 0 -or $_.timed_out}).Count -gt 0){Write-Output 'RUNTIME_FAILURE_PRESERVED';exit 1}
Write-Output 'BOUNDED_RUNTIME_PASS'
