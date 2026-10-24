#!/usr/bin/env python3
"""One-shot real CLI evidence controller. No target execution and no retries."""
from pathlib import Path
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from datetime import datetime
import gzip
import hashlib
import json
import os
import plistlib
import re
import shutil
import signal
import stat
import subprocess
import sys
import threading
import time
from urllib.parse import urlsplit

os.umask(0o077)
FIX = Path('/private/tmp/oe-stopped-update-llil939f')
OUT = Path('/private/tmp/oe-stopped-runtime-20261023-wXymLm')
DOC = Path(__file__).resolve().parent
PREP = DOC.parent / 'sparkle-stopped-fixtures-2026-10-23'
BUILD = Path('/private/tmp/oe-sparkle-cli-build-njko0i6t')
ID = 'dev.openergy.updater-fixture.5b4a5cb7d61d444eaf4a3d673e1df4a5'
V2 = '2846f3adafe9b1f39092bb0e98e672dd66c9ff7f89715c110c282723d89d3cef'
JOBS = [ID + '-sparkle-updater', ID + '-sparkle-progress']
ENV = {'PATH': '/usr/bin:/bin:/usr/sbin:/sbin', 'HOME': '/Users/serge',
       'USER': 'serge', 'LOGNAME': 'serge',
       'CFFIXED_USER_HOME': str(FIX / 'config'), 'TMPDIR': str(FIX / 'config') + '/',
       'MACOSX_DEPLOYMENT_TARGET': '15.0', 'CARGO_BUILD_JOBS': '2'}
CACHES = [Path('/Users/serge/Library/Caches') / ID,
          Path('/Users/serge/Library/Caches') / (ID + '.driver'),
          FIX / 'config/Library/Caches' / ID,
          FIX / 'config/Library/Caches' / (ID + '.driver'),
          Path('/var/folders/s6/_07cr4w13vsg_myjkv9ldpqc0000gn/C') / ID,
          Path('/var/folders/s6/_07cr4w13vsg_myjkv9ldpqc0000gn/C') / (ID + '.driver')]


