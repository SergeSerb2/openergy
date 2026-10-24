$ErrorActionPreference='Stop';(Get-Process -Id $PID).PriorityClass='BelowNormal'
$root='<PROFILE>\AppData\Local\Temp\oe-stop-win-c8a3489c-e1ef124-24ce90f8'
$r=Get-Content -Raw -LiteralPath (Join-Path $root 'runtime-result.json')|ConvertFrom-Json
$identities=@(Get-ChildItem -LiteralPath $root -Filter 'runtime-*-identity.json'|ForEach-Object {Get-Content -Raw -LiteralPath $_.FullName|ConvertFrom-Json})+@($r.controller)
$states=@(foreach($id in $identities){$p=Get-CimInstance Win32_Process -Filter ('ProcessId='+$id.pid);[ordered]@{owned_pid=$id.pid;owned_creation_utc=$id.creation_utc;pid_currently_absent=($null -eq $p);same_identity_remains=($p -and $p.CreationDate.ToUniversalTime().ToString('o') -eq $id.creation_utc)}})
$pipeNames=@('oe-stop-c8a3489c-daemon-24ce90f8','oe-stop-c8a3489c-observer-24ce90f8')
$present=@([IO.Directory]::EnumerateFiles('\\.\pipe\')|Where-Object {[IO.Path]::GetFileName($_) -in $pipeNames})
$sockets=[Collections.Generic.List[object]]::new();$binds=@()
try{foreach($family in @([Net.Sockets.AddressFamily]::InterNetwork,[Net.Sockets.AddressFamily]::InterNetworkV6)){$s=[Net.Sockets.Socket]::new($family,[Net.Sockets.SocketType]::Stream,[Net.Sockets.ProtocolType]::Tcp);$sockets.Add($s);$s.ExclusiveAddressUse=$true;if($family -eq [Net.Sockets.AddressFamily]::InterNetworkV6){$s.DualMode=$false;$address=[Net.IPAddress]::IPv6Loopback}else{$address=[Net.IPAddress]::Loopback};$s.Bind([Net.IPEndPoint]::new($address,59367));$binds+=@([ordered]@{family=$family.ToString();exclusive_bind=$true;listen_called=$false;connect_called=$false})}}finally{foreach($s in $sockets){$s.Dispose()}}
$value=[ordered]@{recorded_utc=[DateTime]::UtcNow.ToString('o');owned_identities=$states;private_pipe_names_present=$present;port=59367;exclusive_loopback_bind=$binds;no_product_relaunch=$true;no_application_request=$true;normal_stop_requests_sent=$r.stops_sent;recovery_exit_code_not_captured=$true;normal_exit_timing_not_measured=$true;recovery_metadata_limit='only_after_declared_deadline=true is a generic harness field. No stop request was sent, so the 10-second normal-stop deadline never began. Recovery followed a log-reader error before the 15-second readiness ceiling.'}
$value.passed=(@($states|Where-Object {$_.same_identity_remains}).Count -eq 0 -and $present.Count -eq 0 -and $binds.Count -eq 2)
$value|ConvertTo-Json -Depth 10 -Compress
if(-not $value.passed){exit 1}
