# M1 readiness after the recorder review

One fresh read-only query reached the M1 with strict host-key checking. The
task-owned known-host key still matches the user's verified ED25519 fingerprint.
The host reports arm64 and macOS27.0.1/26A434. The exact retained app, daemon,
private Python executable and frozen-source Cargo.lock hashes all match.
The query returned0, ran at nice15 and invoked no product.

[result.json](result.json) retains the scoped facts and explicit missing gates.
[The private index](private-evidence-index.json) binds the source, invocation,
raw output, first-use guard and actual query receipt without publishing the
connection route. No previous query or app-flow guard was reused.

The corrected ordinary controller `2b5fc1f` has a source-clear delta review
`b8a4221`; its complete inherited review and limits remain applicable. The next
operation still needs fresh complete source/bundle/Python-prefix verification,
the one new preparation, and private-copy qualification before one bounded
diagnostic app flow. These four matching files do not supply those full gates.
No app, daemon service, updater, build, signing, native input or new product
flow ran here. All previous failures and the unknown bind cause remain unchanged.
