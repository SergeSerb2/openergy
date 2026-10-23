$ErrorActionPreference='Stop';$ProgressPreference='SilentlyContinue'
$tool=Join-Path $env:USERPROFILE '.rustup/toolchains/1.99.1-x86_64-pc-windows-msvc/bin/rustc.exe'
if(-not (Test-Path -LiteralPath $tool)){throw 'pinned existing Rust unavailable'}
$pwsh=(Get-Command pwsh.exe -ErrorAction Stop).Source
if((Get-Process -Id $PID).SessionId -ne 0){throw 'session 0 required'}
$sid=[Security.Principal.WindowsIdentity]::GetCurrent().User
$root=Join-Path ([IO.Path]::GetTempPath()) ('oe-win-bounded-e4f48ba-'+[guid]::NewGuid().ToString('N'))
if(Test-Path -LiteralPath $root){throw 'fixture exists'}
$acl=New-Object Security.AccessControl.DirectorySecurity
$acl.SetAccessRuleProtection($true,$false)
$rule=New-Object Security.AccessControl.FileSystemAccessRule($sid,'FullControl','ContainerInherit,ObjectInherit','None','Allow')
$acl.AddAccessRule($rule)
[IO.Directory]::CreateDirectory($root,$acl)|Out-Null
foreach($name in @('source','target','bin','runtime','temp','logs')){[IO.Directory]::CreateDirectory((Join-Path $root $name))|Out-Null}
$a=Get-Acl -LiteralPath $root
if(-not $a.AreAccessRulesProtected -or @($a.Access).Count -ne 1 -or $a.Access[0].IdentityReference.Translate([Security.Principal.SecurityIdentifier]).Value -ne $sid.Value){throw 'fixture ACL mismatch'}
[ordered]@{root=$root;pwsh=$pwsh;created_utc=[DateTime]::UtcNow.ToString('o');session=(Get-Process -Id $PID).SessionId;protected=$a.AreAccessRulesProtected;current_user_only=$true}|ConvertTo-Json
