# Integrated Linux verification — 2026-10-22

**Source:** `b36e840ea40b0c455db41afa6617d376b3b31898`, exported with
`git archive` to a fresh owned temporary directory. The parent checkout was not
used for execution. The exported source was mounted read-only; the existing
Linux helper entrypoint copied it into the container's disposable `/work`.

**Environment:** ARM64 Debian Bookworm, `aarch64-unknown-linux-gnu`, existing
`oe-linux-dev-wp8:rust-1.99` image and existing Cargo/target cache volumes.
The actual compiler reported `rustc 1.99.1 (58cf8f9c3 2026-10-13)`; Cargo was
1.99.1. The archive's `rust-toolchain.toml` selected that installed toolchain.
Commands inherited niceness 15 and `CARGO_BUILD_JOBS=2`.

## Results

- Locked release workspace build, including the daemon and external runner: PASS.
- Strict release-mode workspace Clippy, all targets: PASS, zero warnings.
- Release metadata/version consistency check: PASS.
- Public CLI/real virtual IPC smoke: PASS. This exercised shipped version,
  absent-daemon error, status, invalid-method error and graceful daemon stop.
- Requested external E2E selection: **9/9 PASS**, zero exclusions and zero cleanup
  failures: A5, Q13, Q18, Q19, N2, S2-edge-mapping, S2-wire-order, S8 and D2.
- Focused Xvfb `clipboard` and `recovery`: PASS. Coverage included 10,498,508-byte
  INCR transfers both ways, independent RGBA comparison, PRIMARY isolation,
  unsupported/refused/missing/changing reads, real clears and zero-byte text/HTML,
  grab rollback, local input ownership, lock keys, Unicode mapping restoration,
  1,000 own-write stamps, 32 destroyed requestors, retry after an oversized read,
  and 32 backend drops with descriptors/threads returning to baseline.

## Commands

The existing release script, helper scripts and daemon verification scripts were
read first. Inside the isolated container:

```sh
rustc -V
rustc -Vv
python3 tools/release/release.py check
target=aarch64-unknown-linux-gnu
cargo build --release --locked --workspace --target "$target"
cargo clippy --release --locked --workspace --all-targets --target "$target" -- -D warnings
```

The inspected `tools/release/release.py` was imported and its public-CLI smoke
function called as `smoke(Path(binary), "26000-26019")`, with `binary` set to
`/target/aarch64-unknown-linux-gnu/release/openergy`. Then:

```sh
for prefix in A5 Q13 Q18 Q19 N2 S2 S8 D2; do
  /target/$target/release/openergy-e2e run \
    --openergy /target/$target/release/openergy --scenario "$prefix-*" \
    --no-docker --keep --port-range 26000-26019 --lab-port-range 26020-26039 \
    --report "/evidence/e2e-$prefix.json"
done
bash tools/linux-dev/verify.sh clipboard
bash tools/linux-dev/verify.sh recovery
```

The lab range was configured but unused. The requested scenarios used virtual
daemons/raw peers on container loopback. Every daemon had explicit private
configuration, TLS and socket paths and `--platform virtual`. Xvfb probes used
the backend directly, without a native daemon. No host desktop/clipboard,
port 24800, SSH, system configuration, production process or published port was
used. No unit tests or mocks ran. The owned container was removed after exit.

## Integrity and limits

All **477 archived source files** matched their initial content hashes after the
run. Cargo.lock matched the git object before and after, with SHA-256:

```text
7c293887f5785f252387741d9e47ffab39e016e699ad0e14ed3c29992c5bb2d0
```

Executed release binary SHA-256 values:

```text
openergy      31a4bc5986cb4d550740e9879c72cdad53f95751a23eaa741c603ea47830e21f
openergy-e2e  9ed9891dab5f8f4a8e73b8b8ec8c4812569199ced6b9c0dbfd2d927b4d9ce8dc
```

Raw logs, JSON reports, scenario artifacts and source manifests remain in the
owned temporary evidence directory. Only this sanitized report is committed.
No integration defect was found and no product code was changed.

The release script's `native()` guard reported
`unsupported native release host: aarch64-unknown-linux-gnu`; its supported Linux
archive host is x86_64. Thus the top-level release `verify`/packaging workflow and
x86_64 Linux distribution were not verified. The requested build, lint, smoke and
selected E2E checks were run directly on native ARM64 with the pinned compiler.
Physical desktops, external devices, native-daemon operation, Docker peer
interop and the rest of the scenario catalogue remain outside this run.
