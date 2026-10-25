"""Reviewer-owned public Git/text/strictJSON/hash/AST DATA reader.
Never imports, executes, renders, or executable-compiles supplied source.
Writes only this review folder. Not a runtime checker or test.
"""
import ast
import difflib
import hashlib
import io
import json
import math
import os
from pathlib import Path
import re
import stat
import subprocess
import tokenize

OUT = Path(__file__).parent
D = Path('docs/verification/m1-prepare-local-parent-driver-2026-10-25-t3-def6ed35')
C = Path('docs/verification/m1-local-direct-signal-composition-2026-10-25-t3-52422617')
R = Path('docs/verification/m1-prepare-parent-root-layout-repair-2026-10-25-t3-1f8914f8')
ENV = dict(os.environ, GIT_OPTIONAL_LOCKS='0')
checks = []
bindings = []

def check(name, value):
    checks.append({'name': name, 'passed': bool(value)})
    if not value:
        raise AssertionError(name)

def git(*args):
    return subprocess.check_output(['git', *args], env=ENV)

def sha(raw):
    return hashlib.sha256(raw).hexdigest()

def pairs(rows):
    result = {}
    for key, value in rows:
        if key in result:
            raise ValueError('duplicate JSON key')
        result[key] = value
    return result

def finite(text):
    value = float(text)
    if not math.isfinite(value):
        raise ValueError('nonfinite JSON number')
    return value

def reject(text):
    raise ValueError('nonfinite JSON constant')

def load(raw):
    return json.loads(raw, object_pairs_hook=pairs, parse_float=finite, parse_constant=reject)

def canonical(value):
    return (json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=True, allow_nan=False)+'\n').encode('ascii')

def write(name, value):
    (OUT/name).write_text(json.dumps(value, indent=2, sort_keys=True, allow_nan=False)+'\n')

def bind(rev, path, scope):
    path = str(path)
    row = git('ls-tree', rev, '--', path).decode().strip()
    mode, kind, blobpath = row.split(' ', 2)
    blob, actualpath = blobpath.split('\t')
    raw = git('show', rev+':'+path)
    check('working equality '+rev+':'+path, Path(path).read_bytes() == raw and stat.S_IMODE(Path(path).stat().st_mode)==(0o755 if mode=='100755' else 0o644))
    record = dict(commit=git('rev-parse',rev).decode().strip(),path=path,mode=mode,blob=blob,bytes=len(raw),sha256=sha(raw),read_scope=scope)
    bindings.append(record)
    return raw

def datum(node):
    # Closed AST-data decoder. No eval, literal_eval, executable compile or import.
    if isinstance(node, ast.Constant): return node.value
    if isinstance(node,(ast.List,ast.Tuple,ast.Set)):
        values=[datum(x) for x in node.elts]
        return tuple(values) if isinstance(node,ast.Tuple) else set(values) if isinstance(node,ast.Set) else values
    if isinstance(node,ast.Dict): return {datum(k):datum(v) for k,v in zip(node.keys,node.values)}
    if isinstance(node,ast.UnaryOp) and isinstance(node.op,ast.USub): return -datum(node.operand)
    raise ValueError('not literal data: '+type(node).__name__)

def assignment(tree,name):
    rows=[n for n in tree.body if isinstance(n,ast.Assign) and any(isinstance(t,ast.Name) and t.id==name for t in n.targets)]
    check('unique assignment '+name,len(rows)==1)
    return rows[0].value

def fragment(raw, node):
    lines=raw.splitlines(keepends=True)
    return b''.join(lines[min([node.lineno]+[d.lineno for d in getattr(node,'decorator_list',[])])-1:node.end_lineno])

def record(raw,node,name=None):
    data=fragment(raw,node)
    return dict(name=name,kind=type(node).__name__,first_line=min([node.lineno]+[d.lineno for d in getattr(node,'decorator_list',[])]),last_line=node.end_lineno,bytes=len(data),sha256=sha(data))

def callable_name(node):
    if isinstance(node,ast.Name): return node.id
    if isinstance(node,ast.Attribute): return callable_name(node.value)+'.'+node.attr
    return '<'+type(node).__name__+'>'

