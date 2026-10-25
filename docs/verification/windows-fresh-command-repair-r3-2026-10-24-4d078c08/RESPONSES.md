# Response to P2-1

The authoritative review is accepted. R1 re-encoded the original script's entire
encoded argument inside an uncompressed recovery script. The safety module alone
therefore required at least 37,920 encoded characters on recovery. That could not
fit a complete Windows command line of 32,767 UTF-16 units including NUL.

The repair changes only the controller transport construction and its nested
PowerShell 7 size admission. It keeps the existing SSH channel, arguments,
immediate stdin EOF, exclusive command guards, and single workflow clock.
No stdin protocol, remote file dependency, new installation, or selector
shortening is introduced. The original inline safety code remains complete.

`transport_payload` bounds source UTF-8 bytes at 65,536 and compressed literal
Base64 at 10,400 ASCII characters. It uses fixed raw-DEFLATE parameters. The
in-memory decoder refuses excess expansion and requires exact byte length and
SHA-256 before strict UTF-8 decoding and script-block creation. Its error path
keeps the first error and independently attempts all four resource disposals.
The compressed bytes and their encoder need not be trusted as a source binding:
the complete plaintext hash must match, and actual final command size must fit.
No compressed byte sequence or compression ratio is claimed for unbound inputs.

`remote` constructs the original wrapper once. It inserts that exact encoded
argument into the unchanged recovery-script literal. It then wraps the complete
recovery script. Both final command lines pass admission before `command` can
launch the original. The timeout path receives these exact precomputed recovery
bytes and argv; it does not recompress, reread a remote script, or start another
recursive recovery. Local private records bind the original script, both wrapper
texts, recovery text, both argv, and both lengths. They do not grant acceptance.

The exact `CommandLine.EndsWith` selector remains intact. It is not replaced by
a digest-only, nonce-only, name-only, or PID-only selector. Zero owners and
non-unique owners retain their original outcomes. The complete recovery body,
retained-handle/generation checks, executable and command-line equality checks,
descendant restrictions, waits, disposal, and recovery acceptance assertion remain.
The one recovery guard and 30-second outer reserve remain. Forced exit still
cannot supply normal-stop acceptance. Unrepresentable preparation refuses launch;
it does not truncate code, reset a guard, retry a flow, or grant cleanup authority.

`windows_command_units` counts the complete serialized Windows argv using
`subprocess.list2cmdline`, UTF-16LE code units, and one terminating NUL. It rejects
NUL arguments and encoding failures. The nested PowerShell 7 executable includes
its spaces and quotes. Actual root strings are encoded before measurement.
The source still validates TEMP, ancestry, ACL, SID, nonce, generation, and each
tree entry. Compression changes none of these checks. See COMMAND-BOUNDS.md for
the exact formulas, the affected call paths, and the external-shell proof limit.

`prepare_inputs` also requires a fresh independently reviewed binding for the
existing selector's actual Windows launch route. It admits no guessed default.
For each original and recovery command, `transport_argv` measures every bound
complete shell command layer, including its exact prefix/suffix, quoted paths,
numeric flags, non-ASCII UTF-16 width and NUL, against that layer's actual limit
of at most 32,767. A smaller shell limit remains binding. Unknown or nonliteral
launch constructions refuse execution. These templates describe the existing
route; they do not change the SSH command or authorize a shell change. This new
private binding is absent here and cannot be fabricated from a boolean assertion.

All seven native files are exact R1 bytes. Thus the pinned offline build, R9
missing-PID handling and compiler lifecycle, R2 shared reader and real wire/CLI
flow, full ten-second normal-stop allowance, first errors, and collection before
removal remain. Apart from `prepare_inputs`, `command`, `remote`,
`recover_transport_owner` and `stage`, the controller's existing methods remain
byte-identical. All original native safety and failure bodies remain byte-identical.
The plan's baseline, budgets, data
bounds, one-shot counts, pins, failure policy, and removal policy remain exact.

No test or native check was substituted for the prohibited execution. The saved
static report verifies source structure and immutable bytes only. PowerShell/C#
syntax, raw-DEFLATE interoperability, actual command lines on the target, exit
semantics, ownership, settlement, and real stop acceptance remain UNVERIFIED.
The operation stays unexecuted pending fresh independent review and binding.
