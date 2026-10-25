# Complete function inventory and full before/after source

Status IMPLEMENTED_UNREVIEWED. Physical line hashes include decorators and final LF.
The baseline is the full cleared f78f2be SOURCE launcher. Only the new COPY
folder changes. No baseline/caller/common/helper/prepare/controller file changes.

Every current function/class, including nested declarations:

- `Stop`
- `require`
- `clock_ns`
- `remaining`
- `sha`
- `encoded`
- `hex_string`
- `same_data`
- `stamp`
- `open_root`
- `read_leaf`
- `startup`
- `latch`
- `observe_exit`
- `collection_complete`
- `until`
- `until.expired`
- `closed_json`
- `closed_json.pairs`
- `closed_json.nonfinite`
- `closed_json.finite_float`
- `absent`
- `read_new_leaf`
- `reference`
- `canonical_prepared_directory`
- `existing_ledger`
- `common_deadline`
- `inspect_artifact`
- `fail`
- `raw_binding`
- `bind_config`
- `console_admission`
- `inspect_fixed_inputs`
- `inspect_saved_source`
- `inspect_published_artifact`
- `inspect_new_prepared`
- `system_bootstrap`
- `import_system_common`
- `reject_entry_caches`
- `reserved_put`
- `collect`
- `inspect_entry`
- `copy_parent_stdin`
- `copy_record_complete`
- `read_record_ref`
- `normalize`
- `inspect_copy_result`
- `emit_once`
- `main`

All changed/added/removed source ranges follow. Unchanged ranges are identified
by exact before/after hashes in function-inventory.json. The complete exact
module delta, including constants/imports/admission/tail, is in the patch.

## Stop — UNCHANGED

before: lines 125..126, 35 bytes, SHA256 `fd5f0e9e870ae08920c1d407fc73b253c407fdd0aef25c908048f181025ae7f5`.

after: lines 80..81, 35 bytes, SHA256 `fd5f0e9e870ae08920c1d407fc73b253c407fdd0aef25c908048f181025ae7f5`.

## absent — ADDED

after: lines 263..268, 186 bytes, SHA256 `947f6b50c9344014a2e2d2a3cc77b865affc8ba26d46a98c5b8bbcc62d600b0f`.

before:

```python
# No declaration in this version.
```

after:

```python
def absent(rootfd, name):
    try:
        os.stat(name, dir_fd=rootfd, follow_symlinks=False)
    except FileNotFoundError:
        return
    raise Stop('must remain absent: ' + name)
```

## bind_config — CHANGED

before: lines 202..235, 2206 bytes, SHA256 `406faf6d0c061c2ed58a25141ca642350f85cc095b09fc6311b7ad042ec222bb`.

after: lines 404..463, 4832 bytes, SHA256 `4953f75a56b3a951110f6415b156abaa192b5c1121fe9e23065c79eecf11a2b4`.

before:

```python
def bind_config():
    require(type(CONFIG) is dict and set(CONFIG) == {
        'schema', 'execution', 'evidence_root', 'template_sha256', 'caller',
        'payload_review', 'prerequisites', 'initial_inputs', 'completed_files',
        'manifest_sha256', 'saved_audit_commit'}, 'closed config keys')
    require(CONFIG['schema'] == 'openergy.source-stdin-config.v1'
            and CONFIG['execution'] is False and CONFIG['evidence_root'] == ROOT,
            'inert fixed SOURCE configuration')
    require(hex_string(CONFIG['template_sha256'], 64), 'pending template hash')
    caller = CONFIG['caller']
    require(type(caller) is dict and set(caller) == {
        'candidate_commit', 'bytes', 'sha256', 'review_commit', 'review_sha256',
        'review_verdict'}, 'closed caller binding')
    require(hex_string(caller['candidate_commit'], 40)
            and type(caller['bytes']) is int and 0 < caller['bytes'] <= RAW_CAP
            and hex_string(caller['sha256'], 64)
            and hex_string(caller['review_commit'], 40)
            and hex_string(caller['review_sha256'], 64)
            and caller['review_verdict'] == 'CLEAR', 'pending final independent CLEAR caller')
    review = CONFIG['payload_review']
    require(type(review) is dict and set(review) == {'commit', 'sha256', 'verdict'}
            and hex_string(review['commit'], 40) and hex_string(review['sha256'], 64)
            and review['verdict'] == 'CLEAR', 'pending independent payload review')
    require(same_data(CONFIG['prerequisites'], PREREQUISITES), 'real parent prerequisites required')
    require(same_data(CONFIG['initial_inputs'], INITIAL_INPUTS)
            and same_data(CONFIG['completed_files'], COMPLETED)
            and CONFIG['manifest_sha256'] == MANIFEST_HASH
            and CONFIG['saved_audit_commit'] == AUDIT
            and sha(encoded(COMPLETED)) == MANIFEST_HASH, 'fixed input/fact/manifest bindings')
    # Canonical CONFIG digest omits LF. Template digest is the unrendered file.
    config_hash = sha(encoded(CONFIG)[:-1])
    return dict(template_sha256=CONFIG['template_sha256'],
                config_canonical_sha256=config_hash,
                caller=caller, payload_review=review)
```

after:

```python
def bind_config():
    value = closed_json(CONFIG_TEXT)
    require(type(value) is dict and set(value) == {'schema', 'execution', 'evidence_root',
        'template_sha256', 'reviews', 'parent', 'prepared', 'private_COPY_startup'}, 'closed CONFIG keys')
    require(value['schema'] == 'openergy.copy-parent-config.v1' and value['execution'] is False
            and value['evidence_root'] == ROOT and value['private_COPY_startup'] == 'UNMEASURED',
            'fixed inert COPY; no startup inference')
    require(hex_string(value['template_sha256'], 64), 'unbound template hash')
    require(type(value['reviews']) is dict and set(value['reviews']) == {'capsule', 'prepare'},
            'closed actual independent reviews')
    for label, review in value['reviews'].items():
        require(type(review) is dict and set(review) == {'candidate_commit', 'source_sha256',
            'review_commit', 'review_sha256', 'verdict'} and hex_string(review['candidate_commit'], 40)
            and hex_string(review['source_sha256'], 64) and hex_string(review['review_commit'], 40)
            and hex_string(review['review_sha256'], 64) and review['verdict'] == 'CLEAR_SOURCE_ONLY',
            'pending independent full review: ' + label)
    require(value['reviews']['capsule']['source_sha256'] == value['template_sha256']
            and value['reviews']['prepare']['source_sha256'] != PREPARE_TEMPLATE['sha256'],
            'actual exact reviews; original prepare bytes are HOLD_SOURCE, repair binding pending')
    require(same_data(value['parent'], {name: True for name in PARENT_DUTIES}),
            'EXTERNAL causal parent acts mandatory; booleans do not authenticate them')
    prepared = value['prepared']
    require(type(prepared) is dict and set(prepared) == {'file', 'private_root', 'bundle',
        'prepare_receipt', 'prepare_guard', 'prepare_stdout', 'prepare_stderr', 'prepare_status',
        'external_parent_proof'}, 'closed new prepared binding')
    for key in ('file', 'prepare_receipt', 'prepare_guard', 'prepare_stdout', 'prepare_stderr', 'prepare_status'):
        raw_binding(prepared[key], RAW_CAP)
    require(prepared['prepare_stderr'] == {'bytes': 0, 'sha256': sha(b'')}, 'prepare raw stderr empty')
    private = prepared['private_root']
    require(type(private) is str and Path(private).parent == Path('/private/tmp')
            and private.startswith('/private/tmp/oe-m1-ordinary-fresh-')
            and len(Path(private).name) > len('oe-m1-ordinary-fresh-')
            and len(private) <= 255 and prepared['bundle'] == private + '/Openergy.app',
            'new tuple pending; no invented/historical path')
    proof = prepared['external_parent_proof']
    require(type(proof) is dict and set(proof) == {'schema', 'parent_record', 'transport_receipt',
        'local_once_guard', 'payload_sha256', 'actual_collector_exit', 'actual_ssh_exit', 'both_eof',
        'no_errors_loss_timeout_kill', 'actual_wait_or_poll_returned', 'started_ns', 'work_end_ns',
        'absolute_end_ns', 'exit_observed_ns', 'ssh_exit_observed_ns', 'receipt_reads_before_exit_observation', 'phase',
        'same_local_host_boot', 'clock_domain'}, 'closed actual prepare parent proof')
    for key in ('parent_record', 'transport_receipt', 'local_once_guard'):
        raw_binding(proof[key], 32768)
    require(proof['schema'] == 'openergy.prepare-external-parent-proof.v1'
            and proof['phase'] == 'prepare' and proof['clock_domain'] == 'local-parent-only'
            and hex_string(proof['payload_sha256'], 64)
            and type(proof['actual_collector_exit']) is int and proof['actual_collector_exit'] == 0
            and type(proof['actual_ssh_exit']) is int and proof['actual_ssh_exit'] == 0
            and all(proof[k] is True for k in ('both_eof', 'no_errors_loss_timeout_kill',
                'actual_wait_or_poll_returned', 'same_local_host_boot'))
            and type(proof['receipt_reads_before_exit_observation']) is int
            and proof['receipt_reads_before_exit_observation'] == 0, 'actual prepare outer/D4 required')
    require(all(type(proof[k]) is int and 0 <= proof[k] < 10 ** 20 for k in
            ('started_ns', 'work_end_ns', 'absolute_end_ns', 'exit_observed_ns', 'ssh_exit_observed_ns'))
            and proof['work_end_ns'] == proof['started_ns'] + 130 * NS
            and proof['absolute_end_ns'] == proof['started_ns'] + 135 * NS
            and proof['started_ns'] <= proof['ssh_exit_observed_ns'] < proof['work_end_ns']
            and proof['started_ns'] <= proof['exit_observed_ns'] < proof['absolute_end_ns'],
            'original prepare local135 D4; NEVER compare with target clocks')
    return value, dict(template_sha256=value['template_sha256'],
        config_canonical_sha256=sha(encoded(value)[:-1]), reviews=value['reviews'], fixed=FIXED_BINDINGS)
```

## canonical_prepared_directory — ADDED

after: lines 304..320, 887 bytes, SHA256 `0eba03c23e2c12eafe1e1a13d91721ed671ead77aaaaadd2f4725d575ff6bf1d`.

before:

```python
# No declaration in this version.
```

after:

```python
def canonical_prepared_directory(value, rootfd):
    require(type(value) is str and value.startswith('/') and '..' not in Path(value).parts,
            'new prepared canonical path')
    fd = os.open('/', os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        for part in value.split('/')[1:]:
            child = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=fd)
            os.close(fd)
            fd = child
        info = os.fstat(fd)
        require(info.st_uid == os.getuid() and stat.S_IMODE(info.st_mode) == 0o700
                and info.st_dev == os.fstat(rootfd).st_dev
                and stamp(info) == stamp(os.stat(value, follow_symlinks=False)),
                'new prepared directory owner/mode/filesystem')
        require(str(Path(value).resolve(strict=True)) == value, 'canonical prepared directory')
    finally:
        os.close(fd)
```

## clock_ns — UNCHANGED

before: lines 134..139, 342 bytes, SHA256 `0a708be90d476ccafb3f658a6edeae5a969cd1fed47fc9e4568632c78144e7bf`.

after: lines 91..96, 342 bytes, SHA256 `0a708be90d476ccafb3f658a6edeae5a969cd1fed47fc9e4568632c78144e7bf`.

## closed_json — CHANGED

before: lines 176..183, 254 bytes, SHA256 `10175102b311e37056ff24471573c7607986e70a11ff37918b02fb8d7086aec6`.

after: lines 246..259, 524 bytes, SHA256 `019219040de9c1e8fb305ae0ee2e61a3335591568ac79d70de9011d53b77b2eb`.

before:

