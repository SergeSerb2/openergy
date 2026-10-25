"""Parent-owned static driver binding. Imports only our source-data builder."""
from pathlib import Path
import ast
import hashlib
import json
import runpy

OUT=Path(__file__).resolve().parent
HELP=runpy.run_path(str(OUT/'build-path-variant.py'),run_name='parent_source_data_builder')
REPO=HELP['REPO']
canonical,sha,put,span=HELP['canonical'],HELP['sha'],HELP['put'],HELP['span']
COMPONENT='b94f1c7cba63a15f86aa6dc2f7301158e6bf5b1d'
before=HELP['current_identity']()
old=(HELP['DRIVER']/'driver.py.txt').read_bytes()
assert sha(old)=='5b0fe6c9fada122bf8dc30b0814d49959f4f7863da32b3c444792a978d0a2a46'
paths,path_edits=HELP['variant'](old)
tree=ast.parse(paths)
nodes={n.targets[0].id:n.value for n in tree.body if isinstance(n,ast.Assign)
       and len(n.targets)==1 and isinstance(n.targets[0],ast.Name)}
required=ast.literal_eval(nodes['REQUIRED_TEMPLATES'])
baseline_required=json.loads(json.dumps(required))
for name in ['observer','assessor']:
    raw=(OUT/(name+'.py.txt')).read_bytes()
    required[name]={'bytes':len(raw),'sha256':sha(raw)}
assert required['collector']==baseline_required['collector']
edits=[]
for name,value in [('REQUIRED_COMPOSITION_CANDIDATE',COMPONENT),('REQUIRED_TEMPLATES',required)]:
    node=nodes[name]
    left,right=span(paths,node)
    edits.append((left,right,repr(value).encode('ascii')))
actual=paths
for left,right,raw in sorted(edits,reverse=True):actual=actual[:left]+raw+actual[right:]
new_tree=ast.parse(actual)
# Restore only the two global values; the complete AST must equal path-only data.
for target,value in [('REQUIRED_COMPOSITION_CANDIDATE','8bde42c5d18bb548aa435ee8f25321f236150da8'),
                     ('REQUIRED_TEMPLATES',baseline_required)]:
    node=next(n for n in new_tree.body if isinstance(n,ast.Assign) and len(n.targets)==1
              and isinstance(n.targets[0],ast.Name) and n.targets[0].id==target)
    node.value=ast.parse(repr(value),mode='eval').body
assert ast.dump(new_tree,include_attributes=False)==ast.dump(ast.parse(paths),include_attributes=False)
# Historical forbidden template hashes stay old, rather than rejecting new hashes.
for digest in ['845d90024ad47f6b76d6d6e646b679d00bb97b8ca0478d0fa86523de3dbec60f',
               '1e52486a991886c0dbaa463ef3203695cabcd61aeedb841bb45275850541cbd9',
               '695dc01461b31e04eb7b8c4a12848350f26a45731f05b0ddca971c05749e4df6']:
    assert actual.count(digest.encode())==old.count(digest.encode())
put('driver.py.txt',actual)
put('driver-paths-and-bindings.patch',HELP['patch'](old,actual,'driver.py.txt'))
schema_old=(HELP['DRIVER']/'config.schema.json').read_bytes()
schema=json.loads(schema_old)
changes=[]
mapping={'8bde42c5d18bb548aa435ee8f25321f236150da8':COMPONENT,
         '7b7e9dd96e0d7e3c1168c8dd2f5acc266967441061fefe2d29622add9729d5a6':required['observer']['sha256'],
         '517c7cc82336d1a924e1578d3c13c68960834554596a668e80013fd8818ed913':required['assessor']['sha256'],
         7673:required['observer']['bytes'],25746:required['assessor']['bytes']}
def update(value,path=()):
    if type(value)is dict:
        if set(value).intersection(['const']) and type(value['const'])in [int,str] and value['const'] in mapping:
            oldvalue=value['const'];value['const']=mapping[oldvalue]
            changes.append({'path':list(path)+['const'],'old':oldvalue,'new':value['const']})
        for key,item in value.items():update(item,path+(key,))
    elif type(value)is list:
        for index,item in enumerate(value):update(item,path+(index,))
update(schema)
assert len(changes)==8
restored=json.loads(json.dumps(schema))
for change in changes:
    item=restored
    for key in change['path'][:-1]:item=item[key]
    item[change['path'][-1]]=change['old']
assert restored==json.loads(schema_old)
put('config.schema.json',(json.dumps(schema,indent=2,sort_keys=True)+'\n').encode('ascii'))
for name in ['config.template.json','root-source-association.template.json']:
    put(name,(HELP['DRIVER']/name).read_bytes())
after=HELP['current_identity']()
assert before==after
put('driver-bindings.json',canonical({'schema':'openergy.prepare-repaired-driver-source.v1',
    'component_candidate':COMPONENT,'driver':{'bytes':len(actual),'sha256':sha(actual)},
    'original_driver':{'candidate':'7c92909fcd41580183c224549a0adb768602f9f8','bytes':len(old),'sha256':sha(old)},
    'literal_path_edits':path_edits,'required_templates':required,'schema_const_changes':changes,
    'old_declarations':HELP['inventory'](old),'new_declarations':HELP['inventory'](actual),
    'only_path_literals_and_two_global_bindings_changed':True,'historical_forbidden_hashes_preserved':True,
    'old_entry_preservation_before':before,'old_entry_preservation_after':after,
    'execution':False,'independent_full_review':None,'runtime':'UNVERIFIED'}))
put('finite-plan.json',canonical({'schema':'openergy.prepare-repaired-local-plan.v1',
    'execution':False,'component_candidate':COMPONENT,'driver_source_sha256':sha(actual),
    'driver_candidate':None,'full_composition_review':None,'repaired_remote_capsule':None,
    'requirements_before_prepare_gate':None,'bound_driver_config':None,'actual_renders':None,
    'actual_guard':None,'actual_handles_exits_EOF':None,'actual_D4':None,
    'prepare_accepted':False,'copy_accepted':False,'product_accepted':False,
    'old_failed_attempt_retained':True,'old_once_guard_reusable':False}))
print(canonical({'driver_bytes':len(actual),'driver_sha256':sha(actual),
    'path_literal_edits':len(path_edits),'schema_const_changes':len(changes),
    'provided_programs_executed':0}).decode('ascii'),end='')
