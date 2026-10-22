# Forced-interruption verifier follow-up

The standalone `tools/e2e/verify_teardown.py` expected an `oe-lab-wp7-` Docker
container, while the actual runner and guardian use `oe-lab-wp6-`. Looking up a
nonexistent name could make container removal appear successful without checking
the real lab. This was a verifier defect, not evidence of a product cleanup leak.

The verifier now observes the running lab before interruption, validates its
ownership label and canonical source directory, retains the actual container ID,
and checks that ID disappears after SIGKILL. Docker query errors fail closed.
Its cleanup deadline includes the guardian's bounded stop/removal grace periods.

Real-process verification used the release runner/daemon at `29fa8ba`, only
private virtual daemon state and owned Docker/Xvfb peers:

- Adding the existence prerequisite while retaining the old prefix failed with
  `owned lab was not observed before interruption`.
- With the correct prefix and ownership/ID checks, forced runner SIGKILL passed:
  the actual daemon exited and the observed owned container was removed.
- Pointing the real Docker CLI at an absent private socket refused success.

```sh
nice -n 15 python3 tools/e2e/verify_teardown.py \
  --runner target/release/openergy-e2e \
  --port-range 26340-26359 --lab-port-range 26360-26379
```

No native desktop, installed input process, production config, or port 24800 was
used. No unit tests or mocks were added. Raw private logs remain local; this
record omits their machine-specific temporary paths and process IDs.