def now():
    return datetime.now().astimezone().isoformat()


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def save(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def append(path, value):
    with path.open('a') as f:
        f.write(json.dumps(value) + '\n')


def tool(label, args, directory, timeout=25):
    started = now()
    t = time.monotonic()
    with (directory / (label + '.stdout')).open('xb') as out, (directory / (label + '.stderr')).open('xb') as err:
        p = subprocess.Popen(['/usr/bin/nice', '-n', '15', *map(str, args)],
                             env=ENV, stdout=out, stderr=err, start_new_session=True)
        expired = False
        try:
            p.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            expired = True
            os.killpg(p.pid, signal.SIGTERM)
            try:
                p.wait(timeout=3)
            except subprocess.TimeoutExpired:
                os.killpg(p.pid, signal.SIGKILL)
                p.wait(timeout=3)
    result = {'arguments': list(map(str, args)), 'started_at': started, 'ended_at': now(),
              'seconds': time.monotonic() - t, 'raw_exit': p.returncode, 'timeout': expired}
    save(directory / (label + '.json'), result)
    return result


def tree(root):
    result = {}
    for p in sorted(root.rglob('*')):
        mode = stat.S_IMODE(p.lstat().st_mode)
        rel = p.relative_to(root).as_posix()
        if p.is_symlink():
            assert p.resolve().is_relative_to(root.resolve()), str(p)
            result[rel] = ['link', os.readlink(p)]
        elif p.is_dir():
            result[rel] = ['dir', mode]
        elif p.is_file():
            result[rel] = ['file', mode, sha(p)]
        else:
            raise RuntimeError('Non-file in bundle: ' + str(p))
    return result


def inventory(root, directories=False, prefix=''):
    result = {}
    paths = [root, *sorted(root.rglob('*'))] if directories else sorted(root.rglob('*'))
    for p in paths:
        key = (prefix + '/' if prefix else '') + p.relative_to(root).as_posix()
        key = key.removesuffix('/.')
        if p.is_symlink():
            value = {'symlink': os.readlink(p)}
            if directories:
                value = {'mode': oct(stat.S_IMODE(p.lstat().st_mode)), **value}
        elif p.is_file():
            value = {'sha256': sha(p), 'bytes': p.stat().st_size,
                     'mode': oct(stat.S_IMODE(p.stat().st_mode))}
        elif p.is_dir() and directories:
            value = {'directory': True, 'mode': oct(stat.S_IMODE(p.stat().st_mode))}
        else:
            continue
        result[key] = value
    return result


def processes():
    p = subprocess.run(['/bin/ps', '-axo', 'pid=,ppid=,pgid=,ni=,comm='],
                       capture_output=True, text=True, timeout=5, env=ENV)
    assert p.returncode == 0
    found = []
    for line in p.stdout.splitlines():
        fields = line.strip().split(None, 4)
        if len(fields) != 5:
            continue
        matched = str(FIX) in fields[4] or ID in fields[4]
        if not matched and fields[4] in ['/usr/bin/gktool', '/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister']:
            q = subprocess.run(['/bin/ps', '-p', fields[0], '-o', 'args='],
                               capture_output=True, text=True, timeout=3, env=ENV)
            matched = str(FIX) in q.stdout or ID in q.stdout
        if matched:
            pid = int(fields[0])
            q = subprocess.run(['/bin/ps', '-p', str(pid), '-o', 'args='],
                               capture_output=True, text=True, timeout=3, env=ENV)
            found.append({'pid': pid, 'ppid': int(fields[1]), 'pgid': int(fields[2]),
                          'nice': int(fields[3]), 'executable': fields[4],
                          'arguments': q.stdout.strip()})
    return found


def target_process(p):
    return '/Contents/MacOS/Openergy' in p['executable'] or '/Contents/Helpers/openergy' in p['executable']


def jobs():
    p = subprocess.run(['/bin/launchctl', 'list'], capture_output=True, text=True, timeout=5, env=ENV)
    assert p.returncode == 0
    return [s for s in p.stdout.splitlines() if ID in s]


def cache_snapshot():
    result = []
    for root in CACHES:
        entry = {'path': str(root), 'exists': root.exists(), 'symlink': root.is_symlink()}
        if root.exists() and not root.is_symlink():
            entry['entries'] = inventory(root, directories=True)
        result.append(entry)
    return result


def ls_receipt(label, directory):
    # Only the fixture-matching lines leave this read-only query. Never save the full database.
    command = ['/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister', '-dump']
    t = time.monotonic()
    p = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=ENV, start_new_session=True)
    try:
        stdout, stderr = p.communicate(timeout=25)
        expired = False
    except subprocess.TimeoutExpired:
        os.killpg(p.pid, signal.SIGTERM)
        stdout, stderr = p.communicate(timeout=3)
        expired = True
    matching = [line for line in stdout.decode(errors='replace').splitlines() if ID in line or str(FIX) in line]
    save(directory / (label + '.json'), {'arguments': command, 'raw_exit': p.returncode,
         'timeout': expired, 'seconds': time.monotonic() - t, 'matching_lines': matching,
         'stderr': stderr.decode(errors='replace'), 'scope': 'fixture matching lines only; no unregister or assessment requested'})


def metadata(app):
    p = app / 'Contents/Info.plist'
    info = plistlib.loads(p.read_bytes())
    s = app.stat()
    return {'path': str(app), 'id': info['CFBundleIdentifier'], 'build': info['CFBundleVersion'],
            'executable_sha256': sha(app / 'Contents/MacOS' / info['CFBundleExecutable']),
            'uid': s.st_uid, 'gid': s.st_gid, 'mode': oct(stat.S_IMODE(s.st_mode)),
            'inode': s.st_ino, 'device': s.st_dev, 'mtime_ns': s.st_mtime_ns, 'atime_ns': s.st_atime_ns,
            'xattrs': {n: hashlib.sha256(os.getxattr(app, n)).hexdigest() for n in os.listxattr(app)}}


