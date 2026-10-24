# Windows failed-replacement witness design — 2026-10-23

**UNVERIFIED. The existing public API cannot guarantee the required ordering.**
It can produce `Disconnected` under some schedules, as the saved evidence shows.
It cannot make the caller drop a new receiver before the worker's initial send
without another scheduling race. No finite caller-only revision found here
closes all four Windows witnesses on immutable `e6a9125`. No product defect is
inferred. This is source research and an audit of committed evidence, not new
runtime proof or approval to implement or execute the conditional design below.

## Fixed inputs

- Parent: `a121376fe6c1715fa99d72559336ca3b3a73b439`.
- Source: `e6a9125be2fc59fbe92fef6ea351b222298bfc52`.
  `vendor/mdns-sd/src/service_daemon.rs` Git blob:
  `31c356c0625e78cea5c46067ea9d0653c43867c5`.
  `crates/net/examples/discovery_recovery.rs` Git blob:
  `0a0898da910fbf457a462a71cd8e280c445813e9`.
  These two files also match the parent byte for byte.
- Windows evidence: `1d46512fb816d43e9fa4691c5bd7ae6a15d377cb`;
  `windows-observed-startup.json` blob:
  `c3809b6f2d4d3ce3f8366f2f875ece8c50d5e77e`.
- [R8](discovery-observed-startup-review-r8.md), committed at
  `787a4afb8aa80b47192bd28c0ac60e662bef810d`: source review only.
- [R9](discovery-final-evidence-review-r9.md), committed at
  `52481804256d7492a6b53a5cba274173887143f0`: saved Mac evidence audit;
  reviewed evidence `8e891f1575483f003756b6b84dc157eb049a9986`.
- [R10](windows-observed-startup-review-r10.md), committed at
  `7ad8265f9237aad8d37672095c884f051b5e4c2f`: saved Windows evidence audit.

The [Windows report](windows-observed-startup.md) retains startup 1 **FAIL,
105/106**: limited-over-limited observed **8 Accepted / 0 Disconnected**.
Startup 2 remains **FAIL, 113/115**: standard-over-limited and
limited-over-standard each observed **8 Accepted / 0 Disconnected**. These
missing cases also lack their consequent preservation checks in those runs.
The literal coverage failures are `startup-1.stdout:157` and
`startup-2.stdout:201,307` under `windows-observed-startup/outputs/` at the
evidence commit. Separate runs cannot pool witnesses. Saved active standard
Drop cleanup passed in both positive runs; it remains mandatory in any revision.

## Concrete source limit

Line references in this section refer to **e6a9125**, not a future revision.

1. `service_daemon.rs:492–497` creates a live, empty event receiver, submits
   `Browse`, and only then returns the receiver. `browse_limited` does the same
   with its private lifetime receiver and state at `529–545`. `LimitedBrowse`
   exposes only snapshots and closure status (`68–100`). It has no separate
   prepare, submit, or cancel-before-submit operation.
2. `send_cmd` queues the command at `447`, then creates and sends the loopback
   wake signal at `452–465`. The worker can run before this method returns.
   It drains commands and executes them at `1850–1857`.
3. The initial `listener.try_send(SearchStarted)` runs at `4240`. Only its
   actual error can take the failed-start return at `4250–4255`. Success commits
   replacement at `4261–4268`. The observer at `4241–4249` reports the actual
   result; it runs before commit and cannot prove commit by itself.

A valid schedule for a finite workload of successful public submissions is:
enqueue a browse;
let the worker execute its initial send while the receiver is still inside the
public method; return the receiver; let the caller drop it. Every such initial
send is `Accepted`, even if the worker later removes the dropped subscription.
This schedule also exists for a finite batch of concurrent callers or a finite
backlog of ordinary public commands. Extra cache replay, advertisements, sleeps,
thread priorities, or more attempts change likelihood, not this ordering.
An after-call metrics barrier proves processing has progressed; it cannot
retroactively change the initial send result.

`browse_cache` has the same receiver-before-submit race (`515–520`). It is not
a substitute for the required querying/repeat path (`4285–4288`). A preceding
`stop_browse` removes the old subscription (`4494–4505`), so it destroys the
preservation precondition. Shutdown ends the worker and its subscriptions
(`1851–1855`), rather than exercising a failed replacement. Queue rejection
(`Again`) and `DaemonShutdown` from submission are not an initial event-send
`Disconnected`. Invalid arguments fail before submission. Inducing a wake-socket
error is neither an ordering guarantee nor an allowed fixture here.

No worker-blocking logger, interface predicate, held internal lock, rendezvous,
private command, state inspection, synthetic observation, or injected channel
is an acceptable way around this limit. `ServiceDaemon` fields and `Command`
are private (`340–349`, `4881–4891`). The observer must remain passive.

## Smallest missing prerequisite and conditional caller workflow

The missing capability is **caller ownership of the new receiver before the
browse command is submitted**. A possible future normal public API could
separate construction from submission for both browse kinds: prepare a request
and its real receiver/view; drop all new receiver ownership; submit that request
once through the ordinary daemon API. Preparation must not queue or wake the
worker. Submission must still execute the real initial `try_send`, including
when its receiver was already dropped. A caller-side early-cancel shortcut
would not exercise this gate.

This API does **not exist** at e6a9125. It would require a separately authorized
source change, review, source/artifact identity, and new real Windows evidence.
It is a public lifetime-design prerequisite, not permission to expose `Command`,
pause a worker, add a test-only hook, or claim acceptance for e6a9125.

If that prerequisite is met, use one fixed invocation with four serialized
cases: limited-over-limited, standard-over-limited, limited-over-standard, and
standard-over-standard. Use one unique service type per case and one independent
limited anchor. Submit **one** preclosed replacement per case, with no
replacement retries. Keep the old handle alive throughout its case.

