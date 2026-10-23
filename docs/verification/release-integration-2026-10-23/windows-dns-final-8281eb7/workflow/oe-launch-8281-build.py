import pathlib,json,hashlib,subprocess,sys
ptr=json.loads(pathlib.Path('/private/tmp/oe-win-dns-8281eb7-private-pointer.json').read_text());p=pathlib.Path(ptr['local']);sys.path.insert(0,str(p));from transport import *
b=p/'build.ps1';s=b.read_text().replace(". (Join-Path $PSScriptRoot 'audit.ps1')", ". (Join-Path $PSScriptRoot 'runtime-common.ps1')",1);b.write_text(s)
state=json.loads((p/'state.json').read_text());dest=state['root'].replace('\\','/')
inputs=['source.tar','source-manifest.json','common.ps1','audit.ps1','build.ps1','runtime-common.ps1','run.ps1','postflight.ps1','state-input.json','native-tool-reference.json']
(p/'final-input-hashes.json').write_text(json.dumps([{'path':f,'sha256':sha(p/f)} for f in inputs],indent=2)+'\n')
files=inputs+['final-input-hashes.json']
c=subprocess.run(['scp','-O','-q','-o','BatchMode=yes','-o','ConnectTimeout=8']+[str(p/f) for f in files]+['<SSH_ALIAS>:'+dest+'/'],stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=55)
(p/'transfer-build.stdout').write_bytes(c.stdout);(p/'transfer-build.stderr').write_bytes(c.stderr);assert c.returncode==0
script="$root='"+state['root']+r"''" if False else "$root='"+state['root']+r"';$ErrorActionPreference='Stop';foreach($e in @(Get-Content -Raw (Join-Path $root 'final-input-hashes.json')|ConvertFrom-Json)){if((Get-FileHash (Join-Path $root $e.path) -Algorithm SHA256).Hash -ne $e.sha256){throw 'input transfer hash mismatch'};(Get-Item (Join-Path $root $e.path)).IsReadOnly=$true};(Get-Item (Join-Path $root 'final-input-hashes.json')).IsReadOnly=$true;foreach($name in @('common.ps1','audit.ps1','build.ps1','runtime-common.ps1','run.ps1','postflight.ps1')){$tokens=$null;$errors=$null;$null=[System.Management.Automation.Language.Parser]::ParseFile((Join-Path $root $name),[ref]$tokens,[ref]$errors);if($errors.Count -gt 0){throw ($errors|Out-String)}};& (Join-Path $root 'build.ps1')"
r,out=remote('build-attached',script,timeout=1400,exe='pwsh.exe');print(json.dumps(r));print(out.replace(state['root'],'<DNS_FIXTURE>')[:2500]);assert r['transport_exit']==0
