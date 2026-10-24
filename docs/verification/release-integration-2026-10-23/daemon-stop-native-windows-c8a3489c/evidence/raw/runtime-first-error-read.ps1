$ErrorActionPreference='Stop';(Get-Process -Id $PID).PriorityClass='BelowNormal'
$root='<PROFILE>\AppData\Local\Temp\oe-stop-win-c8a3489c-e1ef124-24ce90f8'
foreach($n in @('runtime-first-stage-failure.stderr','runtime-first-failure.stderr','runtime-result.json')){$p=Join-Path $root $n;if(Test-Path -LiteralPath $p){Write-Output $n;[IO.File]::ReadAllText($p)}}
