import ast,hashlib,json,pathlib,subprocess,sys,tarfile
p=pathlib.Path(json.loads(pathlib.Path('/private/tmp/oe-historical-refusal-pointer.json').read_text())['local']);sys.path.insert(0,str(p));from transport import remote,sha
state=json.loads((p/'state.json').read_text());root=state['root'];dest=root.replace('\\','/')
script="$ErrorActionPreference='Stop';& '"+root+"\\postflight.ps1'"
r,out=remote('postflight-attached',script,timeout=700,exe='pwsh.exe');print(json.dumps(r),flush=True);print(out.replace(root,'<HISTORICAL_FIXTURE>')[:1800],flush=True)
# Always preserve raw files even if an audit reports failure. No second runtime command.
script="$root='"+root+r"';$ErrorActionPreference='Stop';Set-Location -LiteralPath $root;$files=@(Get-ChildItem -LiteralPath $root -File -Force|Where-Object {$_.Name -notin @('raw-evidence.tar') }|Select-Object -ExpandProperty Name);$files+=@('logs','bin');& tar.exe -cf (Join-Path $root 'raw-evidence.tar') @files;if($LASTEXITCODE -ne 0){throw 'raw evidence archive failed'};$f=Get-Item (Join-Path $root 'raw-evidence.tar');[ordered]@{bytes=$f.Length;sha256=(Get-FileHash $f.FullName -Algorithm SHA256).Hash.ToLower()}|ConvertTo-Json"
r,out=remote('raw-bundle',script,timeout=55,exe='pwsh.exe');print(json.dumps(r),flush=True);assert r['transport_exit']==0;bundle=json.loads(out.lstrip('\ufeff'));(p/'bundle.json').write_text(json.dumps(bundle,indent=2)+'\n')
tree=ast.parse((p/'transport.py').read_text());alias=next(n.value.elts[5].value for n in ast.walk(tree) if isinstance(n,ast.Assign) and any(isinstance(t,ast.Name) and t.id=='argv' for t in n.targets))
c=subprocess.run(['scp','-O','-q','-o','BatchMode=yes','-o','ConnectTimeout=8',alias+':'+dest+'/raw-evidence.tar',str(p/'raw-evidence.tar')],stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=55);(p/'download.stdout').write_bytes(c.stdout);(p/'download.stderr').write_bytes(c.stderr);(p/'download.json').write_text(json.dumps({'exit':c.returncode,'bundle_sha256':sha(p/'raw-evidence.tar') if (p/'raw-evidence.tar').exists() else None},indent=2)+'\n');assert c.returncode==0;assert sha(p/'raw-evidence.tar')==bundle['sha256']
(p/'raw').mkdir(exist_ok=False)
with tarfile.open(p/'raw-evidence.tar') as tar:
 for m in tar:
  assert not m.name.startswith('/') and '..' not in pathlib.PurePosixPath(m.name).parts and not m.issym() and not m.islnk()
 tar.extractall(p/'raw',filter='data')
print(json.dumps({'raw_bundle_verified':True,**bundle}),flush=True)
