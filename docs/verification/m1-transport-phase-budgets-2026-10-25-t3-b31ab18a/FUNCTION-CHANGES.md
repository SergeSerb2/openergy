# Complete changed function blocks

IMPLEMENTED_UNREVIEWED. execution=false.

The sole module addition is the fixed `PHASE_WORK_SECONDS` table. No duration flag is added.

```python
PHASE_WORK_SECONDS = {
    "metadata-source": 130, "metadata-copy": 130,
    "prepare": 130, "controller": 335,
}
```

`arguments` admits only the four fixed phase names. All other CLI checks stay exact.
`Collector.__init__` chooses the fixed work allowance and derives the original start-based work, settle and absolute ends.
`Collector.deadlines` records the selected allowance in the existing guard/receipt deadline shape.

All other 22 functions remain byte-identical. `Collector.settle` and `Collector.run` are among them. The unchanged run comment that mentions work130 describes the original metadata case; operative deadlines use the selected fields. No comment-only edit expands the diff.

## arguments

Baseline; lines 74-93; 860 bytes; SHA-256 `a11c1eb28b4d16b691d502b957498c01cea56ad915802fdd85770b4fb6fb64e2`.

```python
def arguments(tokens):
    if len(tokens) != 2 * len(FLAGS):
        raise Rejected("closed_cli")
    result = {}
    for index in range(0, len(tokens), 2):
        key, value = tokens[index:index + 2]
        if key not in FLAGS or key in result or not value:
            raise Rejected("closed_cli")
        result[key] = value
    if set(result) != set(FLAGS):
        raise Rejected("closed_cli")
    if result["--phase"] not in ("metadata-source", "metadata-copy"):
        raise Rejected("phase")
    for key in ("--profile-sha256", "--payload-sha256", "--collector-sha256",
                "--clock-sha256"):
        if re.fullmatch(r"[0-9a-f]{64}", result[key]) is None:
            raise Rejected("digest_shape")
    if re.fullmatch(r"[0-9]{1,20}", result["--envelope-start-ns"]) is None:
        raise Rejected("clock_start_shape")
    return result
```

Candidate; lines 78-97; 842 bytes; SHA-256 `1fd53674cd076039e4b75e62a8eefcd74e53ad60f0a3988fd5daba06c1bfe4ab`.

```python
def arguments(tokens):
    if len(tokens) != 2 * len(FLAGS):
        raise Rejected("closed_cli")
    result = {}
    for index in range(0, len(tokens), 2):
        key, value = tokens[index:index + 2]
        if key not in FLAGS or key in result or not value:
            raise Rejected("closed_cli")
        result[key] = value
    if set(result) != set(FLAGS):
        raise Rejected("closed_cli")
    if result["--phase"] not in PHASE_WORK_SECONDS:
        raise Rejected("phase")
    for key in ("--profile-sha256", "--payload-sha256", "--collector-sha256",
                "--clock-sha256"):
        if re.fullmatch(r"[0-9a-f]{64}", result[key]) is None:
            raise Rejected("digest_shape")
    if re.fullmatch(r"[0-9]{1,20}", result["--envelope-start-ns"]) is None:
        raise Rejected("clock_start_shape")
    return result
```

## Collector.__init__

Baseline; lines 217-249; 1426 bytes; SHA-256 `634b2e5e77800ae24f14046ac45dc9b76bc908e7316e7d2083bb23a0436f03dc`.

```python
    def __init__(self, args, clock):
        self.args = args
        self.start = int(args["--envelope-start-ns"])
        self.work = self.start + 130 * SECOND
        self.settle_end = self.start + 132 * SECOND
        self.absolute = self.start + 135 * SECOND
        self.clock = clock
        self.first_failure = None
        self.publication_failures = []
        self.publication_failure_count = 0
        self.process = None
        self.actual_exit = None
        self.exit_observed_ns = None
        self.kill = {"attempted": False, "returned": False, "observed_ns": None,
                     "policy": "exact retained local SSH only; cap or deadline"}
        self.root = None
        self.outputs = {}
        self.inputs_verified = {name: False for name in INPUT_CAPS}
        self.input_bytes = {name: None for name in INPUT_CAPS}
        self.streams = {
            name: {"observed_bytes": 0, "written_bytes": 0, "eof": False,
                   "overflow": False, "read_drop": False,
                   "unknown_unread_bytes": None, "fsynced": False,
                   "prefix_sha256": None}
            for name in CAPS
        }
        self.timeout = False
        self.spawn_started_ns = None
        self.spawn_returned_ns = None
        self.guard_durable = False
        self.settle_stop_ns = None
        self.final_stop_ns = None
        self.hashers = {name: hashlib.sha256() for name in CAPS}
```

Candidate; lines 221-254; 1504 bytes; SHA-256 `37eb991c3f414687c22fc31b056c136cc46dcb97c322adf28fcf6ddab0a131c1`.

```python
    def __init__(self, args, clock):
        self.args = args
        self.start = int(args["--envelope-start-ns"])
        self.work_seconds = PHASE_WORK_SECONDS[args["--phase"]]
        self.work = self.start + self.work_seconds * SECOND
        self.settle_end = self.work + 2 * SECOND
        self.absolute = self.settle_end + 3 * SECOND
        self.clock = clock
        self.first_failure = None
        self.publication_failures = []
        self.publication_failure_count = 0
        self.process = None
        self.actual_exit = None
        self.exit_observed_ns = None
        self.kill = {"attempted": False, "returned": False, "observed_ns": None,
                     "policy": "exact retained local SSH only; cap or deadline"}
        self.root = None
        self.outputs = {}
        self.inputs_verified = {name: False for name in INPUT_CAPS}
        self.input_bytes = {name: None for name in INPUT_CAPS}
        self.streams = {
            name: {"observed_bytes": 0, "written_bytes": 0, "eof": False,
                   "overflow": False, "read_drop": False,
                   "unknown_unread_bytes": None, "fsynced": False,
                   "prefix_sha256": None}
            for name in CAPS
        }
        self.timeout = False
        self.spawn_started_ns = None
        self.spawn_returned_ns = None
        self.guard_durable = False
        self.settle_stop_ns = None
        self.final_stop_ns = None
        self.hashers = {name: hashlib.sha256() for name in CAPS}
```

## Collector.deadlines

Baseline; lines 320-325; 337 bytes; SHA-256 `2ac77958ca674996d2f1290e248ad707763288bb403bbdeedd068ea0f5417e9a`.

```python
    def deadlines(self):
        return {"started_ns": self.start, "entry_ns": ENTRY_NS,
                "work_until_ns": self.work, "settle_until_ns": self.settle_end,
                "absolute_until_ns": self.absolute,
                "work_seconds": 130, "settle_seconds": 2,
                "final_seconds": 3, "total_seconds": 135}
```

Candidate; lines 325-331; 389 bytes; SHA-256 `7b9f8628ee119a06ebf19b6563a30d4373f92e1376babfd683c6005c4a856635`.

```python
    def deadlines(self):
        return {"started_ns": self.start, "entry_ns": ENTRY_NS,
                "work_until_ns": self.work, "settle_until_ns": self.settle_end,
                "absolute_until_ns": self.absolute,
                "work_seconds": self.work_seconds, "settle_seconds": 2,
                "final_seconds": 3,
                "total_seconds": self.work_seconds + 2 + 3}
```
