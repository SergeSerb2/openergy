. (Join-Path $PSScriptRoot 'audit.ps1')
function Inventory($label){
 $all=@(& netstat.exe -ano);$nativeExit=$LASTEXITCODE
 $tcp=@($all|Where-Object {$_ -match '^\s*TCP\s+'});$udp=@($all|Where-Object {$_ -match '^\s*UDP\s+'})
 if($nativeExit -ne 0 -or $tcp.Count -eq 0 -or $udp.Count -eq 0){throw 'successful nonempty IPv4/IPv6 endpoint inventory required'}
 $all|Set-Content -LiteralPath (Join-Path $root ('logs/'+$label+'-all-endpoints.txt'))
 $processes=@(Get-CimInstance Win32_Process -ErrorAction Stop|Select-Object Name,ProcessId,ParentProcessId,ExecutablePath,CommandLine,CreationDate,SessionId)
 if($processes.Count -lt 10){throw 'successful nonempty process inventory required'}
 Save-Json $processes (Join-Path $root ('logs/'+$label+'-processes.json'))
 [ordered]@{recorded_utc=[DateTime]::UtcNow.ToString('o');endpoint_query=@('netstat.exe','-ano');endpoint_exit=$nativeExit;tcp_rows=$tcp.Count;udp_rows=$udp.Count;process_inventory_count=$processes.Count;assigned_tcp_listeners=@($tcp|Where-Object {$_ -match '^\s*TCP\s+\S+:(2510[0-9]|2511[0-9])\s+\S+\s+LISTENING\s+'});assigned_tcp_non_timewait=@($tcp|Where-Object {$_ -match '^\s*TCP\s+\S+:(2510[0-9]|2511[0-9])\s+' -and $_ -notmatch '\sTIME_WAIT\s+'});assigned_tcp_timewait=@($tcp|Where-Object {$_ -match '^\s*TCP\s+\S+:(2510[0-9]|2511[0-9])\s+\S+\s+TIME_WAIT\s+'});assigned_udp_endpoints=@($udp|Where-Object {$_ -match '^\s*UDP\s+\S+:(2510[0-9]|2511[0-9])\s+'});fixture_processes=@($processes|Where-Object {$_.ExecutablePath -and $_.ExecutablePath.StartsWith($root+'\',[StringComparison]::OrdinalIgnoreCase)});build_processes=@($processes|Where-Object {$_.Name -in @('cargo.exe','rustc.exe','cargo-clippy.exe','clippy-driver.exe')});sharing_processes=@($processes|Where-Object {$_.Name -match '^(synergy|deskflow|openergy)'})}
}
