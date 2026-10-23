# Combined binary idle observation — 2026-10-23

The new frozen daemon used by [combined virtual acceptance](combined-virtual.md)
was observed through its real CLI and private Unix socket. Source commit is
`f7b10e7550aa11200385b06b69f6dcb2328dd9b5`; binary SHA-256 is
`0fc946a8fdb45d666e86739242a719212cafe0fcabc7bc48557e6af4d36608cd`.
This observation is separate from the earlier checkpoint measurement.

One private virtual daemon ran with role off, no peers, TLS disabled, discovery
disabled, nice 15 and private config/socket/TLS/TMP directories set before startup.
Each phase lasted about 30 seconds and took 30 RSS/CPU samples at 1 Hz.

| Phase | Daemon RSS | Displayed CPU-counter increase | Status CLI calls |
| --- | ---: | ---: | ---: |
| Idle without status calls | 8.265625 MiB throughout | 0.00 s | 0 |
| One status CLI call each second | 8.296875–8.609375 MiB in samples | 0.00 s | 30 |

The `ps` CPU counter displays hundredths of a second. A displayed 0.00-second
increase does not prove zero CPU work. The counter covers only the daemon and
excludes CLI-process and other system CPU use.

The 30 status CLI round trips took 6.53–12.63 ms, with median 8.25 ms. This includes
CLI process launch and IPC. It is not native input or clipboard delivery latency.
Stop acknowledgement took 7.50 ms; actual process exit took 1.276 seconds. Exit
code was 0 and the private socket was removed. These are observations of this run,
not a speedup or memory-regression comparison.

The config and frozen binary stayed unchanged. Postflight found no matching
process or private TMP entry. An unrelated SMB mount-stat warning made the lsof
inventory incomplete. The warning is preserved. A direct IPv4 loopback bind,
immediately closed, proved the assigned port was available. No runtime measurement
was repeated to repair the export check.

Exact samples, command aliases, artifact/helper/report hashes and timing values
are in the [structured record](combined-idle-resources.json). Private raw records
remain available through its state pointer.

This short role-off virtual observation establishes neither native performance
nor loaded-peer resource use, long-term retention, leak freedom or release
readiness. Discovery, native input/clipboard, Windows migration, stopped IPC work,
security/provenance and distribution gates keep their separate status. No live
sharing or installed configuration changed. No unit tests or mocks ran.
