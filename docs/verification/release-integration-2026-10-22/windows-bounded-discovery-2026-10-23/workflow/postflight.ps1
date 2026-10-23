. (Join-Path $PSScriptRoot 'common.ps1')
Set-Location -LiteralPath $root
$pre=Get-Content -Raw -LiteralPath (Join-Path $root 'preflight.json')|ConvertFrom-Json
$artifacts=Get-Content -Raw -LiteralPath (Join-Path $root 'artifact-manifest.json')|ConvertFrom-Json
$files=@(Get-ChildItem -LiteralPath (Join-Path $root 'target') -File -Recurse -Force)+@(Get-ChildItem -LiteralPath (Join-Path $root 'bin') -File -Recurse -Force)
if($files.Count -ne $artifacts.Count){throw 'artifact file count mismatch'}
foreach($e in $artifacts){$f=Get-Item -LiteralPath (Join-Path $root $e.path);if($f.Length -ne $e.bytes -or -not $f.IsReadOnly -or (Hash $f.FullName) -ne $e.sha256){throw ('artifact mismatch: '+$e.path)}}
$source=Assert-Source;$acl=Assert-ACL;$inventory=Inventory 'postflight';$tools=Tools
if(($tools.tools|ConvertTo-Json -Compress) -ne ($pre.tools.tools|ConvertTo-Json -Compress) -or $tools.rustup_settings_sha256 -ne $pre.tools.rustup_settings_sha256 -or ($tools.toolchains -join "`n") -ne ($pre.tools.toolchains -join "`n") -or ($tools.cargo_configs|ConvertTo-Json -Compress) -ne ($pre.tools.cargo_configs|ConvertTo-Json -Compress)){throw 'global tool/default/config changed'}
$inputs=Get-Content -Raw -LiteralPath (Join-Path $root 'input-hashes.json')|ConvertFrom-Json
foreach($e in $inputs){if((Hash (Join-Path $root $e.path)) -ne $e.sha256){throw ('fixture input changed: '+$e.path)}}
$temp=@(Get-ChildItem -LiteralPath (Join-Path $root 'temp') -Force)
$pass=$inventory.fixture_processes.Count -eq 0 -and $inventory.assigned_tcp_listeners.Count -eq 0 -and $inventory.assigned_udp_endpoints.Count -eq 0 -and $temp.Count -eq 0
$result=[ordered]@{recorded_utc=[DateTime]::UtcNow.ToString('o');passed=$pass;source=$source;artifacts_unchanged_readonly=$files.Count;inputs_unchanged=$inputs.Count;acl=$acl;inventory=$inventory;tools=$tools;toolchain_defaults_unchanged=$true;temp_entries=@($temp|Select-Object -ExpandProperty Name)}
Save-Json $result (Join-Path $root 'postflight.json')
if(-not $pass){throw 'postflight cleanup incomplete'}
Write-Output 'POSTFLIGHT_PASS'
