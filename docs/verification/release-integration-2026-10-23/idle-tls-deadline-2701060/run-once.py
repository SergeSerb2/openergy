import datetime
import hashlib
import json
import os
import pathlib
import shutil
import signal
import subprocess
import time

root = pathlib.Path(__file__).resolve().parent
raw = root / 'raw'
binary = root / 'target/release/examples/dns_queue_cancel_c4806a3d'
runtime = root / 'runtime'
# An existing record prevents a second case, including after failure or timeout.
with (raw / 'runtime-once.guard').open('x') as guard:
    guard.write(datetime.datetime.now().astimezone().isoformat() + '\n')
assert not runtime.exists()
assert all(json.loads((raw / (name + '.json')).read_text())['exit_code'] == 0
           for name in ['clippy', 'release', 'format-check'])
os.umask(0o077)
env = os.environ.copy()
env.update(CARGO_BUILD_JOBS='2', MACOSX_DEPLOYMENT_TARGET='15.0', CARGO_NET_OFFLINE='true')
record = {
    'command': ['/usr/bin/nice', '-n', '15', str(binary), str(runtime), '--idle-tls-deadline-only'],
    'start_local': datetime.datetime.now().astimezone().isoformat(),
    'timeout_seconds': 15,
    'runtime_attempt': 1,
    'runtime_retries_allowed': 0,
    'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
    'binary_size': binary.stat().st_size,
    'environment': {key: env[key] for key in ['CARGO_BUILD_JOBS', 'MACOSX_DEPLOYMENT_TARGET', 'CARGO_NET_OFFLINE']},
}
for name, command in [
    ('rustc', ['/Users/serge/.cargo/bin/rustc', '+1.99.1', '-Vv']),
    ('cargo', ['/Users/serge/.cargo/bin/cargo', '+1.99.1', '-V']),
    ('host', ['/usr/bin/sw_vers']),
    ('binary-build', ['/usr/bin/xcrun', 'vtool', '-show-build', str(binary)]),
]:
    result = subprocess.run(command, capture_output=True, text=True, check=False)
    (raw / (name + '.stdout')).write_text(result.stdout)
    (raw / (name + '.stderr')).write_text(result.stderr)
    record[name] = {'command': command, 'exit_code': result.returncode}
start = time.monotonic_ns()
with (raw / 'runtime.stdout').open('xb') as out, (raw / 'runtime.stderr').open('xb') as err:
    process = subprocess.Popen(record['command'], env=env, stdout=out, stderr=err, start_new_session=True)
    record['owned_pid'] = process.pid
    ps = subprocess.run(['/bin/ps', '-o', 'pid=,ppid=,ni=,stat=,command=', '-p', str(process.pid)], capture_output=True, text=True)
    (raw / 'runtime-process.stdout').write_text(ps.stdout)
    try:
        record['exit_code'] = process.wait(timeout=15)
        record['watchdog_fired'] = False
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        record['exit_code'] = process.wait(timeout=2)
        record['watchdog_fired'] = True
record['elapsed_ns'] = time.monotonic_ns() - start
record['end_local'] = datetime.datetime.now().astimezone().isoformat()
record['process_exited'] = process.poll() is not None
inventory = []
if runtime.exists():
    for path in sorted(runtime.rglob('*')):
        info = path.lstat()
        assert not path.is_symlink()
        inventory.append({'path': str(path.relative_to(runtime)), 'mode': oct(info.st_mode & 0o777), 'bytes': info.st_size if path.is_file() else None})
    record['runtime_root_mode'] = oct(runtime.stat().st_mode & 0o777)
    shutil.rmtree(runtime)
record['private_config_inventory_before_removal'] = inventory
record['fresh_runtime_and_identities_removed'] = not runtime.exists()
(raw / 'runtime.json').write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps(record, indent=2))
print('--- SELECTED RAW OUTPUT ---')
for line in (raw / 'runtime.stdout').read_text().splitlines():
    if line.startswith(('SCOPE', 'DEADLINE ', 'ACCEPT ', 'CLIENT_HELLO', 'PRECONDITION', 'IDLE_', 'FIRST_ELIGIBLE', 'FAILED_CLIENT', 'RECOVERY ', 'WORKFLOW', 'CLEANUP', 'TRACE ', 'TRANSCRIPT ', 'RESULT')):
        print(line)
print((raw / 'runtime.stderr').read_text())