def preflight(directory):
    assert sha(PREP / 'operation-v2-DO-NOT-RUN.json') == V2
    assert sha(FIX / 'operation-v2-DO-NOT-RUN.json') == V2
    records = json.loads((PREP / 'results.json').read_text())['records']
    checked = []
    for record in records:
        public = PREP / record['file']
        assert sha(public) == record['sha256'], str(public)
        original = Path(record['original'])
        expected = record.get('original_sha256', record['sha256'])
        assert sha(original) == expected, str(original)
        checked.append({'path': str(original), 'sha256': expected})
    prep = json.loads((FIX / 'preparation.json').read_text())
    driver = json.loads((FIX / 'driver-preparation.json').read_text())
    for name, value in prep['files'].items():
        path = FIX / name
        assert sha(path) == value['sha256'] and path.stat().st_size == value['bytes'], str(path)
        checked.append({'path': str(path), **value})
    assert sha(prep['signer']['path']) == prep['signer']['sha256']
    assert sha(Path(driver['source_cli']) / 'Contents/MacOS/sparkle') == driver['source_cli_binary_sha256']
    assert sha(FIX / 'driver/sparkle.app/Contents/MacOS/sparkle') == driver['prepared_cli_binary_sha256']
    assert sha(FIX / 'driver/sparkle.app/Contents/Frameworks/Sparkle.framework/Versions/B/Sparkle') == driver['framework_executable_sha256']
    pinned = FIX / 'prepared-driver-tree.json'
    assert inventory(FIX / 'driver/sparkle.app') == json.loads(pinned.read_text()), 'driver tree mismatch'
    expected_build = json.loads(gzip.decompress((DOC.parent / 'sparkle-cli-build-preparation-2026-10-23/raw/sparkle-app-files.json.gz').read_bytes()))
    assert inventory(Path(driver['source_cli']), True, 'sparkle.app') == expected_build, 'immutable original CLI tree mismatch'
    expected = json.loads((FIX / 'bundle-trees.json').read_text())
    for name, key in [('old', 'old'), ('candidate', 'candidate'), ('roundtrip', 'candidate')]:
        actual = tree(FIX / name / 'Openergy.app')
        assert actual == expected[key], name + ' tree mismatch'
        save(directory / (name + '-tree.json'), actual)
    assert not (FIX / 'ephemeral-seed').exists() and not (FIX / 'public-verifier').exists()
    for path in [FIX, FIX / 'old', FIX / 'candidate', FIX / 'config', FIX / 'driver']:
        assert path.is_dir() and not path.is_symlink() and path.stat().st_uid == os.getuid(), str(path)
    active = processes()
    assert not active, active
    active_jobs = jobs()
    assert not active_jobs, active_jobs
    before_caches = cache_snapshot()
    assert not any(c['exists'] or c['symlink'] for c in before_caches), 'existing fixture cache'
    for label, path in [('old', FIX / 'old/Openergy.app'), ('candidate', FIX / 'candidate/Openergy.app'),
                        ('driver', FIX / 'driver/sparkle.app')]:
        r = tool('preflight-seal-' + label, ['/usr/bin/codesign', '--verify', '--all-architectures', '--deep', '--strict', path], directory)
        assert r['raw_exit'] == 0 and not r['timeout'], label + ' seal'
        tool('identity-' + label, ['/usr/bin/codesign', '-dvvv', path], directory)
    assert sha(BUILD / 'source/Autoupdate/SUPlainInstaller.m') == 'd5191e8b7684df756383c2122dcfcaaa011d5bae237b2ddbf9fa05986c8f5b55'
    save(directory / 'preflight.json', {'at': now(), 'checked_preparation_files': checked,
         'driver_full_tree_matches': True, 'original_cli_full_tree_matches': True,
         'old_candidate_roundtrip_full_trees_match': True, 'processes': active, 'jobs': active_jobs,
         'caches': before_caches, 'old': metadata(FIX / 'old/Openergy.app'),
         'candidate': metadata(FIX / 'candidate/Openergy.app'), 'explicit_environment': ENV,
         'approval': {'received': True, 'at': '2026-10-23T17:16:55-07:00',
                      'answer': 'Allow the reviewed fixture effects (Recommended)', 'proposal_sha256': V2},
         'all_preflight_checks_passed': True})
    return expected


class Server(ThreadingHTTPServer):
    allow_reuse_address = False
    daemon_threads = True