def inventory(raw):
    tree=ast.parse(raw.decode('ascii'))
    declarations=[]
    def visit(body,prefix=''):
        for node in body:
            if isinstance(node,(ast.FunctionDef,ast.AsyncFunctionDef,ast.ClassDef)):
                name=prefix+node.name
                row=record(raw,node,name)
                row['calls']=[dict(line=n.lineno,callable=callable_name(n.func)) for n in ast.walk(node) if isinstance(n,ast.Call)]
                declarations.append(row)
                visit(node.body,name+'.')
    visit(tree.body)
    blocks=[record(raw,n) for n in tree.body]
    # Non-overlapping full physical reconstruction: statements plus all gaps.
    segments=[]; cursor=1; lines=raw.splitlines(keepends=True)
    for n in tree.body:
        start=min([n.lineno]+[d.lineno for d in getattr(n,'decorator_list',[])])
        if cursor<start:
            data=b''.join(lines[cursor-1:start-1]); segments.append(dict(kind='gap',first_line=cursor,last_line=start-1,bytes=len(data),sha256=sha(data)))
        row=record(raw,n); segments.append(row); cursor=n.end_lineno+1
    if cursor<=len(lines):
        data=b''.join(lines[cursor-1:]); segments.append(dict(kind='gap',first_line=cursor,last_line=len(lines),bytes=len(data),sha256=sha(data)))
    rebuilt=b''.join(b''.join(lines[r['first_line']-1:r['last_line']]) for r in segments)
    check('whole module segment reconstruction '+sha(raw),rebuilt==raw)
    return tree,dict(source=dict(bytes=len(raw),lines=len(lines),sha256=sha(raw)),declarations=declarations,module_statements=blocks,complete_partition=segments)

def span(raw,node):
    lines=raw.splitlines(keepends=True)
    return sum(map(len,lines[:node.lineno-1]))+node.col_offset,sum(map(len,lines[:node.end_lineno-1]))+node.end_col_offset

def method(raw):
    tree=ast.parse(raw.decode('ascii')); node=assignment(tree,'CONFIG_TEXT'); left,right=span(raw,node)
    tokens=[t for t in tokenize.tokenize(io.BytesIO(raw).readline) if t.type==tokenize.STRING and sum(map(len,raw.splitlines(keepends=True)[:t.start[0]-1]))+t.start[1]==left and sum(map(len,raw.splitlines(keepends=True)[:t.end[0]-1]))+t.end[1]==right]
    check('one lexical CONFIG_TEXT string '+sha(raw),len(tokens)==1 and type(node.value) is str)
    node.value='<CONFIG_TEXT>'
    return dict(schema='openergy.single-config-render-method.v1',template=dict(bytes=len(raw),sha256=sha(raw)),token_start=left,token_end=right,token_sha256=sha(raw[left:right]),prefix_sha256=sha(raw[:left]),suffix_sha256=sha(raw[right:]),normalized_ast_sha256=sha(ast.dump(tree,annotate_fields=True,include_attributes=False).encode('ascii')),normalization='ast.dump annotate_fields=True include_attributes=False; CONFIG_TEXT value=<CONFIG_TEXT>',render='one ASCII Python STRING via json.dumps(JSON-text,ensure_ascii=True); exact restore',excludes=['own reference','bound config','future full rendered source hash'])

def reconstruct(basis,patch):
    rows=patch.splitlines(keepends=True); old=basis.splitlines(keepends=True); out=[]; pos=0; i=2; hunks=0
    check('unified patch headers',rows[0].startswith(b'--- ') and rows[1].startswith(b'+++ '))
    while i<len(rows):
        m=re.fullmatch(rb'@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@[^\n]*\n',rows[i]); check('hunk header '+str(i),m is not None)
        start=int(m[1]); oldcount=int(m[2] or b'1'); newstart=int(m[3]); newcount=int(m[4] or b'1')
        begin=0 if start==0 else start-1; out.extend(old[pos:begin]); pos=begin; i+=1; before=after=0; newpos=len(out)
        check('new hunk location '+str(i),newpos==(0 if newstart==0 else newstart-1))
        while i<len(rows) and not rows[i].startswith(b'@@ '):
            row=rows[i]; check('patch line prefix '+str(i),row[:1] in (b' ',b'+',b'-'))
            if row[:1] in (b' ',b'-'):
                check('old/context exact '+str(i),pos<len(old) and row[1:]==old[pos]); pos+=1; before+=1
            if row[:1] in (b' ',b'+'): out.append(row[1:]); after+=1
            i+=1
        check('hunk counts '+str(i),(before,after)==(oldcount,newcount)); hunks+=1
    out.extend(old[pos:]); return b''.join(out),hunks

