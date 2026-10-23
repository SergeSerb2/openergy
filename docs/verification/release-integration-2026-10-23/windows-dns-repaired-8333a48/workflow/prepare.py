from transport import *
import io,tarfile
old=pathlib.Path('<LOCAL_PRIOR_OBSERVED>')
commit='8333a48df046bdc7e5f0cfc687ed09f2b75b39f3'
code=(old/'package.py').read_text();code=code[:code.index('review=subprocess')];code=code.replace("commit='e6a9125be2fc59fbe92fef6ea351b222298bfc52'",f"commit='{commit}'").replace("assert subprocess.run(['git','merge-base','--is-ancestor','2253049d72f4ee5aa079045432c5014bb0c4be78',commit]).returncode==0", "assert subprocess.run(['git','merge-base','--is-ancestor','c7f6989',commit]).returncode==0").replace("'included_parent':'2253049d72f4ee5aa079045432c5014bb0c4be78'", "'included_parent':'c7f6989'")
exec(compile(code,'exact-archive-manifest','exec'))
for f in ['common.ps1','audit.ps1','build.ps1','postflight.ps1']:(ROOT/f).write_text((old/f).read_text())
p=ROOT/'common.ps1';s=p.read_text().replace('2508[0-9]|2509[0-9]|25241','2510[0-9]|2511[0-9]');p.write_text(s)
p=ROOT/'build.ps1';s=p.read_text().replace('e6a9125be2fc59fbe92fef6ea351b222298bfc52',commit).replace('discovery_recovery','dns_queue_cancel_c4806a3d');p.write_text(s)
script=pathlib.Path('<LOCAL_PRIOR_BOUNDED>/setup.input.ps1').read_text().replace('oe-win-bounded-e4f48ba-','oe-win-dns-8333a48-')
r,out=remote('setup',script);assert r['transport_exit']==0,r
state=json.loads(out);prior=json.loads((old/'state.json').read_text());state['prior_fixtures']=prior['prior_fixtures']+[{'label':'observed-startup-e6a9125','root':prior['root']}]
dns=json.loads(pathlib.Path('<LOCAL_PRIOR_DNS_STATE>').read_text())
for key in ['remote','diag_remote']:
 v=dns[key];assert isinstance(v,str);state['prior_fixtures'].append({'label':'historical-dns-'+key,'root':v})
(ROOT/'state.json').write_text(json.dumps(state,indent=2)+'\n');(ROOT/'state-input.json').write_text(json.dumps(state,indent=2)+'\n')
cache=(old/'cache-discovery.input.ps1').read_text();cache=cache.replace(prior['root'],state['root']);r,out=remote('cache-discovery',cache,timeout=55,exe='pwsh.exe');assert r['transport_exit']==0,r
print('New protected fixture created; private original fixture paths retained. Exact archive ready:',sha(ROOT/'source.tar'))
