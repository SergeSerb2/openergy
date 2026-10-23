param([Parameter(Mandatory=$true)][string]$FrozenPath,[Parameter(Mandatory=$true)][string]$PreviousDiagnosticPath)
. (Join-Path $PSScriptRoot 'common.ps1')
if(Test-Path (Join-Path $root 'build-start.json')){throw 'build already attempted; inspect durable state'}
if(@(Get-CimInstance Win32_Process -ErrorAction Stop|Where-Object {$_.Name -in @('cargo.exe','rustc.exe','cargo-clippy.exe','clippy-driver.exe')}).Count -ne 0){throw 'another build is active'}
$oldInputs=@(foreach($label in @('frozen','startup-diagnostic')){
 $path=if($label -eq 'frozen'){$FrozenPath}else{$PreviousDiagnosticPath}
 $acl=Get-Acl -LiteralPath $path
 if(-not $acl.AreAccessRulesProtected){throw 'old fixture ACL is not protected'}
 [ordered]@{label=$label;root=$path;files=@(Get-ChildItem -LiteralPath $path -File -Recurse -Force|ForEach-Object {[ordered]@{path=$_.FullName.Substring($path.Length+1).Replace('\','/');bytes=$_.Length;readonly=$_.IsReadOnly;sha256=(Hash $_.FullName)}})}
})
Save-Json $oldInputs (Join-Path $root 'old-inputs-before.json')
$manifest=Get-Content -Raw (Join-Path $FrozenPath 'source-manifest.json')|ConvertFrom-Json
$artifacts=Get-Content -Raw (Join-Path $FrozenPath 'artifact-manifest.json')|ConvertFrom-Json
if($manifest.commit -ne 'e4f48ba47ea69aa76a08de5cf0cf61c68d903c03'){throw 'wrong frozen source'}
foreach($e in $manifest.files){$f=Get-Item (Join-Path $FrozenPath ('source/'+$e.path));if(-not $f.IsReadOnly -or $f.Length -ne $e.bytes -or (Hash $f.FullName) -ne $e.sha256){throw 'old source mismatch'}}
foreach($e in $artifacts){$f=Get-Item (Join-Path $FrozenPath $e.path);if(-not $f.IsReadOnly -or $f.Length -ne $e.bytes -or (Hash $f.FullName) -ne $e.sha256){throw 'old artifact mismatch'}}
$release=Join-Path $FrozenPath 'target/x86_64-pc-windows-msvc/release'
$net=Join-Path $release 'libopenergy_net.rlib';$netHash=Hash $net
if($netHash -ne 'c36d0a889628e3bfd84a60818df38ea1e3bb3ff0e78f5fd87d53249f64ef03ff'){throw 'wrong frozen net library'}
$matching=@(Get-ChildItem (Join-Path $release 'deps') -File -Filter 'libopenergy_net-*.rlib'|Where-Object {(Hash $_.FullName) -eq $netHash})
if($matching.Count -ne 1){throw 'cannot identify exact root net fingerprint'}
$fingerprint=$matching[0].BaseName.Substring('libopenergy_net-'.Length)
$netJson=Join-Path $release ('.fingerprint/openergy-net-'+$fingerprint+'/lib-openergy_net.json')
$metadata=Get-Content -Raw $netJson|ConvertFrom-Json
$selected=@{}
foreach($crate in @('log','mdns_sd')){
 $wanted=@($metadata.deps|Where-Object {$_[1] -eq $crate});if($wanted.Count -ne 1){throw 'missing net dependency'}
 $value=[UInt64]$wanted[0][3];$package=$crate.Replace('_','-')
 $matches=@(foreach($dir in @(Get-ChildItem (Join-Path $release '.fingerprint') -Directory -Filter ($package+'-*'))){
  $finger=Join-Path $dir.FullName ('lib-'+$crate);if(-not (Test-Path $finger)){continue};$hex=(Get-Content -Raw $finger).Trim()
  if($hex -notmatch '^[0-9a-f]{16}$'){continue};$bytes=@(for($i=0;$i -lt 16;$i+=2){[Convert]::ToByte($hex.Substring($i,2),16)})
  if([BitConverter]::ToUInt64([byte[]]$bytes,0) -ne $value){continue}
  $lib=Join-Path $release ('deps/lib'+$crate+'-'+$dir.Name.Substring($package.Length+1)+'.rlib')
  if(Test-Path $lib){[ordered]@{crate=$crate;path=$lib;sha256=(Hash $lib);fingerprint_hex=$hex;recorded_dependency_value=$value}}
 })
 if($matches.Count -ne 1){throw ('cannot identify exact net dependency '+$crate)};$selected[$crate]=$matches[0]
}
$source=Join-Path $root 'owned_cleanup.rs';(Get-Item $source).IsReadOnly=$true
$out=Join-Path $root 'bin/owned_cleanup.exe'
$args=@('--edition=2021','--crate-name','owned_cleanup','--target','x86_64-pc-windows-msvc','-D','warnings','-C','opt-level=3','-C','panic=abort','-C','codegen-units=1','-C','lto=thin','-C','strip=symbols','--extern',('openergy_net='+$net),'--extern',('log='+$selected.log.path),'-L',('dependency='+(Join-Path $release 'deps')),'-L',('dependency='+(Join-Path $FrozenPath 'target/release/deps')))
foreach($dir in @(Get-ChildItem (Join-Path $release 'build') -Directory|ForEach-Object {Join-Path $_.FullName 'out'}|Where-Object {Test-Path $_})){$args+=@('-L',('native='+$dir))}
$args+=@($source,'-o',$out)
$pre=[ordered]@{started_utc=[DateTime]::UtcNow.ToString('o');source_commit=$manifest.commit;source_sha256=(Hash $source);source_bytes=(Get-Item $source).Length;net_sha256=$netHash;net_fingerprint=$netJson;net_fingerprint_sha256=(Hash $netJson);selected_dependencies=$selected;tools=(Tools);old_fixture_counts=@($oldInputs|ForEach-Object {[ordered]@{label=$_.label;files=$_.files.Count}});session=(Get-Process -Id $PID).SessionId;priority=(Get-Process -Id $PID).PriorityClass.ToString();cargo_build_jobs=$env:CARGO_BUILD_JOBS;compiler_invocations_at_once=1;arguments=$args;acl=(Assert-ACL)}
Save-Json $pre (Join-Path $root 'build-start.json')
$watch=[Diagnostics.Stopwatch]::StartNew();$started=[DateTime]::UtcNow.ToString('o')
& (Join-Path $toolRoot 'rustc.exe') @args *> (Join-Path $root 'logs/rustc.log')
$code=$LASTEXITCODE;$watch.Stop()
$rust=[ordered]@{started_utc=$started;ended_utc=[DateTime]::UtcNow.ToString('o');elapsed_ms=$watch.ElapsedMilliseconds;exit=$code;arguments=$args;source_sha256=(Hash $source);compiler_sha256=(Hash (Join-Path $toolRoot 'rustc.exe'));log_sha256=(Hash (Join-Path $root 'logs/rustc.log'))}
Save-Json $rust (Join-Path $root 'rustc-result.json');if($code -ne 0){exit $code}
$observerSource=Join-Path $root 'native-observer.cs';$observerDll=Join-Path $root 'bin/OwnedNativeObserver.dll'
$watch=[Diagnostics.Stopwatch]::StartNew();$started=[DateTime]::UtcNow.ToString('o')
Add-Type -Path $observerSource -OutputAssembly $observerDll -CompilerOptions @('/warnaserror+','/optimize+')
$watch.Stop()
$csharp=[ordered]@{started_utc=$started;ended_utc=[DateTime]::UtcNow.ToString('o');elapsed_ms=$watch.ElapsedMilliseconds;exit=0;source_sha256=(Hash $observerSource);dll_sha256=(Hash $observerDll);compiler_options=@('/warnaserror+','/optimize+');runtime=[Runtime.InteropServices.RuntimeInformation]::FrameworkDescription;roslyn=@(Get-ChildItem $PSHOME -File -Filter 'Microsoft.CodeAnalysis*.dll'|ForEach-Object {[ordered]@{name=$_.Name;sha256=(Hash $_.FullName)}})}
Save-Json $csharp (Join-Path $root 'csharp-result.json')
foreach($f in @(Get-ChildItem (Join-Path $root 'bin') -File)){$f.IsReadOnly=$true}
Save-Json @((Get-Item $source),(Get-Item $observerSource)) (Join-Path $root 'source-metadata-private.json')
Save-Json @((Get-ChildItem (Join-Path $root 'bin') -File)|ForEach-Object {File-Record $_}) (Join-Path $root 'frozen-binaries.json')
Write-Output 'OWNED_HELPER_BUILD_PASS'
