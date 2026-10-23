#!/usr/bin/env python3
"""Check the saved R7 real-process records without synthetic inputs."""
import collections
import json
from pathlib import Path
import re
import sys


def read(root, label):
    return ((root / f'{label}.stdout').read_text(),
            (root / f'{label}.stderr').read_text(),
            json.loads((root / f'{label}.result.json').read_text()))


def position(trace, pattern, after=0):
    match = re.search(pattern, trace[after:])
    if not match:
        raise ValueError(f'missing trace: {pattern}')
    return after + match.start()


def inspect(root, label, baseline=False, missing_cleanup=False):
    stdout, trace, result = read(root, label)
    calls = collections.Counter(re.findall(
        r'CALL (?:held|drop-distinct|drop-replacement) type=(\S+) kind=(limited|standard)', trace))
    sends = re.findall(r'SEND type=(\S+) kind=(limited|standard) outcome=(\w+)', trace)
    if calls != collections.Counter((ty, kind) for ty, kind, _ in sends):
        raise ValueError(f'{label}: unmatched request/send evidence')
    if any(value not in ('Accepted', 'Disconnected') for _, _, value in sends):
        raise ValueError(f'{label}: unknown/Full initial result')
    distinct = [row for row in sends if re.fullmatch(r'_oe-c\d{3}\._tcp\.local\.', row[0])]
    if len(distinct) != 32 or {row[0] for row in distinct} != {f'_oe-c{i:03}._tcp.local.' for i in range(32)}:
        raise ValueError(f'{label}: missing/ambiguous distinct results')
    failed = sum(row[2] == 'Disconnected' for row in distinct)
    if failed == 0:
        raise ValueError(f'{label}: no actual failed-start witness')
    rows = re.findall(r'REPLACEMENT case=(\S+) attempt=(\d+) type=(\S+) outcome=(\w+) before=(\d+) after=(\d+) old_closed=(\w+)', stdout)
    cases = {'limited-over-limited', 'standard-over-limited', 'limited-over-standard', 'standard-over-standard'}
    if {row[0] for row in rows if row[3] == 'Disconnected'} != cases:
        raise ValueError(f'{label}: failed replacement coverage missing')
    checks = dict(re.findall(r'^CHECK (\S+)=(true|false)$', stdout, re.M))
    summary = re.search(r'STARTUP_SUMMARY passed=(\d+) total=(\d+)', stdout)
    false_checks = [name for name, ok in checks.items() if ok == 'false']
    if not summary or int(summary[2]) != len(checks) or int(summary[1]) != len(checks) - len(false_checks):
        raise ValueError(f'{label}: absent/inconsistent summary')
    if baseline:
        if (result['exit'] != 1 or 'unverified initial send' in trace
                or not {'distinct-eventual-exact-guard-state', 'limited-over-limited-failed-preserves-exact-identity-state'}.issubset(false_checks)):
            raise ValueError('baseline is not red on actual retention/identity')
    elif missing_cleanup:
        if result['exit'] != 1 or checks.get('required-held-standard-start-resolution-commit-active') != 'true':
            raise ValueError('missing-cleanup control did not reach actual active standard case')
        if not {'required-accepted-standard-drop-automatic-removal', 'required-accepted-standard-drop-coverage'}.issubset(false_checks):
            raise ValueError('missing automatic cleanup evidence passed')
    elif result['exit'] != 0 or false_checks:
        raise ValueError(f'{label}: fixed flow failed')
    repeat_order = []
    if not baseline:
        for case, _, ty, outcome, *_ in rows:
            if case.endswith('-over-standard') and outcome == 'Disconnected':
                kind = case.split('-over-')[0]
                sent = position(trace, rf'SEND type={re.escape(ty)} kind={kind} outcome=Disconnected')
                drain = position(trace, rf'DRAIN_AFTER_FAILED_BARRIER type={re.escape(ty)} outcome=Disconnected', sent)
                wait = re.search(r'REPEAT_WAIT observed_starts=(\d+) deadline_secs=(\d+)', trace[drain:])
                if not wait:
                    raise ValueError('fresh repeat deadline absent')
                count, seconds = map(int, wait.groups())
                if count > 7 or seconds != (1 << max(0, count - 2)) + 2:
                    raise ValueError('repeat deadline does not cover observed retry interval')
                repeat = position(trace, rf'REPEAT received {re.escape(ty)} on ', drain)
                repeat_order.append({'case': case, 'type': ty, 'failed_send_before_post_barrier_drain_before_fresh_repeat': sent < drain < repeat,
                                     'observed_starts': count, 'deadline_seconds': seconds})
    ty = '_oe-drop-std._tcp.local.'
    sent = position(trace, rf'SEND type={re.escape(ty)} kind=standard outcome=Accepted')
    dropped = position(trace, rf'DROP held-active-standard-complete type={re.escape(ty)}', sent)
    removal = re.findall(rf'REMOVE type={re.escape(ty)} kind=(\w+) reason=(\w+) completed=(\w+)', trace)
    if missing_cleanup:
        if removal or f'AUTOMATIC_CLEANUP type={ty} kind=standard witnessed_in_time=false matched=0 valid=0 states=1 passed=false' not in stdout:
            raise ValueError('missing cleanup control did not reject gauge=1 without removal evidence')
        barrier = position(trace, rf'CLEANUP_BARRIER type={re.escape(ty)} kind=standard count=0 valid=0 passed=false', dropped)
        position(trace, rf'EXPLICIT_STOP_AFTER_FAILED_AUTOMATIC_CHECK type={re.escape(ty)}', barrier)
    else:
        if removal != [('standard', 'Disconnected', 'true')]:
            raise ValueError('required normal standard removal absent/duplicate/Full/unknown')
        removed = position(trace, rf'REMOVE type={re.escape(ty)} kind=standard reason=Disconnected completed=true', sent)
        barrier = position(trace, rf'CLEANUP_BARRIER type={re.escape(ty)} kind=standard count=1 valid=1 passed=true', max(dropped, removed))
        if 'EXPLICIT_STOP_AFTER_FAILED_AUTOMATIC_CHECK type=' + ty in trace:
            raise ValueError('explicit stop masked required automatic cleanup')
        if checks.get('required-accepted-standard-drop-coverage') != 'true':
            raise ValueError('accepted standard drop case missing')
    samples = re.findall(r'STATE_SAMPLE elapsed_ms=(\d+) expected=(\d+) actual=(\d+)', stdout)
    return {'label': label, 'exit': result['exit'], 'passed': int(summary[1]), 'total': int(summary[2]),
            'matched_initial_requests': sum(calls.values()), 'distinct_disconnected': failed,
            'distinct_accepted': 32 - failed, 'replacement_observations': rows,
            'fresh_repeat_evidence': repeat_order, 'required_standard_normal_removal': not missing_cleanup,
            'false_checks': false_checks, 'state_samples': samples}


def main():
    root = Path(sys.argv[1]) if len(sys.argv) == 2 else Path(__file__).parent / 'outputs'
    records = [inspect(root, 'baseline-startup', baseline=True)]
    records += [inspect(root, f'fixed-startup-{i}') for i in (1, 2)]
    records += [inspect(root, 'missing-cleanup-observer', missing_cleanup=True)]
    for label, count in (('missing-observer', 0), ('ambiguous-observer', 2)):
        stdout, trace, result = read(root, label)
        sends = re.findall(r'SEND type=\S+ kind=\S+ outcome=Accepted', trace)
        if (result['exit'] != 1 or f'matches={count}' not in trace or len(sends) != count
                or 'STARTUP_SUMMARY' in stdout
                or 'UNVERIFIED_CLEANUP publisher_exits_zero=true worker_shutdown_ack=true' not in stdout):
            raise ValueError(f'{label}: negative evidence or normal cleanup missing')
        records.append({'label': label, 'exit': 1, 'real_send_records': count, 'unverified_evidence_rejected': True})
    print(json.dumps(records, indent=2))


if __name__ == '__main__':
    main()
