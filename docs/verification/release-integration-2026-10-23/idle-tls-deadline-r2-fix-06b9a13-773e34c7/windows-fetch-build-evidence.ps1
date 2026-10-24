$ErrorActionPreference='Stop';$ProgressPreference='SilentlyContinue'
$root=Join-Path $env:TEMP 'oe-idle-tls-r2-build-773e34c7-06b9a13'
$names=@('build-once.guard','build-start.json','build-result.json','build.stdout','build.stderr','shared-inputs-before.json','shared-inputs-after.json')
$files=@(foreach($n in $names){$p=Join-Path $root $n;[ordered]@{name=$n;exists=(Test-Path -LiteralPath $p -PathType Leaf);base64=$(if(Test-Path -LiteralPath $p -PathType Leaf){[Convert]::ToBase64String([IO.File]::ReadAllBytes($p))}else{$null})}})
$a=Get-Acl -LiteralPath $root
[ordered]@{files=$files;root_acl_protected=$a.AreAccessRulesProtected;root_acl_rule_count=@($a.Access).Count;root_acl_current_user_only=(@($a.Access).Count -eq 1 -and $a.Access[0].IdentityReference.Translate([Security.Principal.SecurityIdentifier]).Value -eq [Security.Principal.WindowsIdentity]::GetCurrent().User.Value);binary_execution_requested=$false}|ConvertTo-Json -Depth 8 -Compress
