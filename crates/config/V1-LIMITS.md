# Version 1 option limits

`input.sync_language` is retained for config compatibility but is not yet
implemented. Clients ignore peer language strings. The server sends ordinary
DKDN key downs and retains the required Synergy 1.8 LSYN setup message.

Half-duplex lock options (`half_duplex_{caps,num,scroll}_lock`) are
implemented in the session layer: for a flagged lock the server forwards only
the key-down edge and a flagged client injects it as an atomic down/up pair and
drops key-up/repeat, while sinks stay level-based and an unset flag keeps the
exact full-duplex behavior. Modifier remaps apply to modifier masks and to the
modifier keys themselves, as in the reference client; a modifier mapped to
`none` is dropped.

`openergy connect HOST` rewrites and saves `client.remote` and the client role by
design. `connection.connect` with a host follows the same rule.

Scroll sensitivity and vertical inversion belong to the receiving computer.
The client transforms wheel units before its platform sink applies its normal
notch-to-line conversion (3 lines per notch on macOS).
