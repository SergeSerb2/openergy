$ErrorActionPreference='Stop';$ProgressPreference='SilentlyContinue'
$msvc='C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64'
$native=@(foreach($n in @('cl.exe','link.exe','lib.exe','c1.dll','c2.dll')){$p=Join-Path $msvc $n;[ordered]@{name=$n;exists=(Test-Path -LiteralPath $p -PathType Leaf);sha256=$(if(Test-Path -LiteralPath $p -PathType Leaf){(Get-FileHash -LiteralPath $p -Algorithm SHA256).Hash.ToLower()}else{$null})}})
$sdkLib='C:\Program Files (x86)\Windows Kits\10\Lib'
$sdks=@(if(Test-Path -LiteralPath $sdkLib){Get-ChildItem -LiteralPath $sdkLib -Directory|ForEach-Object {[ordered]@{version=$_.Name;ucrt_x64=(Test-Path -LiteralPath (Join-Path $_.FullName 'ucrt\x64\ucrt.lib'));um_x64=(Test-Path -LiteralPath (Join-Path $_.FullName 'um\x64\kernel32.lib'))}}})
[ordered]@{read_only_preflight=$true;native_tools=$native;windows_sdks=$sdks;pwsh7_exists=(Test-Path -LiteralPath 'C:\Program Files\PowerShell\7\pwsh.exe' -PathType Leaf);tar_exists=(Test-Path -LiteralPath (Join-Path $env:WINDIR 'System32\tar.exe') -PathType Leaf)}|ConvertTo-Json -Depth 8 -Compress
