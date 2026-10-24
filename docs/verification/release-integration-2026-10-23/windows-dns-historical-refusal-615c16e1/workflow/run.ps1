. (Join-Path $PSScriptRoot 'runtime-common.ps1')
if(Test-Path (Join-Path $root 'run-start.json')){throw 'one historical command already attempted; no retry'}
$build=Get-Content -Raw (Join-Path $root 'build-result.json')|ConvertFrom-Json
$freeze=Get-Content -Raw (Join-Path $root 'freeze.json')|ConvertFrom-Json
if(-not $build.passed){throw 'unchanged historical build prerequisite failed'}
$exe=Join-Path $root 'bin/dns_queue_cancel_c4806a3d.exe'
if(-not (Get-Item $exe).IsReadOnly -or (Hash $exe) -ne $freeze.binary.sha256){throw 'frozen executable mismatch'}
$null=Assert-Source
$before=Inventory 'historical-refusal-before'
Save-Json $before (Join-Path $root 'historical-refusal-before.json')
if($before.fixture_processes.Count -ne 0 -or $before.build_processes.Count -ne 0 -or $before.assigned_tcp_non_timewait.Count -ne 0 -or $before.assigned_udp_endpoints.Count -ne 0){throw 'process or endpoint prerequisite occupied; do not run'}
$runtime=Join-Path $root 'runtime/historical-refusal'
if(Test-Path -LiteralPath $runtime){throw 'new absolute runtime directory required'}
$arguments=@($runtime,'--refusal-only','--observe-attempts')
Save-Json ([ordered]@{started_utc=[DateTime]::UtcNow.ToString('o');binary_sha256=$freeze.binary.sha256;source_commit=$build.source_commit;tools=(Tools);timeout_seconds=300;max_runtime_commands=1;sequence=@('historical-refusal');process_environment_only=$true;env=@{TEMP=$env:TEMP;TMP=$env:TMP;CARGO_BUILD_JOBS=$env:CARGO_BUILD_JOBS;RUSTUP_AUTO_INSTALL=$env:RUSTUP_AUTO_INSTALL;OPENERGY_ALLOW_PORT_24800=$env:OPENERGY_ALLOW_PORT_24800;OPENERGY_DISABLE_DISCOVERY=$env:OPENERGY_DISABLE_DISCOVERY}}) (Join-Path $root 'run-start.json')
$info=[Diagnostics.ProcessStartInfo]::new($exe);$info.UseShellExecute=$false;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true;$info.WorkingDirectory=Join-Path $root 'runtime';foreach($a in $arguments){$info.ArgumentList.Add($a)}
$started=[DateTime]::UtcNow.ToString('o');$watch=[Diagnostics.Stopwatch]::StartNew();$process=[Diagnostics.Process]::Start($info);$startTime=$process.StartTime;$stdout=$process.StandardOutput.ReadToEndAsync();$stderr=$process.StandardError.ReadToEndAsync()
$record=[ordered]@{name='historical-refusal';command=@($exe)+$arguments;cwd=$info.WorkingDirectory;started_utc=$started;binary_sha256=$freeze.binary.sha256;source_commit=$build.source_commit;pid=$process.Id;session=$process.SessionId;priority=$process.PriorityClass.ToString();timeout_seconds=300;timed_out=$false;forced_cleanup=$false;samples=@()}
Save-Json $record (Join-Path $root 'historical-refusal-process.json')
while(-not $process.WaitForExit(500)){
 $record.samples+=@([ordered]@{at_ms=$watch.ElapsedMilliseconds;session=$process.SessionId;priority=$process.PriorityClass.ToString()})
 if($watch.Elapsed.TotalSeconds -ge 300){$record.timed_out=$true;$owned=Get-Process -Id $process.Id -ErrorAction Stop;if($owned.Path -ne $exe -or $owned.StartTime -ne $startTime){throw 'refuse cleanup of non-owned process'};$owned.Kill();$record.forced_cleanup=$true;break}
}
if(-not $process.WaitForExit(5000)){throw 'owned verifier did not exit after watchdog'};$watch.Stop()
$record.exit=$process.ExitCode;$record.ended_utc=[DateTime]::UtcNow.ToString('o');$record.elapsed_ms=$watch.ElapsedMilliseconds
[IO.File]::WriteAllText((Join-Path $root 'logs/historical-refusal.stdout'),$stdout.Result,[Text.UTF8Encoding]::new($false));[IO.File]::WriteAllText((Join-Path $root 'logs/historical-refusal.stderr'),$stderr.Result,[Text.UTF8Encoding]::new($false))
$record.stdout_sha256=Hash (Join-Path $root 'logs/historical-refusal.stdout');$record.stderr_sha256=Hash (Join-Path $root 'logs/historical-refusal.stderr')
$record.result_lines=@($stdout.Result -split '\r?\n'|Where-Object {$_ -match '^(RESULT|REFUSAL|FIXTURE |FIXTURE_SETUP|EXCLUSIVE_BIND|OS_CONTROL|EVENT|RESOLVER)'})
Save-Json $record (Join-Path $root 'historical-refusal-result.json');Save-Json @($record) (Join-Path $root 'run-result.json')
$identities=@(Get-ChildItem (Join-Path $root 'runtime') -File -Recurse -Force|Where-Object {$_.Name -in @('cert.pem','key.pem','.identity-lock')})
Save-Json @($identities|ForEach-Object {File-Record $_}) (Join-Path $root 'private-identities-removed.json')
foreach($file in $identities){Remove-Item -LiteralPath $file.FullName -Force}
$after=Inventory 'historical-refusal-after';Save-Json $after (Join-Path $root 'historical-refusal-after.json')
if($after.fixture_processes.Count -ne 0 -or $after.assigned_tcp_non_timewait.Count -ne 0 -or $after.assigned_udp_endpoints.Count -ne 0){throw 'owned cleanup failed'}
Write-Output ('ONE_HISTORICAL_REFUSAL_COMMAND_COMPLETED exit='+$record.exit+' elapsed_ms='+$record.elapsed_ms)
exit $record.exit
