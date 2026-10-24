"""One native package command. Private build state; original caches read-only.

Not a reusable release tool. Owned verification evidence for t3-8c51bae6 only.
Modes: prepare, run, audit. An exclusive guard prevents another package call.
"""
import ctypes
import gzip
import hashlib
import json
import os
from pathlib import Path
import plistlib
import shutil
import signal
import socket
import stat
import subprocess
import sys
import time
import traceback
import zipfile

ROOT = Path(__file__).resolve().parent
REPO = Path('/Users/serge/.t3/worktrees/openergy/t3-8c51bae6')
OLD = Path('/var/folders/s6/_07cr4w13vsg_myjkv9ldpqc0000gn/T/oe-sparkle-combined-20261023-a_es9dei')
PRIOR = REPO / 'docs/verification/release-priority-2026-10-23'
PYTHON = '/Users/serge/.local/bin/python3.11'
SOURCE = 'd3b30e1719dae5c885ded57cc7523b92d9686adc'
BASE = 'b713d7fbe480a1a3ecadd737ae5de58256169c66'
PIN = 'eef1a539a373c1f1a320624b1130fc5de7b2e100'
PORTS = (65420, 65429)
SEEDS = ['artifacts/sparkle/Sparkle', 'checkouts/Sparkle', 'repositories/Sparkle-09d89c53']


def save(name, value):
    with (ROOT / name).open('x') as output:
        json.dump(value, output, indent=2, sort_keys=True)
        output.write('\n')


def load(name):
    return json.loads((ROOT / name).read_text())


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def command(argv, **kwargs):
    return subprocess.run(argv, cwd=REPO, capture_output=True, text=True, check=True,
                          env={**os.environ, 'GIT_OPTIONAL_LOCKS': '0',
                               'GIT_CONFIG_NOSYSTEM': '1', 'GIT_CONFIG_GLOBAL': '/dev/null'}, **kwargs).stdout.strip()


def inventory(base):
    result = {}
    for path in sorted(base.rglob('*')):
        name = str(path.relative_to(base))
        if path.is_symlink():
            result[name] = {'link': os.readlink(path)}
        elif path.is_file():
            result[name] = {'sha256': sha(path), 'bytes': path.stat().st_size}
    return result


def metadata(base):
    result = {}
    for path in [base] + sorted(base.rglob('*')):
        info = path.lstat()
        item = dict(mode=stat.S_IMODE(info.st_mode), kind=stat.S_IFMT(info.st_mode),
                    uid=info.st_uid, gid=info.st_gid, nlink=info.st_nlink,
                    flags=info.st_flags, mtime_ns=info.st_mtime_ns)
        if path.is_symlink():
            item['link'] = os.readlink(path)
        result[str(path.relative_to(base))] = item
    return result


def old_inventory():
    return {name: inventory(OLD / name) for name in ('swift', 'cargo')}


def old_metadata():
    return {name: metadata(OLD / name) for name in ('swift', 'cargo')}


def source_inventory():
    names = command(['git', 'ls-files', '-z']).split('\0')
    return {name: sha(REPO / name) for name in names if name and
            not name.startswith('docs/verification/') and (REPO / name).is_file()}


def prior_integrity():
    entries = json.loads((PRIOR / 'copy-ledger.json').read_text())
    records = []
    for item in entries:
        zipped = PRIOR / item['path']
        raw = gzip.decompress(zipped.read_bytes())
        original = Path(item['private_original'])
        record = dict(path=item['path'], compressed_sha256=sha(zipped),
                      compressed_bytes=zipped.stat().st_size, private_original=str(original),
                      original_sha256=sha(original), original_bytes=original.stat().st_size,
                      decompressed_sha256=hashlib.sha256(raw).hexdigest())
        assert record['compressed_sha256'] == item['compressed_sha256']
        assert record['compressed_bytes'] == item['compressed_bytes']
        assert record['original_sha256'] == record['decompressed_sha256'] == item['sha256']
        assert record['original_bytes'] == len(raw) == item['bytes']
        records.append(record)
    return dict(ledger=records, complete_prior_folder=inventory(PRIOR))


