$ErrorActionPreference='Stop';(Get-Process -Id $PID).PriorityClass='BelowNormal'
$root='<PROFILE>\AppData\Local\Temp\oe-stop-win-c8a3489c-e1ef124-24ce90f8'
if((Split-Path $root -Leaf) -ne 'oe-stop-win-c8a3489c-e1ef124-24ce90f8' -or (Split-Path $root -Parent) -ne '<PROFILE>\AppData\Local\Temp'){throw 'owned root identity mismatch'}
$owned=@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -ne $PID -and (($_.ExecutablePath -and $_.ExecutablePath.StartsWith($root+'\',[StringComparison]::OrdinalIgnoreCase)) -or ($_.CommandLine -and $_.CommandLine.Contains($root)))}|Select-Object Name,ProcessId,CreationDate,ExecutablePath,CommandLine)
if($owned.Count){throw 'owned process remains; fixture retained'}
Remove-Item -LiteralPath $root -Recurse -Force
[ordered]@{recorded_utc=[DateTime]::UtcNow.ToString('o');owned_processes_before_removal=$owned;owned_root_removed=(-not (Test-Path -LiteralPath $root));removed_only_fresh_owned_root=$true;artifact_and_raw_evidence_verified_and_retained_privately_before_removal=$true}|ConvertTo-Json -Depth 8 -Compress
