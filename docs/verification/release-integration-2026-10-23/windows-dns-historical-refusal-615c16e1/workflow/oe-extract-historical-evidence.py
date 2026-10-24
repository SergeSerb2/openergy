import hashlib,json,pathlib,tarfile
p=pathlib.Path(json.loads(pathlib.Path('/private/tmp/oe-historical-refusal-pointer.json').read_text())['local']);bundle=json.loads((p/'bundle.json').read_text());archive=p/'raw-evidence.tar';assert hashlib.sha256(archive.read_bytes()).hexdigest()==bundle['sha256'];raw=p/'raw';assert raw.is_dir() and not list(raw.iterdir())
error='Traceback (most recent call last):\n  File "/private/tmp/oe-finish-historical-refusal.py", line 15, in <module>\n    tar.extractall(p/\'raw\',filter=\'data\')\nTypeError: extractall() got an unexpected keyword argument \'filter\'\n'
(p/'local-extraction-first-failure.stderr').write_text(error)
(p/'local-extraction-first-failure.json').write_text(json.dumps({'exit':1,'phase':'local evidence archive extraction after successful postflight and verified download','error':"TypeError: extractall() got an unexpected keyword argument 'filter'",'original_script_sha256':hashlib.sha256((p/'oe-finish-historical-refusal.py').read_bytes()).hexdigest(),'runtime_retried':False,'remote_audit_retried':False,'recovery':'Explicitly validate every archive member as a relative regular file or directory, then extract with the installed Python API.'},indent=2)+'\n')
with tarfile.open(archive) as tar:
 for member in tar:
  path=pathlib.PurePosixPath(member.name)
  assert not path.is_absolute() and '..' not in path.parts and (member.isfile() or member.isdir()) and '\\' not in member.name
  target=raw.joinpath(*path.parts)
  if member.isdir():target.mkdir(parents=True,exist_ok=True)
  else:
   target.parent.mkdir(parents=True,exist_ok=True);assert not target.exists();data=tar.extractfile(member).read();assert len(data)==member.size;target.write_bytes(data)
(p/'local-extraction-recovery.json').write_text(json.dumps({'verified_bundle_sha256':bundle['sha256'],'explicit_archive_member_validation':True,'extracted_files':sum(x.is_file() for x in raw.rglob('*')),'remote_or_product_command_invocations':0},indent=2)+'\n')
(p/'oe-extract-historical-evidence.py').write_bytes(pathlib.Path(__file__).read_bytes())
print(json.dumps({'raw_bundle_verified':True,'extraction_complete':True,**bundle}))
