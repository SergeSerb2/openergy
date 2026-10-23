. (Join-Path $PSScriptRoot 'common.ps1')
Set-Location -LiteralPath $root
Add-Type -Path (Join-Path $root 'bin/OwnedNativeObserver.dll')
$old=Get-Content -Raw (Join-Path $root 'old-inputs-before.json')|ConvertFrom-Json
$oldAudits=@(foreach($fixture in $old){
 $files=@(Get-ChildItem -LiteralPath $fixture.root -File -Recurse -Force)
 if($files.Count -ne $fixture.files.Count){throw ('old fixture file count changed: '+$fixture.label)}
 foreach($e in $fixture.files){$f=Get-Item (Join-Path $fixture.root $e.path);if($f.Length -ne $e.bytes -or $f.IsReadOnly -ne $e.readonly -or (Hash $f.FullName) -ne $e.sha256){throw ('old fixture changed: '+$fixture.label+'/'+$e.path)}}
 $a=Get-Acl $fixture.root;$sid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value
 if(-not $a.AreAccessRulesProtected -or @($a.Access).Count -ne 1 -or $a.Access[0].IdentityReference.Translate([Security.Principal.SecurityIdentifier]).Value -ne $sid){throw 'old protected ACL changed'}
 $owned=@(Get-CimInstance Win32_Process -ErrorAction Stop|Where-Object {$_.ExecutablePath -and $_.ExecutablePath.StartsWith($fixture.root+'\',[StringComparison]::OrdinalIgnoreCase)})
 if($owned.Count -ne 0){throw 'an old fixture executable remains'}
 [ordered]@{label=$fixture.label;files=$files.Count;all_bytes_hashes_readonly_flags_match=$true;root_current_user_only_protected=$true;fixture_executables=0}
})
$inputs=Get-Content -Raw (Join-Path $root 'input-hashes-final.json')|ConvertFrom-Json
foreach($e in $inputs){$f=Get-Item (Join-Path $root $e.path);if(-not $f.IsReadOnly -or (Hash $f.FullName) -ne $e.sha256){throw 'new source/workflow input changed'}}
$bins=Get-Content -Raw (Join-Path $root 'frozen-binaries.json')|ConvertFrom-Json
foreach($e in $bins){$f=Get-Item (Join-Path $root $e.path);if(-not $f.IsReadOnly -or $f.Length -ne $e.bytes -or (Hash $f.FullName) -ne $e.sha256){throw 'new binary/observer changed'}}
$observation=Get-Content -Raw (Join-Path $root 'observation-start.json')|ConvertFrom-Json
$tools=Tools
if(($tools.tools|ConvertTo-Json -Compress) -ne ($observation.tools.tools|ConvertTo-Json -Compress) -or $tools.rustup_settings_sha256 -ne $observation.tools.rustup_settings_sha256 -or ($tools.toolchains -join "`n") -ne ($observation.tools.toolchains -join "`n") -or ($tools.cargo_configs|ConvertTo-Json -Compress) -ne ($observation.tools.cargo_configs|ConvertTo-Json -Compress)){throw 'tool/default/config changed'}
$cases=Get-Content -Raw (Join-Path $root 'observation-result.json')|ConvertFrom-Json
$native=[OwnedNativeObserver]::ReadUdp()
$inventory=Inventory 'postflight';$acl=Assert-ACL
$sharingBefore=@($observation.inventory.sharing_processes|Sort-Object ProcessId|Select-Object Name,ProcessId,CreationDate,SessionId,ExecutablePath)
$sharingAfter=@($inventory.sharing_processes|Sort-Object ProcessId|Select-Object Name,ProcessId,CreationDate,SessionId,ExecutablePath)
$sharingSame=($sharingBefore|ConvertTo-Json -Compress) -eq ($sharingAfter|ConvertTo-Json -Compress)
$temp=@(Get-ChildItem (Join-Path $root 'temp') -Force)
$result=[ordered]@{recorded_utc=[DateTime]::UtcNow.ToString('o');old_fixture_audits=$oldAudits;new_inputs_unchanged=$inputs.Count;new_binaries_unchanged=$bins;tools=$tools;tool_defaults_unchanged=$true;inventory=$inventory;acl=$acl;native_udp_queries=$native.Queries;native_global_udp_count=$native.Rows.Count;temp_entries=@($temp|Select-Object -ExpandProperty Name);live_sharing_process_identity_unchanged=$sharingSame;case_verdicts=@($cases|ForEach-Object {[ordered]@{mode=$_.mode;passed=$_.passed;exit=$_.exit;resources_absent_observed_ms=$_.resources_absent_observed_ms}})}
$result.passed=$inventory.fixture_processes.Count -eq 0 -and $inventory.assigned_tcp_listeners.Count -eq 0 -and $inventory.assigned_udp_endpoints.Count -eq 0 -and $temp.Count -eq 0 -and $sharingSame
Save-Json $result (Join-Path $root 'postflight.json')
if(-not $result.passed){throw 'final owned fixture cleanup or sharing audit failed'}
Write-Output 'OWNED_CLEANUP_POSTFLIGHT_PASS'
