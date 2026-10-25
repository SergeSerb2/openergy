# Complete R5 function changes

SOURCE/DATA ONLY. execution=false. These are source excerpts, not executed code.

## Writer._put

changed; caller lines 194-226; SHA-256 `e18f876fcafea72413cc166e25c3c22cf299ca36d0fdb62709031437856acf10`.

```python
    def _put(self, name, data):
        require(Path(name).name == name and len(data) <= FILE_CAP, 'fixed output filename/file cap')
        path = self.root / name
        with until(self.deadline):
            if self.mode == 'source':
                require(self.initial + len(data) <= INITIAL_CAP, 'initial aggregate reservation')
                # Exclusive0600 initial evidence. Prepare alone charges it later.
                self.initial += len(data)
                fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
                write_error = None
                try:
                    # One raw write. No Python buffer, short-write retry or flush.
                    require(os.write(fd, data) == len(data), 'short initial write')
                    os.fsync(fd)
                except BaseException as exc:
                    write_error = exc
                    raise
                finally:
                    # One close, including after an alarm/write/fsync failure.
                    # A raw fd owns no deferred payload. Never retry close.
                    try:
                        os.close(fd)
                    except Exception as exc:
                        if write_error is None:
                            raise
                        # Preserve the earlier failure and retain close separately.
                        self.publication_errors.append(dict(name=name, error='source close after earlier failure: '
                                                            + type(exc).__name__ + ': ' + str(exc)[:1024]))
            else:
                # Existing common charges regular capacity BEFORE temporary/final growth.
                self.common.write_bytes(path, data, common_deadline(self.deadline), critical=False)
            sync_file(path, self.deadline)
        return dict(path=str(path), bytes=len(data), sha256=sha(data))
```

## emit_summary

changed; caller lines 632-640; SHA-256 `a45df69a5f8e61d57bc814aaa0e5633feb46ea26e6d0c3a258160006a82faca7`.

```python
def emit_summary(result, deadline):
    with until(deadline):
        data = encoded(result)
        require(len(data) <= STREAM_CAP, 'bounded outer fallback summary')
        # Leave stdout nonblocking. No BufferedWriter ever receives this payload.
        # One raw attempt only: EAGAIN, partial, alarm or other error ends it.
        os.set_blocking(1, False)
        remaining(deadline)
        require(os.write(1, data) == len(data), 'short outer summary')
```

## main

changed; caller lines 643-751; SHA-256 `625f6dafc76afe0bdaa230c1d90a64f371cee8dfd7a30482c3b8a9a1c5cf69eb`.

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
            writer.save('metadata-' + mode + '-result.json', result)
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
        emit_summary(result, deadline)
        remaining(deadline)
    except Exception:
        return 1
    return 0 if result['observation_sequence_complete'] else 1
```

## normalize_startup

added; caller lines 252-268; SHA-256 `80e5fba7591a29e84112904ed36cd7a349da7fb354bde96f4d07008b04912062`.

```python
def normalize_startup(root, mode):
    """Closed local bootstrap only; SYSTEM evidence grants COPY no exception."""
    expected = startup(root)
    require(all(name in os.environ and os.environ[name] == value for name, value in expected.items()),
            'required caller startup inputs missing or changed')
    if mode == 'source':
        require(sys.version_info[:3] == (3, 9, 6) and sys.flags.isolated and sys.flags.no_site
                and sys.flags.dont_write_bytecode and not sys.flags.optimize, 'measured SYSTEM3.9.6 -I -S -B startup')
        require(set(os.environ) == set(expected) | set(SOURCE_BOOTSTRAP_EXTRA_KEYS),
                'exact measured source bootstrap key set')
        # Values are deliberately neither inspected nor trusted. Their origins
        # were not traced. Discard all six in this process before pins or work.
        for name in SOURCE_BOOTSTRAP_EXTRA_KEYS:
            del os.environ[name]
    else:
        require(mode == 'copy' and set(os.environ) == set(expected), 'exact copy bootstrap key set')
    require(dict(os.environ) == expected, 'exact effective caller environment after bootstrap')
```

## Module constant

```python
SOURCE_BOOTSTRAP_EXTRA_KEYS = ('CPATH', 'LC_CTYPE', 'LIBRARY_PATH', 'MANPATH',
                               'SDKROOT', '__CF_USER_TEXT_ENCODING')
```

## Complete module entry block

Caller lines 754-761; SHA-256 `8ec8764a7b580214511d27dded2041f7397f8641117c6c8f700938d75e4456ba`.

```python
if __name__ == '__main__':
    # Bypass finalization on every return or uncaught pre-admission exception.
    # No shutdown flush, traceback writer, handle destructor or payload retry.
    try:
        exit_status = main()
    except BaseException:
        exit_status = 1
    os._exit(exit_status)
```
