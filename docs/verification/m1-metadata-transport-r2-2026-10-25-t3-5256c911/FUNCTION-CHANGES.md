# Full function changes from f252

SOURCE/DATA ONLY. execution=false. Runtime UNVERIFIED. Independent review required.
Baseline: `f2520dfb9b07c6ab54c85f438f2ed0ecb5c05c37`, collector SHA-256 `5c27140ebe7c7ab575168187f08d4a28a4f52e0e53b789e75b4187abf5e2eb4e`.

Final collector SHA-256 `0aa1032bfbeb6eb13642c5c7cd4805c10a3b2c9ea897f1810ff67ec2f8a0b389`. The exact source patch is [collector-from-f252.patch](collector-from-f252.patch). No source or AST was executed/imported.
Exactly four functions change. No functions are added or removed. All other 21 functions are byte-identical. The module docstring changes the coordinator version. `OPTIONS` changes from the Windows16-token sequence to the exact M1 public20-token prefix; `PRIVATE_OPTIONS` and `SOURCE_GUARD_SHA256` are new constants.

## clock_descriptor

Accept trusted local macOS CPython >=3.10. Preserve isolated/no-site/no-bytecode/no-optimization flags and the complete Mach absolute-clock descriptor. This does not compare M1 and local clocks.

Before (full function):

```python
def clock_descriptor():
    info = time.get_clock_info("monotonic")
    if (sys.platform != "darwin" or sys.implementation.name != "cpython"
            or sys.version_info[:2] != (3, 15)
            or not info.monotonic or info.adjustable
            or info.implementation != "mach_absolute_time()"):
        raise Rejected("unqualified_coordinator")
    if not (sys.flags.isolated and sys.flags.no_site
            and sys.flags.dont_write_bytecode and not sys.flags.optimize):
        raise Rejected("startup_flags")
    return {
        "schema": "openergy.local-macos-clock.v1", "clock": "time.monotonic_ns",
        "implementation": info.implementation, "monotonic": info.monotonic,
        "adjustable": info.adjustable, "resolution_seconds": info.resolution,
        "unit": "integer_nanoseconds", "origin": "macOS Mach absolute clock",
        "scope": "same running local Mac only; CPython >=3.10",
        "cross_host_comparison": False,
    }
```

After (full function):

```python
def clock_descriptor():
    info = time.get_clock_info("monotonic")
    if (sys.platform != "darwin" or sys.implementation.name != "cpython"
            or sys.version_info[:2] < (3, 10)
            or not info.monotonic or info.adjustable
            or info.implementation != "mach_absolute_time()"):
        raise Rejected("unqualified_coordinator")
    if not (sys.flags.isolated and sys.flags.no_site
            and sys.flags.dont_write_bytecode and not sys.flags.optimize):
        raise Rejected("startup_flags")
    return {
        "schema": "openergy.local-macos-clock.v1", "clock": "time.monotonic_ns",
        "implementation": info.implementation, "monotonic": info.monotonic,
        "adjustable": info.adjustable, "resolution_seconds": info.resolution,
        "unit": "integer_nanoseconds", "origin": "macOS Mach absolute clock",
        "scope": "same running local Mac only; CPython >=3.10",
        "cross_host_comparison": False,
    }
```

## directory

Keep component-wise no-follow traversal and canonical path syntax. Require the final opened containing directory to have the current user as owner. System ancestors need not have that owner.

Before (full function):

```python
def directory(path):
    """Open every absolute directory component without following links."""
    if (not path.startswith("/") or "\x00" in path
            or (path != "/" and any(x in ("", ".", "..")
                                     for x in path[1:].split("/")))):
        raise Rejected("absolute_path")
    fd = os.open("/", os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        for part in path[1:].split("/") if path != "/" else ():
            next_fd = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW
                              | os.O_NONBLOCK, dir_fd=fd)
            os.close(fd)
            fd = next_fd
        return fd
    except BaseException:
        os.close(fd)
        raise
```

