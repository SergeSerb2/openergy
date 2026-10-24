"""Independent bounded recovery process. Never cancels on a controller exception."""
from common import *
from ownership import Inventory
from macproc import api, bsd, detail, key
import signal
import sys
import traceback


def app_role(role):
    return role == 'app' or role.endswith('supervised-daemon')


class Recovery:
    def __init__(self, spec, runtime, request, actor='watchdog'):
        self.spec, self.runtime, self.request = spec, Path(runtime), request
        self.actor = actor
        self.deadline = request['cleanup_deadline']
        self.inventory = Inventory(spec, runtime, actor)
        self.phase = 'app-grace' if request['normal'] else 'app-term'
        self.until = min(self.deadline, time.monotonic() + (10 if request['normal'] else 5))
        self.sent = set()
        self.failures = []
        self.last_good = None
        self.done = False
        self.aborting = not request['normal']
        self.emergency_phase = 'app-term'
        self.emergency_until = None
        if self.aborting:
            first(self.runtime / 'abort.json', dict(reason='declared recovery request',
                  monotonic=time.monotonic()), min(self.deadline, time.monotonic() + 0.5))

    def fail(self, message):
        value = dict(monotonic=time.monotonic(), phase=self.phase, error=message)
        self.failures.append(value)
        try:
            first(self.runtime / ('first-' + self.actor + '-failure.json'), value,
                  min(self.deadline, time.monotonic() + 0.5))
        except Exception as error:
            # A failing evidence sink must not throw out of cleanup either.
            value['evidence_write_error'] = repr(error)

    def move(self, phase, grace):
        self.phase, self.until = phase, min(self.deadline, time.monotonic() + grace)

    def signals(self, roles, sig):
        # Prefer the current product stage over the old same-PID wrapper stage.
        records = sorted(self.inventory.records.values(), key=lambda r: r['stage'] == 'product', reverse=True)
        attempted = set()
        current = {r['pid']: r for r in self.last_good['rows']}
        for record in records:
            if not roles(record['role']) or record['pid'] in attempted:
                continue
            row = current.get(record['pid'])
            if row is None or row['status'] == 5:
                continue
            attempted.add(record['pid'])
            token = (record['identity_id'], int(sig))
            if token in self.sent:
                continue
            # A failed verification is recorded and retried only within this
            # finite phase. It never falls through to Popen.terminate/kill.
            try:
                limit = min(self.deadline, time.monotonic() + 1)
                if row['status'] == 4 and sig == signal.SIGTERM:
                    self.inventory.signal(record, signal.SIGCONT, limit)
                self.inventory.signal(record, sig, limit)
                self.sent.add(token)
            except Exception:
                self.fail(traceback.format_exc())

    def step(self):
        try:
            with io_until(min(self.deadline, time.monotonic() + 2)):
                self.advance()
        except Exception:
            self.fail(traceback.format_exc())

    def emergency(self):
        """Known-identity recovery survives enumeration or sample-budget failure.

        Targeted reads may stop the verified app/children, but cannot establish
        whole-tree absence or authorize peer teardown without a full inventory.
        """
        limit = min(self.deadline, time.monotonic() + 1)
        known = {key(r) for r in self.inventory.records.values()} | {key(self.spec['owner'])}
        known |= {r['parent_identity'] for r in self.inventory.records.values()}
        direct = {}
        for label in self.inventory.commands:
            path = self.runtime / (label + '-exec.json')
            if path.exists():
                direct[label] = load(path, limit)
                known.add(key(direct[label]))
        data = json.loads(command([PYTHON, HERE / 'probe.py', str(limit), self.spec['private_root'],
                                   '--targeted', *sorted(known)], limit))
        first(self.runtime / ('first-' + self.actor + '-targeted-recovery.json'),
              dict(data=data, monotonic=time.monotonic()), limit)
        rows = {row['pid']: row for row in data['rows']}
        for row in data['rows']:
            if row['pid'] == self.spec['owner']['pid'] or row.get('read_race') or 'argv' not in row:
                continue
            try:
                record = self.inventory.identify(row, rows, direct, limit)
                if record is not None:
                    identity_id = key(row) + '-' + record['stage']
                    record.update(row, identity_id=identity_id, observed_monotonic=time.monotonic())
                    first(self.runtime / ('identity-' + identity_id + '.json'), record, limit)
                    self.inventory.records[identity_id] = record
            except Exception:
                self.fail(traceback.format_exc())
        self.last_good = data
        now = time.monotonic()
        if self.emergency_until is None:
            self.emergency_until = min(self.deadline, now + 5)
        app_records = [r for r in self.inventory.records.values() if r['role'] == 'app']
        app_absent = all(data['absence'].get(str(r['pid'])) == 'ESRCH' for r in app_records)
        if self.emergency_phase == 'app-term':
            self.signals(lambda role: role == 'app', signal.SIGTERM)
            if app_absent:
                self.emergency_phase, self.emergency_until = 'children-term', min(self.deadline, now + 5)
            elif now >= self.emergency_until:
                self.emergency_phase, self.emergency_until = 'app-kill', min(self.deadline, now + 5)
        if self.emergency_phase == 'app-kill':
            self.signals(lambda role: role == 'app', signal.SIGKILL)
            if app_absent:
                self.emergency_phase, self.emergency_until = 'children-term', min(self.deadline, now + 5)
        if self.emergency_phase == 'children-term':
            self.signals(lambda role: role.endswith('supervised-daemon'), signal.SIGTERM)
            if now >= self.emergency_until:
                self.emergency_phase = 'children-kill'
        if self.emergency_phase == 'children-kill':
            self.signals(lambda role: role.endswith('supervised-daemon'), signal.SIGKILL)

    def advance(self):
        """Never lets an observer exception unwind the cleanup state machine."""
        if self.done:
            return
        self.inventory.last = None
        validated = False
        try:
            self.last_good = self.inventory.observe(min(self.deadline, time.monotonic() + 1), validate=True)
            validated = True
        except Exception:
            self.fail(traceback.format_exc())
            if not self.aborting:
                first(self.runtime / 'abort.json', dict(reason='cleanup observation failed',
                      monotonic=time.monotonic()), min(self.deadline, time.monotonic() + 0.5))
                self.aborting = True
                self.move('app-term', 5)
            # Persisted identities remain usable only with an independent fresh
            # pre-signal read. Never use an old row to declare absence.
            self.last_good = self.inventory.last
        if self.last_good is None:
            # Even enumeration/budget failure must not disable exact owned
            # recovery. The pre-signal verifier performs a new targeted read.
            # No whole-tree absence or peer-teardown decision uses this route.
            self.emergency()
            return
        data = self.last_good
        app_gone = validated and self.inventory.absent(data, lambda role: role == 'app')
        app_tree_gone = validated and self.inventory.absent(data, app_role)
        now = time.monotonic()
        # CLI waits never suspend observation or postpone their five-second
        # limit until the peer's later grace period. TERM gets its own five
        # seconds before escalation; every signal rechecks the exact identity.
        for label in ('peer-status', 'peer-stop'):
            path = self.runtime / (label + '-command.json')
            if path.exists():
                command_row = load(path, min(self.deadline, now + 0.5))
                if now >= command_row['absolute_deadline']:
                    active = [r for r in self.inventory.records.values() if r['role'] == label
                              and any(row['pid'] == r['pid'] for row in data['rows'])]
                    if active:
                        self.fail(label + ' exceeded its absolute CLI deadline')
                        self.signals(lambda role, label=label: role == label,
                                     signal.SIGKILL if now >= command_row['absolute_deadline'] + 5 else signal.SIGTERM)
        if self.phase == 'app-grace':
            if app_tree_gone:
                self.move('peer-request', 0)
            elif now >= self.until:
                self.fail('app/children remained after normal 10-second grace')
                first(self.runtime / 'abort.json', dict(reason='normal cleanup grace expired', monotonic=now), self.deadline)
                self.move('app-term', 5)
        if self.phase == 'app-term':
            self.signals(lambda role: role == 'app', signal.SIGTERM)
            if app_gone:
                self.move('children-term', 5)
            elif now >= self.until:
                self.move('app-kill', 5)
        if self.phase == 'app-kill':
            self.signals(lambda role: role == 'app', signal.SIGKILL)
            if app_gone:
                self.move('children-term', 5)
            elif now >= self.until:
                self.fail('owned app absence not established; peer must remain alive')
                self.move('app-blocked', max(0, self.deadline - now))
        if self.phase == 'children-term':
            self.signals(lambda role: role.endswith('supervised-daemon'), signal.SIGTERM)
            if app_tree_gone:
                self.move('peer-request', 0)
            elif now >= self.until:
                self.move('children-kill', 5)
        if self.phase == 'children-kill':
            self.signals(lambda role: role.endswith('supervised-daemon'), signal.SIGKILL)
            if app_tree_gone:
                self.move('peer-request', 0)
            elif now >= self.until:
                self.fail('owned child absence not established; peer must remain alive')
                self.move('app-blocked', max(0, self.deadline - now))
        if self.phase == 'peer-request':
            # This is the only transition that permits peer teardown. Unknown
            # private candidates also prevent this transition via observe().
            first(self.runtime / 'peer-stop-permitted.json',
                  dict(monotonic=now, app_tree_absent=True,
                       declared_abort=(self.runtime / 'abort.json').exists()), self.deadline)
            self.move('peer-grace', 10)
        if self.phase == 'peer-grace':
            if validated and self.inventory.absent(data, lambda role: role == 'peer'):
                self.move('all-exits', 0)
            elif now >= self.until:
                self.fail('peer remained after 10-second public-stop allowance')
                self.move('peer-term', 5)
        if self.phase == 'peer-term':
            self.signals(lambda role: role.startswith('peer'), signal.SIGTERM)
            if validated and self.inventory.absent(data, lambda role: role.startswith('peer')):
                self.move('all-exits', 0)
            elif now >= self.until:
                self.move('peer-kill', 5)
        if self.phase == 'peer-kill':
            self.signals(lambda role: role.startswith('peer'), signal.SIGKILL)
            if validated and self.inventory.absent(data, lambda role: role.startswith('peer')):
                self.move('all-exits', 0)
            elif now >= self.until:
                self.fail('peer/CLI absence not established')
                self.move('blocked', max(0, self.deadline - now))
        if self.phase == 'all-exits' and validated:
            final = self.inventory.final(data, self.deadline)
            # The controller must reap every direct product/helper child.
            launches = [load(p, self.deadline) for p in self.runtime.glob('*-spawn.json')]
            exits = {p.name.removesuffix('-exit.json') for p in self.runtime.glob('*-exit.json')}
            collected = all(item['label'] in exits for item in launches)
            for label in ('peer-status', 'peer-stop'):
                if label in exits:
                    exit_row = load(self.runtime / (label + '-exit.json'), self.deadline)
                    if exit_row['code'] != 0:
                        self.fail(label + ' OS exit was nonzero: ' + str(exit_row['code']))
            if final['absent'] and collected:
                self.finish(final, True)

    def finish(self, final=None, collected=False):
        if self.done:
            return
        # A failed bounded recovery is a terminal, explicit failure. It never
        # writes PASS merely because a deadline or a signal was reached.
        result = dict(actor=self.actor, normal_request=self.request['normal'], phase=self.phase,
                      start=self.request['cleanup_start'], end=time.monotonic(),
                      cleanup_deadline=self.deadline, failures=self.failures,
                      all_direct_exits_collected=collected, inventory=final,
                      recovered=bool(final and final['absent'] and collected),
                      passed=bool(final and final['absent'] and collected and not self.failures))
        first(self.runtime / 'recovery-done.json', result, time.monotonic() + 1)
        self.done = True


