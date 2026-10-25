# Static evidence and limits

Own standard-library data readers performed these checks. They did not import,
evaluate or execute any supplied source, AST, checker, product or helper.

- All41 bound input files equal their immutable Git blob bytes, SHA-256 and
  checkout bytes. JSON inputs parse as data. This includes complete R9 and
  admission packages, full independent R9/admission reviews, saved-attempt
  public README/review/bindings, the existing Darwin lifetime source, the
  unchanged AppEndToEnd source, and the exact build source inventory.
- The seven original admission hunks reconstruct all8 baseline module hashes.
  The new36-hunk patch reconstructs all9 complete `.py.txt` outputs exactly,
  including the added coverage module. Both positions, counts and removed
  bytes match, with no fuzz. No executable module was materialized on disk.
- All9 final source texts parse as AST data. All76 unchanged function ASTs
  compare exactly against the reconstructed admission candidate. Four complete
  modules are byte-exact. No original function is removed.
- Nine narrow normalized comparisons pass: journal transaction after removing
  the focus argument/default; signal helper, targeted recovery and port function
  after removing only the recorder keyword; cleanup/advance after removing only
  their phase labels; original launch prefix/return; original command exception
  kill/reap body; collection after removing only the sticky failure operand.
  `source-checks.json` names every changed/added/unchanged function.
- All5 active helper-command call sites have the recorder argument. No new
  Popen, exec, signal, product switch, product source or ABI is added. The old
  unused historical Git helper remains outside the fresh admission path.
- Original cap, accounting, first-record, deadline, archive, identity,
  uncertainty-restore/absence/revision and TERM-grace functions compare exactly.
  New prompt/helper reads reserve their own rounds/rows before querying; full
  snapshots retain the original post-read accounting plus control-row charges.

The first normalization reader failed because it removed an added argument but
left that argument's default in the AST comparison. `static-reader-errors.json`
retains the error and correction. No implementation or old evidence changed
to make that comparison pass. This was a data-reader correction, with no
runtime invocation or retry of a product scope.

`manifest.json` binds all other package files except itself and SHA256SUMS.
SHA256SUMS binds the manifest too; the Git commit binds all files. The final
commit diff adds only this owned folder. All prior tracked blobs remain exact.
No private historical run or archive was reopened or requalified. Public saved
summaries remain statements about retained evidence, not fresh runtime proof.

These checks establish syntax, reconstruction, identity of protected source
fragments and bounded design facts. They do not execute control flow, establish
ABI agreement or filesystem/signal timing, measure CPU/memory/latency, prove
short-stage capture, or establish a successful real ordinary flow. The parent
must independently review the patch. **Runtime remains UNVERIFIED.**
