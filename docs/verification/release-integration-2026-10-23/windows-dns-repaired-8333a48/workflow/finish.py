from transport import *
p=ROOT/'runtime-common.ps1';(ROOT/'runtime-common.unexecuted.ps1').write_text(p.read_text());p.write_text(p.read_text().split('function New-ConflictSocket')[0])
(ROOT/'native-tool-reference.json').write_bytes(pathlib.Path('<LOCAL_PRIOR_OBSERVED>/native-tools-final.json').read_bytes())
s=json.loads((ROOT/'state.json').read_text());dest=s['root'].replace('\\','/')
files=['runtime-common.ps1','finish-stopped.ps1','native-tool-reference.json','r9-stop-instruction.json']
p=subprocess.run(['scp','-O','-q','-o','BatchMode=yes','-o','ConnectTimeout=8']+[str(ROOT/f) for f in files]+['<SSH_ALIAS>:'+dest+'/'],stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=55);(ROOT/'transfer-finish.stderr').write_bytes(p.stderr);assert p.returncode==0
inputs=['source.tar','source-manifest.json','common.ps1','audit.ps1','build.ps1','state-input.json']+files
(ROOT/'final-input-hashes.json').write_text(json.dumps([{'path':f,'sha256':sha(ROOT/f)} for f in inputs],indent=2)+'\n')
p=subprocess.run(['scp','-O','-q','-o','BatchMode=yes','-o','ConnectTimeout=8',str(ROOT/'final-input-hashes.json'),'<SSH_ALIAS>:'+dest+'/'],stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=55);(ROOT/'transfer-final-hashes.stderr').write_bytes(p.stderr);assert p.returncode==0
script="$root='"+s['root']+r"';$ErrorActionPreference='Stop';foreach($e in @(Get-Content -Raw (Join-Path $root 'final-input-hashes.json')|ConvertFrom-Json)){if((Get-FileHash (Join-Path $root $e.path) -Algorithm SHA256).Hash -ne $e.sha256){throw 'final input transfer mismatch'};(Get-Item (Join-Path $root $e.path)).IsReadOnly=$true};foreach($name in @('runtime-common.ps1','finish-stopped.ps1')){$tokens=$null;$errors=$null;$null=[System.Management.Automation.Language.Parser]::ParseFile((Join-Path $root $name),[ref]$tokens,[ref]$errors);if($errors.Count -gt 0){throw ($errors|Out-String)}};& (Join-Path $root 'finish-stopped.ps1');$items=@(Get-ChildItem $root -File|Where-Object {$_.Extension -in @('.json','.ps1')}|Select-Object -ExpandProperty Name)+@('logs','runtime');& tar.exe -cf (Join-Path $root 'raw-evidence.tar') -C $root @items;if($LASTEXITCODE -ne 0){throw 'evidence archive failed'};[ordered]@{sha256=(Get-FileHash (Join-Path $root 'raw-evidence.tar') -Algorithm SHA256).Hash.ToLower();bytes=(Get-Item (Join-Path $root 'raw-evidence.tar')).Length}|ConvertTo-Json"
r,out=remote('stopped-postflight-attached',script,timeout=300,exe='pwsh.exe');print(r)
if r['transport_exit']!=0:print(out[:500]);raise SystemExit(1)
bundle=json.loads(out[out.find('{'):]);(ROOT/'bundle.json').write_text(json.dumps(bundle,indent=2)+'\n');print(bundle)
p=subprocess.run(['scp','-O','-q','-o','BatchMode=yes','-o','ConnectTimeout=8','<SSH_ALIAS>:'+dest+'/raw-evidence.tar',str(ROOT/'raw-evidence.tar')],stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=55);(ROOT/'download.stderr').write_bytes(p.stderr);assert p.returncode==0;assert sha(ROOT/'raw-evidence.tar')==bundle['sha256']
import tarfile
raw=ROOT/'raw';raw.mkdir(exist_ok=False)
with tarfile.open(ROOT/'raw-evidence.tar') as t:
 for m in t:
  assert m.isfile() or m.isdir();assert not m.name.startswith('/') and '..' not in pathlib.PurePosixPath(m.name).parts
 t.extractall(raw)
print('Stopped runtime audit complete; raw originals downloaded and hash verified.')
