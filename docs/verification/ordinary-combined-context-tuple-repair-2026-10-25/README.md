# Restore the composed recorder's context argument

Execution=false; runtime UNVERIFIED. Independent review `c138853` found one P2
in composition `1ecac53`: its pre-bind coverage tuple omitted `self.spec`.
`common.command` needs three values, so that candidate would raise TypeError
before spawning the first helper or attempting its first bind.

The repair adds that one argument. The full corrected controller is carried
here; the other nine source modules are referenced byte-for-byte from the
immutable composition package. Materialization and source-check records bind
all ten files. Limits, coverage policy and the disabled operation template
remain exact. No earlier package or review is changed.

The repaired coverage keyword's AST now equals the original priority package's
actual keyword, including `self.spec`. This direct comparison closes the gap
in the earlier composition check, which compared the enclosing changed method
too broadly and did not verify this argument list. The exact one-line patch,
old/new hashes and nine unchanged module bindings are retained.

The complete prior review covered the full priority repair, standalone port
diagnostic and composition, and found no other concrete P1/P2. A fresh narrow
independent review of this repair remains pending. The prior source-clear
port diagnostic verdict remains separate from runtime qualification.

No source/helper execution, import, native parser, process/port/network query,
fixture preparation, product, build, signing, test or mock ran. Static AST and
hash checks passed. The new plan remains disabled. All previous M1, ordinary
and Windows failures remain unchanged; port cause remains UNKNOWN. Missing
priority samples still fail coverage. No deadline, cap, product predicate,
signal authority, retry limit, receipt, or original guard changes.
