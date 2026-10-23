$ErrorActionPreference='Stop';$ProgressPreference='SilentlyContinue'
$root=$PSScriptRoot;(Get-Process -Id $PID).PriorityClass='BelowNormal'
$env:TEMP=Join-Path $root 'temp';$env:TMP=$env:TEMP;$env:CARGO_BUILD_JOBS='2'
$env:OPENERGY_ALLOW_PORT_24800='0';$env:OPENERGY_DISABLE_DISCOVERY='1'
$compile=Get-Content -Raw -LiteralPath (Join-Path $root 'compile-result.json')|ConvertFrom-Json
if($compile.exit -ne 0){throw 'helper compile failed'}
if((Get-Process -Id $PID).SessionId -ne 0){throw 'session 0 required'}
$binary=Join-Path $root 'bin/windows_dns_supplement.exe'
if((Get-FileHash -LiteralPath $binary -Algorithm SHA256).Hash -ne $compile.binary.sha256){throw 'helper binary mismatch'}
$results=@()
foreach($mode in @('os','driver','contracts')){
 $directory=Join-Path $root ('runtime/'+$mode)
 if(Test-Path -LiteralPath $directory){throw 'runtime directory already exists'}
 $tcp=@(& netstat.exe -ano -p TCP);$netExit=$LASTEXITCODE
 if($netExit -ne 0 -or $tcp.Count -lt 3){throw 'endpoint audit failed'}
 if(@($tcp|Where-Object {$_ -match '^s*TCPs+S+:(2510[0-9]|2511[0-9])s+S+s+LISTENINGs+'}).Count -ne 0){throw 'assigned port occupied'}
 $watch=[Diagnostics.Stopwatch]::StartNew();$started=[DateTime]::UtcNow.ToString('o')
 $process=Start-Process -FilePath $binary -ArgumentList @($mode,$directory) -WorkingDirectory (Join-Path $root 'runtime') -RedirectStandardOutput (Join-Path $root ('logs/'+$mode+'.stdout')) -RedirectStandardError (Join-Path $root ('logs/'+$mode+'.stderr')) -PassThru
 $record=[ordered]@{mode=$mode;arguments=@($mode,$directory);binary_sha256=$compile.binary.sha256;source_sha256=$compile.source_sha256;pid=$process.Id;session=$process.SessionId;priority=$process.PriorityClass.ToString();started_utc=$started;timeout_seconds=180;timed_out=$false;forced_cleanup=$false}
 $record|ConvertTo-Json -Depth 6|Set-Content -LiteralPath (Join-Path $root ($mode+'-process.json'))
 while(-not $process.WaitForExit(1000)){if($watch.Elapsed.TotalSeconds -ge 180){$record.timed_out=$true;$owned=Get-Process -Id $process.Id -ErrorAction Stop;if($owned.Path -ne $binary){throw 'refuse cleanup of non-fixture process'};$owned.Kill();$record.forced_cleanup=$true;break}}
 $process.WaitForExit();$watch.Stop();$record.exit=$process.ExitCode;$record.elapsed_ms=$watch.ElapsedMilliseconds;$record.ended_utc=[DateTime]::UtcNow.ToString('o')
 foreach($kind in @('stdout','stderr')){$record[$kind+'_sha256']=(Get-FileHash -LiteralPath (Join-Path $root ('logs/'+$mode+'.'+$kind)) -Algorithm SHA256).Hash.ToLower()}
 $record|ConvertTo-Json -Depth 6|Set-Content -LiteralPath (Join-Path $root ($mode+'-result.json'))
 $results+=$record
}
$results|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $root 'runtime-result.json')
$results|ConvertTo-Json -Depth 8
# Preserve any strict OS matrix failure; driver/contracts run independently.
if(@($results|Where-Object {$_.exit -ne 0 -or $_.timed_out}).Count -gt 0){exit 1}