After (full function):

```python
def directory(path):
    """Open every absolute directory component without following links."""
    if (not path.startswith("/") or "\x00" in path
            or (path != "/" and any(x in ("", ".", "..")
                                     for x in path[1:].split("/")))):
        raise Rejected("absolute_path")
    fd = os.open("/", os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        for part in path[1:].split("/") if path != "/" else ():
            next_fd = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW
                              | os.O_NONBLOCK, dir_fd=fd)
            os.close(fd)
            fd = next_fd
        if os.fstat(fd).st_uid != os.getuid():
            raise Rejected("directory_owner")
        return fd
    except BaseException:
        os.close(fd)
        raise
```

## profile_vector

Require the fixed saved-guard hash and canonical absolute reference path without opening it. Admit exactly the ordered 29-token M1 vector, fixed executable/command, three validated private option slots and selector. Parent must establish complete saved-guard equality and bind the whole profile hash.

Before (full function):

```python
def profile_vector(data):
    profile = json.loads(data.decode("utf-8"), object_pairs_hook=unique_object)
    if (not isinstance(profile, dict)
            or set(profile) != {"schema", "source_guard", "argv"}
            or profile["schema"] != "openergy.m1-bound-ssh-profile.v1"):
        raise Rejected("profile_shape")
    reference = profile["source_guard"]
    if (not isinstance(reference, dict) or set(reference) != {"path", "sha256"}
            or not isinstance(reference["path"], str)
            or not reference["path"].startswith("/")
            or not isinstance(reference["sha256"], str)
            or re.fullmatch(r"[0-9a-f]{64}", reference["sha256"]) is None):
        raise Rejected("source_guard_reference")
    argv = profile["argv"]
    if (not isinstance(argv, list) or len(argv) != len(OPTIONS) + 3
            or not all(isinstance(token, str) for token in argv)
            or argv[0] not in ("ssh", "/usr/bin/ssh")
            or argv[1:1 + len(OPTIONS)] != OPTIONS or argv[-1] != REMOTE
            or re.fullmatch(r"[A-Za-z0-9_][A-Za-z0-9_.@:-]{0,254}",
                            argv[-2]) is None):
        raise Rejected("closed_ssh_vector")
    return argv
```

After (full function):

```python
def profile_vector(data):
    profile = json.loads(data.decode("utf-8"), object_pairs_hook=unique_object)
    if (not isinstance(profile, dict)
            or set(profile) != {"schema", "source_guard", "argv"}
            or profile["schema"] != "openergy.m1-bound-ssh-profile.v1"):
        raise Rejected("profile_shape")
    reference = profile["source_guard"]
    if (not isinstance(reference, dict) or set(reference) != {"path", "sha256"}
            or not isinstance(reference["path"], str)
            or not reference["path"].startswith("/")
            or "\x00" in reference["path"]
            or any(part in ("", ".", "..")
                   for part in reference["path"][1:].split("/"))
            or reference["sha256"] != SOURCE_GUARD_SHA256):
        raise Rejected("source_guard_reference")
    argv = profile["argv"]
    if (not isinstance(argv, list) or len(argv) != 29
            or not all(isinstance(token, str) for token in argv)
            or argv[0] != "/usr/bin/ssh"
            or argv[1:21] != OPTIONS or argv[21:27:2] != ["-o"] * 3
            or argv[-1] != REMOTE
            or re.fullmatch(r"[A-Za-z0-9_][A-Za-z0-9_.@:-]{0,254}",
                            argv[-2]) is None):
        raise Rejected("closed_ssh_vector")
    for index, name in enumerate(PRIVATE_OPTIONS):
        token = argv[22 + 2 * index]
        prefix = name + "="
        if not token.startswith(prefix):
            raise Rejected("private_option_slot")
        value = token[len(prefix):]
        if name == "UserKnownHostsFile":
            if (re.fullmatch(r"/[A-Za-z0-9_./:@+-]+", value) is None
                    or any(part in ("", ".", "..")
                           for part in value[1:].split("/"))):
                raise Rejected("known_hosts_path_shape")
        elif re.fullmatch(r"[A-Za-z0-9_\[\]:][A-Za-z0-9_.:\[\]-]{0,254}",
                          value) is None:
            raise Rejected("host_option_shape")
    # Syntax is not provenance. Parent must compare the complete argv with the
    # exact saved 1563-byte guard before binding the whole profile SHA-256.
    # Do not open the private guard or infer effective inherited SSH config.
    return argv
```

