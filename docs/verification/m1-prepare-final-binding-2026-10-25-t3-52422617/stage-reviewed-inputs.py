"""Parent-owned constant renderer and local staging. No target command runs.

Supplied Python is parsed as data only. Staging is exclusive, with no overwrite.
The separate reviewed driver is the sole future preparation entry.
"""
import ast
import hashlib
import io
import json
import os
from pathlib import Path
import stat
import subprocess
import sys
import time
import tokenize

REPO = Path(__file__).resolve().parents[3]
OUT = Path(__file__).resolve().parent
BASE = Path('/Users/serge/.local/state/openergy-verification/m1-source-metadata-launch-prepared-20261025-7ae23486940a')
LAUNCH = BASE / 'prepare-launch'
TOOLS = BASE / 'tools'
DOCS = REPO / 'docs/verification'
CAPSULE = DOCS / 'm1-prepare-parent-root-layout-repair-2026-10-25-t3-1f8914f8'
COMPOSITION = DOCS / 'm1-local-direct-signal-composition-2026-10-25-t3-52422617'
DRIVER = DOCS / 'm1-prepare-local-parent-driver-2026-10-25-t3-def6ed35'
CAPSULE_CANDIDATE = '02f94afbe3e1001b8d3e8fc4d79976475485f661'
CAPSULE_REVIEW = '63b6a3b564d6a557f60e0e2b0e4a1943ffa2b19f'
COMPOSITION_CANDIDATE = '8bde42c5d18bb548aa435ee8f25321f236150da8'
COMPOSITION_REVIEW = '4fa8b37b4d8cdbb357287a36ce15a251594674cc'
READS = []


def canonical(value):
    return (json.dumps(value, sort_keys=True, separators=(',', ':'),
                       ensure_ascii=True, allow_nan=False) + '\n').encode('ascii')


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def ref(raw):
    return {'bytes': len(raw), 'sha256': sha(raw)}


def stamp(info):
    return (info.st_dev, info.st_ino, info.st_mode, info.st_uid, info.st_gid,
            info.st_nlink, info.st_size, info.st_mtime_ns, info.st_ctime_ns)


def read(path, cap=1048576, mode=None):
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    try:
        initial = os.fstat(fd)
        assert stat.S_ISREG(initial.st_mode) and initial.st_nlink == 1
        assert initial.st_uid == os.getuid() and 0 <= initial.st_size <= cap
        if mode is not None:
            assert stat.S_IMODE(initial.st_mode) == mode
        parts, seen = [], 0
        while True:
            chunk = os.read(fd, min(65536, cap + 1 - seen))
            seen += len(chunk)
            assert seen <= cap
            if not chunk:
                break
            parts.append(chunk)
        raw = b''.join(parts)
        assert len(raw) == initial.st_size
        assert stamp(initial) == stamp(os.fstat(fd)) == stamp(os.stat(path, follow_symlinks=False))
        READS.append({'bytes': len(raw), 'sha256': sha(raw)})
        return raw
    finally:
        os.close(fd)


def unique(pairs):
    value = {}
    for key, item in pairs:
        assert key not in value
        value[key] = item
    return value


def load(raw):
    return json.loads(raw, object_pairs_hook=unique,
                      parse_constant=lambda value: (_ for _ in ()).throw(ValueError(value)))


def literal(raw, name):
    tree = ast.parse(raw.decode('ascii'))
    matches = [n.value for n in tree.body if isinstance(n, ast.Assign)
               and any(isinstance(t, ast.Name) and t.id == name for t in n.targets)]
    assert len(matches) == 1
    return tree, matches[0]


def span(raw, node):
    lines = raw.splitlines(keepends=True)
    return (sum(map(len, lines[:node.lineno - 1])) + node.col_offset,
            sum(map(len, lines[:node.end_lineno - 1])) + node.end_col_offset)