```python
def closed_json(data):
    def pairs(items):
        out = {}
        for key, value in items:
            require(key not in out, 'duplicate JSON key')
            out[key] = value
        return out
    return json.loads(data, object_pairs_hook=pairs)
```

after:

```python
def closed_json(data):
    def pairs(items):
        out = {}
        for key, value in items:
            require(key not in out, 'duplicate JSON key')
            out[key] = value
        return out
    def nonfinite(_value):
        raise Stop('nonfinite JSON constant')
    def finite_float(value):
        result = float(value)
        require(math.isfinite(result), 'nonfinite JSON number')
        return result
    return json.loads(data, object_pairs_hook=pairs, parse_constant=nonfinite, parse_float=finite_float)
```

## closed_json.finite_float — ADDED

after: lines 255..258, 145 bytes, SHA256 `580f75ecd092de39c675e9c790c854f42fe72decd96690c24b15ad302f5d7b85`.

before:

```python
# No declaration in this version.
```

after:

```python
    def finite_float(value):
        result = float(value)
        require(math.isfinite(result), 'nonfinite JSON number')
        return result
```

## closed_json.nonfinite — ADDED

after: lines 253..254, 73 bytes, SHA256 `f32a3a64e87b6448d7ef0b65c9c39395f23789cd334acc2948a07eb81d192925`.

before:

```python
# No declaration in this version.
```

after:

```python
    def nonfinite(_value):
        raise Stop('nonfinite JSON constant')
```

## closed_json.pairs — UNCHANGED

before: lines 177..182, 178 bytes, SHA256 `0631fc06468cfa7e32fdd726ce494a810b7000bdc90d5bc09083833869579093`.

after: lines 247..252, 178 bytes, SHA256 `0631fc06468cfa7e32fdd726ce494a810b7000bdc90d5bc09083833869579093`.

## collect — CHANGED

before: lines 438..549, 6348 bytes, SHA256 `1c5c67d0cbb061573921e5291c1cde20eaaa5bc2f484e5e8df1c8b7a57b16d21`.

after: lines 725..871, 8433 bytes, SHA256 `e1a6ec1b79b46bc03455d6ec8bf665029d7b0effbf6d3d3d682ea7ed8b188104`.

before:

```python
def collect(started, end, parent_sha):
    global HANDLE
    # Stop ordinary collection 2s early, reserve .5s for direct kill/reap,
    # then publication/console inside the SAME original120. Never reset it.
    work_end, reap_end = end - 2 * NS, end - 1500000000
    buffers = {'stdout': bytearray(), 'stderr': bytearray()}
    argv = [SYSTEM, '-I', '-S', '-B', ROOT + '/metadata-caller.py.txt',
            'source', ROOT, str(started), parent_sha]
    status = dict(schema='openergy.source-caller-collection.v1', clock=CLOCK,
                  argv=argv, cwd=ROOT,
                  phase_started_ns=started, phase_deadline_ns=end,
                  collection_end_ns=work_end, reap_end_ns=reap_end,
                  launch_started_ns=None, handle_retained=False, actual_exit=None,
                  actual_exit_observed_ns=None, reaped=False,
                  eof={'stdout': False, 'stderr': False}, observed_bytes=0,
                  dropped_known_bytes=0, unread_or_dropped_unknown=False,
                  overflow=False, timed_out=False, deadline_loss=False,
                  kill_attempted=False, kill_delivered=False, kill_or_reap_error=None,
                  kill_eligibility_exit=None, kill_eligibility_observed_ns=None,
                  first_error=None, close_errors=[], finished_ns=None)
    proc, selector = None, None
    try:
        with until(work_end):
            require(clock_ns() + 500000000 < work_end, 'insufficient caller budget')
            status['launch_started_ns'] = clock_ns()
            proc = subprocess.Popen(argv,
                                    stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                                    stderr=subprocess.PIPE, cwd=ROOT, env=startup(),
                                    shell=False, close_fds=True, bufsize=0)
            HANDLE = proc
            status['handle_retained'] = True
            selector = selectors.DefaultSelector()
            for name in ('stdout', 'stderr'):
                stream = getattr(proc, name)
                os.set_blocking(stream.fileno(), False)
                selector.register(stream, selectors.EVENT_READ, name)
            while True:
                code = proc.poll()
                observe_exit(proc, status, code)
                if code is not None and all(status['eof'].values()):
                    break
                require(clock_ns() < work_end, 'caller work ceiling')
                for key, _ in selector.select(min(0.05, remaining(work_end))):
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
                        raise Stop('one overflow detection byte; no drain')
    except BaseException as exc:
        latch(status, 'collect', exc)
        if isinstance(exc, TimeoutError) or clock_ns() >= work_end:
            status['timed_out'] = True
    finally:
        if proc is not None and not status['reaped']:
            try:
                with until(reap_end):
                    code = proc.poll()
                    observed_ns = observe_exit(proc, status, code)
                    status['kill_eligibility_exit'] = code
                    status['kill_eligibility_observed_ns'] = observed_ns
                    if code is None:
                        if not (observed_ns < reap_end and observed_ns < end):
                            status['timed_out'] = True
                            status['unread_or_dropped_unknown'] = True
                            raise TimeoutError('late KILL eligibility poll; no signal')
                        status['kill_attempted'] = True
                        proc.kill()  # Retained direct SYSTEM caller ONLY; no group/PID sweep.
                        status['kill_delivered'] = True
                        code = proc.wait(timeout=remaining(reap_end))
                        observe_exit(proc, status, code)
            except BaseException as exc:
                status['kill_or_reap_error'] = type(exc).__name__
                latch(status, 'kill_or_reap', exc)
        if proc is not None:
            for name in ('stdout', 'stderr'):
                stream = getattr(proc, name)
                if stream is not None:
                    try:
                        stream.close()
                    except BaseException as exc:
                        status['close_errors'].append(dict(stream=name, error=type(exc).__name__))
                        latch(status, 'pipe_close', exc)
        if selector is not None:
            try:
                selector.close()
            except BaseException as exc:
                status['close_errors'].append(dict(stream='selector', error=type(exc).__name__))
                latch(status, 'selector_close', exc)
    status['unread_or_dropped_unknown'] |= not all(status['eof'].values())
    status['retained_bytes'] = {name: len(data) for name, data in buffers.items()}
    status['finished_ns'] = clock_ns()
    status['deadline_loss'] = status['finished_ns'] >= end
    if (type(status['actual_exit']) is not int or status['actual_exit'] != 0
            or not status['reaped'] or not all(status['eof'].values())
            or status['kill_attempted'] or status['overflow'] or status['timed_out']
            or status['deadline_loss'] or status['unread_or_dropped_unknown']
            or status['actual_exit_observed_ns'] is None
            or status['actual_exit_observed_ns'] >= end):
        latch(status, 'incomplete_actual_collection', Stop('incomplete'))
    # Full bounded raw prefix and complete first status retained BEFORE any write.
    return status, bytes(buffers['stdout']), bytes(buffers['stderr'])
```

after:

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

## collection_complete — UNCHANGED

before: lines 552..561, 799 bytes, SHA256 `1e392aa0f5b155a79c614f7f8b967bf8edeefba7fb7729ccaa3ceacc25c6a793`.

after: lines 211..220, 799 bytes, SHA256 `1e392aa0f5b155a79c614f7f8b967bf8edeefba7fb7729ccaa3ceacc25c6a793`.

## common_deadline — ADDED

after: lines 339..354, 928 bytes, SHA256 `1d8dcb38b35c9dd3390e5275b12940e0d8b5662ccf01a52aa230ad8a87da6b72`.

before:

```python
# No declaration in this version.
```

after:

```python
def common_deadline(common, deadline, conversions):
    # Cleared R1 method: common local FIRST, shared AFTER; discard the gap.
    shared_before = clock_ns()
    local_before = common.time.monotonic()
    shared_after = clock_ns()
    budget = deadline - shared_after - 1000000
    require(shared_before <= shared_after and budget > 0 and math.isfinite(local_before),
            'conservative common before/after bounds')
    duration = math.nextafter(budget / NS, -math.inf)
    local_end = math.nextafter(local_before + duration, -math.inf)
    require(math.isfinite(local_end) and local_end > local_before, 'common safe local deadline')
    remaining(deadline)
    conversions.append(dict(shared_before_ns=shared_before, local_before=local_before,
                            shared_after_ns=shared_after, shared_end_ns=deadline,
                            discarded_ns=1000000, local_end=local_end))
    return local_end
```

## console_admission — ADDED

after: lines 466..476, 664 bytes, SHA256 `f3bfa84a414e25628656a7c199629a223794a867a5a7509384f7701dd5874949`.

before:

```python
# No declaration in this version.
```

after:

```python
def console_admission():
    require(stat.S_ISFIFO(os.fstat(1).st_mode), 'actual FD1 FIFO before ANY target write')
    pipe_buf = os.fpathconf(1, 'PC_PIPE_BUF')
    require(type(pipe_buf) is int and pipe_buf >= CONSOLE_CAP, 'actual PIPE_BUF>=512')
    require(CONFIG['parent']['console_single_writer_live_reader_and_topology_owned'] is True,
            'EXTERNAL single writer/live reader topology')
    os.set_blocking(1, False)
    require(os.get_blocking(1) is False, 'actual nonblocking console')
    remaining(COPY_END)
    return dict(fifo=True, pipe_buf=pipe_buf, nonblocking=True,
                single_writer_live_reader='EXTERNAL_ACTUAL_PARENT_DUTY')
```

## copy_parent_stdin — ADDED

after: lines 903..909, 372 bytes, SHA256 `45b3ba775122419c524ac759d885a85ef9dd0f08c8cd136c744d98353219fa11`.

before:

```python
# No declaration in this version.
```

after:

```python
def copy_parent_stdin(authentication):
    value = dict(schema='openergy.metadata-parent-boundary.v2', mode='copy', clock=CLOCK,
        phase_started_ns=COPY_START, phase_deadline_ns=COPY_END,
        source_file_qualification=None, entry_authentication=authentication)
    raw = encoded(value)
    require(len(raw) <= RAW_CAP, 'canonical COPY stdin1MiB')
    return raw
```

## copy_record_complete — ADDED

after: lines 912..934, 1949 bytes, SHA256 `571357b367e6b93c3a9fa68c32bfd27f6eb0d96f98c2950637ab847dd9e815a1`.

before:

```python
# No declaration in this version.
```

after:

```python
def copy_record_complete(status, begun, end):
    # Specific immutable R9 record interface, including the R9 eligibility facts.
    require(type(status) is dict and set(status) == {'label', 'argv', 'pid', 'handle_retained',
        'actual_exit', 'reaped', 'stdout_eof', 'stderr_eof', 'observed_stream_bytes',
        'dropped_observed_bytes', 'unread_or_dropped_unknown', 'overflow', 'timed_out', 'deadline_loss',
        'kill_attempted', 'kill_delivered', 'kill_eligibility_exit', 'kill_eligibility_observed_ns',
        'error', 'kill_or_reap_error', 'clock', 'started_ns', 'command_deadline_ns', 'finished_ns',
        'stdout', 'stderr'}, 'closed exact R9 status keys')
    require(type(status['actual_exit']) is int and status['actual_exit'] == 0
            and status['reaped'] is True and status['handle_retained'] is True
            and type(status['pid']) is int and status['pid'] > 0
            and status['stdout_eof'] is True and status['stderr_eof'] is True
            and all(status[k] is False for k in ('overflow', 'timed_out', 'deadline_loss',
                'unread_or_dropped_unknown', 'kill_attempted', 'kill_delivered'))
            and type(status['dropped_observed_bytes']) is int and status['dropped_observed_bytes'] == 0
            and status['error'] is None and status['kill_or_reap_error'] is None
            and same_data(status['clock'], CLOCK), 'actual R9 command0/reap/EOF/noerror')
    require(all(type(status[k]) is int for k in ('started_ns', 'finished_ns', 'command_deadline_ns',
                                               'kill_eligibility_exit', 'kill_eligibility_observed_ns'))
            and status['kill_eligibility_exit'] == 0
            and begun <= status['started_ns'] <= status['kill_eligibility_observed_ns']
            <= status['finished_ns'] <= status['command_deadline_ns']
            <= min(status['started_ns'] + 10 * NS, end - NS), 'same original phase/command10/R9 poll gate')
```

