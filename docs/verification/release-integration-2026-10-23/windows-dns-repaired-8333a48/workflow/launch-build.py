from transport import *
s=json.loads((ROOT/'state.json').read_text());dest=s['root'].replace('\\','/')
files=['source.tar','source-manifest.json','common.ps1','audit.ps1','build.ps1','state-input.json']
(ROOT/'build-input-hashes.json').write_text(json.dumps([{'path':f,'sha256':sha(ROOT/f)} for f in files],indent=2)+'\n')
p=subprocess.run(['scp','-O','-q','-o','BatchMode=yes','-o','ConnectTimeout=8']+[str(ROOT/f) for f in files+['build-input-hashes.json']]+['<SSH_ALIAS>:'+dest+'/'],stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=55);(ROOT/'transfer-build.stderr').write_bytes(p.stderr);assert p.returncode==0
script="$root='"+s['root']+r"';$ErrorActionPreference='Stop';foreach($e in @(Get-Content -Raw (Join-Path $root 'build-input-hashes.json')|ConvertFrom-Json)){if((Get-FileHash (Join-Path $root $e.path) -Algorithm SHA256).Hash -ne $e.sha256){throw 'build input transfer mismatch'}};$tokens=$null;$errors=$null;$null=[System.Management.Automation.Language.Parser]::ParseFile((Join-Path $root 'build.ps1'),[ref]$tokens,[ref]$errors);if($errors.Count -gt 0){throw ($errors|Out-String)};& (Join-Path $root 'build.ps1')"
r,out=remote('build-attached',script,timeout=900,exe='pwsh.exe');print(json.dumps(r));print(out.replace(s['root'],'<DNS_FIXTURE>')[:2000])
