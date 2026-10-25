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

The existing saved metadata identifies one literal PowerShell 7 wrapper and one
Windows PowerShell child with a quoted resolved executable path. This repair
adds an exact representation of the latter. It does not change any launched argv.
The v2 binding has 1..4 `command_layers`, with only these closed constructions:

- `literal_native`: `prefix + native_line + suffix`, as in R3. The actual wrapper
  uses this construction. Prefix and suffix are exact serialized strings.
- `resolved_executable`: `"` + exact bound executable + `"` + the unchanged tail
  of `native_line` after its fixed `powershell.exe` token. This represents the
  actual child. It preserves the leading space, every option and the entire
  encoded argument. It has no prefix/suffix fields or other replacement rules.

The source validates exact keys and rejects other construction types. The resolved
path must be a normalized absolute local drive path with an ASCII drive letter,
backslash root, one colon, and leaf `powershell.exe` under case folding. Quotes,
NUL, CR and LF refuse. The source preserves the supplied path spelling; it does
not obtain, normalize, discover or select an executable for launch. The path is
admission data only. It is not substituted into SSH or native argv.

For BOTH main and recovery, `transport_argv` measures each complete constructed
line as UTF-16LE plus one NUL, and refuses encoding, NUL or effective-limit errors
before the main launch. Each limit must be a positive integer <=32,767 and must
include any smaller applicable launcher or shell limit. Each actual complete
child/shell/wrapper line must occur once in the private binding. No entry can be
padding, an approximate line, or a guessed limit. The unchanged native argv has
its separate 32,767-unit gate. The two v2 construction types are not composable;
unknown nested transformations require another source review, not a callback.

The retained query's native line is 2,415 units including NUL. Its child is exactly
a quoted 57-unit executable followed by the unchanged tail, 2,460 units including
NUL. Its wrapper is exactly a 45-unit prefix, unchanged native line without NUL,
and one-unit suffix, 2,461 units including NUL. The child does not contain the
complete unchanged native line, which explains why R3 could not represent it.
The observed executable spelling comes from the saved child, not a registry
spelling assumption. The three sshd ancestor lines contain no payload; they are
not invented as payload wrappers.

Conditional on this observed construction remaining unchanged, the capped encoded
argument yields these complete maxima for BOTH main and recovery:

- submitted native: `66 + 32,068 + 1 = 32,135`;
- quoted resolved child: `(2 + 57 + (66 - 13)) + 32,068 + 1 = 32,180`;
- literal outer wrapper: `45 + 66 + 32,068 + 1 + 1 = 32,181`.

These leave 632, 587 and 586 units below the 32,767 API ceiling. The wrapper is
one unit longer than the child for this exact relation. Admission does not rely
on that inequality: v2 can represent and measure both actual lines independently.
No candidate payload, compressor, decoder or renderer was executed for these
counts. Static string comparisons use only the already saved query as data.

The saved query does not bind installed implementation versions or prove effective
limits for near-ceiling commands. The parent must separately bind those facts,
the same exact selector, unchanged route/configuration/argument construction,
all layers and their effective limits. No private preparation/binding or true
admission assertions were materialized here. Independent review of this exact
repair is pending. The previous source clearance applies only to R3.
Unknown, changed, unbound or unrepresentable routes must refuse admission. No
query, route/default/configuration change, retry, truncation or relaxed owner
selection is part of this repair. All runtime acceptance remains UNVERIFIED.
