"""Read-only parent verification of actual staged bytes. ASTs remain data."""
import ast
import hashlib
import json
import os
from pathlib import Path
import stat
import sys

OUT = Path(__file__).resolve().parent
REPO = OUT.parents[2]
BASE = Path('/Users/serge/.local/state/openergy-verification/m1-source-metadata-launch-prepared-20261025-7ae23486940a')


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def canonical(value):
    return (json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=True, allow_nan=False)+'\n').encode('ascii')


def assignment(raw, name):
    tree = ast.parse(raw)
    nodes = [n.value for n in tree.body if isinstance(n, ast.Assign)
             and any(isinstance(t, ast.Name) and t.id == name for t in n.targets)]
    assert len(nodes) == 1
    return tree, nodes[0]


def span(raw, node):
    lines = raw.splitlines(keepends=True)
    return (sum(map(len, lines[:node.lineno-1]))+node.col_offset,
            sum(map(len, lines[:node.end_lineno-1]))+node.end_col_offset)


def method(raw):
    tree, node = assignment(raw, 'CONFIG_TEXT')
    left, right = span(raw, node)
    node.value = '<CONFIG_TEXT>'
    return {'schema':'openergy.single-config-render-method.v1','template':{'bytes':len(raw),'sha256':sha(raw)},
        'token_start':left,'token_end':right,'token_sha256':sha(raw[left:right]),
        'prefix_sha256':sha(raw[:left]),'suffix_sha256':sha(raw[right:]),
        'normalized_ast_sha256':sha(ast.dump(tree,annotate_fields=True,include_attributes=False).encode('ascii')),
        'normalization':'ast.dump annotate_fields=True include_attributes=False; CONFIG_TEXT value=<CONFIG_TEXT>',
        'render':'one ASCII Python STRING via json.dumps(JSON-text,ensure_ascii=True); exact restore',
        'excludes':['own reference','bound config','future full rendered source hash']}


def render_check(template, actual, config):
    old_tree, old = assignment(template, 'CONFIG_TEXT')
    new_tree, new = assignment(actual, 'CONFIG_TEXT')
    assert json.loads(new.value) == config
    a,b = span(template,old)
    c,d = span(actual,new)
    assert a == c and actual[:c] + template[a:b] + actual[d:] == template
    old.value = new.value = '<CONFIG_TEXT>'
    assert ast.dump(old_tree,include_attributes=False) == ast.dump(new_tree,include_attributes=False)


os.setpriority(os.PRIO_PROCESS,0,15)
assert sys.version_info[:2] == (3,15) and sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode
staging = json.loads((OUT/'staging.json').read_bytes())
assert len(staging['files']) == 13
for leaf, expected in staging['files'].items():
    path = BASE/leaf
    old = path.lstat()
    raw = path.read_bytes()
    assert stat.S_ISREG(old.st_mode) and old.st_uid == os.getuid() and old.st_nlink == 1 and stat.S_IMODE(old.st_mode) == 0o400
    assert (old.st_dev,old.st_ino,old.st_size,old.st_mtime_ns,old.st_ctime_ns) == (lambda s:(s.st_dev,s.st_ino,s.st_size,s.st_mtime_ns,s.st_ctime_ns))(path.lstat())
    assert {'bytes':len(raw),'sha256':sha(raw)} == expected
config = json.loads((OUT/'driver-config.bound.json').read_bytes())
driver_template = (REPO/'docs/verification/m1-prepare-local-parent-driver-2026-10-25-t3-def6ed35/driver.py.txt').read_bytes()
driver_actual = (BASE/'prepare-launch/prepare-local-parent-driver.py').read_bytes()
render_check(driver_template,driver_actual,config)
capsule_template = (REPO/'docs/verification/m1-prepare-parent-root-layout-repair-2026-10-25-t3-1f8914f8/prepare-parent.py.txt').read_bytes()
render_check(capsule_template,(BASE/'prepare-launch/prepare.payload.py').read_bytes(),json.loads((OUT/'capsule-config.bound.json').read_bytes()))
assessor = (BASE/'tools/prepare-transport-assessor.direct-signal.template.py.txt').read_bytes()
actual_method = method(assessor)
assert sha(canonical(actual_method)) == config['assessor_method_sha256'] == '9d9335d13a74ba57bbae2dfd7787c66e4651ed888ddcf70332cf9c12a32d7848'
association = json.loads((BASE/'prepare-launch/root-source-association.proof.json').read_bytes())
assert association['driver_normalized_ast_sha256'] == method(driver_actual)['normalized_ast_sha256']
assert association['proofs'] == {k:v for k,v in config['proofs'].items() if k!='root_source_association'}
assert association['dependencies'] == config['dependencies'] and association['runtime'] == config['runtime']
for name in ['prepare-local-parent-once.json','prepare-parent-request.json','prepare-parent-exit.json','prepare-run','prepare-transport-assessment.json']:
    assert not os.path.lexists(BASE/name)
for name in ['prepare-transport-assessor.py','prepare-assessor-render-method.json','prepare-assessor-render-proof.json']:
    assert not os.path.lexists(BASE/'tools'/name)
value = {'schema':'openergy.prepare-staged-data-verification.v1','actual_staged_files_verified':13,
    'exact_payload_and_driver_constant_renders':True,'qualified_315_assessor_method':actual_method,
    'qualified_315_assessor_method_sha256':sha(canonical(actual_method)),
    'qualified_315_driver_normalized_ast_sha256':method(driver_actual)['normalized_ast_sha256'],
    'reviewer_39_AST_observation':{'assessor_normalized_ast_sha256':'4b25a4245574548453beecc94ca9c3891d25223769de380b0165cafbbb9d657f',
        'driver_normalized_ast_sha256':'81f8617d3bcc8e0fcd0700fa918528bbcf8332bb792206b2956e660e579d2924',
        'method_sha256':'bef51d00e4635867714e48fe57b229d2111474e5284ffbdd0ec43b3521b3103a'},
    'difference':'CPython3.15 AST FunctionDef adds type_params. Complete normalization uses actual qualified runtime, not the reviewer3.9 hash.',
    'runtime':sys.version,'function_fields':list(ast.FunctionDef._fields),
    'new_attempt_output_names_absent':True,'actual_prepare_invocations':0,'supplied_source_executed':False}
raw = canonical(value)
fd = os.open(OUT/'staged-data-verification.json',os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
try:
    assert os.write(fd,raw)==len(raw)
    os.fsync(fd)
finally:
    os.close(fd)
print(canonical({'staged_files':13,'qualified_method_matches_config':True,'exact_renders':True,'target_attempts':0}).decode('ascii'),end='')
