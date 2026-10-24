# macOS listener-shutdown refusal qualification

This bounded real OS command checked one possible prerequisite for a safely
owned refusal fixture. It did not run Openergy or change product source.
IPv4 and IPv6 loopback owners each bound an ephemeral port and listened. Each
fresh case tried one shutdown direction: read, write or both.

All six shutdown calls returned Darwin `ENOTCONN` (57). Competing ordinary binds
failed with `EADDRINUSE` (48), but all six real TCP connection attempts succeeded.
The sockets remained listeners. These Connected results are correct OS behavior,
not a false-Connected product defect. No actual refusal was observed, so this
method does not qualify a refusal fixture and must not replace a strict control.

The command closed every owned socket in `finally`. All six owned server ports
allowed a fresh exclusive bind afterward. No service, network setting, DNS,
firewall, native backend or installed application changed. No unit test, mock,
synthetic event or callback was used. The retained
[result](macos-owned-refusal-qualification/result.json),
[controller](macos-owned-refusal-qualification/command.py),
[postflight](macos-owned-refusal-qualification/postflight.json) and
[hash ledger](macos-owned-refusal-qualification/ledger.json) record the finite
qualification check. It adds no product or deadline acceptance.
