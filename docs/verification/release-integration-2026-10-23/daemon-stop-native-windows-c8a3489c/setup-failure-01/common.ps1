$ErrorActionPreference='Stop'
$ProgressPreference='SilentlyContinue'
$PSNativeCommandUseErrorActionPreference=$false
$root=$PSScriptRoot
(Get-Process -Id $PID).PriorityClass='BelowNormal'
if((Get-Process -Id $PID).SessionId -ne 0){throw 'session 0 required'}
$state=Get-Content -Raw -LiteralPath (Join-Path $root 'state.json')|ConvertFrom-Json
$toolRoot=Join-Path $env:USERPROFILE '.rustup/toolchains/1.99.1-x86_64-pc-windows-msvc/bin'
$registry=Join-Path $env:USERPROFILE '.cargo/registry'
$msvc='C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207'
$sdk='C:\Program Files (x86)\Windows Kits\10'
$sdkVersion='10.0.26100.0'
function Save($name,$value){[IO.File]::WriteAllText((Join-Path $root $name),(ConvertTo-Json -InputObject $value -Depth 30),[Text.UTF8Encoding]::new($false))}
function Hash($p){(Get-FileHash -LiteralPath $p -Algorithm SHA256).Hash.ToLower()}
function Digest($s){$h=[Security.Cryptography.SHA256]::Create();try{[Convert]::ToHexString($h.ComputeHash([Text.Encoding]::UTF8.GetBytes($s))).ToLower()}finally{$h.Dispose()}}
function Snapshot($label,$path){
 $items=@((Get-Item -LiteralPath $path -Force))+@(Get-ChildItem -LiteralPath $path -Force -Recurse)
 $entries=@(foreach($f in $items|Sort-Object FullName){
  if($f.Attributes -band [IO.FileAttributes]::ReparsePoint){throw ('reparse point refused: '+$label)}
  $relative=if($f.FullName -eq $path){'.'}else{$f.FullName.Substring($path.Length+1).Replace('\','/')}
  $e=[ordered]@{path=$relative;directory=$f.PSIsContainer;readonly=[bool]($f.Attributes -band [IO.FileAttributes]::ReadOnly);acl_sha256=(Digest (Get-Acl -LiteralPath $f.FullName).Sddl)}
  if(-not $f.PSIsContainer){$e.bytes=$f.Length;$e.sha256=Hash $f.FullName};$e
 })
 [ordered]@{label=$label;root=$path;files=@($entries|Where-Object {-not $_.directory}).Count;items=$entries.Count;entries=$entries}
}
function Shared {
 $trees=@(foreach($f in $state.prior_fixtures){Snapshot $f.label $f.root})
 $trees+=@(Snapshot 'original-registry' $registry)
 $trees+=@(Snapshot 'pinned-toolchain' (Split-Path $toolRoot -Parent))
 $names=@('.rustup/settings.toml','.cargo/config','.cargo/config.toml','.cargo/.global-cache','.cargo/.global-cache-wal','.cargo/.global-cache-shm','.cargo/.package-cache','.cargo/.package-cache-mutate')
 $defaults=@(foreach($n in $names){$p=Join-Path $env:USERPROFILE $n;if(Test-Path -LiteralPath $p){$f=Get-Item -LiteralPath $p;[ordered]@{path=$n;exists=$true;bytes=$f.Length;readonly=$f.IsReadOnly;sha256=(Hash $p);acl_sha256=(Digest (Get-Acl -LiteralPath $p).Sddl)}}else{[ordered]@{path=$n;exists=$false}}})
 $paths=@(foreach($n in @('cargo.exe','rustc.exe','cargo-clippy.exe','clippy-driver.exe','rustdoc.exe')){Join-Path $toolRoot $n})
 $paths+=@(foreach($n in @('cl.exe','link.exe','lib.exe','c1.dll','c1xx.dll','c2.dll','mspdbcore.dll')){Join-Path $msvc ('bin/Hostx64/x64/'+$n)})
 $paths+=@((Join-Path $sdk ('bin/'+$sdkVersion+'/x64/rc.exe')),(Join-Path $sdk ('bin/'+$sdkVersion+'/x64/mt.exe')),(Join-Path $env:WINDIR 'System32/tar.exe'),(Get-Process -Id $PID).Path)
 $tools=@(foreach($p in $paths){$f=Get-Item -LiteralPath $p;[ordered]@{path=$p;bytes=$f.Length;sha256=(Hash $p);acl_sha256=(Digest (Get-Acl -LiteralPath $p).Sddl)}})
 $sharing=@(Get-CimInstance Win32_Process -Filter "Name LIKE 'synergy%' OR Name LIKE 'deskflow%' OR Name LIKE 'openergy%'"|Sort-Object ProcessId|Select-Object Name,ProcessId,CreationDate,ExecutablePath,SessionId)
 [ordered]@{trees=$trees;defaults=$defaults;tools=$tools;sharing=$sharing;old_failed_root_absent=(-not (Test-Path -LiteralPath (Join-Path $state.original_temp 'oe-idle-tls-r2-build-773e34c7-06b9a13')))}
}
function Assert-Source {
 $manifest=Get-Content -Raw -LiteralPath (Join-Path $root 'source-manifest.json')|ConvertFrom-Json
 $source=Join-Path $root 'source'
 $files=@(Get-ChildItem -LiteralPath $source -Recurse -Force -File)
 if($files.Count -ne $manifest.files.Count){throw 'source file count mismatch'}
 $actual=@(foreach($e in $manifest.files){$f=Get-Item -LiteralPath (Join-Path $source $e.path);$h=Hash $f.FullName;if($f.Length -ne $e.bytes -or $h -ne $e.sha256 -or -not $f.IsReadOnly){throw ('source bytes/readonly mismatch: '+$e.path)};$b=[IO.File]::ReadAllBytes($f.FullName);$lf=0;$crlf=0;for($j=0;$j -lt $b.Length;$j++){if($b[$j] -eq 10){$lf++;if($j -gt 0 -and $b[$j-1] -eq 13){$crlf++}}};if($lf -ne $e.lf_bytes -or $crlf -ne $e.crlf_pairs){throw 'line endings changed'};[ordered]@{path=$e.path;bytes=$f.Length;sha256=$h;readonly=$f.IsReadOnly;crlf_pairs=$crlf;lf_bytes=$lf}})
 [ordered]@{source_commit=$manifest.source_commit;files=$files.Count;files_match=$true;actual=$actual;archive_sha256=(Hash (Join-Path $root 'source.tar'));manifest_sha256=(Hash (Join-Path $root 'source-manifest.json'));module_sha256=(Hash (Join-Path $source 'crates/net/examples/dns_queue_cancel_c4806a3d/idle_tls_deadline.rs'))}
}
function Owned-Processes {
 @(Get-CimInstance Win32_Process|Where-Object {($_.ExecutablePath -and $_.ExecutablePath.StartsWith($root+'\',[StringComparison]::OrdinalIgnoreCase)) -or ($_.CommandLine -and $_.CommandLine.Contains($root) -and $_.ProcessId -ne $PID)}|Select-Object Name,ProcessId,ParentProcessId,CreationDate,SessionId,ExecutablePath,CommandLine)
}

function Identity($process){
 $null=$process.SafeHandle
 $cim=Get-CimInstance Win32_Process -Filter ('ProcessId='+$process.Id)
 [ordered]@{pid=$process.Id;start_utc=$process.StartTime.ToUniversalTime().ToString('o');creation_utc=$cim.CreationDate.ToUniversalTime().ToString('o');parent_pid=$cim.ParentProcessId;path=$process.Path;command_line=$cim.CommandLine;session=$process.SessionId;priority=$process.PriorityClass.ToString();retained_handle=$true;recorded_utc=[DateTime]::UtcNow.ToString('o')}
}
function Ancestors($process){
 $items=[Collections.Generic.List[object]]::new();$p=$process
 for($j=0;$j -lt 8;$j++){$id=Identity $p;$items.Add($id);if(-not $id.parent_pid){break};$p=Get-Process -Id $id.parent_pid -ErrorAction SilentlyContinue;if(-not $p){break}}
 $items.ToArray()
}
