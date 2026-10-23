$ErrorActionPreference='Stop';$ProgressPreference='SilentlyContinue';$PSNativeCommandUseErrorActionPreference=$false
$root=$PSScriptRoot
(Get-Process -Id $PID).PriorityClass='BelowNormal'
if((Get-Process -Id $PID).SessionId -ne 0){throw 'session 0 required'}
$env:TEMP=Join-Path $root 'temp';$env:TMP=$env:TEMP
$env:CARGO_BUILD_JOBS='2';$env:RUSTUP_AUTO_INSTALL='0'
$env:OPENERGY_ALLOW_PORT_24800='0';$env:OPENERGY_DISABLE_DISCOVERY='1'
$toolRoot=Join-Path $env:USERPROFILE '.rustup/toolchains/1.99.1-x86_64-pc-windows-msvc/bin'
function Save-Json($value,$path){$value|ConvertTo-Json -Depth 24|Set-Content -LiteralPath $path -Encoding utf8}
function Hash($path){(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLower()}
function File-Record($file){[ordered]@{path=$file.FullName.Substring($root.Length+1).Replace('\','/');bytes=$file.Length;sha256=(Hash $file.FullName);readonly=$file.IsReadOnly}}
function Assert-Source {
 $manifest=Get-Content -Raw -LiteralPath (Join-Path $root 'source-manifest.json')|ConvertFrom-Json
 $source=Join-Path $root 'source';$all=@(Get-ChildItem -LiteralPath $source -File -Recurse -Force)
 if($all.Count -ne $manifest.files.Count){throw 'source file count mismatch'}
 foreach($e in $manifest.files){$f=Get-Item -LiteralPath (Join-Path $source $e.path);if($f.Length -ne $e.bytes -or -not $f.IsReadOnly -or (Hash $f.FullName) -ne $e.sha256){throw ('source mismatch: '+$e.path)}}
 [ordered]@{files=$all.Count;all_hashes_match=$true;all_readonly=$true;archive_sha256=(Hash (Join-Path $root 'source.tar'));manifest_sha256=(Hash (Join-Path $root 'source-manifest.json'))}
}
function Assert-ACL {
 $sid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value
 $a=Get-Acl -LiteralPath $root
 if(-not $a.AreAccessRulesProtected){throw 'root ACL not protected'}
 $items=@((Get-Item -LiteralPath $root))+@(Get-ChildItem -LiteralPath $root -Recurse -Force)
 foreach($i in $items){$a=Get-Acl -LiteralPath $i.FullName;$rules=@($a.Access);if(($i.Attributes -band [IO.FileAttributes]::ReparsePoint) -or $rules.Count -ne 1 -or $rules[0].AccessControlType -ne 'Allow' -or $rules[0].IdentityReference.Translate([Security.Principal.SecurityIdentifier]).Value -ne $sid){throw ('ACL/reparse mismatch: '+$i.FullName)}}
 [ordered]@{root_protected=$true;current_user_only_items=$items.Count;reparse_points=0}
}
function Inventory($label){
 $tcp=@(& netstat.exe -ano -p TCP);$te=$LASTEXITCODE;$udp=@(& netstat.exe -ano -p UDP);$ue=$LASTEXITCODE
 if($te -ne 0 -or $ue -ne 0 -or $tcp.Count -lt 3 -or $udp.Count -lt 3){throw 'endpoint inventory unavailable'}
 $tcp|Set-Content -LiteralPath (Join-Path $root ('logs/'+$label+'-tcp.txt'));$udp|Set-Content -LiteralPath (Join-Path $root ('logs/'+$label+'-udp.txt'))
 $processes=@(Get-CimInstance Win32_Process -ErrorAction Stop|Select-Object Name,ProcessId,ParentProcessId,ExecutablePath,CommandLine,CreationDate,SessionId)
 if($processes.Count -lt 10){throw 'process inventory unavailable'}
 Save-Json $processes (Join-Path $root ('logs/'+$label+'-processes.json'))
 [ordered]@{recorded_utc=[DateTime]::UtcNow.ToString('o');tcp_inventory_lines=$tcp.Count;udp_inventory_lines=$udp.Count;process_inventory_count=$processes.Count;tcp_exit=$te;udp_exit=$ue;assigned_tcp_listeners=@($tcp|Where-Object {$_ -match '^\s*TCP\s+\S+:(2508[0-9]|2509[0-9]|25241)\s+\S+\s+LISTENING\s+'});assigned_udp_endpoints=@($udp|Where-Object {$_ -match '^\s*UDP\s+\S+:(2508[0-9]|2509[0-9]|25241)\s+'});fixture_processes=@($processes|Where-Object {$_.ExecutablePath -and $_.ExecutablePath.StartsWith($root+'\',[StringComparison]::OrdinalIgnoreCase)});build_processes=@($processes|Where-Object {$_.Name -in @('cargo.exe','rustc.exe','cargo-clippy.exe','clippy-driver.exe')});sharing_processes=@($processes|Where-Object {$_.Name -match '^(synergy|deskflow|openergy)'})}
}
function Tools {
 $names=@('cargo.exe','rustc.exe','cargo-clippy.exe','clippy-driver.exe','rustfmt.exe')
 $tools=@(foreach($n in $names){$f=Get-Item -LiteralPath (Join-Path $toolRoot $n);[ordered]@{name=$n;bytes=$f.Length;sha256=(Hash $f.FullName)}})
 $settings=Join-Path $env:USERPROFILE '.rustup/settings.toml'
 $configs=@(foreach($p in @((Join-Path $env:USERPROFILE '.cargo/config'),(Join-Path $env:USERPROFILE '.cargo/config.toml'))){if(Test-Path -LiteralPath $p){[ordered]@{path=$p;sha256=(Hash $p)}}})
 $list=@(& rustup.exe toolchain list);if($LASTEXITCODE -ne 0){throw 'rustup list failed'}
 [ordered]@{tools=$tools;rustup_settings_sha256=(Hash $settings);cargo_configs=$configs;toolchains=$list;rustc=@(& (Join-Path $toolRoot 'rustc.exe') -vV);cargo=@(& (Join-Path $toolRoot 'cargo.exe') -V);powershell=$PSVersionTable.PSVersion.ToString();powershell_path=(Get-Process -Id $PID).Path;powershell_sha256=(Hash (Get-Process -Id $PID).Path);os=[Environment]::OSVersion.ToString();architecture=$env:PROCESSOR_ARCHITECTURE}
}
