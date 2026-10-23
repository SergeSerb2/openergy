param([Parameter(Mandatory=$true)][string]$FrozenPath)
$ErrorActionPreference='Stop';$ProgressPreference='SilentlyContinue';$PSNativeCommandUseErrorActionPreference=$false
$root=$PSScriptRoot;(Get-Process -Id $PID).PriorityClass='BelowNormal'
if((Get-Process -Id $PID).SessionId -ne 0){throw 'session 0 required'}
$env:TEMP=Join-Path $root 'temp';$env:TMP=$env:TEMP;$env:CARGO_BUILD_JOBS='2';$env:RUSTUP_AUTO_INSTALL='0';$env:RUSTUP_TOOLCHAIN='1.99.1-x86_64-pc-windows-msvc'
$env:OPENERGY_ALLOW_PORT_24800='0';$env:OPENERGY_DISABLE_DISCOVERY='1'
function Hash($p){(Get-FileHash -LiteralPath $p -Algorithm SHA256).Hash.ToLower()}
function Save($v,$n){$v|ConvertTo-Json -Depth 24|Set-Content -LiteralPath (Join-Path $root $n) -Encoding utf8}
if(Test-Path (Join-Path $root 'compile-start.json')){throw 'compile already attempted'}
$active=@(Get-CimInstance Win32_Process -ErrorAction Stop|Where-Object {$_.Name -in @('cargo.exe','rustc.exe','cargo-clippy.exe','clippy-driver.exe')})
if($active.Count -ne 0){throw 'another build is active'}
$all=@(Get-ChildItem -LiteralPath $FrozenPath -Recurse -File -Force)
$original=@(foreach($f in $all){[ordered]@{path=$f.FullName.Substring($FrozenPath.Length+1).Replace('\','/');bytes=$f.Length;readonly=$f.IsReadOnly;sha256=(Hash $f.FullName)}})
Save $original 'frozen-all-files-before-v2.json'
$manifest=Get-Content -Raw -LiteralPath (Join-Path $FrozenPath 'source-manifest.json')|ConvertFrom-Json
foreach($e in $manifest.files){$f=Get-Item (Join-Path $FrozenPath ('source/'+$e.path));if($f.Length -ne $e.bytes -or -not $f.IsReadOnly -or (Hash $f.FullName) -ne $e.sha256){throw 'old frozen source changed'}}
$artifact=Get-Content -Raw -LiteralPath (Join-Path $FrozenPath 'artifact-manifest.json')|ConvertFrom-Json
foreach($e in $artifact){$f=Get-Item (Join-Path $FrozenPath $e.path);if($f.Length -ne $e.bytes -or -not $f.IsReadOnly -or (Hash $f.FullName) -ne $e.sha256){throw 'old artifact changed'}}
$toolRoot=Join-Path $env:USERPROFILE '.rustup/toolchains/1.99.1-x86_64-pc-windows-msvc/bin'
$tool=Join-Path $toolRoot 'rustc.exe'
$tools=@(foreach($n in @('rustc.exe','cargo.exe','cargo-clippy.exe','clippy-driver.exe')){$p=Join-Path $toolRoot $n;[ordered]@{name=$n;sha256=(Hash $p)}})
$globalSettings=Join-Path $env:USERPROFILE '.rustup/settings.toml'
$source=Join-Path $root 'startup_diagnostic.rs';(Get-Item $source).IsReadOnly=$true
$release=Join-Path $FrozenPath 'target/x86_64-pc-windows-msvc/release'
# Select the direct mdns/log dependencies of the frozen discovery_recovery example.
$exampleDirs=@(Get-ChildItem (Join-Path $release '.fingerprint') -Directory -Filter 'openergy-net-*'|Where-Object {(Test-Path (Join-Path $_.FullName 'example-discovery_recovery.json')) -and (Test-Path (Join-Path $release ('examples/discovery_recovery-'+$_.Name.Substring('openergy-net-'.Length)+'.d')))})
if($exampleDirs.Count -ne 1){throw 'expected one frozen discovery_recovery fingerprint'}
$example=Get-Content -Raw (Join-Path $exampleDirs[0].FullName 'example-discovery_recovery.json')|ConvertFrom-Json
$selected=@{}
foreach($crate in @('mdns_sd','log')){
 $dep=@($example.deps|Where-Object {$_[1] -eq $crate});if($dep.Count -ne 1){throw ('fingerprint dependency missing: '+$crate)}
 $wanted=[UInt64]$dep[0][3];$package=$crate.Replace('_','-')
 $matches=@(foreach($dir in @(Get-ChildItem (Join-Path $release '.fingerprint') -Directory -Filter ($package+'-*'))){$finger=Join-Path $dir.FullName ('lib-'+$crate);if(Test-Path $finger){$hex=(Get-Content -Raw $finger).Trim();if($hex -match '^[0-9a-f]{16}$'){$bytes=@(for($i=0;$i -lt 16;$i+=2){[Convert]::ToByte($hex.Substring($i,2),16)});$value=[BitConverter]::ToUInt64([byte[]]$bytes,0);if($value -eq $wanted){$candidate=Join-Path $release ('deps/lib'+$crate+'-'+($dir.Name.Substring($package.Length+1))+'.rlib');if(Test-Path $candidate){[ordered]@{crate=$crate;fingerprint=$finger;fingerprint_hex=$hex;dependency_value=$wanted;path=$candidate;sha256=(Hash $candidate)}}}}}})
 if($matches.Count -ne 1){throw ('cannot select exact frozen dependency: '+$crate)}
 $selected[$crate]=$matches[0]
}
$arguments=@('--edition=2021','--crate-name','startup_diagnostic','--target','x86_64-pc-windows-msvc','-D','warnings','-C','opt-level=3','-C','panic=abort','-C','codegen-units=1','-C','lto=thin','-C','strip=symbols','--extern',('openergy_net='+(Join-Path $release 'libopenergy_net.rlib')),'--extern',('mdns_sd='+$selected.mdns_sd.path),'--extern',('log='+$selected.log.path),'-L',('dependency='+(Join-Path $release 'deps')),'-L',('dependency='+(Join-Path $FrozenPath 'target/release/deps')))
foreach($dir in @(Get-ChildItem (Join-Path $release 'build') -Directory|ForEach-Object {Join-Path $_.FullName 'out'}|Where-Object {Test-Path $_})){$arguments+=@('-L',('native='+$dir))}
$out=Join-Path $root 'bin/startup_diagnostic.exe';$arguments+=@($source,'-o',$out)
$pre=[ordered]@{recorded_utc=[DateTime]::UtcNow.ToString('o');old_file_count=$all.Count;old_source_files=$manifest.files.Count;old_artifact_files=$artifact.Count;old_source_all_hashes_readonly_match=$true;old_artifacts_all_hashes_readonly_match=$true;old_fixture_access='read only; attributes and evidence hashes recorded, not changed';tools=$tools;rustup_settings_sha256=(Hash $globalSettings);toolchains=@(& rustup.exe toolchain list);selected_dependencies=$selected;example_fingerprint_sha256=(Hash (Join-Path $exampleDirs[0].FullName 'example-discovery_recovery.json'));source_sha256=(Hash $source);source_bytes=(Get-Item $source).Length;session=(Get-Process -Id $PID).SessionId;priority=(Get-Process -Id $PID).PriorityClass.ToString();jobs=2;native_compiler_invocations=1;arguments=$arguments}
Save $pre 'compile-start.json'
$watch=[Diagnostics.Stopwatch]::StartNew();$started=[DateTime]::UtcNow.ToString('o')
& $tool @arguments *> (Join-Path $root 'logs/compile.log')
$code=$LASTEXITCODE;$watch.Stop()
$result=[ordered]@{started_utc=$started;ended_utc=[DateTime]::UtcNow.ToString('o');exit=$code;elapsed_ms=$watch.ElapsedMilliseconds;source_sha256=$pre.source_sha256;compiler_sha256=(Hash $tool);arguments=$arguments;log_sha256=(Hash (Join-Path $root 'logs/compile.log'))}
if($code -eq 0){$f=Get-Item $out;$f.IsReadOnly=$true;$result.binary=[ordered]@{path=$out;bytes=$f.Length;sha256=(Hash $out);readonly=$true};foreach($f in @(Get-ChildItem (Join-Path $root 'bin') -File)){$f.IsReadOnly=$true}}
Save $result 'compile-result.json'
Write-Output ('COMPILE_EXIT='+$code)
exit $code
