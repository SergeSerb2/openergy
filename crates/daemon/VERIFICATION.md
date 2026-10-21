# WP6 verification

This file is an integration work log. Final scenario results will replace the
pending section after the remaining leaf implementations are merged.

## Gates verified so far

Commands run from the shared checkout with Rust 1.99.1:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo clippy --release -p openergy-session -- -D warnings
```

```text
Checking openergy-session v0.1.0
Finished `release` profile [optimized]
```

Before daemon wiring was added, `cargo build --release` completed with zero
warnings at session increment 9e70aa6. That is not the final workspace gate.

The native app's existing E2E executable was refreshed from unchanged app sources:

```sh
cd apps/macos && swift build
```

```text
Build complete! (16.95 secs.)
```

## Integration currently pending

`cargo check --release -p openergy-daemon` currently stops on missing
`openergy_net::{NetDriver,NetConfig,MdnsConfig}` and the contracted
`NetEvent::{ListenFailed,ConnectFailed}` variants. WP4 owns those additions.
The platform/config/session APIs resolve. Native macOS construction temporarily
returns typed `PlatformError::NotImplemented` until WP5 lands.

No S/I scenario pass is claimed before the real binaries run. Ready drivers:

- `python3 crates/daemon/scripts/verify_virtual.py`
- `python3 crates/daemon/scripts/verify_security_lifetime.py`
- `python3 crates/daemon/scripts/verify_latency.py`
- `python3 crates/daemon/scripts/verify_interop.py`

Each driver starts real `openergy` processes and uses ctl, framed IPC, or raw
loopback sockets. It keeps failure artifacts under `/tmp/oe-wp6-*` and stops only
its own processes. There are no unit tests or substituted network peers.
The interop driver copies the lab scripts into its temp directory to isolate
lab state, sets `LAB_NAME=oe-lab-wp6`, `LAB_PORT_RANGE=24950-24959`, and always
runs `lab.sh down`. Local daemons use only ports 24980–24989. The app E2E will
use `--join 127.0.0.1:24981 --host-port 24985` with isolated config/socket paths.
