M1 adds an isolated read-only Wayland discovery example and an owned Weston
lane. X11 remains the only production Linux backend. There are no daemon,
platform-selection, shared capture-contract or production dependency changes.
No commit or push was made.

Source base: `d1e27a46f29228e6b66659b51da4f86ac15ba8cf`, with uncommitted changes
in the existing isolated `t3/wayland-discovery-probe` worktree. The design's
Decision, contract gaps, degradation table, milestones and verification rules
were read before implementation.

The probe has only `discover` mode. It reads the actual registry, core output
metadata, xdg-output logical geometry, seat capabilities and keyboard keymap
format/size/repeat metadata. It never maps a surface, reads keymap contents or
clipboard data, injects input, creates/starts portal sessions or opens EIS.
It reports all globals and explicit absence of plan-relevant interfaces.
Portal reads use non-activating D-Bus NameHasOwner, Introspect and property
reads. Authorization and EI device/region/version state stay unverified.
Every role remains `ready=false`.

Files changed:

- `crates/platform-linux/examples/wayland_probe.rs`: new discovery example,
  private-session guards, bounded Wayland discovery, portal introspection,
  capability facts and independent client/server/direct-path gaps.
- `crates/platform-linux/Cargo.toml` and `Cargo.lock`: Linux dev-dependencies only.
- `crates/platform-linux/README.md`: usage, prerequisites, guard contract and JSON meanings.
- `tools/linux-dev/wayland-discover.py`: real isolated Weston/bus supervisor,
  refusal/loss/timeout flows, optional real portal frontend and retained evidence.
- `tools/linux-dev/README.md`: link to the separate Wayland lane.
- `tools/release/notices/generate-notices.py`: recognize `LICENSE-MIT.md` and
  preserve core/xdg-output XML copyright notices from generated bindings.
- `THIRD_PARTY_NOTICES.md` and `tools/release/notices/dependency-licenses.json`:
  regenerated locked inventory and notices.
- This verification directory: exact JSON/stderr, redacted process logs,
  environment metadata and this report.

Dependency justification:

- `wayland-client =0.31.15` provides the real Wayland connection, registry and
  core object dispatch. `wayland-protocols =0.32.13`, with only client/unstable
  features, provides xdg-output bindings. Both are the families selected by
  the design. No native libwayland client development package is needed.
- `serde_json` reuses the existing workspace dependency for structured output.
- `roxmltree =0.21.1` is a small XML reader for real D-Bus introspection. It
  handles the standard introspection DOCTYPE without fetching external DTDs.
- `busctl` and GNU `timeout` are documented lab/runtime tools. This avoids
  introducing ashpd, zbus, an async runtime or EI dependencies for M1. The
  command disables service activation and interactive authorization. Each bus
  call has a two-second reply bound and a three-second process bound.
- Cargo added ten locked transitive/direct package entries. All new direct
  dependencies are under Linux `dev-dependencies`. The production daemon's
  `normal,build` dependency tree contains no Wayland or roxmltree package.

Commands used (from the repository root):

```sh
export PATH="$HOME/.cargo/bin:$PATH"
export CARGO_TARGET_DIR=/tmp/oe-target-impl-wayland
cargo build --release --locked -p openergy-platform-linux --example wayland_probe
cargo clippy --release --locked -p openergy-platform-linux --example wayland_probe -- -D warnings
CARGO_BUILD_JOBS=2 nice -n 15 cargo clippy --release --locked --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
python3 tools/release/notices/generate-notices.py
python3 tools/release/notices/generate-notices.py --check
cargo tree --locked -p openergy-daemon --edges normal,build --prefix none
env -u WAYLAND_DISPLAY "$CARGO_TARGET_DIR/release/examples/wayland_probe" discover
python3 tools/linux-dev/wayland-discover.py \
  --probe "$CARGO_TARGET_DIR/release/examples/wayland_probe" \
  --portal /usr/libexec/xdg-desktop-portal
CARGO_BUILD_JOBS=2 nice -n 15 cargo build --release --locked --bin openergy --bin openergy-e2e
"$CARGO_TARGET_DIR/release/openergy-e2e" run --scenario 'S0-*' \
  --port-range 25710-25729 --lab-port-range 25730-25749 \
  --report /tmp/oe-wayland-smoke.json
```

The host run deliberately removes WAYLAND_DISPLAY. It does not connect to the
host bus. All native runs use private mode-0700 HOME/XDG paths and owned socket
identities. The lab passes a clean environment and points the system bus at
an unavailable private path. It never uses host input/DRM devices, desktop
sockets or the installed daemon. It uses no TCP ports. The S0 regression uses
separate virtual daemons on loopback, with no discovery or production port.
No unit tests or mocks were added or run.

**Probe findings appendix.**

Weston headless advertises `wl_output` v4 and xdg-output v2. It reports a real
logical 1280×720 output at (0,0), a matching pixel mode and integer scale 1.
The probe keeps fractional scale unknown. It does not infer EI regions from
these values. Vendor signals are `weston_capture_v1`, `weston_desktop_shell`
and output make/model `weston`/`headless`; these do not gate capabilities.

