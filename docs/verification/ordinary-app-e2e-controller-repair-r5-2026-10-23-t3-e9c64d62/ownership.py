"""Shared strict ownership policy; normal sampling and recovery have separate callers."""
from common import *
from macproc import key
import fcntl


def budget(runtime, deadline, row_count, actor):
    """One cross-process budget: 6000 rounds, 24000 retained process rows."""
    with io_until(deadline):
        with (runtime / 'budget.lock').open('a+b') as lock:
            while True:
                try:
                    fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
                    break
                except BlockingIOError:
                    pause(deadline)
            path = runtime / 'budget.json'
            value = load(path, deadline) if path.exists() else dict(rounds=0, rows=0)
            proposed = dict(rounds=value['rounds'] + 1, rows=value['rows'] + row_count)
            if actor == 'main' and (proposed['rounds'] > 4976 or proposed['rows'] > 19904):
                raise RuntimeError('main observation share exhausted; cleanup reserve untouched')
            if proposed['rounds'] > 6000 or proposed['rows'] > 24000:
                raise RuntimeError('global observation budget exhausted')
            value = proposed
            # Reserve cleanup coverage inside the existing total, rather than
            # consuming the whole budget in the failed main sampler.
            value['main_budget_exhausted'] = actor == 'main' and (
                value['rounds'] >= 4976 or value['rows'] >= 19904)
            temporary = runtime / ('budget-next-' + str(os.getpid()))
            with temporary.open('w') as stream:
                json.dump(value, stream)
            os.replace(temporary, path)
            return value


def expected(spec):
    root, bundle = Path(spec['private_root']), Path(spec['bundle'])
    app, daemon = str(bundle / 'Contents/MacOS/Openergy'), str(bundle / 'Contents/Helpers/openergy')
    return {
        'peer': [daemon, 'daemon', '--platform', 'virtual', '--config', str(root / 'peer/config.toml'),
                 '--socket', str(root / 'peer/control.sock'), '--foreground'],
        'app': [app, '--e2e', str(root / 'app'), '--join', f'127.0.0.1:{PORTS[0]}',
                '--host-port', str(PORTS[1]), '--require-peer'],
        'peer-status': [daemon, 'status', '--socket', str(root / 'peer/control.sock')],
        'peer-stop': [daemon, 'ctl', 'daemon.stop', '--socket', str(root / 'peer/control.sock'), '--raw'],
    }


def wrapper_argv(spec, runtime, label):
    return [spec['python_executable'], str(HERE / 'exec_owned.py'), str(runtime), label]


def exact_argv(actual, wanted):
    return (len(actual) == len(wanted) and actual[1:] == wanted[1:]
            and Path(actual[0]).resolve() == Path(wanted[0]).resolve())


