import json,pathlib,sys
p=pathlib.Path(json.loads(pathlib.Path('/private/tmp/oe-historical-refusal-pointer.json').read_text())['local']);sys.path.insert(0,str(p));from transport import remote
state=json.loads((p/'state.json').read_text());script="$ErrorActionPreference='Stop';& '"+state['root']+"\\run.ps1'"
r,out=remote('one-historical-refusal-attached',script,timeout=340,exe='pwsh.exe');print(json.dumps(r),flush=True);print(out.replace(state['root'],'<HISTORICAL_FIXTURE>')[:1800],flush=True)
