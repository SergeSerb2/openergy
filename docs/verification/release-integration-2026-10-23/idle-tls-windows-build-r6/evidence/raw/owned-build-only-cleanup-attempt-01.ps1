$ErrorActionPreference='Stop'
(Get-Process -Id $PID).PriorityClass='BelowNormal'
$root='<PROFILE>\AppData\Local\Temp\oe-idle-tls-win-r6-a3f6033f-06b9a13-dc824a67'
if((Split-Path $root -Leaf) -ne 'oe-idle-tls-win-r6-a3f6033f-06b9a13-dc824a67' -or (Split-Path $root -Parent) -ne '<PROFILE>\AppData\Local\Temp'){throw 'owned root identity mismatch'}
$owned=@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -ne $PID -and (($_.ExecutablePath -and $_.ExecutablePath.StartsWith($root+'\',[StringComparison]::OrdinalIgnoreCase)) -or ($_.CommandLine -and $_.CommandLine.Contains($root)))}|Select-Object Name,ProcessId,CreationDate,ExecutablePath,CommandLine)
if($owned.Count){throw 'owned process remains; retain fixture'}
Remove-Item -LiteralPath $root -Recurse -Force
[ordered]@{recorded_utc=[DateTime]::UtcNow.ToString('o');owned_processes_before_removal=$owned;owned_root_removed=(-not (Test-Path -LiteralPath $root));old_failed_root_absent=(-not (Test-Path -LiteralPath (Join-Path '<PROFILE>\AppData\Local\Temp' 'oe-idle-tls-r2-build-773e34c7-06b9a13')));removed_only_fresh_owned_root=$true;runtime_invocations=0}|ConvertTo-Json -Depth 8 -Compress
