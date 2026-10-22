# Version 1 option limits

`input.sync_language` is retained for config compatibility but is not yet
implemented. Clients ignore peer language strings. The server sends ordinary
DKDN key downs and retains the required Synergy 1.8 LSYN setup message.

Half-duplex lock options are parsed but do not change v1 key behavior. Modifier
remaps apply to modifier masks; remapping the key IDs themselves is not applied
in v1.

`openergy connect HOST` rewrites and saves `client.remote` and the client role by
design. `connection.connect` with a host follows the same rule.

Scroll sensitivity and vertical inversion belong to the receiving computer.
The client transforms wheel units before its platform sink applies its normal
notch-to-line conversion (3 lines per notch on macOS).
