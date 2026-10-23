# Bounded virtual idle resource observation — 2026-10-23

One private real daemon used the frozen checkpoint binary from source
`ca618418dc8c4d93b8fd04eeda3fefa01003b8e7`. It ran the virtual backend with role
off, discovery disabled, TLS disabled and no peer connections. It never captured
or injected native input or clipboard. This is a bounded observation, not a
native performance benchmark or sustained memory-leak verdict.

The executable is 3,725,472 bytes, SHA-256
`eae3201b0e793c3533afe385a1f6c3d5eaa9992eff3a4cbd4d1a433134768367`.
Its hash was checked before and after use. The ordinary daemon and CLI use real
private Unix control sockets. The child ran at nice15 with new private config,
TLS/state and temporary paths. HOME and installed/live sharing state stayed
unchanged. Role off opened no configured peer listener.

## Observations

| Same process phase | Wall duration | Daemon CPU-time delta | RSS before / after | Status calls |
| --- | ---: | ---: | ---: | ---: |
| Idle, no status polling | 30.014 s | 0.00 s at displayed resolution | 8.25 / 8.25 MiB | 0 |
| One CLI status call per second | 30.011 s | 0.01 s | 8.25 / 8.58 MiB | 30 |

The idle CPU counter did not increase at ps's displayed 0.01-second resolution;
this does not mean the daemon did zero work. The second phase's displayed delta
is about 0.033% of one core over that interval. It excludes the separate CLI
processes and has coarse timing resolution. The machine was not isolated for
benchmarking. No comparison with Synergy or another build is claimed.

RSS samples came from ps once per second. Idle RSS was exactly 8,650,752 bytes
for all 30 samples. Polled RSS ended at 8,994,816 bytes, an increase of 344,064
bytes. These are resident-memory samples, not allocator capacity or a hard
high-water mark. Two short phases in this order do not prove long-term stability
or attribute the small increase to a leak.

The real stop command acknowledged in 7.50 ms; process exit took 1.262 s and
returned zero. Its private socket disappeared. The pending unintegrated IPC
candidate's faster shutdown is not claimed for this checkpoint. Private config
and the executable kept their hashes, and postflight found no owned child.
[Samples, exact command and identities](virtual-idle-resources.json).

## First helper result and safety

The first helper exited 1 before measurements because it treated successful
`ctl --raw` output as a response envelope. The real CLI returns the result object
on success. The corrected wrapper reads role directly. The first helper/hash
and error remain in the private evidence indexed by
`/tmp/oe-idle-resource-20261023-first-state.json`. Its finally block sent SIGTERM
only to its own private child; the daemon logged a graceful stop and removed
its socket. No first-attempt measurement is counted as passing evidence.

The final executed helper and all original samples/logs remain in the directory
indexed by `/tmp/oe-idle-resource-20261023-state.json`; the record contains its
exact source hash. No unit tests, mocks or computer-use checks were used. This
measurement does not resume or run the provider-stopped IPC/import or expiry/
provenance tasks. Native input/clipboard, Windows startup acceptance, positive
migration and trusted distribution keep their separate open gates.