## Collector.run

Keep the exclusive0400 payload snapshot descriptor open, rewind it and pass that same descriptor to SSH. Remove the close/reopen pathname interval. All other function statements, caps, guard order, capture, settle and final publication remain unchanged.

Before (full function):

```python
    def run(self):
        payload_fd = None
        publication_stage = None
        try:
            if self.start > ENTRY_NS:
                raise Rejected("future_clock_start")
            if digest(canonical(self.clock)) != self.args["--clock-sha256"]:
                raise Rejected("clock_binding")
            self.admission_clock()
            parent, name = leaf(self.args["--run-dir"])
            try:
                os.mkdir(name, 0o700, dir_fd=parent)  # New root; never reset.
                self.root = os.open(name, os.O_RDONLY | os.O_DIRECTORY
                                    | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=parent)
                info = os.fstat(self.root)
                if info.st_uid != os.getuid() or stat.S_IMODE(info.st_mode) != 0o700:
                    raise Rejected("run_directory_owner_mode")
            finally:
                os.close(parent)
            for label, path in (("collector", os.path.abspath(__file__)),
                                ("profile", self.args["--profile"]),
                                ("payload", self.args["--payload"])):
                data = read_bound(path, self.args["--" + label + "-sha256"],
                                  INPUT_CAPS[label], self.admission_clock)
                self.inputs_verified[label] = True
                self.input_bytes[label] = len(data)
                if label == "profile":
                    argv = profile_vector(data)
                elif label == "payload":
                    payload = data
            guard = {"schema": "openergy.metadata-transport-once.v1",
                     "phase": self.args["--phase"], "bindings": self.bindings(),
                     "clock": self.clock, "deadlines": self.deadlines(),
                     "python": {"resolved_executable": os.path.realpath(sys.executable),
                                "version": sys.version,
                                "implementation": sys.implementation.name},
                     "caps": {"streams": CAPS, "combined": COMBINED_CAP,
                              "inputs": INPUT_CAPS},
                     "retries": 0, "remote_state": "UNVERIFIED",
                     "remote_child_exit": None}
            publication_stage = "once_guard"
            self.put("once.guard.json", canonical(guard), 16384, self.work)
            self.guard_durable = True
            # Snapshot the already hash-checked bounded bytes. Retain this FD as
            # stdin so later edits of the original pathname cannot change input.
            payload_fd = self.create("payload.stdin", 0o400)
            publication_stage = "payload_snapshot"
            self.write(payload_fd, payload, self.work, "payload_snapshot")
            os.fsync(payload_fd)
            os.fsync(self.root)
            os.close(payload_fd)
            payload_fd = None
            payload_fd = os.open("payload.stdin", os.O_RDONLY | os.O_NOFOLLOW
                                 | os.O_NONBLOCK, dir_fd=self.root)
            for name in CAPS:
                publication_stage = name + "_create"
                self.outputs[name] = self.create(name + ".prefix.bin")
            publication_stage = None
            self.check(self.work, "before_spawn")
            self.spawn_started_ns = time.monotonic_ns()
            self.process = subprocess.Popen(
                argv, executable="/usr/bin/ssh", stdin=payload_fd,
                stdout=subprocess.PIPE, stderr=subprocess.PIPE, shell=False,
                close_fds=True, cwd="/", env=ENVIRONMENT, bufsize=0,
            )
            self.spawn_returned_ns = time.monotonic_ns()
            self.check(self.work, "spawn_returned")
            self.capture()
        except BaseException as error:
            self.fail(str(error) if isinstance(error, Rejected) else
                      type(error).__name__, "work")
            if publication_stage is not None:
                self.publication_failure(publication_stage, error)
        finally:
            # This outer finally also handles a spawn that returns after work130.
            if self.process is not None:
                for name in CAPS:
                    pipe = getattr(self.process, name)
                    if pipe is not None and not pipe.closed:
                        self.streams[name]["read_drop"] = True
                        pipe.close()
            try:
                self.settle()
            except BaseException as error:
                self.fail(type(error).__name__, "settle")
            if payload_fd is not None:
                os.close(payload_fd)
        if self.root is None:
            return 1  # No safe new directory exists in which to publish.
        self.final_stop_ns = min(self.absolute, time.monotonic_ns() + 3 * SECOND)
        try:
            self.check(self.final_stop_ns, "final_start")
            for name, fd in self.outputs.items():
                try:
                    self.check(self.final_stop_ns, name + "_fsync")
                    os.fsync(fd)
                    self.check(self.final_stop_ns, name + "_fsync")
                    self.streams[name]["fsynced"] = True
                except BaseException as error:
                    self.publication_failure(name + "_fsync", error)
                finally:
                    os.close(fd)
                    self.streams[name]["prefix_sha256"] = self.hashers[name].hexdigest()
            complete = (self.first_failure is None and self.actual_exit == 0
                        and self.exit_observed_ns is not None
                        and self.exit_observed_ns < self.work
                        and all(item["eof"] and not item["read_drop"]
                                and not item["overflow"] and item["fsynced"]
                                for item in self.streams.values()))
            if not complete and self.first_failure is None:
                self.fail("incomplete_transport", "final")
            for item in self.streams.values():
                item["read_dropped_bytes"] = item["observed_bytes"] - item["written_bytes"]
            receipt = {
                "schema": "openergy.metadata-transport-receipt.v1",
                "status": "TRANSPORT_COMPLETE" if complete else "FAILED_UNVERIFIED",
                "phase": self.args["--phase"], "bindings": self.bindings(),
                "clock": self.clock, "deadlines": self.deadlines(),
                "python": {"resolved_executable": os.path.realpath(sys.executable),
                           "version": sys.version,
                           "implementation": sys.implementation.name},
                "guard_durable": self.guard_durable,
                "spawn_started_ns": self.spawn_started_ns,
                "spawn_returned_ns": self.spawn_returned_ns,
                "settle_stop_ns": self.settle_stop_ns,
                "final_stop_ns": self.final_stop_ns,
                "actual_local_ssh_pid": self.process.pid if self.process else None,
                "actual_local_ssh_exit": self.actual_exit,
                "actual_exit_observed_ns": self.exit_observed_ns,
                "handle_retained": self.process is not None,
                "kill": self.kill, "timeout": self.timeout,
                "streams": self.streams, "first_observed_failure": self.first_failure,
                "secondary_publication_failures": self.publication_failures,
                "publication_failure_count": self.publication_failure_count,
                "caps": {"streams": CAPS, "combined": COMBINED_CAP,
                         "inputs": INPUT_CAPS, "guard": 16384, "receipt": 32768},
                "retries": 0, "completion_passes": 0,
                "remote_child_exit": None, "remote_current_state": "UNVERIFIED",
                "payload_receipt_assessed": False, "artifact_accepted": False,
                "r9_reserved_publisher": False,
                "own_cli_exit_observed": False,
                "independent_parent_observation_required": True,
                "receipt_prewrite_ns": time.monotonic_ns(),
            }
            self.put("receipt.json", canonical(receipt), 32768, self.final_stop_ns)
            self.check(self.final_stop_ns, "after_receipt")
            return 0 if complete and self.first_failure is None else 1
        except BaseException as error:
            self.publication_failure("final_publication", error)
            # One bounded exclusive side record; no receipt replacement/retry.
            try:
                self.put("publication-failure.json", canonical({
                    "first_observed_failure": self.first_failure,
                    "secondary_publication_failures": self.publication_failures,
                    "publication_failure_count": self.publication_failure_count,
                    "receipt_may_be_partial_or_stale": True,
                }), 16384, self.final_stop_ns)
            except BaseException:
                pass  # Parent observes nonzero/missing facts; never claim durable.
            return 1
        finally:
            os.close(self.root)
```