## emit_once — CHANGED

before: lines 564..571, 406 bytes, SHA256 `e48b251396c9e4ae367c38bac5a11728923032b0f6c3ff0ed476f047e71bae55`.

after: lines 1072..1082, 728 bytes, SHA256 `a1011b7942fda9cad3a36c91aa1c0808f0add453bef0d07484bc0a96a08aaf5e`.

before:

```python
def emit_once(value, end):
    # No buffered stdio and no Python pending payload for finalization to retry.
    data = encoded(value)
    require(len(data) <= CONSOLE_CAP, 'small console envelope cap')
    with until(end):
        os.set_blocking(1, False)
        remaining(end)  # No output attempt after phase expiry.
        require(os.write(1, data) == len(data), 'partial console attempt; no retry')
```

after:

```python
def emit_once(receipt_ref, complete):
    value = dict(v='copy-parent.v1', state=('COPY_DATA_COMPLETE_EXTERNAL_PENDING' if complete
                                           else 'FAILED_UNVERIFIED'),
                 r=([receipt_ref['bytes'], receipt_ref['sha256']] if receipt_ref else None))
    data = encoded(value)
    require(len(data) <= CONSOLE_CAP, 'closed compact console<=512')
    with until(COPY_END):
        require(stat.S_ISFIFO(os.fstat(1).st_mode) and os.get_blocking(1) is False
                and os.fpathconf(1, 'PC_PIPE_BUF') >= CONSOLE_CAP, 'same admitted console FIFO')
        remaining(COPY_END)
        require(os.write(1, data) == len(data), 'single console write; partial/EAGAIN fails, no retry')
```

## encoded — UNCHANGED

before: lines 171..173, 172 bytes, SHA256 `47546d7a82bbb5e109d0a86a34e3046f3adab68837513ddf922600b33ce202d2`.

after: lines 112..114, 172 bytes, SHA256 `47546d7a82bbb5e109d0a86a34e3046f3adab68837513ddf922600b33ce202d2`.

## existing_ledger — ADDED

after: lines 324..335, 791 bytes, SHA256 `f14b8a32e7ffa286b368e1e3fb4e870b85200b59a20fac15b6a87e1b94869d81`.

before:

```python
# No declaration in this version.
```

after:

```python
def existing_ledger(rootfd):
    # No initialization, reconstruction, recovery, or fallback writer.
    data, ref = read_new_leaf(rootfd, 'bytes.json', 4096)
    lock, lock_ref = read_new_leaf(rootfd, 'bytes.lock', 0)
    ledger = closed_json(data)
    require(type(ledger) is dict and set(ledger) == {'charged', 'regular', 'product_reserved',
            'cleanup_reserved', 'infrastructure_reserved'}
            and all(type(v) is int for v in ledger.values())
            and 0 <= ledger['regular'] <= ledger['charged'] <= TOTAL_CAP
            and ledger['product_reserved'] == 16777216 and ledger['cleanup_reserved'] == 4194304
            and ledger['infrastructure_reserved'] == 65536, 'only valid prepare-created ledger')
    return dict(ledger=ref, lock=lock_ref, observed=ledger)
```

## fail — ADDED

after: lines 392..395, 139 bytes, SHA256 `de1bd3450d76eb0036a1f40c2d29b91d542d1715469a2bb1256f8f309997adcc`.

before:

```python
# No declaration in this version.
```

after:

```python
def fail(stage, exc):
    global FIRST_ERROR
    if FIRST_ERROR is None:
        FIRST_ERROR = dict(stage=stage, error=type(exc).__name__)
```

## hex_string — UNCHANGED

before: lines 186..188, 146 bytes, SHA256 `fd9f19e03ed808aeded45d465053a19b34fd34724e58ee0d2c21641f1df07d78`.

after: lines 118..120, 146 bytes, SHA256 `fd9f19e03ed808aeded45d465053a19b34fd34724e58ee0d2c21641f1df07d78`.

## import_system_common — ADDED

after: lines 641..665, 1585 bytes, SHA256 `3b170a965428cfbeb167124a0e1bb44a9d8cf15fcb661f6c02860f3f77aac375`.

before:

```python
# No declaration in this version.
```

after:

```python
def import_system_common(rootfd):
    # This is SYSTEM code, not a private interpreter entry. All fixed inputs,
    # artifact/raw/prepared/ledger boundaries were verified BEFORE this point.
    # Two fixed SYSTEM-only bindings; neither is inherited by a child except
    # through the exact unchanged ten-key startup() map.
    os.environ['OPENERGY_ORDINARY_SOURCE_ROOT'] = SOURCE_ROOT
    os.environ['OPENERGY_ORDINARY_PYTHON'] = PRIVATE_PYTHON
    system_bootstrap()
    for name, binding in PINS.items():
        read_leaf(rootfd, name, binding)
    require('common' not in sys.modules, 'one exact common import')
    reject_entry_caches(rootfd)  # -B forbids writes, not reads of stale bytecode.
    before_modules = set(sys.modules)
    spec = importlib.util.spec_from_file_location('common', ROOT + '/common.py')
    module = importlib.util.module_from_spec(spec)
    sys.modules['common'] = module
    spec.loader.exec_module(module)
    require(set(sys.modules) - before_modules == {'common'}, 'all common stdlib imports already SYSTEM-loaded')
    system_bootstrap(module)  # Includes transitive import identity, before any writer.
    require(module.HERE == Path(ROOT) and str(module.REPO) == SOURCE_ROOT and module.PYTHON == PRIVATE_PYTHON
            and module.LOG_CAP == FILE_CAP and module.EVIDENCE_CAP == TOTAL_CAP
            and module.PRODUCT_RESERVE == 16777216 and module.CLEANUP_RESERVE == 4194304
            and module.ACCOUNTING_RESERVE == 65536 and module.time is time,
            'unchanged common identity/constants/local-clock object')
    return module
```

## inspect_artifact — ADDED

after: lines 358..388, 2240 bytes, SHA256 `1824e62a3616f2a40b03f3d9316fa1c96120a00626ab308ffff2f14150185c30`.

before:

```python
# No declaration in this version.
```

after:

```python
def inspect_artifact(rootfd):
    raw = {name: read_leaf(rootfd, name, binding) for name, binding in EIGHT.items()}
    require(sum(len(v) for v in raw.values()) == 136360, 'exact eight pending bytes')
    artifact = closed_json(raw['artifact-receipt.json'])
    operation = closed_json(raw['operation.bound.json'])
    require(same_data(operation, dict(schema='openergy.m1-ordinary-operation.v1',
            execution=True, artifact_receipt_sha256=EIGHT['artifact-receipt.json']['sha256'])),
            'fixed operation binds exact actual receipt bytes')
    require(type(artifact) is dict and set(artifact) == set(ARTIFACT_SHAPE), 'closed artifact keys')
    for key in ('schema', 'build_candidate', 'build_review', 'source_commit', 'source_root',
                'source_bundle', 'build_bundle_path', 'app_sha256', 'daemon_sha256', 'python'):
        require(same_data(artifact[key], ARTIFACT_SHAPE[key]), 'fixed artifact identity: ' + key)
    require(same_data(artifact['qualification_gates'],
                     {k: True for k in ARTIFACT_SHAPE['qualification_gates']}), 'all ten static artifact gates')
    require(type(artifact['evidence']) is dict and set(artifact['evidence']) == set(ARTIFACT_SHAPE['evidence']),
            'closed nine artifact references')
    qualifiers = {'artifact_qualification': 'qualification-artifact-observations.json',
                  'python_qualification': 'qualification-python.json',
                  'abi_qualification': 'qualification-abi.json',
                  'transfer_qualification': 'qualification-transfer.json'}
    for label, record in artifact['evidence'].items():
        reference(record)
        expected = BUILD_INPUTS[label] if label in BUILD_INPUTS else dict(
            path=ROOT + '/' + qualifiers[label], **EIGHT[qualifiers[label]])
        require(same_data(record, expected), 'artifact exact evidence binding')
    support = artifact['supporting_evidence']
    require(type(support) is list and 1 <= len(support) <= 128, 'supporting evidence unchanged cap')
    for record in support:
        reference(record)  # No reads selected by these data paths.
    require(len({r['path'] for r in support}) == len(support), 'unique support references')
    return artifact
```

## inspect_copy_result — ADDED

after: lines 949..1069, 10406 bytes, SHA256 `0389af1ff2c6ab0cfb48e1e1442604c16eda444701cd2788e73b1cf846442ec4`.

before:

```python
# No declaration in this version.
```

after:

