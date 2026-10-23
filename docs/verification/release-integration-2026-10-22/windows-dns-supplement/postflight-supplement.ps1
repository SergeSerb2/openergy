param([Parameter(Mandatory=$true)][string]$FrozenPath)
$ErrorActionPreference='Stop';$ProgressPreference='SilentlyContinue';$root=$PSScriptRoot
(Get-Process -Id $PID).PriorityClass='BelowNormal';$env:TEMP=Join-Path $root 'temp';$env:TMP=$env:TEMP
$manifest=Get-Content -Raw -LiteralPath (Join-Path $FrozenPath 'source-manifest.json')|ConvertFrom-Json
$files=@(Get-ChildItem -LiteralPath (Join-Path $FrozenPath 'source') -Recurse -File -Force)
if($files.Count -ne $manifest.files.Count){throw 'frozen source count mismatch'}
foreach($entry in $manifest.files){$file=Get-Item -LiteralPath (Join-Path (Join-Path $FrozenPath 'source') $entry.path);if($file.Length -ne $entry.bytes -or -not $file.IsReadOnly -or (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash -ne $entry.sha256){throw 'frozen source changed'}}
$original=Get-Content -Raw -LiteralPath (Join-Path $FrozenPath 'postflight.json')|ConvertFrom-Json
foreach($entry in $original.binaries){if((Get-FileHash -LiteralPath (Join-Path $FrozenPath $entry.path) -Algorithm SHA256).Hash -ne $entry.sha256){throw 'frozen executable changed'}}
foreach($entry in $original.file_hashes){if((Get-FileHash -LiteralPath (Join-Path $FrozenPath $entry.name) -Algorithm SHA256).Hash -ne $entry.sha256){throw 'frozen archive/manifest/script changed'}}
$libs=Get-Content -Raw -LiteralPath (Join-Path $FrozenPath 'library-artifacts.json')|ConvertFrom-Json
foreach($entry in $libs.libraries){if((Get-FileHash -LiteralPath (Join-Path $FrozenPath $entry.path) -Algorithm SHA256).Hash -ne $entry.sha256){throw 'frozen library changed'}}
$hashes=Get-Content -Raw -LiteralPath (Join-Path $root 'frozen-recorded-hashes.json')|ConvertFrom-Json
foreach($property in $hashes.PSObject.Properties){if((Get-FileHash -LiteralPath (Join-Path $FrozenPath $property.Name) -Algorithm SHA256).Hash -ne $property.Value){throw ('original evidence changed: '+$property.Name)}}
$compile=Get-Content -Raw -LiteralPath (Join-Path $root 'compile-result.json')|ConvertFrom-Json
$binary=Get-Item -LiteralPath (Join-Path $root 'bin/windows_dns_supplement.exe')
if(-not $binary.IsReadOnly -or (Get-FileHash -LiteralPath $binary.FullName -Algorithm SHA256).Hash -ne $compile.binary.sha256){throw 'diagnostic binary changed'}
$helper=Get-Item -LiteralPath (Join-Path $root 'source/windows_dns_supplement_v3.rs')
if(-not $helper.IsReadOnly -or (Get-FileHash -LiteralPath $helper.FullName -Algorithm SHA256).Hash -ne $compile.source_sha256){throw 'helper source changed'}
$tcp=@(& netstat.exe -ano -p TCP);$tcpExit=$LASTEXITCODE;$udp=@(& netstat.exe -ano -p UDP);$udpExit=$LASTEXITCODE
if($tcpExit -ne 0 -or $udpExit -ne 0 -or $tcp.Count -lt 3 -or $udp.Count -lt 3){throw 'endpoint inventory unavailable'}
$tcp|Set-Content -LiteralPath (Join-Path $root 'logs/postflight-netstat-tcp.txt');$udp|Set-Content -LiteralPath (Join-Path $root 'logs/postflight-netstat-udp.txt')
$assignedTcp=@($tcp|Where-Object {$_ -match '^s*TCPs+S+:(2510[0-9]|2511[0-9])s+S+s+LISTENINGs+'})
$assignedUdp=@($udp|Where-Object {$_ -match '^s*UDPs+S+:(2510[0-9]|2511[0-9])s+'})
$owned=@(foreach($path in @($root,$FrozenPath)){$escaped=$path.Replace('\','\\');Get-CimInstance Win32_Process -Filter ("ExecutablePath LIKE '"+$escaped+"\\%' ") -ErrorAction Stop|Select-Object Name,ProcessId,ExecutablePath})
$sid=[System.Security.Principal.WindowsIdentity]::GetCurrent().User.Value;$acl=Get-Acl -LiteralPath $root
if(-not $acl.AreAccessRulesProtected){throw 'diagnostic root ACL not protected'}
$items=@((Get-Item -LiteralPath $root))+@(Get-ChildItem -LiteralPath $root -Recurse -Force)
foreach($item in $items){$a=Get-Acl -LiteralPath $item.FullName;$rules=@($a.Access);if($rules.Count -ne 1 -or $rules[0].IdentityReference.Translate([System.Security.Principal.SecurityIdentifier]).Value -ne $sid -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)){throw 'diagnostic ACL or reparse mismatch'}}
$toolRoot=Join-Path $env:USERPROFILE '.rustup/toolchains/1.99.1-x86_64-pc-windows-msvc/bin'
$tools=@(foreach($entry in $original.tools){$file=Get-Item -LiteralPath (Join-Path $toolRoot $entry.name);$hash=(Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToLower();if($hash -ne $entry.sha256){throw 'pinned Rust tool changed'};[ordered]@{name=$entry.name;bytes=$file.Length;sha256=$hash}})
$result=[ordered]@{recorded_utc=[DateTime]::UtcNow.ToString('o');passed=($owned.Count -eq 0 -and $assignedTcp.Count -eq 0 -and $assignedUdp.Count -eq 0);frozen_source_files=$files.Count;frozen_source_readonly_all_match=$true;frozen_binaries_all_match=$true;frozen_libraries_all_match=$true;original_evidence_files_unchanged=@($hashes.PSObject.Properties).Count;diagnostic_helper=[ordered]@{bytes=$helper.Length;sha256=$compile.source_sha256;readonly=$helper.IsReadOnly};diagnostic_binary=[ordered]@{bytes=$binary.Length;sha256=$compile.binary.sha256;readonly=$binary.IsReadOnly};tools=$tools;acl_protected=$acl.AreAccessRulesProtected;current_user_only_acl_items=$items.Count;remaining_fixture_executables=$owned;remaining_assigned_tcp_listeners=$assignedTcp;remaining_assigned_udp_endpoints=$assignedUdp;tcp_inventory_lines=$tcp.Count;udp_inventory_lines=$udp.Count;tcp_inventory_exit=$tcpExit;udp_inventory_exit=$udpExit;temp_entries=@(Get-ChildItem -LiteralPath (Join-Path $root 'temp') -Force|Select-Object -ExpandProperty Name);toolchains=@(& rustup.exe toolchain list);diagnostic_file_hashes=@(Get-ChildItem -LiteralPath $root -File -Recurse|Where-Object {$_.FullName -notlike '*postflight-netstat*'}|ForEach-Object {[ordered]@{path=$_.FullName.Substring($root.Length+1).Replace('\','/');bytes=$_.Length;sha256=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLower()}})}
$result|ConvertTo-Json -Depth 11|Set-Content -LiteralPath (Join-Path $root 'postflight.json');$result|ConvertTo-Json -Depth 11
if(-not $result.passed){exit 1}
