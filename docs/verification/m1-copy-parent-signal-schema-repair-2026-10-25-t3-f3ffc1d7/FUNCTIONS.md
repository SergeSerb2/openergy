# Full changed declaration before and after

Exact physical source slices, including final LF. No supplied source ran.

## collect before

Lines 725..871; SHA256 e1a6ec1b79b46bc03455d6ec8bf665029d7b0effbf6d3d3d682ea7ed8b188104.

```python
def collect(label, parent_bytes=None):
    require(label in ('entry', 'copy'), 'exact helper/COPY child kinds')
    begun = clock_ns()
    end = min(begun + 10 * NS, COPY_END - 2 * NS) if label == 'entry' else COPY_END
    work_end = end - 500000000 if label == 'entry' else COPY_END - 2 * NS
    reap_end = end if label == 'entry' else COPY_END - 1500000000
    argv = ([SYSTEM, '-I', '-S', '-B', ROOT + '/file-qualification.recipe.py.txt', ROOT, 'entry']
        if label == 'entry' else [PRIVATE_PYTHON, '-B', ROOT + '/metadata-caller.py.txt',
                                 'copy', ROOT, str(COPY_START), sha(parent_bytes)])
    require(label == 'entry' or (type(parent_bytes) is bytes and len(parent_bytes) <= RAW_CAP),
            'bounded canonical COPY stdin')
    buffers = {'stdout': bytearray(), 'stderr': bytearray()}
    status = dict(schema='openergy.copy-parent-child.v1', label=label, clock=CLOCK,
        argv=argv, cwd=ROOT, environment=startup(), phase_started_ns=COPY_START,
        phase_deadline_ns=COPY_END, child_started_ns=begun, child_deadline_ns=end,
        collection_end_ns=work_end, reap_end_ns=reap_end, launch_started_ns=None,
        launch_returned_ns=None, pid_from_retained_handle=None, handle_retained=False,
        actual_exit=None, actual_exit_observed_ns=None, reaped=False,
        eof={'stdout': False, 'stderr': False}, observed_bytes=0, dropped_known_bytes=0,
        unread_or_dropped_unknown=False, overflow=False, timed_out=False, deadline_loss=False,
        kill_attempted=False, kill_delivered=False, kill_or_reap_error=None,
        kill_eligibility_exit=None, kill_eligibility_observed_ns=None,
        first_error=None, close_errors=[], finished_ns=None, stdin_bytes_sent=0,
        stdin_complete=(label == 'entry'))
    proc, selector = None, None
    retained = dict(label=label, handle=None, status=status, buffers=buffers)
    HANDLES.append(retained)
    try:
        with until(work_end):
            status['launch_started_ns'] = clock_ns()
            require(status['launch_started_ns'] + 750000000 < work_end,
                    'fresh actual launch sample fits remaining child work/reap allowance')
            proc = subprocess.Popen(argv, stdin=(subprocess.DEVNULL if label == 'entry' else subprocess.PIPE),
                stdout=subprocess.PIPE, stderr=subprocess.PIPE, cwd=ROOT,
                env=startup(), shell=False, close_fds=True, bufsize=0)
            retained['handle'] = proc
            status['handle_retained'] = True
            status['pid_from_retained_handle'] = proc.pid
            status['launch_returned_ns'] = clock_ns()
            require(status['launch_returned_ns'] < work_end, 'late spawn return')
            selector = selectors.DefaultSelector()
            for name in ('stdout', 'stderr'):
                stream = getattr(proc, name)
                os.set_blocking(stream.fileno(), False)
                selector.register(stream, selectors.EVENT_READ, name)
            if label == 'copy':
                os.set_blocking(proc.stdin.fileno(), False)
                selector.register(proc.stdin, selectors.EVENT_WRITE, 'stdin')
            while True:
                code = proc.poll()
                observe_exit(proc, status, code)
                if code is not None and all(status['eof'].values()):
                    break
                require(clock_ns() < work_end, 'original child work ceiling')
                for key, _ in selector.select(min(0.05, remaining(work_end))):
                    if key.data == 'stdin':
                        offset = status['stdin_bytes_sent']
                        try:
                            count = os.write(key.fd, parent_bytes[offset:offset + 65536])
                        except BlockingIOError:
                            continue
                        require(count > 0, 'COPY stdin stalled write')
                        status['stdin_bytes_sent'] += count
                        if status['stdin_bytes_sent'] == len(parent_bytes):
                            selector.unregister(key.fileobj)
                            proc.stdin.close()  # Explicit EOF, not a second launch.
                            status['stdin_complete'] = True
                        continue
                    allowance = RAW_CAP - sum(len(v) for v in buffers.values())
                    try:
                        chunk = os.read(key.fd, min(65536, allowance + 1))
                    except BlockingIOError:
                        continue
                    if not chunk:
                        status['eof'][key.data] = True
                        selector.unregister(key.fileobj)
                        continue
                    status['observed_bytes'] += len(chunk)
                    buffers[key.data].extend(chunk[:allowance])
                    if len(chunk) > allowance:
                        status['overflow'] = True
                        status['dropped_known_bytes'] += len(chunk) - allowance
                        status['unread_or_dropped_unknown'] = True
                        raise Stop('combined1MiB plus one detection byte; no drain')
    except BaseException as exc:
        latch(status, 'collect', exc)
        fail(label + '-collection', exc)  # FIRST actual failure precedes ALL writers.
        if isinstance(exc, (TimeoutError, subprocess.TimeoutExpired)):
            status['timed_out'] = True
        try:
            if clock_ns() >= work_end:
                status['timed_out'] = True
        except BaseException:
            status['deadline_loss'] = True
    finally:
        if proc is not None and not status['reaped']:
            try:
                code = proc.poll()
                observed_ns = observe_exit(proc, status, code)  # Immediate clock, even None.
                status['kill_eligibility_exit'] = code
                status['kill_eligibility_observed_ns'] = observed_ns
                if code is None:
                    if not (observed_ns < reap_end and observed_ns < COPY_END):
                        status['timed_out'] = True
                        status['unread_or_dropped_unknown'] = True
                        raise TimeoutError('late None eligibility; no KILL or new wait')
                    if status['overflow'] or status['timed_out']:
                        status['kill_attempted'] = True
                        proc.kill()  # Exact retained LIVE handle; no OS call since sample.
                        status['kill_delivered'] = True
                    with until(reap_end):
                        code = proc.wait(timeout=remaining(reap_end))
                        observe_exit(proc, status, code)
            except BaseException as exc:
                status['kill_or_reap_error'] = type(exc).__name__
                latch(status, 'kill-or-reap', exc)
                fail(label + '-kill-or-reap', exc)
        if proc is not None:
            for name in ('stdin', 'stdout', 'stderr'):
                stream = getattr(proc, name)
                if stream is not None and not stream.closed:
                    try:
                        stream.close()
                    except BaseException as exc:
                        status['close_errors'].append(dict(stream=name, error=type(exc).__name__))
                        latch(status, 'pipe-close', exc)
                        fail(label + '-pipe-close', exc)
        if selector is not None:
            try:
                selector.close()
            except BaseException as exc:
                status['close_errors'].append(dict(stream='selector', error=type(exc).__name__))
                latch(status, 'selector-close', exc)
                fail(label + '-selector-close', exc)
    status['unread_or_dropped_unknown'] |= not all(status['eof'].values())
    status['retained_bytes'] = {name: len(data) for name, data in buffers.items()}
    try:
        status['finished_ns'] = clock_ns()
        status['deadline_loss'] |= status['finished_ns'] >= end
    except BaseException as exc:
        status['deadline_loss'] = True
        latch(status, 'finish-clock', exc)
        fail(label + '-finish-clock', exc)
    if not collection_complete(status) or not status['stdin_complete']:
        latch(status, 'incomplete-actual-collection', Stop('incomplete'))
        fail(label + '-actual-status', Stop('incomplete'))
    return status, bytes(buffers['stdout']), bytes(buffers['stderr'])
```

