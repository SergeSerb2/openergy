# Acquisition and coverage policy

This is source design. No branch, timing, ABI or cleanup action ran.

## Prompt product acquisition

`Controller._launch` keeps the original durable intent, retained Popen handle,
wrapper observation, exact same-start registration, and guarded app-tree
absence check for peer-stop. After it publishes that gate, it calls
`Inventory.observe(..., focus=label)` before normal tick sleep or helper launch.
The call is inside a `try/finally` that still collects every direct handle.

`Inventory.observe` retains R9's existing lock, restore, durable begin,
checkpoint and loss boundary. Only after restore/begin does `_observe` call
`acquire_product`. That function directly calls the unchanged `macproc.targeted`
with the registered PID/start and controller PID/start. This avoids the previous
Python-process startup and whole-host enumeration before the first product read.

Each prompt read reserves one round and two rows from the existing shared
budget BEFORE querying. Two is conservative if fewer rows return. The raw
result, start/end monotonic timestamps and counts go to the actor's
`*-prompt-observations.jsonl` before the next acquisition or full helper starts.
There are no fabricated rows or product callbacks. `Inventory.identify` remains
AST-exact R9. It validates both wrapper and product stages, including the
same-start gate, exact interpreter/argv, UID, group and current parent chain.
Positive identities enter the existing records and identity writer. Priority
mismatch fails immediately after the identity is retained. A complete valid
row can resolve pending state as in R9; a partial read uses the original
`note_pending` clock/count. Eight uncertain reads or two seconds still fail.

While the same verified wrapper remains, the controller yields for at most
1ms between prompt reads. This waits in the observer; it adds no product sleep.
The existing wrapper can take up to its current 20ms poll interval to notice
the gate. The unchanged gate/CLI/flow deadline bounds the entire acquisition,
including lock restore, writes, reads, yields and the subsequent full snapshot.
Global/main budgets can end it earlier. There is no new two-second allowance.
Explicit ESRCH stops acquisition with no product row. A launch is never repeated.

The following full snapshot is still mandatory. Prompt data never replaces its
rows, resolves an unknown private candidate by inference, grants absence, releases
peer permission, or authorizes a signal. Full processing still sees all private
candidates and restores/checkpoints the same uncertainty journal. An exception
before complete full processing leaves R9's lost/dirty handoff and blocks absence.
Already acquired prompt data remains evidence of what was read; it is no clean
handoff certificate. A sink failure retains the in-memory failure and handles.

## Control actors and late stages

Full snapshots query controller and armed-watchdog starts as known identities.
Their detail rows are retained in a separate `controls` list in the full packet.
They are query/coverage identities only. They do not enter owned-product records,
groups, registered-start journal entries or the product signal set. Identity
checks bind PID/start, UID/real UID, group, executable and exact argv. Watchdog
reparenting is allowed only with explicit controller ESRCH, as recorded data.
Missing/changed detail and observed non15 nice readings fail observation.

The final policy requires actual controller and watchdog samples in `flow`,
`flow-complete`, and `cleanup:all-exits`. Recovery marks its current phase before
the existing full observation. Thus a single early cleanup sample cannot fill
late control coverage. Real helper-parent samples also supply the controller's
postflight phase. No ready receipt is reused as a late observation.

All four direct roles require both wrapper and product samples: peer,
peer-status, app and peer-stop. Initial and recovered supervised-daemon product
samples remain required. Peer/app/both daemon starts each need separated actual
samples at least20ms apart; app also needs a sample after recovered-daemon
operation begins. Peer needs an actual cleanup-actor product sample. The
unchanged app-flow checks still require exactly two daemon starts, real SIGSTOP
coverage, all25 unchanged checks and real app exit0. An observed last row is
only the last retained observation, never proof of a process's entire lifetime.

## Helpers

Every active-flow `command()` call passes its runtime and actor to the recorder:
full/targeted/signal probes, pre/post netstat and the final watchdog-absence probe.
The admission preparation is file-only and has no active `command()` call.
The unused historical `git_file` path retains its original behavior.

