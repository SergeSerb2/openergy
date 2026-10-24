$ErrorActionPreference='Stop';(Get-Process -Id $PID).PriorityClass='BelowNormal'
$root='<PROFILE>\AppData\Local\Temp\oe-stop-win-c8a3489c-e1ef124-11fa8bad'
$names=@('transfer-confirmed.json','source-before.json','shared-before.json','qualification.json','qualify-first-stage-failure.stderr','build-start.json','clippy-start.json')
@(foreach($n in $names){$p=Join-Path $root $n;[ordered]@{name=$n;exists=(Test-Path -LiteralPath $p)}})|ConvertTo-Json -Compress
