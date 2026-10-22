# Real session-0 CLI/pipe verification. No desktop input or production config.
$ErrorActionPreference='Stop'
if ([Diagnostics.Process]::GetCurrentProcess().SessionId -ne 0) { throw 'run only through session-0 SSH' }
$root="$env:USERPROFILE\openergy-build"
$exe="$root\target\release\openergy.exe"
$lab="$root\lab\checks"
$env:OPENERGY_SOCKET='\\.\pipe\openergy-port-control-lab'
$p=Start-Process $exe -ArgumentList @('daemon','--platform','virtual','--config',"$lab\config.toml") -PassThru -RedirectStandardError "$lab\errors.stderr" -RedirectStandardOutput "$lab\errors.stdout"
$p.Id | Set-Content "$lab\errors.pid"
try {
 Start-Sleep -Milliseconds 500
 for($i=0;$i -lt 100;$i++) {
  $c=[IO.Pipes.NamedPipeClientStream]::new('.','openergy-port-control-lab',[IO.Pipes.PipeDirection]::InOut)
  $c.Connect(2000);$c.Dispose()
 }
 $null=& $exe status 2> "$lab\errors-cli.stderr"
 if($LASTEXITCODE){throw 'quick-close clients stopped daemon'}
 Write-Output 'PASS 100 clients connect and close before a request'
 $c=[IO.Pipes.NamedPipeClientStream]::new('.','openergy-port-control-lab',[IO.Pipes.PipeDirection]::InOut)
 $c.Connect(2000);$c.Write([byte[]]@(1,0,0,1),0,4);Start-Sleep -Milliseconds 100;$c.Dispose()
 $null=& $exe status 2> "$lab\errors-cli.stderr"
 if($LASTEXITCODE){throw 'oversized frame stopped daemon'}
 Write-Output 'PASS oversized frame closes only that client'
} finally {
 $null=& $exe ctl daemon.stop --raw 2> "$lab\errors-cli.stderr"
 if(!$p.WaitForExit(6000)){throw 'error-check daemon exit deadline'}
 "ERROR_CHECK_EXIT=$($p.ExitCode)"
}
$server=[IO.Pipes.NamedPipeServerStream]::new('openergy-port-busy-lab',[IO.Pipes.PipeDirection]::InOut,1,[IO.Pipes.PipeTransmissionMode]::Byte,[IO.Pipes.PipeOptions]::Asynchronous)
$c=[IO.Pipes.NamedPipeClientStream]::new('.','openergy-port-busy-lab',[IO.Pipes.PipeDirection]::InOut)
try {
 $c.Connect(2000);$server.WaitForConnection()
 $timer=[Diagnostics.Stopwatch]::StartNew()
 $null=& $exe status --socket '\\.\pipe\openergy-port-busy-lab' 2> "$lab\busy.stderr"
 if($LASTEXITCODE -ne 1 -or $timer.Elapsed.TotalSeconds -gt 2 -or $timer.Elapsed.TotalSeconds -lt .8){throw 'busy pipe deadline'}
 Write-Output 'PASS busy pipe fails after about 1 second'
} finally {$c.Dispose();$server.Dispose()}
