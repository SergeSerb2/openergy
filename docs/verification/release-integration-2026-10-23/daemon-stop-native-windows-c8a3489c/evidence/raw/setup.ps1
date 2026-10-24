$ErrorActionPreference='Stop';$ProgressPreference='SilentlyContinue'
(Get-Process -Id $PID).PriorityClass='BelowNormal'
if((Get-Process -Id $PID).SessionId -ne 0){throw 'session 0 required'}
if(@(Get-Process -Name cargo,rustc,cargo-clippy,clippy-driver -ErrorAction SilentlyContinue).Count){throw 'another native build active'}
foreach($p in @('C:\Program Files\PowerShell\7\pwsh.exe',(Join-Path $env:USERPROFILE '.rustup/toolchains/1.99.1-x86_64-pc-windows-msvc/bin/cargo.exe'),'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64\link.exe','C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\um\x64\kernel32.lib')){if(-not (Test-Path -LiteralPath $p -PathType Leaf)){throw 'installed prerequisite absent'}}
$root=Join-Path $env:TEMP 'oe-stop-win-c8a3489c-e1ef124-24ce90f8'
if(Test-Path -LiteralPath $root){throw 'fresh owned root exists; no retry'}
New-Item -ItemType Directory -Path $root|Out-Null
$sid=[Security.Principal.WindowsIdentity]::GetCurrent().User
$acl=[Security.AccessControl.DirectorySecurity]::new();$acl.SetAccessRuleProtection($true,$false);$acl.SetOwner($sid)
$acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new($sid,'FullControl','ContainerInherit,ObjectInherit','None','Allow'))
Set-Acl -LiteralPath $root -AclObject $acl
$a=Get-Acl -LiteralPath $root
if(-not $a.AreAccessRulesProtected -or $a.Owner -ne $sid.Translate([Security.Principal.NTAccount]).Value -or @($a.Access).Count -ne 1){throw 'protected current-user-owned ACL required'}
[ordered]@{root=$root;original_temp=$env:TEMP;profile=$env:USERPROFILE;session=(Get-Process -Id $PID).SessionId;priority=(Get-Process -Id $PID).PriorityClass.ToString();created_utc=[DateTime]::UtcNow.ToString('o');root_acl_protected=$a.AreAccessRulesProtected}|ConvertTo-Json -Compress
