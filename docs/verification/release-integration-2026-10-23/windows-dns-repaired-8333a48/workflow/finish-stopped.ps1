. (Join-Path $PSScriptRoot 'runtime-common.ps1')
Set-Location -LiteralPath $root
if(Test-Path (Join-Path $root 'run-start.json')){throw 'unexpected probe marker; inspect and stop owned probe before audit'}
$before=Get-Content -Raw (Join-Path $root 'build-start.json')|ConvertFrom-Json
$build=Get-Content -Raw (Join-Path $root 'build-result.json')|ConvertFrom-Json
$immutable=Get-Content -Raw (Join-Path $root 'immutable-inputs-before.json')|ConvertFrom-Json
$comparisons=@(foreach($snapshot in $immutable){Compare-Tree $snapshot})
$globals=Get-Content -Raw (Join-Path $root 'original-global-files-before.json')|ConvertFrom-Json
foreach($e in $globals){$exists=Test-Path -LiteralPath $e.path;if($exists -ne $e.exists){throw 'original Cargo/Rustup global presence changed'};if($exists){$f=Get-Item -LiteralPath $e.path;if($f.Length -ne $e.bytes -or $f.IsReadOnly -ne $e.readonly -or (Hash $f.FullName) -ne $e.sha256 -or (String-Hash (Get-Acl -LiteralPath $f.FullName).Sddl) -ne $e.acl_sha256){throw 'original Cargo/Rustup global file changed'}}}
$privateSource=Get-Content -Raw (Join-Path $root 'private-registry-source-before.json')|ConvertFrom-Json
foreach($e in $privateSource){$f=Get-Item (Join-Path $root $e.path);if(-not $f.IsReadOnly -or $f.Length -ne $e.bytes -or (Hash $f.FullName) -ne $e.sha256){throw 'private registry source changed'}}
$source=Assert-Source
$artifacts=Get-Content -Raw (Join-Path $root 'artifact-manifest.json')|ConvertFrom-Json
$all=@(Get-ChildItem (Join-Path $root 'target') -File -Recurse -Force)+@(Get-ChildItem (Join-Path $root 'bin') -File -Recurse -Force)
if($all.Count -ne $artifacts.Count){throw 'frozen artifact count changed'}
foreach($e in $artifacts){$f=Get-Item (Join-Path $root $e.path);if(-not $f.IsReadOnly -or $f.Length -ne $e.bytes -or (Hash $f.FullName) -ne $e.sha256){throw 'frozen artifact changed'}}
$inputs=Get-Content -Raw (Join-Path $root 'final-input-hashes.json')|ConvertFrom-Json
foreach($e in $inputs){$f=Get-Item (Join-Path $root $e.path);if(-not $f.IsReadOnly -or (Hash $f.FullName) -ne $e.sha256){throw 'archive/manifest/executed workflow changed'}}
$tools=Tools
if(($tools.tools|ConvertTo-Json -Compress) -ne ($before.tools_before.tools|ConvertTo-Json -Compress) -or $tools.rustup_settings_sha256 -ne $before.tools_before.rustup_settings_sha256 -or ($tools.toolchains -join "`n") -ne ($before.tools_before.toolchains -join "`n") -or ($tools.cargo_configs|ConvertTo-Json -Compress) -ne ($before.tools_before.cargo_configs|ConvertTo-Json -Compress)){throw 'tool/default/config changed'}
$nativeReference=Get-Content -Raw (Join-Path $root 'native-tool-reference.json')|ConvertFrom-Json
$nativeTools=@(foreach($e in $nativeReference.native_tools){$f=Get-Item -LiteralPath $e.path;if($f.Length -ne $e.bytes -or (Hash $f.FullName) -ne $e.sha256){throw 'installed native tool differs from prior recorded hash'};[ordered]@{name=$e.name;path=$e.path;bytes=$f.Length;sha256=(Hash $f.FullName);file_version=$f.VersionInfo.FileVersion}})
Save-Json ([ordered]@{recorded_utc=[DateTime]::UtcNow.ToString('o');native_tools=$nativeTools;matches_prior_snapshot=$true;scope='post-build hashes matched immutable prior recorded reference; not a new before-build sample'}) (Join-Path $root 'native-tools-final.json')
$inventory=Inventory 'stopped-postflight'
$shareBefore=@($before.inventory_before.sharing_processes|Sort-Object ProcessId|Select-Object Name,ProcessId,CreationDate,SessionId,ExecutablePath)
$shareAfter=@($inventory.sharing_processes|Sort-Object ProcessId|Select-Object Name,ProcessId,CreationDate,SessionId,ExecutablePath)
$sharingSame=($shareBefore|ConvertTo-Json -Compress) -eq ($shareAfter|ConvertTo-Json -Compress)
$processes=@(Get-CimInstance Win32_Process -ErrorAction Stop);if($processes.Count -lt 10){throw 'process inventory unavailable'}
$oldProcesses=@(foreach($old in $immutable|Where-Object {$_.label -notin @('original-offline-registry','pinned-rust-1.99.1-toolchain')}){$processes|Where-Object {$_.ExecutablePath -and $_.ExecutablePath.StartsWith($old.root+'\',[StringComparison]::OrdinalIgnoreCase)}|Select-Object Name,ProcessId})
$runtime=@(Get-ChildItem (Join-Path $root 'runtime') -Recurse -Force);$temp=@(Get-ChildItem (Join-Path $root 'temp') -Force)
$runtimeWorkflowAbsent=-not (Test-Path (Join-Path $root 'run.ps1'))
$acl=Assert-ACL
$passed=$inventory.fixture_processes.Count -eq 0 -and $inventory.assigned_tcp_non_timewait.Count -eq 0 -and $inventory.assigned_udp_endpoints.Count -eq 0 -and $oldProcesses.Count -eq 0 -and $temp.Count -eq 0 -and $runtime.Count -eq 0 -and $sharingSame -and $runtimeWorkflowAbsent
$r=[ordered]@{recorded_utc=[DateTime]::UtcNow.ToString('o');passed=$passed;stop_reason='R9 localhost fixture ownership gap; candidate runtime unaccepted';normal_full='NOT_RUN';owned_conflict_full='NOT_RUN';probe_invocations=0;run_start_exists=$false;runtime_workflow_not_transferred=$runtimeWorkflowAbsent;tls_identities_created=0;tls_identities_remaining=0;forced_process_stops=0;immutable_input_comparisons=$comparisons;original_global_files_unchanged=$globals.Count;private_registry_source_files_unchanged=$privateSource.Count;candidate_source=$source;frozen_artifact_files_unchanged=$all.Count;workflow_inputs_unchanged=$inputs.Count;tools=$tools;tool_defaults_unchanged=$true;native_tools_match_prior_record=$nativeTools.Count;live_sharing_process_identity_unchanged=$sharingSame;inventory=$inventory;prior_fixture_processes=$oldProcesses;acl=$acl;temp_entries=@($temp|Select-Object -ExpandProperty Name);runtime_entries=@($runtime|Select-Object -ExpandProperty Name)}
Save-Json $r (Join-Path $root 'postflight.json')
if(-not $passed){throw 'stopped fixture cleanup/invariance audit failed'}
Write-Output 'STOPPED_BEFORE_RUNTIME_INPUT_AND_CLEANUP_AUDIT_PASS'
