param([ValidateSet('before','after')][string]$Phase)
. (Join-Path $PSScriptRoot 'common.ps1')
$vswhere=Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
$vs=& $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if($LASTEXITCODE -ne 0 -or -not $vs){throw 'existing MSVC installation not found'}
$version=(Get-Content -Raw -LiteralPath (Join-Path $vs 'VC/Auxiliary/Build/Microsoft.VCToolsVersion.default.txt')).Trim()
$native=Join-Path $vs ('VC/Tools/MSVC/'+$version+'/bin/Hostx64/x64')
$paths=@($vswhere)+@('cl.exe','link.exe','lib.exe','c1.dll','c1xx.dll','c2.dll','mspdbcore.dll'|ForEach-Object {Join-Path $native $_})+@('tar.exe','netstat.exe','rustup.exe','pwsh.exe','powershell.exe','ssh.exe'|ForEach-Object {(Get-Command $_ -ErrorAction Stop).Source})
$records=@(foreach($p in $paths){$f=Get-Item -LiteralPath $p;[ordered]@{path=$f.FullName;name=$f.Name;bytes=$f.Length;sha256=(Hash $f.FullName);file_version=$f.VersionInfo.FileVersion}})
$registry=Join-Path $env:USERPROFILE '.cargo/registry/src'
$checksums=@(Get-ChildItem -LiteralPath $registry -Filter '.cargo-checksum.json' -File -Recurse|ForEach-Object {[ordered]@{path=$_.FullName;bytes=$_.Length;sha256=(Hash $_.FullName)}})
$record=[ordered]@{recorded_utc=[DateTime]::UtcNow.ToString('o');phase=$Phase;msvc_version=$version;tools=$records;registry_checksum_manifests=$checksums;scope='Tool versions/hashes, installed cached crate checksum manifests; no tool installation or default mutation.'}
Save-Json $record (Join-Path $root ('tool-audit-'+$Phase+'.json'))
if($Phase -eq 'before'){Save-Json ([ordered]@{fixture=$root;userprofile=$env:USERPROFILE;localappdata=$env:LOCALAPPDATA;temp_original=[Environment]::GetEnvironmentVariable('TEMP','User');computername=$env:COMPUTERNAME;username=$env:USERNAME;sid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value}) (Join-Path $root 'private-identity.json')}
if($Phase -eq 'after'){$before=Get-Content -Raw -LiteralPath (Join-Path $root 'tool-audit-before.json')|ConvertFrom-Json;if(($before.tools|ConvertTo-Json -Compress) -ne ($records|ConvertTo-Json -Compress) -or ($before.registry_checksum_manifests|ConvertTo-Json -Compress) -ne ($checksums|ConvertTo-Json -Compress)){throw 'native tools or cached crate checksum manifests changed'}}
Write-Output 'TOOL_AUDIT_PASS'
