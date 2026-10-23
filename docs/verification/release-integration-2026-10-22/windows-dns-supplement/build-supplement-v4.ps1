param([Parameter(Mandatory=$true)][string]$FrozenPath)
$ErrorActionPreference='Stop';$ProgressPreference='SilentlyContinue';$PSNativeCommandUseErrorActionPreference=$false
$root=$PSScriptRoot;(Get-Process -Id $PID).PriorityClass='BelowNormal'
$env:TEMP=Join-Path $root 'temp';$env:TMP=$env:TEMP;$env:CARGO_BUILD_JOBS='2';$env:RUSTUP_AUTO_INSTALL='0';$env:RUSTUP_TOOLCHAIN='1.99.1-x86_64-pc-windows-msvc'
$env:OPENERGY_ALLOW_PORT_24800='0';$env:OPENERGY_DISABLE_DISCOVERY='1'
$source=Join-Path $root 'source/windows_dns_supplement_v3.rs'
if((Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash -ne '009193b58d548fd5b2453599db81e7167ff35aee9f623aef1995c2fe70c92675'){throw 'helper source mismatch'}
(Get-Item -LiteralPath $source).IsReadOnly=$true
$release=Join-Path $FrozenPath 'target/x86_64-pc-windows-msvc/release'
$libs=Get-Content -Raw -LiteralPath (Join-Path $FrozenPath 'library-artifacts.json')|ConvertFrom-Json
foreach($entry in $libs.libraries){if((Get-FileHash -LiteralPath (Join-Path $FrozenPath $entry.path) -Algorithm SHA256).Hash -ne $entry.sha256){throw 'frozen library mismatch'}}
$tool=Join-Path $env:USERPROFILE '.rustup/toolchains/1.99.1-x86_64-pc-windows-msvc/bin/rustc.exe'
$out=Join-Path $root 'bin/windows_dns_supplement.exe'
$arguments=@('--edition=2021','--crate-name','windows_dns_supplement','--target','x86_64-pc-windows-msvc','-D','warnings','-C','opt-level=3','-C','panic=abort','-C','codegen-units=1','-C','lto=thin','-C','strip=symbols','--extern',('openergy_net='+(Join-Path $release 'libopenergy_net.rlib')),'--extern',('openergy_session='+(Join-Path $release 'libopenergy_session.rlib')),'-L',('dependency='+(Join-Path $release 'deps')))
$arguments+=@('-L',('dependency='+(Join-Path $FrozenPath 'target/release/deps')))
$nativeDirs=@(Get-ChildItem -LiteralPath (Join-Path $release 'build') -Directory|ForEach-Object {Join-Path $_.FullName 'out'}|Where-Object {Test-Path -LiteralPath $_})
foreach($dir in $nativeDirs){$arguments+=@('-L',('native='+$dir))}
$arguments+=@($source,'-o',$out)
$watch=[Diagnostics.Stopwatch]::StartNew();$started=[DateTime]::UtcNow.ToString('o')
& $tool @arguments *> (Join-Path $root 'logs/compile.log')
$exit=$LASTEXITCODE;$watch.Stop()
$result=[ordered]@{started_utc=$started;ended_utc=[DateTime]::UtcNow.ToString('o');exit=$exit;elapsed_ms=$watch.ElapsedMilliseconds;session=(Get-Process -Id $PID).SessionId;priority=(Get-Process -Id $PID).PriorityClass.ToString();cargo_build_jobs=$env:CARGO_BUILD_JOBS;concurrent_rustc_invocations=1;tool_sha256=(Get-FileHash -LiteralPath $tool -Algorithm SHA256).Hash.ToLower();source_sha256=(Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLower();arguments=$arguments;library_inputs=$libs.libraries}
if($exit -eq 0){$file=Get-Item -LiteralPath $out;$result.binary=[ordered]@{bytes=$file.Length;sha256=(Get-FileHash -LiteralPath $out -Algorithm SHA256).Hash.ToLower()};$file.IsReadOnly=$true}
$result|ConvertTo-Json -Depth 10|Set-Content -LiteralPath (Join-Path $root 'compile-result.json')
$result|ConvertTo-Json -Depth 10
exit $exit