packets={}
for rev,folder,count in [('7c92909fcd41580183c224549a0adb768602f9f8',D,20),('8bde42c5d18bb548aa435ee8f25321f236150da8',C,13)]:
    names=git('ls-tree','-r','--name-only',rev,'--',str(folder)).decode().splitlines()
    check('packet count '+str(folder),len(names)==count)
    check('exact final folder tree '+str(folder),git('rev-parse',rev+':'+str(folder))==git('rev-parse','HEAD:'+str(folder)))
    for name in names:
        raw=bind(rev,name,'complete source/text or complete strictJSON data traversal')
        packets[name]=load(raw) if name.endswith('.json') else raw
    sums=packets[str(folder/'SHA256SUMS')].decode().splitlines()
    check('manifest cardinality '+str(folder),len(sums)==count-1)
    for line in sums:
        digest,name=line.split('  ',1); check('manifest '+name,sha(packets[str(folder/name)] if isinstance(packets[str(folder/name)],bytes) else Path(folder/name).read_bytes())==digest)

source=packets[str(D/'driver.py.txt')]; dt,di=inventory(source)
check('driver frozen source',len(source)==47041 and len(source.splitlines())==957 and sha(source)=='5b0fe6c9fada122bf8dc30b0814d49959f4f7863da32b3c444792a978d0a2a46')
check('driver function count',len(di['declarations'])==38)
patch=packets[str(D/'driver-added.patch')]
check('exact driver patch SHA',sha(patch)=='00eafee424f7369a0c25f2acb23f8ca252078a269e5b74bc0859f30ab03fabf3')
rebuilt,hunks=reconstruct(b'',patch); check('complete driver added patch',rebuilt==source and hunks==1)
inv=packets[str(D/'function-global-inventory.json')]
for expected,actual in zip(inv['functions'],di['declarations']):
    check('driver function binding '+actual['name'],all(expected[k]==actual[k] for k in ['name','kind','first_line','last_line','bytes','sha256']) and expected['calls']==[dict(line=n.lineno,callable=ast.unparse(n.func)) for n in ast.walk(next(x for x in dt.body if isinstance(x,ast.FunctionDef) and x.name==actual['name'])) if isinstance(n,ast.Call)])
nonfunc=[n for n in dt.body if not isinstance(n,ast.FunctionDef)]
check('all module statements count',len(nonfunc)==len(inv['non_function_module_statements']))
for node,expected in zip(nonfunc,inv['non_function_module_statements']):
    actual=record(source,node); check('driver module statement '+str(node.lineno),all(expected[k]==actual[k] for k in ['kind','first_line','last_line','bytes','sha256']) and expected['ast_data']==ast.dump(node,annotate_fields=True,include_attributes=False))
config=load(assignment(dt,'CONFIG_TEXT').value); check('decoded config equals template',config==packets[str(D/'config.template.json')])
def nullpaths(value,prefix=''):
    if value is None: return [prefix]
    if isinstance(value,dict): return sum((nullpaths(v,prefix+'/'+k) for k,v in value.items()),[])
    if isinstance(value,list): return sum((nullpaths(v,prefix+'/'+str(k)) for k,v in enumerate(value)),[])
    return []
check('null42 count',len(nullpaths(config))==42)
check('schema inert constant',config==packets[str(D/'config.schema.json')]['oneOf'][0]['const'])
def closed_schema(v):
    if isinstance(v,dict):
        if v.get('type')=='object': check('closed schema '+str(len(checks)),v.get('additionalProperties') is False and set(v['properties'])==set(v['required']))
        for x in v.values(): closed_schema(x)
    elif isinstance(v,list):
        for x in v: closed_schema(x)