def method(raw):
    tree, node = literal(raw, 'CONFIG_TEXT')
    assert isinstance(node, ast.Constant) and type(node.value) is str
    left, right = span(raw, node)
    lines = raw.splitlines(keepends=True)
    exact = [t for t in tokenize.tokenize(io.BytesIO(raw).readline)
             if t.type == tokenize.STRING
             and sum(map(len, lines[:t.start[0]-1])) + t.start[1] == left
             and sum(map(len, lines[:t.end[0]-1])) + t.end[1] == right]
    assert len(exact) == 1
    node.value = '<CONFIG_TEXT>'
    return {'schema': 'openergy.single-config-render-method.v1',
            'template': ref(raw), 'token_start': left, 'token_end': right,
            'token_sha256': sha(raw[left:right]), 'prefix_sha256': sha(raw[:left]),
            'suffix_sha256': sha(raw[right:]),
            'normalized_ast_sha256': sha(ast.dump(tree, annotate_fields=True, include_attributes=False).encode('ascii')),
            'normalization': 'ast.dump annotate_fields=True include_attributes=False; CONFIG_TEXT value=<CONFIG_TEXT>',
            'render': 'one ASCII Python STRING via json.dumps(JSON-text,ensure_ascii=True); exact restore',
            'excludes': ['own reference', 'bound config', 'future full rendered source hash']}


def render(raw, config, cap):
    proof = method(raw)
    left, right = proof['token_start'], proof['token_end']
    text = canonical(config)[:-1].decode('ascii')
    rendered = raw[:left] + json.dumps(text, ensure_ascii=True).encode('ascii') + raw[right:]
    tree, node = literal(rendered, 'CONFIG_TEXT')
    a, b = span(rendered, node)
    assert node.value == text and a == left and len(rendered) <= cap
    assert rendered[:a] == raw[:left] and rendered[b:] == raw[right:]
    assert rendered[:a] + raw[left:right] + rendered[b:] == raw
    node.value = '<CONFIG_TEXT>'
    assert sha(ast.dump(tree, annotate_fields=True, include_attributes=False).encode('ascii')) == proof['normalized_ast_sha256']
    # The remote capsule's grammar remains Python3.9-compatible after this one string edit.
    ast.parse(rendered.decode('ascii'), feature_version=(3, 9))
    return rendered, dict(schema='openergy.parent-single-config-render.v1',
        method=proof, config=ref(text.encode('ascii')), rendered=ref(rendered),
        single_string_token=True, exact_restore=True, prefix_suffix_exact=True,
        normalized_ast_equal=True, supplied_source_executed=False)


def public(path, commit, expected=None):
    raw = read(path)
    frozen = subprocess.check_output(['git', '-C', str(REPO), 'show', commit + ':' + str(path.relative_to(REPO))])
    assert raw == frozen
    if expected:
        assert sha(raw) == expected
    return raw