def main():
    nice15()
    runtime = Path(sys.argv[1])
    spec = load(runtime / 'session.json', time.monotonic() + 2)
    hard_end = spec['flow_deadline'] + CLEANUP_SECONDS
    with io_until(time.monotonic() + 1):
        lib = api()
        identity = detail(lib, bsd(lib, os.getpid()))
        save(runtime / 'watchdog-ready.json', dict(identity=identity, monotonic=time.monotonic(),
             flow_deadline=spec['flow_deadline'], hard_end=hard_end), time.monotonic() + 1)
    request = None
    while time.monotonic() < spec['flow_deadline']:
        try:
            with io_until(min(spec['flow_deadline'], time.monotonic() + 1)):
                if (runtime / 'cleanup-request.json').exists():
                    request = load(runtime / 'cleanup-request.json', min(hard_end, time.monotonic() + 1))
                    break
                if (runtime / 'abort.json').exists() or os.getppid() != spec['owner']['pid']:
                    start = time.monotonic()
                    first(runtime / 'abort.json', dict(reason='controller parent lost', monotonic=start), hard_end)
                    request = dict(normal=False, cleanup_start=start,
                                   cleanup_deadline=min(start + CLEANUP_SECONDS, hard_end))
                    break
            pause(spec['flow_deadline'])
        except DeadlineError:
            break
    if request is None:
        start = time.monotonic()
        first(runtime / 'abort.json', dict(reason='independent flow watchdog expired', monotonic=start), hard_end)
        request = dict(normal=False, cleanup_start=start, cleanup_deadline=hard_end)
    # Clamp even a corrupt/late controller request to the predeclared hard end.
    request['cleanup_deadline'] = min(request['cleanup_deadline'], hard_end)
    recovery = Recovery(spec, runtime, request)
    while time.monotonic() < recovery.deadline and not recovery.done:
        try:
            recovery.step()
        except Exception:
            # Includes evidence-write failures. Keep the watchdog and collector
            # budget; the outer boundary still produces a failed outcome.
            recovery.fail(traceback.format_exc())
        if not recovery.done:
            try:
                pause(recovery.deadline)
            except DeadlineError:
                break
    if not recovery.done:
        recovery.finish()
    return 0 if recovery.done and load(runtime / 'recovery-done.json', time.monotonic() + 1)['passed'] else 1


if __name__ == '__main__':
    try:
        sys.exit(main())
    except BaseException:
        # Fatal watchdog loss is detected by the parent's direct-child poll.
        # Parent then enters separate fallback recovery, never cancels products.
        raise