def port_check(name):
    # No SO_REUSEADDR; save the state before the FIRST exclusive bind attempt.
    state = subprocess.run(['/usr/sbin/lsof', '-nP', '-iTCP:%d-%d' % PORTS],
                           text=True, capture_output=True)
    result = dict(range=list(PORTS), lsof_before=dict(exit=state.returncode,
                  stdout=state.stdout, stderr=state.stderr), attempts=[])
    held = []
    try:
        for port in range(PORTS[0], PORTS[1] + 1):
            current = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            held.append(current)
            try:
                current.bind(('127.0.0.1', port))
                current.listen(1)
                result['attempts'].append(dict(port=port, first_exclusive_bind=True))
            except OSError as error:
                result['attempts'].append(dict(port=port, first_exclusive_bind=False, error=repr(error)))
                raise
    finally:
        for current in held:
            current.close()
        save(name, result)


def environment():
    toolchain = '/Users/serge/.rustup/toolchains/1.99.1-aarch64-apple-darwin/bin'
    return dict(PATH=toolchain + ':/usr/bin:/bin:/usr/sbin:/sbin', LANG='C',
        HOME=str(ROOT / 'home'), CFFIXED_USER_HOME=str(ROOT / 'home'),
        TMPDIR=str(ROOT / 'tmp') + '/', XDG_CACHE_HOME=str(ROOT / 'xdg-cache'),
        XDG_CONFIG_HOME=str(ROOT / 'xdg-config'), CARGO_HOME=str(ROOT / 'cargo-home'),
        RUSTUP_HOME='/Users/serge/.rustup', RUSTUP_TOOLCHAIN='1.99.1',
        CARGO_BUILD_JOBS='2', CARGO_NET_OFFLINE='true', CARGO_TARGET_DIR=str(ROOT / 'cargo-target'),
        SWIFT_SCRATCH_PATH=str(ROOT / 'swift'), CLANG_MODULE_CACHE_PATH=str(ROOT / 'module-cache'),
        SWIFTPM_MODULECACHE_OVERRIDE=str(ROOT / 'module-cache'),
        SWIFT_MODULECACHE_PATH=str(ROOT / 'module-cache'),
        MACOSX_DEPLOYMENT_TARGET='15.0', PYTHON=PYTHON, PYTHONPATH=str(ROOT / 'observe'),
        PYTHONDONTWRITEBYTECODE='1', OPENERGY_BUILD='3', OPENERGY_REQUIRE_UPDATES='1',
        GIT_OPTIONAL_LOCKS='0', GIT_CONFIG_NOSYSTEM='1', GIT_CONFIG_GLOBAL='/dev/null',
        GIT_TERMINAL_PROMPT='0')


def prepare():
    assert command(['git', 'rev-parse', 'HEAD']) == BASE
    assert command(['git', 'status', '--porcelain']) == ''
    diff = command(['git', 'diff', '--name-only', SOURCE, 'HEAD']).splitlines()
    assert all(name.startswith('docs/verification/') for name in diff)
    save('scope.json', dict(base=BASE, production_source=SOURCE, evidence_destination=
        'docs/verification/release-priority-isolated-2026-10-23-t3-8c51bae6',
        package_invocations_allowed=1, watchdog_seconds=900, cleanup_seconds=60,
        architecture='arm64', priority='absolute NI15; caller also NI15',
        excluded='GUI/input/clipboard/update/relaunch/installer/keys/notary/timestamp/internet/services/DNS/production IPC/TLS/24800/5353/Docker/push/publication/provider-stopped tasks',
        observation='read-only sys.setprofile hook; no functions or results replaced',
        metadata_limit='New before/after only. No claim about full old prebreach metadata, ACLs, xattrs, atime or ctime.',
        immutable_old_read_inputs=[str(OLD / 'swift' / name) for name in SEEDS] +
                                  [str(OLD / 'swift/workspace-state.json')],
        reference_recipe='/Users/serge/.t3/worktrees/openergy/t3-54de37ad/docs/verification/private-update-bootstrap-repair-2026-10-23-t3-54de37ad/raw/build-once.py',
        adaptation='Read as data only. Selected seed + new state. No import/execution of old controllers.'))
    save('source-before.json', source_inventory())
    save('prior-before.json', prior_integrity())
    before = old_inventory()
    save('old-cache-before.json', before)
    assert before == json.loads(gzip.decompress((PRIOR / 'raw/cache-after-recovery.json.gz').read_bytes()))
    save('old-metadata-before.json', old_metadata())
    for name in ('home', 'tmp', 'module-cache', 'xdg-cache', 'xdg-config', 'cargo-home', 'cargo-target', 'swift'):
        (ROOT / name).mkdir(mode=0o700)
    # Cargo: registry only; no credentials, config, bin, or previous target output.
    shutil.copytree(Path('/Users/serge/.cargo/registry'), ROOT / 'cargo-home/registry', symlinks=True)
    for relative in SEEDS:
        destination = ROOT / 'swift' / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copytree(OLD / 'swift' / relative, destination, symlinks=True)
    save('seed-copy-exact.json', {name: inventory(ROOT / 'swift' / name) for name in SEEDS})
    for relative in SEEDS:
        assert inventory(ROOT / 'swift' / relative) == inventory(OLD / 'swift' / relative)
    finish_prepare()


