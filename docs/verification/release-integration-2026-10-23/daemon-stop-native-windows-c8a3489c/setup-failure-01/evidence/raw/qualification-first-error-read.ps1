$ErrorActionPreference='Stop';(Get-Process -Id $PID).PriorityClass='BelowNormal'
$p='<PROFILE>\AppData\Local\Temp\oe-stop-win-c8a3489c-e1ef124-11fa8bad\qualify-first-stage-failure.stderr'
[IO.File]::ReadAllText($p)
