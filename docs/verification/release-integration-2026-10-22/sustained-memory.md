# Sustained workload and diagnostic memory control

Two real-process checks used the packaged release daemon from clean source
`655e59f69c490841dc2cd2c8128eca51545c15f7`, SHA-256
`1405abcc08a4f8b175d4116a413ac0426c5e708578e9e250487ff13d18ae97e5`.
The observer checkout was `2b902d9921d3dd5c93e997edb0b0335594fcde44`.
Its six documentation-only differences were checked against 514 other tracked
files. No build or source change occurred during either check.

Both checks used two owned virtual daemons, private config/control/TLS/XDG/temp
paths, mutual TLS approved through the real CLI, discovery disabled and custom
loopback ports. Niceness was 15. No installed app, native input or clipboard,
desktop, service, login item or production port was used. No unit tests or
mocks were added.

## Declared workload gate

The first run completed 90 cycles in 270.006 seconds. It verified 180 transfers
of 1–8 MiB, totaling 684 MiB, with new remote write stamps and exact byte counts.
Thirty-six transfers also compared complete remote content hashes. It verified
90 cancellations, 180 genuine empty clears, 90 input batches, live config
patches and nine reconnects with held-key/button release recovery.

The gates were declared before the run: sampled per-process peak RSS at most
96 MiB, and late median growth at most 16 MiB over the post-warmup reference.
Both passed. Periodic samples do not establish an absolute unsampled peak.

| Measurement | Server | Client |
| --- | ---: | ---: |
| Reference RSS, cycles 11–20 median | 14,213,120 B | 14,254,080 B |
| Late RSS, cycles 81–90 median | 18,407,424 B | 18,694,144 B |
| Late minus reference | 4.00 MiB | 4.23 MiB |
| Periodic sampled peak | 37.89 MiB | 34.06 MiB |
| Fitted checkpoint trend | 1.3334 MiB/min | 1.3330 MiB/min |

The positive trend prompted a separate causal control. Passing these gates did
not establish flat memory use. The original driver read complete diagnostic
history several times merely to obtain stats.

## Polling control

The second run completed 90 cycles without flow failures, taking 290.413 seconds
including setup and cleanup. It repeated the transfer mix and first filled each
diagnostic ring through 5200 actual virtual input events. Each ring stayed at
4096 records and 393,216 reserved bytes.

Stats reads used `since=u64::MAX`; event reads used sequence cursors. Two extra
history reads per node per cycle then selected stats or full history in five
phases: stats/stats, full/stats, stats/full, full/full, stats/stats. The first four
phases had 20 cycles each; the final phase had ten. This reproduced the polling
effect separately in both roles. Stats replies were 321–324 bytes; full replies
were approximately 864–960 kB.

Switching to full polling raised settled RSS by 18.37 MiB on the server and
18.09 MiB on the client. OS allocation snapshots explain most of that step:

| Darwin malloc measurement | Before full polling | After full polling |
| --- | ---: | ---: |
| Server resident pages | 4928K | 23.1M |
| Server allocated blocks | 725K | 730K |
| Client resident pages | 4560K | 22.5M |
| Client allocated blocks | 715K | 715K |

These are the OS tool's rounded display values. Full-history serialization left
mostly free or fragmented allocator pages resident. It did not retain tens of
MiB of live clipboard data. With both nodes reading full history, consecutive
ten-cycle RSS medians fell from 33,603,584 to 31,916,032 bytes on the server and
32,849,920 to 32,006,144 bytes on the client. Some pages were swapped. Malloc
zone sizes, region counts and resident-plus-swapped sizes also stopped expanding
within this control, so the conclusion does not depend only on falling RSS.

Small live-allocation growth remained: roughly 17K server and 1K client after
the first phase. Allocation stacks were not captured. The logger's bounded
1000-line ring was not saturated. This control identifies the diagnostic
contribution; it does not classify every byte of the original trend or prove a
bound for every workload. No unbounded clipboard retention was reproduced.

## Cleanup, evidence and limits

Both pairs exited cleanly, removed their sockets and had temporary private keys
removed. A later independent PID readback found no owned process. Binary hashes
were unchanged. The control preserved all 61 original evidence files.

The sanitized [machine record](sustained-memory.json) retains measurements,
plans, provenance and raw-evidence hashes. Raw synthetic drivers, samples and
OS snapshots remain local. Preliminary niceness, port-preflight and socket
readiness errors were corrected in the drivers before accepted runs; they are
not reported as product failures or silently discarded.

Use stats-only or cursor-based diagnostic polling as described in
[Virtual IPC](../../../crates/daemon/VIRTUAL-IPC.md). Native desktop endurance,
hardware input latency and a matched Synergy runtime comparison remain outside
these checks.
