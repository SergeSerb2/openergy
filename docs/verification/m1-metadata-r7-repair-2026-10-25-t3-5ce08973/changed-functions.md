# R7 changed functions

SOURCE/DATA ONLY. execution=false. Full functions; no executable excerpts.

Only `emit_summary` and `main` change. No function is added or removed.
`main` changes only null-ref initialization, capturing the existing result save,
and passing that ref to the existing stdout call. All other main bytes agree.

## emit_summary

1457 bytes; lines632-654; SHA-256 `b057ff1f16a45846212b252bfbdfac69c58dd239f43b8a07388428b0f861dadc`.

```python
def emit_summary(result, deadline, result_ref):
    with until(deadline):
        if result['observation_sequence_complete']:
            # Only the exact return from the existing durable full-result save.
            require(result_ref is not None, 'success requires durable full result')
            receipt = dict(schema='openergy.metadata-success.v1', mode=result['mode'],
                           phase_started_ns=result['started_ns'], phase_deadline_ns=deadline,
                           result=result_ref)
            data = encoded(receipt)
            require(len(data) <= 512, 'compact success receipt byte ceiling')
            # Atomic allowance is distinct from pipe storage or reader capture.
            require(stat.S_ISFIFO(os.fstat(1).st_mode), 'success receipt requires pipe')
            pipe_buf = os.fpathconf(1, 'PC_PIPE_BUF')
            require(type(pipe_buf) is int and pipe_buf >= 512, 'success pipe atomic allowance')
        else:
            # Preserve full failed/partial status. Never turn fallback into success.
            data = encoded(result)
            require(len(data) <= STREAM_CAP, 'bounded outer fallback summary')
        # Leave stdout nonblocking. No BufferedWriter ever receives this payload.
        # One raw attempt only: EAGAIN, partial, alarm or other error ends it.
        os.set_blocking(1, False)
        remaining(deadline)
        require(os.write(1, data) == len(data), 'short outer summary')
```

## main

8135 bytes; lines657-766; SHA-256 `b761d755f6f8190128df3893d628c21e4e6648be1cc2d09a647384b93de0d236`.