def finish_prepare():
    assert not (ROOT / 'package-once.guard').exists()
    assert old_inventory() == load('old-cache-before.json')
    assert old_metadata() == load('old-metadata-before.json')
    # Redirect copied Git metadata to the new private repository. No old output paths.
    checkout = ROOT / 'swift/checkouts/Sparkle'
    repository = ROOT / 'swift/repositories/Sparkle-09d89c53'
    command(['git', '-C', str(checkout), 'config', '--local', 'remote.origin.url', str(repository)])
    alternates = checkout / '.git/objects/info/alternates'
    assert alternates.read_text().strip() == str(OLD / 'swift/repositories/Sparkle-09d89c53/objects')
    alternates.chmod(0o600)  # Private copy only; retain the original's read-only mode.
    alternates.write_text(str(repository / 'objects') + '\n')
    pins = json.loads((REPO / 'apps/macos/Package.resolved').read_text())['pins']
    assert len(pins) == 1 and pins[0]['state'] == dict(revision=PIN, version='2.10.0')
    assert command(['git', '-C', str(checkout), 'rev-parse', 'HEAD']) == PIN
    assert command(['git', '-C', str(checkout), 'status', '--porcelain']) == ''
    old_state = json.loads((OLD / 'swift/workspace-state.json').read_text())
    artifact = old_state['object']['artifacts'][0]
    assert artifact['source']['checksum'] == '17e28312b8e18ab7cdbbe09a6fb28cc55a5479ec6c371dbc07cdecd2a14fd959'
    package = dict(identity='sparkle', kind='remoteSourceControl',
                   location='https://github.com/sparkle-project/Sparkle', name='Sparkle')
    state = dict(version=7, object=dict(artifacts=[dict(kind={'xcframework': {}}, packageRef=package,
        path=str(ROOT / 'swift/artifacts/sparkle/Sparkle/Sparkle.xcframework'),
        source=artifact['source'], targetName='Sparkle')], dependencies=[dict(basedOn=None,
        packageRef=package, state=dict(checkoutState=dict(revision=PIN, version='2.10.0'),
        name='sourceControlCheckout'), subpath='Sparkle')], prebuilts=[]))
    (ROOT / 'swift/workspace-state.json').write_text(json.dumps(state, indent=2) + '\n')
    save('generated-state.json', state)
    save('seed-before-build.json', {name: inventory(ROOT / 'swift' / name) for name in SEEDS})
    # Reject retained build databases, output trees and external write destinations.
    assert sorted(p.name for p in (ROOT / 'swift').iterdir()) == ['artifacts', 'checkouts', 'repositories', 'workspace-state.json']
    assert list((ROOT / 'cargo-target').iterdir()) == []
    assert sorted(p.name for p in (ROOT / 'cargo-home').iterdir()) == ['registry']
    assert str(OLD) not in json.dumps(state)
    for relative in SEEDS:
        for path in (ROOT / 'swift' / relative).rglob('*'):
            if path.is_symlink():
                assert path.resolve(strict=True).is_relative_to(ROOT / 'swift')
    assert str(OLD) not in (checkout / '.git/config').read_text()
    assert alternates.read_text().strip() == str(repository / 'objects')
    port_check('ports-preflight-first.json')
    profile = '''(version 1)
(allow default)
(deny file-write*)
(allow file-write* (subpath "%s") (literal "/dev/null")
 (regex #"^/private/tmp/oe-(package|release-smoke)-[A-Za-z0-9_]+(/|$)"))
(deny network*)
(allow network* (local ip "localhost:%d") (remote ip "localhost:%d")
 (local unix-socket (regex #"^/private/tmp/oe-release-smoke-[A-Za-z0-9_]+/control[.]sock$"))
 (remote unix-socket (regex #"^/private/tmp/oe-release-smoke-[A-Za-z0-9_]+/control[.]sock$")))
''' % (ROOT, PORTS[0], PORTS[0])
    (ROOT / 'package.sb').write_text(profile)
    env = environment()
    args = [PYTHON, str(REPO / 'tools/release/release.py'), 'package', '--port-range',
            '%d-%d' % PORTS, '--output', str(ROOT / 'output')]
    save('plan.json', dict(argv=args, environment=env, sandbox=str(ROOT / 'package.sb'),
                          controller_nice=os.getpriority(os.PRIO_PROCESS, 0)))
    sandbox_preflight()


