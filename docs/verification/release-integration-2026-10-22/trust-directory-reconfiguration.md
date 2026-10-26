# Retained trust directories over real TLS — 2026-10-22

## Scope

The [held-peer refresh record](trust-refresh.md) left one runtime path open:
connections can retain their original TLS directory after a direct
`NetDriver::reconfigure` call. A shared refresh snapshot must not apply the
current directory's trust decisions to those older connections.

Source commit: `6b257db359d5a5014c3f5d59d05697147f2a099f`.
This follow-up adds an [end-to-end transport CLI](../../../crates/net/examples/trust_directories.rs)
and an opt-in `--echo-control` mode to the existing
[separate peer CLI](../../../crates/net/examples/tls_peer.rs). It uses the public
transport API, actual child processes, TCP, mutual TLS, framed traffic, private
files, and real offline `openergy fingerprint` / `trust approve|revoke` commands.
The wrong-role fixture uses the public `TrustStore::trust_server` file API.
There are no unit tests, mocks, native backends, input or clipboard operations.

No production runtime change was needed. The tested runtime includes the
snapshot change in `e1798698320ebf1fb3b45e4005e1abc5c767666a`.

## Real-interface results

All four scenarios passed. Each keeps one driver, its original waker and handle,
and the same listener. Two peers connect under directory A, then a new peer
connects under directory B after reconfiguration. Surviving peers must return
control frames after each mutation. A removed peer must report actual stream
closure from its separate process.

| Scenario | Required result observed |
| --- | --- |
| Retained-directory revoke and admission | Revoking one A peer closes it; its A sibling and the B peer still exchange frames. Reapproving in A does not admit a new B connection. A wrong-role B pin also leaves it held. Correct-role approval in B admits it. |
| Current-directory read failure | Invalid UTF-8 in B closes only the B peer. A peers keep exchanging frames. Global and B-connection error events appear, with no A-connection error. Restore and reconnect succeeds. |
| Retained-directory read failure | Invalid UTF-8 in A closes both A peers while the B peer still exchanges frames. Errors identify only the A connections. Restore and reverse reconfiguration keeps the now-retained B peer live and admits the A peers again. |
| Failed reconfiguration preparation | Invalid target trust data rejects the switch and preserves the original identity and established connection. A new connection still uses A's factory. Restoring B permits the switch and all peers exchange frames. |

These mutation checks send `ReloadTrust` through the original public
`NetHandle`. They verify directory selection and decisions without waiting for
the periodic timer; they do not measure a 500 ms refresh deadline. The earlier
daemon cohort independently exercises periodic refresh with one directory.

The CLI also passed **8/8 refusal checks**: missing arguments, nonempty root,
unsafe port, symlink root, group/other-accessible root, and echo mode combined
with message mode, hello-only mode, or server role. Each refusal preserved its
fixture; the unsafe-port argument was rejected before any bind or file creation.

The existing [lifecycle CLI](../../../crates/net/examples/lifecycle.rs) passed
both its TLS and plaintext cycles using the new peer binary with echo mode
disabled. It verified returned frames, delivery after `StopListening`, shutdown
closure, and reuse of the original driver/waker/handle across TLS disablement.
All owned children exited and all assigned listeners closed.

## Reproduce

Build the examples with the pinned Rust 1.99.1 toolchain. The daemon executable
is needed only for offline identity and trust commands. This run reused the
unchanged release daemon identified below.

```sh
CARGO_BUILD_JOBS=2 MACOSX_DEPLOYMENT_TARGET=15.0 nice -n15 \
  cargo build --locked --release -p openergy-net \
  --example tls_peer --example trust_directories --example lifecycle
umask 077
lab_dir=$(mktemp -d /tmp/openergy-trust-dirs.XXXXXX)
nice -n15 target/release/examples/trust_directories \
  "$lab_dir" 25090 target/release/openergy
```

Use an existing empty private real directory. The first assigned port must be
25080–25096; the CLI uses it and the next three loopback ports. This run used
25090–25093. The separate legacy lifecycle run used a different private root
and its fixed integration ports 24974–24975. The directory run had a 45-second
outer timeout for its owned process group and finished normally. No child
process remained after either run. Fixture keys and raw peer logs stay private
and are not committed.

## Source, artifact and validation scope

Locked release builds of both new/changed examples and the legacy example,
strict release workspace/all-targets Clippy (`-D warnings`), formatting and
diff checks passed. Builds used two Cargo jobs, reduced priority, minimum
macOS 15.0, and macOS 27.2 ARM64.

The [machine record](trust-directory-reconfiguration.json) preserves all
113 tracked Rust/Cargo/toolchain hashes, binary hashes, output and report
hashes. Of the prior record's 112 inputs, only `tls_peer.rs` changed; the
new input is `trust_directories.rs`. No runtime source, manifest, lockfile,
dependency, or daemon artifact changed.

A clean fresh local clone of the source commit matched all 113 input hashes.
All 31 local Markdown links in the three affected records resolved. The six
changed source/evidence files passed the personal-path and private-key-marker
scan; raw TLS identities remain outside the repository.

The daemon remains **3,707,520 bytes**, SHA-256
`4ba143a7f99f03e2a0c5a844b6c40589ac10dc0c8543d6ca3f3f0341effc3091`.
The prior **89/89** full virtual-suite pass and **30 excluded Docker cases**
therefore remain exact-artifact evidence. That full suite was not rerun for
these example-only changes. This follow-up makes no new speed or size claim.

No new Linux, Windows or native acceptance ran. Installed software, live
sharing, native input/clipboard, permissions, services, tasks and port 24800
were preserved. Native, security-review and signed/notarized distribution
gates remain open. GitHub private reporting setup remains deferred.
