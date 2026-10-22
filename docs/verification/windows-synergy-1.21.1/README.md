# Openergy with the real Synergy core on Windows

Date: 2026-10-21. Windows PC: Windows 11 Pro Insider Preview (build 26340), Intel
i7-12700K, Synergy core **1.21.1-beta** (installed with Synergy 3). Mac: Openergy at
this commit, `--platform virtual`, macOS 27.2.

The Synergy in daily use on that PC kept running throughout. The tests used a private
copy of its core in an SSH session (Windows session 0, no desktop), on `127.0.0.1`
only, through SSH tunnels. Method and scripts: [tools/windows-lab](../../../tools/windows-lab/README.md).
After cleanup, the installed Synergy processes had the same process IDs as before the
tests, and its connection to the Mac was still up.

## Results

| Check | Result |
| --- | --- |
| Synergy server → Openergy client: TLS 1.3, Openergy pins the server certificate (RSA, SHA-1, v3, no extensions: the form Synergy makes) | Pass |
| Server registers the client: `client "oe-client" has connected`, screen 1920×1080 | Pass |
| Stays connected with Synergy's 5 s heartbeat | Pass, 3 min |
| Server checks the client certificate (`checkPeerFingerprints=true`): reads `C:/ProgramData/Synergy/tls/trusted-clients`, accepts Openergy's ECDSA certificate | Pass |
| Openergy server → Synergy client: TLS 1.3, Synergy pins Openergy's certificate from `C:/ProgramData/Synergy/tls/trusted-servers` | Pass |
| Synergy client says hello back as Synergy 1.8 and enters the screen when Openergy switches to it | Pass |
| Move, click, key and wheel events reach the client without a protocol error or disconnect | Pass |
| Clipboard Openergy → Windows, read from the real Windows clipboard | Pass, including `Grüße — 日本語 ✓` |
| Clipboard Windows → Openergy, sent when the pointer leaves the Windows screen | Pass, including `Ñandú 🎉 测试` |
| Server certificate in X.509 v1 form (what LibreSSL's `openssl req -x509` makes) | **Failed before this change** with `UnsupportedCertVersion`; passes after it |

Not possible in session 0: Windows refuses synthetic input there. So the Synergy
server cannot move its pointer to an edge to switch to Openergy, and the Synergy
client cannot inject the input it receives. The input path from a Synergy server to an
Openergy client is covered by the Deskflow 1.27 and Barrier 2.4 scenarios
(`I3-*`, `L*`), which use the same 1.8 protocol. A live check with the Synergy server
on this PC needs the person at the desk.

## TLS handshake matrix (after the change)

Real handshakes from an Openergy client to `openssl s_server` on loopback:

| Certificate | TLS 1.3 | TLS 1.2 |
| --- | --- | --- |
| RSA, X.509 v1 (LibreSSL) | Pass | Pass |
| ECDSA P-256, X.509 v1 (LibreSSL) | Pass | Pass |
| RSA, SHA-1, X.509 v3 (Synergy's form) | Pass | Pass |
| ECDSA P-256, X.509 v3 | — | Pass |
| Wrong pin | Rejected | Rejected |
| One bit flipped in the ServerKeyExchange signature (proxy) | — | Rejected, `BadSignature` |
