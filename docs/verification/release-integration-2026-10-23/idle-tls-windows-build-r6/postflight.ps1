. (Join-Path $PSScriptRoot 'common.ps1')
Save 'source-after.json' (Assert-Source)
$after=Shared;Save 'shared-after.json' $after
$before=Get-Content -Raw -LiteralPath (Join-Path $root 'shared-before.json')|ConvertFrom-Json -AsHashtable
$same=($before|ConvertTo-Json -Depth 30 -Compress) -ceq ($after|ConvertTo-Json -Depth 30 -Compress)
$a=Get-Acl -LiteralPath $root;$sid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value
$items=@((Get-Item -LiteralPath $root))+@(Get-ChildItem -LiteralPath $root -Recurse -Force)
$bad=@(foreach($i in $items){$acl=Get-Acl -LiteralPath $i.FullName;$rules=@($acl.Access);if(($i.Attributes -band [IO.FileAttributes]::ReparsePoint) -or $rules.Count -ne 1 -or $rules[0].AccessControlType -ne 'Allow' -or $rules[0].IdentityReference.Translate([Security.Principal.SecurityIdentifier]).Value -ne $sid){$i.FullName}})
$owned=Owned-Processes
$samples=@(foreach($n in @('clippy','release')){$p=Join-Path $root ($n+'-processes.json');if(Test-Path -LiteralPath $p){Get-Content -Raw -LiteralPath $p|ConvertFrom-Json}})
$names=@($samples|Select-Object -ExpandProperty name -Unique)
$priorityBad=@($samples|Where-Object {$_.priority -ne 'BelowNormal' -or $_.session -ne 0})
$actualProof=@(foreach($n in @('cargo.exe','rustc.exe','link.exe')){[ordered]@{name=$n;observed=($n -in $names);samples=@($samples|Where-Object {$_.name -eq $n}).Count}})
$record=[ordered]@{recorded_utc=[DateTime]::UtcNow.ToString('o');shared_inputs_equal=$same;shared_trees=@($after.trees|ForEach-Object {[ordered]@{label=$_.label;files=$_.files;items=$_.items}});defaults_equal=$same;sharing_identity_equal=$same;old_failed_root_absent=$after.old_failed_root_absent;root_acl_protected=$a.AreAccessRulesProtected;acl_items=$items.Count;invalid_acl_or_reparse=$bad;owned_build_processes=$owned;process_observations=$samples.Count;bad_priority_or_session=$priorityBad;actual_process_proof=$actualProof;temp_entries=@(Get-ChildItem -LiteralPath (Join-Path $root 'temp') -Recurse -Force).Count;runtime_invocations=0;runtime_or_deadline_acceptance_claimed=$false}
$record.passed=($same -and $a.AreAccessRulesProtected -and $bad.Count -eq 0 -and $owned.Count -eq 0 -and $priorityBad.Count -eq 0 -and @($actualProof|Where-Object {-not $_.observed}).Count -eq 0)
Save 'postflight.json' $record
$files=@(Get-ChildItem -LiteralPath $root -File|Where-Object {$_.Extension -in @('.json','.stdout','.stderr','.guard') -and $_.Name -notin @('state.json','transfer-manifest.json')})
$manifest=@($files|ForEach-Object {[ordered]@{path=$_.Name;bytes=$_.Length;sha256=(Hash $_.FullName)}})
Save 'evidence-manifest.json' $manifest
$names=@($files.Name)+@('evidence-manifest.json')
if(Test-Path -LiteralPath (Join-Path $root 'target/x86_64-pc-windows-msvc/release/examples/dns_queue_cancel_c4806a3d.exe')){$names+=@('target/x86_64-pc-windows-msvc/release/examples/dns_queue_cancel_c4806a3d.exe')}
& (Join-Path $env:WINDIR 'System32/tar.exe') -cf (Join-Path $root 'evidence.tar') -C $root @names
if($LASTEXITCODE -ne 0){throw 'evidence packaging failed'}
$bundle=Get-Item -LiteralPath (Join-Path $root 'evidence.tar')
[ordered]@{bytes=$bundle.Length;sha256=(Hash $bundle.FullName);postflight_passed=$record.passed}|ConvertTo-Json -Compress
if(-not $record.passed){exit 1}
