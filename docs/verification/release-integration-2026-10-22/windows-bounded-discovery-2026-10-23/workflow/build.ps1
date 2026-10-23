. (Join-Path $PSScriptRoot 'common.ps1')
if(Test-Path -LiteralPath (Join-Path $root 'build-start.json')){throw 'build already attempted; inspect durable state'}
if(@(Get-ChildItem -LiteralPath (Join-Path $root 'source') -Force).Count -ne 0 -or @(Get-ChildItem -LiteralPath (Join-Path $root 'target') -Force).Count -ne 0){throw 'source/target must be new and empty'}
$before=Inventory 'preflight'
if($before.build_processes.Count -ne 0 -or $before.fixture_processes.Count -ne 0 -or $before.assigned_tcp_listeners.Count -ne 0 -or $before.assigned_udp_endpoints.Count -ne 0){throw 'active build/owned process or assigned endpoint exists'}
$pre=[ordered]@{inventory=$before;tools=(Tools);acl=(Assert-ACL);source_files_before_extraction=0;target_files=0;bin_files=0;session=(Get-Process -Id $PID).SessionId;priority=(Get-Process -Id $PID).PriorityClass.ToString();jobs=2;recorded_utc=[DateTime]::UtcNow.ToString('o')}
Save-Json $pre (Join-Path $root 'preflight.json')
$manifest=Get-Content -Raw -LiteralPath (Join-Path $root 'source-manifest.json')|ConvertFrom-Json
if($manifest.commit -ne 'e4f48ba47ea69aa76a08de5cf0cf61c68d903c03' -or (Hash (Join-Path $root 'source.tar')) -ne $manifest.archive_sha256){throw 'archive/commit mismatch'}
& tar.exe -xf (Join-Path $root 'source.tar') -C (Join-Path $root 'source')
if($LASTEXITCODE -ne 0){throw 'tar extraction failed'}
Get-ChildItem -LiteralPath (Join-Path $root 'source') -File -Recurse -Force|ForEach-Object {$_.IsReadOnly=$true}
$sourceBefore=Assert-Source
$env:CARGO_TARGET_DIR=Join-Path $root 'target';$env:RUSTUP_TOOLCHAIN='1.99.1-x86_64-pc-windows-msvc'
Set-Location -LiteralPath (Join-Path $root 'source')
$packages=@('-p','openergy-net','-p','openergy-config','-p','openergy-session','-p','openergy-protocol','-p','openergy-ipc','-p','openergy-platform','-p','openergy-platform-windows','-p','openergy-daemon')
$base=@('--locked','--offline','--release','--target','x86_64-pc-windows-msvc','--jobs','2')
$steps=@(@{name='clippy';args=(@('clippy')+$base+$packages+@('--all-targets','--','-D','warnings'))},@{name='release';args=(@('build')+$base+$packages)},@{name='examples';args=(@('build')+$base+@('-p','openergy-net','--example','discovery_recovery','--example','discovery'))})
$result=[ordered]@{source_commit=$manifest.commit;started_utc=[DateTime]::UtcNow.ToString('o');source_before=$sourceBefore;preflight_sha256=(Hash (Join-Path $root 'preflight.json'));session=(Get-Process -Id $PID).SessionId;priority=(Get-Process -Id $PID).PriorityClass.ToString();pid=$PID;jobs=2;target='x86_64-pc-windows-msvc';toolchain=$env:RUSTUP_TOOLCHAIN;passed=$false;steps=@()}
Save-Json $result (Join-Path $root 'build-start.json')
foreach($step in $steps){
 $watch=[Diagnostics.Stopwatch]::StartNew();$started=[DateTime]::UtcNow.ToString('o');$args=$step.args
 Save-Json ([ordered]@{step=$step.name;started_utc=$started;arguments=$args;pid=$PID}) (Join-Path $root 'build-progress.json')
 & (Join-Path $toolRoot 'cargo.exe') @args *> (Join-Path $root ('logs/'+$step.name+'.log'))
 $code=$LASTEXITCODE;$watch.Stop()
 $result.steps+=@([ordered]@{name=$step.name;arguments=$args;started_utc=$started;ended_utc=[DateTime]::UtcNow.ToString('o');elapsed_ms=$watch.ElapsedMilliseconds;exit=$code;log_sha256=(Hash (Join-Path $root ('logs/'+$step.name+'.log')))})
 Save-Json $result (Join-Path $root 'build-result.json')
 if($code -ne 0){exit $code}
}
$result.source_after=Assert-Source;$result.ended_utc=[DateTime]::UtcNow.ToString('o');$result.passed=$true
Save-Json $result (Join-Path $root 'build-result.json')
$release=Join-Path $root 'target/x86_64-pc-windows-msvc/release'
foreach($pair in @(@('openergy.exe','openergy.exe'),@('examples/discovery_recovery.exe','discovery_recovery.exe'),@('examples/discovery.exe','discovery.exe'))){Copy-Item -LiteralPath (Join-Path $release $pair[0]) -Destination (Join-Path $root ('bin/'+$pair[1]))}
[IO.Directory]::CreateDirectory((Join-Path $root 'bin/libraries'))|Out-Null
$libs=@(Get-ChildItem -LiteralPath $release -File -Filter 'libopenergy*.rlib')+@(Get-ChildItem -LiteralPath (Join-Path $release 'deps') -File -Filter 'libmdns_sd-*.rlib')
if($libs.Count -ne 8){throw 'expected seven selected libraries and one vendored mdns-sd library'}
foreach($f in $libs){Copy-Item -LiteralPath $f.FullName -Destination (Join-Path $root ('bin/libraries/'+$f.Name))}
$artifacts=@(Get-ChildItem -LiteralPath (Join-Path $root 'target') -File -Recurse -Force)+@(Get-ChildItem -LiteralPath (Join-Path $root 'bin') -File -Recurse -Force)
foreach($f in $artifacts){$f.IsReadOnly=$true}
Save-Json @($artifacts|ForEach-Object {File-Record $_}) (Join-Path $root 'artifact-manifest.json')
Save-Json ([ordered]@{frozen_utc=[DateTime]::UtcNow.ToString('o');artifact_files=$artifacts.Count;artifact_manifest_sha256=(Hash (Join-Path $root 'artifact-manifest.json'));frozen_bin=@(Get-ChildItem -LiteralPath (Join-Path $root 'bin') -File -Recurse|ForEach-Object {File-Record $_});source=(Assert-Source);acl=(Assert-ACL)}) (Join-Path $root 'freeze.json')
Write-Output 'BUILD_AND_FREEZE_PASS'
