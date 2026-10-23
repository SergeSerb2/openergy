# Background task and IPC lifetime source review

Review `oe-macos-background-ipc-lifetime-review-r1-043757a-20261023` found no
concrete actionable P1/P2 at `043757a`. It inspected the real AppModel,
DaemonClient, DaemonSupervisor, IPCSocket and existing AppEndToEnd flows.
The review ran no commands, tests, traffic or desktop actions and changed no
source. It adds no independent runtime proof.

Two observations still need real measurements:

- `AppModel.swift` creates a status-refresh task for each `peer.untrusted` event.
  Task buildup during actual trust-request bursts has not been measured.
- `DaemonClient.swift` dispatches reader callbacks onto the main queue. Sustained
  log delivery while that queue is busy has not been measured.

No P1/P2 impact is established by either source observation. The existing
25/25 real app flow checks stalled writes and supervised crash recovery, but
does not establish caller-cancellation or burst-load behavior. A separate bounded
real-event measurement is in progress. Missing genuine event witnesses must
remain unverified.
