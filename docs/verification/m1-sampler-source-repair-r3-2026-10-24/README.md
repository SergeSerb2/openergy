# M1 filesystem sampler repair R3 — source only

This is the narrow implementation response to P2-1 in independent review
`931f7fd79357eeb8f6b7a245bbf094408f578f75`, against candidate
`65b95a7c6643beba780e6284968b246496bbd4c9`. Execution remains false.
Fresh independent review of this exact repair is required before a build.

The original `usage()` callback raised when `os.walk` tried to scan a child
directory that ordinary build staging had already moved or removed. The
existing vanished-file handler covered only the later `lstat` call. No runtime
failure or incidence rate is claimed; this defect was found from source.

The repaired sampler ignores only `FileNotFoundError` with errno ENOENT when
the error names a canonical absolute descendant of the owned root. It refuses
the root path itself, unknown or relative paths, parent components, permissions,
ENOTDIR, other I/O errors and all other exceptions. The strict `unreadable`
callback used for hashing immutable Python-prefix data is unchanged.

The parent binds the new root's device and inode immediately after creation.
Every usage sample requires that exact root identity, current-user ownership,
directory type and mode0700 before and after traversal. Loss or replacement of
the root, including between samples, fails. Normal descendant removal does not
change that root identity. The existing lstat disappearance handling remains.

The original 24 GiB / 200,000-entry sampled caps, two-second nominal sample
period, common absolute deadline, five-second TERM grace, output limits,
identity/session recovery and all build/source/input/signing predicates remain
unchanged. This is still sampled accounting, not a hard filesystem quota or a
guarantee against concurrent same-user mutation and arbitrary OS-call delay.

Only `usage()` and its root-identity plumbing in `main()` change in the full
controller. The Swift entry, source bindings, workspace closure and budgets
are exact R1 bytes. The operation template remains disabled; its operation ID
and controller hash identify R3. [repair.patch](repair.patch) is the exact
controller/template delta. [static-checks.json](static-checks.json) records
file/Git/hash/JSON/AST comparisons only. No supplied controller, helper, AST,
test, mock, simulation, fixture, filesystem/process/OS probe, build, signing,
product or remote command ran for this repair.

The complete inherited finite operation and proof limits remain in the
[R1 plan](../m1-fresh-build-plan-2026-10-24-t3-c52ad306/README.md). The parent
must prepare a new binding for this exact controller after review. The prior
staged R1 binding remains execution=false and unexecuted. Its build root was
not created. No spent guard is reset and no previous flow is retried.

The successful M1 prerequisite tool commands remain separately scoped evidence.
No app build has run. Original FAILED/UNVERIFIED results, the old stopped Swift
causal diagnostic, and the closed expired Sparkle operation remain unchanged.
This repair changes no production source and grants no runtime, native-input,
updater, distribution or release acceptance.
