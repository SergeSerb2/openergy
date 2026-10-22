# Isolated Linux development

Requirements: a running Docker engine and a POSIX host with Bash. No host X11,
Rust installation, native clipboard access or input permission is needed. The
image uses `rust:1.99-bookworm` (Rust 1.99.1 in the recorded run), its C compiler
and linker, Clippy, rustfmt, Xvfb, xdotool, xinput, xclip, xev and Python 3.
Docker selects the host architecture; the recorded native run used ARM64.

To build directly on Linux, install a workspace-compatible Rust toolchain and a
C compiler/linker. This crate uses x11rb's Rust protocol implementation and does
not require Xlib/X11 development headers or libraries. A running backend needs
an authenticated X11 display (`DISPLAY`, and `XAUTHORITY` where applicable),
XInput 2, XTEST, XFixes 4 and XKB. RandR monitor discovery and the screensaver
extension provide optional display notifications and sleep inhibition. Wayland
is not supported. The real probes also need the X11 utilities and Python listed
above. The workspace's minimum Rust version is a separate workspace gate; this
report establishes builds with 1.99.1 only.

```sh
tools/linux-dev/linux-dev.sh build               # build image and Linux platform crate
tools/linux-dev/linux-dev.sh run inject
tools/linux-dev/linux-dev.sh run clipboard
tools/linux-dev/linux-dev.sh run clipboard-contract # unavailable versus clear acceptance
tools/linux-dev/linux-dev.sh run capture
tools/linux-dev/linux-dev.sh run recovery        # grab, ownership, keymap and INCR recovery
tools/linux-dev/linux-dev.sh run resources       # repeated create/drop, keymap and worker cleanup
tools/linux-dev/linux-dev.sh run all
tools/linux-dev/linux-dev.sh run check           # locked release build and native Clippy
tools/linux-dev/linux-dev.sh shell              # interactive container shell
tools/linux-dev/linux-dev.sh shell cargo check --locked -p openergy-platform-linux
tools/linux-dev/linux-dev.sh down
```

`build` and `check` select only `openergy-platform-linux`. Linux Clippy checks all
of that crate's targets. Windows native validation belongs to its MSVC workflow.
The entrypoint enforces `CARGO_BUILD_JOBS=2` and runs commands with `nice -n 15`.
`run` bounds the probe command to 120 seconds, then allows five seconds for exit
before killing it. A failed Xvfb startup stops the command before Cargo starts.

The worktree mounts read-only at `/src`. Each container copies source to a tmpfs
at `/work`; the build and probe commands use `--locked`. Target output goes to
`oe-linux-dev-wp8-target`; dependencies use `oe-linux-dev-wp8-cargo`. Cargo locks
these shared caches when commands overlap. Container names include a worktree
identifier and each container carries its worktree label. `down` selects that
label, so it leaves other worktrees' containers alone. The caches remain for
later builds. Commands use `--rm`; Docker removes their Xvfb and client processes
when the command exits.

Xvfb listens only on its Unix socket at `DISPLAY=:99`, with `-noreset` to avoid
resetting between probe clients. No host display socket, clipboard, device,
input service or published TCP port is mounted. No Synergy binary is started.
Docker's bridge network permits build dependency downloads. The probes use only
local X11 clients inside their container and allocate no TCP ports.

The example refuses to run without both `DISPLAY=:99` and
`OPENERGY_X11_PROBE_ISOLATED=1` (the script sets these inside the container).
The probes create synthetic input only in Xvfb. xev and xinput logs in `/tmp`
contain only that input; the backend does not log input contents. Independent
Python checks verify pixel preservation after actual backend/xclip transfers,
including a 10,498,508-byte PNG. Recovery checks use the public backend API,
competing X clients, actual server grabs/key state/key mappings and selection
requests. They cover failed capture, local input ownership, newer external
keymaps, held lock keys, own-write stamps, destroyed INCR requestors, repeated
backend drop, and an undersized INCR read
followed by a successful read from the same xclip owner.

The shared Backend routes CLIPBOARD only. PRIMARY is compiled and locally
exercised as a post-v1 extension; its updates must not change the shared
CLIPBOARD data or stamp. The image is independent of `openergy-interop:latest`.
See [the recorded verification](../../crates/platform-linux/AUDIT-2026-10-21.md)
for commands, failure evidence and remaining limits.
