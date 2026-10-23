# Security policy

## Supported versions

Openergy is alpha software. No stable release or security backport window is
defined as of 2026-10-21. Maintainers review reports against the current source
on a best-effort basis. There is no response-time commitment or claim of an
independent security audit. Do not treat old verification reports as validation
of a different commit, OS build, peer version, or configuration.

## Report a vulnerability

The maintainer chose to skip GitHub private vulnerability reporting setup on
2026-10-22. It is not a prerequisite for the current hardening work. No private
reporting route has been enabled or verified for this repository. This decision
does not change repository visibility or establish another contact service.

Until a private route is available, [open a repository issue](https://github.com/SergeSerb2/openergy/issues/new)
asking the maintainer for a private reporting channel. Include only a general
description; do not put exploit steps, keys, personal data, or a working exploit
in that request. A repository issue is not confidential. No separate security
email or paid bounty program is currently specified.

A private report should include the commit/version, OS/backend, peer roles,
TLS and trust settings, impact, and a minimal reproduction in an isolated lab.
Describe any data exposure or unauthorized input. Use synthetic clipboard and
input data. Allow maintainers to agree on a disclosure plan before publishing
sensitive details.

## Trust and permission model

Openergy gives other computers access to input and clipboard data. Trust only
computers and users that should have that access.

- A host captures physical input and can send keys, buttons, and pointer events
  to clients. A joined computer accepts that remote input in its user session.
  Input can invoke actions in applications with that user's permissions.
- TLS is on by default, with SHA-256 certificate fingerprint pinning. This is
  not WebPKI certificate-chain/hostname authentication. Compare first-use pins
  through a channel you already trust. Discovery names and fingerprint hints
  are untrusted network announcements.
- Clients pin the server. Servers require client pins by default through
  `tls.require_client_trust = true`. Setting it to `false` removes the stored
  client-pin requirement. Setting `tls.enabled = false` removes encryption and
  certificate authentication. Imported configurations can change these values;
  inspect the result before connecting.
- A new or changed identity can need approval. Do not approve an unexpected
  fingerprint to clear an error. Verify a deliberate key rotation on both
  computers. Revoke obsolete pins and reconnect to check the resulting state.
- Use the app or `ctl trust.revoke` for an active peer. A successful live revoke
  removes matching connections and their temporary explicit pin before replying.
  Offline trust commands edit stored lists; they do not clear an in-memory
  explicit pin on an existing connection.
- If another local process holds the trust-file lock, live approval/revocation
  returns `trust_failed` with a retry message. Existing pins and connections
  stay unchanged. Retry after the other update finishes. Offline CLI updates
  still wait for that lock so concurrent writers preserve each other's changes.
- If another local process holds the identity-file lock, daemon startup refuses
  with a busy error. A live TLS change returns `config_apply_failed` or
  `connect_failed` and preserves the configuration and existing connections.
  Retry after the creator finishes. Identity reads still share the creator lock;
  an incomplete key/certificate pair is never accepted or silently replaced.
- Offline `trust approve` stores a pin in **both** server and client lists.
  Live IPC approval knows the pending connection's role. Use the app or
  `ctl trust.approve` when you need approval for that role only.
- The local control API can change configuration, trust, and connection state.
  Unix sockets use owner access restrictions. Windows pipes use the current
  user's SID and reject remote pipe clients. These controls do not isolate
  Openergy from another process running as that same user or an administrator.
- macOS Accessibility permits event injection; Input Monitoring permits input
  capture. Host needs both and Join needs Accessibility. Secure Input and OS
  restrictions can prevent capture. Openergy is not a way to bypass them.
- X11 desktop access is a broad trust boundary. Windows secure-desktop and
  elevated/UIPI input are not established supported paths. Do not run the
  daemon as root or elevate it merely to work around an input failure.

TLS protects the configured peer connection. It does not protect against a
compromised peer, a compromised local account, malicious clipboard content,
OS input restrictions, or every denial-of-service attempt. Use a trusted local
network or your own secure network transport. Do not expose the listener to
the public internet. Restrict firewall access to intended peers.

## Data and privacy

The app and daemon have no account requirement, telemetry upload, cloud relay,
or automatic update service. Native backends use mDNS to browse local peers and
advertise a host. This exposes a service name and connection metadata on that
network. `OPENERGY_DISABLE_DISCOVERY=1` disables that daemon discovery path;
manual connections still work. Virtual-platform runs disable discovery.

Default state roots are `~/Library/Application Support/Openergy` on macOS and
`%APPDATA%\Openergy` on Windows. For Linux path details, see the
[setup guide](docs/plan/setup.md). With explicit `--config`, daemon TLS and logs
are under that config's parent directory. `--socket` is a separate selector.

State includes configuration, `tls/cert.pem`, `tls/key.pem`, trusted server/client
fingerprints, and logs. The private key is stored unencrypted. Unix identity
files are restricted to mode `0600`. Windows identity and trust storage applies
protected access lists for the current user. The control pipe also checks the
server's user identity. Use a private user directory and protected backups.
Do not share private keys or copy one computer's identity to another.

Clipboard sharing is enabled by default, in both directions. Its default local
limit is 64 MiB; peer limits and supported formats can reduce it. Text, HTML,
and bitmap data can contain secrets. Disable `[clipboard] enabled`, `send`, or
`receive` as appropriate. Disabling future sharing does not erase data already
delivered to a peer or held by clipboard history software.

Normal logs describe state and errors rather than input/clipboard payloads.
They can contain names, IP addresses, paths, certificate fingerprints, transfer
sizes, and application names. Daemon logs rotate locally; the macOS app also
keeps daemon stderr and OS diagnostics. Virtual E2E artifacts can contain the
synthetic events and clipboard bytes used by the scenario. Review and redact
all diagnostics before sharing them. Do not use personal data in a lab.

## Safe verification and distribution

Use the isolated virtual backend and the [external E2E runner](tools/e2e/README.md)
to report parser, transport, trust, and lifecycle problems. Never test an exploit
against someone else's active input connection. Native behavior needs a separate
test account or machine with recovery input available.

Build signatures and notarization are separate from this protocol trust model.
An ad-hoc signed local macOS bundle is not a notarized distributable. Check the
[maintainer release procedure](docs/releasing.md) and the artifact's actual
signature and checksum. No published release artifacts exist as of 2026-10-21.
