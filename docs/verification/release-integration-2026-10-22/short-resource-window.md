# Allocation and idle CPU checks

The newly catalogued `R5-incoming-clipboard-cancellation` scenario checks that
a declared 64 MiB clipboard transfer does not allocate its payload eagerly.
Its one-second resource window inherited the runner's default 0.1% CPU ceiling,
although the scenario's requirement specifies allocation and cancellation.

The first pinned integration run passed 87 of 88 scenarios. This scenario
failed only the inherited CPU threshold: RSS was **9,289,728 bytes**, below its
32 MiB limit, while one centisecond of CPU time over 1,064 ms was **0.939%**.
The short window is too coarse for the steady idle threshold.

The allocation scenario now states its CPU ceiling explicitly as 100% for this
short startup window. Its 32 MiB RSS limit and all transfer/cancellation checks
remain unchanged. The real affected-scenario replay passed. This is a test
scope correction; it does not establish CPU efficiency for that workload.

`S15-resources-latency` retains the separate **0.1% CPU ceiling over 60 seconds
per daemon**. It passed in that same pinned integration run at approximately
0.0666% and 0.0499%, with RSS below 10 MB. Its 1,000-event external observation
had p99 1.0403 ms. These remain virtual, local observations, not hardware or
comparative product measurements.

The initial failed report and corrected replay remain local at
`/tmp/openergy-round5-pinned-virtual.json` and
`/tmp/openergy-round5-cancellation-corrected.json`. The final integration record
identifies any later complete run separately. No product code changed for this
correction, and no unit tests or mocks were added.
