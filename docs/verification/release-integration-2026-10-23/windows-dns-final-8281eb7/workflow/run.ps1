. (Join-Path $PSScriptRoot 'runtime-common.ps1')
if(Test-Path (Join-Path $root 'run-start.json')){throw 'fixed two-run sequence already attempted; no retry'}
$build=Get-Content -Raw (Join-Path $root 'build-result.json')|ConvertFrom-Json
$freeze=Get-Content -Raw (Join-Path $root 'freeze.json')|ConvertFrom-Json
if(-not $build.passed){throw 'build prerequisite failed'}
$exe=Join-Path $root 'bin/dns_queue_cancel_c4806a3d.exe'
if(-not (Get-Item $exe).IsReadOnly -or (Hash $exe) -ne $freeze.binary.sha256){throw 'frozen executable mismatch'}
$null=Assert-Source
Save-Json ([ordered]@{started_utc=[DateTime]::UtcNow.ToString('o');binary_sha256=$freeze.binary.sha256;source_commit=$build.source_commit;tools=(Tools);timeout_seconds=300;sequence=@('normal-full','owned-conflict-full');process_environment_only=$true;env=@{TEMP=$env:TEMP;TMP=$env:TMP;CARGO_BUILD_JOBS=$env:CARGO_BUILD_JOBS;RUSTUP_AUTO_INSTALL=$env:RUSTUP_AUTO_INSTALL;OPENERGY_ALLOW_PORT_24800=$env:OPENERGY_ALLOW_PORT_24800;OPENERGY_DISABLE_DISCOVERY=$env:OPENERGY_DISABLE_DISCOVERY}}) (Join-Path $root 'run-start.json')
$results=@()
foreach($name in @('normal-full','owned-conflict-full')){
 $before=Inventory ($name+'-before');Save-Json $before (Join-Path $root ($name+'-before.json'))
 if($before.fixture_processes.Count -ne 0 -or $before.assigned_tcp_non_timewait.Count -ne 0 -or $before.assigned_udp_endpoints.Count -ne 0){throw 'owned process or assigned endpoint occupied before case'}
 if((Hash $exe) -ne $freeze.binary.sha256){throw 'frozen executable changed'}
 $conflict=$null
 try{
  if($name -eq 'owned-conflict-full'){
   $conflict=New-ConflictSocket
   $duplicate=$null;$duplicateSucceeded=$false;$duplicateError=$null
   try{$duplicate=New-ConflictSocket;$duplicateSucceeded=$true}catch{$exception=$_.Exception;while($exception.InnerException){$exception=$exception.InnerException};$duplicateError=[ordered]@{type=$exception.GetType().FullName;message=$exception.Message;native_error=$exception.NativeErrorCode;socket_error=$exception.SocketErrorCode.ToString()}}finally{if($duplicate){$duplicate.Dispose()}}
   $record=[ordered]@{created_utc=[DateTime]::UtcNow.ToString('o');endpoint=$conflict.LocalEndPoint.ToString();exclusive=$conflict.ExclusiveAddressUse;ipv6_only=[int]$conflict.GetSocketOption([Net.Sockets.SocketOptionLevel]::IPv6,[Net.Sockets.SocketOptionName]::IPv6Only);listening=$false;listen_called=$false;duplicate_succeeded=$duplicateSucceeded;duplicate_error=$duplicateError;held_until_verifier_return=$true;owner_pid=$PID;session=(Get-Process -Id $PID).SessionId;priority=(Get-Process -Id $PID).PriorityClass.ToString()}
   Save-Json $record (Join-Path $root 'owned-conflict.json')
   if($duplicateSucceeded -or -not $record.exclusive -or $record.ipv6_only -ne 1 -or $duplicateError.native_error -ne 10048){throw 'genuine owned AddrInUse conflict prerequisite unavailable'}
  }
  $arguments=@((Join-Path $root ('runtime/'+$name)),'--observe-attempts')
  $info=[Diagnostics.ProcessStartInfo]::new($exe);$info.UseShellExecute=$false;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true;$info.WorkingDirectory=Join-Path $root 'runtime';foreach($a in $arguments){$info.ArgumentList.Add($a)}
  $started=[DateTime]::UtcNow.ToString('o');$watch=[Diagnostics.Stopwatch]::StartNew();$process=[Diagnostics.Process]::Start($info);$startTime=$process.StartTime;$stdout=$process.StandardOutput.ReadToEndAsync();$stderr=$process.StandardError.ReadToEndAsync()
  $record=[ordered]@{name=$name;command=@($exe)+$arguments;cwd=$info.WorkingDirectory;started_utc=$started;binary_sha256=$freeze.binary.sha256;source_commit=$build.source_commit;pid=$process.Id;session=$process.SessionId;priority=$process.PriorityClass.ToString();timeout_seconds=300;timed_out=$false;forced_cleanup=$false;samples=@()}
  Save-Json $record (Join-Path $root ($name+'-process.json'))
  while(-not $process.WaitForExit(500)){
   $record.samples+=@([ordered]@{at_ms=$watch.ElapsedMilliseconds;session=$process.SessionId;priority=$process.PriorityClass.ToString()})
   if($watch.Elapsed.TotalSeconds -ge 300){$record.timed_out=$true;$owned=Get-Process -Id $process.Id -ErrorAction Stop;if($owned.Path -ne $exe -or $owned.StartTime -ne $startTime){throw 'refuse cleanup of non-owned process'};$owned.Kill();$record.forced_cleanup=$true;break}
  }
  if(-not $process.WaitForExit(5000)){throw 'owned verifier failed to exit after watchdog'};$watch.Stop()
  $record.exit=$process.ExitCode;$record.ended_utc=[DateTime]::UtcNow.ToString('o');$record.elapsed_ms=$watch.ElapsedMilliseconds
  [IO.File]::WriteAllText((Join-Path $root ('logs/'+$name+'.stdout')),$stdout.Result,[Text.UTF8Encoding]::new($false));[IO.File]::WriteAllText((Join-Path $root ('logs/'+$name+'.stderr')),$stderr.Result,[Text.UTF8Encoding]::new($false))
  $record.stdout_sha256=Hash (Join-Path $root ('logs/'+$name+'.stdout'));$record.stderr_sha256=Hash (Join-Path $root ('logs/'+$name+'.stderr'))
  $record.result_lines=@($stdout.Result -split '\r?\n'|Where-Object {$_ -match '^(RESULT|QUEUE|CONTRACTS|RECOVERY|REFUSAL|DEADLINE|FIXTURE |FIXTURE_SETUP|EXCLUSIVE_BIND|TLS_HANDOFF|OS_CONTROL)'})
  Save-Json $record (Join-Path $root ($name+'-result.json'));$results+=@($record);Save-Json $results (Join-Path $root 'run-result.json')
 }finally{if($conflict){$conflict.Dispose();Save-Json ([ordered]@{disposed_utc=[DateTime]::UtcNow.ToString('o');normal_finally=$true;owner_pid=$PID}) (Join-Path $root 'owned-conflict-disposed.json')}}
 $identities=@(Get-ChildItem (Join-Path $root ('runtime/'+$name)) -File -Recurse -Force|Where-Object {$_.Name -in @('cert.pem','key.pem','.identity-lock')})
 Save-Json @($identities|ForEach-Object {File-Record $_}) (Join-Path $root ($name+'-private-identities-removed.json'))
 foreach($file in $identities){Remove-Item -LiteralPath $file.FullName -Force}
 if(@(Get-ChildItem (Join-Path $root ('runtime/'+$name)) -File -Recurse -Force).Count -ne 0){throw 'unexpected runtime files remain after private TLS identity removal'}
 $after=Inventory ($name+'-after');Save-Json $after (Join-Path $root ($name+'-after.json'))
 if($after.fixture_processes.Count -ne 0 -or $after.assigned_tcp_non_timewait.Count -ne 0 -or $after.assigned_udp_endpoints.Count -ne 0){throw 'cleanup prerequisite failed; stop dependent case'}
}
if(@($results|Where-Object {$_.exit -ne 0 -or $_.timed_out}).Count -gt 0){Write-Output 'FIXED_DNS_SEQUENCE_COMPLETED_WITH_NONZERO_VERDICT';exit 1}
Write-Output 'FIXED_DNS_SEQUENCE_COMPLETE_PASS'