def run(case):
    directory = OUT / case
    directory.mkdir(mode=0o700)  # A completed/failed case can never be run again here.
    started = now()
    result = {'case': case, 'controller_started': started, 'cli_executed': False}
    server = None
    log = None
    cli = None
    observation_stop = threading.Event()
    breach = threading.Event()
    observer_errors = []
    requests = []
    sequence_lock = threading.Lock()
    expected = None
    try:
        expected = preflight(directory)
        ls_receipt('launchservices-before', directory)

        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *_args):
                pass

            def do_GET(self):
                path = urlsplit(self.path).path
                receipt = {'at': now(), 'client': list(self.client_address), 'method': 'GET',
                           'path': self.path, 'range': self.headers.get('Range'), 'sent_bytes': 0}
                allowed = {'/signature-negative.xml', '/valid.xml', '/openergy-fixture.zip'}
                try:
                    assert self.client_address[0] == '127.0.0.1'
                    if path not in allowed:
                        self.send_error(404)
                        receipt['status'] = 404
                        return
                    data = (FIX / 'public' / path[1:]).read_bytes()
                    self.send_response(200)
                    self.send_header('Content-Length', str(len(data)))
                    self.send_header('Content-Type', 'application/octet-stream' if path.endswith('.zip') else 'application/xml')
                    self.end_headers()
                    receipt.update(status=200, expected_bytes=len(data), sha256=hashlib.sha256(data).hexdigest())
                    for offset in range(0, len(data), 65536):
                        chunk = data[offset:offset + 65536]
                        self.wfile.write(chunk)
                        receipt['sent_bytes'] += len(chunk)
                    self.wfile.flush()
                    receipt['complete'] = receipt['sent_bytes'] == len(data)
                except Exception as e:
                    receipt['error'] = repr(e)
                finally:
                    receipt['finished_at'] = now()
                    with sequence_lock:
                        requests.append(receipt)
                        append(directory / 'server.jsonl', receipt)

        server = Server(('127.0.0.1', 25600), Handler)
        threading.Thread(target=server.serve_forever, daemon=True).start()
        save(directory / 'server-bound.json', {'at': now(), 'address': list(server.server_address),
                                             'reuse_address': False, 'pid': os.getpid()})
        predicate = 'eventMessage CONTAINS "' + ID + '" OR eventMessage CONTAINS "' + str(FIX) + '"'
        log_out = (directory / 'unified-fixture.ndjson').open('xb')
        log_err = (directory / 'unified-fixture.stderr').open('xb')
        log = subprocess.Popen(['/usr/bin/nice', '-n', '15', '/usr/bin/log', 'stream', '--style', 'ndjson',
                                '--level', 'info', '--predicate', predicate], env=ENV,
                               stdout=log_out, stderr=log_err, start_new_session=True)
        save(directory / 'unified-fixture-start.json', {'pid': log.pid, 'predicate': predicate,
                                                       'started_at': now(), 'scope': 'fixture paths or ID only'})

        def observe():
            last_jobs = None
            last_paths = None
            while not observation_stop.is_set():
                try:
                    found = processes()
                    append(directory / 'process-samples.jsonl', {'at': now(), 'processes': found})
                    for process in found:
                        if target_process(process):
                            breach.set()
                            append(directory / 'target-breach.jsonl', {'at': now(), 'process': process})
                            # Recheck executable ownership immediately before stopping this exact PID.
                            if any(p['pid'] == process['pid'] and p['executable'] == process['executable'] for p in processes()):
                                os.kill(process['pid'], signal.SIGSTOP)
                                os.kill(process['pid'], signal.SIGTERM)
                                os.kill(process['pid'], signal.SIGCONT)
                    if time.monotonic() - getattr(observe, 'last_slow', 0) >= 0.8:
                        current_jobs = jobs()
                        paths = [{'path': str(p), 'exists': p.exists(), 'symlink': p.is_symlink()} for p in CACHES]
                        if current_jobs != last_jobs or paths != last_paths:
                            append(directory / 'job-cache-changes.jsonl', {'at': now(), 'jobs': current_jobs, 'caches': paths})
                            last_jobs, last_paths = current_jobs, paths
                        observe.last_slow = time.monotonic()
                except Exception as e:
                    observer_errors.append(repr(e))
                    breach.set()  # Monitoring failure stops acceptance and the command.
                    return
                observation_stop.wait(0.10)

        observer = threading.Thread(target=observe, daemon=True)
        observer.start()
        assert not processes() and not jobs(), 'fixture started after preflight'
        command = json.loads((PREP / 'operation-v2-DO-NOT-RUN.json').read_text())['commands'][case]
        with (directory / 'cli.stdout').open('xb') as stdout, (directory / 'cli.stderr').open('xb') as stderr:
            t = time.monotonic()
            cli = subprocess.Popen(['/usr/bin/nice', '-n', '15', *command], env=ENV,
                                   stdout=stdout, stderr=stderr, start_new_session=True)
            result.update(cli_executed=True, arguments=command, wrapper=['/usr/bin/nice', '-n', '15'],
                          cli_pid=cli.pid, process_group=cli.pid, cli_started_at=now(), timeout_seconds=120)
            save(directory / 'cli-started.json', result)
            reason = None
            while cli.poll() is None:
                if breach.is_set() or time.monotonic() - t >= 120:
                    reason = 'monitoring failure or target breach' if breach.is_set() else '120 second timeout'
                    os.killpg(cli.pid, signal.SIGTERM)
                    try:
                        cli.wait(timeout=3)
                    except subprocess.TimeoutExpired:
                        os.killpg(cli.pid, signal.SIGKILL)
                        cli.wait(timeout=3)
                    break
                time.sleep(0.10)
            result.update(raw_exit=cli.returncode, cli_ended_at=now(), seconds=time.monotonic() - t,
                          termination_reason=reason)
        # Continue observing detached fixture helpers through a bounded settle window.
        quiet_since = time.monotonic()
        deadline = time.monotonic() + 12
        while time.monotonic() < deadline:
            if processes():
                quiet_since = time.monotonic()
            elif time.monotonic() - quiet_since >= 2:
                break
            time.sleep(0.20)
        observation_stop.set()
        observer.join(timeout=5)
        result.update(target_breach=breach.is_set(), observer_errors=observer_errors,
                      process_sampling='ps nominal 100ms plus command time; short-lived processes can be missed')
        save(directory / 'cache-after.json', cache_snapshot())
        save(directory / 'jobs-after.json', jobs())
        save(directory / 'processes-after.json', processes())
        old_tree = tree(FIX / 'old/Openergy.app')
        save(directory / 'old-tree-after.json', old_tree)
        result.update(old_tree_unchanged=old_tree == expected['old'],
                      oldpath_exact_candidate_tree=old_tree == expected['candidate'],
                      old_after=metadata(FIX / 'old/Openergy.app'))
        result['archive_complete_gets'] = [r for r in requests if urlsplit(r['path']).path == '/openergy-fixture.zip' and r.get('complete')]
        ls_receipt('launchservices-after', directory)
        if case == 'positive' and old_tree == expected['candidate']:
            nested = ['Contents/Frameworks/Sparkle.framework/Versions/B/Autoupdate',
                      'Contents/Frameworks/Sparkle.framework/Versions/B/Updater.app',
                      'Contents/Frameworks/Sparkle.framework/Versions/B/XPCServices/Downloader.xpc',
                      'Contents/Frameworks/Sparkle.framework/Versions/B/XPCServices/Installer.xpc',
                      'Contents/Frameworks/Sparkle.framework', 'Contents/Helpers/openergy', '.']
            seals = []
            for i, relative in enumerate(nested):
                r = tool('replacement-seal-' + str(i), ['/usr/bin/codesign', '--verify', '--all-architectures', '--strict', FIX / 'old/Openergy.app' / relative], directory)
                seals.append(r)
            seals.append(tool('replacement-deep-seal', ['/usr/bin/codesign', '--verify', '--all-architectures', '--deep', '--strict', FIX / 'old/Openergy.app'], directory))
            result['replacement_nested_seals_pass'] = all(r['raw_exit'] == 0 and not r['timeout'] for r in seals)
    except Exception as e:
        result['controller_error'] = repr(e)
        import traceback
        (directory / 'controller-error.txt').write_text(traceback.format_exc())
    finally:
        observation_stop.set()
        if cli is not None:
            try:
                os.killpg(cli.pid, signal.SIGTERM)
                result['owned_cli_group_final_cleanup'] = 'SIGTERM sent'
            except ProcessLookupError:
                result['owned_cli_group_final_cleanup'] = 'already absent'
        if log is not None:
            os.killpg(log.pid, signal.SIGTERM)
            try:
                log.wait(timeout=3)
            except subprocess.TimeoutExpired:
                os.killpg(log.pid, signal.SIGKILL)
                log.wait(timeout=3)
            log_out.close()
            log_err.close()
            result['log_raw_exit'] = log.returncode
        if server is not None:
            server.shutdown()
            server.server_close()
            result['server_closed_at'] = now()
        result['controller_ended'] = now()
        save(directory / 'result.json', result)
    print(json.dumps(result, indent=2), flush=True)


if __name__ == '__main__':
    assert len(sys.argv) == 2 and sys.argv[1] in ['negative', 'positive']
    run(sys.argv[1])
