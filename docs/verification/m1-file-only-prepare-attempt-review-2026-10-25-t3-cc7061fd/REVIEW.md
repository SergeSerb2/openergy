# Independent failed preparation packet review

2026-10-25, America/Vancouver. SOURCE/SAVED-DATA ONLY.

**HOLD: 0 P1 / 3 P2 findings. The failed attempt remains FAILED_UNVERIFIED.**
The packet accurately retains failure and the main evidence limits. Its saved
stream, reference and local clock facts are internally consistent. Preparation,
COPY, requirements qualification and ordinary acceptance remain blocked.
execution=false; runtime_acceptance=false; updater CLOSED.

The three findings concern two related requirements issues and one independent
locality error. They do not authorize a repair, another attempt or any target
write. No source implementation was changed.

## Findings

**FP-P2-1 — The inherited requirements vector can bypass the reserved writer.**
The frozen metadata recipe uses `--requirements`, `:-` for all seven objects.
All 50 actual vectors match the recipe. R9 native Popen uses cwd ROOT at425;
the full R11 caller retains this at428. R11 `pin_inputs`281–296 reads the same
vectors, and main745–746 runs them against the prepared copy. The SOURCE/COPY
writer saves captured pipes after the native command; it cannot reserve or
control a native write to a different leaf.

The primary public local manual, `/usr/share/man/man1/codesign.1`, documents
the display requirements argument as a destination path at238–243 and531–543.
A dash selects stdout. The entitlements option has its own semantics; its
conventions cannot establish the requirements argument's meaning. This manual
is public text evidence, not qualification of the installed remote executable.

The saved seven actual0 commands have empty stdout and stderr containing only
an Executable line. The observed literal `:-` leaf contains a requirement
comment. These bytes and the documented destination semantics establish a
concrete unsafe vector, even though no filesystem creator trace exists.
Repeating that vector in COPY can overwrite the existing leaf outside
`common.write_bytes`, its locked precharge, exclusive publication and byte
accounting. The saved comparison of empty pipes would not detect that write.
No new COPY ran here.

Minimal repair implication: separately review the exact requirements output
vector and capture contract, its pins, consumers and accounting. Use the
documented stdout destination only after its exact future contract is bound;
do not transpose an entitlements convention. Preserve the old vectors and
failed evidence. A layout change alone cannot fix this issue.

**FP-P2-2 — Raw equality does not supply seven required requirements bodies.**
Every one of the28 actual requirements files was read, not just its reference.
Each raw stream, OS record and comparison matches the original retained
reference and the full frozen SOURCE manifest. The four last-object display
files also match. All seven OS records retain actual0, reap, both EOF, correct
argv/clock order and no recorded kill, loss, timeout or overflow. Their
`saved_basis=true` comparisons are genuine comparisons of the captured bytes.
They contain no requirements body in either pipe.

The seven stdout lengths are all0. The stderr lengths, in object order, are
177,180,201,219,217,143,141. Each stderr is an Executable line. This is not proof
that the seven designated requirements were captured or compared. The original
receipt contract expressly requires signature mode/requirements and full
entitlements as data. Its semantic requirements gate therefore remains open.

The one observed `:-` leaf is consistent with the last object's CDHash
`07f9138c67bb59ebdc8c04536fc0c9cb108b0fb0`. It is not an authenticated collection
of seven per-object bodies. Do not assign it to all seven objects or invent
the six other bodies. Its creator is an inference, not a directly traced actor.

Minimal repair implication: keep historical53/212/50 facts and their raw pass
labels intact, but separate them from requirements semantic acceptance. A
separately commissioned evidence contract must identify how actual per-object
bodies will be obtained, retained and compared. This review supplies no bodies
and no authority to repeat SOURCE or consume the burned preparation guard.

**FP-P2-3 — The closed remote name set includes two local parent originals.**
The exact prepare source66–79 declares COMPLETED7. Its union925–927 includes
all seven names in remote ROOT. `closed_root_layout`1127–1166 consequently
requires both `source-files-once.guard` and `source-files-parent-result.json`
as remote leaves. Main1804 applies that gate before reads, growth or helpers.
The first diagnostic actually records both names absent.

This is a locality error, not missing authority to copy those local records.
The same prepare source's complete `inspect_remote`1052–1086 reads exactly
five remote files and explicitly leaves the other two with the parent.
The frozen SOURCE launcher `inspect_remote`284–318, the four-input materializer
`inspect_completed`273–310 and the publisher `inspect_completed`626–663 all do
the same. Their COMPLETED7 dictionaries agree as provenance data; that equality
does not make every manifest entry a remote file. SOURCE `parent_record`367–375
puts all seven references in data without publishing the two local originals.
Publisher main938–993 publishes its eight inputs, guard and receipt. Its actual
receipt explicitly says `remote_completed_files_authenticated=5`.

The prior layout derivation and R6 review treated agreement of seven-name
declarations as remote name provenance. That inference is not supported by the
actual reader/writer paths. Publisher's historical259 entries plus its receipt
equals260, and its bytes likewise equal1,242,294. These scalars remain true;
they do not authenticate the proposed name set. The later diagnostic has the
same count and total with a different set.

