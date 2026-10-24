. (Join-Path $PSScriptRoot 'runtime-common.ps1')
Set-Location -LiteralPath $root
$immutable=Get-Content -Raw (Join-Path $root 'immutable-inputs-before.json')|ConvertFrom-Json
$comparisons=@(foreach($snapshot in $immutable){Compare-Tree $snapshot})
$globals=Get-Content -Raw (Join-Path $root 'original-global-files-before.json')|ConvertFrom-Json
foreach($e in $globals){$exists=Test-Path -LiteralPath $e.path;if($exists -ne $e.exists){throw 'original global presence changed'};if($exists){$f=Get-Item -LiteralPath $e.path;if($f.Length -ne $e.bytes -or $f.IsReadOnly -ne $e.readonly -or (Hash $f.FullName) -ne $e.sha256 -or (String-Hash (Get-Acl -LiteralPath $f.FullName).Sddl) -ne $e.acl_sha256){throw 'original global changed'}}}
$privateSource=Get-Content -Raw (Join-Path $root 'private-registry-source-before.json')|ConvertFrom-Json
foreach($e in $privateSource){$f=Get-Item (Join-Path $root $e.path);if(-not $f.IsReadOnly -or $f.Length -ne $e.bytes -or (Hash $f.FullName) -ne $e.sha256){throw 'private copied registry source changed'}}
$source=Assert-Source
$artifacts=@()
if(Test-Path (Join-Path $root 'artifact-manifest.json')){
 $artifacts=@(Get-Content -Raw (Join-Path $root 'artifact-manifest.json')|ConvertFrom-Json)
 $all=@(Get-ChildItem (Join-Path $root 'target') -File -Recurse -Force)+@(Get-ChildItem (Join-Path $root 'bin') -File -Recurse -Force)
 if($all.Count -ne $artifacts.Count){throw 'frozen artifact count changed'}
 foreach($e in $artifacts){$f=Get-Item (Join-Path $root $e.path);if(-not $f.IsReadOnly -or $f.Length -ne $e.bytes -or (Hash $f.FullName) -ne $e.sha256){throw 'frozen artifact changed'}}
}
$inputs=Get-Content -Raw (Join-Path $root 'final-input-hashes.json')|ConvertFrom-Json
foreach($e in $inputs){$f=Get-Item -LiteralPath (Join-Path $root $e.path);if(-not $f.IsReadOnly -or (Hash $f.FullName) -ne $e.sha256){throw 'frozen input changed'}}
$native=Get-Content -Raw (Join-Path $root 'native-tools-before.json')|ConvertFrom-Json
foreach($e in $native.native_tools){$f=Get-Item -LiteralPath $e.path;if($f.Length -ne $e.bytes -or (Hash $f.FullName) -ne $e.sha256){throw 'native tool bytes changed'}}
$metadata=Get-Content -Raw (Join-Path $root 'native-metadata-before.json')|ConvertFrom-Json
foreach($e in $metadata){$f=Get-Item -LiteralPath $e.path;if($f.IsReadOnly -ne $e.readonly -or (String-Hash (Get-Acl -LiteralPath $f.FullName).Sddl) -ne $e.acl_sha256){throw 'native tool flags or ACL changed'}}
Save-Json ([ordered]@{recorded_utc=[DateTime]::UtcNow.ToString('o');native_tools=$native.native_tools;bytes_readonly_acls_unchanged=$true}) (Join-Path $root 'native-tools-after.json')
$build=Get-Content -Raw (Join-Path $root 'build-start.json')|ConvertFrom-Json;$tools=Tools
if(($tools.tools|ConvertTo-Json -Compress) -ne ($build.tools_before.tools|ConvertTo-Json -Compress) -or $tools.rustup_settings_sha256 -ne $build.tools_before.rustup_settings_sha256 -or ($tools.toolchains -join "`n") -ne ($build.tools_before.toolchains -join "`n") -or ($tools.cargo_configs|ConvertTo-Json -Compress) -ne ($build.tools_before.cargo_configs|ConvertTo-Json -Compress)){throw 'tools defaults or config changed'}
$identities=@(Get-ChildItem (Join-Path $root 'runtime') -File -Recurse -Force|Where-Object {$_.Name -in @('cert.pem','key.pem','.identity-lock')})
Save-Json @($identities|ForEach-Object {File-Record $_}) (Join-Path $root 'postflight-identities-removed.json');foreach($f in $identities){Remove-Item -LiteralPath $f.FullName -Force}
$inventory=Inventory 'postflight';$acl=Assert-ACL
$shareBefore=@($build.inventory_before.sharing_processes|Sort-Object ProcessId|Select-Object Name,ProcessId,CreationDate,SessionId,ExecutablePath)
$shareAfter=@($inventory.sharing_processes|Sort-Object ProcessId|Select-Object Name,ProcessId,CreationDate,SessionId,ExecutablePath)
$sharingSame=($shareBefore|ConvertTo-Json -Compress) -eq ($shareAfter|ConvertTo-Json -Compress)
$oldProcesses=@(foreach($old in $immutable|Where-Object {$_.label -notin @('original-offline-registry','pinned-rust-1.99.1-toolchain')}){Get-CimInstance Win32_Process -ErrorAction Stop|Where-Object {$_.ExecutablePath -and $_.ExecutablePath.StartsWith($old.root+'\',[StringComparison]::OrdinalIgnoreCase)}|Select-Object Name,ProcessId})
$temp=@(Get-ChildItem (Join-Path $root 'temp') -Force);$runtimeFiles=@(Get-ChildItem (Join-Path $root 'runtime') -File -Recurse -Force)
$cases=@();if(Test-Path (Join-Path $root 'run-result.json')){$cases=@(Get-Content -Raw (Join-Path $root 'run-result.json')|ConvertFrom-Json)}
$passed=$inventory.fixture_processes.Count -eq 0 -and $inventory.build_processes.Count -eq 0 -and $inventory.assigned_tcp_non_timewait.Count -eq 0 -and $inventory.assigned_udp_endpoints.Count -eq 0 -and $oldProcesses.Count -eq 0 -and $temp.Count -eq 0 -and $runtimeFiles.Count -eq 0 -and $sharingSame
$r=[ordered]@{recorded_utc=[DateTime]::UtcNow.ToString('o');passed=$passed;immutable_input_comparisons=$comparisons;original_global_files_unchanged=$globals.Count;private_registry_source_files_unchanged=$privateSource.Count;historical_source=$source;frozen_artifact_files_unchanged=$artifacts.Count;workflow_inputs_unchanged=$inputs.Count;native_tools_bytes_flags_acls_unchanged=$native.native_tools.Count;tools=$tools;tool_defaults_unchanged=$true;live_sharing_process_identity_unchanged=$sharingSame;inventory=$inventory;prior_fixture_processes=$oldProcesses;acl=$acl;temp_entries=@($temp|Select-Object -ExpandProperty Name);tls_identity_files_remaining=$runtimeFiles.Count;case_count=$cases.Count;case_exits=@($cases|ForEach-Object {[ordered]@{name=$_.name;exit=$_.exit;timed_out=$_.timed_out;forced_cleanup=$_.forced_cleanup}})}
Save-Json $r (Join-Path $root 'postflight.json');if(-not $passed){throw 'preservation or cleanup audit failed'}
Write-Output 'HISTORICAL_SOURCE_PRIOR_FIXTURES_CACHE_TOOLS_ACLS_AND_CLEANUP_PASS'