## collect after

Lines 725..872; SHA256 524602c4f3a2b47e4fae0043af8fa0955ce9d04061391a77fd47bb3c16461c7c.

```python
def collect(label, parent_bytes=None):
    require(label in ('entry', 'copy'), 'exact helper/COPY child kinds')
    begun = clock_ns()
    end = min(begun + 10 * NS, COPY_END - 2 * NS) if label == 'entry' else COPY_END
    work_end = end - 500000000 if label == 'entry' else COPY_END - 2 * NS
    reap_end = end if label == 'entry' else COPY_END - 1500000000
    argv = ([SYSTEM, '-I', '-S', '-B', ROOT + '/file-qualification.recipe.py.txt', ROOT, 'entry']
        if label == 'entry' else [PRIVATE_PYTHON, '-B', ROOT + '/metadata-caller.py.txt',
                                 'copy', ROOT, str(COPY_START), sha(parent_bytes)])
    require(label == 'entry' or (type(parent_bytes) is bytes and len(parent_bytes) <= RAW_CAP),
            'bounded canonical COPY stdin')
    buffers = {'stdout': bytearray(), 'stderr': bytearray()}
    status = dict(schema='openergy.copy-parent-child.v1', label=label, clock=CLOCK,
        argv=argv, cwd=ROOT, environment=startup(), phase_started_ns=COPY_START,
        phase_deadline_ns=COPY_END, child_started_ns=begun, child_deadline_ns=end,
        collection_end_ns=work_end, reap_end_ns=reap_end, launch_started_ns=None,
        launch_returned_ns=None, pid_from_retained_handle=None, handle_retained=False,
        actual_exit=None, actual_exit_observed_ns=None, reaped=False,
        eof={'stdout': False, 'stderr': False}, observed_bytes=0, dropped_known_bytes=0,
        unread_or_dropped_unknown=False, overflow=False, timed_out=False, deadline_loss=False,
        kill_attempted=False, kill_syscall_returned=False, kill_or_reap_error=None,
        kill_eligibility_exit=None, kill_eligibility_observed_ns=None,
        first_error=None, close_errors=[], finished_ns=None, stdin_bytes_sent=0,
        stdin_complete=(label == 'entry'))
    proc, selector = None, None
    retained = dict(label=label, handle=None, status=status, buffers=buffers)
    HANDLES.append(retained)
    try:
        with until(work_end):
            status['launch_started_ns'] = clock_ns()
            require(status['launch_started_ns'] + 750000000 < work_end,
                    'fresh actual launch sample fits remaining child work/reap allowance')
            proc = subprocess.Popen(argv, stdin=(subprocess.DEVNULL if label == 'entry' else subprocess.PIPE),
                stdout=subprocess.PIPE, stderr=subprocess.PIPE, cwd=ROOT,
                env=startup(), shell=False, close_fds=True, bufsize=0)
            retained['handle'] = proc
            status['handle_retained'] = True
            status['pid_from_retained_handle'] = proc.pid
            status['launch_returned_ns'] = clock_ns()
            require(status['launch_returned_ns'] < work_end, 'late spawn return')
            selector = selectors.DefaultSelector()
            for name in ('stdout', 'stderr'):
                stream = getattr(proc, name)
                os.set_blocking(stream.fileno(), False)
                selector.register(stream, selectors.EVENT_READ, name)
            if label == 'copy':
                os.set_blocking(proc.stdin.fileno(), False)
                selector.register(proc.stdin, selectors.EVENT_WRITE, 'stdin')
            while True:
                code = proc.poll()
                observe_exit(proc, status, code)
                if code is not None and all(status['eof'].values()):
                    break
                require(clock_ns() < work_end, 'original child work ceiling')
                for key, _ in selector.select(min(0.05, remaining(work_end))):
                    if key.data == 'stdin':
                        offset = status['stdin_bytes_sent']
                        try:
                            count = os.write(key.fd, parent_bytes[offset:offset + 65536])
                        except BlockingIOError:
                            continue
                        require(count > 0, 'COPY stdin stalled write')
                        status['stdin_bytes_sent'] += count
                        if status['stdin_bytes_sent'] == len(parent_bytes):
                            selector.unregister(key.fileobj)
                            proc.stdin.close()  # Explicit EOF, not a second launch.
                            status['stdin_complete'] = True
                        continue
                    allowance = RAW_CAP - sum(len(v) for v in buffers.values())
                    try:
                        chunk = os.read(key.fd, min(65536, allowance + 1))
                    except BlockingIOError:
                        continue
                    if not chunk:
                        status['eof'][key.data] = True
                        selector.unregister(key.fileobj)
                        continue
                    status['observed_bytes'] += len(chunk)
                    buffers[key.data].extend(chunk[:allowance])
                    if len(chunk) > allowance:
                        status['overflow'] = True
                        status['dropped_known_bytes'] += len(chunk) - allowance
                        status['unread_or_dropped_unknown'] = True
                        raise Stop('combined1MiB plus one detection byte; no drain')
    except BaseException as exc:
        latch(status, 'collect', exc)
        fail(label + '-collection', exc)  # FIRST actual failure precedes ALL writers.
        if isinstance(exc, (TimeoutError, subprocess.TimeoutExpired)):
            status['timed_out'] = True
        try:
            if clock_ns() >= work_end:
                status['timed_out'] = True
        except BaseException:
            status['deadline_loss'] = True
    finally:
        if proc is not None and not status['reaped']:
            try:
                code = proc.poll()
                observed_ns = observe_exit(proc, status, code)  # Immediate clock, even None.
                status['kill_eligibility_exit'] = code
                status['kill_eligibility_observed_ns'] = observed_ns
                if code is None:
                    if not (observed_ns < reap_end and observed_ns < COPY_END):
                        status['timed_out'] = True
                        status['unread_or_dropped_unknown'] = True
                        raise TimeoutError('late None eligibility; no KILL or new wait')
                    if status['overflow'] or status['timed_out']:
                        status['kill_attempted'] = True
                        # Only this retained, still-unreaped child; no new poll/OS query.
                        os.kill(proc.pid, signal.SIGKILL)
                        status['kill_syscall_returned'] = True  # Return is not delivery or exit.
                    with until(reap_end):
                        code = proc.wait(timeout=remaining(reap_end))
                        observe_exit(proc, status, code)
            except BaseException as exc:
                status['kill_or_reap_error'] = type(exc).__name__
                latch(status, 'kill-or-reap', exc)
                fail(label + '-kill-or-reap', exc)
        if proc is not None:
            for name in ('stdin', 'stdout', 'stderr'):
                stream = getattr(proc, name)
                if stream is not None and not stream.closed:
                    try:
                        stream.close()
                    except BaseException as exc:
                        status['close_errors'].append(dict(stream=name, error=type(exc).__name__))
                        latch(status, 'pipe-close', exc)
                        fail(label + '-pipe-close', exc)
        if selector is not None:
            try:
                selector.close()
            except BaseException as exc:
                status['close_errors'].append(dict(stream='selector', error=type(exc).__name__))
                latch(status, 'selector-close', exc)
                fail(label + '-selector-close', exc)
    status['unread_or_dropped_unknown'] |= not all(status['eof'].values())
    status['retained_bytes'] = {name: len(data) for name, data in buffers.items()}
    try:
        status['finished_ns'] = clock_ns()
        status['deadline_loss'] |= status['finished_ns'] >= end
    except BaseException as exc:
        status['deadline_loss'] = True
        latch(status, 'finish-clock', exc)
        fail(label + '-finish-clock', exc)
    if not collection_complete(status) or not status['stdin_complete']:
        latch(status, 'incomplete-actual-collection', Stop('incomplete'))
        fail(label + '-actual-status', Stop('incomplete'))
    return status, bytes(buffers['stdout']), bytes(buffers['stderr'])
```
