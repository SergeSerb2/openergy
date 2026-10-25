# Existing Windows launch binding: independent DATA/SOURCE review

**UNSATISFIED for the exact R3 contract. No private launch binding was created.**
This review covers only `preparation.windows_launch_binding`. It does not repeat
the separate R3 controller review. This is a data/source prerequisite result,
not a human approval gate or an executable receipt.

The reviewed source commit is `23b41681e4019f086f1f760c9817553d79acff04`.
The three requested R3 files match that commit byte for byte in both the parent
checkout and this isolated checkout. Their hashes are in `review.json`.

The private evidence label is `windows-launcher-metadata-e879066ad8a7`.
Its retained receipt SHA-256 is
`5fd1f40dc8caf85226a9ce047862d5c72a34ef0d6ab68aa02f96775ffc2a2c00`.
The existing selector hash matches the supplied binding, and the receipt binds
that same hash. The query argv uses its alias exactly once. The receipt's command,
stdout, and stderr hashes match their bytes. Parsed stdout equals metadata.
The last argv element decodes exactly to the retained query text as UTF-16LE.
The receipt records exit code 0, zero retries, and no product/controller execution.
These are retained query facts. This reviewer made no host query.

The retained registry value selects PowerShell 7. The command-option and
escape-arguments properties are null in the saved metadata. Null properties do
not by themselves establish which OpenSSH branch supplied defaults. The observed
parent chain directly supplies the wrapper's actual command and executable path.
It contains a Windows PowerShell child, its PowerShell 7 parent, three sshd
ancestors, then service/startup ancestors. All six saved parent links match.
The last two ancestors have null executable/command fields. No additional
command-bearing shell appears in this saved chain.

The wrapper executable path equals the registry path under case folding, but
not as exact text. A future exact template must use the observed serialized
spelling. A registry-derived spelling alone does not match the retained command.

All following sizes are complete UTF-16 code units including one NUL. The
submitted ASCII native line is called N here; its Base64 is never reproduced.

- N is 2,415 units for the retained query. Its flags and executable token match
  R3's transport shape. Joining its argv with spaces equals its Windows argv
  serialization because these particular elements need no argument quotes.
- The PowerShell 7 wrapper is exactly a 45-unit literal prefix, N without its
  counted NUL, and a one-unit literal suffix. N occurs exactly once. The prefix
  includes the quoted 38-unit executable path, the command option, separators,
  and the opening command quote. The suffix closes that quote. Its complete
  saved line is 2,461 units. This is one exact wrapper-template candidate.
- The child executable path is 57 units and matches the saved command-resolution
  result. The child's command uses that full path with two executable quotes,
  then exactly the submitted options and Base64. Its complete saved line is
  2,460 units, which is 45 units longer than N. This is an actual command-line
  boundary, even though it invokes the same Windows PowerShell executable.
- The three saved sshd lines are 42, 42, and 37 units. They do not contain N and
  are not per-payload command wrappers. They must not be invented as additional
  `prefix + N + suffix` layers.

R3 `transport_argv` measures each template independently as
`prefix + native_line + suffix`. The input contract requires every complete
Windows shell/wrapper layer to be represented once. The child line does not
contain N anywhere: its full-path replacement and closing executable quote
interrupt N before the options. Thus no prefix/suffix pair can equal that saved
child line with N unchanged. Adding up to four independent templates cannot
repair this internal replacement. An empty template repeats the short submitted
line. Padding a template or reducing a limit would invent a serialized line or
an actual limit. Neither satisfies this contract.

The child needs an explicit bound for its actual construction. Under the observed
unchanged ASCII construction, the outer wrapper is one unit longer than the
child for every payload length. An outer size check would therefore also bound
the child's size. This useful inequality does not meet the contract's separate
exact representation requirement. This review does not silently redefine that
requirement or claim an overflow from the 45-unit expansion.

Independent inert string extraction confirms the 1,588-unit R3 wrapper literal,
12,025-unit substituted maximum, 32,068-unit encoded maximum, and 66-unit native
prefix. Conditional on the observed construction continuing unchanged, the
maximum submitted line is 32,135 units, the expanded child is 32,180 units, and
the outer wrapper is 32,181 units. Their margins below the Windows API ceiling
are 632, 587, and 586 units respectively. These bounds apply to both original
and recovery transport payloads that pass the same R3 cap. No payload or native
parser was run to obtain these counts.

Microsoft documents the CreateProcessW ceiling as 32,767 including the terminal
Unicode NUL. That is an API ceiling, not proof that a particular installed shell
or launcher has no smaller limit. Its separate executable-name MAX_PATH
constraint is not a 260-unit whole-command limit; the observed 57-unit native
path is below it. See [CreateProcessW](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessw).

Microsoft documents the OpenSSH `DefaultShell` registry setting in
[OpenSSH Server configuration](https://learn.microsoft.com/en-us/windows-server/administration/openssh/openssh-server-configuration#configuring-the-default-shell-for-openssh-in-windows).
PowerShell documents string-command parsing and UTF-16LE encoded commands in
[about_Pwsh](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_pwsh?view=powershell-7.5)
and [about_PowerShell_exe](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_powershell_exe?view=powershell-5.1).
The Base64 alphabet used here contains no argument-mode metacharacters requiring
further expansion, as described in
[about_Parsing](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_parsing?view=powershell-7.5#argument-mode).

Primary source is consistent with the observed route: OpenSSH
[`do_exec_windows`](https://github.com/PowerShell/openssh-portable/blob/274893f8d1fe86c4d128b514c3af4bc10cdc0c99/contrib/win32/win32compat/w32-doexec.c#L279)
quotes the shell path and constructs the command option/wrapper; PowerShell
[`GetProcessStartInfo`](https://github.com/PowerShell/PowerShell/blob/v7.5.4/src/System.Management.Automation/engine/NativeCommandProcessor.cs#L1285)
uses the resolved executable path; and Microsoft's runtime
[`BuildCommandLine`](https://github.com/dotnet/runtime/blob/v9.0.10/src/libraries/System.Diagnostics.Process/src/System/Diagnostics/Process.Windows.cs#L613)
quotes that executable before appending arguments. These references explain the
construction. They are not bindings to the target's installed binaries. The
saved metadata does not identify those implementation versions or establish
all smaller implementation limits. The short successful query alone cannot
prove that a future near-ceiling command takes the same route without a smaller
limit. No actual per-layer limit is asserted by this review.

The exact missing prerequisites are (1) a contract-supported exact representation
of the expanded child command, or an explicitly reviewed contract that accepts
the proved child/wrapper size relation; and (2) evidence binding the installed
launcher/PowerShell implementation to the claimed per-layer limits. The child
path itself is already known. Repeating the metadata query cannot solve the
literal-template representability problem. No route/configuration/source change
is made or authorized here.

There is one observed exact wrapper-template candidate and zero complete
admissible template sets for this evidence and exact R3 contract. Consequently,
no preparation object or true admission assertions were materialized. No new
private binding/audit directory was created; the retained private input remains
the evidence source. Alias, selector hash, private paths, exact launcher literals,
and query payload are omitted from this public folder.

Only new review documents are committed. Validation consists of retained byte,
hash, JSON, Git-object, and string-arithmetic reads, plus primary documentation
reads. No supplied candidate/helper was executed or imported. No new remote
command, host/process/port query, native parser, test, build, signing, controller,
daemon, or CLI run occurred. Existing private files were not changed. All native
acceptance remains **UNVERIFIED**. The old **FAILED** result and zero-stop count
remain unchanged. The expired Sparkle window remains closed.