def sandbox_preflight():
    assert not (ROOT / 'package-once.guard').exists()
    env = environment()
    # Parse policy and exercise only the setup process, without any build or package call.
    probe = subprocess.run(['/usr/bin/sandbox-exec', '-f', str(ROOT / 'package.sb'),
                            PYTHON, '-c', 'import os; print(os.getpriority(os.PRIO_PROCESS,0))'],
                           cwd=REPO, env=env, text=True, capture_output=True)
    name = 'sandbox-preflight-first.json' if not (ROOT / 'sandbox-preflight-first.json').exists() else 'sandbox-preflight-second.json'
    save(name, dict(exit=probe.returncode, stdout=probe.stdout, stderr=probe.stderr))
    assert probe.returncode == 0 and probe.stdout.strip() == '15'
    # Real local Unix-socket setup under the same policy; no product, compiler or package.
    probe_code = '''import socket,tempfile,os
with tempfile.TemporaryDirectory(prefix="oe-release-smoke-",dir="/tmp") as d:
 p=d+"/control.sock"
 with socket.socket(socket.AF_UNIX,socket.SOCK_STREAM) as s:
  s.bind(p);s.listen(1)
  with socket.socket(socket.AF_UNIX,socket.SOCK_STREAM) as c:
   c.connect(p)
   with s.accept()[0] as a:
    c.sendall(b"owned setup");assert a.recv(32)==b"owned setup"
 print("private Unix IPC setup passed:",d)
'''
    ipc = subprocess.run(['/usr/bin/sandbox-exec', '-f', str(ROOT / 'package.sb'), PYTHON, '-c', probe_code],
                          cwd=REPO, env=env, text=True, capture_output=True)
    save('sandbox-unix-setup.json', dict(exit=ipc.returncode, stdout=ipc.stdout, stderr=ipc.stderr))
    assert ipc.returncode == 0
    save('preflight-complete.json', dict(passed=True, package_started=False,
        fresh_cargo_target=True, fresh_swift_build_state=True, no_old_output_seeded=True,
        paths_validated=True, signing_intent_variables_unset=True))
    print('Preflight complete. Package has not started.', flush=True)


class BsdInfo(ctypes.Structure):
    _fields_ = [(name, ctypes.c_uint32) for name in
        ('flags', 'status', 'xstatus', 'pid', 'ppid', 'uid', 'gid', 'ruid', 'rgid', 'svuid', 'svgid', 'rfu')]
    _fields_ += [('comm', ctypes.c_char * 16), ('name', ctypes.c_char * 32)]
    _fields_ += [(name, ctypes.c_uint32) for name in ('nfiles', 'pgid', 'pjobc', 'tdev', 'tpgid')]
    _fields_ += [('nice', ctypes.c_int32), ('start_sec', ctypes.c_uint64), ('start_usec', ctypes.c_uint64)]