The recorder creates no helper process of its own. It keeps an intent before
the existing Popen, records that handle's PID, and reads its actual child and
parent details before the first blocking communicate wait. Each20ms timeout
slice can supply another actual sample before the next wait. This repeats the
wait for the SAME process; it never repeats the command. Each query reserves
one round/two rows first. The first observed child start is retained. Reuse
blocks detail publication, and a reaped handle is never queried again.

The recorder validates real parent PID, UID/real UID, actual inherited group,
canonical executable and exact argv, plus nice15 before/after detail. It
records raw partial data and failures too. The ledger closes each intent with
one spawn and a real complete OS status. An unclosed intent, missing product
detail, uncertain attempted read, bad identity or missing/capped sink remains
UNVERIFIED. No inherited-nice assertion replaces a missing sample. A helper's
spawn-to-collection interval is not asserted to be its process lifetime.
For Python helpers, the expected actual argv0/executable comes from the existing
in-memory session binding. It stays distinct from the selected launcher path.
The recorder adds no session-file reload prerequisite to targeted recovery.

The original `command()` timeout kill/reap exception body is AST-exact. All
observation and communication work fits inside its fixed80-percent child-work
window. The remaining original deadline share stays available for its original
owned-helper kill/reap. Recorder publication/budget errors are sticky acceptance
failures; they do not suppress returned probe data or grant a product signal.
The command's actual error and timeout cleanup still propagate normally.

## Limits and failure closure

All new payloads use the existing locked pre-write common writer. Main
observations use regular space; cleanup/fallback/cleanup-launch/postflight use
nonterminal cleanup space. Final coverage receipts use terminal space. No new
reservation exists. Keep8MiB/file,32MiB total, eight2MiB product slots,
4MiB cleanup including1MiB terminal,64KiB accounting, and rename-only archival.
Summary readers also use the original8MiB file ceiling.

The shared6000 rounds/24000 rows now include prompt and helper observations,
control rows in full packets, and one explicit zero-row initialization charge.
Main stays within4976/19904; cleanup's1024/4096 reserve is not enlarged or lent
to main. `coverage-budget-initialization.json` records that one charged round,
so counters can be reconciled without pretending it was a process observation.
New captures reduce the budget available to ordinary snapshots. Byte or count
capacity can fail before all nominal maxima fit. No limit is reset or raised.

Keep120s flow,60s cleanup and product end at start+180s. The existing3s control
reap and20s evidence archival allowances remain evidence-only. Product/IPC/
frame/stalled-write/request/wait predicates, actual TERM-delivery times and full
five-second grace remain unchanged. Helper recorder failure cannot bypass
app-first cleanup, uncertainty restore, the guarded peer-stop launch, retained
actual exits, fallback construction or fresh verified single-PID recovery.

Admission checks helper coverage before the first product. Each main launch
checks for its real product sample after prompt acquisition. Missing coverage
aborts through the existing failure path. Cleanup continues despite coverage
failure, then final coverage requires peer-stop, late controls and all helper
closures. `coverage-before-postflight.json` preserves that assessment even if
the next strict bind fails. `coverage-final.json` can exist only if the required
samples are present. Its status is REQUIRED_SAMPLES_PRESENT, never continuous
priority proof. Missing evidence remains UNVERIFIED and blocks acceptance.

## Darwin options and proof limits

Only existing sources were inspected. R9 uses libproc BSD/path and
KERN_PROCARGS2 reads, staged executable/argv coherence, and Popen plus execve.
`crates/daemon/src/lifetime.rs` uses Rust libc `EVFILT_PROC` with `NOTE_EXIT` for
parent lifetime. It supplies no exec-stop interface, `NOTE_EXEC` observation,
ptrace protocol or suspended-spawn contract. No such ABI is invented here.
An exec notification alone would not be a retained nice/detail observation.
The repaired source improves capture opportunity, but cannot guarantee that
every ultra-short CLI or metadata helper remains observable. Missing stages
retain explicit UNVERIFIED evidence. All samples leave time between reads.
No continuous NI15, latency improvement, stable runtime or port-cause fix is
claimed. Parent review and any separately scoped real full-flow proof remain.