class Inventory:
    def __init__(self, spec, runtime, actor):
        self.spec, self.runtime, self.actor = spec, Path(runtime), actor
        self.root = Path(spec['private_root'])
        self.commands = expected(spec)
        self.records = {}
        self.pending = {}
        self.sequence = 0
        self.last = None
        self.evidence = Evidence()
        self.deliveries = {}
        self.direct, self.launch_labels = {}, set()
        self.reload(min(spec['flow_deadline'] + CLEANUP_SECONDS, time.monotonic() + 2))

    def reload(self, deadline):
        with io_until(deadline):
            for path in sorted(self.runtime.glob('identity-*.json')):
                record = self.evidence.attempt(load, path, min(deadline, time.monotonic() + 0.05))
                if record is not None:
                    self.records[record['identity_id']] = record

    def known_pids(self):
        return {record['pid'] for record in self.records.values()}

    def observe(self, deadline, validate=True):
        with io_until(deadline):
            return self._observe(deadline, validate)

    def _observe(self, deadline, validate):
        """Raw partial rows are published before ownership/NI assertions can fail."""
        self.reload(deadline)
        self.sequence += 1
        groups = {record['pgid'] for record in self.records.values()}
        pids = self.known_pids()
        owner = self.spec['owner']
        direct = dict(self.direct)
        self.launch_labels |= {p.name.removesuffix('-command.json') for p in self.runtime.glob('*-command.json')}
        for label in self.commands:
            path = self.runtime / (label + '-exec.json')
            if path.exists():
                receipt = self.evidence.attempt(load, path, deadline)
                if receipt is not None:
                    direct[label] = receipt
            if label in direct:
                pids.add(direct[label]['pid'])
        self.direct = direct
        data = json.loads(command([PYTHON, HERE / 'probe.py', str(deadline), self.root,
                                   *sorted({key(r) for r in self.records.values()} |
                                           {key(r) for r in direct.values()} | {key(owner)})], deadline))
        selected = [r for r in data['rows'] if r['pid'] in pids or r['ppid'] in pids
                    or r['pgid'] in groups or (r.get('executable') and
                        Path(r['executable']).is_relative_to(self.root))]
        self.last = data  # Fresh rows survive failed publication or accounting.
        budget_error = None
        try:
            counts = budget(self.runtime, min(deadline, time.monotonic() + 0.05), len(selected), self.actor)
        except Exception as error:
            budget_error, counts = repr(error), None
        packet = dict(monotonic=time.monotonic(), actor=self.actor, sequence=self.sequence,
                      rows=selected, races=[r for r in data['races'] if r.get('known') or r['pid'] in pids],
                      absence=data['absence'], counts=counts)
        if counts is not None:
            self.evidence.attempt(append, self.runtime / (self.actor + '-observations.jsonl'),
                                  packet, min(deadline, time.monotonic() + 0.05), critical=self.actor != 'main')
        errors, seen_pending = [], set()
        all_rows = {r['pid']: r for r in data['rows']}
        direct_pids = {r['pid'] for r in direct.values()}
        for row in sorted(selected, key=lambda r: r['pid'] not in direct_pids):
            # Group membership alone never establishes ownership.
            candidate = (row['pid'] in pids or row['ppid'] in pids or (row.get('executable') and
                         Path(row['executable']).is_relative_to(self.root)))
            if not candidate:
                continue
            if row.get('identity_reused'):
                errors.append(dict(row=row, error='known PID reused; no argv read or signal'))
                continue
            if row.get('read_race') or 'argv' not in row:
                # Zombies can lack an executable/argv. A known start remains visible
                # for absence checks, but it cannot be signalled from a partial row.
                if row['status'] == 5 and any(key(r) == key(row) for r in self.records.values()):
                    continue
                seen_pending.add(row['pid'])
                started, attempts = self.pending.get(row['pid'], (time.monotonic(), 0))
                self.pending[row['pid']] = (started, attempts + 1)
                if attempts + 1 >= 8 or time.monotonic() - started >= 2:
                    errors.append('finite detail observation budget: ' + str(row['pid']))
                continue
            try:
                record = self.identify(row, all_rows, direct, deadline, data['absence'])
                if record is None:
                    raise RuntimeError('private candidate has no exact ownership chain')
                identity_id = key(row) + '-' + record['stage']
                record.update(row, identity_id=identity_id, observed_monotonic=time.monotonic())
                if identity_id not in self.records:
                    # Save ownership even if NI validation fails next.
                    self.records[identity_id] = record
                    self.evidence.attempt(first, self.runtime / ('identity-' + identity_id + '.json'), record, min(deadline, time.monotonic() + 0.05))
                if row['uid'] != owner['uid'] or row['ruid'] != owner['uid']:
                    raise RuntimeError('UID mismatch')
                if validate and (row['nice'] != 15 or row['nice_after'] != 15):
                    raise RuntimeError('observed product/wrapper NI != 15')
            except Exception as error:
                errors.append(dict(row=row, error=repr(error)))
        for race in packet['races']:
            seen_pending.add(race['pid'])
            started, attempts = self.pending.get(race['pid'], (time.monotonic(), 0))
            self.pending[race['pid']] = (started, attempts + 1)
            if attempts + 1 >= 8 or time.monotonic() - started >= 2:
                errors.append(dict(race=race, error='finite known-PID read budget'))
        for pid in list(self.pending):
            if pid not in seen_pending:
                del self.pending[pid]
        if budget_error:
            errors.append(dict(observation_budget_error=budget_error))
        if counts is not None and counts['main_budget_exhausted']:
            errors.append('main observation budget ended; cleanup reserve retained')
        if errors:
            raise RuntimeError(repr(errors))
        return data

    def identify(self, row, rows, direct, deadline, absence):
        if row['uid'] != self.spec['owner']['uid'] or row['ruid'] != row['uid']:
            raise RuntimeError('candidate UID mismatch')
        existing = [r for r in self.records.values() if key(r) == key(row)
                    and r['executable'] == row['executable'] and r['argv'] == row['argv']]
        if existing:
            record = dict(existing[0])
            parent_pid = int(record['parent_identity'].split('-')[0])
            if row['ppid'] == parent_pid:
                parent = rows.get(parent_pid)
                if parent is None or key(parent) != record['parent_identity']:
                    raise RuntimeError('verified parent absent, uncertain, or reused')
            elif not (row['ppid'] == 1 and parent_pid not in rows and
                      absence.get(str(parent_pid)) == 'ESRCH'):
                raise RuntimeError('unverified parent transition')
            else:
                record['orphaned_from_verified_parent'] = parent_pid
            if row['pgid'] != record['pgid']:
                raise RuntimeError('group changed after identity registration')
            return record
        for label, receipt in direct.items():
            if key(receipt) != key(row):
                continue
            if row['pgid'] != receipt['pgid']:
                raise RuntimeError('direct child group changed across exec')
            owner = self.spec['owner']
            parent = rows.get(owner['pid'])
            live_parent = (row['ppid'] == owner['pid'] and parent is not None and
                           key(parent) == key(owner) and not parent.get('read_race') and
                           parent.get('executable') == owner['executable'] and
                           parent.get('argv') == owner['argv'] and
                           parent['uid'] == owner['uid'] and parent['ruid'] == owner['uid'])
            if (row['executable'] == self.spec['python_executable'] and
                    exact_argv(row['argv'], wrapper_argv(self.spec, self.runtime, label))):
                if not live_parent:
                    raise RuntimeError('new wrapper requires live matched controller')
                return dict(role=label, stage='wrapper', parent_identity=key(owner))
            wanted = self.commands[label]
            if row['executable'] == str(Path(wanted[0]).resolve()) and exact_argv(row['argv'], wanted):
                registered = load(self.runtime / (label + '-registered.json'), deadline)
                if (key(registered) != key(row) or registered['stage'] != 'wrapper' or
                        registered['role'] != label or registered['pgid'] != row['pgid'] or
                        registered['ppid'] != owner['pid'] or registered['nice'] != 15 or
                        registered['nice_after'] != 15 or
                        registered['uid'] != owner['uid'] or registered['ruid'] != owner['uid'] or
                        registered['parent_identity'] != key(owner) or
                        registered['executable'] != self.spec['python_executable'] or
                        not exact_argv(registered['argv'], wrapper_argv(self.spec, self.runtime, label))):
                    raise RuntimeError('same-PID exec verified-wrapper gate mismatch')
                orphan_transition = (row['ppid'] == 1 and parent is None and
                                     absence.get(str(owner['pid'])) == 'ESRCH')
                if not (live_parent or orphan_transition):
                    raise RuntimeError('direct product requires live controller or explicit controller ESRCH')
                return dict(role=label, stage='product', parent_identity=key(owner),
                            controller_absent_exec_transition=orphan_transition)
        app_records = [r for r in self.records.values() if r['role'] == 'app' and r['stage'] == 'product']
        for app in app_records:
            parent = rows.get(app['pid'])
            if parent is None or key(parent) != key(app) or row['ppid'] != app['pid']:
                continue
            if parent.get('executable') != app['executable'] or parent.get('argv') != app['argv']:
                raise RuntimeError('supervised parent details uncertain')
            if parent['uid'] != self.spec['owner']['uid'] or parent['ruid'] != parent['uid']:
                raise RuntimeError('supervised parent UID changed')
            wanted = [str(Path(self.spec['bundle']) / 'Contents/Helpers/openergy'), 'daemon',
                      '--parent-pid', str(app['pid']), '--socket', str(self.root / 'app/config/control.sock'),
                      '--config', str(self.root / 'app/config/config.toml'), '--platform', 'virtual']
            if row['executable'] != wanted[0] or not exact_argv(row['argv'], wanted):
                continue
            if (row['start_sec'], row['start_usec']) < (app['start_sec'], app['start_usec']):
                raise RuntimeError('child started before matched parent')
            prior = {key(r) for r in self.records.values() if r['role'].endswith('supervised-daemon')}
            role = 'initial-supervised-daemon' if not prior else 'recovered-supervised-daemon'
            return dict(role=role, stage='product', parent_identity=key(app))
        return None

    def signal(self, identity, sig, deadline):
        with io_until(deadline):
            return self._signal(identity, sig, deadline)

    def _signal(self, identity, sig, deadline):
        # Fresh independent read immediately before each single-PID signal.
        data = json.loads(command([PYTHON, HERE / 'probe.py', str(deadline), self.root,
                                   '--targeted', key(identity), identity['parent_identity']], deadline))
        rows = {r['pid']: r for r in data['rows']}
        row = rows.get(identity['pid'])
        if row is None:
            if data['absence'].get(str(identity['pid'])) == 'ESRCH':
                return dict(delivered=False, result='ESRCH')
            raise RuntimeError('signal target absence uncertain')
        if (key(row) != key(identity) or row.get('read_race') or
                row.get('executable') != identity['executable'] or row.get('argv') != identity['argv'] or
                row['uid'] != self.spec['owner']['uid'] or row['ruid'] != row['uid'] or
                row['pgid'] != identity['pgid']):
            raise RuntimeError('refuse changed/reused/uncertain signal target')
        parent_pid = int(identity['parent_identity'].split('-')[0])
        parent = rows.get(parent_pid)
        if row['ppid'] == parent_pid:
            if parent is None or key(parent) != identity['parent_identity']:
                raise RuntimeError('refuse uncertain signal parent')
            parents = [r for r in self.records.values() if key(r) == identity['parent_identity']]
            parents.sort(key=lambda r: r['stage'] == 'product', reverse=True)
            expected_parent = self.spec['owner'] if key(self.spec['owner']) == identity['parent_identity'] else (
                parents[0] if parents else None)
            if (expected_parent is None or parent.get('read_race') or
                    parent.get('executable') != expected_parent['executable'] or
                    parent.get('argv') != expected_parent['argv'] or
                    parent['uid'] != self.spec['owner']['uid'] or parent['ruid'] != parent['uid']):
                raise RuntimeError('refuse changed/uncertain signal parent details')
        elif not (row['ppid'] == 1 and data['absence'].get(str(parent_pid)) == 'ESRCH'):
            raise RuntimeError('refuse unmatched signal parent')
        self.evidence.attempt(append, self.runtime / (self.actor + '-signals.jsonl'),
               dict(identity=identity, current=row, signal=int(sig), before_monotonic=time.monotonic()),
               min(deadline, time.monotonic() + 0.05), critical=True)
        # Verification, never evidence publication, authorizes this single PID.
        # The final read-to-signal interval is not an atomic macOS identity lease.
        # Keep a delivered signal and its in-memory time together across our
        # own I/O alarm. Only this constant-size, nonblocking action is masked.
        previous_mask = signal.pthread_sigmask(signal.SIG_BLOCK, {signal.SIGALRM})
        try:
            try:
                os.kill(row['pid'], sig)
            except ProcessLookupError:
                return dict(delivered=False, result='ESRCH after verification')
            receipt = dict(identity_id=key(identity), signal=int(sig), delivered=True,
                           delivered_monotonic=time.monotonic())
            self.deliveries.setdefault((key(identity), int(sig)), receipt)
        finally:
            signal.pthread_sigmask(signal.SIG_SETMASK, previous_mask)
        self.evidence.attempt(append, self.runtime / (self.actor + '-signals.jsonl'),
                              receipt, min(deadline, time.monotonic() + 0.05), critical=True)
        return receipt

    def absent(self, data, roles=None):
        records = [r for r in self.records.values() if roles is None or roles(r['role'])]
        # A launch intent with no verified identity is uncertainty, never absence.
        if any((roles is None or roles(label)) and not any(r['role'] == label for r in records)
               for label in self.launch_labels):
            return False
        rows = {r['pid']: r for r in data['rows']}
        # PID reuse is a recorded cleanup failure, never treated as disappearance.
        return (not self.pending and all(r['pid'] not in rows and
                    data['absence'].get(str(r['pid'])) == 'ESRCH' for r in records))

    def final(self, data, deadline):
        with io_until(deadline):
            return self._final(data)

    def _final(self, data):
        pids = self.known_pids()
        groups = {r['pgid'] for r in self.records.values()}
        leftovers = [r for r in data['rows'] if r['pid'] in pids or r['pgid'] in groups
                     or (r.get('executable') and Path(r['executable']).is_relative_to(self.root))]
        sockets = []
        for path in (self.root / 'app/config/control.sock', self.root / 'peer/control.sock'):
            sockets.append(dict(path=str(path), exists=os.path.lexists(path)))
        return dict(known_pids=sorted(pids), actual_groups=sorted(groups),
                    absence=data['absence'], remaining=leftovers, sockets=sockets,
                    absent=self.absent(data) and not leftovers and not any(s['exists'] for s in sockets))
