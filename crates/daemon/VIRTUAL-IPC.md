# Virtual IPC additions

The daemon follows `crates/platform/VIRTUAL.md`. `virtual.recorded` returns the
`{events,stats}` with sink events and release markers, including monotonic
`timestamp_us`. Capture, cursor, and clipboard audit markers stay in the platform
ledger and do not affect input-order assertions.

For large clipboard fixtures, `virtual.clipboard` also accepts
`{"set":{"formats":[{"kind":"dib","bytes_base64":"..."}]}}`.
The original numeric `bytes` array remains supported. Get responses encode clipboard contents as `bytes_base64`. Set returns a size/stamp
summary unless `get:true` is also supplied. Exactly one input byte encoding is required.
The aggregate decoded cap is 64 MiB; invalid base64 is a typed IPC error.

Small convenience inputs for the real-process driver are `{"text":"..."}` and
`{"dib_bytes":8388608}` (a zero-filled virtual DIB fixture). `{"summary":true}`
returns sizes, kinds, and write stamps without copying bytes into IPC JSON.
These operations only apply with `--platform virtual`.

The v1 daemon routes the normal clipboard (wire ID 0). It drains selection
clipboard frames (ID 1) without changing the platform's normal clipboard.

On macOS the daemon maps allocations of at least 1 MiB directly, then unmaps
them when dropped. Smaller allocations use the system allocator. This prevents
Darwin's malloc cache from retaining freed clipboard and IPC copies. The server
also drops its marshalled cache after all senders drain; later entry reads the
current platform clipboard again. No allocator work runs for input events.

`virtual.recorded {"audit":true}` includes capture and other audit markers.
Each record also includes `timestamp_unix_us` so an external driver can compare
capture and sink timestamps across two local daemons without timing its IPC
observer. `timestamp_us` remains monotonic relative to backend creation. The
latency driver checks a 1,000-move same-host sample and reports both measurements;
wall-clock comparison assumes the host clock does not step during that sample.

`connection.connect` accepts an optional SHA-256 `fingerprint` with `host` and
`port`. It requires TLS and pins that address through retries for the daemon's
lifetime. A mismatch yields `status.last_error.code = tls_pin_mismatch` and no
application data. Normal TOFU approval continues to use the role's trust file.
