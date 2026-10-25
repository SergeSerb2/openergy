# Full changed functions: R2 to R4

SOURCE/DATA ONLY. execution=false. These are inert source blocks.

The baseline is the exact collector at 88db0b54cf4d07092ea464392fe9b16ac7da2940.
Only Collector.settle and Collector.run change. All other 23 functions and
all module statements remain byte-identical. The inventories and hashes are
in static-checks.json. The exact two-hunk reconstruction is collector-from-88db0b5.patch.

## Collector.settle - Before (88db0b5)

Lines 396-433.

```python
    def settle(self):
        self.settle_stop_ns = min(self.settle_end, time.monotonic_ns() + 2 * SECOND)
        self.observe_exit()
        if time.monotonic_ns() >= self.settle_stop_ns:
            self.timeout = True
            self.fail("settle_deadline", "settle")
        if self.process is None or self.actual_exit is not None:
            return
        if self.first_failure is None:
            self.fail("unreaped_transport", "settle")
        # Only this retained local SSH handle. poll() rechecks before kill().
        eligible = (self.timeout or any(item["overflow"] for item in self.streams.values())
                    or time.monotonic_ns() >= self.work)
        if (eligible and time.monotonic_ns() < self.settle_stop_ns
                and self.process.poll() is None):
            self.kill["attempted"] = True
            self.kill["observed_ns"] = time.monotonic_ns()
            try:
                self.process.kill()
                self.kill["returned"] = True
            except OSError:
                pass
        self.observe_exit()
        remaining = (self.settle_stop_ns - time.monotonic_ns()) / SECOND
        if self.actual_exit is None and remaining > 0:
            try:
                code = self.process.wait(timeout=remaining)
                observed = time.monotonic_ns()  # BEFORE any receipt writes.
                self.actual_exit = code
                self.exit_observed_ns = observed
                if code != 0:
                    self.fail("ssh_exit_nonzero", "wait")
            except subprocess.TimeoutExpired:
                self.timeout = True
                self.fail("ssh_reap_timeout", "settle")
        if time.monotonic_ns() >= self.settle_stop_ns:
            self.timeout = True
            self.fail("settle_deadline", "settle")
```

## Collector.settle - After (R4)

Lines 396-443.

```python
    def settle(self):
        self.settle_stop_ns = min(self.settle_end, time.monotonic_ns() + 2 * SECOND)
        self.observe_exit()
        if time.monotonic_ns() >= self.settle_stop_ns:
            self.timeout = True
            self.fail("settle_deadline", "settle")
        if self.process is None or self.actual_exit is not None:
            return
        if self.first_failure is None:
            self.fail("unreaped_transport", "settle")
        # Only this retained local SSH handle. poll() rechecks before kill().
        eligible = (self.timeout or any(item["overflow"] for item in self.streams.values())
                    or time.monotonic_ns() >= self.work)
        if eligible and time.monotonic_ns() < self.settle_stop_ns:
            code = self.process.poll()
            observed = time.monotonic_ns()  # Immediately after the eligibility poll.
            if code is not None:
                self.actual_exit = code
                self.exit_observed_ns = observed
                if code != 0:
                    self.fail("ssh_exit_nonzero", "poll")
            elif observed < self.settle_stop_ns and observed < self.absolute:
                self.kill["attempted"] = True
                self.kill["observed_ns"] = observed
                try:
                    self.process.kill()
                    self.kill["returned"] = True
                except OSError:
                    pass
            else:
                self.timeout = True
                self.fail("settle_deadline", "settle")
        self.observe_exit()
        remaining = (self.settle_stop_ns - time.monotonic_ns()) / SECOND
        if self.actual_exit is None and remaining > 0:
            try:
                code = self.process.wait(timeout=remaining)
                observed = time.monotonic_ns()  # BEFORE any receipt writes.
                self.actual_exit = code
                self.exit_observed_ns = observed
                if code != 0:
                    self.fail("ssh_exit_nonzero", "wait")
            except subprocess.TimeoutExpired:
                self.timeout = True
                self.fail("ssh_reap_timeout", "settle")
        if time.monotonic_ns() >= self.settle_stop_ns:
            self.timeout = True
            self.fail("settle_deadline", "settle")
```

## Collector.run - Before (88db0b5)

Lines 435-593.

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

## Collector.run - After (R4)

Lines 445-606.

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
                self.admission_clock()
                os.fsync(parent)  # Persist the new root entry before the once guard.
                self.admission_clock()
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
