"""Reviewer-owned manifest, public-library, AST interface and bound data reader."""
import ast
import hashlib
import json
from pathlib import Path
import re
import stat
import subprocess

OUT = Path(__file__).resolve().parent.relative_to(Path.cwd())
NEW = OUT.parent / 'm1-copy-parent-signal-schema-repair-2026-10-25-t3-f3ffc1d7'
rows = []


def check(name, condition):
    rows.append({'name': name, 'equal_or_satisfied': bool(condition)})
    if not condition:
        raise ValueError(name)


def read(path):
    return json.loads(path.read_bytes())


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def raw_json(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=True, allow_nan=False).encode()


def save(name, value):
    (OUT / name).write_text(json.dumps(value, indent=2) + '\n')


manifest = read(NEW / 'packet-manifest.json')['files']
names = {p.name for p in NEW.iterdir()}
check('complete 51-row nonself packet manifest', len(manifest) == 51 and {r['path'] for r in manifest} == names - {'packet-manifest.json', 'SHA256SUMS'})
for r in manifest:
    path = NEW / r['path']
    data = path.read_bytes()
    blob = hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()
    check('all manifest identity/mode fields ' + r['path'], r['bytes'] == len(data) and r['sha256'] == sha(data) and r['raw_git_blob_sha1'] == blob and r['git_mode'] == '100644' and r['working_mode'] == '0644' and stat.S_IMODE(path.stat().st_mode) == 0o644)
summed_names = []
for row in (NEW / 'SHA256SUMS').read_text().splitlines():
    digest, name = row.split('  ', 1)
    summed_names.append(name)
    check('packet checksum ' + name, sha((NEW / name).read_bytes()) == digest)
check('complete unique 52 checksum entries', len(summed_names) == 52 and len(set(summed_names)) == 52 and set(summed_names) == names - {'SHA256SUMS'})

library = []
records = [read(NEW / 'public-stdlib-source-evidence.json')] + read(NEW / 'public-bootstrap-source-evidence.json')['inputs']
for r in records:
    check('only public Xcode library text', r['path'].startswith('/Applications/Xcode.app/Contents/Developer/Library/Frameworks/Python3.framework/Versions/3.9/'))
    data = Path(r['path']).read_bytes()
    check('public library full bytes ' + r['path'], len(data) == r['bytes'] and sha(data) == r['sha256'])
    lines = data.splitlines(keepends=True)
    for span in r['ranges']:
        check('public library exact excerpt ' + r['path'] + ':' + str(span['start']), b''.join(lines[span['start'] - 1:span['end']]).decode() == span['text'])
    library.append(r)
audit = read(NEW / 'direct-signal-library-audit.json')
os_record = audit['os_module']
os_raw = Path(os_record['path']).read_bytes()
check('public os POSIX export full identity', len(os_raw) == os_record['bytes'] and sha(os_raw) == os_record['sha256'])
excerpt = audit['os_posix_binding_excerpt']
check('public os POSIX export excerpt', b''.join(os_raw.splitlines(keepends=True)[excerpt['start'] - 1:excerpt['end']]).decode() == excerpt['text'])
save('public-library-bindings.json', {'records': library, 'os': os_record, 'os_excerpt': excerpt, 'C_implementation_body_read': False, 'target_installed_identity_or_audit_policy': 'UNVERIFIED', 'signal_delivery_reap': 'UNVERIFIED', 'runtime_execution': False})

source = (NEW / 'copy-parent.py.txt').read_bytes()
tree = ast.parse(source.decode(), feature_version=(3, 9))
functions = {n.name: n for n in tree.body if isinstance(n, ast.FunctionDef)}
calls = []
for name, node in functions.items():
    for child in ast.walk(node):
        if isinstance(child, ast.Call):
            calls.append({'function': name, 'line': child.lineno, 'target': ast.unparse(child.func)})
signals = [r for r in calls if r['target'] in ('os.kill', 'proc.kill', 'proc.send_signal')]
waits = [r for r in calls if r['target'].endswith(('.poll', '.wait', '.waitpid'))]
check('one direct os.kill call only in collect', signals == [{'function': 'collect', 'line': 834, 'target': 'os.kill'}])
check('only retained child polls/wait are inside collect', len(waits) == 3 and all(r['function'] == 'collect' and r['target'] in ('proc.poll', 'proc.wait') for r in waits))
check('single authorized common writer call', [r for r in calls if r['target'].startswith('common.') and r['target'] != 'common.time.monotonic'] == [{'function': 'reserved_put', 'line': 707, 'target': 'common.write_bytes'}])
check('only caller stdin and compact console raw writes', sorted((r['function'], r['target']) for r in calls if r['target'] == 'os.write') == [('collect', 'os.write'), ('emit_once', 'os.write')])
save('signal-and-publication-observations.json', {'signals': signals, 'actual_poll_wait_sites': sorted(waits, key=lambda r: r['line']), 'all_calls': sorted(calls, key=lambda r: r['line']), 'manual_control_flow': {'eligibility_poll': 822, 'immediate_observe_exit_sample': 823, 'both_strict_original_end_checks': 827, 'cap_or_deadline_only': 831, 'attempt_flag': 832, 'direct_signal': 834, 'syscall_return_flag': 835, 'original_reap_remainder': [836, 837, 838], 'ESRCH_and_other_exception_retention': [839, 840, 841, 842], 'actual_code_and_reap_before_clock': [202, 203, 204, 205], 'retained_handle_binding': [750, 751, 760, 761, 762], 'no_other_waiter_in_capsule': True}, 'PID_lifetime_limit': 'Same sole direct child, retained in HANDLES until os._exit. No other capsule thread/reaper/handle borrower, waitpid scan, PID lookup or group target. Fresh actual None plus unreaped handle keeps PID ownership under the trusted POSIX child lifecycle and no external/automatic reaper premise. Target bootstrap/signal disposition/audit behavior and scheduling remain unverified.', 'no_signal_delivery_claim': True})

