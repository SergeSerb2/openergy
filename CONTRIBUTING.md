# Contributing

Openergy is an alpha project. Start with [README.md](README.md), the
[setup guide](docs/plan/setup.md), and [readiness limits](docs/plan/readiness.md).
Small changes with a clear reproduction and real verification are easiest to
review. Follow the [code of conduct](CODE_OF_CONDUCT.md).

## Report a problem

Use a bug report for non-sensitive problems. Include the source commit, exact
OS build, architecture, backend, host/client roles, peer versions, and whether
TLS was enabled. State what you expected, what happened, and how to reproduce
it in a disposable setup. Attach only redacted logs. Do not include private
keys, credentials, personal clipboard contents, or other people's input.

Report possible vulnerabilities through [SECURITY.md](SECURITY.md), not a public
issue with exploit details. A passing build or virtual test is not proof of
native desktop compatibility.

## Prepare a checkout

Install Git and Rust/Cargo. Use the pinned Rust toolchain when present; the
maintainer build procedure in [docs/releasing.md](docs/releasing.md) defines
release prerequisites. macOS app work also needs Swift 6 and the macOS SDK.
Windows native builds need the MSVC C/C++ build tools and Windows SDK. X11
verification needs an isolated X server; Docker lab prerequisites are listed
in the respective lab README files.

```sh
git clone https://github.com/SergeSerb2/openergy.git
cd openergy
export PATH="$HOME/.cargo/bin:$PATH"
CARGO_BUILD_JOBS=2 nice -n 15 cargo build --release --locked --bin openergy --bin openergy-e2e
```

Do not change dependency versions or the lockfile as a side effect of a docs or
unrelated behavior fix. Explain dependency and protocol changes in the PR.

## Protect the user's input connection

The installed app may be someone's only working keyboard and mouse connection.
Never use it or its account state as a development fixture.

- Use only processes, files, configs, sockets, TLS identities, and containers
  created for your run. Keep them under a new private temporary directory.
- On a working Mac, use `--platform virtual`. Do not request macOS permissions,
  inject desktop input, or read or write the native clipboard for verification.
- Supply explicit daemon `--config` and `--socket` paths. TLS state follows the
  config's parent directory. Offline identity/trust commands need that same
  directory via `--dir`; control commands need the same `--socket`.
- Do not bind or connect port **24800** in tests. Reserve separate local and
  lab ranges. Do not set `OPENERGY_ALLOW_PORT_24800=1` during verification.
- Do not stop installed Synergy/Openergy, edit login items, or change live
  Application Support, preferences, trust, or service settings.
- Native desktop acceptance belongs on a separate test machine or account,
  with a person who can recover local input. Record OS build and permissions.

The existing external runner supplies isolated paths and confines local nodes
to loopback. Read [its safety rules](tools/e2e/README.md) before changing it.
Do not bypass a runner safety refusal to make a scenario pass.

## Verify a change

Do not add unit tests, mocks, or isolated-function tests. Exercise the real
command, IPC API, network connection, or app flow that the change affects. Add
an external TOML scenario when it proves a behavior through real daemons.
Include a failure path when practical. Leave existing tests in place.

From the repository root:

```sh
cargo fmt --all -- --check
CARGO_BUILD_JOBS=2 nice -n 15 cargo clippy --release --locked --workspace --all-targets -- -D warnings
target/release/openergy-e2e list
target/release/openergy-e2e run --scenario 'S0-*' \
  --port-range 25400-25419 --lab-port-range 25420-25439 \
  --report /tmp/openergy-smoke.json
```

Choose unused ranges for each concurrent worker. Run the scenarios that cover
your change as well as the smoke flow. The full catalogue also needs the
interop image and Docker; follow [tools/e2e](tools/e2e/README.md) and
[tools/interop-lab](tools/interop-lab/README.md). Do not call an unavailable lab a
pass. Report skipped or blocked checks and the reason.

For macOS app changes, build the real app and run its existing E2E flow in
isolated state. UI, permission, focus, and native input claims need separate
acceptance; see [readiness](docs/plan/readiness.md). Do not run a GUI harness in
the active input account. For docs-only changes, check local links and verify
changed command examples against the real CLI where safe.

## Submit a change

Describe the problem, the resulting behavior, and why the change fixes it.
List exact validation commands, source revision, environment, results, and any
limits. Separate historical evidence from checks run for the change. Do not
commit temporary TLS keys, unredacted logs, personal paths, IPs, or hostnames.

Preserve research evidence under `docs/re` and dated verification records.
Design targets in old plans are not current product guarantees. Update public
docs when behavior, prerequisites, support limits, or security boundaries
change. Follow the existing Rust and Swift style; avoid unrelated refactors.

Submit only work you have the right to license under the project's MIT OR
Apache-2.0 terms. Link external protocol/behavior research and identify any
third-party material and its license. Do not copy incompatible implementation
code into the project.

## Maintainer releases

Follow [docs/releasing.md](docs/releasing.md) for validation, artifacts, signing,
and release steps. A merged PR, passing CI, or local signature does not by itself
authorize publication or a repository visibility change. Record pending native
acceptance and support limits in release notes.
