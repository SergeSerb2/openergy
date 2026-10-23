. (Join-Path $PSScriptRoot 'common.ps1')
if(Test-Path -LiteralPath (Join-Path $root 'freeze.json')){throw 'already frozen'}
$build=Get-Content -Raw -LiteralPath (Join-Path $root 'build-result.json')|ConvertFrom-Json
if(-not $build.passed -or @($build.steps|Where-Object {$_.exit -ne 0}).Count -ne 0 -or $build.steps.Count -ne 3){throw 'all original build steps must pass'}
$inventory=Inventory 'before-freeze'
if($inventory.build_processes.Count -ne 0 -or $inventory.fixture_processes.Count -ne 0){throw 'build or fixture executable remains'}
$null=Assert-Source
$release=Join-Path $root 'target/x86_64-pc-windows-msvc/release'
foreach($pair in @(@('openergy.exe','openergy.exe'),@('examples/discovery_recovery.exe','discovery_recovery.exe'),@('examples/discovery.exe','discovery.exe'))){$original=Join-Path $release $pair[0];$frozen=Join-Path $root ('bin/'+$pair[1]);if((Hash $original) -ne (Hash $frozen)){throw 'original copied binary mismatch'}}
$expected=@('libopenergy_config.rlib','libopenergy_ipc.rlib','libopenergy_net.rlib','libopenergy_platform_windows.rlib','libopenergy_platform.rlib','libopenergy_protocol.rlib','libopenergy_session.rlib')
$selected=@(Get-ChildItem -LiteralPath $release -File -Filter 'libopenergy*.rlib');$vendor=@(Get-ChildItem -LiteralPath (Join-Path $release 'deps') -File -Filter 'libmdns_sd-*.rlib')
if((($selected.Name|Sort-Object) -join ',') -ne (($expected|Sort-Object) -join ',') -or $vendor.Count -ne 2){throw 'observed library inventory changed'}
foreach($f in @($selected)+@($vendor)){Copy-Item -LiteralPath $f.FullName -Destination (Join-Path $root ('bin/libraries/'+$f.Name))}
$artifacts=@(Get-ChildItem -LiteralPath (Join-Path $root 'target') -File -Recurse -Force)+@(Get-ChildItem -LiteralPath (Join-Path $root 'bin') -File -Recurse -Force)
foreach($f in $artifacts){$f.IsReadOnly=$true}
Save-Json @($artifacts|ForEach-Object {File-Record $_}) (Join-Path $root 'artifact-manifest.json')
$fingerprints=@(Get-ChildItem -LiteralPath (Join-Path $release '.fingerprint') -Directory -Filter 'mdns-sd-*'|ForEach-Object {Get-ChildItem -LiteralPath $_.FullName -Filter '*.json' -File}|ForEach-Object {[ordered]@{path=$_.FullName.Substring($root.Length+1).Replace('\','/');sha256=(Hash $_.FullName);content=(Get-Content -Raw -LiteralPath $_.FullName|ConvertFrom-Json)}})
Save-Json ([ordered]@{frozen_utc=[DateTime]::UtcNow.ToString('o');artifact_files=$artifacts.Count;artifact_manifest_sha256=(Hash (Join-Path $root 'artifact-manifest.json'));frozen_bin=@(Get-ChildItem -LiteralPath (Join-Path $root 'bin') -File -Recurse|ForEach-Object {File-Record $_});source=(Assert-Source);acl=(Assert-ACL);vendor_variants=$fingerprints;prior_freeze_failure='Original build.ps1 expected eight libraries but observed seven selected libraries plus two mdns-sd variants. No build/probe/assertion rerun or source change.'}) (Join-Path $root 'freeze.json')
Write-Output 'FREEZE_PASS'