closed_schema(packets[str(D/'config.schema.json')])
dm=method(source)
check('driver normalized AST',dm['normalized_ast_sha256']==inv['CONFIG_TEXT_token']['normalized_full_ast_sha256'])
for actual,key in [('token_start','start'),('token_end','end'),('token_sha256','sha256'),('prefix_sha256','prefix_sha256'),('suffix_sha256','suffix_sha256')]: check('driver token '+key,dm[actual]==inv['CONFIG_TEXT_token'][key])
check('entry block',sha(b''.join(source.splitlines(keepends=True)[950:]))==inv['entry_block']['sha256'])
for row in packets[str(D/'record-layouts.json')]['literal_record_key_layouts']:
    matches=[n for n in ast.walk(dt) if isinstance(n,ast.Dict) and n.lineno==row['first_line'] and n.end_lineno==row['last_line']]
    check('literal record keys '+str(row['first_line']),len(matches)==1 and [datum(k) for k in matches[0].keys]==row['keys_in_source_order'] and len(matches[0].keys)==row['key_count'])
# Full author inventory: compare all old public Git/working blobs and modes.
opening=packets[str(D/'opening-inventory.json')]
check('author opening base',opening['head']=='7e5de706e7457de5f4880cdef180b27536dc0d5e' and opening['clean'] is True and len(opening['entries'])==4422)
oldtree={r.split(b'\t',1)[1].decode():r.split(b'\t',1)[0].decode().split() for r in git('ls-tree','-r','-z',opening['head']).split(b'\0') if r}
check('author full opening path set',set(oldtree)=={r['path'] for r in opening['entries']})
for row in opening['entries']:
    p=Path(row['path']); raw=p.read_bytes(); mode,kind,blob=oldtree[str(p)]
    check('author old entry '+str(p),mode==row['git_mode'] and blob==row['blob'] and len(raw)==row['working_bytes'] and sha(raw)==row['working_sha256'] and stat.S_IMODE(p.stat().st_mode)==row['working_mode'])
check('author opening hash',sha(Path(D/'opening-inventory.json').read_bytes())==packets[str(D/'preservation.json')]['opening_inventory_sha256'])
# Exact import trees and residue, without applying any candidate commit.
for original,imported in [('96924b700055e8b57ac871ef2924eb2188d03602','5c492d5'),('7c92909fcd41580183c224549a0adb768602f9f8','f207502')]:
    check('import packet tree '+imported,git('rev-parse',original+':'+str(D))==git('rev-parse',imported+':'+str(D)))
    check('import exact patch delta '+imported,git('diff',original+'^',original,'--',str(D))==git('diff',imported+'^',imported,'--',str(D)))
    check('no import outside residue '+imported,all(r.split('\t')[-1].startswith(str(D)+'/') for r in git('diff-tree','--no-commit-id','--name-status','-r',imported).decode().splitlines()))
