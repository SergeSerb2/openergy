#!/usr/bin/env python3
"""Exercise the complete metadata-reader CLI with real local child commands.

This does not contact Windows or reopen a historical attempt. Each case runs
once in a fresh private directory. A failure stops the remaining cases.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import signal
import stat
import subprocess
import sys
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--reader', type=Path, required=True)
    parser.add_argument('--reader-sha256', required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    os.setpriority(os.PRIO_PROCESS, 0, 15)
    if os.getpriority(os.PRIO_PROCESS, 0) != 15:
        raise RuntimeError('nice 15 required')
    reader = args.reader.resolve(strict=True)
    actual = hashlib.sha256(reader.read_bytes()).hexdigest()
    if actual != args.reader_sha256:
        raise ValueError('reader hash differs from reviewed input')
    parent = args.output.parent.lstat()
    if (not args.output.is_absolute() or args.output.exists() or
            not stat.S_ISDIR(parent.st_mode) or stat.S_IMODE(parent.st_mode) != 0o700 or
            parent.st_uid != os.getuid()):
        raise ValueError('fresh output under owned private 0700 parent required')
    args.output.mkdir(mode=0o700)
    os.umask(0o077)
    started = time.monotonic()
    result = dict(schema='openergy.metadata-reader-local-cli.v1',
                  status='INCOMPLETE', reader_sha256=actual, cases=[],
                  windows_contacted=False, windows_qualification=False,
                  historical_attempt_reopened=False, retries=0)
    cases = (
        ('success', ['/usr/bin/uname', '-s'], 2.0, 32768, 0),
        ('nonzero', ['/usr/bin/false'], 2.0, 32768, 1),
        ('partial-timeout', ['/bin/sh', '-c', 'printf "prefix\\n"; exec /bin/sleep 2'],
         0.25, 32768, 1),
        ('output-cap', ['/usr/bin/yes', 'coverage'], 2.0, 128, 1),
    )
    try:
        for label, argv, work, cap, expected in cases:
            if time.monotonic() - started >= 28:
                raise TimeoutError('whole local command-flow budget exhausted')
            binding = args.output / (label + '-command.json')
            encoded = (json.dumps(dict(schema='openergy.metadata-reader-command.v1',
                       purpose='local-reader-verification', argv=argv), sort_keys=True) + '\n').encode()
            binding.write_bytes(encoded)
            run_dir = args.output / label
            command = [sys.executable, '-B', str(reader), '--binding', str(binding),
                       '--binding-sha256', hashlib.sha256(encoded).hexdigest(),
                       '--run-dir', str(run_dir), '--work-seconds', str(work),
                       '--cleanup-seconds', '0.5', '--finalize-seconds', '0.5',
                       '--total-seconds', str(work + 1), '--stdout-cap', str(cap),
                       '--stderr-cap', str(cap), '--aggregate-cap', str(cap)]
            record = dict(label=label, command=command, expected_reader_exit=expected,
                          started_monotonic=time.monotonic(), accepted=False)
            result['cases'].append(record)
            process = subprocess.Popen(command, stdin=subprocess.DEVNULL,
                                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            record['reader_pid'] = process.pid
            try:
                code = process.wait(timeout=min(8, 28 - (time.monotonic() - started)))
            except subprocess.TimeoutExpired:
                # This retained, unreaped direct handle cannot have its PID reused.
                # No group/remote signal, retry, or descendant-cleanup claim.
                process.kill()
                record['reader_exit'] = process.wait(timeout=2)
                record['reader_outer_timeout'] = True
                record['reader_child_state'] = 'UNVERIFIED'
                raise
            record.update(reader_exit=code, ended_monotonic=time.monotonic())
            receipt_data = (run_dir / 'receipt.json').read_bytes()
            receipt = json.loads(receipt_data)
            stdout = (run_dir / 'stdout.bin').read_bytes()
            stderr = (run_dir / 'stderr.bin').read_bytes()
            if code != expected or not receipt['local_exit_observed']:
                raise RuntimeError(label + ': reader exit or actual child exit missing')
            if len(stdout) + len(stderr) > cap:
                raise RuntimeError(label + ': accepted bytes exceed combined cap')
            for name, data in (('stdout', stdout), ('stderr', stderr)):
                entry = receipt['streams'][name]
                if entry['bytes'] != len(data) or entry['sha256'] != hashlib.sha256(data).hexdigest():
                    raise RuntimeError(label + ': raw stream binding differs')
            if label == 'success':
                valid = receipt['status'] == 'COMPLETE' and receipt['local_child_returncode'] == 0 and stdout == b'Darwin\n' and not stderr
            elif label == 'nonzero':
                valid = receipt['status'] == 'INCOMPLETE' and receipt['local_child_returncode'] == 1 and not stdout and not stderr
            elif label == 'partial-timeout':
                valid = (receipt['status'] == 'INCOMPLETE' and stdout == b'prefix\n' and
                         receipt['local_kill_called'] and receipt['local_child_returncode'] == -signal.SIGKILL and
                         receipt['first_error']['class'] == 'TimeoutError')
            else:
                valid = (receipt['status'] == 'INCOMPLETE' and len(stdout) == cap and
                         receipt['streams']['stdout']['overflow_observed'] and
                         receipt['local_kill_called'] and receipt['local_child_returncode'] == -signal.SIGKILL)
            if not valid:
                raise RuntimeError(label + ': full CLI result differs from required outcome')
            record.update(accepted=True, receipt_sha256=hashlib.sha256(receipt_data).hexdigest(),
                          local_child_returncode=receipt['local_child_returncode'])
        result['status'] = 'LOCAL_CLI_CASES_PASSED'
    except BaseException as error:
        result['first_failure'] = dict(type=type(error).__name__)
    finally:
        result['elapsed_seconds'] = time.monotonic() - started
        (args.output / 'result.json').write_text(json.dumps(result, indent=2, sort_keys=True) + '\n')
    return 0 if result['status'] == 'LOCAL_CLI_CASES_PASSED' else 1


if __name__ == '__main__':
    raise SystemExit(main())
