$ErrorActionPreference='Stop';$ProgressPreference='SilentlyContinue'
$root=$PSScriptRoot
(Get-Process -Id $PID).PriorityClass='BelowNormal'
if((Get-Process -Id $PID).SessionId -ne 0){throw 'noninteractive session 0 required'}
if(Test-Path -LiteralPath (Join-Path $root 'build-once.guard')){throw 'build already attempted; no retry'}
[IO.File]::WriteAllText((Join-Path $root 'build-once.guard'),[DateTime]::UtcNow.ToString('o'))
$toolRoot=Join-Path $env:USERPROFILE '.rustup/toolchains/1.99.1-x86_64-pc-windows-msvc/bin'
$registry=Join-Path $env:USERPROFILE '.cargo/registry'
function Hash($p){(Get-FileHash -LiteralPath $p -Algorithm SHA256).Hash.ToLower()}
function Save($p,$v){[IO.File]::WriteAllText($p,($v|ConvertTo-Json -Depth 24),[Text.UTF8Encoding]::new($false))}
function Digest($text){$h=[Security.Cryptography.SHA256]::Create();try{([BitConverter]::ToString($h.ComputeHash([Text.Encoding]::UTF8.GetBytes($text)))).Replace('-','').ToLower()}finally{$h.Dispose()}}
function Registry-Snapshot($path){
 $items=@(Get-ChildItem -LiteralPath $path -Recurse -Force)
 if(@($items|Where-Object {$_.Attributes -band [IO.FileAttributes]::ReparsePoint}).Count){throw 'registry reparse point refused'}
 $files=@($items|Where-Object {-not $_.PSIsContainer}|Sort-Object FullName|ForEach-Object {[ordered]@{path=$_.FullName.Substring($path.Length+1).Replace('\','/');bytes=$_.Length;sha256=(Hash $_.FullName)}})
 [ordered]@{files=$files.Count;bytes=($files|Measure-Object bytes -Sum).Sum;manifest_sha256=(Digest ($files|ConvertTo-Json -Depth 8 -Compress))}
}
function Defaults {
 $names=@('.rustup/settings.toml','.cargo/config','.cargo/config.toml','.cargo/.global-cache','.cargo/.global-cache-wal','.cargo/.global-cache-shm','.cargo/.package-cache','.cargo/.package-cache-mutate')
 @(foreach($n in $names){$p=Join-Path $env:USERPROFILE $n;[ordered]@{path=$n;exists=(Test-Path -LiteralPath $p -PathType Leaf);sha256=$(if(Test-Path -LiteralPath $p -PathType Leaf){Hash $p}else{$null})}})
}
function Tools {
 $paths=@(foreach($n in @('rustc.exe','cargo.exe','cargo-clippy.exe','clippy-driver.exe')){Join-Path $toolRoot $n})
 $msvc='C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64'
 $paths+=@(foreach($n in @('cl.exe','link.exe','lib.exe','c1.dll','c2.dll')){Join-Path $msvc $n})
 @(foreach($p in $paths){$f=Get-Item -LiteralPath $p;[ordered]@{name=$f.Name;bytes=$f.Length;sha256=(Hash $p)}})
}
$active=@(Get-Process -Name cargo,rustc,cargo-clippy,clippy-driver -ErrorAction SilentlyContinue)
if($active.Count){throw 'another native build is active'}
$manifest=Get-Content -Raw -LiteralPath (Join-Path $root 'source-manifest.json')|ConvertFrom-Json
if($manifest.source_commit -ne '06b9a139ec855a6f697ded393fa4c2c894110613' -or (Hash (Join-Path $root 'source.tar')) -ne $manifest.archive_sha256){throw 'source archive binding mismatch'}
$source=Join-Path $root 'source';New-Item -ItemType Directory -Path $source|Out-Null
& (Join-Path $env:WINDIR 'System32/tar.exe') -xf (Join-Path $root 'source.tar') -C $source
if($LASTEXITCODE -ne 0){throw 'source extraction failed'}
function Assert-Source {
 $files=@(Get-ChildItem -LiteralPath $source -File -Recurse -Force)
 if($files.Count -ne $manifest.files.Count){throw 'source file count mismatch'}
 foreach($entry in $manifest.files){$p=Join-Path $source $entry.path;$f=Get-Item -LiteralPath $p;if($f.Length -ne $entry.bytes -or (Hash $p) -ne $entry.sha256){throw ('source mismatch '+$entry.path)}}
 [ordered]@{files=$files.Count;all_hashes_match=$true;archive_sha256=(Hash (Join-Path $root 'source.tar'))}
}
$sourceBefore=Assert-Source
Get-ChildItem -LiteralPath $source -File -Recurse -Force|ForEach-Object {$_.IsReadOnly=$true}
$before=[ordered]@{registry=(Registry-Snapshot $registry);defaults=(Defaults);tools=(Tools)}
Save (Join-Path $root 'shared-inputs-before.json') $before
$cargoHome=Join-Path $root 'cargo-home';New-Item -ItemType Directory -Path $cargoHome|Out-Null
Copy-Item -LiteralPath $registry -Destination (Join-Path $cargoHome 'registry') -Recurse -Force
$copy=Registry-Snapshot (Join-Path $cargoHome 'registry')
if(($copy|ConvertTo-Json -Compress) -ne ($before.registry|ConvertTo-Json -Compress)){throw 'private cache byte binding mismatch'}
Get-ChildItem -LiteralPath (Join-Path $cargoHome 'registry/src') -File -Recurse -Force|ForEach-Object {$_.IsReadOnly=$true}
$env:CARGO_HOME=$cargoHome;$env:CARGO_TARGET_DIR=Join-Path $root 'target';$env:CARGO_BUILD_JOBS='2';$env:CARGO_NET_OFFLINE='true'
$env:RUSTUP_AUTO_INSTALL='0';$env:RUSTUP_TOOLCHAIN='1.99.1-x86_64-pc-windows-msvc'
$env:RUSTC=Join-Path $toolRoot 'rustc.exe';$env:RUSTDOC=Join-Path $toolRoot 'rustdoc.exe'
$env:RUSTC_WRAPPER='';$env:RUSTC_WORKSPACE_WRAPPER='';$env:PATH=$toolRoot+';'+$env:PATH
$env:TEMP=Join-Path $root 'temp';$env:TMP=$env:TEMP;New-Item -ItemType Directory -Path $env:TEMP|Out-Null
$arguments=@('build','--offline','--locked','--release','--target','x86_64-pc-windows-msvc','-j','2','-p','openergy-net','--example','dns_queue_cancel_c4806a3d')
$record=[ordered]@{source_commit=$manifest.source_commit;started_utc=[DateTime]::UtcNow.ToString('o');command=@((Join-Path $toolRoot 'cargo.exe'))+$arguments;session=(Get-Process -Id $PID).SessionId;priority=(Get-Process -Id $PID).PriorityClass.ToString();jobs=2;source_before=$sourceBefore;private_cache_copy=$copy;original_cache_never_selected=$true;rustc=@(& $env:RUSTC -vV);cargo=@(& (Join-Path $toolRoot 'cargo.exe') -V);runtime_attempts=0;binary_executed=$false;unit_tests=$false;mocks=$false;timeout_seconds=600}
Save (Join-Path $root 'build-start.json') $record
$info=[Diagnostics.ProcessStartInfo]::new((Join-Path $toolRoot 'cargo.exe'));$info.UseShellExecute=$false;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true;$info.WorkingDirectory=$source
foreach($arg in $arguments){$info.ArgumentList.Add($arg)}
$watch=[Diagnostics.Stopwatch]::StartNew();$child=[Diagnostics.Process]::Start($info)
$out=$child.StandardOutput.ReadToEndAsync();$err=$child.StandardError.ReadToEndAsync()
$record.child_priority=$child.PriorityClass.ToString();$record.child_session=$child.SessionId
$record.timed_out=-not $child.WaitForExit(600000)
if($record.timed_out){$child.Kill($true);$child.WaitForExit()}
$record.exit_code=$child.ExitCode;$record.elapsed_ms=$watch.ElapsedMilliseconds;$record.ended_utc=[DateTime]::UtcNow.ToString('o')
[IO.File]::WriteAllText((Join-Path $root 'build.stdout'),$out.Result,[Text.UTF8Encoding]::new($false))
[IO.File]::WriteAllText((Join-Path $root 'build.stderr'),$err.Result,[Text.UTF8Encoding]::new($false))
$after=[ordered]@{registry=(Registry-Snapshot $registry);defaults=(Defaults);tools=(Tools)}
Save (Join-Path $root 'shared-inputs-after.json') $after
$record.shared_inputs_unchanged=($before|ConvertTo-Json -Depth 16 -Compress) -eq ($after|ConvertTo-Json -Depth 16 -Compress)
$record.source_after=Assert-Source
$record.stdout_sha256=Hash (Join-Path $root 'build.stdout');$record.stderr_sha256=Hash (Join-Path $root 'build.stderr')
$binary=Join-Path $root 'target/x86_64-pc-windows-msvc/release/examples/dns_queue_cancel_c4806a3d.exe'
if($record.exit_code -eq 0){$f=Get-Item -LiteralPath $binary;$record.binary=[ordered]@{bytes=$f.Length;sha256=(Hash $binary)}}
$record.passed=($record.exit_code -eq 0 -and -not $record.timed_out -and $record.shared_inputs_unchanged)
Save (Join-Path $root 'build-result.json') $record
if(-not $record.passed){exit 1}
Write-Output 'NATIVE_WINDOWS_COMPILE_PASS_NO_RUNTIME'