For each case, retain all existing behavior checks at
`discovery_recovery.rs:931–1059`:

1. Hold the old browse; observe its actual `Accepted`, receive a later public
   metrics reply, resolve a case-specific `old-live` value, and require the same
   open handle. With only the anchor and this case active, the exact limited
   state count is `1 + old_is_limited`.
2. Prepare the new request/view without submission, drop the new view, and then
   submit once. Require exactly one real initial `Disconnected` for this
   serialized type/kind. Receive a subsequent metrics reply before assertions.
   `Accepted`, missing/duplicate/unknown/`Full` observations, submission error,
   or timeout fails coverage. Preserve that first result; do not tune or rerun.
3. Require the identical old handle to remain open and the exact prior state
   count to remain unchanged. After the failed-send barrier and classification,
   drain the old standard receiver. Require a new `SearchStarted` after that
   drain, using the existing observed-count deadline (`704–752`). Counts over
   seven fail coverage. Limited lifetime events stay hidden; retain its open
   view, fresh resolution, and removal checks rather than inventing a repeat.
4. Publish a new case-specific value only after the failure barrier/drain.
   Require it on that old handle. Withdraw the exact instance and require its
   removal, then publish a different returned value and require it on that same
   handle with the same exact state count.
5. Hold an accepted successor of the new kind. Require a later command barrier,
   current returned data, closure of the old handle, and exactly
   `1 + new_is_limited` states. Hold a newest limited successor, with its own
   accepted outcome and barrier; require successor closure and exactly two
   states. Drop the two old generations, publish another fresh value, and
   require the newest handle to remain open, fresh, and at exactly two states.
   Its drop must return to the anchor-only count within eight seconds. A final
   held limited view plus explicit stop must close that view and leave one.

Then perform the independent **mandatory active standard Drop** at `1068–1106`.
Require accepted outcome, later barrier, resolution, a fresh repeat after drain,
an open receiver, and the anchor-only limited gauge **before** Drop. Observe
exactly one matching `kind=standard reason=Disconnected completed=true` normal
removal, then a later public metrics reply. Use the existing observed-count
deadline, at most 34 seconds. No same-type replacement or explicit stop may
precede this check. A gauge of one alone cannot pass. Missing, duplicate,
wrong-kind, `Full`, or late removal fails. Only after recording failure may
explicit stop clean up. Finish with fresh anchor data, exact anchor identity,
explicit anchor stop, and exact zero states.

The worker emits completed normal removal after removing the subscription,
limited state, and retransmissions (`service_daemon.rs:1574–1578,1603–1649`).
Replacement and explicit stop do not supply that normal-removal witness.

## Finite bounds and falsifiable acceptance

**Authorized execution budget for this research: zero runtime invocations, zero
network traffic, zero builds, zero remote access.** Only this document changes.

The conditional four-case workflow has 22 initial browse submissions: four per
case held/accepted views plus its one failed replacement (20 total), the anchor,
and the mandatory standard Drop view. It needs 20 ordinary registration/update
requests and ten withdrawals if each case's returned instance is retired after
its checks: 16 case registrations and eight withdrawals, three anchor values,
one standard value, and the final two withdrawals. No load/replay backlog is
needed. These are caller-request bounds, not a claim about DNS packet, byte,
cache, expiry, or timer bounds. No wire budget is authorized or measured here.

Use a fixed 360-second whole-invocation deadline including normal teardown.
Retain existing four-second data/removal waits, three-second command replies,
eight-second newest-drop wait, and the observed-count repeat/standard-cleanup
deadlines. Exhaustion is FAIL/UNVERIFIED; it cannot skip a check or create a pass.
No outer retry, added attempt, or cross-run coverage merge is allowed.

Keep the hard 1,024-record observer cap and overflow failure (`617–647`). Record
ordered real call/drop/send/barrier/drain/repeat/removal events; match all initial
sends by serialized service type/kind and cursor interval. Keep source, binary,
workflow, command, publisher, stdout/stderr, timing, exit, and cleanup hashes
correlated to each invocation. Do not replace real traces with planned events.
Observer records remain distinct from public commit barriers and worker joins.

This four-case design is not the entire startup/release acceptance gate. A full
revised startup invocation must also retain the 32 distinct-start checks
(`872–921`), including an actual failed initial send, exact surviving anchor,
fresh data, and bounded accepted-drop cleanup. Adding those fixed 32 submissions
gives 54 total initial sends; no added retries are permitted. Their coverage also
cannot be inferred from another run. Retain working-observer old-bug failure,
missing-startup/cleanup-observer controls, retained-800, and owned-shutdown gates
at their stated scope. Old evidence does not certify a future changed API.

If implementation/runtime is later authorized, it must retain uniquely owned
literal-loopback fixtures, private configuration before startup, disabled
application discovery, and virtual platform where relevant. The existing probe
uses custom-port loopback multicast; this document neither runs nor grants
permission for that traffic. TCP 24800, UDP 5353, live sharing, installed apps,
production configuration/TLS/default IPC/TCC/login/services/accounts/firewall/
DNS/toolchain defaults remain outside the workflow. Any later build retains
Rust 1.99.1, nice 15/Windows BelowNormal, two jobs, macOS deployment 15 where
applicable, and locked/offline operation where possible.

**Decision:** do not schedule another eight-attempt Windows run or substitute
timing load. The preparation/submission capability is absent from the immutable
public API. Leave both saved Windows gates FAIL/UNVERIFIED. The conditional
design preserves the acceptance contract, but its prerequisite, implementation,
native execution, and all resulting witnesses remain unverified. No stopped
provenance/security review, crafted expiry/NSEC/orphan experiment, or IPC/Synergy3
work was resumed.
