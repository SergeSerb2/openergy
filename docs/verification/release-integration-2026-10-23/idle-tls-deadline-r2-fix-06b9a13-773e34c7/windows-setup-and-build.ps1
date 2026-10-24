$ErrorActionPreference='Stop';$ProgressPreference='SilentlyContinue'
(Get-Process -Id $PID).PriorityClass='BelowNormal'
$root=Join-Path $env:TEMP 'oe-idle-tls-r2-build-773e34c7-06b9a13'
if(Test-Path -LiteralPath $root){throw 'owned root exists; no retry'}
$sid=[Security.Principal.WindowsIdentity]::GetCurrent().User
New-Item -ItemType Directory -Path $root|Out-Null
$acl=[Security.AccessControl.DirectorySecurity]::new();$acl.SetAccessRuleProtection($true,$false);$acl.SetOwner($sid)
$rule=[Security.AccessControl.FileSystemAccessRule]::new($sid,'FullControl','ContainerInherit,ObjectInherit','None','Allow');$acl.AddAccessRule($rule)
Set-Acl -LiteralPath $root -AclObject $acl
$bytes=[Convert]::FromBase64String([Console]::In.ReadToEnd())
$payload=Join-Path $root 'payload.tar';[IO.File]::WriteAllBytes($payload,$bytes)
if((Get-FileHash -LiteralPath $payload -Algorithm SHA256).Hash.ToLower() -ne 'bcc0e2f1f4e60001ec5b51979cb8338ca201a77d9f9285b3db8d15275a7472d3'){throw 'payload hash mismatch'}
& (Join-Path $env:WINDIR 'System32/tar.exe') -xf $payload -C $root
if($LASTEXITCODE -ne 0){throw 'payload extraction failed'}
& 'C:\Program Files\PowerShell\7\pwsh.exe' -NoLogo -NoProfile -NonInteractive -File (Join-Path $root 'build.ps1')
$buildExit=$LASTEXITCODE
Write-Output ('NATIVE_BUILD_EXIT='+$buildExit)
exit $buildExit