Minimal repair implication: review an explicit remote-five/local-two contract,
actual local parent inspection provenance and every expected-name consumer.
Authenticate `coverage-policy.json` against the frozen1864-byte policy, and
retain the67-byte `:-` leaf as failed evidence with its exact binding. Merely
loosening the set, deleting the extras, copying private originals to ROOT or
resetting ROOT/guards is not an acceptable repair. No such action occurred.

## Attempt and diagnostics

`tool-observation.json` contains the saved direct tool return: driver OS1 and
the complete206-byte failed line, bound to the7703-byte driver result. The
result records observer1/reap, both EOFs,270 stdout bytes and empty stderr.
It records no assessor. `capture_complete_ns=null` records that the success
capture path did not complete; it does not erase the separately observed EOFs.

The observer's retained-handle record gives collector1, observed before its
original135-second end. The collector records SSH1,63 stdout bytes,0 stderr
bytes, both EOFs, fsynced prefixes and no timeout, KILL, overflow, dropped or
unknown unread bytes, or publication failure. The actual saved remote line is
`FAILED_UNVERIFIED` with `r=null`. There is no full remote receipt. SSH1 does
not supply a separately observed remote real-child handle or precise exception.
The driver's observer-stage ValueError is not the remote exception location.

These are saved tool/parent/collector observations at their respective layers.
Producer Boolean inspection duties and narrative pass flags do not add OS
facts. This review made no fresh OS observation. The driver does not prove its
own exit; the direct tool-return record supplies that saved status.

The checked local order runs from driver start through collector SSH exit,
observer D4, line/EOF/exit and result prewrite to the later same-Mac sample:
`36038696545708 < 36173225749833`. That last sample is a causal upper bound for
the completed tool result, not an exact driver exit timestamp. Driver and
observer each retain their original start+135 arithmetic. No local timestamp
was compared with a remote origin. No new envelope replaces the failed one.

Both distinct diagnostic parent records retain actual collector0. Both
collectors retain SSH0, complete streams and original135-second envelopes.
Their payloads read only fixed ROOT data, with their own20-second read ceilings;
the local parents stage/record locally and launch transport. The remote query
payloads do not launch a helper or mutate ROOT. Their fresh transport ceilings
do not renew preparation. The first parent's buffered console is empty; its
fsynced parent record is the retained wait/poll observation.

Diagnostic1 scans all260 names, totaling1,242,294 bytes. Its saved per-entry
metadata checks reject no entry: regular, owned,0600,nlink1,same filesystem.
The missing names are the two local originals. The extra names are `:-` and
`coverage-policy.json`. All eight parent outputs, prepared.json,
prepare-guard.json, bytes.json and bytes.lock are absent. This scan publishes
aggregate metadata and differences, not a full content inventory or all leaf
stamps. It is not independent qualification of every actual leaf body.

Under the exact rendered source association, exclusive ROOT and no-deletion
premises, source order places the durable parent guard before every helper,
common import and private preparation. Its absence and the absent outputs
support the inference that private preparation never entered. This is an
inference from order and retained absence, not a remote child observation.
The layout mismatch is a confirmed admission blocker; the precise remote
pre-guard exception remains unobserved.

Diagnostic2 reads only the two named extras. Its full source enforces8192-byte
caps, nofollow regular0600/owner/nlink1/device checks and repeated leaf metadata.
The policy exactly equals the frozen policy, SHA256
`35516fa08202a4ccd26ece557daecc5bf4e1e8f8e690f18b59d5cc70c3d30d36`.
The `:-` leaf SHA is
`d55daee618ec400cc662eb3fb480e13b7ea93ff24c00b69bd4b2e9c871d48c22`.
**Literal correction to the brief:** the actual67 bytes end in one LF, not two.
Its exact text is `# designated => cdhash H"07f9138c67bb59ebdc8c04536fc0c9cb108b0fb0"`
followed by one LF. Two LF bytes would make68 bytes and a different hash.
Reconstruction from these actual bodies reproduces diagnostic2's2977-byte
console and saved transport hash; no missing raw console is invented.

## Exact bindings and scope of older verdicts

[input-bindings.json](input-bindings.json) binds original commits, Git blobs,
modes, full lengths and hashes. The failed packet at
`6dea2532eeae763150d7efb6701632f8ebeb9163` has folder tree
`3b14c0aab8c76dffd9d56b427779257385d40cad`:60 files /97,801 bytes.
All60 entries match frozen Git and working bytes/modes; all59 SHA256SUMS rows
match the exact payload set. Canonical entry digest is
`50fa93d11c0a1644f8dde45a7d264ce22c5504763c6fc6b66b1bbee3df874e5c`.
The evidence defines the canonical encoding and records actual per-entry check
counts and mismatches, without duplicating a large inventory.

