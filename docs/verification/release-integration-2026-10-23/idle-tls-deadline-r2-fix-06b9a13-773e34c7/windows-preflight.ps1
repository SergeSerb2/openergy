$ErrorActionPreference='Stop'
$toolRoot=Join-Path $env:USERPROFILE '.rustup/toolchains/1.99.1-x86_64-pc-windows-msvc/bin'
$registry=Join-Path $env:USERPROFILE '.cargo/registry'
$tools=@(foreach($n in @('rustc.exe','cargo.exe','cargo-clippy.exe','clippy-driver.exe','rustfmt.exe')){[ordered]@{name=$n;exists=(Test-Path -LiteralPath (Join-Path $toolRoot $n) -PathType Leaf)}})
$native=@(foreach($n in @('cl.exe','link.exe','lib.exe','rc.exe')){[ordered]@{name=$n;available=($null -ne (Get-Command $n -ErrorAction SilentlyContinue))}})
$builds=@(Get-Process -Name cargo,rustc,cargo-clippy,clippy-driver -ErrorAction SilentlyContinue)
[ordered]@{windows=([Environment]::OSVersion.Platform.ToString());architecture=$env:PROCESSOR_ARCHITECTURE;session=(Get-Process -Id $PID).SessionId;tools=$tools;registry_exists=(Test-Path -LiteralPath $registry -PathType Container);registry_has_reparse_point=([bool](@(Get-ChildItem -LiteralPath $registry -Force -Recurse -Attributes ReparsePoint -ErrorAction SilentlyContinue).Count));native_path_tools=$native;active_build_process_count=$builds.Count;rustc_version=$(if($tools[0].exists){@(& (Join-Path $toolRoot 'rustc.exe') -vV)}else{@()});read_only_preflight=$true}|ConvertTo-Json -Depth 8 -Compress
