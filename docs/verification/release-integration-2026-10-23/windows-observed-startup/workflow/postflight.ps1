. (Join-Path $PSScriptRoot 'audit.ps1')
Set-Location -LiteralPath $root
$immutable=Get-Content -Raw (Join-Path $root 'immutable-inputs-before.json')|ConvertFrom-Json
$comparisons=@(foreach($snapshot in $immutable){Compare-Tree $snapshot})
$globals=Get-Content -Raw (Join-Path $root 'original-global-files-before.json')|ConvertFrom-Json
foreach($e in $globals){
 $exists=Test-Path -LiteralPath $e.path;if($exists -ne $e.exists){throw 'original Cargo/Rustup global file presence changed'}
 if($exists){$f=Get-Item -LiteralPath $e.path;if($f.Length -ne $e.bytes -or $f.IsReadOnly -ne $e.readonly -or (Hash $f.FullName) -ne $e.sha256 -or (String-Hash (Get-Acl -LiteralPath $f.FullName).Sddl) -ne $e.acl_sha256){throw 'original Cargo/Rustup global file changed'}}
}
$privateSource=Get-Content -Raw (Join-Path $root 'private-registry-source-before.json')|ConvertFrom-Json
foreach($e in $privateSource){$f=Get-Item (Join-Path $root $e.path);if(-not $f.IsReadOnly -or $f.Length -ne $e.bytes -or (Hash $f.FullName) -ne $e.sha256){throw 'private copied registry source changed'}}
$source=Assert-Source
$artifacts=Get-Content -Raw (Join-Path $root 'artifact-manifest.json')|ConvertFrom-Json
$all=@(Get-ChildItem (Join-Path $root 'target') -File -Recurse -Force)+@(Get-ChildItem (Join-Path $root 'bin') -File -Recurse -Force)
if($all.Count -ne $artifacts.Count){throw 'frozen artifact count changed'}
foreach($e in $artifacts){$f=Get-Item (Join-Path $root $e.path);if(-not $f.IsReadOnly -or $f.Length -ne $e.bytes -or (Hash $f.FullName) -ne $e.sha256){throw 'frozen artifact changed'}}
$inputs=Get-Content -Raw (Join-Path $root 'final-input-hashes.json')|ConvertFrom-Json
foreach($e in $inputs){$f=Get-Item (Join-Path $root $e.path);if(-not $f.IsReadOnly -or (Hash $f.FullName) -ne $e.sha256){throw 'source archive/manifest/workflow input changed'}}
$runStart=Get-Content -Raw (Join-Path $root 'run-start.json')|ConvertFrom-Json
$tools=Tools
if(($tools.tools|ConvertTo-Json -Compress) -ne ($runStart.tools.tools|ConvertTo-Json -Compress) -or $tools.rustup_settings_sha256 -ne $runStart.tools.rustup_settings_sha256 -or ($tools.toolchains -join "`n") -ne ($runStart.tools.toolchains -join "`n") -or ($tools.cargo_configs|ConvertTo-Json -Compress) -ne ($runStart.tools.cargo_configs|ConvertTo-Json -Compress)){throw 'tool/default/config changed'}
$inventory=Inventory 'postflight';$acl=Assert-ACL
$before=Get-Content -Raw (Join-Path $root 'build-start.json')|ConvertFrom-Json
$shareBefore=@($before.inventory_before.sharing_processes|Sort-Object ProcessId|Select-Object Name,ProcessId,CreationDate,SessionId,ExecutablePath)
$shareAfter=@($inventory.sharing_processes|Sort-Object ProcessId|Select-Object Name,ProcessId,CreationDate,SessionId,ExecutablePath)
$sharingSame=($shareBefore|ConvertTo-Json -Compress) -eq ($shareAfter|ConvertTo-Json -Compress)
$oldProcesses=@(foreach($old in $immutable|Where-Object {$_.label -in @('e4f48ba-original','startup-diagnosis','owned-cleanup-dfa405c')}){Get-CimInstance Win32_Process -ErrorAction Stop|Where-Object {$_.ExecutablePath -and $_.ExecutablePath.StartsWith($old.root+'\',[StringComparison]::OrdinalIgnoreCase)}|Select-Object Name,ProcessId})
$temp=@(Get-ChildItem (Join-Path $root 'temp') -Force)
$results=Get-Content -Raw (Join-Path $root 'run-result.json')|ConvertFrom-Json
$passed=$inventory.fixture_processes.Count -eq 0 -and $inventory.assigned_tcp_listeners.Count -eq 0 -and $inventory.assigned_udp_endpoints.Count -eq 0 -and $oldProcesses.Count -eq 0 -and $temp.Count -eq 0 -and $sharingSame -and $results.Count -eq 5
$r=[ordered]@{recorded_utc=[DateTime]::UtcNow.ToString('o');passed=$passed;immutable_input_comparisons=$comparisons;original_global_files_unchanged=$globals.Count;private_registry_source_files_unchanged=$privateSource.Count;candidate_source=$source;frozen_artifact_files_unchanged=$all.Count;workflow_inputs_unchanged=$inputs.Count;tools=$tools;tool_defaults_unchanged=$true;live_sharing_process_identity_unchanged=$sharingSame;inventory=$inventory;prior_fixture_processes=$oldProcesses;acl=$acl;temp_entries=@($temp|Select-Object -ExpandProperty Name);case_count=$results.Count;case_exits=@($results|ForEach-Object {[ordered]@{name=$_.name;exit=$_.exit;check_count=$_.check_count;passed_checks=$_.passed_checks;timed_out=$_.timed_out;parent_control_env_unchanged=$_.parent_control_env_unchanged}})}
Save-Json $r (Join-Path $root 'postflight.json')
if(-not $passed){throw 'input invariance or process/endpoint cleanup failed'}
Write-Output 'INPUT_ACL_CACHE_COMPILER_AND_CLEANUP_AUDIT_PASS'
