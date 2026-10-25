"""Parent-owned source-data path variant builder. No supplied code executes."""
from pathlib import Path
import ast
import difflib
import hashlib
import json
import os
import stat
import subprocess

OUT = Path(__file__).resolve().parent
REPO = OUT.parents[2]
OLD = REPO/'docs/verification/m1-local-direct-signal-composition-2026-10-25-t3-52422617'
DRIVER = REPO/'docs/verification/m1-prepare-local-parent-driver-2026-10-25-t3-def6ed35'

NAMES = {
    'prepare-launch':'prepare-repaired-launch',
    'prepare-run':'prepare-repaired-run',
    'prepare-parent-request.json':'prepare-repaired-parent-request.json',
    'prepare-parent-exit.json':'prepare-repaired-parent-exit.json',
    'prepare-transport-assessment.json':'prepare-repaired-transport-assessment.json',
    'prepare-local-parent-once.json':'prepare-repaired-local-parent-once.json',
    'prepare-local-parent-result.json':'prepare-repaired-local-parent-result.json',
    'prepare-observer-handle-source.json':'prepare-repaired-observer-handle-source.json',
    'prepare-local-direct-topology.json':'prepare-repaired-local-direct-topology.json',
    'prepare-transport-assessor.py':'prepare-repaired-transport-assessor.py',
    'prepare-transport-assessor.direct-signal.template.py.txt':'prepare-repaired-transport-assessor.template.py.txt',
    'prepare-assessor-render-method.json':'prepare-repaired-assessor-render-method.json',
    'prepare-assessor-render-proof.json':'prepare-repaired-assessor-render-proof.json'}


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def canonical(value):
    return (json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=True,allow_nan=False)+'\n').encode('ascii')


def put(name, raw):
    path=OUT/name
    fd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    try:
        assert os.write(fd,raw)==len(raw)
        os.fsync(fd)
    finally:os.close(fd)


def span(raw, node):
    lines=raw.splitlines(keepends=True)
    return (sum(map(len,lines[:node.lineno-1]))+node.col_offset,
            sum(map(len,lines[:node.end_lineno-1]))+node.end_col_offset)


def current_identity():
    raw=subprocess.check_output(['git','-C',str(REPO),'ls-files','-s','-z'])
    rows=[]
    for entry in raw.split(b'\0'):
        if not entry:continue
        meta,path=entry.split(b'\t',1)
        p=REPO/os.fsdecode(path)
        info=p.lstat()
        data=os.fsencode(os.readlink(p)) if stat.S_ISLNK(info.st_mode) else p.read_bytes()
        rows.append([meta.decode('ascii'),os.fsdecode(path),stat.S_IMODE(info.st_mode),len(data),sha(data)])
    return {'count':len(rows),'git_index_sha256':sha(raw),'working_and_index_canonical_sha256':sha(canonical(rows))}


def variant(raw, more=None):
    tree=ast.parse(raw)
    edits=[]
    for node in ast.walk(tree):
        if not isinstance(node,ast.Constant) or type(node.value) is not str:continue
        value=node.value
        parts=value.split('/')
        new='/'.join(NAMES.get(part,part) for part in parts)
        if more and value in more:new=more[value]
        if new!=value:
            left,right=span(raw,node)
            old=raw[left:right]
            assert old[:1] in [b"'",b'"'] and old[-1:]==old[:1]
            # All replacement paths/hashes are plain ASCII without quote escapes.
            quote=old[:1]
            assert quote not in new.encode('ascii') and '\\' not in new
            edits.append((left,right,quote+new.encode('ascii')+quote,value,new))
            node.value=new
    actual=raw
    for left,right,replacement,old,new in sorted(edits,reverse=True):
        actual=actual[:left]+replacement+actual[right:]
    assert ast.dump(ast.parse(actual),include_attributes=False)==ast.dump(tree,include_attributes=False)
    # Exact reverse replacement proves no source bytes outside the literal tokens changed.
    rebuilt=actual
    delta=0
    positions=[]
    for left,right,replacement,old,new in sorted(edits):
        a=left+delta
        positions.append((a,a+len(replacement),raw[left:right]))
        delta+=len(replacement)-(right-left)
    for a,b,original in sorted(positions,reverse=True):rebuilt=rebuilt[:a]+original+rebuilt[b:]
    assert rebuilt==raw
    return actual,[{'old_token_start':a,'old_token_end':b,'old_value':old,'new_value':new,
                    'old_token_sha256':sha(raw[a:b]),'new_token_sha256':sha(repl)}
                   for a,b,repl,old,new in sorted(edits)]


def patch(old,new,name):
    return ''.join(difflib.unified_diff(old.decode('ascii').splitlines(keepends=True),
        new.decode('ascii').splitlines(keepends=True),fromfile='a/'+name,tofile='b/'+name,n=0)).encode('ascii')


def inventory(raw):
    tree=ast.parse(raw)
    rows=[]
    for node in ast.walk(tree):
        if isinstance(node,(ast.FunctionDef,ast.AsyncFunctionDef,ast.ClassDef)):
            start=min([node.lineno]+[d.lineno for d in node.decorator_list])
            lines=raw.splitlines(keepends=True)
            body=b''.join(lines[start-1:node.end_lineno])
            rows.append({'name':node.name,'start':start,'end':node.end_lineno,
                         'bytes':len(body),'sha256':sha(body)})
    return sorted(rows,key=lambda x:(x['start'],x['end']))


def main():
    before=current_identity()
    candidate=subprocess.check_output(['git','-C',str(REPO),'rev-parse','HEAD']).decode().strip()
    outputs={}
    for name,expected in [
        ('observer','7b7e9dd96e0d7e3c1168c8dd2f5acc266967441061fefe2d29622add9729d5a6'),
        ('assessor','517c7cc82336d1a924e1578d3c13c68960834554596a668e80013fd8818ed913')]:
        old=(OLD/(name+'.py.txt')).read_bytes()
        assert sha(old)==expected
        new,edits=variant(old)
        put(name+'.py.txt',new)
        put(name+'-paths.patch',patch(old,new,name+'.py.txt'))
        put(name+'-bindings.json',canonical({'old':{'bytes':len(old),'sha256':sha(old)},
            'new':{'bytes':len(new),'sha256':sha(new)},'literal_edits':edits,
            'old_declarations':inventory(old),'new_declarations':inventory(new),
            'exact_reverse_token_reconstruction':True,'normalized_AST_only_selected_literals_changed':True}))
        outputs[name]={'bytes':len(new),'sha256':sha(new),'changed_literal_tokens':len(edits)}
    after=current_identity()
    assert before==after
    put('component-bindings.json',canonical({'schema':'openergy.prepare-repaired-local-paths.v1',
        'baseline_parent':candidate,'baseline_tree':subprocess.check_output(['git','-C',str(REPO),'rev-parse','HEAD^{tree}']).decode().strip(),
        'before':before,'after':after,'old_entries_preserved':True,'paths':NAMES,
        'sources':outputs,'collector':{'source_candidate':'8bde42c5d18bb548aa435ee8f25321f236150da8',
            'bytes':28099,'sha256':'f1fa00b2ec61f6e4996b5938388f345a8ddcfb205455bc4482ce25711824f371',
            'path_unchanged':'tools/transport-direct-signal.py'},
        'driver':'separate next source binding after this component has a real commit identity',
        'execution':False,'independent_review':None,'runtime':'UNVERIFIED'}))
    print(canonical(outputs).decode('ascii'),end='')


if __name__=='__main__':main()
