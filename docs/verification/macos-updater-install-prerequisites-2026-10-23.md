# Private Sparkle installation prerequisites

Research `oe-sparkle-private-install-prerequisites-r1-043757a-20261023` inspected
Openergy `043757a` and pinned Sparkle 2.10.0 revision
`eef1a539a373c1f1a320624b1130fc5de7b2e100`. It ran no updater, installer, target
app, build or test and read no private signing key. The existing updater command
still permits only information checks and vetoes installation.

Sparkle supplies an official non-UI CLI using a real `SPUUpdater`. Its
`--check-immediately` flow can download and replace a stopped private target.
The SwiftPM binary artifact does not contain `sparkle.app`; a pinned CLI build
and its complete framework are prerequisites. A build-only preparation task is
now in progress. A successful build will not grant runtime acceptance.

References: [official CLI documentation](https://sparkle-project.org/documentation/sparkle-cli/),
[pinned CLI driver](https://github.com/sparkle-project/Sparkle/blob/eef1a539a373c1f1a320624b1130fc5de7b2e100/sparkle-cli/SPUCommandLineDriver.m),
[pinned CLI user driver](https://github.com/sparkle-project/Sparkle/blob/eef1a539a373c1f1a320624b1130fc5de7b2e100/sparkle-cli/SPUCommandLineUserDriver.m).

## Host effects and relaunch

The stock installer submits/removes transient launchd jobs named for the host
bundle, including `<fixture-id>-sparkle-updater` and `<fixture-id>-sparkle-progress`.
Its helpers also create installation caches through `NSCachesDirectory`.
`SUDefaultsDomain` isolates preferences, not these jobs and caches. Passing a
private home only to the parent command does not establish helper containment.
The parent independently checked the launcher source and its job dictionaries.

Relaunch uses `NSWorkspace.openURL` without the old command arguments or
environment. Shell-only private config cannot safely isolate a relaunched
Openergy target. Both fixture versions would need a fail-closed private
bootstrap before normal startup, or an isolated macOS execution environment.
No production bootstrap or installation mode was added here.

References: [installer launcher](https://github.com/sparkle-project/Sparkle/blob/eef1a539a373c1f1a320624b1130fc5de7b2e100/InstallerLauncher/SUInstallerLauncher.m),
[installation cache](https://github.com/sparkle-project/Sparkle/blob/eef1a539a373c1f1a320624b1130fc5de7b2e100/Autoupdate/AppInstaller.m),
[cache resolver](https://github.com/sparkle-project/Sparkle/blob/eef1a539a373c1f1a320624b1130fc5de7b2e100/Sparkle/SPULocalCacheDirectory.m),
[relaunch implementation](https://github.com/sparkle-project/Sparkle/blob/eef1a539a373c1f1a320624b1130fc5de7b2e100/Sparkle/InstallerProgress/InstallerProgressAppController.m).

## Local environment observation

Read-only local sandbox inventory returned an empty list, exit 0. Runtime
inspection also exited 0: Lume is not installed or serving, QEMU is absent, and
no container engine is reachable. Lume setup would download a binary and start
`lume serve`; no setup, VM creation, service or cloud resource was started.
Private command records remain in `/tmp/oe-local-sandboxes-20261023.json` and
`/tmp/oe-local-runtimes-20261023.json`.

The current private-files/no-service-change scope therefore does not yet permit
an actual installer run on this host. This is a specific installation prerequisite,
not a product failure or a reason to stop independent app hardening.

## Required runtime evidence

For stopped-target verification, use the same valid fixture archive for positive
and negative cases. The negative feed changes only its enclosure signature to
another well-formed 64-byte value, with valid feed-signature handling. Require
complete download plus an actual signature/validation error and an unchanged old
bundle. A generic failing exit is insufficient. Positive replacement must prove
candidate build, executable/tree hashes and nested signatures at the target path.
Keep the production archive command's identity and HTTPS requirements intact.

A stopped-target CLI run supplies no relaunch or app daemon-handoff proof.
Those require observed old-process exit, new app/daemon identities, candidate
paths and readiness through the actual product lifecycle. Visual update UI,
Developer ID, notarization, Gatekeeper and native input/clipboard gates remain
separate.
