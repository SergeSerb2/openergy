# Openergy Windows interop lab

Checks Openergy against the real Synergy core that is installed on a Windows PC,
without disturbing the Synergy that is in use on that PC.

`lab.ps1` runs a **private copy** of `synergy-core.exe` from an SSH session.
SSH sessions run in Windows session 0, which has no desktop: the copy cannot see
or move the real mouse and keyboard. It listens and connects only on `127.0.0.1`;
SSH tunnels carry the traffic to the Mac. The installed Synergy, its settings and
its service are never changed, stopped or restarted.

## What it can and cannot check

| Direction | Checked | Not possible in session 0 |
| --- | --- | --- |
| Synergy server → Openergy client | TLS, pinning both ways, hello, screen info, heartbeats | Screen switches and input: Windows refuses synthetic input in session 0, so the server cannot move its pointer to an edge |
| Openergy server → Synergy client | TLS, pinning both ways, hello, entering the screen, input delivery without protocol errors, clipboard both ways through the real Windows clipboard | Injection on Windows: the client receives the input, but Windows refuses to inject it in session 0 |

## Run

On the Mac, make a certificate in the same form Synergy makes (RSA, SHA-1, X.509 v3,
no extensions) and copy the lab to the PC:

```sh
printf '[req]\ndistinguished_name=dn\nprompt=no\n[dn]\nCN=Synergy\n' > min.cnf
openssl req -x509 -config min.cnf -newkey rsa:2048 -nodes -sha1 -days 7 \
  -keyout key.pem -out cert.pem -addext subjectKeyIdentifier=none -addext authorityKeyIdentifier=none
cat key.pem cert.pem > winlab.pem
ssh windows 'mkdir $env:USERPROFILE\openergy-winlab -Force'
scp tools/windows-lab/lab.ps1 winlab.pem windows:openergy-winlab/
ssh windows 'pwsh -File $env:USERPROFILE\openergy-winlab\lab.ps1 setup'
```

**Synergy server → Openergy client.** Keep this SSH session open; it carries the tunnel:

```sh
ssh -L 127.0.0.1:24810:127.0.0.1:24810 windows \
  'pwsh -File $env:USERPROFILE\openergy-winlab\lab.ps1 server -Cert $env:USERPROFILE\openergy-winlab\winlab.pem'
```

Point an Openergy client with `--platform virtual` at `127.0.0.1:24810`, and put the
lab certificate's `v2:sha256:` line in its `tls/trusted-servers`. Add `-CheckClients`
to make the server check the client too, after
`lab.ps1 trust-client -Line "$(openergy fingerprint --synergy)"`.

**Openergy server → Synergy client.** Start an Openergy server with `--platform virtual`
on `127.0.0.1:24812`, with a screen named `winlab-client`, then:

```sh
ssh windows "pwsh -File \$env:USERPROFILE\openergy-winlab\lab.ps1 trust-server -Line '$(openergy fingerprint --synergy)'"
ssh -R 127.0.0.1:24812:127.0.0.1:24812 windows \
  'pwsh -File $env:USERPROFILE\openergy-winlab\lab.ps1 client -Cert $env:USERPROFILE\openergy-winlab\winlab.pem'
```

Drive the Openergy server with `openergy ctl virtual.inject` and `virtual.clipboard`.
Read and write the Windows clipboard from a second SSH session with
`lab.ps1 clip-get` and `lab.ps1 clip-set -Text '…'`. Logs are in
`%USERPROFILE%\openergy-winlab\{server,client}.log`.

**Clean up.** This stops the lab cores and removes only what the lab created: the
lab folder, its trust files, and the `%APPDATA%\synergy\Synergy.conf` that the core
writes on first start if it was not there before.

```sh
ssh windows 'pwsh -File $env:USERPROFILE\openergy-winlab\lab.ps1 cleanup'
```

## Notes

- The running Synergy core owns the global event `Global\SynergyClose` and denies
  other processes full access to it, so a second copy aborts at start with
  "failed to create event for windows event loop". `setup` renames that event in the
  private copy only.
- When the core runs elevated, it reads `C:\ProgramData\Synergy\tls\trusted-servers`
  (as a client) and `trusted-clients` (as a server with `checkPeerFingerprints`). The
  `trust-*` commands refuse to change files that the lab did not create.
- Only valid log level names work in the core's settings: `Debug`, `Info`, `Warning`.
