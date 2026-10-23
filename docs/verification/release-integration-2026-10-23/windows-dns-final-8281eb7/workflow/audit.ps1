. (Join-Path $PSScriptRoot 'common.ps1')
function String-Hash($s){$sha=[Security.Cryptography.SHA256]::Create();try{([BitConverter]::ToString($sha.ComputeHash([Text.Encoding]::UTF8.GetBytes($s)))).Replace('-','').ToLower()}finally{$sha.Dispose()}}
function Snapshot-Tree($label,$path){
 $items=@((Get-Item -LiteralPath $path -Force))+@(Get-ChildItem -LiteralPath $path -Recurse -Force)
 $entries=@(foreach($i in $items){
  if($i.Attributes -band [IO.FileAttributes]::ReparsePoint){throw ('reparse point in immutable input: '+$label)}
  $relative=if($i.FullName -eq $path){'.'}else{$i.FullName.Substring($path.Length+1).Replace('\','/')}
  $entry=[ordered]@{path=$relative;directory=$i.PSIsContainer;readonly=[bool]($i.Attributes -band [IO.FileAttributes]::ReadOnly);acl_sha256=(String-Hash (Get-Acl -LiteralPath $i.FullName).Sddl)}
  if(-not $i.PSIsContainer){$entry.bytes=$i.Length;$entry.sha256=Hash $i.FullName}
  $entry
 })
 [ordered]@{label=$label;root=$path;files=@($entries|Where-Object {-not $_.directory}).Count;entries=$entries}
}
function Compare-Tree($snapshot){
 $actual=Snapshot-Tree $snapshot.label $snapshot.root
 if($actual.entries.Count -ne $snapshot.entries.Count){throw ('immutable input item count changed: '+$snapshot.label)}
 $byPath=@{};foreach($e in $snapshot.entries){$byPath[$e.path]=$e}
 foreach($e in $actual.entries){$old=$byPath[$e.path];if(-not $old -or $e.directory -ne $old.directory -or $e.readonly -ne $old.readonly -or $e.acl_sha256 -ne $old.acl_sha256 -or (-not $e.directory -and ($e.bytes -ne $old.bytes -or $e.sha256 -ne $old.sha256))){throw ('immutable input changed: '+$snapshot.label+'/'+$e.path)}}
 [ordered]@{label=$snapshot.label;files=$actual.files;items=$actual.entries.Count;bytes_hashes_readonly_acls_unchanged=$true}
}
