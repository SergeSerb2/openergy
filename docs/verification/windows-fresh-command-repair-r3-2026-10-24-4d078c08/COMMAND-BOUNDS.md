# Static command-size admission

This is source/string arithmetic, not a launch, test, mock, simulation, or native
parser result. The bound is for complete native command lines, not only payloads.
All counts below are UTF-16 code units. The Windows ceiling is 32,767 including
one NUL. Supplementary Unicode characters use two units; Python character counts
are not substituted for UTF-16 counts.

Every `remote` caller is covered: setup; qualify, build, runtime and postflight;
cleanup; retained-root audit; and each corresponding exact-owner timeout recovery.
There is one transport encoder. Recovery bypasses `remote` only with the exact
argv already admitted by that same encoder before the original launch.

For each original and recovery script:

1. Encode strict UTF-8. Refuse NUL and lengths outside 1..65,536 bytes.
2. Raw-DEFLATE with level 9, method DEFLATED, window bits -15, default remaining
   parameters; Base64 the result. Refuse more than 10,400 literal characters.
   No favorable compression ratio is assumed. Recovery must also pass this gate.
3. Substitute this literal, the exact decimal byte count (at most five digits,
   used twice), and a 64-character SHA-256 into the fixed ASCII wrapper.
4. Encode that complete wrapper as UTF-16LE Base64 and construct the full native
   argv. Count its complete Windows serialization and NUL before any SSH launch.

The wrapper literal is 1,588 ASCII characters before substitution. Its maximum
complete size is:

`1,588 - 11 + 10,400 - 2*9 + 2*5 - 8 + 64 = 12,025` units.

Its encoded argument therefore has at most:

`4 * ceil((2 * 12,025) / 3) = 32,068` ASCII characters.

The exact native prefix is `powershell.exe -NoLogo -NoProfile -NonInteractive
-EncodedCommand `, 66 characters including the separating space. The executable,
flags and Base64 argument contain no characters that require Windows argv quotes.
The complete maximum is **66 + 32,068 + 1 = 32,135**, leaving 632 units below
32,767. This applies independently to BOTH commands. The recovery source may
contain the entire original encoded argument, but its final compressed literal
is subject to the same 10,400-character admission. It cannot bypass that cap.
The preserved recovery literal is 2,126 ASCII characters, including its
11-character selector placeholder. Thus its maximum source is
`2,126 - 11 + 32,068 = 34,183` UTF-8 bytes, within the 65,536-byte input cap.

`windows_command_units` still measures each actual complete argv, rather than
depending only on this algebra. It uses the Windows argv quoting algorithm in
`subprocess.list2cmdline`, encodes the result as UTF-16LE, and adds one NUL.
Spaces, empty arguments, embedded quotes, runs of backslashes and non-ASCII
characters therefore contribute their actual serialized width. Embedded NUL or
unencodable Unicode fails before launch. Integers must already be their actual
decimal argument strings; neither maximum PID guesses nor shell expansion stands
in for measuring an affected command.

Each stage also admits its complete inner argv before `remote`: the quoted
`C:\Program Files\PowerShell\7\pwsh.exe`, all four options, and the actual
UTF-16LE Base64 of the whole `try/catch` invocation. Its two root occurrences,
both stage-name occurrences, fixed syntax and failure-receipt path are included.
This line has its own 32,767-unit admission; outer compression cannot hide an
oversized inner command. The actual root is measured after substitution. The
existing native TEMP alphabet refuses spaces/non-ASCII TEMP paths before root
creation; this repair does not weaken that gate or assume a convenient path.

The unchanged C# product launcher always quotes the executable and every argv
element and refuses embedded quotes/trailing backslashes. The stage admission
also yields a conservative inherited product-line bound: the inner script
contains two root occurrences, so `root_units <= floor(32766*3/16) = 6143`, even
if every fixed character is ignored. The daemon argv has exactly two root
occurrences (executable and config); the remaining literals, quotes, separators,
pipe nonce and at most ten decimal PID digits fit within 1,024 units. Thus its
complete line, including NUL, is at most `2*6143 + 1024 + 1 = 13,311` units.
The version, absent-status, healthy-status, config, invalid-method and stop CLI
lines are smaller. No numeric-argument width or path byte count is silently zero.
This is string arithmetic over preserved source, not a product process result.

The fixed `/usr/bin/ssh` and `/usr/bin/scp` local argv run on macOS and are not
Windows CreateProcess command lines. The native sources' Cargo/tool/tar calls
and compiler-generated commands remain unchanged by this repair; no new size or
native acceptance claim is made for those independent paths.

The target OpenSSH/default-shell implementation is not known in this task. A
shell can impose a smaller limit or different quoting. `prepare_inputs` therefore
requires a fresh independently reviewed `windows_launch_binding` tied to the
existing selector hash. Its 1..4 exact literal command-layer templates must cover
every Windows shell/wrapper command, with exact serialized prefix/suffix and each
actual limit (positive, at most 32,767). `transport_argv` measures each complete
`prefix + native_line + suffix` as UTF-16LE plus NUL for BOTH commands before the
original launch, and refuses any excess. Quotes, spaces, non-ASCII launcher paths
and numeric flags are part of those exact strings. Unknown or nonliteral launch
construction is unsupported. The binding must be backed by fresh review evidence;
its admission booleans are not host observations or proof of the actual route.

No template or private binding was created here. This report does not infer a
default shell or claim observed command lines. Fresh independent review must
establish that these templates describe the existing launch implementation and
cover all layers and smaller shell limits before execution. Refuse unknown or
unrepresentable routes; do not probe, change the shell, retry, truncate, or weaken
owner selection to make them fit. These bounds close the reported unconditional
37,920-character source defect. They do not supply missing host/transport or
runtime proof. A valid baseline selection can still be refused by command-size
admission; there is no assertion that every selection/path fits every shell.
