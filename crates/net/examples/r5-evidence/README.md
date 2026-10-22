# Round 5 transport and X11 image allocation audit

Base runtime source: `dfed3e9792731eae1c654f8effd0314c9bc1d0fa`.
Fixed code and external probes: `22edbad37881a6b05c0bb91f1cff2b0670331c79`.
The following evidence commit changes records only. This is a focused audit,
not a public-release or native-desktop endorsement.

## Confirmed issues

### Tiny Control packets bypassed the practical queue memory bound

A real virtual client connected to a raw loopback TCP server. The server sent
keepalives while it stopped reading the client's replies. The transport charged
each reply's eight encoded bytes, but omitted `Packet` storage and queue growth.
Its byte budget could admit more than a million tiny replies while omitting
their much larger packet storage.

The same six-second external command now charges encoded allocation capacity
plus packet metadata and queue-growth headroom. The per-connection high-water
budget also counts packet overhead. The 256 KiB Bulk wire window and reserved
Control allowance remain in force. Packet selection and causal ordering were
not changed.

- Before: maximum sampled RSS **134,627,328 bytes**; growth **125,681,664 bytes**.
- First fixed run: maximum sampled RSS **24,379,392 bytes**.
- Committed-source replay: maximum sampled RSS **20,611,072 bytes**; growth
  **11,730,944 bytes**.
- The baseline fails the probe's 64 MiB growth threshold; both fixed runs pass.

See `queue-before.json`, `queue-after-first.json`, and `queue-after.json`.
Both fixed runs have the same binary SHA-256. RSS is sampled about every 200 ms;
it is not a kernel high-water measurement. The 64 MiB gate is a local regression
threshold, not a whole-daemon memory promise. IPC remained usable, but this
experiment does not show a CPU or latency improvement.

### Small indexed DIBs expanded beyond the X11 image envelope

The real public `X11Clipboard::write` path accepted a valid 4096-square, 1-bpp
DIB of **2,097,200 bytes**. Real `xclip` then read a **67,118,148-byte PNG**,
which exceeds 64 MiB. The kernel-reported child peak RSS was **276,992,000
bytes**. This ran in Docker/Xvfb with a 512 MiB memory limit.

X11 image writes now require decoded RGBA and each PNG/BMP representation to
fit **64 MiB**. DIB-to-PNG checks the exact stored-DEFLATE output length before
allocating RGBA or BMP output. PNG writes also bound decoded BMP size. A failed
conversion occurs before selection ownership or the change stamp changes.
This is a deliberate image-write limit, documented on `write_selection`.
The existing 256 MiB backend input/INCR safety limits remain; this does not
change the daemon's 64 MiB clipboard policy.

The R5 public-backend probe rejects both the 4096-square image and an
8192-square, 1-bpp DIB of **8,388,656 bytes**. After each rejection, real `xclip`
still reads the prior text selection and the backend's stamp is unchanged.
The fixed child peak RSS was **18,833,408 bytes**.
See `x11-before.json`, `x11-after.json`, and their logs for exact child RSS,
binary hashes, exit status, and source.

## Real verification

- `focused.json`: **12/12** real daemon/TCP scenarios; zero exclusions and
  cleanup failures. Covers input backpressure/release ordering, clipboard queue
  pressure, reconnect, policy cancellation, empty clear, cap enforcement,
  text, large DIB, and malformed clipboard containers.
- `tls.json`: **2/2** real virtual TLS checks, including verified trust status
  and trust approval before application traffic. Total daemon/TCP checks: 14/14.
- New `R5-incoming-clipboard-cancellation`: a declared 64 MiB Start stays below
  a 32 MiB RSS gate; a newer grab discards a complete payload awaiting End;
  a fresh Start/Data/End then succeeds. No change was justified in
  `crates/session/src/clipboard.rs`. The scenario now lives in the standard
  `tools/e2e/scenarios` catalogue so normal virtual verification runs it.
- X11 `clipboard`: real small-image, text, HTML, and DIB/BMP transfers;
  **10,498,508-byte PNG INCR** in both directions; independent Python pixel
  comparison.
