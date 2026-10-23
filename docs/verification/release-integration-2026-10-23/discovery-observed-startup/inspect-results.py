#!/usr/bin/env python3
"""Inspect only the saved real-process evidence; no generated or altered inputs."""
import collections
import json
from pathlib import Path
import re
import sys


def inspect(root, label, baseline=False):
    stdout = (root / f'{label}.stdout').read_text()
    stderr = (root / f'{label}.stderr').read_text()
    result = json.loads((root / f'{label}.result.json').read_text())
    calls = collections.Counter(re.findall(
        r'CALL (?:held|drop-distinct|drop-replacement) type=(\S+) kind=(limited|standard)', stderr))
    sends = re.findall(r'SEND type=(\S+) kind=(limited|standard) outcome=(\w+)', stderr)
    observed = collections.Counter((ty, kind) for ty, kind, _ in sends)
    if calls != observed or any(outcome not in ('Accepted', 'Disconnected') for _, _, outcome in sends):
        raise ValueError(f'{label}: missing, extra or unknown send outcome')
    distinct = [item for item in sends if re.fullmatch(r'_oe-c\d{3}\._tcp\.local\.', item[0])]
    expected = {f'_oe-c{i:03}._tcp.local.' for i in range(32)}
    if len(distinct) != 32 or {ty for ty, _, _ in distinct} != expected:
        raise ValueError(f'{label}: 32 distinct outcomes not matched')
    failed = sum(outcome == 'Disconnected' for _, _, outcome in distinct)
    if failed == 0:
        raise ValueError(f'{label}: failed distinct-start path not covered')
    replacements = re.findall(
        r'REPLACEMENT case=(\S+) attempt=(\d+) type=(\S+) outcome=(\w+) before=(\d+) after=(\d+) old_closed=(\w+)', stdout)
    cases = {'limited-over-limited', 'standard-over-limited', 'limited-over-standard', 'standard-over-standard'}
    if {row[0] for row in replacements if row[3] == 'Disconnected'} != cases:
        raise ValueError(f'{label}: incomplete failed replacement coverage')
    checks = re.findall(r'^CHECK (\S+)=(true|false)$', stdout, re.M)
    summary = re.search(r'STARTUP_SUMMARY passed=(\d+) total=(\d+)', stdout)
    if not summary or int(summary[2]) != len(checks) or int(summary[1]) != sum(ok == 'true' for _, ok in checks):
        raise ValueError(f'{label}: absent or inconsistent check summary')
    false_checks = [name for name, ok in checks if ok == 'false']
    if baseline:
        needed = {'distinct-eventual-exact-guard-state', 'limited-over-limited-failed-preserves-exact-identity-state'}
        if result['exit'] != 1 or not needed.issubset(false_checks) or 'unverified initial send' in stderr:
            raise ValueError('baseline did not fail on the target behavior with a working observer')
    elif result['exit'] != 0 or false_checks:
        raise ValueError(f'{label}: fixed acceptance failed')
    samples = re.findall(r'STATE_SAMPLE elapsed_ms=(\d+) expected=(\d+) actual=(\d+)', stdout)
    return {'label': label, 'exit': result['exit'], 'checks_passed': int(summary[1]),
            'checks_total': int(summary[2]), 'calls_matched': sum(calls.values()),
            'distinct_disconnected': failed, 'distinct_accepted': 32 - failed,
            'replacement_observations': replacements, 'false_checks': false_checks,
            'state_samples': samples}


def main():
    root = Path(sys.argv[1]) if len(sys.argv) == 2 else Path(__file__).parent / 'outputs'
    reports = [inspect(root, 'baseline-startup-final-v2', baseline=True)]
    reports.extend(inspect(root, f'fixed-startup-final-{i}') for i in (1, 2))
    for label, count in (('missing-observer', 0), ('ambiguous-observer', 2)):
        stdout = (root / f'{label}.stdout').read_text()
        stderr = (root / f'{label}.stderr').read_text()
        result = json.loads((root / f'{label}.result.json').read_text())
        sends = re.findall(r'SEND type=\S+ kind=\S+ outcome=Accepted', stderr)
        if (result['exit'] != 1 or f'matches={count}' not in stderr or len(sends) != count
                or 'STARTUP_SUMMARY' in stdout
                or 'UNVERIFIED_CLEANUP publisher_exits_zero=true worker_shutdown_ack=true' not in stdout):
            raise ValueError(f'{label}: evidence rejection/cleanup not established')
        reports.append({'label': label, 'exit': 1, 'real_send_records': count,
                        'unverified_evidence_rejected': True, 'normal_cleanup': True})
    print(json.dumps(reports, indent=2))


if __name__ == '__main__':
    main()
