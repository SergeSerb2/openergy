# Complete changed function

SOURCE/DATA ONLY. execution=false. Only `collect` changes. No function is added or removed.
The other32 functions and every byte outside the three bounded collector edits stay exact.

```python
def collect(label, argv, root, phase_deadline):
    """Direct child only. Bounded nonblocking reads; no communicate or PID inference."""
    started = clock_ns()
    deadline = min(started + 10 * NS, phase_deadline - NS)  # Leave publication time inside phase.
    require(deadline - started > 750000000, label + ': no command/reap allowance')
    work_end = deadline - 500000000  # Kill/reap share the command's original ceiling.
    buffers = {'stdout': bytearray(), 'stderr': bytearray()}
    eof = {'stdout': False, 'stderr': False}
    observed, dropped, drop_unknown = 0, 0, False
    overflow, timed_out, deadline_loss = False, False, False
    proc, error, kill_error, reaped = None, None, None, False
    selector = selectors.DefaultSelector()
    kill_attempted, kill_delivered = False, False
    kill_eligibility_exit, kill_eligibility_observed_ns = None, None
    try:
        with until(work_end):
            proc = subprocess.Popen(argv, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                                    stderr=subprocess.PIPE, cwd=root, env=native_environment(root),
                                    shell=False, close_fds=True)
            HANDLES.append(dict(label=label, argv=list(argv), handle=proc, collector_status=None))
            for name in ('stdout', 'stderr'):
                stream = getattr(proc, name)
                os.set_blocking(stream.fileno(), False)
                selector.register(stream, selectors.EVENT_READ, name)
            while True:
                status = proc.poll()  # Actual waitpid status from the retained handle.
                if status is not None and all(eof.values()):
                    reaped = True
                    break
                if clock_ns() >= work_end:
                    timed_out = deadline_loss = True
                    break
                if not selector.get_map():
                    # EOF without exit: block only within the remaining work interval.
                    try:
                        proc.wait(timeout=remaining(work_end))
                    except subprocess.TimeoutExpired:
                        timed_out = deadline_loss = True
                    continue
                for key, _mask in selector.select(min(0.05, remaining(work_end))):
                    # Read at most remaining capacity + one overflow byte. Never capture then cap.
                    allowance = STREAM_CAP - sum(len(x) for x in buffers.values())
                    try:
                        chunk = os.read(key.fd, min(65536, allowance + 1))
                    except BlockingIOError:
                        continue
                    if not chunk:
                        eof[key.data] = True
                        selector.unregister(key.fileobj)
                        continue
                    observed += len(chunk)
                    buffers[key.data].extend(chunk[:allowance])
                    if len(chunk) > allowance:
                        dropped += len(chunk) - allowance
                        overflow, drop_unknown = True, True
                        break
                if overflow:
                    break
    except Exception as exc:
        error = type(exc).__name__ + ': ' + str(exc)[:1024]
        if isinstance(exc, (TimeoutError, subprocess.TimeoutExpired)):
            timed_out = deadline_loss = True
    finally:
        if proc is not None:
            try:
                with until(deadline):
                    kill_eligibility_exit = proc.poll()
                    try:
                        kill_eligibility_observed_ns = clock_ns()
                    finally:
                        # A real poll exit remains reaped even if the clock raises.
                        if kill_eligibility_exit is not None:
                            reaped = True
                    if kill_eligibility_exit is None:
                        if kill_eligibility_observed_ns < deadline and kill_eligibility_observed_ns < phase_deadline:
                            kill_attempted = True
                            proc.kill()  # Only this retained, unreaped direct child.
                            kill_delivered = True
                        else:
                            timed_out = deadline_loss = True
                    proc.wait(timeout=remaining(deadline))
                    reaped = True
            except Exception as exc:
                kill_error = type(exc).__name__ + ': ' + str(exc)[:1024]
                deadline_loss = True
            # No drain pass after failure; unread bytes remain explicitly unknown.
            if not all(eof.values()):
                drop_unknown = True
            if clock_ns() > deadline:
                deadline_loss = True
            for name in ('stdout', 'stderr'):
                stream = getattr(proc, name)
                if stream is not None:
                    try:
                        stream.close()
                    except Exception as exc:
                        if error is None:
                            error = type(exc).__name__ + ': ' + str(exc)[:1024]
        try:
            selector.close()
        except Exception as exc:
            if error is None:
                error = type(exc).__name__ + ': ' + str(exc)[:1024]
    finished = clock_ns()
    if finished > deadline:
        deadline_loss = True
    status = proc.returncode if proc is not None else None
    record = dict(label=label, argv=argv, pid=proc.pid if proc is not None else None,
                  handle_retained=proc is not None, actual_exit=status, reaped=reaped,
                  stdout_eof=eof['stdout'], stderr_eof=eof['stderr'],
                  observed_stream_bytes=observed, dropped_observed_bytes=dropped,
                  unread_or_dropped_unknown=drop_unknown, overflow=overflow,
                  timed_out=timed_out, deadline_loss=deadline_loss,
                  kill_attempted=kill_attempted, kill_delivered=kill_delivered,
                  kill_eligibility_exit=kill_eligibility_exit,
                  kill_eligibility_observed_ns=kill_eligibility_observed_ns,
                  error=error, kill_or_reap_error=kill_error, clock=CLOCK, started_ns=started,
                  command_deadline_ns=deadline, finished_ns=finished)
    if proc is not None:
        HANDLES[-1]['collector_status'] = record
    return record, bytes(buffers['stdout']), bytes(buffers['stderr'])
```
