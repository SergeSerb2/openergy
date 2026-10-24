"""Same-PID NI15 exec with a durable parent gate. No products run on import."""
from common import *
from macproc import api, bsd
from ownership import expected
import resource
import sys


def main():
    nice15()
    resource.setrlimit(resource.RLIMIT_FSIZE, (PRODUCT_FILE_CAP, PRODUCT_FILE_CAP))
    runtime, label = Path(sys.argv[1]), sys.argv[2]
    spec = load(runtime / 'session.json', time.monotonic() + 2)
    launch = load(runtime / (label + '-command.json'), time.monotonic() + 1)
    deadline = min(launch['absolute_deadline'], time.monotonic() + 2)
    argv = expected(spec)[label]
    environment = spec['app_environment'] if label == 'app' else spec['peer_environment']
    with io_until(deadline):
        row = bsd(api(), os.getpid())
        if row is None or row['ppid'] != spec['owner']['pid'] or row['uid'] != spec['owner']['uid']:
            raise RuntimeError('exec helper ownership mismatch')
        digest = sha(argv[0], deadline)
        required = spec['app_sha256'] if label == 'app' else spec['daemon_sha256']
        if digest != required:
            raise RuntimeError('exec artifact changed')
        save(runtime / (label + '-exec.json'), dict(row, argv=argv, binary_sha256=digest,
              environment_allowlist=environment, monotonic=time.monotonic()), deadline)
        gate = runtime / (label + '-registered.json')
        while not gate.exists():
            pause(deadline)
        registered = load(gate, deadline)
        if any(registered[k] != row[k] for k in ('pid', 'start_sec', 'start_usec')):
            raise RuntimeError('wrong exec registration')
        # Bounds every product stdout/stderr file, including stalled CLI output.
        resource.setrlimit(resource.RLIMIT_FSIZE, (PRODUCT_FILE_CAP, PRODUCT_FILE_CAP))
        remaining(deadline)
        # ITIMER_REAL survives exec, while caught handlers reset. Do not pass
        # this helper's I/O alarm into the unchanged ordinary product. The
        # separate watchdog process retains its independent flow deadline.
        signal.setitimer(signal.ITIMER_REAL, 0)
        signal.signal(signal.SIGALRM, signal.SIG_DFL)
        os.execve(argv[0], argv, environment)


if __name__ == '__main__':
    main()