check('import correct ancestry',subprocess.run(['git','merge-base','--is-ancestor','5c492d5','f207502'],env=ENV).returncode==0)
baseinfo=packets[str(C/'bindings.json')]['sources']; composition={}; oldsources={}; comparisons={}
for name,info in baseinfo.items():
    old=bind(info['baseline']['candidate_commit'],info['baseline']['path'],'complete baseline AST/function/global comparison; shared bodies read in exact new source')
    oldsources[name]=old; new=Path(C/(name+'.py.txt')).read_bytes()
    check(name+' baseline identity',len(old)==info['baseline']['bytes'] and sha(old)==info['baseline']['sha256'])
    check(name+' new identity',len(new)==info['source']['bytes'] and sha(new)==info['source']['sha256'])
    bt,bi=inventory(old); nt,ni=inventory(new); composition[name]=ni
    patch=Path(C/(name+'-from-frozen.patch')).read_bytes(); rebuilt,hunks=reconstruct(old,patch)
    check(name+' patch reconstruction',rebuilt==new)
    diff=''.join(difflib.unified_diff(old.decode().splitlines(keepends=True),new.decode().splitlines(keepends=True),fromfile=info['baseline']['path'],tofile=name+'.py.txt')).encode()
    check(name+' complete patch byte equality',diff==patch)
    bd={r['name']:r for r in bi['declarations']}; nd={r['name']:r for r in ni['declarations']}
    author=packets[str(C/'function-bindings.json')][name]
    check(name+' all declaration names',set(bd)==set(nd)==set(author['bindings']))
    changed=[]
    for key in bd:
        if bd[key]['sha256']!=nd[key]['sha256']: changed.append(key)
        for label,derived in [('before',bd[key]),('after',nd[key])]:
            a=author['bindings'][key][label]; check(name+' '+label+' '+key,all(a[k]==derived[v] for k,v in [('start_line','first_line'),('end_line','last_line'),('bytes','bytes'),('sha256','sha256')]))
    check(name+' changed declaration set',sorted(changed)==sorted(author['changed']))
    comparisons[name]=dict(before=bi,after=ni,changed_declarations=changed,patch_hunks=hunks,exact_patch_reconstruction=True)
    if name=='collector':
        expected=old.replace(b'import selectors\n',b'import selectors\nimport signal\n').replace(b'# Only this retained local SSH handle. poll() rechecks before kill().',b'# The fresh poll/sample below precedes one direct retained-child signal.').replace(b'self.process.kill()',b'os.kill(self.process.pid, signal.SIGKILL)')
    else:
        expected=old.replace(b'transport-phase-budgets.py',b'transport-direct-signal.py').replace(b'695dc01461b31e04eb7b8c4a12848350f26a45731f05b0ddca971c05749e4df6',b'f1fa00b2ec61f6e4996b5938388f345a8ddcfb205455bc4482ce25711824f371').replace(b'28060',b'28099')
    check(name+' only exact stated whole-file replacements',new==expected)
    check(name+' required driver source constants',datum(assignment(dt,'REQUIRED_TEMPLATES'))[name]==dict(bytes=len(new),sha256=sha(new)))
    for packet in packets[str(D/'composition-handoff.json')]['packet_files']:
        if packet['path'].endswith('/'+name+'.py.txt'): check(name+' handoff identity',packet['sha256']==sha(new) and packet['bytes']==len(new))
check('required method exact',method(Path(C/'assessor.py.txt').read_bytes())==packets[str(D/'required-composition-assessor-render-method.json')])
check('historical method exact',method(oldsources['assessor'])==packets[str(D/'historical-assessor-render-method.json')])
check('required method canonical SHA',sha(canonical(packets[str(D/'required-composition-assessor-render-method.json')]))=='bef51d00e4635867714e48fe57b229d2111474e5284ffbdd0ec43b3521b3103a')
for row in packets[str(D/'input-bindings.json')]['inputs']:
    raw=bind(row['commit'],row['path'],'full public data identity; selected interface scope as recorded separately in REVIEW.md')
    if row['path'].endswith('.json'): load(raw)
    check('driver frozen input '+row['path'],sha(raw)==row['sha256'] and len(raw)==row['bytes'])
reviews=[('98b403d1902ac4d0651c230fb7ec28c4cc154035','docs/verification/m1-transport-phase-budget-review-2026-10-25-t3-04f3d533/REVIEW.md','0c3b311ebff23145ecebd1b64ed38f0e6feaaff46c80c0b1d7a8ecc92d769a3c'),('91f7c8a1f3359d4a322c936712b3ebe9d451901d','docs/verification/m1-prepare-local-exit-observer-review-2026-10-25-t3-c2d0d953/REVIEW.md','bb8ed39275540eabc116cca932a40d42246c939c7245f8c04eaba03c87c97ed1'),('8d9494216ca01ef5258ee104b04cf58cab7073cb','docs/verification/m1-prepare-transport-assessor-full-review-2026-10-25-t3-0b5fe49c/REVIEW.md','d4c126c625ec04dd7f523599496a565413ce691d0797f6a069b4a8ec46aa0f62'),('63b6a3b564d6a557f60e0e2b0e4a1943ffa2b19f','docs/verification/m1-prepare-parent-root-layout-review-2026-10-25-t3-98186f1b/REVIEW.md','dc8e986c6a3e914b65a13686fc9f1a7e15702e8548e545075d6967f340541c4e')]
for rev,path,digest in reviews: check('full baseline/dependency report '+rev,sha(bind(rev,path,'complete report read'))==digest)
remote=bind('02f94afbe3e1001b8d3e8fc4d79976475485f661',R/'prepare-parent.py.txt','complete relevant bind_config, inspect_frozen, import_common, common_deadline, reserved_put, collect, emit_once, main; global identity data; no new full remote verdict')
check('remote candidate source',len(remote)==109444 and sha(remote)=='b3fbd793a0709c7857e34efca667a081a0486accfd0f3ad13a8068484b7eb8d9')
rt=ast.parse(remote.decode('ascii'))
historical=packets[str(C/'historical-status-recheck.json')]
raws=datum(assignment(rt,'SOURCE_RAWS')); expected_statuses=[x for x in raws if x['leaf'].endswith('-os.json')]
check('public root53 references',len(expected_statuses)==53 and historical['raw_status_refs']==expected_statuses)
v=historical['validation']; validation=bind('HEAD',v['path'],'complete public historical validation as DATA; no private raw reopened')
check('historical validation exact',len(validation)==v['bytes'] and sha(validation)==v['sha256']); load(validation)
for row in packets[str(D/'composition-handoff.json')]['packet_files']:
    check('entire handoff packet binding '+row['path'],sha(Path(row['path']).read_bytes())==row['sha256'] and len(Path(row['path']).read_bytes())==row['bytes'])