def put(path, raw, mode):
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW | os.O_NONBLOCK, mode)
    try:
        assert os.write(fd, raw) == len(raw)
        os.fsync(fd)
    finally:
        os.close(fd)
    parent = os.open(path.parent, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        os.fsync(parent)
    finally:
        os.close(parent)
    assert read(path, mode=mode) == raw


def main():
    os.umask(0o077)
    os.setpriority(os.PRIO_PROCESS, 0, 15)
    assert os.getpriority(os.PRIO_PROCESS, 0) == 15
    assert sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode and not sys.flags.optimize
    assert sys.platform == 'darwin' and sys.version_info[:2] == (3, 15)
    runtime = {'resolved_python': os.path.realpath(sys.executable), 'full_version': sys.version}
    for path in [BASE, LAUNCH, TOOLS]:
        info = path.lstat()
        assert stat.S_ISDIR(info.st_mode) and info.st_uid == os.getuid() and stat.S_IMODE(info.st_mode) == 0o700
    old_names = ['config.pending.json', 'pending-bindings.json', 'observer.template.py.txt',
                 'observer-template.binding.json', 'observer.reviewed.template.py.txt',
                 'observer-reviewed-template.binding.json']
    old = {name: ref(read(LAUNCH / name)) for name in old_names}
    capsule = public(CAPSULE / 'prepare-parent.py.txt', CAPSULE_CANDIDATE,
                     'b3fbd793a0709c7857e34efca667a081a0486accfd0f3ad13a8068484b7eb8d9')
    capsule_review = public(DOCS / 'm1-prepare-parent-root-layout-review-2026-10-25-t3-98186f1b/REVIEW.md',
                           CAPSULE_REVIEW, 'dc8e986c6a3e914b65a13686fc9f1a7e15702e8548e545075d6967f340541c4e')
    composition_review = public(DOCS / 'm1-prepare-local-parent-composition-review-2026-10-25-t3-4d289a8c/REVIEW.md', COMPOSITION_REVIEW)
    templates = {name: public(COMPOSITION / (name + '.py.txt'), COMPOSITION_CANDIDATE)
                 for name in ['collector', 'observer', 'assessor']}
    expected = {'collector': 'f1fa00b2ec61f6e4996b5938388f345a8ddcfb205455bc4482ce25711824f371',
                'observer': '7b7e9dd96e0d7e3c1168c8dd2f5acc266967441061fefe2d29622add9729d5a6',
                'assessor': '517c7cc82336d1a924e1578d3c13c68960834554596a668e80013fd8818ed913'}
    assert all(sha(templates[name]) == expected[name] for name in templates)
    driver = public(DRIVER / 'driver.py.txt', '7c92909fcd41580183c224549a0adb768602f9f8',
                    '5b0fe6c9fada122bf8dc30b0814d49959f4f7863da32b3c444792a978d0a2a46')
    config = load(read(LAUNCH / 'config.pending.json', mode=0o600))
    config['template_sha256'] = sha(capsule)
    config['reviews']['capsule'] = dict(candidate_commit=CAPSULE_CANDIDATE,
        source_sha256=sha(capsule), review_commit=CAPSULE_REVIEW,
        review_sha256=sha(capsule_review), verdict='CLEAR')
    for name, binding in config['completed_SOURCE'].items():
        assert ref(read(BASE / ('retained-SOURCE-' + name), mode=0o600)) == binding
    for name, binding in config['artifact_publication'].items():
        assert ref(read(BASE / 'artifact-binding/retained' / name, mode=0o600)) == binding
    source_raw = read(BASE / 'retained-SOURCE-metadata-source-result.json', mode=0o600)
    assert len(source_raw) == 327813 and sha(source_raw) == '7482a637159e0389c5cd4c505778919bb49354437bc9feb1cf008d9db395712d'
    source_validation = load(read(BASE / 'SOURCE-full-validation.json'))
    assert source_validation['all50_saved_basis_matches'] is True
    assert type(source_validation['all_raw_status_comparison_files_retained']) is int
    assert source_validation['all_raw_status_comparison_files_retained'] == 212
    assert source_validation['D4_actual_collector0_before_original_135s'] is True
    assert source_validation['no_timeout_kill_overflow_drop_or_publication_error'] is True
    _, raw_node = literal(capsule, 'SOURCE_RAWS')
    rows = ast.literal_eval(raw_node)  # Literal DATA, never Python module execution.
    assert len(rows) == 212
    for row in rows:
        actual = read(BASE / 'retained-SOURCE-records' / row['leaf'], mode=0o600)
        assert ref(actual) == {k: row[k] for k in ['bytes', 'sha256']}
    publisher = load(read(BASE / 'artifact-binding/retained/artifact-binding-receipt.json', mode=0o600))
    for name, binding in publisher['files'].items():
        assert ref(read(BASE / 'artifact-binding/retained' / name, mode=0o600)) == {k: binding[k] for k in ['bytes', 'sha256']}
    profile_raw = read(BASE / 'm1-profile.json', 65536, 0o400)
    assert len(profile_raw) == 821 and sha(profile_raw) == 'dc9ff86519970ad740961012b0740b1b1a7aab3135a6fe79e47aa0de071acf4f'
    profile = load(profile_raw)
    guard_raw = read(Path(profile['source_guard']['path']), 16384, 0o600)
    assert len(guard_raw) == 1563 and sha(guard_raw) == profile['source_guard']['sha256'] == '9e0b9a5afb47b7755d1ebaf5f50edf45cf0a7372baaa2f83b83d9255011aed4c'
    assert len(profile['argv']) == 29 and profile['argv'] == load(guard_raw)['argv']
    assert profile['argv'][-1] == '/usr/bin/python3 -I -S -B -'
    profile_proof = dict(schema='openergy.whole-profile-guard-inspection.v1',
        profile=ref(profile_raw), original_guard=ref(guard_raw), complete_29_tokens_equal=True,
        final_SYSTEM_isolated_stdin_entry_equal=True, private_bodies_included=False,
        no_new_connection=True, no_known_hosts_body_read=True)
    # These TRUE values bind obligations owned by this parent, not completed future acts.
    config['parent'] = {name: True for name in config['parent']}
    payload, payload_proof = render(capsule, config, 1048576)
    payload_proof.update(candidate=CAPSULE_CANDIDATE, review=CAPSULE_REVIEW,
        review_sha256=sha(capsule_review), execution=False,
        future_exit_EOF_retention_and_original_enclosure_owned=True,
        future_actual_prepare_exit_proved=False, no_product_invocation=True)
    observer = templates['observer']
    rendered_observer = observer
    edits = []
    for name, value in [('PAYLOAD_BYTES', len(payload)), ('PAYLOAD_SHA256', sha(payload))]:
        _, node = literal(observer, name)
        assert isinstance(node, ast.Constant) and node.value is None
        a, b = span(observer, node)
        edits.append((a, b, json.dumps(value).encode('ascii')))
    for a, b, token in sorted(edits, reverse=True):
        rendered_observer = rendered_observer[:a] + token + rendered_observer[b:]
    restored, actual_tree = rendered_observer, ast.parse(rendered_observer.decode('ascii'))
    reverse = []
    for name in ['PAYLOAD_BYTES', 'PAYLOAD_SHA256']:
        _, original_node = literal(observer, name)
        matches = [n.value for n in actual_tree.body if isinstance(n, ast.Assign)
                   and any(isinstance(t, ast.Name) and t.id == name for t in n.targets)]
        assert len(matches) == 1
        node = matches[0]
        reverse.append((*span(rendered_observer, node), observer[slice(*span(observer, original_node))]))
        node.value = None
    for a, b, token in sorted(reverse, reverse=True):
        restored = restored[:a] + token + restored[b:]
    assert restored == observer and len(rendered_observer) <= 16384
    assert ast.dump(actual_tree, include_attributes=False) == ast.dump(ast.parse(observer), include_attributes=False)
    observer_proof = dict(schema='openergy.parent-observer-render.v1',
        template=ref(observer), rendered=ref(rendered_observer), payload=ref(payload),
        exactly_two_constant_literals=True, exact_restore=True, normalized_ast_equal=True,
        supplied_source_executed=False, composition_candidate=COMPOSITION_CANDIDATE,
        review=COMPOSITION_REVIEW, review_sha256=sha(composition_review))
    dependency_proof = dict(schema='openergy.prepare-dependency-composition-inspection.v1',
        candidate=COMPOSITION_CANDIDATE, review=COMPOSITION_REVIEW,
        review_report=ref(composition_review), sources={n: ref(v) for n, v in templates.items()},
        capsule=dict(candidate=CAPSULE_CANDIDATE, source=ref(capsule), review=CAPSULE_REVIEW,
                     report=ref(capsule_review)),
        historical_fixed_source_bindings_mean='completed SOURCE basis only; not active local adapter',
        active_adapter='transport-direct-signal.py', all_four_complete_sources_and_reports_inspected=True,
        verdict_scope='BOUNDED SOURCE CLEAR; actual renders and runtime separate', execution=False,
        future_prepare_copy_controller_product_acceptance=False)
    proofs = {'payload_render': canonical(payload_proof), 'observer_render': canonical(observer_proof),
              'profile_guard': canonical(profile_proof), 'driver_review': composition_review,
              'assessor_review': composition_review, 'dependency_composition': canonical(dependency_proof)}
    driver_config = load(read(DRIVER / 'config.template.json'))
    driver_config.update(payload=ref(payload), observer_rendered=ref(rendered_observer),
        capsule={'candidate': CAPSULE_CANDIDATE, 'review': CAPSULE_REVIEW, 'review_sha256': sha(capsule_review)},
        runtime=runtime, assessor_method_sha256=sha(canonical(method(templates['assessor']))))
    deps = {name: dict(candidate=COMPOSITION_CANDIDATE, review=COMPOSITION_REVIEW,
                      review_sha256=sha(composition_review), template=ref(templates[name]))
            for name in templates}
    deps['composition'] = dict(candidate=COMPOSITION_CANDIDATE, review=COMPOSITION_REVIEW,
                               review_sha256=sha(composition_review))
    driver_config['dependencies'] = deps
    driver_config['proofs'] = {name: ref(raw) for name, raw in proofs.items()}
    association = load(read(DRIVER / 'root-source-association.template.json'))
    association.update(driver_normalized_ast_sha256=method(driver)['normalized_ast_sha256'],
        dependencies=deps, payload=ref(payload), observer_rendered=ref(rendered_observer),
        capsule=driver_config['capsule'], runtime=runtime,
        proofs=dict(driver_config['proofs']), assessor_method_sha256=driver_config['assessor_method_sha256'])
    proofs['root_source_association'] = canonical(association)
    driver_config['proofs']['root_source_association'] = ref(proofs['root_source_association'])
    actual_driver, driver_proof = render(driver, driver_config, 131072)
    driver_proof.update(candidate='7c92909fcd41580183c224549a0adb768602f9f8',
        review=COMPOSITION_REVIEW, review_sha256=sha(composition_review), runtime=runtime,
        future_actual_tool_exit_EOF_and_later_causal_upper_bound_required=True,
        execution=False)
    staging = {LAUNCH / 'prepare.payload.py': payload,
               LAUNCH / 'prepare-local-exit-observer.py': rendered_observer,
               LAUNCH / 'prepare-local-parent-driver.py': actual_driver,
               LAUNCH / 'observer.direct-signal.template.py.txt': observer,
               TOOLS / 'prepare-transport-assessor.direct-signal.template.py.txt': templates['assessor'],
               TOOLS / 'transport-direct-signal.py': templates['collector']}
    proof_leaves = {'payload_render':'payload-render.proof.json', 'observer_render':'observer-render.proof.json',
        'profile_guard':'whole-profile-guard.proof.json', 'driver_review':'driver-full-review.txt',
        'assessor_review':'assessor-full-review.txt', 'dependency_composition':'dependency-composition.proof.json',
        'root_source_association':'root-source-association.proof.json'}
    staging.update({LAUNCH / proof_leaves[name]: raw for name, raw in proofs.items()})
    for path in staging:
        assert not os.path.lexists(path)
    _, new_node = literal(driver, 'NEW')
    for label, leaf in ast.literal_eval(new_node):
        assert not os.path.lexists({'base':BASE, 'tools':TOOLS, 'launch':LAUNCH}[label] / leaf)
    for path, raw in staging.items():
        put(path, raw, 0o400)
    assert old == {name: ref(read(LAUNCH / name)) for name in old_names}
    for name, raw in {'capsule-config.bound.json':canonical(config),
                      'driver-config.bound.json':canonical(driver_config),
                      'driver-render-proof.json':canonical(driver_proof),
                      'runtime-qualified.json':canonical(dict(runtime=runtime, uid=os.getuid(),
                          nice=15, flags={'isolated':sys.flags.isolated,'no_site':sys.flags.no_site,
                          'dont_write_bytecode':sys.flags.dont_write_bytecode,'optimize':sys.flags.optimize},
                          monotonic_implementation=time.get_clock_info('monotonic').implementation,
                          resolution_seconds=time.get_clock_info('monotonic').resolution)),
                      'staging.json':canonical(dict(schema='openergy.prepare-final-local-staging.v1',
                          files={str(p.relative_to(BASE)):ref(r) for p,r in staging.items()},
                          old_six_launch_inputs_preserved=old, local_bounded_read_opens=len(READS),
                          local_bounded_read_bytes=sum(x['bytes'] for x in READS),
                          actual_prepare_invocations=0, actual_copy_invocations=0,
                          actual_controller_invocations=0, acceptance=False))}.items():
        put(OUT / name, raw, 0o600)
    print(canonical({'staged':len(staging), 'payload':ref(payload), 'observer':ref(rendered_observer),
                     'driver':ref(actual_driver), 'target_commands':0, 'prepare_invocations':0}).decode('ascii'), end='')


if __name__ == '__main__':
    main()
