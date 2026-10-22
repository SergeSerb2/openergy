# Dead-corner coordinate bounds

The accepted `corner_size_px = 2147483647` overflowed signed 32-bit additions
at a positive display origin, and subtractions at a negative display origin.
The release daemon could cross an edge covered by the configured dead corner.
The comparison now uses 64-bit arithmetic without changing the accepted range.

Two real daemon-pair scenarios cover origins `(100, 100)` and `(-3000, -3000)`.
Each checks that the large corner blocks crossing, then sets its size to zero
and verifies that ordinary crossing resumes. Both fail at the blocked-edge
assertion with the release daemon from `5b02f8c`. Both pass with the integrated
daemon based on `dce55c5` plus this arithmetic change. Cleanup failures: zero.

An initial control sent two absolute moves in one injection batch. Capture
coalesced them, leaving no retreat before the control's edge hit. The final
scenarios wait for the retreat position before sending the next hit. They also
do this before the first hit: changing a display origin can clamp the initial
cursor onto the edge, making an absolute move to that position have zero delta.
Without this prerequisite, one repeat passed on the old binary without testing
a physical edge hit. The final identical scenarios fail **0/2** on the old
binary and pass **2/2** on the fixed binary. Reports are
`/tmp/openergy-round5-corner-before-physical.json` and
`/tmp/openergy-round5-corner-after-physical.json`.

```sh
nice -n 15 target/release/openergy-e2e run \
  --scenario 'R5-dead-corner-*' --no-docker \
  --port-range 26340-26359 --lab-port-range 26360-26379 \
  --report /tmp/openergy-round5-corner-after.json
```

The locked release daemon/runner build, scoped all-targets session Clippy with
warnings denied, formatting and diff checks passed. Only private virtual
daemons and loopback TCP ran. This does not verify physical monitor geometry,
native input, or installed desktop behavior. No unit tests or mocks were added.