```python
def main():
    require(len(sys.argv) == 5 and sys.argv[1] in ('source', 'copy'), 'fixed CLI')
    mode, root = sys.argv[1], canonical(sys.argv[2])
    require(root == Path(__file__).resolve(strict=True).parent and Path.cwd() == root, 'materialized caller cwd')
    require(root.parent == Path('/Users/m1-dev/.local/state/openergy-verification') and root.name.startswith('m1-ordinary-reviewed-flow-20261025-'), 'fresh evidence namespace')
    info = root.stat()
    require(info.st_uid == os.getuid() and stat.S_IMODE(info.st_mode) == 0o700 and info.st_dev == Path('/private/tmp').stat().st_dev, 'evidence owner/mode/filesystem')
    require(sys.argv[3].isascii() and sys.argv[3].isdigit() and len(sys.argv[3]) <= 20, 'integer shared-clock CLI')
    started = int(sys.argv[3])
    require(str(started) == sys.argv[3] and started <= clock_ns(), 'parent explicit shared-clock start')
    deadline = started + 120 * NS
    os.umask(0o077)
    writer = None
    result_ref = None
    result = dict(schema='openergy.metadata-observations.v2', clock=CLOCK, mode=mode, expected_basis_sha256=EXPECTED_HASH,
                  started_ns=started, phase_deadline_ns=deadline, native_records=[], auxiliary_records=[],
                  completed_native_commands=0, observation_sequence_complete=False, first_error=None,
                  artifact_gate_authority=False, runtime='UNVERIFIED', actual_direct_children=[],
                  first_failed_command=None, first_failed_comparison=None, collected_commands=[],
                  active_command=None, publication_errors=[])
    try:
        with until(deadline):
            normalize_startup(root, mode)
            vectors, expected = pin_inputs(root, deadline)
            if mode == 'source':
                parent_bytes = bound(root / 'metadata-source-parent.json', sys.argv[4], deadline, STREAM_CAP)
            else:
                # Parent captures entry authentication in memory. No unmetered
                # post-prepare file or second private writer entry is needed.
                parent_bytes = sys.stdin.buffer.read(STREAM_CAP + 1)
                require(len(parent_bytes) <= STREAM_CAP and sha(parent_bytes) == sys.argv[4], 'copy bounded stdin parent binding')
            parent = json.loads(parent_bytes)
            require(set(parent) == {'schema', 'mode', 'clock', 'phase_started_ns', 'phase_deadline_ns', 'source_file_qualification', 'entry_authentication'}, 'parent record shape')
            require(parent['schema'] == 'openergy.metadata-parent-boundary.v2' and parent['mode'] == mode
                    and parent['clock'] == CLOCK and type(parent['phase_started_ns']) is int
                    and type(parent['phase_deadline_ns']) is int and parent['phase_started_ns'] == started
                    and parent['phase_deadline_ns'] == deadline, 'parent phase binding')
            common = None
            if mode == 'source':
                # Parent owns exact /usr/bin/python3 launch. A system launcher
                # can report its toolchain executable in sys.executable.
                require(sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode and not sys.flags.optimize, 'source system -I -S -B flags')
                require(parent['entry_authentication'] is None, 'source authenticates entry-template itself')
                inspect_completed_source(root, parent['source_file_qualification'], started, deadline)
            else:
                require(parent['source_file_qualification'] is None, 'copy parent boundary')
                auth = parent['entry_authentication']
                parent_streams = inspect_parent_helper(root, auth, started, deadline)
                # This check binds data AFTER entry. It cannot replace the parent's pre-entry gate.
                common = import_common(root, deadline)
            writer = Writer(root, mode, deadline, common)
            writer.save('metadata-' + mode + '-once.json', dict(mode=mode, clock=CLOCK, started_ns=started, phase_deadline_ns=deadline, parent_record_sha256=sys.argv[4], execution_attempts=1))
            if mode == 'copy':
                # ALL new post-prepare evidence uses regular pre-write reservations.
                writer.put('metadata-copy-parent.json', parent_bytes)
                for name, data in parent_streams.items():
                    writer.put('metadata-copy-parent-entry.' + name, data)
            bundle = BUNDLE if mode == 'source' else copy_bundle(root, deadline)
            helper = [SYSTEM, '-I', '-S', '-B', str(root / 'file-qualification.recipe.py.txt'), str(root)]
            if mode == 'source':
                result['auxiliary_records'].append(perform(writer, 'entry-template', helper + ['entry-template'], bundle, helper_mode='entry-template'))
                # Exact first private-Python entry only AFTER the trusted helper completed.
                result['auxiliary_records'].append(perform(writer, 'python-version', [PYTHON, '-I', '-S', '-B', '--version'], bundle, exact_stdout='Python 3.13.9\n'))
                result['auxiliary_records'].append(perform(writer, 'python-architecture', ['/usr/bin/lipo', '-archs', PYTHON], bundle, exact_stdout='arm64\n'))
                fresh = [None] * 50
            else:
                fresh = fresh_source(root, deadline, vectors)
                result['auxiliary_records'].append(perform(writer, 'copy-files-before', helper + ['copy'], bundle, helper_mode='copy'))
            for vector, basis, source in zip(vectors, expected, fresh):
                result['native_records'].append(perform(writer, vector['label'], expand(vector, bundle), bundle, basis=basis, fresh=source))
                result['completed_native_commands'] += 1
            if mode == 'copy':
                result['auxiliary_records'].append(perform(writer, 'copy-files-after', helper + ['copy'], bundle, helper_mode='copy'))
            result['observation_sequence_complete'] = True
    except Exception as exc:
        result['first_error'] = type(exc).__name__ + ': ' + str(exc)[:2048]
        result['observation_sequence_complete'] = False
    if writer is not None:
        result.update(collected_commands=writer.collected_commands, active_command=writer.active_command,
                      first_failed_command=writer.first_failed_command, first_failed_comparison=writer.first_failed_comparison,
                      publication_errors=writer.publication_errors)
        if writer.first_failed_command is not None:
            result['stop_detail'] = result['first_error']
            result['first_error'] = writer.first_failed_command['label'] + ': first failed COMMAND status'
        elif writer.first_failed_comparison is not None:
            result['stop_detail'] = result['first_error']
            result['first_error'] = writer.first_failed_comparison['label'] + ': first failed status/comparison'
    # This is a pre-publication observation, never a claim about caller actual exit.
    # Parent owns actual outer exit/EOF and must include it within the SAME phase.
    try:
        with until(deadline):
            result['observations_finished_ns'] = clock_ns()
            result['actual_direct_children'] = direct_children()
            require(writer is not None, 'no admitted writer')
            result_ref = writer.save('metadata-' + mode + '-result.json', result)
    except Exception as exc:
        result['observation_sequence_complete'] = False
        result['result_publication_error'] = type(exc).__name__ + ': ' + str(exc)[:1024]
        if result['first_error'] is None:
            result['first_error'] = result['result_publication_error']
        result['actual_direct_children'] = direct_children()
    # One stdout summary attempt, no retry or late writer/extra allowance. If
    # time is gone, the parent keeps its actual handle/partial streams/exit and
    # missing facts stay missing. A partial summary is never issued a second time.
    try:
        emit_summary(result, deadline, result_ref)
        remaining(deadline)
    except Exception:
        return 1
    return 0 if result['observation_sequence_complete'] else 1
```