LIBPROC = ctypes.CDLL('/usr/lib/libproc.dylib', use_errno=True)
LIBPROC.proc_pidinfo.argtypes = [ctypes.c_int, ctypes.c_int, ctypes.c_uint64, ctypes.c_void_p, ctypes.c_int]


def identity(pid):
    data = BsdInfo()
    if LIBPROC.proc_pidinfo(pid, 3, 0, ctypes.byref(data), ctypes.sizeof(data)) != ctypes.sizeof(data):
        return None
    return dict(pid=data.pid, ppid=data.ppid, pgid=data.pgid, nice=data.nice,
                start_sec=data.start_sec, start_usec=data.start_usec,
                executable=data.name.decode(errors='replace') or data.comm.decode(errors='replace'))


def same(a, b):
    return b is not None and all(a[k] == b[k] for k in ('pid', 'start_sec', 'start_usec', 'pgid'))


def snapshot(root_pid, root_identity, known):
    rows = command(['/bin/ps', '-axo', 'pid=,ppid=,pgid=,ni=,comm='], timeout=5).splitlines()
    table = {}
    for line in rows:
        fields = line.strip().split(None, 4)
        if len(fields) == 5:
            pid, ppid, pgid, nice = map(int, fields[:4])
            table[pid] = dict(pid=pid, ppid=ppid, pgid=pgid, nice=nice, comm=fields[4])
    owned = {root_pid} if same(root_identity, identity(root_pid)) else set()
    # Keep previously attributed identities, including an observed orphan.
    owned |= {pid for pid, old in known.items() if pid in table and same(old, identity(pid))}
    while True:
        added = owned | {pid for pid, row in table.items() if row['ppid'] in owned}
        if added == owned:
            break
        owned = added
    records = []
    for pid in sorted(owned):
        row = table.get(pid)
        current = identity(pid)
        if row is None or current is None:
            continue
        # Every new descendant must still belong to this command's process group.
        if current['pgid'] != root_identity['pgid']:
            continue
        records.append(dict(**current, comm=row['comm'], ps_parent=row['ppid'],
                            ps_group=row['pgid'], ps_nice=row['nice']))
        known[pid] = current
    return records


def cleanup(known):
    started = time.monotonic()
    actions = []
    for sig, budget in ((signal.SIGTERM, 10), (signal.SIGKILL, 10)):
        active = [old for pid, old in known.items() if same(old, identity(pid))]
        for old in reversed(active):
            if same(old, identity(old['pid'])):
                try:
                    os.kill(old['pid'], sig)
                    actions.append(dict(identity=old, signal=int(sig)))
                except ProcessLookupError:
                    pass
        deadline = time.monotonic() + budget
        while time.monotonic() < deadline and any(same(old, identity(pid)) for pid, old in known.items()):
            time.sleep(.1)
    remaining = [old for pid, old in known.items() if same(old, identity(pid))]
    save('owned-cleanup.json', dict(actions=actions, remaining=remaining,
                                   elapsed_seconds=time.monotonic() - started, budget_seconds=60))
    return remaining


