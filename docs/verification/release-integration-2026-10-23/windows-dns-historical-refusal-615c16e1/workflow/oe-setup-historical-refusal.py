import json,pathlib,sys
ptr=json.loads(pathlib.Path('/private/tmp/oe-historical-refusal-pointer.json').read_text());p=pathlib.Path(ptr['local']);prior=pathlib.Path(ptr['prior']);sys.path.insert(0,str(p));from transport import remote
r,out=remote('authenticated-readonly-preflight',(prior/'authenticated-readonly-preflight.input.ps1').read_text(),timeout=25);print(json.dumps(r));print(out);assert r['transport_exit']==0
script=(prior/'setup.input.ps1').read_text().replace('oe-win-dns-8281eb7-','oe-historical-refusal-615c16e1-')
r,out=remote('setup',script,timeout=30);print(json.dumps(r));assert r['transport_exit']==0
state=json.loads(out.lstrip('\ufeff'));old=json.loads((prior/'state.json').read_text());state['prior_fixtures']=old['prior_fixtures']+[{'label':'prior-exact-r12-8281eb7','root':old['root']}]
(p/'state.json').write_text(json.dumps(state,indent=2)+'\n');(p/'state-input.json').write_text(json.dumps(state,indent=2)+'\n')
script=(prior/'cache-discovery.input.ps1').read_text().replace(old['root'],state['root'])
r,out=remote('cache-discovery',script,timeout=45);print(json.dumps(r));assert r['transport_exit']==0
cache=json.loads(out.lstrip('\ufeff'));(p/'cache-discovery.json').write_text(json.dumps(cache,indent=2)+'\n')
assert not cache['active_builds'];assert not any(cache['build_env'].values());assert cache['reparse_points']==0
print(json.dumps({'fresh_remote_fixture_created':True,'session':state['session'],'current_user_only':state['current_user_only'],'prior_fixtures_to_preserve':len(state['prior_fixtures']),'registry_files':cache['registry_files'],'active_build_count':len(cache['active_builds']),'inherited_build_overrides_empty':True}))
