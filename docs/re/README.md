# Research provenance and limits

These reports document interoperability research performed on 2026-10-21.
They describe specific Synergy 3.7.0-beta macOS artifacts and the
1.21.1-beta core. Product versions and wire versions are separate.
Each report records its artifact hashes, method, evidence anchors, and limits.

The research includes reading bundled JavaScript, embedded source maps,
binary symbols, disassembly, and selected decompiler output. It also includes
passive inspection of existing process metadata and redacted configuration
shapes. The core report records version/help execution from a disposable copy.
These methods do not establish a separated clean-room development process.
In particular, source-map inspection is source-informed research. Earlier
clean-room labels in the UI, service, and auxiliary reports overstated what
the recorded method proves.

The tracked reports retain original explanations, interface identifiers,
protocol constants, independently organized schema descriptions, artifact
hashes, and short diagnostic strings. The public-release audit at
`06b61287afd7f31aa3002d252389f2c2b17f73f5` found no tracked Synergy/Deskflow
executable, application archive, extracted source tree, or decompiled function
body. This is a bounded repository inspection, not proof of every contributor's
development history or an automatic license grant for upstream material.

The inspected applications and third-party reference peers keep their own
licenses. Openergy's MIT/Apache-2.0 license does not relicense them. Do not
commit copied target executables, bundles, source maps, fonts, private
configuration, license keys, certificates, or private keys as evidence.
Keep inert analysis copies outside the repository. Preserve interface facts
and useful sanitized evidence with their source and confidence limits.

The schema blocks in `01-electron-ui.md` are documentation of interface
fields. They are not TypeScript package source and are not compiled into the
app. The research reports do not establish end-to-end compatibility with
every version, platform, or optional feature of a reference product.

Synergy, Deskflow, Barrier, and Input Leap identify separate projects. Their
names and protocol identifiers describe compatibility targets. They do not
imply affiliation or endorsement. The current Synergy
[vendor EULA](https://symless.com/synergy/legal/eula) identifies Synergy App Ltd
(formerly Symless); artifact-era company strings in the reports remain
historical observations. No trademark rights are granted by Openergy's license.

See [third-party notices](../../THIRD_PARTY_NOTICES.md) for dependency licenses
and X11 data attribution. See the
[release audit](../verification/public-release-audit-2026-10-21/README.md)
for the actual scan results and remaining release gates.
