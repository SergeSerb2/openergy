# Identity-file resource bounds — 2026-10-22

## Reproduction

Source `a0447c3a8c7d8210d6ef5d6a8bb85fe14708b1e0` passed private identity
files directly to the PEM stream parser without a size limit. A real offline
`fingerprint` command accepted a valid certificate followed by 16 MiB of ignored
padding. Its peak resident memory rose from about 8.2 MB for a normal file to
41 MB. Padding before a valid key also increased memory. Padding after the first
key was ignored without reading the whole file. These checks used private
synthetic identities and the actual release command, not a substituted parser.

The final same-script baseline had **10 passing controls and 10 failures**.
All four valid files just below or at the new limit passed, as did all six
existing non-regular-file refusal controls. The failures covered oversized
reads, startup, live TLS changes and generated output. The baseline accepted
oversized live files and committed the TLS change instead of preserving the old
configuration/session under the new size policy.

The read fix exposed a writer problem. On intermediate source
`1a4ea4a01214eb4ba680e724ccde222c4c4b5a34`, a 1 MiB configured name caused
generation of a 2,840,392-byte certificate. Startup refused the oversized file
only after saving both the key and certificate. The 20-case command passed
19 cases and failed this new writer regression. That failure is retained.

## Fix

The read change is `1a4ea4a01214eb4ba680e724ccde222c4c4b5a34`; generated-output
preflight is `bbaf747e21f138c5ed6d2aba0e2a17912766d648`.

Each certificate and key PEM file is now limited to **1 MiB (1,048,576 bytes)**,
including ignored text. The reader checks the opened file's length, reads at
most the limit plus one byte, checks the consumed length, then parses the bounded
slice. The second check rejects growth after metadata rather than silently
accepting a truncated input. That growth race was reviewed in source, not forced
in this run. Existing larger files now produce an explicit size error and are
not replaced.

Both generated PEM outputs are checked before either is saved. The very-long-name
case leaves neither key nor certificate behind; changing to a normal name allows
creation and startup. Existing creator locks, key/certificate matching and
partial-identity refusal remain in place.

Unix private TLS file opens now reject symlink traversal, use nonblocking open
and verify the opened handle is regular before changing its permissions or
reading. Static FIFO, symlink and directory cases passed before and after the
change. A separate 80-trial real path-replacement probe found no baseline stall;
this record does not claim to have forced that race or established new Windows
path-race acceptance. Ordinary filesystem operations remain synchronous.

## Real-interface verification

The committed [file command](../../../crates/daemon/scripts/verify_identity_files.py)
starts actual CLI processes, virtual daemons and loopback TLS peers:

```sh
python3 crates/daemon/scripts/verify_identity_files.py \
  --openergy target/release/openergy --port-range 28400-28419 \
  --report /tmp/identity-files.json
```

The final binary passed **20/20 cases**. Certificate and key files of 1,048,575
and 1,048,576 bytes kept the same fingerprint. Files of 1,048,577 and 16,777,216
bytes were refused, including ignored bytes after a valid key. The same-script
16 MiB certificate case decreased from 41,091,072 to 7,405,568 peak resident
bytes; the prefixed-key case decreased from 24,870,912 to 7,520,256 bytes. These
are per-command macOS measurements for those fixtures, not a general idle-memory
or product comparison.

Startup refused oversized existing files and removed its owned socket. Live
server `config.patch` and pinned-client `connection.connect` refused before
changing configuration, peers or held input; a new input probe still arrived.
Restoring the valid file allowed retry and released old held input on reconnect.
All file bytes stayed intact on refusal. The generated-output case recovered
with a normal configured name and created a complete valid pair. Every owned
daemon and socket was cleaned up.

The final source also passed the **4/4 identity-lock** and **6/6 trust-lock/
offline-concurrency** matrices and expanded daemon/CLI audit **15/15**. Three
additional real CLI checks refused corrupt certificates, corrupt keys and a
mismatched pair, preserved all bytes, and recovered the original fingerprint
after restoring the valid file. The locked release build, strict release
workspace/all-targets Clippy and formatting passed with Rust 1.99.1 and minimum
macOS 15. No dependency or worker was added. The daemon decreased from
3,674,160 to 3,674,000 bytes.

The final binary passed the full virtual suite **89/89**, with zero cleanup
failures. **30 Docker scenarios were explicitly excluded**, not counted as
passes; the Docker socket was unavailable. S15 measured server/client idle CPU
at 0.033%/0.017% over separate 60-second windows and peak resident memory of
9,420,800/9,388,032 bytes. The 1,000-event plaintext virtual-loopback observation
had p99 1.004 ms. This is an external IPC-to-receiver upper bound, not physical
input-to-screen latency or a Synergy speedup comparison.

Exact final-suite results, binary/script hashes, intermediate failures and 112
tracked Rust/Cargo/toolchain file hashes are in the
[machine record](identity-file-bounds.json).

## Scope

Runtime checks used macOS 27.2 ARM64, virtual input/clipboard, private temporary
state, assigned loopback ports and production-port denial. Build concurrency was
two jobs at reduced priority. No unit tests or mocks were added. No installed
process, live sharing connection, native permission, clipboard, login item,
service or task was changed. Native, security-review and distribution gates
remain open. GitHub private reporting setup remains deferred.
