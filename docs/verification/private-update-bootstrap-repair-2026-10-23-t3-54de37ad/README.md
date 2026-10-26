# Private update test fixture: launcher design

Date: 2026-10-23. This note describes the private test fixture that
`apps/macos/Sources/Openergy/App/PrivateUpdateFixture.swift` refers to. The
fixture lets maintainers run an updater test against a disposable copy of the
app. It never checks for, downloads, or installs updates by itself, and normal
app builds never enter it.

The full original record, with build logs and file hashes, is kept at the Git
tag `evidence-archive-2026-10-25`.

## How the fixture starts

A fixture bundle has three executables:

```
Contents/MacOS/OpenergyPrivateLauncher   CFBundleExecutable, links only libSystem
Contents/MacOS/Openergy                  the normal Swift app image
Contents/Helpers/openergy                the normal daemon
```

1. `OpenergyPrivateLauncher` (source: `apps/macos/PrivateUpdateBootstrap/launcher.c`)
   reads `Contents/Info.plist` itself, through a no-follow descriptor with size,
   owner, file-type, link-count, path, and mode checks. The bytes must match the
   template in `Info.plist.in` exactly. Only six fields can vary: `FIXTURE_ID`,
   `BUILD`, `ROOT`, `NONCE`, `PORT`, and `PUBLIC_ED_KEY`.
2. The launcher checks the private root, the app and daemon images, and the
   private paths. It sets nice 15, umask `077`, and the working directory.
3. It then calls `execve` on the Swift image with a fresh environment. Only
   these values pass through, plus `PATH=/usr/bin:/bin` and `LANG=C`:

   ```
   HOME, CFFIXED_USER_HOME, TMPDIR, XDG_CONFIG_HOME, XDG_CACHE_HOME
   OPENERGY_CONFIG, OPENERGY_CONFIG_DIR, OPENERGY_TLS_DIR, OPENERGY_SOCKET
   OPENERGY_DAEMON, OPENERGY_DAEMON_ARGS=--platform virtual
   OPENERGY_DISABLE_DISCOVERY=1, OPENERGY_ALLOW_PORT_24800=0
   ```

   All of them point inside the fixture's private root. The environment is in
   place before the Swift image and its frameworks load.
4. The Swift image accepts only `--private-update-fixture --e2e` with valid
   fixture metadata. It repeats the metadata, path, config, and marker checks,
   then starts the normal `AppModel` with a supervised private virtual daemon.

## Fail-closed rules

- If `Bundle.main.infoDictionary` is nil, the app exits with code 78. It does
  not fall back to normal startup.
- A bundle whose identifier has the fixture prefix but no fixture metadata
  refuses to start. Invalid metadata refuses to start.
- No environment variable can turn on fixture startup.
- A direct start of the Swift image without arguments refuses. The only
  supported entry is the launcher.

## Packaging

`apps/macos/PrivateUpdateBootstrap/package-entry.py` is opt-in. Normal
`bundle.sh` builds do not use it. It takes `--app` and `--launcher`, requires an
unsigned bundle under a mode-0700 `/private/tmp/oe-updater-fixture-<uuid>` root,
checks that the launcher is a thin PIE Mach-O for macOS 15.0 that links only
libSystem, and then writes the canonical fixture plist. It does not sign, launch,
register, or install anything.

## What was checked

On 2026-10-23 the launcher and the full Swift app compiled and linked for arm64,
macOS 15.0, with network access denied to the compilers. The launcher was
35,416 bytes and had one dylib load (`/usr/lib/libSystem.B.dylib`) and no rpath
or dyld-environment load command.

Not checked yet: a real fixture launch, a refusal at runtime, isolation of a
running fixture, and relaunch after an update. The launcher is not an OS sandbox
and does not claim to resist a hostile process running as the same user.