- X11 `clipboard-recovery`: 1000 own-write stamps, 32 destroyed INCR
  requestors, and a successful read after an oversized INCR abort.
- Locked release builds, strict Clippy for net/session and all Linux platform
  targets, rustfmt check, and diff whitespace check pass.

Host builds used `nice -n 15`, `CARGO_BUILD_JOBS=2`, and an R5 target directory.
Docker's entrypoint enforces the same niceness and job count. All host daemon
commands used explicit temporary config/socket/TLS locations and virtual mode.
TCP ports were 26200 and 26202-26219; the external runner's unused lab range was
26220-26239. Xvfb used its private Unix socket with TCP disabled.

## Reproduction

From the repository root:

```sh
CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=/tmp/oe-r5-net-target nice -n 15 \
  cargo build --release --locked -p openergy-daemon -p openergy-e2e
nice -n 15 python3 crates/net/examples/r5_queue_memory.py \
  --openergy /tmp/oe-r5-net-target/release/openergy --port 26200 \
  --source 22edbad37881a6b05c0bb91f1cff2b0670331c79 \
  --report /tmp/oe-r5-queue.json
nice -n 15 /tmp/oe-r5-net-target/release/openergy-e2e run --no-docker \
  --scenario 'R5-incoming-clipboard-cancellation' \
  --port-range 26202-26219 --lab-port-range 26220-26239 \
  --openergy /tmp/oe-r5-net-target/release/openergy \
  --report /tmp/oe-r5-cancellation.json
```

Run the Linux probe through the existing isolated Linux-dev image
`oe-linux-dev-wp8:rust-1.99` (image ID
`sha256:aa40e749da1db39335f227757e52ea058a0a1e9d10883d59e23f9aab34a81886`).
Use a private R5 target volume. Copy existing dependency artifacts into that
volume if a cold release build exceeds the build container's memory limit.
Touch the owned crate files inside the disposable copy to force their rebuild.
The image's entrypoint copies the read-only
source into `/work`, starts Xvfb `:99` without TCP, and applies niceness/job
limits. Build `openergy-platform-linux --example x11_probe --release --locked`,
then run:

```sh
OPENERGY_X11_PROBE_ISOLATED=1 python3 /work/crates/net/examples/r5_x11_resources.py \
  --probe /target/release/examples/x11_probe \
  --source 22edbad37881a6b05c0bb91f1cff2b0670331c79 \
  --report /artifacts/after.json
bash /work/tools/linux-dev/verify.sh clipboard
bash /work/tools/linux-dev/verify.sh clipboard-recovery
cargo clippy --locked -p openergy-platform-linux --all-targets -- -D warnings
```

For the red run, only `image.rs` and `clipboard.rs` from the base commit were
copied over the disposable container's source before building the same R5
probe. `wait4` measures the actual probe child, excluding prior Cargo usage.
A shared-target replay reused the baseline artifact despite restored source;
that run was rejected by its failing result and unchanged binary hash. Final
fixed evidence uses a separate R5 target volume with copied dependency caches
and a forced rebuild of the owned crate. A cold build hit its container memory
limit; it was not counted as a validation pass. The final image stress and
correctness checks ran with the 512 MiB limit. Source SHA-256 values appear in
`x11-after.log` and match the committed files. Do not accept a cached green
build as proof of the source or the resulting behavior.

## Limits and handoff

These checks do not bound total daemon RSS, kernel socket buffers, TLS buffers,
all 16 simultaneous connections, or retained allocator/queue capacity. Valid
images near the 64 MiB conversion limit can still use several image buffers;
the PNG encoder still uses stored DEFLATE and was not replaced with streaming
compression. No compression dependency was added. The Linux README's generic
256 MiB backend-cap description should also mention the new 64 MiB image-write
limit when the documentation owner updates release guidance.

Native host input/clipboard/TCC, installed applications/state/services, SSH,
production ports, and the desktop app were not used. Native Windows/network
matrices and the parent's full integrated suite were not rerun here. No
manifest, lockfile, toolchain, or MSRV change was made. Nothing was pushed and
no PR was created.
