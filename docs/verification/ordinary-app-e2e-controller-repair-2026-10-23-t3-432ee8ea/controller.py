"""One LATER ordinary AppEndToEnd flow. SOURCE ONLY: never invoked in this repair.

Products are direct children here. A detached watchdog survives any normal
controller exception. The watchdog uses a separate recovery observer, while
this process keeps collecting all direct-child OS exit statuses.
"""
from common import *
from macproc import api, bsd, detail, key
from ownership import Inventory, expected
from prepare import bind_sources
from watchdog import Recovery
import re
import socket
import subprocess
import traceback


class FlowAbort(RuntimeError):
    pass


class Controller:
    def __init__(self, prepared):
        self.prepared = prepared
        self.runtime = HERE / 'runtime'
        self.processes, self.exits, self.reservations = {}, {}, {}
        self.watchdog = None
        self.spec = None
        self.inventory = None
        self.observer_failed = False
        self.failed = False
        self.cleanup_deadline = None
        self.fallback = None

    def collect(self, deadline):
        """Nonblocking waitpid via Popen.poll, also called after sampler failure."""
        for label, process in self.processes.items():
            code = process.poll()
            if code is not None:
                value = self.exits.get(label) or dict(label=label, pid=process.pid, code=code,
                             signal=(-code if code < 0 else None), collected_monotonic=time.monotonic(),
                             collector_pid=os.getpid(), complete_os_exit_status=True)
                self.exits[label] = value
                first(self.runtime / (label + '-exit.json'), value, deadline)
                if code != 0:
                    self.failed = True
                    first(self.runtime / 'first-child-exit-failure.json', value, deadline)

    def abort(self, error, deadline):
        self.failed = True
        first(self.runtime / 'first-runtime-failure.json',
              dict(error=error, monotonic=time.monotonic()), deadline)
        first(self.runtime / 'abort.json', dict(reason=error, monotonic=time.monotonic()), deadline)
        self.request_cleanup(False, deadline)

    def request_cleanup(self, normal, io_deadline):
        if self.cleanup_deadline is None:
            start = time.monotonic()
            self.cleanup_deadline = min(start + CLEANUP_SECONDS,
                                        self.spec['flow_deadline'] + CLEANUP_SECONDS)
            first(self.runtime / 'cleanup-request.json',
                  dict(normal=normal, cleanup_start=start, cleanup_deadline=self.cleanup_deadline), io_deadline)

    def observe(self, deadline):
        if self.observer_failed:
            return
        try:
            self.inventory.observe(min(deadline, time.monotonic() + 1))
            paths = [self.runtime / (label + '.log') for label in self.processes]
            paths += [Path(self.spec['private_root']) / 'app/config' / name
                      for name in ('daemon-stderr.log', 'openergy.log')]
            with io_until(deadline):
                if any(p.exists() and p.stat().st_size > LOG_CAP for p in paths):
                    raise RuntimeError('log byte limit exceeded')
                if sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file()) > EVIDENCE_CAP:
                    raise RuntimeError('evidence byte limit exceeded')
        except Exception:
            self.observer_failed = True
            error = traceback.format_exc()
            first(self.runtime / 'first-observer-failure.json', dict(error=error, monotonic=time.monotonic()),
                  time.monotonic() + 0.5)
            self.abort('observation failed; whole flow aborted: ' + error, time.monotonic() + 0.5)

    def tick(self, deadline):
        self.collect(min(deadline, time.monotonic() + 1))
        self.observe(deadline)
        if self.observer_failed or (self.runtime / 'abort.json').exists():
            raise FlowAbort('declared abort; peer must remain alive until app/children absent')
        if self.watchdog is not None and self.watchdog.poll() is not None:
            raise FlowAbort('independent watchdog exited during flow')
        pause(deadline)

    def launch(self, label, deadline, cleanup=False):
        with io_until(deadline):
            return self._launch(label, deadline, cleanup)

    def _launch(self, label, deadline, cleanup):
        if label in self.processes:
            raise RuntimeError('no second launch: ' + label)
        wanted = expected(self.spec)[label]
        save(self.runtime / (label + '-command.json'),
             dict(label=label, argv=wanted, monotonic=time.monotonic(), absolute_deadline=deadline,
                  environment_allowlist=self.spec['app_environment'] if label == 'app' else self.spec['peer_environment'],
                  exec_helper_sha256=sha(HERE / 'exec_owned.py', deadline)), deadline)
        with io_until(deadline):
            with (self.runtime / (label + '.log')).open('xb') as log:
                process = subprocess.Popen([PYTHON, str(HERE / 'exec_owned.py'), str(self.runtime), label],
                    cwd=self.spec['private_root'], env=self.spec['environment'],
                    stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
            self.processes[label] = process  # Retain handle before fallible I/O.
            save(self.runtime / (label + '-spawn.json'), dict(label=label, pid=process.pid), deadline)
        receipt_path = self.runtime / (label + '-exec.json')
        receipt_limit = min(deadline, time.monotonic() + 2)
        while not receipt_path.exists():
            self.collect(receipt_limit)
            if process.poll() is not None:
                raise RuntimeError('helper exited before same-PID receipt: ' + label)
            if not cleanup:
                self.tick(receipt_limit)
            else:
                pause(receipt_limit)
        receipt = load(receipt_path, receipt_limit)
        if (receipt['pid'] != process.pid or receipt['ppid'] != os.getpid() or
                receipt['uid'] != os.getuid() or receipt['nice'] != 15):
            raise RuntimeError('exec receipt identity/priority mismatch')
        # Recovery has its own observer if the main sampler is failed. A wrapper
        # is safe to release only after a complete current ownership observation.
        observer = Inventory(self.spec, self.runtime, 'cleanup-launch') if cleanup else self.inventory
        observer.observe(receipt_limit, validate=True)
        wrapper = [r for r in observer.records.values() if key(r) == key(receipt) and r['stage'] == 'wrapper']
        if not wrapper:
            raise RuntimeError('wrapper identity not verified before exec gate')
        save(self.runtime / (label + '-registered.json'), wrapper[0], receipt_limit)
        return process

    def cli(self, label, deadline):
        limit = min(deadline, time.monotonic() + 5)
        process = self.launch(label, limit)
        while process.poll() is None:
            self.tick(limit)  # Observation remains active for the whole CLI wait.
        self.collect(limit)
        output = read(self.runtime / (label + '.log'), limit).decode()
        save(self.runtime / (label + '-result.json'), dict(code=process.returncode, stdout=output), limit)
        if process.returncode != 0:
            raise RuntimeError('CLI failed: ' + label)
        return output

    def port(self, port, label, retain, deadline):
        # Only the two declared literal ports. First bind only, no SO_REUSE*.
        raw = command(['/usr/sbin/netstat', '-an', '-p', 'tcp'], deadline).decode()
        scoped = [line for line in raw.splitlines() if re.search(r'\.' + str(port) + r'\s', line)]
        state = dict(filter_port=port, scoped_rows=scoped, monotonic=time.monotonic())
        handle = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        try:
            with io_until(deadline):
                handle.settimeout(remaining(deadline))
                handle.bind(('127.0.0.1', port))
            save(self.runtime / f'port-{port}-{label}.json',
                 dict(success=True, state_before_first_bind=state, no_reuse=True, no_retry=True,
                      held_until_launch=retain, bound=handle.getsockname()), deadline)
            if retain:
                self.reservations[port] = handle
                handle = None
        except Exception:
            first(self.runtime / f'port-{port}-{label}.json',
                  dict(success=False, state_before_first_bind=state, error=traceback.format_exc(),
                       no_reuse=True, no_retry=True), deadline)
            raise
        finally:
            if handle is not None:
                handle.close()

    def flow(self):
        with io_until(self.spec['flow_deadline']):
            return self._flow()

    def _flow(self):
        deadline = self.spec['flow_deadline']
        for port in PORTS:
            self.port(port, 'pre', True, min(deadline, time.monotonic() + 2))
        self.reservations.pop(PORTS[0]).close()
        peer = self.launch('peer', deadline)
        ready = min(deadline, time.monotonic() + 15)
        root = Path(self.spec['private_root'])
        while True:
            if peer.poll() is not None:
                raise RuntimeError('peer exited before readiness')
            text = read(self.runtime / 'peer.log', ready).decode()
            if (root / 'peer/control.sock').exists() and f'listening on 127.0.0.1:{PORTS[0]}' in text:
                break
            self.tick(ready)
        status = json.loads(self.cli('peer-status', deadline))
        if status['daemon']['pid'] != peer.pid or status['daemon']['platform'] != 'virtual' or status['state'] != 'listening':
            raise RuntimeError('peer status/identity mismatch')
        save(self.runtime / 'peer-ready.json', dict(status=status, before_app_launch=True), deadline)
        self.reservations.pop(PORTS[1]).close()
        app = self.launch('app', deadline)
        while app.poll() is None:
            if peer.poll() is not None:
                raise RuntimeError('peer exited during app flow')
            self.tick(deadline)
        if peer.poll() is not None:
            raise RuntimeError('peer exited before app completion was collected')
        self.collect(deadline)
        self.observe(deadline)
        if self.observer_failed:
            raise FlowAbort('final observation failed')
        report = load(root / 'app/report.json', deadline)
        save(self.runtime / 'report.json', report, deadline)
        save(self.runtime / 'app-result.json', dict(exit=self.exits['app'], checks=len(report),
             passed=sum(row['passed'] for row in report), failed=[row for row in report if not row['passed']],
             declared_abort=(self.runtime / 'abort.json').exists()), deadline)
        daemons = {key(r) for r in self.inventory.records.values() if r['role'].endswith('supervised-daemon')}
        if app.returncode != 0 or len(report) != 25 or not all(row['passed'] for row in report):
            raise RuntimeError('ordinary 25-check report/OS exit failed')
        if len(daemons) != 2:
            raise RuntimeError('initial and routine-recovered daemon identities not both observed')
        # Require actual stop coverage rather than inventing a callback or probe
        # that changes the product routine. Missing a brief state fails evidence.
        observations = read(self.runtime / 'main-observations.jsonl', deadline, EVIDENCE_CAP).splitlines()
        if not any(row['status'] == 4 and key(row) in daemons
                   for line in observations for row in json.loads(line)['rows']):
            raise RuntimeError('routine SIGSTOP interval not observed')
        self.request_cleanup(True, deadline)

    def cleanup(self):
        """Collector stays alive; the separate watchdog owns cleanup observation/signals."""
        if self.cleanup_deadline is None:
            try:
                self.request_cleanup(False, time.monotonic() + 1)
            except Exception:
                # The independent watchdog still has the immutable session's
                # 120+60 deadline, even if a request could not be published.
                self.failed = True
                self.cleanup_deadline = self.spec['flow_deadline'] + CLEANUP_SECONDS
        peer_stop_attempted = False
        while time.monotonic() < self.cleanup_deadline:
            limit = min(self.cleanup_deadline, time.monotonic() + 1)
            try:
                self.collect(limit)
                done = self.runtime / 'recovery-done.json'
                if exists(done, limit):
                    return load(done, limit)
                if self.watchdog.poll() is not None and self.fallback is None:
                    self.failed = True
                    first(self.runtime / 'first-watchdog-loss.json', dict(code=self.watchdog.returncode), limit)
                    request = load(self.runtime / 'cleanup-request.json', limit)
                    request['normal'] = False
                    self.fallback = Recovery(self.spec, self.runtime, request, 'fallback')
                if self.fallback is not None:
                    self.fallback.step()
                if exists(self.runtime / 'peer-stop-permitted.json', limit) and not peer_stop_attempted:
                    peer_stop_attempted = True  # An exception can never cause a retry.
                    peer = self.processes.get('peer')
                    if peer is not None and peer.poll() is None:
                        self.launch('peer-stop', min(self.cleanup_deadline, time.monotonic() + 5), cleanup=True)
                # No normal sample() call here. The recovery process remains
                # active during this and every public-stop CLI wait.
            except Exception:
                self.failed = True
                try:
                    first(self.runtime / 'first-cleanup-failure.json',
                          dict(error=traceback.format_exc()), time.monotonic() + 0.5)
                except Exception:
                    # Keep polling/reaping children even if the sink is failed.
                    pass
            try:
                pause(self.cleanup_deadline)
            except DeadlineError:
                break
        if self.fallback is not None:
            self.fallback.finish()
        first(self.runtime / 'failed-bounded-recovery.json',
              dict(monotonic=time.monotonic(), direct_exits=self.exits,
                   reason='60-second cleanup budget ended; no cleanup PASS'), time.monotonic() + 1)
        return dict(passed=False, recovered=False)

    def run(self):
        preflight = time.monotonic() + 120
        with io_until(preflight):
            self.runtime.mkdir(mode=0o700)  # Exclusive flow guard, never removed.
            checked = load(HERE / 'source-checks.json', preflight)
            for name, record in checked['python_files'].items():
                if sha(HERE / name, preflight) != record['sha256']:
                    raise RuntimeError('reviewed controller source changed: ' + name)
            bindings = bind_sources(preflight)
            if bindings != self.prepared['source_bindings']:
                raise RuntimeError('prepared source bindings changed')
            bundle = Path(self.prepared['bundle'])
            if tree(bundle, preflight) != self.prepared['bundle_tree']:
                raise RuntimeError('prepared full signed tree changed')
            root = Path(self.prepared['private_root'])
            if (root / 'app/config').exists() or any((root / part).is_symlink() for part in ('app', 'peer', 'home', 'tmp', 'cache')):
                raise RuntimeError('fixture no longer fresh')
            for part in ('', 'app', 'peer', 'home', 'tmp', 'cache'):
                info = (root / part).stat()
                if info.st_uid != os.getuid() or info.st_mode & 0o077:
                    raise RuntimeError('fixture ownership/mode changed')
            config = (f'version=1\nname="e2e-peer"\nrole="server"\n[server]\nlisten="127.0.0.1:{PORTS[0]}"\nport={PORTS[0]}\n[tls]\nenabled=false\n'
                      '[[screen]]\nname="e2e-peer"\nrect={x=0,y=0,w=1920,h=1080}\n[[screen]]\nname="app"\nrect={x=1920,y=0,w=1920,h=1080}\n')
            with (root / 'peer/config.toml').open('x') as stream:
                stream.write(config)
            common_env = dict(self.prepared['environment'], OPENERGY_DISABLE_DISCOVERY='1',
                              OPENERGY_ALLOW_PORT_24800='0', OPENERGY_DAEMON_ARGS='--platform virtual')
            def environment(directory):
                return dict(common_env, OPENERGY_CONFIG=str(directory / 'config.toml'),
                            OPENERGY_CONFIG_DIR=str(directory), OPENERGY_TLS_DIR=str(directory / 'tls'),
                            OPENERGY_SOCKET=str(directory / 'control.sock'))
            start = time.monotonic()
            lib = api()
            owner = detail(lib, bsd(lib, os.getpid()))
            if owner['uid'] != os.getuid() or owner['ruid'] != os.getuid() or owner['nice'] != 15:
                raise RuntimeError('controller UID/absolute priority mismatch')
            self.spec = dict(self.prepared, owner=owner, python_executable=owner['executable'], flow_start=start,
                             flow_deadline=start + FLOW_SECONDS,
                             peer_environment=environment(root / 'peer'),
                             app_environment=dict(environment(root / 'app/config'), OPENERGY_E2E_NO_RENDER='1'))
            save(self.runtime / 'session.json', self.spec, preflight)
            self.inventory = Inventory(self.spec, self.runtime, 'main')
        ready_deadline = min(self.spec['flow_deadline'], time.monotonic() + 2)
        with io_until(ready_deadline):
            with (self.runtime / 'watchdog.log').open('xb') as log:
                self.watchdog = subprocess.Popen([PYTHON, str(HERE / 'watchdog.py'), str(self.runtime)],
                    cwd=root, env=self.spec['environment'], stdin=subprocess.DEVNULL,
                    stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
            while not (self.runtime / 'watchdog-ready.json').exists():
                if self.watchdog.poll() is not None:
                    raise RuntimeError('watchdog did not arm; no products launched')
                pause(ready_deadline)
            guard = load(self.runtime / 'watchdog-ready.json', ready_deadline)['identity']
            if (guard['pid'] != self.watchdog.pid or guard['ppid'] != owner['pid'] or
                    guard['uid'] != owner['uid'] or guard['ruid'] != owner['uid'] or guard['nice'] != 15 or
                    guard['executable'] != owner['executable'] or
                    guard['argv'][1:] != [str(HERE / 'watchdog.py'), str(self.runtime)]):
                raise RuntimeError('watchdog actual identity/NI15 mismatch; no products launched')
        try:
            self.flow()
        except BaseException:
            self.abort(traceback.format_exc(), time.monotonic() + 1)
        finally:
            try:
                result = self.cleanup()
            finally:
                for handle in self.reservations.values():
                    handle.close()
                self.reservations.clear()
                self.collect(time.monotonic() + 1)
        # Do not cancel or signal the watchdog. It exits only after proven
        # absence + reaping, or an explicit failed bounded-recovery record.
        finish_deadline = min(self.spec['flow_deadline'] + CLEANUP_SECONDS + 3, time.monotonic() + 3)
        while self.watchdog.poll() is None:
            self.collect(finish_deadline)
            pause(finish_deadline)
        save(self.runtime / 'watchdog-exit.json', dict(pid=self.watchdog.pid, code=self.watchdog.returncode), finish_deadline)
        guard_after = json.loads(command([PYTHON, HERE / 'probe.py', str(finish_deadline), root,
                                         key(guard)], finish_deadline))
        guard_absent = (guard_after['absence'].get(str(self.watchdog.pid)) == 'ESRCH' and
                        not any(row['pid'] == self.watchdog.pid or row['pgid'] == guard['pgid']
                                for row in guard_after['rows']))
        save(self.runtime / 'watchdog-absence.json', dict(identity=guard, absent=guard_absent,
             remaining_group_rows=[r for r in guard_after['rows'] if r['pgid'] == guard['pgid']]), finish_deadline)
        if not guard_absent:
            self.failed = True
            result['recovered'] = False
        after = time.monotonic() + 20  # Evidence-only archival; all products must be absent first.
        if not result['recovered']:
            raise RuntimeError('failed bounded recovery; retain guard/watchdog evidence')
        for port in PORTS:
            self.port(port, 'post', False, min(after, time.monotonic() + 2))
        report_path = root / 'app/report.json'
        if exists(report_path, after) and not exists(self.runtime / 'report.json', after):
            with io_until(after):
                with (self.runtime / 'report.raw.json').open('xb') as stream:
                    stream.write(read(report_path, after))
        for name in ('config.toml', 'daemon-stderr.log', 'openergy.log'):
            path = root / 'app/config' / name
            if exists(path, after):
                with io_until(after):
                    with (self.runtime / ('app-' + name)).open('xb') as stream:
                        stream.write(read(path, after))
        unchanged = tree(bundle, after) == self.prepared['bundle_tree']
        save(self.runtime / 'bundle-after.json', dict(complete_signed_tree_unchanged=unchanged,
             app_sha256=sha(bundle / 'Contents/MacOS/Openergy', after),
             daemon_sha256=sha(bundle / 'Contents/Helpers/openergy', after)), after)
        save(self.runtime / 'outcome.json', dict(passed=not self.failed and result['passed'] and unchanged,
             direct_exits=self.exits, recovery=result, product_flows=1, retries=0,
             source_runtime_proof='only this later invocation, never the source-only repair'), after)
        if self.failed or not result['passed'] or not unchanged:
            raise RuntimeError('ordinary evidence flow FAILED; no retry')


def main():
    nice15()
    controller = Controller(load(HERE / 'prepared.json', time.monotonic() + 2))
    controller.run()


if __name__ == '__main__':
    main()
