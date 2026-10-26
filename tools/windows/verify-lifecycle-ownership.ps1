# Real CLI lifecycle regressions. All processes/configs are owned temp virtual peers.
param([Parameter(Mandatory=$true)][string]$BinaryPath, [string]$ScriptsDir=$PSScriptRoot)
$ErrorActionPreference='Stop'
$root=Join-Path ([IO.Path]::GetTempPath()) ('oe-owned-lifecycle-'+[guid]::NewGuid().ToString('N'))
$install=Join-Path $root 'install'
$data=Join-Path $root 'data'
$other=Join-Path $root 'independent'
$socket=Join-Path $data 'openergy.sock'
$manager=Join-Path $ScriptsDir 'openergy.ps1'
$independent=$null
$passed=$false
New-Item -ItemType Directory -Path $other | Out-Null
[IO.File]::WriteAllText("$other\config.toml", "version = 1`nname = `"owned-independent`"`nrole = `"off`"`n[server]`nlisten = `"127.0.0.1:25244`"`nport = 25244`n")
function Check($Value,$Name) { if(!$Value){throw $Name}; "PASS $Name" }
function Manage($Action) { & $manager -Action $Action -InstallDir $install -DataDir $data }
function Spawn([string[]]$Arguments,[string]$Executable=$BinaryPath) {
    $info=[Diagnostics.ProcessStartInfo]::new([IO.Path]::GetFullPath($Executable))
    $info.UseShellExecute=$false; $info.RedirectStandardOutput=$true; $info.RedirectStandardError=$true
    foreach($arg in $Arguments){$info.ArgumentList.Add($arg)}
    $info.Environment['OPENERGY_CONFIG']="$other\config.toml"
    $info.Environment['OPENERGY_CONFIG_DIR']=$other
    $info.Environment['OPENERGY_TLS_DIR']="$other\tls"
    $info.Environment['OPENERGY_SOCKET']=$socket
    $info.Environment['OPENERGY_DISABLE_DISCOVERY']='1'
    $info.Environment['OPENERGY_ALLOW_PORT_24800']='0'
    $p=[Diagnostics.Process]::Start($info)
    $p | Add-Member NoteProperty Output $p.StandardOutput.ReadToEndAsync()
    $p | Add-Member NoteProperty Errors $p.StandardError.ReadToEndAsync()
    return $p
}
function Other-Ctl($Method) {
    $p=Spawn @('ctl',$Method,'--socket',$socket,'--raw')
    try {
        if(!$p.WaitForExit(5000)){$p.Kill();$p.WaitForExit();throw 'CLI deadline'}
        if($p.ExitCode){throw 'independent CLI failed'}
        $p.Output.Result | ConvertFrom-Json
    } finally {$p.Dispose()}
}
try {
    $null=& $manager Install -InstallDir $install -DataDir $data -BinaryPath $BinaryPath -NoStartup -Platform virtual -Port 25243
    $independent=Spawn @('daemon','--platform','virtual','--config',"$other\config.toml",'--socket',$socket)
    $ready=$false
    for($i=0;$i -lt 30;$i++){try{$s=Other-Ctl 'status.get';if($s.daemon.pid -eq $independent.Id){$ready=$true;break}}catch{};Start-Sleep -Milliseconds 100}
    Check $ready 'owned independent daemon owns selected endpoint'
    $rejected=$false
    try{$null=Manage Start}catch{$rejected=$true}
    "OCCUPIED_START_REJECTED=$rejected"
    $null=Manage Stop
    $survives=!$independent.WaitForExit(500)
    "INDEPENDENT_SURVIVES_STOP=$survives"
    Check ($rejected -and $survives) 'occupied endpoint rejected and separate daemon survives managed Stop'
    $s=Other-Ctl 'status.get';Check ($s.daemon.pid -eq $independent.Id) 'independent endpoint still serves real status'
    $null=Other-Ctl 'daemon.stop';Check ($independent.WaitForExit(6000) -and $independent.ExitCode -eq 0) 'independent daemon exits only on its explicit stop'
    $null=Manage Start
    $run="$data\runner.json"
    $original=Get-Content -LiteralPath $run -Raw
    $record=$original | ConvertFrom-Json
    $record.child_started='0'
    $record | ConvertTo-Json | Set-Content -LiteralPath $run
    $rejected=$false
    try{$null=Manage Status}catch{$rejected=$true}finally{[IO.File]::WriteAllText($run,$original)}
    Check $rejected 'stale child start time rejected'
    $otherSocket=Join-Path $other 'separate.sock'
    $mismatch=Spawn @('daemon','--platform','virtual','--config',"$other\config.toml",'--socket',$otherSocket) (Join-Path $install 'openergy.exe')
    try {
        $probe=Spawn @('ctl','status.get','--socket',$otherSocket,'--raw')
        if(!$probe.WaitForExit(5000)){$probe.Kill();throw 'separate daemon startup deadline'}
        Check ($probe.ExitCode -eq 0) 'second owned daemon serves a different pipe'
        $altered=$original | ConvertFrom-Json
        $altered.child_pid=$mismatch.Id
        $altered.child_started=$mismatch.StartTime.ToUniversalTime().Ticks.ToString()
        $altered | ConvertTo-Json | Set-Content -LiteralPath $run
        $rejected=$false
        try{$null=Manage Status}catch{$rejected=$_.Exception.ToString().Contains('not the supervised daemon')}finally{[IO.File]::WriteAllText($run,$original)}
        Check ($rejected -and !$mismatch.HasExited) 'connected pipe server PID must match the recorded child'
    } finally {if(!$mismatch.HasExited){$mismatch.Kill();$null=$mismatch.WaitForExit(5000)}}
    $null=Manage Stop
    # Prove the read-only full installation preflight exists before migration calls.
    $null=Manage Validate
    $marker="$install\install.json"
    $original=Get-Content -LiteralPath $marker -Raw
    foreach($field in @('sid','data_dir')) {
        $invalid=$original | ConvertFrom-Json
        if($field -eq 'sid'){$invalid.sid='S-1-0-0'}else{$invalid.data_dir=Join-Path $root 'wrong-data'}
        $invalid | ConvertTo-Json | Set-Content -LiteralPath $marker
        $rejected=$false
        try{& (Join-Path $ScriptsDir 'switch-from-synergy.ps1') -Start -InstallDir $install -DataDir $data | Out-Null}catch{$rejected=$_.Exception.Message.Contains('owner or paths do not match')}
        Check ($rejected -and !(Test-Path "$data\synergy-state.json")) "migration rejects invalid $field before service cutover"
    }
    [IO.File]::WriteAllText($marker,$original)
    $rejected=$false
    try{& (Join-Path $ScriptsDir 'switch-from-synergy.ps1') -Rollback -InstallDir $install -DataDir $data -BrokerPath (Join-Path $root 'missing\openergy-migration-admin.ps1') | Out-Null}catch{$rejected=$_.Exception.Message.Contains('provision the reviewed helper first')}
    Check $rejected 'rollback without protected helper refused before any control change'
    $null=Manage Uninstall
    $passed=$true
    'WINDOWS_LIFECYCLE_OWNERSHIP_PASS'
} finally {
    if($independent -and !$independent.HasExited){$independent.Kill();$null=$independent.WaitForExit(5000)}
    if(Test-Path "$install\install.json") {
        # Restore only the marker saved by this driver, then stop/uninstall this temp install.
        if($original -and $original.Contains('install_dir')){[IO.File]::WriteAllText("$install\install.json",$original)}
        try{$null=Manage Uninstall}catch{Write-Warning 'Owned lifecycle lab retained for cleanup';throw}
    }
    if($passed){Remove-Item -LiteralPath $root -Recurse -Force}else{"Retained lab: $root"}
}
