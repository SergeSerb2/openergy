. (Join-Path $PSScriptRoot 'common.ps1')
if(Test-Path (Join-Path $root 'frozen-binaries.json')){throw 'binary freeze already completed'}
$rust=Get-Content -Raw (Join-Path $root 'rustc-result.json')|ConvertFrom-Json
$cs=Get-Content -Raw (Join-Path $root 'csharp-result.json')|ConvertFrom-Json
if($rust.exit -ne 0 -or $cs.exit -ne 0){throw 'durable compiler results did not pass'}
if((Hash (Join-Path $root 'owned_cleanup.rs')) -ne $rust.source_sha256 -or (Hash (Join-Path $root 'native-observer.cs')) -ne $cs.source_sha256 -or (Hash (Join-Path $root 'bin/OwnedNativeObserver.dll')) -ne $cs.dll_sha256){throw 'compiled input or native observer changed'}
if(@(Get-CimInstance Win32_Process -ErrorAction Stop|Where-Object {$_.Name -in @('rustc.exe','cargo.exe','cargo-clippy.exe','clippy-driver.exe')}).Count -ne 0){throw 'a compiler is active'}
foreach($f in @(Get-ChildItem (Join-Path $root 'bin') -File)){$f.IsReadOnly=$true}
Save-Json @((Get-Item (Join-Path $root 'owned_cleanup.rs')),(Get-Item (Join-Path $root 'native-observer.cs'))|ForEach-Object {File-Record $_}) (Join-Path $root 'source-metadata.json')
Save-Json @(Get-ChildItem (Join-Path $root 'bin') -File|ForEach-Object {File-Record $_}) (Join-Path $root 'frozen-binaries.json')
Write-Output 'DURABLE_COMPILERS_VERIFIED_AND_FROZEN_NO_REBUILD'