This Weston configuration has no `wl_seat`, virtual keyboard/pointer,
wlroots screencopy, or data-control globals. Relative-pointer and pointer
constraints are present; they do not supply global capture. Both server and
client portal roles are unsupported when the private bus has no portal owner.
This confirms the design's Weston lane is discovery/refusal evidence, not
native input support. The seat/keymap branch compiles but has no positive
seat evidence in this lane.

A separately started real xdg-desktop-portal frontend without an input backend
can be introspected but supplies neither RemoteDesktop nor InputCapture.
A frontend package or owned bus name alone therefore does not imply input
portal availability. The extra real run caught the default XML parser's
DOCTYPE rejection; it was fixed and the real lane was repeated. No fake portal
was used. Portal u32 property reading uses busctl's scalar `get-property` JSON
shape, distinct from the argument array returned by `call`.

Missing guard/private bus identity is refused before connections. A stopped
real compositor triggers the five-second discovery deadline. Removing the
owned compositor produces a prompt nonzero connection failure; the probe
never tries another display. The supervisor removes all run-owned process
groups and temporary session directories on success and failure.

Mutter/KWin native portal acceptance is excluded. Neither compositor, a
GNOME/Plasma session, matching portal backend nor PipeWire is provisioned in
this host lane. There is no nested desktop session here. This does not prove
that those stacks cannot be provisioned; it means their headless feasibility
and IC/RD/EI behavior were not verified. No GNOME/KDE support claim is made.
No running RD/IC interface was available, so positive input-portal version,
device-mask and candidate-after-grant branches remain unverified by this lane.
Only Linux ARM64 was built and run; x86_64 packaging remains unverified.
The native host lane has no container image digest (recorded as null).

M1 is discovery infrastructure. M2 can build on the real discovery command
and isolated session guard, but still requires independently provisioned
Mutter and KWin portal lanes, explicit authorization, EI keyboard/pointer
and absolute-region acceptance. Server readiness still requires the capture
contract and held-transition work in M3. No design-doc edit was needed.

Validation passed: locked release probe build; final probe Clippy with
warnings denied; workspace/all-target release Clippy; locked daemon/runner
release build; formatting; regenerated notice check; and S0 smoke (1/1 passed).
The workspace Clippy ran before the final probe-only fixes; the final targeted
Clippy covers those fixes. The S0 JSON is retained in `oe-wayland-smoke.json`.
No owned Weston, portal frontend or private session bus remained after cleanup.

The final lab results were:

| Real flow | Exit | Seconds |
| --- | ---: | ---: |
| no-wayland | 1 | 0.005 |
| guard-refusal | 1 | 0.041 |
| bus-refusal | 1 | 0.022 |
| weston-discover | 0 | 0.046 |
| portal-frontend-discover | 0 | 0.041 |
| compositor-stalled | 1 | 5.011 |
| compositor-removed | 1 | 0.007 |
| bus-removed | 1 | 0.008 |

Environment, exact compositor flags, package versions and binary SHA-256 are in
[environment.json](environment.json). Redacted native process logs are in
[weston.log](weston.log), [bus.log](bus.log) and [portal.log](portal.log).
The default no-portal run and the optional real-frontend run are separate probe
invocations on the same owned compositor.

Exact host output, exit 1:

```json
{"capability_matrix":{"client":{"possible":"unknown","ready":false,"reasons":["no Wayland session: WAYLAND_DISPLAY is not set"]},"server":{"possible":"unknown","ready":false,"reasons":["no Wayland session: WAYLAND_DISPLAY is not set"]}},"errors":["no Wayland session: WAYLAND_DISPLAY is not set"],"mode":"discover","schema_version":1,"status":"error"}
```

Exact host stderr:

```text
wayland_probe: no Wayland session: WAYLAND_DISPLAY is not set
```

Exact Weston/no-portal discovery stdout, exit 0 (stderr empty):

