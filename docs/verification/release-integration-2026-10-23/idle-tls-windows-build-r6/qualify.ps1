. (Join-Path $PSScriptRoot 'common.ps1')
if(Test-Path -LiteralPath (Join-Path $root 'qualification-once.guard')){throw 'qualification already attempted'}
[IO.File]::WriteAllText((Join-Path $root 'qualification-once.guard'),[DateTime]::UtcNow.ToString('o'))
$inputs=Get-Content -Raw -LiteralPath (Join-Path $root 'transfer-manifest.json')|ConvertFrom-Json
$transferred=@(foreach($e in $inputs.files){$f=Get-Item -LiteralPath (Join-Path $root $e.path);$h=Hash $f.FullName;if($f.Length -ne $e.bytes -or $h -ne $e.sha256){throw ('transfer size/hash mismatch: '+$e.path)};[ordered]@{path=$e.path;bytes=$f.Length;sha256=$h}})
Save 'transfer-confirmed.json' ([ordered]@{files=$transferred;all_match=$true;scp_completed_before_control=$true;control_stdin_payload=$false;recorded_utc=[DateTime]::UtcNow.ToString('o')})
foreach($n in @('common.ps1','qualify.ps1','build.ps1','postflight.ps1')){$tokens=$null;$errors=$null;$null=[Management.Automation.Language.Parser]::ParseFile((Join-Path $root $n),[ref]$tokens,[ref]$errors);if($errors.Count){throw ($errors|Out-String)}}
if(@(Get-Process -Name cargo,rustc,cargo-clippy,clippy-driver -ErrorAction SilentlyContinue).Count){throw 'another build is active'}
$manifest=Get-Content -Raw -LiteralPath (Join-Path $root 'source-manifest.json')|ConvertFrom-Json
if($manifest.schema -ne 'openergy.windows-build-source.v1' -or $manifest.source_commit -ne '06b9a139ec855a6f697ded393fa4c2c894110613' -or $manifest.module_sha256 -ne '9cd3475f9c13c8e28ea54e0f344b55ae795267b22ea1676c45e5659a64f1f1d4'){throw 'source schema/commit/module mismatch'}
if($manifest.files.Count -ne 558){throw 'source must contain exactly 558 files'}
$archive=Get-Item -LiteralPath (Join-Path $root 'source.tar')
if($archive.Length -ne $manifest.archive_bytes -or (Hash $archive.FullName) -ne $manifest.archive_sha256){throw 'archive binding mismatch'}
New-Item -ItemType Directory -Path (Join-Path $root 'source')|Out-Null
& (Join-Path $env:WINDIR 'System32/tar.exe') -xf $archive.FullName -C (Join-Path $root 'source')
if($LASTEXITCODE -ne 0){throw 'source extraction failed'}
Get-ChildItem -LiteralPath (Join-Path $root 'source') -File -Recurse -Force|ForEach-Object {$_.IsReadOnly=$true}
Save 'source-before.json' (Assert-Source)
$before=Shared
if(-not $before.old_failed_root_absent){throw 'old failed root unexpectedly exists'}
Save 'shared-before.json' $before
$cargoHome=Join-Path $root 'cargo-home';New-Item -ItemType Directory -Path $cargoHome|Out-Null
Copy-Item -LiteralPath $registry -Destination (Join-Path $cargoHome 'registry') -Recurse -Force
$original=@($before.trees|Where-Object {$_.label -eq 'original-registry'})[0]
$copy=Snapshot 'private-registry-copy' (Join-Path $cargoHome 'registry')
if($copy.entries.Count -ne $original.entries.Count){throw 'cache copy count mismatch'}
$byPath=@{};foreach($e in $copy.entries){$byPath[$e.path]=$e}
foreach($e in $original.entries){$a=$byPath[$e.path];if(-not $a -or $a.directory -ne $e.directory -or (-not $e.directory -and ($a.bytes -ne $e.bytes -or $a.sha256 -ne $e.sha256))){throw 'cache copy bytes mismatch'}}
Get-ChildItem -LiteralPath (Join-Path $cargoHome 'registry/src') -File -Recurse -Force|ForEach-Object {$_.IsReadOnly=$true}
New-Item -ItemType Directory -Path (Join-Path $root 'temp')|Out-Null
$rustc=@(& (Join-Path $toolRoot 'rustc.exe') -vV);if($LASTEXITCODE -ne 0 -or $rustc[0] -notmatch '^rustc 1\.99\.1 '){throw 'native rustc version mismatch'}
$cargo=@(& (Join-Path $toolRoot 'cargo.exe') -V);if($LASTEXITCODE -ne 0 -or $cargo[0] -notmatch '^cargo 1\.99\.1 '){throw 'cargo identity unavailable or wrong version'}
Save 'qualification.json' ([ordered]@{passed=$true;source_commit=$manifest.source_commit;archive_sha256=$manifest.archive_sha256;module_sha256=$manifest.module_sha256;source_files=$manifest.files.Count;private_cache_files=$copy.files;credentials_or_global_config_copied=$false;rustc=$rustc;cargo=$cargo;session=(Get-Process -Id $PID).SessionId;priority=(Get-Process -Id $PID).PriorityClass.ToString();runtime_invocations=0})
Write-Output 'QUALIFIED_FIXED_FILES_AND_NATIVE_SOURCE_NO_RUNTIME'
