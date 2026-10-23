import pathlib,json,sys,subprocess,tarfile
p=pathlib.Path(json.loads(pathlib.Path('/private/tmp/oe-win-dns-8281eb7-private-pointer.json').read_text())['local']);sys.path.insert(0,str(p));from transport import *
s=json.loads((p/'state.json').read_text());dest=s['root'].replace('\\','/')
script="$root='"+s['root']+r"';$ErrorActionPreference='Stop';& (Join-Path $root 'final-audit.ps1');$items=@(Get-ChildItem $root -File|Where-Object {$_.Extension -in @('.json','.ps1')}|Select-Object -ExpandProperty Name)+@('logs','runtime','bin');& tar.exe -cf (Join-Path $root 'raw-evidence.tar') -C $root @items;if($LASTEXITCODE -ne 0){throw 'evidence archive failed'};[ordered]@{sha256=(Get-FileHash (Join-Path $root 'raw-evidence.tar') -Algorithm SHA256).Hash.ToLower();bytes=(Get-Item (Join-Path $root 'raw-evidence.tar')).Length}|ConvertTo-Json"
r,out=remote('final-postflight-attached',script,timeout=360,exe='pwsh.exe');print(json.dumps(r));print(out[:2500]);assert r['transport_exit']==0
bundle=json.loads(out[out.find('{'):]);(p/'bundle.json').write_text(json.dumps(bundle,indent=2)+'\n')
c=subprocess.run(['scp','-O','-q','-o','BatchMode=yes','-o','ConnectTimeout=8','<SSH_ALIAS>:'+dest+'/raw-evidence.tar',str(p/'raw-evidence.tar')],stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=55);(p/'download.stdout').write_bytes(c.stdout);(p/'download.stderr').write_bytes(c.stderr);assert c.returncode==0;assert sha(p/'raw-evidence.tar')==bundle['sha256']
raw=p/'raw';raw.mkdir(exist_ok=False)
with tarfile.open(p/'raw-evidence.tar') as t:
 for m in t:
  assert m.isfile() or m.isdir();assert not m.name.startswith('/') and '..' not in pathlib.PurePosixPath(m.name).parts
 t.extractall(raw)
print('Full postflight complete; raw originals downloaded with verified bundle hash.')
