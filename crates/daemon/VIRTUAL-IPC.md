# Virtual IPC additions

The daemon follows `crates/platform/VIRTUAL.md`. `virtual.recorded` returns the
full `{events,stats}` audit, including monotonic `timestamp_us` and release markers.

For large clipboard fixtures, `virtual.clipboard` also accepts
`{"set":{"formats":[{"kind":"dib","bytes_base64":"..."}]}}`.
The original numeric `bytes` array remains supported. Get/set responses encode
clipboard contents above 1 MiB as `bytes_base64`; smaller contents keep `bytes`.
The aggregate decoded cap is 64 MiB; invalid base64 is a typed IPC error.

Small convenience inputs for the real-process driver are `{"text":"..."}` and
`{"dib_bytes":8388608}` (a zero-filled virtual DIB fixture). `{"summary":true}`
returns sizes, kinds, and write stamps without copying bytes into IPC JSON.
These operations only apply with `--platform virtual`.