```python
def inspect_copy_result(rootfd, caller, stdout, stderr, prepared, source):
    require(collection_complete(caller) and caller['stdin_complete'] is True
            and stderr == b'' and 0 < len(stdout) <= 512, 'actual private caller0/EOF/compact only')
    compact = closed_json(stdout)
    require(type(compact) is dict and set(compact) == {'schema', 'mode', 'phase_started_ns',
        'phase_deadline_ns', 'result'} and encoded(compact) == stdout
        and compact['schema'] == 'openergy.metadata-success.v1' and compact['mode'] == 'copy'
        and type(compact['phase_started_ns']) is int and compact['phase_started_ns'] == COPY_START
        and type(compact['phase_deadline_ns']) is int and compact['phase_deadline_ns'] == COPY_END,
        'exact canonical compact success boundary')
    result_raw = read_record_ref(rootfd, compact['result'], 'metadata-copy-result.json')
    require(len(result_raw) <= RAW_CAP, 'full COPY result bounded independently from console')
    result = closed_json(result_raw)
    keys = {'schema', 'clock', 'mode', 'expected_basis_sha256', 'started_ns', 'phase_deadline_ns',
        'native_records', 'auxiliary_records', 'completed_native_commands', 'observation_sequence_complete',
        'first_error', 'artifact_gate_authority', 'runtime', 'actual_direct_children', 'first_failed_command',
        'first_failed_comparison', 'collected_commands', 'active_command', 'publication_errors',
        'observations_finished_ns'}
    require(type(result) is dict and set(result) == keys and encoded(result) == result_raw
            and result['schema'] == 'openergy.metadata-observations.v2' and result['mode'] == 'copy'
            and same_data(result['clock'], CLOCK) and type(result['started_ns']) is int
            and result['started_ns'] == COPY_START and type(result['phase_deadline_ns']) is int
            and result['phase_deadline_ns'] == COPY_END and result['observation_sequence_complete'] is True
            and type(result['completed_native_commands']) is int and result['completed_native_commands'] == 50
            and result['first_error'] is None and result['first_failed_command'] is None
            and result['first_failed_comparison'] is None and result['publication_errors'] == []
            and result['artifact_gate_authority'] is False and result['runtime'] == 'UNVERIFIED'
            and type(result['observations_finished_ns']) is int
            and COPY_START <= result['observations_finished_ns'] <= caller['actual_exit_observed_ns'] < COPY_END,
            'full COPY success is pre-exit data plus ACTUAL caller status')
    vectors = closed_json(read_leaf(rootfd, 'command-vectors.json', INITIAL_INPUTS['command-vectors.json']))['commands']
    basis = closed_json(read_leaf(rootfd, 'expected-source-metadata.json', INITIAL_INPUTS['expected-source-metadata.json']))
    require(len(vectors) == len(basis['commands']) == len(result['native_records']) == 50
            and result['expected_basis_sha256'] == INITIAL_INPUTS['expected-source-metadata.json']['sha256']
            and basis['bundle'] == ARTIFACT_SHAPE['source_bundle']
            and len(result['auxiliary_records']) == 2, 'closed50 native/two file observations')
    items = result['auxiliary_records'][:1] + result['native_records'] + result['auxiliary_records'][1:]
    labels = ['copy-files-before'] + [v['label'] for v in vectors] + ['copy-files-after']
    require(len(items) == len(result['collected_commands']) == len(result['actual_direct_children']) == 52
            and [x['label'] for x in items] == labels, 'EXACT52 order; no completion commands')
    retained, statuses, file_facts = [], [], []
    last_finish = caller['launch_started_ns']
    for index, (item, collected, child) in enumerate(zip(items, result['collected_commands'], result['actual_direct_children'])):
        label = labels[index]
        require(set(item) == {'label', 'os_record', 'comparison', 'stdout', 'stderr'}
                and set(collected) == {'label', 'collector_status', 'stdout', 'stderr', 'os_record',
                    'comparison', 'publication', 'comparison_state'}
                and set(collected['publication']) == {'stdout', 'stderr', 'os_record'}
                and set(child) == {'label', 'argv', 'pid', 'actual_exit', 'handle_retained', 'collector_status'}
                and collected['label'] == child['label'] == label
                and collected['comparison_state'] == 'durable', 'R9 record interfaces')
        raws = {}
        for key, suffix in (('stdout', '.stdout'), ('stderr', '.stderr'),
                            ('os_record', '-os.json'), ('comparison', '-comparison.json')):
            leaf = 'metadata-copy-' + label + suffix
            raws[key] = read_record_ref(rootfd, item[key], leaf)
            retained.append(dict(leaf=leaf, bytes=len(raws[key]), sha256=sha(raws[key])))
            require(same_data(collected[key], item[key]), 'collected raw reference')
            if key != 'comparison':
                pub = collected['publication'][key]
                require(pub['state'] == 'durable' and same_data(pub['ref'], item[key])
                        and pub['path'] == item[key]['path'], 'raw/status DURABLE before comparison')
        require(sum(len(raws[k]) for k in ('stdout', 'stderr')) <= RAW_CAP, 'combined child raw cap')
        status, comparison = closed_json(raws['os_record']), closed_json(raws['comparison'])
        copy_record_complete(status, COPY_START, COPY_END)
        require(last_finish <= status['started_ns'] and status['finished_ns'] <= result['observations_finished_ns'],
                'actual ordered command clocks')
        last_finish = status['finished_ns']
        require(same_data(status['stdout'], item['stdout']) and same_data(status['stderr'], item['stderr'])
                and type(status['observed_stream_bytes']) is int
                and status['observed_stream_bytes'] == len(raws['stdout']) + len(raws['stderr']), 'full raw count/binding')
        stripped = {k: v for k, v in status.items() if k not in ('stdout', 'stderr')}
        require(same_data(stripped, collected['collector_status'])
                and type(child['actual_exit']) is int and child['actual_exit'] == 0
                and child['handle_retained'] is True and type(child['pid']) is int
                and child['pid'] == status['pid'] and child['argv'] == status['argv']
                and same_data(child['collector_status'], status), 'actual direct/collected/status agreement')
        require(type(comparison) is dict and set(comparison) == {'label', 'os_record', 'observations_complete',
                'comparisons', 'comparison_error', 'match'} and comparison['label'] == label
                and same_data(comparison['os_record'], item['os_record'])
                and comparison['observations_complete'] is True and comparison['comparison_error'] is None
                and comparison['match'] is True, 'actual successful comparator')
        if index in (0, 51):
            require(status['argv'] == [SYSTEM, '-I', '-S', '-B', ROOT + '/file-qualification.recipe.py.txt', ROOT, 'copy']
                    and same_data(comparison['comparisons'], {'file_observation': True, 'copy_file_observation': True})
                    and raws['stderr'] == b'', 'before/after exact COPY helper')
            facts = closed_json(raws['stdout'])
            expected = dict(mode='copy', prefix_entries=2006, prefix_regular_bytes=46504040,
                prefix_sha256=ARTIFACT_SHAPE['python']['prefix_sha256'], modules=10, prefix_root_mode='0o700',
                schema='openergy.file-qualification-observation.v1', product_invocations=0, runtime='UNVERIFIED',
                source_files=702, bundle_records=167, sdk_headers=10, copied_bundle=prepared['bundle'])
            require(set(facts) == set(expected) | {'elapsed_seconds'}
                    and all(same_data(facts[k], v) for k, v in expected.items())
                    and type(facts['elapsed_seconds']) in (int, float) and math.isfinite(facts['elapsed_seconds'])
                    and 0 <= facts['elapsed_seconds'] <= 10, 'both complete actual copy-file facts')
            file_facts.append(dict(label=label, facts=facts, raw=item['stdout']))
        else:
            vector, saved, fresh = vectors[index - 1], basis['commands'][index - 1], source['native_records'][index - 1]
            argv = [prepared['bundle'] + token[len('${BUNDLE}'):] if token == '${BUNDLE}'
                    or token.startswith('${BUNDLE}/') else token for token in vector['argv']]
            require(status['argv'] == argv and saved['label'] == fresh['label'] == vector['label'] == label
                    and type(saved['exit']) is int and saved['exit'] == 0
                    and same_data(comparison['comparisons'], {'saved_basis': True, 'fresh_source': True}),
                    'fixed COPY argv and BOTH comparators')
            for stream in ('stdout', 'stderr'):
                fresh_raw = read_record_ref(rootfd, fresh[stream], 'metadata-source-' + label + '.' + stream)
                require(normalize(raws[stream], prepared['bundle'], label)
                        == normalize(saved[stream].encode('utf-8'), basis['bundle'], label)
                        == normalize(fresh_raw, basis['bundle'], label), 'actual full raw saved/fresh equality')
        statuses.append(dict(label=label, actual_exit=status['actual_exit'], started_ns=status['started_ns'],
            finished_ns=status['finished_ns'], command_deadline_ns=status['command_deadline_ns'],
            stdout=item['stdout'], stderr=item['stderr'], os_record=item['os_record'], comparison=item['comparison']))
    require(same_data(result['active_command'], result['collected_commands'][-1]) and len(retained) == 208,
            'last active record/full208 retained references')
    return dict(full_result=compact['result'], actual_commands=52, native_commands=50,
        actual_all52_exit0_reap_EOF=True, before_after_file_facts=file_facts,
        all50_saved_basis_and_fresh_source_raw_equal=True, records=statuses,
        retained_raw_status_comparators=retained, retained_raw_bytes=sum(x['bytes'] for x in retained),
        source_result=dict(path=ROOT + '/metadata-source-result.json', **SOURCE_RESULT),
        started_ns=COPY_START, phase_deadline_ns=COPY_END,
        observations_finished_ns=result['observations_finished_ns'])
```

## inspect_entry — ADDED

after: lines 874..900, 2363 bytes, SHA256 `03bab6b7b13b33df652b870e0e8385596601bd5533f0a533e60247214367d079`.

before:

```python
# No declaration in this version.
```

after:

```python
def inspect_entry(status, stdout, stderr):
    require(collection_complete(status) and FIRST_ERROR is None and stderr == b''
            and status['child_started_ns'] <= status['launch_started_ns']
            <= status['actual_exit_observed_ns'] <= status['finished_ns'] < status['child_deadline_ns']
            and status['finished_ns'] - status['child_started_ns'] <= 10 * NS,
            'helper actual0/reap/EOF/close/error/clock within same10')
    facts = closed_json(stdout)
    require(type(facts) is dict and set(facts) == {'mode', 'prefix_entries', 'prefix_regular_bytes',
        'prefix_sha256', 'modules', 'prefix_root_mode', 'schema', 'elapsed_seconds',
        'product_invocations', 'runtime'}, 'closed full entry observation')
    expected = dict(mode='entry', prefix_entries=2006, prefix_regular_bytes=46504040,
        prefix_sha256=ARTIFACT_SHAPE['python']['prefix_sha256'], modules=10, prefix_root_mode='0o700',
        schema='openergy.file-qualification-observation.v1', product_invocations=0, runtime='UNVERIFIED')
    require(all(same_data(facts[k], v) for k, v in expected.items())
            and type(facts['elapsed_seconds']) in (int, float) and math.isfinite(facts['elapsed_seconds'])
            and 0 <= facts['elapsed_seconds'] <= 10, 'full prefix/exe digest/mode700/modules10 facts')
    # Helper SHA authenticates unchanged full executable hash/mode algorithm.
    # elapsed_seconds stays the helper's OWN relative duration, never retagged.
    inspected = clock_ns()  # NEW sample AFTER every actual/raw/fact predicate.
    return dict(schema='openergy.metadata-entry-observation.v2', clock=CLOCK,
        actual_exit=status['actual_exit'], reaped=status['reaped'], both_eof=all(status['eof'].values()),
        overflow=status['overflow'], deadline_loss=status['deadline_loss'], timed_out=status['timed_out'],
        unread_or_dropped_unknown=status['unread_or_dropped_unknown'], kill_attempted=status['kill_attempted'],
        error=status['first_error'], kill_or_reap_error=status['kill_or_reap_error'],
        stdout=base64.b64encode(stdout).decode('ascii'), stderr=base64.b64encode(stderr).decode('ascii'),
        system_python=SYSTEM, helper_sha256=PINS['file-qualification.recipe.py.txt']['sha256'],
        started_ns=status['child_started_ns'], finished_ns=status['finished_ns'], parent_inspected_ns=inspected)
```

## inspect_fixed_inputs — ADDED

after: lines 479..497, 1373 bytes, SHA256 `dfa95fb06f7fc255ef73210be8363b3906b72622c4b580283e8345cf5b57bf16`.

before:

```python
# No declaration in this version.
```

after:

```python
def inspect_fixed_inputs(rootfd):
    for name, binding in INITIAL_INPUTS.items():
        read_leaf(rootfd, name, binding)
    pins = closed_json(read_leaf(rootfd, 'input-pins.json', INITIAL_INPUTS['input-pins.json']))
    require(set(pins) == {'schema', 'execution', 'files'}
            and pins['schema'] == 'openergy.metadata-caller-pins.v1' and pins['execution'] is False
            and len(pins['files']) == 17 and set(pins['files']) == set(PINS), 'exact17 closure')
    for name, expected in PINS.items():
        record = pins['files'][name]
        require(type(record) is dict and set(record) == {'bytes', 'repository_path', 'sha256'}
                and same_data({k: record[k] for k in ('bytes', 'sha256')}, expected), 'original pin identity')
        read_leaf(rootfd, name, expected)
    checks = closed_json(read_leaf(rootfd, 'source-checks.json', PINS['source-checks.json']))
    require(set(checks['python_files']) == {n for n in PINS if n.endswith('.py')}
            and len(checks['python_files']) == 10, 'exact ten unchanged module destinations')
    for name, record in checks['python_files'].items():
        require(record['bytes'] == PINS[name]['bytes'] and record['sha256'] == PINS[name]['sha256'],
                'module/source-check/pin agreement')
    read_leaf(rootfd, 'metadata-caller.py.txt', FIXED_BINDINGS['metadata_caller']['source'])
```

## inspect_initial — REMOVED

before: lines 321..325, 253 bytes, SHA256 `4b7e534ca10f41db43a109cd6115974c9ced0b54b9a1375d5afb46ae68938ecc`.

before:

```python
def inspect_initial(rootfd):
    for name, binding in INITIAL_INPUTS.items():
        read_leaf(rootfd, name, binding)
    caller = CONFIG['caller']
    read_leaf(rootfd, 'metadata-caller.py.txt', {'bytes': caller['bytes'], 'sha256': caller['sha256']})
```

after:

```python
# No declaration in this version.
```

## inspect_new_prepared — ADDED

after: lines 538..607, 5696 bytes, SHA256 `6afa0d4c94cac119bdea57f5617f7484b75a597883b048fa11191349e36d3b25`.

before:

```python
# No declaration in this version.
```

after:

```python
def inspect_new_prepared(rootfd, artifact):
    bindings = CONFIG['prepared']
    names = {'file': 'prepared.json', 'prepare_receipt': 'prepare-parent-receipt.json',
             'prepare_guard': 'prepare-parent-once.guard', 'prepare_stdout': 'prepare-parent-stdout.bin',
             'prepare_stderr': 'prepare-parent-stderr.bin', 'prepare_status': 'prepare-parent-os.json'}
    raw = {key: read_leaf(rootfd, name, bindings[key]) for key, name in names.items()}
    prepared, receipt, primary, guard = (closed_json(raw[k]) for k in
        ('file', 'prepare_receipt', 'prepare_status', 'prepare_guard'))
    require(type(prepared) is dict and set(prepared) == set(PREPARED_KEYS), 'closed actual prepared shape')
    private, bundle = prepared['private_root'], prepared['bundle']
    require(private == bindings['private_root'] and bundle == bindings['bundle']
            and bundle == private + '/Openergy.app', 'tuple ONLY from new full prepared bytes')
    canonical_prepared_directory(private, rootfd)
    canonical_prepared_directory(bundle, rootfd)
    require(receipt['schema'] == 'openergy.prepare-parent-receipt.v1'
            and receipt['state'] == 'PREPARED_DATA_COMPLETE' and receipt['evidence_root'] == ROOT
            and receipt['first_error'] is None and receipt['publication_errors'] == []
            and same_data(receipt['primary'], primary) and collection_complete(primary)
            and primary['argv'] == [PRIVATE_PYTHON, '-B', ROOT + '/prepare.py']
            and same_data(primary['environment'], startup()) and raw['prepare_stderr'] == b''
            and receipt['prepared']['raw'] == dict(path=ROOT + '/prepared.json', **bindings['file'])
            and receipt['prepared']['private_root'] == private and receipt['prepared']['bundle'] == bundle
            and same_data(receipt['identity']['reviews']['capsule'], CONFIG['reviews']['prepare']),
            'actual one prepare0/full raw receipt/status before COPY')
    require(guard['schema'] == 'openergy.prepare-parent-once.v1'
            and guard['primary_maximum'] == guard['helper_maximum'] == 1
            and guard['guard_before_all_spawns'] is True
            and guard['identity'] == receipt['identity'], 'actual prepare once guard')
    for key, name in (('prepare_guard', 'prepare-parent-once.guard'),
                      ('prepare_stdout', 'prepare-parent-stdout.bin'),
                      ('prepare_stderr', 'prepare-parent-stderr.bin'),
                      ('prepare_status', 'prepare-parent-os.json')):
        publication = receipt['publications'][name]
        require(publication['state'] == 'durable'
                and same_data(publication['ref'], dict(path=ROOT + '/' + name, **bindings[key])),
                'actual prepare full raw publication binding')
    require(same_data(guard['clock'], CLOCK) and same_data(receipt['clock'], CLOCK)
            and guard['remote_started_ns'] == receipt['remote_started_ns']
            and guard['remote_deadline_ns'] == receipt['remote_deadline_ns']
            and primary['remote_started_ns'] == receipt['remote_started_ns']
            and primary['remote_deadline_ns'] == receipt['remote_deadline_ns']
            and type(primary['finished_ns']) is int
            and primary['actual_exit_observed_ns'] <= primary['finished_ns'] < receipt['remote_deadline_ns']
            and receipt['helper']['actual_collection']['actual_exit'] == 0
            and collection_complete(receipt['helper']['actual_collection']), 'prepare actual original shared enclosure')
    require(prepared['source_bundle'] == artifact['source_bundle']
            and prepared['app_sha256'] == artifact['app_sha256']
            and prepared['daemon_sha256'] == artifact['daemon_sha256']
            and prepared['source_commit'] == artifact['source_commit']
            and prepared['original_commit'] == '498190feb00f51039f500ae618ac7de5fb113d86'
            and prepared['baseline'] == 'fresh-m1' and same_data(prepared['ports'], [65431, 65432])
            and type(prepared['product_invocations']) is int and prepared['product_invocations'] == 0
            and same_data(prepared['limits'], LIMIT_VALUES), 'new prepared complete fixed data')
    require(same_data(prepared['source_bindings'], dict(artifact=artifact,
        artifact_receipt_sha256=EIGHT['artifact-receipt.json']['sha256'],
        source_bindings_sha256=INITIAL_INPUTS['build-source-bindings.json']['sha256'],
        limits_sha256=INITIAL_INPUTS['limits.json']['sha256'])), 'prepared full source/artifact binding')
    require(type(prepared['bundle_tree']) is list and len(prepared['bundle_tree']) == 167,
            'prepared full167 tree; exact actual copy checks belong to caller')
    # Product environment is bound as DATA only. COPY launches with startup()'s ten keys.
    expected_env = dict(PATH='/usr/bin:/bin', LANG='C', HOME=private + '/home',
        CFFIXED_USER_HOME=private + '/home', TMPDIR=private + '/tmp/', XDG_CONFIG_HOME=private + '/home',
        XDG_CACHE_HOME=private + '/cache', MACOSX_DEPLOYMENT_TARGET='15.0', PYTHONDONTWRITEBYTECODE='1',
        PYTHONNOUSERSITE='1', OPENERGY_DISABLE_UPDATES='1', OPENERGY_ORDINARY_SOURCE_ROOT=SOURCE_ROOT,
        OPENERGY_ORDINARY_PYTHON=PRIVATE_PYTHON)
    require(same_data(prepared['environment'], expected_env), 'unchanged product environment DATA')
    return dict(raw=dict(path=ROOT + '/prepared.json', **bindings['file']), private_root=private,
                bundle=bundle, prepare_receipt=dict(path=ROOT + '/prepare-parent-receipt.json',
                    **bindings['prepare_receipt']), canonical_owner0700_sameFS=True,
                external_actual_prepare_proof=bindings['external_parent_proof'])
```

## inspect_published_artifact — ADDED

after: lines 523..535, 897 bytes, SHA256 `bcd1271d40145c9768dff1697833db1d754c737f6b63ef2ba4f8a42b2a66a6b1`.

before:

```python
# No declaration in this version.
```

after:

```python
def inspect_published_artifact(rootfd):
    raw = {n: read_leaf(rootfd, n, b) for n, b in ARTIFACT_PUBLICATION_BINDINGS.items()}
    receipt = closed_json(raw['artifact-binding-receipt.json'])
    require(receipt['schema'] == 'openergy.eight-artifact-data-binding.v1'
            and receipt['evidence_root'] == ROOT and receipt['execution'] is False
            and receipt['first_error'] is None and receipt['eight_files_state'] == 'durable_barriers_returned'
            and set(receipt['files']) == set(EIGHT), 'exact actual full artifact publisher receipt')
    for name, binding in EIGHT.items():
        require(same_data(receipt['files'][name], dict(path=ROOT + '/' + name, mode='0o600', **binding)),
                'actual eight file binding')
    artifact = inspect_artifact(rootfd)
    require(len(artifact['supporting_evidence']) == 128, 'actual full support128')
    return artifact
```

## inspect_remote — REMOVED

before: lines 284..318, 2530 bytes, SHA256 `1def34c6cbdaffd2021c2ff805c428fcef4a11d3a2ce2bebb4fe512149a4c47b`.

before:

```python
def inspect_remote(rootfd):
    # Exactly five retained remote files; local two originals stay with parent.
    raw = {name: read_leaf(rootfd, name, COMPLETED[name]) for name in (
        'qualification-source-files.json', 'source-files.stdout.bin',
        'source-files.stderr.bin', 'source-files-os-exit.json',
        'source-qualification-guard.json')}
    facts = closed_json(raw['qualification-source-files.json'])
    actual = closed_json(raw['source-files-os-exit.json'])
    guard = closed_json(raw['source-qualification-guard.json'])
    require(raw['qualification-source-files.json'] == raw['source-files.stdout.bin']
            and raw['source-files.stderr.bin'] == b'' and actual['facts'] == facts,
            'original raw/facts agreement')
    expected = dict(schema='openergy.file-qualification-observation.v1', mode='source',
                    prefix_sha256='c46c9234c07785e90b66774ca161f20c7ef72c0a87323f52583fa4edb1186e67',
                    prefix_entries=2006, prefix_regular_bytes=46504040,
                    prefix_root_mode='0o700', modules=10, source_files=702,
                    bundle_records=167, sdk_headers=10, product_invocations=0,
                    runtime='UNVERIFIED', elapsed_seconds=0.4378145)
    require(facts == expected, 'original file-only facts')
    require(type(actual['actual_helper_exit']) is int and actual['actual_helper_exit'] == 0
            and actual['eof'] == {'stdout': True, 'stderr': True}
            and actual['first_error'] is None and actual['local_helper_kill_called'] is False
            and actual['before_deadline'] is True and actual['own_nice'] == 15
            and actual['product_invocations'] == 0
            and actual['status'] == 'SOURCE_FILES_QUALIFIED_RUNTIME_UNVERIFIED', 'original helper status')
    require(guard['one_attempt'] is True and guard['argv'] == [SYSTEM, '-I', '-S', '-B',
            ROOT + '/file-qualification.recipe.py.txt', ROOT, 'source']
            and guard['started_monotonic'] == actual['started_monotonic']
            and guard['deadline_monotonic'] == actual['deadline_monotonic'], 'completed guard/root/vector')
    old_start, old_end, old_exit = (actual[k] for k in (
        'started_monotonic', 'deadline_monotonic', 'exit_observed_monotonic'))
    require(all(type(v) in (int, float) and math.isfinite(v)
                for v in (old_start, old_end, old_exit))
            and old_end == old_start + 120 and old_start <= old_exit <= old_end,
            'original same-process relative deadline only')
```

after:

```python
# No declaration in this version.
```

## inspect_saved_source — ADDED

after: lines 500..520, 1647 bytes, SHA256 `719b8892f8a9b0eac880712c5b5ca3f882d191726a6fa64b3e8bb6dd48ffb672`.

before:

```python
# No declaration in this version.
```

after:

```python
def inspect_saved_source(rootfd):
    # This reauthenticates fixed saved bytes. No SOURCE/native/readiness command runs.
    result = closed_json(read_leaf(rootfd, 'metadata-source-result.json', SOURCE_RESULT))
    require(result['schema'] == 'openergy.metadata-observations.v2' and result['mode'] == 'source'
            and same_data(result['clock'], CLOCK) and result['completed_native_commands'] == 50
            and len(result['native_records']) == 50 and len(result['auxiliary_records']) == 3
            and len(result['actual_direct_children']) == len(result['collected_commands']) == 53
            and result['observation_sequence_complete'] is True and result['first_error'] is None
            and result['first_failed_command'] is None and result['first_failed_comparison'] is None
            and result['publication_errors'] == [], 'saved SOURCE53 full result')
    for row in SOURCE_RAWS:
        read_leaf(rootfd, row['leaf'], row)
    boundary = {n: read_leaf(rootfd, n, b) for n, b in SOURCE_BOUNDARY_BINDINGS.items()}
    actual = closed_json(boundary['source-caller-os.json'])
    compact = closed_json(boundary['source-caller.stdout.bin'])
    require(collection_complete(actual) and boundary['source-caller.stderr.bin'] == b''
            and compact['result'] == dict(path=ROOT + '/metadata-source-result.json', **SOURCE_RESULT)
            and actual['actual_exit_observed_ns'] < result['phase_deadline_ns'], 'saved SOURCE actual caller0/EOF')
    # Full actual SOURCE semantics, all53/50 comparisons and original seven-file
    # causal proof were already inspected by the EXTERNAL parent before COPY.
    return result
```

## inventory — REMOVED

before: lines 329..364, 2065 bytes, SHA256 `12c46f1c5b191b7fdd68455b26c33fc259c4119b9b3b5b9647a93a5737ebb22f`.

before:

```python
def inventory(rootfd):
    # Finite complete count below ROOT only. No followed links; no output listing.
    count, total = 0, 0
    def walk(fd, depth):
        nonlocal count, total
        require(depth <= DEPTH_CAP, 'initial directory depth cap')
        before = os.fstat(fd)
        require(stat.S_ISDIR(before.st_mode) and before.st_uid == os.getuid()
                and stat.S_IMODE(before.st_mode) == 0o700, 'initial directory owner/mode')
        with os.scandir(fd) as entries:
            for entry in entries:
                count += 1
                require(count <= ENTRY_CAP and len(entry.name.encode('utf-8')) <= 255,
                        'initial entry/name cap')
                require(entry.name not in ('bytes.json', 'bytes.lock'), 'SOURCE before ledger')
                info = entry.stat(follow_symlinks=False)
                require(info.st_uid == os.getuid() and info.st_dev == before.st_dev,
                        'initial owner/device')
                if stat.S_ISDIR(info.st_mode):
                    child = os.open(entry.name, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=fd)
                    try:
                        require(stamp(info) == stamp(os.fstat(child)), 'linked directory')
                        walk(child, depth + 1)
                        require(stamp(info) == stamp(os.stat(entry.name, dir_fd=fd, follow_symlinks=False)),
                                'stable linked directory')
                    finally:
                        os.close(child)
                else:
                    require(stat.S_ISREG(info.st_mode) and stat.S_IMODE(info.st_mode) == 0o600
                            and info.st_nlink == 1 and info.st_size <= FILE_CAP,
                            'initial regular file/mode/cap')
                    total += info.st_size
                    require(total <= INITIAL_CAP, 'initial aggregate reservation')
        require(stamp(before) == stamp(os.fstat(fd)), 'no concurrent directory mutation')
    walk(rootfd, 0)
    return dict(entries=count, bytes=total)
```

after:

```python
# No declaration in this version.
```

## inventory.walk — REMOVED

before: lines 332..362, 1871 bytes, SHA256 `14509512ab33bd3066c48331714922123e25b543cae89ca20eb2ed0ffe8845aa`.

before:

```python
    def walk(fd, depth):
        nonlocal count, total
        require(depth <= DEPTH_CAP, 'initial directory depth cap')
        before = os.fstat(fd)
        require(stat.S_ISDIR(before.st_mode) and before.st_uid == os.getuid()
                and stat.S_IMODE(before.st_mode) == 0o700, 'initial directory owner/mode')
        with os.scandir(fd) as entries:
            for entry in entries:
                count += 1
                require(count <= ENTRY_CAP and len(entry.name.encode('utf-8')) <= 255,
                        'initial entry/name cap')
                require(entry.name not in ('bytes.json', 'bytes.lock'), 'SOURCE before ledger')
                info = entry.stat(follow_symlinks=False)
                require(info.st_uid == os.getuid() and info.st_dev == before.st_dev,
                        'initial owner/device')
                if stat.S_ISDIR(info.st_mode):
                    child = os.open(entry.name, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=fd)
                    try:
                        require(stamp(info) == stamp(os.fstat(child)), 'linked directory')
                        walk(child, depth + 1)
                        require(stamp(info) == stamp(os.stat(entry.name, dir_fd=fd, follow_symlinks=False)),
                                'stable linked directory')
                    finally:
                        os.close(child)
                else:
                    require(stat.S_ISREG(info.st_mode) and stat.S_IMODE(info.st_mode) == 0o600
                            and info.st_nlink == 1 and info.st_size <= FILE_CAP,
                            'initial regular file/mode/cap')
                    total += info.st_size
                    require(total <= INITIAL_CAP, 'initial aggregate reservation')
        require(stamp(before) == stamp(os.fstat(fd)), 'no concurrent directory mutation')
```

after:

```python
# No declaration in this version.
```

## latch — UNCHANGED

before: lines 423..425, 145 bytes, SHA256 `1fc81f469b2139c30fcccfcc0ba115eec20e8deb4cd190d363f56bf31fe80640`.

after: lines 194..196, 145 bytes, SHA256 `1fc81f469b2139c30fcccfcc0ba115eec20e8deb4cd190d363f56bf31fe80640`.

## main — CHANGED

before: lines 574..648, 4550 bytes, SHA256 `eb9765c9bbde8701bc4398617fbf1d24707f1d7e24e797bc9e24e2f38f726b73`.

after: lines 1085..1233, 9976 bytes, SHA256 `28b76646b2851ba7068e61b98519fefd052cdee329ba226e93f3e5b7cf67a2a3`.

before:

```python
def main():
    rootfd, end, status = None, None, None
    first_error, publications, publication_errors = None, {}, []
    identity, parent_sha, initial_tally, started = None, None, None, None
    try:
        require(sys.argv == ['-'] and sys.flags.isolated and sys.flags.no_site
                and sys.flags.dont_write_bytecode and not sys.flags.optimize,
                'fixed SYSTEM stdin flags')
        os.umask(0o077)
        identity = bind_config()  # All nulls fail BEFORE private path reads/writes.
        rootfd = open_root()
        inspect_remote(rootfd)
        inspect_initial(rootfd)
        initial_tally = inventory(rootfd)  # First complete count BEFORE any write.
        inspected = clock_ns()  # NEW reading AFTER remote predicates and real-parent prerequisite.
        started = clock_ns()
        end = started + 120 * NS
        parent_bytes = encoded(parent_record(inspected, started, end))
        parent_sha = sha(parent_bytes)
        publication(rootfd, 'metadata-source-parent.json', parent_bytes, end,
                    publications, publication_errors)
        publication(rootfd, 'source-launch-once.json', encoded(dict(
            schema='openergy.source-launch-once.v1', identity=identity,
            phase_started_ns=started, phase_deadline_ns=end, clock=CLOCK,
            parent_record_sha256=parent_sha, caller_attempts_maximum=1)),
            end, publications, publication_errors)
        status, stdout, stderr = collect(started, end, parent_sha)
        # No writer overlaps caller writes. Null reap means no safe re-account;
        # keep unknown status and do not publish files against a stale sum.
        require(status['reaped'] is True, 'unreaped caller: no subsequent file writes')
        publication(rootfd, 'source-caller.stdout.bin', stdout, end,
                    publications, publication_errors)
        publication(rootfd, 'source-caller.stderr.bin', stderr, end,
                    publications, publication_errors)
        publication(rootfd, 'source-caller-os.json', encoded(status), end,
                    publications, publication_errors)
        receipt = dict(schema='openergy.source-launch-receipt.v1', identity=identity,
                       parent_record_sha256=parent_sha, collection=status,
                       raw_stdout=publications['source-caller.stdout.bin']['ref'],
                       raw_stderr=publications['source-caller.stderr.bin']['ref'],
                       collection_complete=collection_complete(status),
                       artifact_gate_authority=False, runtime='UNVERIFIED')
        publication(rootfd, 'source-launch-receipt.json', encoded(receipt), end,
                    publications, publication_errors)
    except BaseException as exc:
        first_error = dict(stage='launcher', error=type(exc).__name__)
    # Keep the first actual collection error distinct from subsequent write errors.
    if status is not None and status['first_error'] is not None:
        first_error = status['first_error']
    envelope = dict(schema='openergy.source-launch-envelope.v1', identity=identity,
                    clock=CLOCK, evidence_root=ROOT, own_nice=OWN_NICE,
                    phase_started_ns=started, phase_deadline_ns=end,
                    parent_record_sha256=parent_sha, initial_tally=initial_tally,
                    collection=status, first_error=first_error,
                    publications=publications, publication_errors=publication_errors,
                    source_collection_complete=(collection_complete(status)
                        and first_error is None and not publication_errors
                        and all(publications.get(name, {}).get('state') == 'durable' for name in OUTPUTS)),
                    artifact_gate_authority=False, caller_result_parsed=False,
                    raw_53_commands_validated=False, runtime='UNVERIFIED')
    if rootfd is not None:
        try:
            os.close(rootfd)
        except BaseException as exc:
            envelope['source_collection_complete'] = False
            if envelope['first_error'] is None:
                envelope['first_error'] = dict(stage='rootfd_close', error=type(exc).__name__)
            publication_errors.append(dict(name='rootfd', error=type(exc).__name__, stage='close'))
    try:
        require(end is not None, 'no admitted phase; no console allowance')
        emit_once(envelope, end)
        remaining(end)
    except BaseException:
        return 1
    return 0 if envelope['source_collection_complete'] else 1
```

after:

```python
def main():
    global CONFIG, STAGE
    rootfd, common, identity, console = None, None, None, None
    source, artifact, prepared, auth = None, None, None, None
    helper, caller, copy_proof, ledger, receipt_ref = None, None, None, None, None
    parent_sha, primary_attempted, complete = None, False, False
    try:
        require(os.getuid() == os.geteuid(), 'actual real/effective owner')
        with until(COPY_END):
            CONFIG, identity = bind_config()  # All future nulls fail BEFORE ROOT/private reads.
            console = console_admission()  # BEFORE ANY target growth.
            rootfd = open_root()
            STAGE = 'postledger-frozen-admission'
            inspect_fixed_inputs(rootfd)
            source = inspect_saved_source(rootfd)
            artifact = inspect_published_artifact(rootfd)
            prepared = inspect_new_prepared(rootfd, artifact)
            ledger = existing_ledger(rootfd)  # Must ALREADY exist. Never initialize/recover.
            for name in OUTPUTS + ['metadata-copy-once.json', 'metadata-copy-parent.json',
                    'metadata-copy-parent-entry.stdout', 'metadata-copy-parent-entry.stderr',
                    'metadata-copy-result.json', 'metadata-copy-first-failure.json']:
                absent(rootfd, name)
            vectors = closed_json(read_leaf(rootfd, 'command-vectors.json',
                                           INITIAL_INPUTS['command-vectors.json']))['commands']
            for label in ['copy-files-before'] + [v['label'] for v in vectors] + ['copy-files-after']:
                for suffix in ('.stdout', '.stderr', '-os.json', '-comparison.json'):
                    absent(rootfd, 'metadata-copy-' + label + suffix)
            STAGE = 'SYSTEM-pinned-common-before-prefix-helper'
            common = import_system_common(rootfd)
        STAGE = 'durable-once-guard-before-every-spawn'
        guard = dict(schema='openergy.copy-parent-once.v1', identity=identity, clock=CLOCK,
            phase_started_ns=COPY_START, phase_deadline_ns=COPY_END, own_nice=OWN_NICE,
            helper_maximum=1, private_COPY_maximum=1, guard_before_all_spawns=True,
            prepared=prepared, original_local135_and_once_guard='EXTERNAL_PARENT_MANDATORY',
            execution=False, runtime='UNVERIFIED')
        reserved_put(rootfd, common, OUTPUTS[0], encoded(guard))
        STAGE = 'one-SYSTEM-entry-helper'
        helper, stdout, stderr = collect('entry')
        require(helper['reaped'] is True, 'helper not actually reaped; no subsequent target write')
        reserved_put(rootfd, common, OUTPUTS[1], stdout)
        reserved_put(rootfd, common, OUTPUTS[2], stderr)
        require(len(encoded(helper)) <= STATUS_CAP, 'bounded helper full status')
        reserved_put(rootfd, common, OUTPUTS[3], encoded(helper))
        with until(COPY_END):
            auth = inspect_entry(helper, stdout, stderr)
            # New auth timestamp follows actual/raw checks, before private entry.
            parent_bytes = copy_parent_stdin(auth)
            parent_sha = sha(parent_bytes)
        reserved_put(rootfd, common, OUTPUTS[4], parent_bytes)
        with until(COPY_END):
            require(FIRST_ERROR is None and auth['parent_inspected_ns'] <= clock_ns() < COPY_END,
                    'failed helper never authenticates private entry')
            for name, binding in PINS.items():
                read_leaf(rootfd, name, binding)
            read_leaf(rootfd, 'metadata-caller.py.txt', FIXED_BINDINGS['metadata_caller']['source'])
            read_leaf(rootfd, 'prepared.json', CONFIG['prepared']['file'])
            reject_entry_caches(rootfd)
            existing_ledger(rootfd)
            absent(rootfd, 'metadata-copy-once.json')
            absent(rootfd, 'prepared-copy-qualification.json')
        STAGE = 'one-private-COPY-entry'
        primary_attempted = True  # Includes spawn failure; never reopen/retry.
        caller, stdout, stderr = collect('copy', parent_bytes)
        require(caller['reaped'] is True and type(caller['actual_exit']) is int,
                'private caller not actually reaped; no parent writes or receipt')
        STAGE = 'postreap-R9-raw-publication'
        reserved_put(rootfd, common, OUTPUTS[5], stdout)
        reserved_put(rootfd, common, OUTPUTS[6], stderr)
        require(len(encoded(caller)) <= STATUS_CAP, 'bounded caller full status')
        reserved_put(rootfd, common, OUTPUTS[7], encoded(caller))
        STAGE = 'full52-actual-record-gate'
        with until(COPY_END):
            require(FIRST_ERROR is None, 'first actual caller failure blocks copy gate')
            copy_proof = inspect_copy_result(rootfd, caller, stdout, stderr, prepared, source)
            # Bind caller's OWN durable auth/once records through existing writer.
            raw, _ = read_new_leaf(rootfd, 'metadata-copy-once.json', STATUS_CAP)
            once = closed_json(raw)
            require(same_data(once, dict(mode='copy', clock=CLOCK, started_ns=COPY_START,
                phase_deadline_ns=COPY_END, parent_record_sha256=parent_sha, execution_attempts=1)),
                'caller exact original once/start/parent binding')
            raw, _ = read_new_leaf(rootfd, 'metadata-copy-parent.json', RAW_CAP)
            require(raw == parent_bytes, 'caller retained exact canonical parent stdin')
            for name, raw in (('stdout', base64.b64decode(auth['stdout'], validate=True)),
                              ('stderr', base64.b64decode(auth['stderr'], validate=True))):
                actual, _ = read_new_leaf(rootfd, 'metadata-copy-parent-entry.' + name, RAW_CAP)
                require(actual == raw, 'caller raw parent authentication')
            read_leaf(rootfd, 'prepared.json', CONFIG['prepared']['file'])
            require(os.getpriority(os.PRIO_PROCESS, 0) == 15, 'own final NI15 observation')
            complete = True
    except BaseException as exc:
        fail(STAGE, exc)
        complete = False
    # One full receipt attempt. Missing actual reap forbids ALL postchild writes.
    safe_to_publish = (common is not None and rootfd is not None
        and all(e['handle'] is None or e['status']['reaped'] is True for e in HANDLES))
    if safe_to_publish:
        try:
            STAGE = 'full-prepared-copy-qualification-receipt'
            with until(COPY_END):
                receipt = dict(schema='openergy.prepared-copy-qualification.v1', execution=False,
                    state=('COPY_DATA_COMPLETE_EXTERNAL_PENDING' if complete and FIRST_ERROR is None
                           and not PUBLICATION_ERRORS else 'FAILED_UNVERIFIED'),
                    evidence_root=ROOT, identity=identity, clock=CLOCK,
                    phase_started_ns=COPY_START, phase_deadline_ns=COPY_END,
                    artifact_receipt=dict(path=ROOT + '/artifact-receipt.json', **EIGHT['artifact-receipt.json']),
                    artifact_operation=dict(path=ROOT + '/operation.bound.json', **EIGHT['operation.bound.json']),
                    artifact_publication=ARTIFACT_PUBLICATION_BINDINGS, artifact_eight_raws=EIGHT,
                    source_result=dict(path=ROOT + '/metadata-source-result.json', **SOURCE_RESULT),
                    source_raw_manifest_sha256=sha(encoded(SOURCE_RAWS)), source_boundary=SOURCE_BOUNDARY_BINDINGS,
                    prepared=prepared, helper_collection=helper, entry_authentication=auth,
                    caller_collection=caller, caller_attempted=primary_attempted,
                    parent_stdin_sha256=parent_sha, copy_gate_proof=copy_proof,
                    publications=dict(PUBLICATIONS), first_error=FIRST_ERROR, publication_errors=list(PUBLICATION_ERRORS),
                    common_clock_conversions_before_receipt=list(CONVERSIONS),
                    receipt_conversion='same conservative common.time.monotonic conversion; SAME original120 alarm',
                    existing_ledger_before_parent_growth=ledger, frozen17=PINS,
                    caps=dict(file=FILE_CAP, total=TOTAL_CAP, combined_each_child=RAW_CAP,
                        console=CONSOLE_CAP, receipt=RECEIPT_CAP, product_reserve=16777216,
                        cleanup_reserve=4194304, accounting_reserve=65536), own_nice=OWN_NICE,
                    console_admission=console, private_COPY_startup='UNMEASURED',
                    external_parent=dict(actual_SSH_exit=None, actual_collector_exit=None, both_EOF=None,
                        D4_actual_exit_before_original135_and_before_receipt_reads=None,
                        all_full_raw_receipt_and52_records_retained=None,
                        full_copy_signature_entitlement_operator_assessment=None,
                        own_receipt_final_barriers_rootclose_console_exit='ACTUAL_EXTERNAL_PARENT_DUTY'),
                    controller_operator_gate='BLOCKED_UNTIL_EXTERNAL_ACTUAL_PARENT_ASSESSMENT',
                    controller_started=False, runtime_acceptance=False, runtime='UNVERIFIED')
                require(len(encoded(receipt)) <= RECEIPT_CAP, 'full receipt bounded before writer admission')
            receipt_ref = reserved_put(rootfd, common, OUTPUTS[8], receipt)
        except BaseException as exc:
            fail(STAGE, exc)
            complete = False
    else:
        complete = False  # Raw prefixes/status remain in retained memory; missing publication is UNKNOWN.
    if rootfd is not None:
        try:
            closing, rootfd = rootfd, None
            with until(COPY_END):
                os.close(closing)
        except BaseException as exc:
            fail('root-close', exc)
            complete = False
    try:
        require(CONFIG is not None and console is not None, 'no admitted console')
        emit_once(receipt_ref, complete and FIRST_ERROR is None and not PUBLICATION_ERRORS)
        remaining(COPY_END)
    except BaseException:
        return 1
    return 0 if complete and FIRST_ERROR is None and not PUBLICATION_ERRORS else 1
```

## normalize — ADDED

after: lines 943..946, 286 bytes, SHA256 `7d2ae2796e6f3e82dc4906c6dc5b45d70c40d387b0fad1b215300826c2fcd5bd`.

before:

```python
# No declaration in this version.
```

after:

```python
def normalize(raw, bundle, label):
    # Exact declared caller comparison; no native-output parser or repair.
    value = raw.decode('utf-8', errors='strict').replace(bundle, ARTIFACT_SHAPE['source_bundle'])
    return sorted(value.splitlines()) if label.endswith('-verify') else value
```

## observe_exit — UNCHANGED

before: lines 428..435, 438 bytes, SHA256 `94a6fafecbef10b6509533a0d792cce1fc61b4fcb99d9183118da1916b83610c`.

after: lines 200..207, 438 bytes, SHA256 `94a6fafecbef10b6509533a0d792cce1fc61b4fcb99d9183118da1916b83610c`.

## open_root — UNCHANGED

before: lines 243..259, 781 bytes, SHA256 `c7aee36ba2ba05b59397e13a30160ffc77a94a2360d705ff22a54dfe69ae163f`.

after: lines 142..158, 781 bytes, SHA256 `c7aee36ba2ba05b59397e13a30160ffc77a94a2360d705ff22a54dfe69ae163f`.

## parent_record — REMOVED

before: lines 367..375, 762 bytes, SHA256 `50dbcadcd9a796e49b2544dc61ddebe5ce930aeb93983c0e81741cfbdf5017f8`.

before:

```python
def parent_record(inspected, started, end):
    return dict(schema='openergy.metadata-parent-boundary.v2', mode='source', clock=CLOCK,
                phase_started_ns=started, phase_deadline_ns=end, entry_authentication=None,
                source_file_qualification=dict(schema='openergy.completed-source-files.v1',
                    saved_audit_commit=AUDIT, retained_files=COMPLETED,
                    parent_inspection=dict(schema='openergy.source-files-parent-inspection.v1',
                        clock=CLOCK, inspected_ns=inspected, evidence_root=ROOT,
                        manifest_sha256=MANIFEST_HASH, actual_helper_exit=0, actual_ssh_exit=0,
                        inspected='completed-facts-actual-exits-and-guards-before-phase-start')))
```

after:

```python
# No declaration in this version.
```

## publication — REMOVED

before: lines 378..412, 1662 bytes, SHA256 `60573f6a350d965fdbba4282843a033c43c3f27115e8182c097e42b64155f44b`.

before:

```python
def publication(rootfd, name, data, end, publications, errors):
    # Only call with exclusive temporal ownership: before launch or AFTER reap.
    require(name in OUTPUTS, 'closed output leaf')
    item = dict(path=ROOT + '/' + name, state='attempted', ref=None)
    publications[name] = item
    fd = None
    try:
        with until(end):
            tally = inventory(rootfd)  # Re-account caller-created files, EVERY later write.
            require(len(data) <= FILE_CAP and tally['bytes'] + len(data) <= INITIAL_CAP,
                    'pre-write INITIAL reservation')
            fd = os.open(name, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
                         0o600, dir_fd=rootfd)
            os.fchmod(fd, 0o600)
            require(os.write(fd, data) == len(data), 'short exclusive single write')
            os.fsync(fd)
            closing = fd
            fd = None
            os.close(closing)  # Do not retry a close whose result is uncertain.
            os.fsync(rootfd)
        item.update(state='durable', ref=dict(path=ROOT + '/' + name,
                                             bytes=len(data), sha256=sha(data)))
        return item['ref']
    except BaseException as exc:
        item['state'] = 'failed_or_partial_unverified'
        errors.append(dict(name=name, error=type(exc).__name__))
        raise
    finally:
        if fd is not None:
            closing = fd
            fd = None
            try:
                os.close(closing)  # Raw fd close has no pending Python payload.
            except OSError as exc:
                errors.append(dict(name=name, error=type(exc).__name__, stage='close'))
```

after:

```python
# No declaration in this version.
```

## raw_binding — ADDED

after: lines 398..401, 261 bytes, SHA256 `11e7f63d64829fef9c8d597c51d3f9dae26c4f6e2184f2fc548d94476f901fd1`.

before:

```python
# No declaration in this version.
```

after:

```python
def raw_binding(value, cap=FILE_CAP):
    require(type(value) is dict and set(value) == {'bytes', 'sha256'}
            and type(value['bytes']) is int and 0 <= value['bytes'] <= cap
            and hex_string(value['sha256'], 64), 'closed actual raw binding')
```

## read_leaf — UNCHANGED

before: lines 262..281, 1057 bytes, SHA256 `963beb17880dcd6f322370c6df7a7d2a0c1cf573b049cbcf990faf02e4af389f`.

after: lines 162..181, 1057 bytes, SHA256 `963beb17880dcd6f322370c6df7a7d2a0c1cf573b049cbcf990faf02e4af389f`.

## read_new_leaf — ADDED

after: lines 272..291, 1063 bytes, SHA256 `2b9c6970ae56804639e26a755b436528d84ae41d4ae0df09f3cea42d86b5b46c`.

before:

```python
# No declaration in this version.
```

after:

```python
def read_new_leaf(rootfd, name, cap):
    # New fixed output only; raw bytes retained before interpretation.
    require('/' not in name and name not in ('', '.', '..'), 'output leaf')
    fd = os.open(name, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=rootfd)
    try:
        info = os.fstat(fd)
        require(stat.S_ISREG(info.st_mode) and info.st_uid == os.getuid()
                and stat.S_IMODE(info.st_mode) == 0o600 and info.st_nlink == 1
                and 0 <= info.st_size <= cap, 'new output metadata/cap')
        data = bytearray()
        while len(data) <= info.st_size:
            chunk = os.read(fd, min(65536, info.st_size + 1 - len(data)))
            if not chunk:
                break
            data.extend(chunk)
        require(len(data) == info.st_size and stamp(info) == stamp(os.fstat(fd))
                == stamp(os.stat(name, dir_fd=rootfd, follow_symlinks=False)), 'stable new raw output')
        return bytes(data), dict(path=ROOT + '/' + name, bytes=len(data), sha256=sha(data))
    finally:
        os.close(fd)
```

## read_record_ref — ADDED

after: lines 937..940, 178 bytes, SHA256 `77169d0938776edb16e31ebe1156b761882ce5452b1a861a4e8b6fcc2b63d2d0`.

before:

```python
# No declaration in this version.
```

after:

```python
def read_record_ref(rootfd, record, leaf):
    reference(record)
    require(record['path'] == ROOT + '/' + leaf, 'exact R9 raw leaf')
    return read_leaf(rootfd, leaf, record)
```

## reference — ADDED

after: lines 295..300, 424 bytes, SHA256 `fd11ab41a612c579f21d048250603a916429d50a7022e540a71a0bb87f698fe3`.

before:

```python
# No declaration in this version.
```

after:

```python
def reference(record):
    require(type(record) is dict and set(record) == {'path', 'bytes', 'sha256'}
            and type(record['path']) is str and record['path'].startswith('/')
            and '..' not in Path(record['path']).parts and '${' not in record['path']
            and type(record['bytes']) is int and 0 <= record['bytes'] <= FILE_CAP
            and hex_string(record['sha256'], 64), 'closed raw reference')
```

## reject_entry_caches — ADDED

after: lines 668..688, 1226 bytes, SHA256 `e90e87064f00f9ec2420ae6a131b764331baf9077ee4207a60567d374f446116`.

before:

```python
# No declaration in this version.
```

after:

```python
def reject_entry_caches(rootfd):
    # Fixed ROOT is flat data plus the ten pinned .py siblings. No package,
    # extension or bytecode candidate may shadow those sources or stdlib imports.
    absent(rootfd, '__pycache__')
    for name in (n for n in PINS if n.endswith('.py')):
        absent(rootfd, name[:-3])  # Package alternative, including a symlink.
        absent(rootfd, name[:-3] + '.pyc')
        absent(rootfd, name[:-3] + '.pyo')
    count = 0
    with os.scandir(rootfd) as entries:
        for entry in entries:
            count += 1
            require(count <= 4096 and len(entry.name.encode('utf-8')) <= 255, 'bounded ROOT cache admission')
            require(not entry.name.endswith(('.pyc', '.pyo', '.so')),
                    'no compatible legacy/cache/extension import candidate')
            require(not entry.name.endswith('.py') or entry.name in PINS,
                    'only ten pinned Python source siblings')
            require(not stat.S_ISDIR(entry.stat(follow_symlinks=False).st_mode)
                    and not stat.S_ISLNK(entry.stat(follow_symlinks=False).st_mode),
                    'flat ROOT; no package/cache directory or symlink import alternative')
    remaining(COPY_END)
```

## remaining — UNCHANGED

before: lines 142..145, 142 bytes, SHA256 `b723e8885f1273197aa575682bee963f2cefbec7b0df8c8b6a051a0e26ac7c78`.

after: lines 100..103, 142 bytes, SHA256 `b723e8885f1273197aa575682bee963f2cefbec7b0df8c8b6a051a0e26ac7c78`.

## require — UNCHANGED

before: lines 129..131, 65 bytes, SHA256 `fa57bb7c8de0a6f63c620d5d56849a58849c546317137baa3463c4ce8f7d4766`.

after: lines 85..87, 65 bytes, SHA256 `fa57bb7c8de0a6f63c620d5d56849a58849c546317137baa3463c4ce8f7d4766`.

## reserved_put — ADDED

after: lines 691..722, 1647 bytes, SHA256 `9fc941564a1ab50756884c0b7966df0d67c90362ff807b955edfd3bd0744d33c`.

before:

```python
# No declaration in this version.
```

after:

```python
def reserved_put(rootfd, common, name, data):
    require(name in OUTPUTS and (type(data) is bytes and len(data) <= FILE_CAP
            or name == OUTPUTS[8] and type(data) is dict), 'closed reserved output')
    require(all(e['handle'] is None or e['status']['reaped'] is True for e in HANDLES),
            'no publication while a direct child is live/unknown')
    item = dict(path=ROOT + '/' + name, state='attempted', ref=None)
    PUBLICATIONS[name] = item
    try:
        with until(COPY_END):
            existing_ledger(rootfd)
            absent(rootfd, name)
            end = common_deadline(common, COPY_END, CONVERSIONS)
            if type(data) is dict:
                data['common_clock_conversions_including_receipt'] = list(CONVERSIONS)
                data = encoded(data)
                require(len(data) <= RECEIPT_CAP, 'full receipt including own conversion cap')
            common.write_bytes(Path(ROOT) / name, data, end, critical=False)
            fd = os.open(name, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=rootfd)
            try:
                os.fsync(fd)
            finally:
                os.close(fd)
            os.fsync(rootfd)
            read_leaf(rootfd, name, dict(bytes=len(data), sha256=sha(data)))
            remaining(COPY_END)
        item.update(state='durable', ref=dict(path=ROOT + '/' + name, bytes=len(data), sha256=sha(data)))
        return item['ref']
    except BaseException as exc:
        fail('reserved-publication', exc)
        item['state'] = 'failed_or_partial_unverified'
        PUBLICATION_ERRORS.append(dict(name=name, error=type(exc).__name__))
        raise
```

## same_data — UNCHANGED

before: lines 191..199, 427 bytes, SHA256 `b03e25037d583929f50a0cc97c4069c4e211552782a8f14037cba370a49734fe`.

after: lines 124..132, 427 bytes, SHA256 `b03e25037d583929f50a0cc97c4069c4e211552782a8f14037cba370a49734fe`.

## sha — UNCHANGED

before: lines 167..168, 59 bytes, SHA256 `4e627faf4b903521e935f7c6f3ae93545856f821d0e2565ba7ea6cf97d3d0e6c`.

after: lines 107..108, 59 bytes, SHA256 `4e627faf4b903521e935f7c6f3ae93545856f821d0e2565ba7ea6cf97d3d0e6c`.

## stamp — UNCHANGED

before: lines 238..240, 152 bytes, SHA256 `5ad54262f0589d8245e2a4e1d0fc970a2264bcd8ad3c9eee89e66022b017e1c0`.

after: lines 136..138, 152 bytes, SHA256 `5ad54262f0589d8245e2a4e1d0fc970a2264bcd8ad3c9eee89e66022b017e1c0`.

## startup — UNCHANGED

before: lines 415..420, 420 bytes, SHA256 `77f6e5efc6f48392d70f305e7f8be86b6d20f82574df1cff818d28389c088bd9`.

after: lines 185..190, 420 bytes, SHA256 `77f6e5efc6f48392d70f305e7f8be86b6d20f82574df1cff818d28389c088bd9`.

## system_bootstrap — ADDED

after: lines 610..636, 1695 bytes, SHA256 `501eda3206125328ee1f0f9bacecba63c04edb97c92346ad19b1539e3256a183`.

before:

```python
# No declaration in this version.
```

after:

```python
def system_bootstrap(common=None):
    require(sys.argv == ['-'] and sys.version_info[:3] == (3, 9, 6)
            and sys.flags.isolated and sys.flags.ignore_environment and sys.flags.no_site
            and sys.flags.dont_write_bytecode and not sys.flags.optimize, 'exact SYSTEM3.9.6 -I-S-B')
    require(sys.pycache_prefix is None, 'no redirected SYSTEM cache prefix')
    require(str(Path(sys.base_prefix).resolve(strict=True)) == SYSTEM_BASE
            and str(Path(sys.prefix).resolve(strict=True)) == SYSTEM_BASE,
            'fixed trusted SYSTEM standard-library prefix')
    for entry in sys.path:
        require(type(entry) is str and entry.startswith(SYSTEM_BASE + '/')
                and '/site-packages' not in entry and '/dist-packages' not in entry,
                'SYSTEM-only import search; never ROOT/private prefix/source')
    for name, module in list(sys.modules.items()):
        if module is None:
            continue
        origin = getattr(getattr(module, '__spec__', None), 'origin', None)
        if origin in (None, 'built-in', 'frozen'):
            continue
        if name == 'common' and common is module:
            require(origin == ROOT + '/common.py', 'only exact common exception')
        else:
            require(type(origin) is str and origin.startswith(SYSTEM_BASE + '/')
                    and '/site-packages/' not in origin and '/dist-packages/' not in origin,
                    'transitive SYSTEM module origin: ' + name)
    require(os.environ.get('OPENERGY_ORDINARY_SOURCE_ROOT') == SOURCE_ROOT
            and os.environ.get('OPENERGY_ORDINARY_PYTHON') == PRIVATE_PYTHON,
            'two fixed ENV bindings before common evaluation')
```

## until — CHANGED

before: lines 148..164, 666 bytes, SHA256 `00ff5b79c8d3acd8bbcd3e298fb472fab7427f91611e93116c199224033746ff`.

after: lines 224..242, 886 bytes, SHA256 `64be8d58aeb7edc2c04ebaaaae0ce4b8708ea7b715c866829b02c0f2bc690f54`.

before:

```python
@contextlib.contextmanager
def until(end):
    # Absolute ceilings only. No restoration of an expired budget as new time.
    duration = remaining(end)
    old_handler = signal.getsignal(signal.SIGALRM)
    old_timer = signal.getitimer(signal.ITIMER_REAL)
    require(old_timer == (0.0, 0.0), 'no inherited/nested alarm')
    def expired(_signum, _frame):
        raise TimeoutError('shared absolute ceiling')
    signal.signal(signal.SIGALRM, expired)
    try:
        signal.setitimer(signal.ITIMER_REAL, duration)
        yield
        remaining(end)
    finally:
        signal.setitimer(signal.ITIMER_REAL, 0)
        signal.signal(signal.SIGALRM, old_handler)
```

after:

```python
@contextlib.contextmanager
def until(deadline):
    """Shared absolute ns deadline; nested alarm and postcheck. Parent owns outer."""
    duration = remaining(deadline)
    previous_handler = signal.getsignal(signal.SIGALRM)
    previous_timer = signal.getitimer(signal.ITIMER_REAL)
    entered = clock_ns()
    def expired(_signum, _frame):
        raise TimeoutError('absolute shared-clock deadline')
    signal.signal(signal.SIGALRM, expired)
    signal.setitimer(signal.ITIMER_REAL, min(duration, previous_timer[0]) if previous_timer[0] else duration)
    try:
        yield
        remaining(deadline)
    finally:
        signal.setitimer(signal.ITIMER_REAL, 0)
        signal.signal(signal.SIGALRM, previous_handler)
        if previous_timer[0]:
            signal.setitimer(signal.ITIMER_REAL, max(0.000001, previous_timer[0] - (clock_ns() - entered) / NS), previous_timer[1])
```

## until.expired — CHANGED

before: lines 155..156, 88 bytes, SHA256 `bc1ec102edf0eae776d4cdfa698bb0528d7764f751efecc51a5cd16bd1bd3505`.

after: lines 231..232, 95 bytes, SHA256 `902795f7baa85b941e4c1118cc87af52f7894304bf8b739fe1e86b5fbea05c15`.

before:

```python
    def expired(_signum, _frame):
        raise TimeoutError('shared absolute ceiling')
```

after:

```python
    def expired(_signum, _frame):
        raise TimeoutError('absolute shared-clock deadline')
```