```json
{"authorization":"not_requested","capability_matrix":{"client":{"possible":"unsupported","ready":false,"reasons":["wayland_portal_missing","authorization_not_requested","EI_absolute_regions_and_pointer_keyboard_devices_unverified","repeat_locks_text_unverified"]},"optional_direct_client":{"possible":"unsupported","ready":false,"reasons":["requires_virtual_keyboard_and_pointer_explicit_opt_in_and_native_acceptance","absolute_mapping_and_policy_unverified; clipboard_requires_separate_data_control","virtual_input_and_screencopy_do_not_supply_server_capture"]},"server":{"possible":"unsupported","ready":false,"reasons":["wayland_portal_missing","authorization_not_requested","EI_receiver_and_barriers_unverified","capture_contract_and_held_transition_acceptance_required","no_passive_global_input_or_inactive_local_hotkeys","no_immediate_grab_or_PassOnce","no_global_cursor_query_hide_or_guaranteed_warp"]}},"compositor":{"desktop_environment_hint":null,"identity":"not_standardized_by_wayland","identity_basis":"environment hints and vendor globals only; never used for capability gating","session_desktop_hint":null,"session_type_hint":"wayland"},"ei":{"absolute_regions":null,"devices":null,"protocol_version":null,"reason":"portal EIS FD requires authorization; no direct socket or negotiated EI capabilities probed","socket_connected":false,"socket_environment_present":false,"socket_fact":"not_configured"},"mode":"discover","portals":{"auto_start":false,"interfaces":{},"status":"portal_service_missing"},"schema_version":1,"status":"ok","wayland":{"geometry_note":"xdg-output is supplemental logical geometry, not EI regions or IC zones; no pixel/scale inference","globals":[{"global":1,"interface":"wl_compositor","version":5},{"global":2,"interface":"wl_subcompositor","version":1},{"global":3,"interface":"wp_viewporter","version":1},{"global":4,"interface":"zxdg_output_manager_v1","version":2},{"global":5,"interface":"wp_presentation","version":1},{"global":6,"interface":"wp_single_pixel_buffer_manager_v1","version":1},{"global":7,"interface":"wp_tearing_control_manager_v1","version":1},{"global":8,"interface":"zwp_relative_pointer_manager_v1","version":1},{"global":9,"interface":"zwp_pointer_constraints_v1","version":1},{"global":10,"interface":"zwp_input_timestamps_manager_v1","version":1},{"global":11,"interface":"weston_capture_v1","version":1},{"global":12,"interface":"wl_data_device_manager","version":3},{"global":13,"interface":"wl_shm","version":1},{"global":14,"interface":"zwp_linux_explicit_synchronization_v1","version":2},{"global":15,"interface":"wl_output","version":4},{"global":16,"interface":"zwp_input_panel_v1","version":1},{"global":17,"interface":"zwp_text_input_manager_v1","version":1},{"global":18,"interface":"xdg_wm_base","version":5},{"global":19,"interface":"weston_desktop_shell","version":1}],"keyboard_note":"seat/keymap metadata only; no key events or keymap contents retained","outputs":[{"core_geometry":{"make":"weston","model":"headless","physical_height_mm":720,"physical_width_mm":1280,"transform":"Value(Normal)","x":0,"y":0},"current_mode":{"height_px":720,"refresh_millihz":60000,"width_px":1280},"description":"headless","fractional_scale":null,"global":15,"integer_scale":1,"logical_position":{"x":0,"y":0},"logical_size":{"height":720,"width":1280},"logical_source":"xdg_output","name":"headless"}],"relevant_globals":{"ext_data_control_manager_v1":{"advertised":false,"versions":[]},"ext_image_copy_capture_manager_v1":{"advertised":false,"versions":[]},"ext_output_image_capture_source_manager_v1":{"advertised":false,"versions":[]},"wl_output":{"advertised":true,"versions":[4]},"wl_seat":{"advertised":false,"versions":[]},"wp_pointer_warp_v1":{"advertised":false,"versions":[]},"zwlr_data_control_manager_v1":{"advertised":false,"versions":[]},"zwlr_screencopy_manager_v1":{"advertised":false,"versions":[]},"zwlr_virtual_pointer_manager_v1":{"advertised":false,"versions":[]},"zwp_keyboard_shortcuts_inhibit_manager_v1":{"advertised":false,"versions":[]},"zwp_pointer_constraints_v1":{"advertised":true,"versions":[1]},"zwp_relative_pointer_manager_v1":{"advertised":true,"versions":[1]},"zwp_virtual_keyboard_manager_v1":{"advertised":false,"versions":[]},"zxdg_output_manager_v1":{"advertised":true,"versions":[2]}},"seats":[]}}
```

The supplemental real-frontend output is retained verbatim in
[portal-frontend-discover.json](portal-frontend-discover.json), with empty stderr.
It reports `portals.status="available"` and `advertised=false` for RD, IC,
Clipboard and ScreenCast. Client/server reasons identify their missing input
interfaces separately. The default no-portal result reports the distinct
`portal_service_missing` status instead.

Exact stopped-compositor output, exit 1:

```json
{"capability_matrix":{"client":{"possible":"unknown","ready":false,"reasons":["wayland_discovery_timeout_or_worker_lost"]},"server":{"possible":"unknown","ready":false,"reasons":["wayland_discovery_timeout_or_worker_lost"]}},"errors":["wayland_discovery_timeout_or_worker_lost"],"mode":"discover","schema_version":1,"status":"error"}
```

Exact removed-compositor output, exit 1:

```json
{"capability_matrix":{"client":{"possible":"unknown","ready":false,"reasons":["wayland_connection_failed: private compositor socket missing"]},"server":{"possible":"unknown","ready":false,"reasons":["wayland_connection_failed: private compositor socket missing"]}},"errors":["wayland_connection_failed: private compositor socket missing"],"mode":"discover","schema_version":1,"status":"error"}
```

A real CLI invocation with a non-UTF-8 argument also exited 1 with the usage
error, without aborting. Its JSON/stderr are retained as `invalid-utf8-argument.*`.