The exact109444-byte remote source and full R6 report match the requested
`02f94afb` / `63b6a3b5` identities and b3fbd793… / dc8e986c… hashes.
Public bound configs independently reconstruct the final payload110985 /
3f9ff298…, observer7737 /8924a041… and driver48889 /3aad8d8d… exactly as
constant-only data edits. The three8bde42c sources and full4fa8b37b report also
match f1fa00b2… /7b7e9dd9… /517c7cc8… /e6956207…. This does not freshly
qualify CPython3.15, its AST shape, private profile/guards or executed-file
identity. Hash-only retained refs remain hash-only; no private originals read.

The original R6 cache/topology-exclusion, fresh-launch and direct-signal branch
closures remain narrow source results. Its remote-set provenance is now subject
to FP-P2-3. The full local driver/composition source verdict stays attached to
its exact bytes; this failed run provides no acceptance. Legacy695dc014 bindings
remain historical SOURCE basis. Active local records bind f1fa00b2; neither
identity silently substitutes for the other.

R11 `23cb52f54cf9ef288500d4b09606d42303ff414f` has full source47491 bytes /
SHA43415cd8…. R12 `c35d061031d277e13c84f7e80ebf8205b63dc06f` has the exact
requested060c4627… report hash. Comparing all36 complete declarations against
R9 leaves only collect/main changed. All50 vectors remain inherited, including
the seven unsafe requirements vectors. R12's signal-order/signed-status and
v3-interface observations do not close vector semantics, accounting or missing
bodies. Its broad source-clear label cannot establish those current gates.

The current COPY parent's complete inspect_fixed_inputs, copy_record_complete
and inspect_copy_result still bind the old leaf, `kill_delivered` and v2.
Its collect vector also names the old leaf. R11's new alias/v3 needs the
separate materializer/parent composition already held in PARENT-DELTA. The
52-observation/208-reference design is not actual COPY evidence. No alias
materialization or runtime receipt exists in this review.

The historical53 saved zero-exit rows,212 raw references /213334 bytes and50
raw-comparison facts remain intact. All212 declared rows agree across frozen
prepare data and saved SOURCE validation; their canonical manifest SHA is
`2baea069155b633485b1e8aed8fe72f3bf746f67de83a84f07a2daf0648fe4ee`.
Only the32 newly supplied requirements/display raw bodies were independently
reopened here; the other180 remain saved public reference/status evidence.
No claim of fresh independent qualification of those180 bodies is made.

Current gates stay held for exact source/metadata repair and review, locality
and ownership, requirements bodies, active alias/materializer/parent/schema
composition, actual reserved publication, original clocks and causal enclosure,
full receipts/raws, genuine exits/reaps/EOF, artifact/operator predicates,
COPY/native qualification, continuous NI15 and ordinary/runtime acceptance.
No source verdict renews an envelope or unburns the one attempt.

## Method and preservation

Only reviewer-authored public Git/text/hash/strict JSON/AST/literal data readers
ran. All33 packet JSON files were parsed with duplicate/nonfinite rejection.
Full packet report, saved records, four diagnostic source files, full remote
prepare and R11 caller, relevant complete local driver/observer/collector and
producer bodies, full linked R6/local-composition/R12 reports and relevant full
contracts were read. Large raw declarations were checked as complete data.
No supplied checker or code ran, was imported, evaluated or compiled to an
executable. No tests/mocks, build, network/SSH, private/state read, native,
process/port/GUI action, child agent, PR or push occurred.

The reader notes retain local data-reader corrections: an existing Windows
CRLF conversion, observer token quote spelling, a shadowed output variable,
an AST literal encode wrapper, bounded display/path corrections, and the brief's
incorrect two-LF literal. Failed intermediate readers did not change inputs or
produce successful evidence. These are not preparation retries.

Opening checkout was clean at7165435b11d6de162240bfe361423322dc695b3f,
tree d055dfa3a1fe07cf349e8551d544a6ada34f9876. All4742 old tracked entries,
77,586,363 working bytes, index blobs and full working permission modes were
checked. Canonical working digest is
`c4ac1bb25f0e0571b7fdfb650a820fc389c9ebe16ffd976085696b1eab02e49f`.
The original Windows CRLF working bytes are explicitly preserved. Closing
evidence repeats every old-entry check and digest. Only this one NEW review
folder is committed, with hooks/signing disabled. Exact commit identity and
post-commit clean status are reported outside the packet to avoid self-reference.

Preserve all prior states: SDK helper1/SSH1 FAILED_UNVERIFIED; M1 app25/25,
app0/controller1 FAILED_NOT_ACCEPTED; earlier23/25 FAILED; first post65431
EADDRINUSE cause UNKNOWN; whole-flow NI15/runtime UNVERIFIED; missing original
second post-bind, bundle-after/outcome/archive; stripping warning and unknown
full outer build duration/exit; Windows SSH255/noRoute FAILED_UNVERIFIED with
remote state UNKNOWN; V-READER-STARTUP UNVERIFIED; all failed guards; updater
CLOSED. Confidence is high for the exact public bytes and source defects.
Creator attribution, remote exception and all unobserved runtime facts retain
their stated limits. No root modification, retry or execution authority follows.
