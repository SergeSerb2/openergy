# Trust-list scaling — 2026-10-22

## Reproduction

Baseline source: `4c9cb8c63a2b89c8b14ed273766adf09f5fea9ec`.
The trust reader searched every previously accepted pin for each valid record.
Bulk imports repeated that pattern for additions. Fingerprint formatting also
allocated a small string for every byte. Real commands showed increasing cost
within the existing 1 MiB policy; no oversize file or product mock was needed.

The final regression declares a **100 ms local lab budget**. CLI figures use
the median of three commands after executable warm-up. Each bulk sample restores
the original destination, so an actual overlapping import is never replaced by
a later no-op sample. Every sample checks exact output/order and persistence.
Live checks use one input probe during each unrelated trust update on real TLS
virtual peers with 13,000 pins per role. Mutation and status timings also have
to meet the budget. This is a declared lab check, not a product latency promise.

The baseline passed the small-list control and failed the other eight timing
cases. All baseline semantic checks passed: this was a scaling failure, not
incorrect trust or persistence. Earlier exploratory and intermediate results
remain in the [machine record](trust-scaling.json).

## Change

The sorting change is `66ca22f46a76a6345dd5aab8617b593739c74632`; formatting
is `c3e333bf92c5a10d4f1cc924faef6424501cb2b2`; the final bulk-memory refinement
is `0389e69e8b0dbc3420b062721c85aedc96f879dc`.

Long reads sort temporary indices by fingerprint and original position, mark
later duplicates, and retain the original vector order. Small lists keep their
existing linear path. Bulk imports search an index of existing pins and append
only new pins, in input order. Their additions come from the already deduplicated
reader; the real import cases include duplicate source records. No index or
cache is retained in the trust store.

Uppercase colon fingerprints and lowercase `v2:sha256:` records now use one
preallocated output string, instead of per-byte formatting allocations. SHA-256,
the pin format, role separation, invalid/SHA-1 handling, locking and persistence
policies remain unchanged.

The first sorting build passed eight of nine cases: compact revocation still
took 111 ms. Formatting then passed all nine, but no-op imports had about 2 MB
of extra peak memory from appending duplicate candidates. The final existing-pin
index removes that growth. These intermediate measurements used the earlier
single-sample bulk command; final baseline/candidate results below use the same
three-sample script. The original failure is not relabeled as a pass.

## Paired real-interface results

The committed [regression command](../../../crates/daemon/scripts/verify_trust_scaling.py)
starts actual CLI processes, private daemons, loopback TLS connections and public IPC:

```sh
python3 crates/daemon/scripts/verify_trust_scaling.py \
  --openergy target/release/openergy --port-range 30000-30019 \
  --report /tmp/trust-scaling.json
```

| Operation | Before, ms | After, ms |
| --- | ---: | ---: |
| Small list; 3 pins per role | 10.786 | 11.643 |
| List; 16,000 distinct pins per role | 190.018 | 36.942 |
| No-op bulk import; 13,000 pins per role | 331.467 | 28.861 |
| Overlapping import; 12,000 final pins per role | 187.680 | 38.620 |
| Refused canonical expansion; 16,000 source pins per role | 295.613 | 20.831 |
| Revoke from compact 16,000-pin lists | 363.745 | 48.771 |
| Input during server approval | 209.256 | 0.532 |
| Input during client approval | 206.412 | 0.604 |
| Input during unrelated server revocation | 267.834 | 0.507 |

The final binary passed **9/9 cases**. Live mutation replies took 14.0–23.8 ms
and status replies 0.251–0.277 ms. Exact role files, configuration and the held
key stayed correct, and every owned daemon/socket was cleaned up. Imports kept
their source bytes unchanged; refused saves preserved destinations and left no
temporary output. Compact revocation still used the allowed compact encoding.
Duplicate and legacy records kept the same first-seen, nonmatching-SHA-1 behavior.

The small-list timing did not improve in this run. The large-list command's peak
RSS increased from 9,551,872 to 9,781,248 bytes, a 229,376-byte temporary-index
cost. No-op import peak RSS was 12,140,544 before and 12,042,240 after; overlapping
import peaks were equal at 13,352,960 bytes. These per-command observations do
not establish universal memory behavior.

The daemon increased from 3,674,000 to 3,690,816 bytes: 16,816 bytes, about 0.46%.
No dependency or worker was added. The locked release build, strict release
workspace/all-targets Clippy and formatting passed. The final binary also passed
the **6/6 trust-lock/concurrency**, **20/20 identity-file** and **15/15 daemon/CLI**
checks. Exact final-suite results and 112 tracked Rust/Cargo/toolchain file hashes
are in the machine record.

The final full virtual suite passed **89/89** with zero cleanup failures. **30
Docker scenarios were explicitly excluded**, not counted as passes; the Docker
socket was unavailable. S15 measured server/client idle CPU at 0.050%/0.017%
over separate 60-second windows, peak RSS of 9,437,184 bytes each, and p99
0.783 ms for 1,000 plaintext virtual-loopback moves. These separate samples do
not establish a physical-input or comparative product speedup.

## Scope

Checks used macOS 27.2 ARM64, Rust 1.99.1, minimum macOS 15, two Cargo jobs at
reduced priority, private temporary state and assigned loopback ports. The paired
timing commands ran sequentially, without a concurrent build. The host was not a
controlled benchmark appliance. These figures compare Openergy revisions; they
do not measure Synergy or physical input-to-screen latency. Filesystem work is
still synchronous.

No unit tests or mocks were added. Installed software, live sharing, native
input/clipboard, permissions, login items, services, tasks and port 24800 were
preserved. Native, security-review and distribution gates remain open. GitHub
private reporting setup remains deferred.