After (full function):

```python
    def run(self):
        payload_fd = None
        publication_stage = None
        try:
            if self.start > ENTRY_NS:
                raise Rejected("future_clock_start")
            if digest(canonical(self.clock)) != self.args["--clock-sha256"]:
                raise Rejected("clock_binding")
            self.admission_clock()
            parent, name = leaf(self.args["--run-dir"])
            try:
                os.mkdir(name, 0o700, dir_fd=parent)  # New root; never reset.
                self.root = os.open(name, os.O_RDONLY | os.O_DIRECTORY
                                    | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=parent)
                info = os.fstat(self.root)
                if info.st_uid != os.getuid() or stat.S_IMODE(info.st_mode) != 0o700:
                    raise Rejected("run_directory_owner_mode")
            finally:
                os.close(parent)
            for label, path in (("collector", os.path.abspath(__file__)),
                                ("profile", self.args["--profile"]),
                                ("payload", self.args["--payload"])):
                data = read_bound(path, self.args["--" + label + "-sha256"],
                                  INPUT_CAPS[label], self.admission_clock)
                self.inputs_verified[label] = True
                self.input_bytes[label] = len(data)
                if label == "profile":
                    argv = profile_vector(data)
                elif label == "payload":
                    payload = data
            guard = {"schema": "openergy.metadata-transport-once.v1",
                     "phase": self.args["--phase"], "bindings": self.bindings(),
                     "clock": self.clock, "deadlines": self.deadlines(),
                     "python": {"resolved_executable": os.path.realpath(sys.executable),
                                "version": sys.version,
                                "implementation": sys.implementation.name},
                     "caps": {"streams": CAPS, "combined": COMBINED_CAP,
                              "inputs": INPUT_CAPS},
                     "retries": 0, "remote_state": "UNVERIFIED",
                     "remote_child_exit": None}
            publication_stage = "once_guard"
            self.put("once.guard.json", canonical(guard), 16384, self.work)
            self.guard_durable = True
            # Snapshot the already hash-checked bounded bytes. Retain this FD as
            # stdin so later edits of the original pathname cannot change input.
            payload_fd = self.create("payload.stdin", 0o400)
            publication_stage = "payload_snapshot"
            self.write(payload_fd, payload, self.work, "payload_snapshot")
            os.fsync(payload_fd)
            os.fsync(self.root)
            os.lseek(payload_fd, 0, os.SEEK_SET)  # Retain the exclusive snapshot FD.
            for name in CAPS:
                publication_stage = name + "_create"
                self.outputs[name] = self.create(name + ".prefix.bin")
            publication_stage = None
            self.check(self.work, "before_spawn")
            self.spawn_started_ns = time.monotonic_ns()
            self.process = subprocess.Popen(
                argv, executable="/usr/bin/ssh", stdin=payload_fd,
                stdout=subprocess.PIPE, stderr=subprocess.PIPE, shell=False,
                close_fds=True, cwd="/", env=ENVIRONMENT, bufsize=0,
            )
            self.spawn_returned_ns = time.monotonic_ns()
            self.check(self.work, "spawn_returned")
            self.capture()
        except BaseException as error:
            self.fail(str(error) if isinstance(error, Rejected) else
                      type(error).__name__, "work")
            if publication_stage is not None:
                self.publication_failure(publication_stage, error)
        finally:
            # This outer finally also handles a spawn that returns after work130.
            if self.process is not None:
                for name in CAPS:
                    pipe = getattr(self.process, name)
                    if pipe is not None and not pipe.closed:
                        self.streams[name]["read_drop"] = True
                        pipe.close()
            try:
                self.settle()
            except BaseException as error:
                self.fail(type(error).__name__, "settle")
            if payload_fd is not None:
                os.close(payload_fd)
        if self.root is None:
            return 1  # No safe new directory exists in which to publish.
        self.final_stop_ns = min(self.absolute, time.monotonic_ns() + 3 * SECOND)
        try:
            self.check(self.final_stop_ns, "final_start")
            for name, fd in self.outputs.items():
                try:
                    self.check(self.final_stop_ns, name + "_fsync")
                    os.fsync(fd)
                    self.check(self.final_stop_ns, name + "_fsync")
                    self.streams[name]["fsynced"] = True
                except BaseException as error:
                    self.publication_failure(name + "_fsync", error)
                finally:
                    os.close(fd)
                    self.streams[name]["prefix_sha256"] = self.hashers[name].hexdigest()
            complete = (self.first_failure is None and self.actual_exit == 0
                        and self.exit_observed_ns is not None
                        and self.exit_observed_ns < self.work
                        and all(item["eof"] and not item["read_drop"]
                                and not item["overflow"] and item["fsynced"]
                                for item in self.streams.values()))
            if not complete and self.first_failure is None:
                self.fail("incomplete_transport", "final")
            for item in self.streams.values():
                item["read_dropped_bytes"] = item["observed_bytes"] - item["written_bytes"]
            receipt = {
                "schema": "openergy.metadata-transport-receipt.v1",
                "status": "TRANSPORT_COMPLETE" if complete else "FAILED_UNVERIFIED",
                "phase": self.args["--phase"], "bindings": self.bindings(),
                "clock": self.clock, "deadlines": self.deadlines(),
                "python": {"resolved_executable": os.path.realpath(sys.executable),
                           "version": sys.version,
                           "implementation": sys.implementation.name},
                "guard_durable": self.guard_durable,
                "spawn_started_ns": self.spawn_started_ns,
                "spawn_returned_ns": self.spawn_returned_ns,
                "settle_stop_ns": self.settle_stop_ns,
                "final_stop_ns": self.final_stop_ns,
                "actual_local_ssh_pid": self.process.pid if self.process else None,
                "actual_local_ssh_exit": self.actual_exit,
                "actual_exit_observed_ns": self.exit_observed_ns,
                "handle_retained": self.process is not None,
                "kill": self.kill, "timeout": self.timeout,
                "streams": self.streams, "first_observed_failure": self.first_failure,
                "secondary_publication_failures": self.publication_failures,
                "publication_failure_count": self.publication_failure_count,
                "caps": {"streams": CAPS, "combined": COMBINED_CAP,
                         "inputs": INPUT_CAPS, "guard": 16384, "receipt": 32768},
                "retries": 0, "completion_passes": 0,
                "remote_child_exit": None, "remote_current_state": "UNVERIFIED",
                "payload_receipt_assessed": False, "artifact_accepted": False,
                "r9_reserved_publisher": False,
                "own_cli_exit_observed": False,
                "independent_parent_observation_required": True,
                "receipt_prewrite_ns": time.monotonic_ns(),
            }
            self.put("receipt.json", canonical(receipt), 32768, self.final_stop_ns)
            self.check(self.final_stop_ns, "after_receipt")
            return 0 if complete and self.first_failure is None else 1
        except BaseException as error:
            self.publication_failure("final_publication", error)
            # One bounded exclusive side record; no receipt replacement/retry.
            try:
                self.put("publication-failure.json", canonical({
                    "first_observed_failure": self.first_failure,
                    "secondary_publication_failures": self.publication_failures,
                    "publication_failure_count": self.publication_failure_count,
                    "receipt_may_be_partial_or_stale": True,
                }), 16384, self.final_stop_ns)
            except BaseException:
                pass  # Parent observes nonzero/missing facts; never claim durable.
            return 1
        finally:
            os.close(self.root)
```
