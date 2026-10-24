. (Join-Path $PSScriptRoot 'runtime-common.ps1')
if(Test-Path (Join-Path $root 'build-start.json')){throw 'build already attempted; inspect durable state'}
$state=Get-Content -Raw (Join-Path $root 'state-input.json')|ConvertFrom-Json
$cache=Get-Content -Raw (Join-Path $root 'cache-discovery.json')|ConvertFrom-Json
$preInventory=Inventory 'preflight'
if($preInventory.build_processes.Count -ne 0 -or $preInventory.assigned_tcp_listeners.Count -ne 0 -or $preInventory.assigned_udp_endpoints.Count -ne 0){throw 'another build or assigned endpoint is active'}
if($cache.reparse_points -ne 0 -or $cache.registry_files -eq 0){throw 'offline cache prerequisite unavailable'}
$compilerRoot=Split-Path $toolRoot -Parent
$immutable=@(foreach($f in $state.prior_fixtures){Snapshot-Tree $f.label $f.root})
$immutable+=@(Snapshot-Tree 'original-offline-registry' $cache.registry)
$immutable+=@(Snapshot-Tree 'pinned-rust-1.99.1-toolchain' $compilerRoot)
Save-Json $immutable (Join-Path $root 'immutable-inputs-before.json')
$globalFiles=@(foreach($p in @((Join-Path $env:USERPROFILE '.rustup/settings.toml'),(Join-Path $cache.original_cargo_home '.global-cache'),(Join-Path $cache.original_cargo_home '.global-cache-wal'),(Join-Path $cache.original_cargo_home '.global-cache-shm'),(Join-Path $cache.original_cargo_home '.package-cache'),(Join-Path $cache.original_cargo_home '.package-cache-mutate'),(Join-Path $cache.original_cargo_home 'config'),(Join-Path $cache.original_cargo_home 'config.toml'))){
 if(Test-Path $p){$f=Get-Item -LiteralPath $p;[ordered]@{path=$p;exists=$true;bytes=$f.Length;readonly=$f.IsReadOnly;sha256=(Hash $p);acl_sha256=(String-Hash (Get-Acl -LiteralPath $p).Sddl)}}else{[ordered]@{path=$p;exists=$false}}
})
Save-Json $globalFiles (Join-Path $root 'original-global-files-before.json')
$nativeReference=Get-Content -Raw (Join-Path $root 'native-tool-reference.json')|ConvertFrom-Json
$nativeBefore=@(foreach($e in $nativeReference.native_tools){$f=Get-Item -LiteralPath $e.path;if($f.Length -ne $e.bytes -or (Hash $f.FullName) -ne $e.sha256){throw 'native compiler/tool reference mismatch before build'};[ordered]@{name=$e.name;path=$e.path;bytes=$f.Length;sha256=(Hash $f.FullName);file_version=$f.VersionInfo.FileVersion}})
Save-Json ([ordered]@{recorded_utc=[DateTime]::UtcNow.ToString('o');native_tools=$nativeBefore;fresh_prebuild_sample=$true}) (Join-Path $root 'native-tools-before.json')
$nativeMetadata=@(foreach($e in $nativeReference.native_tools){$f=Get-Item -LiteralPath $e.path;[ordered]@{path=$f.FullName;readonly=$f.IsReadOnly;acl_sha256=(String-Hash (Get-Acl -LiteralPath $f.FullName).Sddl)}})
Save-Json $nativeMetadata (Join-Path $root 'native-metadata-before.json')
$preTools=Tools
$privateCargo=Join-Path $root 'cargo-home';New-Item -ItemType Directory -Path $privateCargo|Out-Null
Copy-Item -LiteralPath $cache.registry -Destination (Join-Path $privateCargo 'registry') -Recurse -Force
$originalRegistry=$immutable|Where-Object {$_.label -eq 'original-offline-registry'}
$cacheCopyCount=0
foreach($e in $originalRegistry.entries|Where-Object {-not $_.directory}){$f=Get-Item (Join-Path (Join-Path $privateCargo 'registry') $e.path);if($f.Length -ne $e.bytes -or (Hash $f.FullName) -ne $e.sha256){throw 'private offline cache copy mismatch'};$cacheCopyCount++}
$registrySource=Join-Path $privateCargo 'registry/src'
Get-ChildItem $registrySource -File -Recurse -Force|ForEach-Object {$_.IsReadOnly=$true}
Save-Json @((Get-ChildItem $registrySource -File -Recurse -Force)|ForEach-Object {File-Record $_}) (Join-Path $root 'private-registry-source-before.json')
$manifest=Get-Content -Raw (Join-Path $root 'source-manifest.json')|ConvertFrom-Json
if($manifest.commit -ne '615c16e1df71bc0b350aeee4f1f34bceba5a59e0' -or (Hash (Join-Path $root 'source.tar')) -ne $manifest.archive_sha256){throw 'candidate archive mismatch'}
& tar.exe -xf (Join-Path $root 'source.tar') -C (Join-Path $root 'source');if($LASTEXITCODE -ne 0){throw 'archive extraction failed'}
Get-ChildItem (Join-Path $root 'source') -File -Recurse -Force|ForEach-Object {$_.IsReadOnly=$true}
$sourceBefore=Assert-Source
$env:CARGO_HOME=$privateCargo;$env:CARGO_TARGET_DIR=Join-Path $root 'target'
$env:RUSTUP_TOOLCHAIN='1.99.1-x86_64-pc-windows-msvc';$env:PATH=$toolRoot+';'+$env:PATH
$env:RUSTC=Join-Path $toolRoot 'rustc.exe';$env:RUSTDOC=Join-Path $toolRoot 'rustdoc.exe'
$env:RUSTC_WRAPPER='';$env:RUSTC_WORKSPACE_WRAPPER=''
Set-Location -LiteralPath (Join-Path $root 'source')
$base=@('--locked','--offline','--release','--target','x86_64-pc-windows-msvc','--jobs','2','-p','openergy-net','--lib','--example','dns_queue_cancel_c4806a3d')
$steps=@(@{name='clippy';args=(@('clippy')+$base+@('--','-D','warnings'))},@{name='release';args=(@('build')+$base)})
$result=[ordered]@{source_commit=$manifest.commit;started_utc=[DateTime]::UtcNow.ToString('o');source_before=$sourceBefore;tools_before=$preTools;inventory_before=$preInventory;private_cargo_home=$privateCargo;private_cache_copy_files=$cacheCopyCount;original_cargo_home_never_selected=$true;original_cache_writes_allowed=$false;private_source_readonly=$true;cargo_target=$env:CARGO_TARGET_DIR;pinned_rustc=$env:RUSTC;session=(Get-Process -Id $PID).SessionId;priority=(Get-Process -Id $PID).PriorityClass.ToString();jobs=2;steps=@();passed=$false}
Save-Json $result (Join-Path $root 'build-start.json')
foreach($step in $steps){
 $watch=[Diagnostics.Stopwatch]::StartNew();$started=[DateTime]::UtcNow.ToString('o');$arguments=$step.args
 Save-Json ([ordered]@{step=$step.name;started_utc=$started;arguments=$arguments}) (Join-Path $root 'build-progress.json')
 $info=[Diagnostics.ProcessStartInfo]::new((Join-Path $toolRoot 'cargo.exe'));$info.UseShellExecute=$false;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true;$info.WorkingDirectory=Join-Path $root 'source'
 foreach($arg in $arguments){$info.ArgumentList.Add($arg)}
 $child=[Diagnostics.Process]::Start($info);$childStart=$child.StartTime;$out=$child.StandardOutput.ReadToEndAsync();$err=$child.StandardError.ReadToEndAsync();$samples=@();$timedOut=$false
 $samples+=@([ordered]@{name='cargo.exe';pid=$child.Id;parent_pid=$PID;path=$child.Path;created=$child.StartTime.ToUniversalTime().ToString('o');session=$child.SessionId;priority=$child.PriorityClass.ToString();at_ms=$watch.ElapsedMilliseconds})
 $seen=@{}
 while(-not $child.WaitForExit(250)){
  foreach($c in @(Get-CimInstance Win32_Process -ErrorAction Stop|Where-Object {$_.ExecutablePath -and $_.ExecutablePath.StartsWith($toolRoot+'\',[StringComparison]::OrdinalIgnoreCase) -and $_.Name -in @('cargo.exe','cargo-clippy.exe','rustc.exe','clippy-driver.exe') -and $_.CreationDate.ToUniversalTime() -ge $childStart.ToUniversalTime()})){
   $key=[string]$c.ProcessId+':'+$c.CreationDate.ToString('o')
   if(-not $seen.ContainsKey($key)){$actual=Get-Process -Id $c.ProcessId -ErrorAction SilentlyContinue;if($actual){$samples+=@([ordered]@{name=$c.Name;pid=$c.ProcessId;parent_pid=$c.ParentProcessId;path=$c.ExecutablePath;created=$c.CreationDate.ToUniversalTime().ToString('o');session=$c.SessionId;priority=$actual.PriorityClass.ToString();at_ms=$watch.ElapsedMilliseconds});$seen[$key]=$true}}
  }
  if($watch.Elapsed.TotalSeconds -gt 600){$timedOut=$true;$owned=Get-Process -Id $child.Id -ErrorAction Stop;if($owned.Path -ne (Join-Path $toolRoot 'cargo.exe') -or $owned.StartTime -ne $childStart){throw 'build watchdog ownership mismatch'};$owned.Kill($true);break}
 }
 if(-not $child.WaitForExit(5000)){throw 'build child did not exit'}
 $exit=$child.ExitCode;$watch.Stop()
 [IO.File]::WriteAllText((Join-Path $root ('logs/'+$step.name+'.stdout')),$out.Result,[Text.UTF8Encoding]::new($false));[IO.File]::WriteAllText((Join-Path $root ('logs/'+$step.name+'.stderr')),$err.Result,[Text.UTF8Encoding]::new($false))
 [IO.File]::WriteAllText((Join-Path $root ('logs/'+$step.name+'.log')),$out.Result+$err.Result,[Text.UTF8Encoding]::new($false))
 Save-Json ([ordered]@{step=$step.name;timed_out=$timedOut;samples=$samples;stdout_sha256=(Hash (Join-Path $root ('logs/'+$step.name+'.stdout')));stderr_sha256=(Hash (Join-Path $root ('logs/'+$step.name+'.stderr')))}) (Join-Path $root ($step.name+'-child-observation.json'))
 if($timedOut){throw 'build watchdog exceeded; no retry'}
 $result.steps+=@([ordered]@{name=$step.name;arguments=$arguments;started_utc=$started;ended_utc=[DateTime]::UtcNow.ToString('o');elapsed_ms=$watch.ElapsedMilliseconds;exit=$exit;log_sha256=(Hash (Join-Path $root ('logs/'+$step.name+'.log')))})
 Save-Json $result (Join-Path $root 'build-result.json');if($exit -ne 0){exit $exit}
}
$result.source_after=Assert-Source;$result.passed=$true;$result.ended_utc=[DateTime]::UtcNow.ToString('o')
Save-Json $result (Join-Path $root 'build-result.json')
$release=Join-Path $root 'target/x86_64-pc-windows-msvc/release'
Copy-Item (Join-Path $release 'examples/dns_queue_cancel_c4806a3d.exe') (Join-Path $root 'bin/dns_queue_cancel_c4806a3d.exe')
New-Item -ItemType Directory -Path (Join-Path $root 'bin/libraries')|Out-Null
$libs=@(Get-ChildItem $release -File -Filter 'libopenergy*.rlib')+@(Get-ChildItem (Join-Path $release 'deps') -File -Filter 'libmdns_sd-*.rlib')
foreach($f in $libs){Copy-Item $f.FullName (Join-Path $root ('bin/libraries/'+$f.Name))}
$artifacts=@(Get-ChildItem (Join-Path $root 'target') -File -Recurse -Force)+@(Get-ChildItem (Join-Path $root 'bin') -File -Recurse -Force)
foreach($f in $artifacts){$f.IsReadOnly=$true}
Save-Json @($artifacts|ForEach-Object {File-Record $_}) (Join-Path $root 'artifact-manifest.json')
Save-Json ([ordered]@{recorded_utc=[DateTime]::UtcNow.ToString('o');binary=(File-Record (Get-Item (Join-Path $root 'bin/dns_queue_cancel_c4806a3d.exe')));libraries=@(Get-ChildItem (Join-Path $root 'bin/libraries') -File|ForEach-Object {File-Record $_});artifact_files=$artifacts.Count;artifact_manifest_sha256=(Hash (Join-Path $root 'artifact-manifest.json'));acl=(Assert-ACL);source=(Assert-Source)}) (Join-Path $root 'freeze.json')
Write-Output 'EXACT_HISTORICAL_OVERLAY_NATIVE_BUILD_AND_FREEZE_PASS'