def run():
    assert load('preflight-complete.json')['passed']
    assert command(['git', 'rev-parse', 'HEAD']) == BASE
    assert source_inventory() == load('source-before.json')
    assert prior_integrity() == load('prior-before.json')
    plan = load('plan.json')
    assert plan['environment'] == environment()
    controller_identity = identity(os.getpid())
    assert controller_identity and controller_identity['nice'] == 15
    save('execution-bindings.json', dict(controller_sha256=sha(ROOT / 'controller.py'),
        observer_sha256=sha(ROOT / 'observe/sitecustomize.py'),
        sandbox_sha256=sha(ROOT / 'package.sb'), plan_sha256=sha(ROOT / 'plan.json'),
        preflight_sha256=sha(ROOT / 'preflight-complete.json'), controller_identity=controller_identity))
    with (ROOT / 'package-once.guard').open('x') as guard:
        guard.write('ONE actual release.py package invocation; never reset this guard.\n')
    start = time.monotonic()
    known, samples, timed_out = {}, [], False
    with (ROOT / 'package.stdout').open('xb') as out, (ROOT / 'package.stderr').open('xb') as err:
        process = subprocess.Popen(['/usr/bin/sandbox-exec', '-f', str(ROOT / 'package.sb'), *plan['argv']],
            cwd=REPO, env=plan['environment'], stdin=subprocess.DEVNULL,
            stdout=out, stderr=err, start_new_session=True)
        first = identity(process.pid)
        assert first and first['nice'] == 15 and first['pgid'] == process.pid
        save('package-start.json', dict(identity=first, monotonic=start, command=plan['argv']))
        try:
            while process.poll() is None:
                records = snapshot(process.pid, first, known)
                samples.append(dict(elapsed_seconds=time.monotonic() - start, owned=records))
                if any(row['nice'] != 15 or row['ps_nice'] != 15 for row in records):
                    raise ValueError('owned process priority differs from absolute NI15')
                if time.monotonic() - start >= 900:
                    timed_out = True
                    cleanup(known)
                    break
                time.sleep(.01)
        except BaseException:
            (ROOT / 'first-error-runtime-controller.txt').write_text(traceback.format_exc())
            cleanup(known)
        code = process.wait(timeout=10)
    save('process-samples.json', samples)
    save('package-result.json', dict(exit=code, elapsed_seconds=time.monotonic() - start,
        timeout=timed_out, root_identity=first, stdout_sha256=sha(ROOT / 'package.stdout'),
        stderr_sha256=sha(ROOT / 'package.stderr')))
    if not (ROOT / 'owned-cleanup.json').exists():
        cleanup(known)
    port_check('ports-postflight-first.json')
    print(json.dumps(load('package-result.json')), flush=True)
    if code:
        print((ROOT / 'package.stderr').read_text()[-3000:], flush=True)
    audit()


