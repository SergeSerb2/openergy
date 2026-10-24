This record requests one temporary running-app update flow. **Approval is still
missing. No operation code or product was executed by this preparation.**

The flow launches temporary build 1 with no arguments, starts its private app
model and virtual role-Off daemon, and invokes the pinned external Sparkle CLI
once. Sparkle downloads the prepared archive, quits and replaces build 1, then
relaunches build 2 with no arguments. The controller checks the new app and
daemon. It makes one normal final quit only after it independently validates
the new build and process identity. It then checks owned cleanup. There is no
retry or fallback port. The control cutoff is 180 seconds, with 60 seconds for
recovery inside a 240-second controller ceiling. Failure can retain fixture
state and incomplete cleanup evidence.

The operation uses the prepared private fixture at
`/private/tmp/oe-updater-fixture-915c0654-e982-43e4-be3e-a44fa05799eb` and literal
`127.0.0.1:56327`. It permits at most four HTTP requests and four private IPC
requests. The app and daemon must measure absolute nice 15. Official Sparkle
helpers use normal nice 0. The operation can create its exact fixture launchd
jobs and caches. It also permits normal LaunchServices registration and
timestamp notifications, plus normal Gatekeeper, gktool and XProtect work.
Persistent shared registration and assessment state is unknown. Full host
restoration and trusted distribution are not claimed. The exact cleanup scope
and delayed recovery signals are in the frozen proposal.

The marker expires **2026-10-24 00:56:25.164527 PDT**. At least 240 seconds must
remain at live preflight. Expiry, binding drift or a failed precondition blocks
the operation. Marker renewal, artifact repair and a second launch are outside
this request. The fixture still uses the older reviewed daemon at `ad280644`;
later daemon-stop measurements do not apply to it.

The earlier approval at 2026-10-23 17:16:55 PDT covered the two completed
stopped-target cases only. This request adds the initial running app, model and
daemon startup, normal quit, replacement and no-argument relaunch. A new explicit
approval must bind this exact proposal, controller, scope and review receipt.

The [independent R6 review](../sparkle-running-repair-R6-review-2026-10-23-t3-bbbc0189.md)
reports no actionable P1/P2. It closes all five previous findings in source.
The [corrected proposal](../sparkle-running-repair-2026-10-23-t3-6133e534-r2/operation-DO-NOT-RUN.json)
and [controller text](../sparkle-running-repair-2026-10-23-t3-6133e534-r2/controller-DO-NOT-RUN.py.txt)
remain byte-exact. This parent prepared a separate JSON receipt that records
that independent result. The receipt grants no execution approval.

| Record | SHA-256 |
| --- | --- |
| Controller | `a934611e4bef5d4b355ae11279d6b1aaef0dadf665503efcb788df4869453567` |
| Proposal | `cb15b124d6c76c77bad90d05bcc4ea1384aa55aaac48c959bb03ff3cfceba41d` |
| Independent review document | `266c23f4f398ea0c5a7a89a4a51ab3741365e8ac806e3315994aa51fc8065130` |
| Separate review receipt | `6af6a1df11c504917b8631e502491135f723b19c6bdcd956c5e78dc56f5adfc4` |
| ZIP | `9dd967f30ffcba5388e1d093e84243e1db9feb4ed6c7f66299609112cc6dbf69` |
| Appcast | `f4b565d1ab3a0a59bf20dd858bc5a528b04c911b281aeef0bf7a9b475ce2fea2` |

The file-only refresh at 20:29:52 PDT matched both 35-entry freeze copies and
their 36 Git-bound files, all 451 source hashes, six saved evidence hashes,
six private fixture files, and the complete 167/167/159-entry old/candidate/
driver inventories. It checked file modes, links and ownership for those
inventories. It made no signature-verifier or runtime claim. See
[file-bindings.json](file-bindings.json) and the exact read-only checker
[check-files.py](check-files.py). The first checker invocation failed because
the existing Python lacks `hashlib.file_digest`. That failure is preserved in
[first-file-reader-failure.json](first-file-reader-failure.json). A streaming
SHA-256 reader corrected the local checker. No toolchain changed.

The separately hashed [review receipt](independent-review-receipt.json) is ready.
[approval-request.json](approval-request.json) is an unapproved request record,
with `approval_received:false`. It must never be treated as an approval event.
The frozen proposal's historical missing-review fields remain unchanged;
the separate R6 document and receipt supply that result.

Live preflight remains required after approval: immutable bindings, modes and
ownership; stopped fixture; absent fixture processes, jobs, caches, receipts
and socket; unchanged unexpired marker; and one exclusive bind to the exact
port. It must refuse before launch on failure. No live preflight, supplied
helper, target, daemon, driver, updater, installer, signing or network command
ran here. Source review and file hashes establish no runtime behavior. The
normal visual AppUpdater flow, native input, Developer ID, notarization and
publication remain separate open gates.
