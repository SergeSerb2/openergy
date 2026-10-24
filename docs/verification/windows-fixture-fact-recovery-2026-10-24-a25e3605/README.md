# Windows fixture fact recovery — 2026-10-24

**Bounded recovery exhausted: 0/9 qualified bindings. Runtime remains UNVERIFIED.**

The 160-item retrieval budget is consumed. Six relevant project threads yielded
155 unique items, with all five duplicate retrievals charged again. Item text
totalled 446,627 UTF-8 bytes; the largest item was 37,607 bytes. No retrieved item
was truncated. No parent-read position range was repeated.

One exact historical Windows root was recovered as JSON data. It is saved only
in partial-fixture-facts.json, under the new private root identified in
[result.json](result.json). It does not establish a required label association
and containment under original TEMP. The original setup uses LOCALAPPDATA plus
a private component; this does not establish the original TEMP.
The diagnosis item repeats the original frozen root, without a proved diag_remote
value. No basename or conventional TEMP prefix was promoted to a binding.

All nine exact ordered bindings remain missing:

- e4f48ba-original
- startup-diagnosis
- owned-cleanup-dfa405c
- observed-startup-e6a9125
- historical-dns-remote
- historical-dns-diag_remote
- stopped-dns-8333a48
- prior-exact-r12-8281eb7
- historical-refusal-615c16e1

No baseline-selection.json exists. Partial data has a distinct schema, no
selection entries, and explicit missing labels. It is not old state, a current
baseline, a raw receipt, an executable approval receipt, or permission to run.
The private folder is mode 0700; all five private files are mode 0600. Their
hashes are bound in [result.json](result.json).

The redacted [history ledger](history-ledger.json) records thread/item IDs,
positions, byte counts and text hashes. The [source ledger](source-ledger.json)
binds the exact public inputs and six named private JSON-location checks. All
six private locations were absent. Conservative charges are 48/64 exact
locations and 2,097,152/33,554,432 explicit file/Git bytes, including owned artifact readback. No other-project,
foreign-user, home/temp/worktree-wide, database or private application-store
search was used. Raw history was not saved. Only redacted excerpts and derived
metadata were surfaced.

The parent is a2a83194efbab6559aacb30def21b8b9890027c3. Every pre-existing tracked file remains unchanged.
The native evidence package contains 179 tracked files at this parent; all stay
unchanged, including the inherited 178-file evidence and guards.
Its tree is e96bfbeeceb2958e5468ec963788fba5b2b0511f. No prior evidence, guard,
private original or production source was changed. No transport selector,
credential or configuration was inspected or used.

Only independent data readers, JSON decoding, hashes, lexical path checks and
Git metadata checks ran. No supplied code, parser/compiler, build, runtime,
preflight, fixture preparation, SSH, native/API/DNS query or bind ran. The R7
seven-source candidate stays inert and under separate review. Existing failed
results, stopped gates and the closed expired Sparkle attempt remain unchanged.

The exhaustion is bounded to the stated reads. It does not assert that every
possible historical source is absent. Exact missing facts remain prerequisites;
no current prefix scan, remote probe, empty selection or historical hash can
substitute for them.
