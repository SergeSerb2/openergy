# Version 1 option limits

`input.sync_language` is implemented and still defaults to `true`. **Upgrade
behavior change:** an Openergy server now sends its available languages and
current language to compatible peers. Existing Deskflow/Synergy clients may
start switching their keyboard layout. Set `input.sync_language = false` on
the sender or receiver to opt out locally.

Sync selects the first installed keyboard layout matching a two-letter
lowercase language code. It cannot sync variants, dead-key tables, or IME
state (`en` cannot distinguish US from GB). X11 selects an already configured
XKB group; macOS selects an enabled TIS keyboard-layout source (not an input
method); Windows activates our thread's HKL and posts a best-effort request
to the foreground app, which may ignore it or override it on focus changes.
On Windows, `language.sync` result `applied` means **request posted**, not
confirmed OS/app selection; `attempted: true` makes this explicit.
Unsupported platforms, invalid or empty codes and missing installed layouts
keep typing unchanged. Same-language repeats do not request another switch.

The server uses only existing LSYN (setup before DSOP), DKDL (wire >= 1.8),
and DKRP suffix fields. Older peers get DKDN and can still receive the repeat
suffix. Hello and DSOP are unchanged. The client applies a new valid language
before the triggering key. Peer status exposes setup `languages` and the last
valid requested `active_language`; the latter does not confirm platform success.
`language.sync` reports `applied`, `no_match`, or `unsupported` on changes.

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

`clipboard.share_primary_selection` (default `false`) opts in to X11 PRIMARY
selection sharing on clipboard id 1. It is X11-to-X11 only: other platforms
never originate id 1, receivers drop it rather than touching their single
system clipboard, and a server relays it only to peers known to be Linux
(configured screen `os = "linux"` or a peer that itself sent id 1). An empty
PRIMARY is not a clear event. With the flag off, behavior is exactly the
single-clipboard behavior above.