root = read(NEW / 'config.schema.json')


def bound(node):
    if '$ref' in node:
        value = root
        for part in node['$ref'].split('/')[1:]:
            value = value[part]
        return bound(value)
    if 'const' in node:
        return len(raw_json(node['const']))
    if 'enum' in node:
        return max(len(raw_json(v)) for v in node['enum'])
    if 'oneOf' in node or 'anyOf' in node:
        return max(bound(v) for v in node.get('oneOf', node.get('anyOf')))
    if 'allOf' in node:
        return bound(next(v for v in node['allOf'] if '$ref' in v or 'type' in v or 'const' in v))
    kind = node.get('type')
    if kind == 'object':
        check('closed CONFIG bound object', node['additionalProperties'] is False)
        fields = [len(raw_json(k)) + 1 + bound(v) for k, v in node['properties'].items()]
        return 2 + sum(fields) + max(0, len(fields) - 1)
    if kind == 'string':
        match = re.fullmatch(r'\^\[0-9a-f\]\{(\d+)\}\$', node.get('pattern', ''))
        return 2 + int(match.group(1)) if match else 2 + 12 * node['maxLength']
    if kind == 'integer':
        return max(len(str(node['minimum'])), len(str(node['maximum'])))
    if kind == 'boolean':
        return 5
    if kind == 'null':
        return 4
    raise ValueError('unbounded CONFIG schema data')


config_bound = bound(root)
proof = read(OUT / 'template-bindings.json')
token_bound = 2 * config_bound + 2
full_bound = len(source) - proof['token']['bytes'] + token_bound
check('independent finite CONFIG/render bound', (config_bound, token_bound, full_bound) == (9977, 19956, 138912) and full_bound <= 1048576)
save('byte-bound-observations.json', {'canonical_CONFIG_bound': config_bound, 'Python_string_token_bound': token_bound, 'full_render_bound': full_bound, 'actual_future_render': None, 'method': 'Walk complete closed CONFIG schema. Count canonical fixed bytes and all possible fields; use at most12 ensure_ascii bytes per Unicode code point, fixed hash widths, integer endpoints, max of alternatives, and the containing review ref for allOf. Other allOf restrictions can only reduce the bound. No receipt or rendered future CONFIG is constructed.', 'compact_console_maximum': 149, 'compact_console_reason': '2 braces + four JSON strings for keys/values lengths including punctuation: v copy-parent.v1, state longest COPY_DATA_COMPLETE_EXTERNAL_PENDING, r [262144, quoted64hex] + LF =149; failed/null shorter', 'console_actual_cap': 512, 'receipt_actual_encoded_cap': 262144, 'all_streams_fit_reservation_claim': False})

historical_patch = 'prior-from-cleared-source-launcher.patch'
base = ['git', 'diff', '--check', '7e5de706e7457de5f4880cdef180b27536dc0d5e', 'c82cc12b8a2d709d8ce1e384545475159b24d2a9', '--']
specs = {'full': [str(NEW)], 'patch_excluded': [str(NEW), ':(exclude)' + str(NEW) + '/*.patch'], 'new_repair_patches': [str(NEW / n) for n in ('from-85c668f.patch', 'from-85c668f-source.patch', 'from-85c668f-schema.patch')]}
whitespace = {}
for label, paths in specs.items():
    result = subprocess.run(base + paths, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    whitespace[label] = {'argv': base + paths, 'exit_code': result.returncode, 'stdout': result.stdout.decode(), 'stderr': result.stderr.decode()}
check('full whitespace remains exit2', whitespace['full']['exit_code'] == 2)
check('patch excluded whitespace exit0', whitespace['patch_excluded']['exit_code'] == 0)
check('new repair patches whitespace exit0', whitespace['new_repair_patches']['exit_code'] == 0)
check('42 historical blank patch context diagnostics only', whitespace['full']['stdout'].count(': trailing whitespace.') == 42 and all(historical_patch in line for line in whitespace['full']['stdout'].splitlines() if ': trailing whitespace.' in line))
save('whitespace-observations.json', whitespace)
save('closure-observations.json', {'execution': False, 'count': len(rows), 'observations': rows, 'failed': [r for r in rows if not r['equal_or_satisfied']]})
print(json.dumps({'observations': len(rows), 'manifest': len(manifest), 'checksums': len(summed_names), 'direct_signals': len(signals), 'whitespace': {k: v['exit_code'] for k, v in whitespace.items()}, 'render_bound': full_bound}))
