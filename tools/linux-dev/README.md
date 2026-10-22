# Isolated Linux development

Requirements: Docker (OrbStack on this Mac). The image uses ARM64 on this host,
Debian Bookworm, Rust 1.99.1, Xvfb, xdotool, xinput, xclip, xev and Python 3.
X11 access uses x11rb and libc; no native X library is linked.

```sh
tools/linux-dev/linux-dev.sh build               # image, then both platform package release builds
tools/linux-dev/linux-dev.sh run inject
tools/linux-dev/linux-dev.sh run clipboard
tools/linux-dev/linux-dev.sh run capture
tools/linux-dev/linux-dev.sh run all
tools/linux-dev/linux-dev.sh run check           # platform package release build and Linux clippy
tools/linux-dev/linux-dev.sh shell              # interactive container shell
tools/linux-dev/linux-dev.sh shell cargo check -p openergy-platform-linux
tools/linux-dev/linux-dev.sh down
```

The build/check commands select `openergy-platform-linux` and
`openergy-platform-windows`, so an unrelated daemon integration failure does not
block native backend checks. Linux runs clippy for its native crate and executes
the real probe. Windows still requires the separate MSVC check/clippy commands.

The worktree mounts read-only at `/src`. Each container copies source to a tmpfs
at `/work`; Cargo can generate its lockfile there. All target output goes to the
`oe-linux-dev-wp8-target` named volume. Dependencies use `oe-linux-dev-wp8-cargo`.
Container names start with `oe-linux-dev-wp8-`, and `down` only removes that prefix.
The target and dependency caches remain for later builds.

Xvfb listens only on its Unix socket at DISPLAY=:99. No host display socket,
clipboard, device, input service or published TCP port is mounted. No Synergy
binary is started. Docker's network is used only for build dependencies. Runtime
probes use local X11 clients in the same container.

The example refuses to run without both DISPLAY=:99 and
OPENERGY_X11_PROBE_ISOLATED=1 (the script sets these inside the container).
The probes create synthetic input only in Xvfb. xev and xinput logs in `/tmp`
contain only that generated input; the backend does not log input contents.
Selection fixtures include a 10,498,508-byte PNG. Python independently checks
pixel preservation after actual backend/xclip selection transfers. The probe also
constructs the shared Backend, calls its default permission requests and proves
PRIMARY updates do not change the v1 CLIPBOARD data or change stamp. PRIMARY remains
compiled and locally exercised as a post-v1 extension; v1 does not route id 1.

The image is independent of `openergy-interop:latest` and other agents' containers.
