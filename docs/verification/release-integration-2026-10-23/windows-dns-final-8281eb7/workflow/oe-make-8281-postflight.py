import pathlib,json
p=pathlib.Path(json.loads(pathlib.Path('/private/tmp/oe-win-dns-8281eb7-private-pointer.json').read_text())['local'])
(p/'final-audit.ps1').write_text(r'''. (Join-Path $PSScriptRoot 'runtime-common.ps1')
Set-Location -LiteralPath $root
$before=Get-Content -Raw (Join-Path $root 'native-tools-before.json')|ConvertFrom-Json
$after=@(foreach($e in $before.native_tools){$f=Get-Item -LiteralPath $e.path;if($f.Length -ne $e.bytes -or (Hash $f.FullName) -ne $e.sha256){throw 'installed native tool changed since fresh prebuild sample'};[ordered]@{name=$e.name;path=$e.path;bytes=$f.Length;sha256=(Hash $f.FullName);file_version=$f.VersionInfo.FileVersion}})
Save-Json ([ordered]@{recorded_utc=[DateTime]::UtcNow.ToString('o');native_tools=$after;fresh_prebuild_hashes_unchanged=$true}) (Join-Path $root 'native-tools-after.json')
$build=Get-Content -Raw (Join-Path $root 'build-start.json')|ConvertFrom-Json
$tools=Tools
if(($tools.tools|ConvertTo-Json -Compress) -ne ($build.tools_before.tools|ConvertTo-Json -Compress) -or $tools.rustup_settings_sha256 -ne $build.tools_before.rustup_settings_sha256 -or ($tools.toolchains -join "`n") -ne ($build.tools_before.toolchains -join "`n") -or ($tools.cargo_configs|ConvertTo-Json -Compress) -ne ($build.tools_before.cargo_configs|ConvertTo-Json -Compress)){throw 'tool/default/config changed since before build'}
# Detect and remove only this fixture's generated TLS identity files. Record hashes first.
$identities=@(Get-ChildItem (Join-Path $root 'runtime') -File -Recurse -Force|Where-Object {$_.Name -in @('cert.pem','key.pem','.identity-lock')})
Save-Json @($identities|ForEach-Object {File-Record $_}) (Join-Path $root 'final-private-identities-removed.json')
foreach($file in $identities){Remove-Item -LiteralPath $file.FullName -Force}
$input=Get-Content -Raw (Join-Path $root 'final-audit-input-hashes.json')|ConvertFrom-Json
foreach($e in $input){$f=Get-Item -LiteralPath (Join-Path $root $e.path);if(-not $f.IsReadOnly -or (Hash $f.FullName) -ne $e.sha256){throw 'final audit workflow input changed'}}
Save-Json ([ordered]@{recorded_utc=[DateTime]::UtcNow.ToString('o');fresh_native_tools_unchanged=$after.Count;tools_defaults_unchanged_since_before_build=$true;extra_identity_files_removed=$identities.Count;final_audit_inputs_unchanged=$input.Count}) (Join-Path $root 'extra-invariance.json')
& (Join-Path $root 'postflight.ps1')
if(-not (Get-Content -Raw (Join-Path $root 'postflight.json')|ConvertFrom-Json).passed){throw 'postflight failed'}
Write-Output 'FRESH_NATIVE_TOOL_AND_FULL_POSTFLIGHT_AUDIT_PASS'
''')
(p/'source-manifest-metadata-clarification.json').write_text(json.dumps({'manifest_sha256':__import__('hashlib').sha256((p/'source-manifest.json').read_bytes()).hexdigest(),'literal_field':'included_parent','literal_value':'9a4ac2eed77a8ed8bddd61e060be14a506df570a','meaning':'Reference for the two reviewed file byte comparisons only. No ancestry or complete-tree inclusion of this reviewed commit was asserted or verified. The full source tree is exact 8281eb79f54725ddc8fe8bae83dbd0f44fa5d885.','unchanged_frozen_manifest':True},indent=2)+'\n')
print('Prepared final audit; source-manifest metadata label clarified without changing frozen input.')
