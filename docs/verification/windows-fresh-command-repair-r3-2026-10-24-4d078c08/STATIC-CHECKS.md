# Static evidence boundary

The implementation check used inline data readers. No supplied source/helper was
imported or run. Python AST nodes were inspected as data; no node was evaluated
or compiled into executable code. No PowerShell/C# parser, compressor/decoder
exercise, test, mock, simulation, native check or host probe ran.

The retained report is [static-checks.json](static-checks.json). Checks read exact
file bytes and immutable Git objects, decode JSON, compare literal strings and
source spans, reconstruct unified-diff hunks as string data, calculate SHA-256,
and count complete command-line widths algebraically. They do not exercise the
controller or substitute responses for its future real CLI flow.

The 22 R1 package files and three authoritative review files match their named
immutable commits. All 21 R1 manifest members, eight R1 source bindings and 48
public input bindings match. The original full R9 diff regenerates exactly.
Only the controller changes in R3; all seven native files match R1. The six-hunk
minimal R1 patch reconstructs the exact new controller as string data.

The unchanged methods, exact recovery PowerShell literal, setup/removal/audit
native literals, and declared changes to command/stage/input admission were
compared directly. The historical, baseline, budget, data-bound, one-shot, pinned,
failure, removal and proof-limit plan fields match R1. The result schema matches
R1 bytes. Existing private input fields and fresh-selection checks remain.

All 179 historical folder files match their published hashes and immutable Git
blobs. All 558 current-reference source records match their Git blob bytes,
hashes and line endings. These checks do not supply any missing historical
fixture binding, archive, native result, or fresh execution input.

Two authoring-reader errors are retained in
[static-reader-errors.json](static-reader-errors.json): a source-line escaping
mistake in the reader, followed by a link check before manifest generation.
The source comparison predicate stayed unchanged. The corrected reader completed
the source/patch predicates; final file/link/manifest checks follow generation.
Neither error ran a helper or native command. No failed historical result changed.

The first staged Git whitespace check reported a blank context marker at the end
of the exact three-context-line patch. Its separate exit code was not retained.
The patch was regenerated with four context lines and reconstructed exactly as
string data. No source byte or whitespace predicate changed. The separate final
Git check records its own exit status.

The final pass decodes every new JSON file, checks local Markdown link targets,
requires regular nonexecutable inert source files with no trailing whitespace,
verifies every source/package binding, and runs `git diff --cached --check`.
The package manifest excludes itself and SHA256SUMS. SHA256SUMS binds the manifest;
the delivery commit binds both. Native syntax and runtime remain UNVERIFIED.