def audit():
    if not (ROOT / 'old-cache-after.json').exists():
        save('old-cache-after.json', old_inventory())
        save('old-metadata-after.json', old_metadata())
        save('source-after.json', source_inventory())
        save('prior-after.json', prior_integrity())
        save('seed-after-build.json', {name: inventory(ROOT / 'swift' / name) for name in SEEDS})
    before, after = load('seed-before-build.json'), load('seed-after-build.json')
    differences = {}
    for name in SEEDS:
        differences[name] = [dict(path=p, before=before[name].get(p), after=after[name].get(p))
            for p in sorted(set(before[name]) | set(after[name])) if before[name].get(p) != after[name].get(p)]
    save('private-seed-differences.json', differences)
    allowed = {'checkouts/Sparkle': {'.git/index'}}
    private_dependency_unchanged = all(all(d['path'] in allowed.get(name, set()) for d in changes)
                                       for name, changes in differences.items())
    traces = [json.loads(line) for p in sorted(ROOT.glob('trace-*.jsonl')) for line in p.read_text().splitlines()]
    smoke = [row for row in traces if row['kind'] == 'smoke-return']
    commands = [row for row in traces if row['kind'] == 'real-command-return']
    smoke_commands = [row for row in commands if 'smoke' in row]
    cleanups = [row for row in traces if row['kind'] == 'before-temp-cleanup']
    save('full-smoke.json', dict(smoke=smoke, commands=smoke_commands,
        daemon_starts=[r for r in traces if r['kind'] == 'daemon-started'],
        before_cleanup=cleanups))
    artifacts = [dict(path=str(p), bytes=p.stat().st_size, sha256=sha(p))
                 for p in sorted((ROOT / 'output').glob('*')) if p.is_file()]
    save('artifacts.json', artifacts)
    binding = {}
    archive = next(iter((ROOT / 'output').glob('*.zip')), None)
    if archive:
        with zipfile.ZipFile(archive) as zipped:
            assert zipped.testzip() is None
            manifest = zipped.read('BUILD.json')
            assert manifest == archive.with_suffix('.json').read_bytes()
            info = plistlib.loads(zipped.read('Openergy.app/Contents/Info.plist'))
            original = plistlib.loads((REPO / 'apps/macos/Support/Info.plist').read_bytes())
            assert all(info[k] == original[k] for k in ('SUFeedURL', 'SUPublicEDKey', 'LSMinimumSystemVersion'))
            assert info['SUEnableAutomaticChecks'] is True
            assert json.loads(manifest)['commit'] == BASE
            assert json.loads(manifest)['signing'] == 'adhoc'
            assert json.loads(manifest)['target'] == 'aarch64-apple-darwin'
            assert not json.loads(manifest)['notarized']
            assert archive.with_name(archive.name + '.sha256').read_text() == sha(archive) + '  ' + archive.name + '\n'
            binding = dict(entries=len(zipped.infolist()), manifest=json.loads(manifest),
                embedded_manifest_equals_sidecar=True, checksum_sidecar_matches=True,
                production_feed_key_defaults_preserved=True, automatic_checks=True)
    save('artifact-bindings.json', binding)
    samples = load('process-samples.json')
    observations = [row for sample in samples for row in sample['owned']]
    gaps = [b['elapsed_seconds'] - a['elapsed_seconds'] for a, b in zip(samples, samples[1:])]
    nice_values = sorted({r['nice'] for r in observations} | {r['ps_nice'] for r in observations})
    owned_pids = sorted({r['pid'] for r in observations})
    post = dict(original_cache_bytes_links_unchanged=load('old-cache-before.json') == load('old-cache-after.json'),
        original_selected_metadata_unchanged=load('old-metadata-before.json') == load('old-metadata-after.json'),
        source_bytes_unchanged=load('source-before.json') == load('source-after.json'),
        prior_gzip_private_originals_unchanged=load('prior-before.json') == load('prior-after.json'),
        private_dependencies_unchanged_except_allowed_git_index=private_dependency_unchanged,
        copied_git_index_is_not_a_dependency_integrity_hash=True,
        checkout_pinned_revision=command(['git', '-C', str(ROOT / 'swift/checkouts/Sparkle'), 'rev-parse', 'HEAD']),
        checkout_source_status=command(['git', '-C', str(ROOT / 'swift/checkouts/Sparkle'), 'status', '--porcelain']),
        observed_nice_values=nice_values, observed_pids=owned_pids, samples=len(samples),
        sample_first_seconds=samples[0]['elapsed_seconds'] if samples else None,
        sample_last_seconds=samples[-1]['elapsed_seconds'] if samples else None,
        max_sample_gap_seconds=max(gaps, default=None),
        sampling_limit='Short-lived processes and all times outside/between observations are not proven.',
        observed_owned_remaining=load('owned-cleanup.json')['remaining'],
        temporary_paths_absent={r['directory']: not Path(r['directory']).exists() for r in cleanups},
        smoke_completions=len(smoke), smoke_daemon_exits=[r['daemon_exit'] for r in smoke],
        package=load('package-result.json'), artifact_bindings=binding)
    save('postflight.json', post)
    assert all(post[k] for k in ('original_cache_bytes_links_unchanged', 'original_selected_metadata_unchanged',
        'source_bytes_unchanged', 'prior_gzip_private_originals_unchanged',
        'private_dependencies_unchanged_except_allowed_git_index'))
    assert post['checkout_pinned_revision'] == PIN and not post['checkout_source_status']
    assert nice_values == [15] and not post['observed_owned_remaining']
    assert post['package']['exit'] == 0 and not post['package']['timeout']
    assert len(smoke) == 2 and post['smoke_daemon_exits'] == [0, 0]
    assert all(post['temporary_paths_absent'].values())
    print('Postflight preservation and real package flow passed.', flush=True)


if __name__ == '__main__':
    os.umask(0o077)
    os.setpriority(os.PRIO_PROCESS, 0, 15)
    assert os.getpriority(os.PRIO_PROCESS, 0) == 15
    try:
        {'prepare': prepare, 'finish-prepare': finish_prepare, 'sandbox-preflight': sandbox_preflight, 'run': run, 'audit': audit}[sys.argv[1]]()
    except BaseException:
        name = 'first-error-' + sys.argv[1] + '.txt'
        if not (ROOT / name).exists():
            (ROOT / name).write_text(traceback.format_exc())
        raise