for leaf in ['PARENT-CONTRACT.md','pin-bindings.json','frozen-dependency-signal-audit.json']:
    raw=bind('02f94afbe3e1001b8d3e8fc4d79976475485f661',R/leaf,'full public contract or data read')
    if leaf.endswith('.json'):load(raw)
for leaf in ['operator-plan.md','receipt-contract.md']:
    bind('HEAD',Path('docs/verification/m1-ordinary-fresh-flow-plan-2026-10-24-t3-5463ce44')/leaf,'complete original operator/receipt contract')
for name,wanted in [('prepare','16f2c5d9dce3f3bce8a871bd53eea06ebf1818080de5f9bdaae3997b86304e9a'),('common','20a9c16030b2de53b99870f25bada68137bfc6fbc5b67cc29f8e878312d4129c')]:
    raw=bind('HEAD',Path('docs/verification/ordinary-combined-recorder-diagnostic-2026-10-25/source')/(name+'.py.txt'),'full prepare source or common filesystem/write closure read; callgraph AST data')
    check('exact pinned '+name,sha(raw)==wanted)
    tree,invx=inventory(raw); write(name+'-callgraph.json',invx)
# Library text only, never imported or executed.
library=Path(packets[str(C/'bindings.json')]['runtime_public_source']['path']); libraw=library.read_bytes(); libmeta=packets[str(C/'bindings.json')]['runtime_public_source']
check('public local3.15 library text',len(libraw)==libmeta['bytes'] and sha(libraw)==libmeta['sha256'])
write('public-library-text.json',dict(path=str(library),bytes=len(libraw),sha256=sha(libraw),ranges=libmeta['ranges'],scope='public source text only; no target runtime, signal, OS status or installed remote3.13.9 proof'))
# Cross-module closed interfaces and call-site identities from actual AST data.
def split_assignment(tree,name):
    node=assignment(tree,name)
    check('literal split data '+name,isinstance(node,ast.Call) and isinstance(node.func,ast.Attribute) and node.func.attr=='split' and isinstance(node.func.value,ast.Constant) and not node.args and not node.keywords)
    return node.func.value.value.split()
