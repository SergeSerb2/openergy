param([Parameter(Mandatory=$true)][string]$FrozenPath)
. (Join-Path $PSScriptRoot 'common.ps1')
Set-Location -LiteralPath $root
$before=Get-Content -Raw (Join-Path $root 'frozen-all-files-before.json')|ConvertFrom-Json
$all=@(Get-ChildItem -LiteralPath $FrozenPath -Recurse -File -Force)
if($all.Count -ne $before.Count){throw 'original fixture file count changed'}
foreach($e in $before){$f=Get-Item -LiteralPath (Join-Path $FrozenPath $e.path);if($f.Length -ne $e.bytes -or $f.IsReadOnly -ne $e.readonly -or (Hash $f.FullName) -ne $e.sha256){throw ('original fixture changed: '+$e.path)}}
foreach($name in @('input-hashes.json','input-hashes-v2.json')){
 $inputs=Get-Content -Raw (Join-Path $root $name)|ConvertFrom-Json
 foreach($e in $inputs){$f=Get-Item (Join-Path $root $e.path);if(-not $f.IsReadOnly -or (Hash $f.FullName) -ne $e.sha256){throw 'diagnostic source/script input changed'}}
}
$compile=Get-Content -Raw (Join-Path $root 'compile-result.json')|ConvertFrom-Json
$second=Get-Content -Raw (Join-Path $root 'compile-v2-result.json')|ConvertFrom-Json
$binaries=@(foreach($c in @($compile,$second)){$f=Get-Item $c.binary.path;if(-not $f.IsReadOnly -or (Hash $f.FullName) -ne $c.binary.sha256){throw 'diagnostic binary changed'};File-Record $f})
$inv=Inventory 'postflight';$acl=Assert-ACL;$tools=Tools
$runBefore=Get-Content -Raw (Join-Path $root 'diagnostic1-before.json')|ConvertFrom-Json
if(($tools.tools|ConvertTo-Json -Compress) -ne ($runBefore.tools.tools|ConvertTo-Json -Compress) -or $tools.rustup_settings_sha256 -ne $runBefore.tools.rustup_settings_sha256 -or ($tools.toolchains -join "`n") -ne ($runBefore.tools.toolchains -join "`n") -or ($tools.cargo_configs|ConvertTo-Json -Compress) -ne ($runBefore.tools.cargo_configs|ConvertTo-Json -Compress)){throw 'tools/default/config changed'}
$oldAcl=Get-Acl -LiteralPath $FrozenPath;$sid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value
if(-not $oldAcl.AreAccessRulesProtected -or @($oldAcl.Access).Count -ne 1 -or $oldAcl.Access[0].IdentityReference.Translate([Security.Principal.SecurityIdentifier]).Value -ne $sid){throw 'old fixture ACL mismatch'}
$oldProcesses=@(Get-CimInstance Win32_Process -ErrorAction Stop|Where-Object {$_.ExecutablePath -and $_.ExecutablePath.StartsWith($FrozenPath+'\',[StringComparison]::OrdinalIgnoreCase)})
$temp=@(Get-ChildItem (Join-Path $root 'temp') -Force)
$result=[ordered]@{recorded_utc=[DateTime]::UtcNow.ToString('o');old_fixture_files_unchanged=$all.Count;old_all_file_bytes_hashes_readonly_flags_unchanged=$true;old_root_acl_protected_current_user_only=$true;old_fixture_processes=$oldProcesses;new_acl=$acl;new_binaries=$binaries;inventory=$inv;tools=$tools;global_tool_defaults_unchanged=$true;temp_entries=@($temp|Select-Object -ExpandProperty Name);diagnostic_runs=@(Get-ChildItem $root -File -Filter 'diagnostic?-result.json'|Select-Object -ExpandProperty Name);retained_files=@(Get-ChildItem $root -File -Recurse|ForEach-Object {File-Record $_})}
$result.passed=$inv.fixture_processes.Count -eq 0 -and $oldProcesses.Count -eq 0 -and $inv.assigned_tcp_listeners.Count -eq 0 -and $inv.assigned_udp_endpoints.Count -eq 0 -and $temp.Count -eq 0 -and $result.diagnostic_runs.Count -eq 2
Save-Json $result (Join-Path $root 'postflight.json')
if(-not $result.passed){throw 'diagnostic cleanup incomplete'}
Write-Output 'DIAGNOSTIC_POSTFLIGHT_PASS'