at=ast.parse(Path(C/'assessor.py.txt').read_text())
check('24-field parent exact shared key set',split_assignment(dt,'PARENT_KEYS')==split_assignment(at,'PARENT_KEYS') and len(split_assignment(dt,'PARENT_KEYS'))==24)
check('18-field request keys',len(split_assignment(at,'REQUEST_KEYS'))==18)
check('33-field receipt keys',len(split_assignment(at,'RECEIPT_KEYS'))==33)
check('10-field guard keys',len(split_assignment(at,'GUARD_KEYS'))==10)
acfg=load(assignment(at,'CONFIG_TEXT').value)
check('unchanged null24 assessor config',len(nullpaths(acfg))==24)
check('assessor old schema inert constant',acfg==load(Path(baseinfo['assessor']['baseline']['path']).parent.joinpath('config.schema.json').read_bytes())['oneOf'][0]['const'])
cc=ast.parse(Path(C/'collector.py.txt').read_text())
check('four fixed phase work map',datum(assignment(cc,'PHASE_WORK_SECONDS'))=={'metadata-source':130,'metadata-copy':130,'prepare':130,'controller':335})
check('nine flags',len(datum(assignment(cc,'FLAGS')))==9)
check('collector cap constants',datum(assignment(cc,'CAPS'))=={'stdout':131072,'stderr':65536} and datum(assignment(cc,'COMBINED_CAP'))==196608)
calls=[n for n in ast.walk(dt) if isinstance(n,ast.Call)]
check('two literal Popen sites',[n.lineno for n in calls if callable_name(n.func)=='subprocess.Popen']==[829,883])
check('no driver signal/wait/discovery/eval/exec/compile calls',not any(callable_name(n.func) in ['os.kill','os.killpg','os.waitpid','os.wait','subprocess.run','eval','exec','compile'] or isinstance(n.func,ast.Attribute) and n.func.attr in ['kill','terminate','send_signal','wait','communicate'] for n in calls))
check('collector one direct signal',[n.lineno for n in ast.walk(cc) if isinstance(n,ast.Call) and callable_name(n.func)=='os.kill']==[428])
check('no Popen signal methods in new composition',not any(isinstance(n,ast.Call) and isinstance(n.func,ast.Attribute) and n.func.attr in ['kill','send_signal','terminate'] and callable_name(n.func)!='os.kill' for tree in [cc,at,ast.parse(Path(C/'observer.py.txt').read_text())] for n in ast.walk(tree)))
# Exact remote metadata is historic: no executed dependency vector references it.
fixed=datum(assignment(rt,'FIXED_BINDINGS'))
check('historic remote transport basis retained',fixed['transport']['source']['sha256']=='695dc01461b31e04eb7b8c4a12848350f26a45731f05b0ddca971c05749e4df6')
remote_calls=[dict(line=n.lineno,callable=callable_name(n.func)) for n in ast.walk(rt) if isinstance(n,ast.Call)]
check('remote common command unreachable by direct call',not any(x['callable'] in ['common.command','module.command','common.git_file'] for x in remote_calls))
pt=ast.parse(Path('docs/verification/ordinary-combined-recorder-diagnostic-2026-10-25/source/prepare.py.txt').read_text())
check('pinned prepare has no command/git_file/port diagnostic call',not any(isinstance(n,ast.Call) and callable_name(n.func) in ['command','git_file','query','subprocess.Popen'] for n in ast.walk(pt)))
write('closed-interface-observations.json',dict(execution=False,parent_fields=split_assignment(dt,'PARENT_KEYS'),request_fields=split_assignment(at,'REQUEST_KEYS'),guard_fields=split_assignment(at,'GUARD_KEYS'),receipt_fields=split_assignment(at,'RECEIPT_KEYS'),assessor_null_paths=nullpaths(acfg),driver_read_table=datum(assignment(dt,'FILES')),driver_write_table=datum(assignment(dt,'WRITES')),remote_historical_fixed_source_bindings=fixed,remote_calls=remote_calls,actual_composition_bindings_location='local driver REQUIRED_TEMPLATES/CONFIG and observed future transport records; no remote active-local-source assertion',runtime_and_future_render='UNVERIFIED'))
write('driver-full-source-bindings.json',dict(**di,render_method=dm,config_null_paths=nullpaths(config)))
write('composition-full-source-bindings.json',comparisons)
write('input-bindings.json',dict(schema='independent-full-source-inputs-v1',execution=False,inputs=bindings))
write('static-observations.json',dict(schema='independent-full-source-data-observations-v1',execution=False,all_passed=True,check_count=len(checks),checks=checks,target_runtime_AST_agreement='UNVERIFIED',no_future_render=True,historical_private_originals_reopened=False))
print(json.dumps(dict(checks=len(checks),all_passed=True,input_records=len(bindings),driver_functions=len(di['declarations']),new_source_lines={n:i['source']['lines'] for n,i in composition.items()})))
