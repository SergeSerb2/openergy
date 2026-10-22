# Layout and config contract

`Config::load_toml` and `from_json` reject unknown keys at every object level.
TOML errors include source lines. Omitted fields use the Step 0 defaults.
The JSON list key is `screen`, not `screens`. `super_` remains `super_` in
modifier JSON/TOML. Existing enum names stay in snake_case.

`apply_merge_patch(&Config, &serde_json::Value)` implements RFC 7386 on a
separate JSON value, then deserializes and validates it. Arrays replace whole
arrays. Null object members remove keys; defaults apply to omitted fields.
An object patch first replaces a nonobject target with an empty object.
An invalid result returns `ConfigError` without modifying the input config.
The caller owns persistence and live replacement.

Validation checks names and aliases for global uniqueness; named targets;
version 1; nonzero ports up to 65535; finite positive dimensions and scroll
sensitivity; nonnegative layout origins; positive-area rect intersections;
wire integer bounds; modifier targets; hotkey syntax; and overlapping source
ranges on each edge. Port in `server.port` wins over a port in `listen`;
a patch returns a warning if they differ. Heartbeat zero disables timers.
Coordinates and rect extents must fit positive i32 space. Native layouts still
require nonnegative origins under the existing config validation rules.

## Native screen-point layouts

Architecture decision: Openergy's `[[screen]] rect` values use screen points.
The app snaps adjacent edges to the same coordinate, with zero spacing.
`derive_links` uses the original rects without expansion or probes:

- A right link from A to B requires `abs(A.max_x - B.min_x) <= 1.0` and a
  Y-span intersection whose length is **greater than** 1.0 point.
- Left compares `A.min_x` with `B.max_x`, also using Y overlap.
- Top compares `A.min_y` with `B.max_y`, using X overlap.
- Bottom compares `A.max_y` with `B.min_y`, using X overlap.

The target must lie on the indicated side. Origin order enforces this even
for screens smaller than the one-point tolerance. A corner-only contact and
an overlap of exactly one point produce no link. A gap of exactly one point
is allowed; a larger gap is not.

For shared perpendicular span `[s,e]`, origin `o`, and length `d`, the range
is `(round(100*(s-o)/d), round(100*(e-o)/d))`. Compute source and target
ranges independently from their own origins and lengths. Explicit links
still override each screen/edge. Different overlapping source ranges fail
validation; native derivation does not overwrite an earlier candidate.
Output order is screen order, left/right/top/bottom order, then target order.

The frozen range fields are `(u8,u8)`. Omit a derived link if either range
rounds to an empty interval. Config patches and the import/export CLI paths
return a warning for this case. Suppressed edges produce no rounding warning.
The public `derive_links` signature remains unchanged and has no warning
return. Explicit empty ranges are invalid rather than silently dropped.

At runtime each percentage interval is start-inclusive and end-exclusive.
For source fraction `p` inside `[s0,s1)`, map to
`t0 + ((p-s0)/(s1-s0))*(t1-t0)`. A point exactly at `s1` belongs to the next
interval. Export writes the integer endpoints directly.

## Partial overlap: 1512×982 Mac beside 2560×1440 PC

The original points are Mac `(0,300,1512,982)` and PC
`(1512,0,2560,1440)`. The Mac's right edge and PC's left edge both equal
1512. Their shared Y interval is `[300,1282]`, with length 982:

- Mac: `round(100*(300-300)/982)=0`,
  `round(100*(1282-300)/982)=100`.
- PC: `round(100*300/1440)=21`, `round(100*1282/1440)=89`.

The result is **exactly**:

```text
Mac.right(0,100) = PC(21,89)
PC.left(21,89) = Mac(0,100)
```

There is no `PC.right → Mac` link. No explicit suppression or layout gap
is required. Moving PC x to 1513 keeps the same pair because the gap equals
the one-point tolerance. Moving it to 1513.01 removes both links.
The real daemon evidence is in VERIFICATION.md.

## Three-screen L

`fixtures/layout/l-shape.toml` has A=`studio-pc` `(0,0,1000,1000)`,
B=`macbook` `(1000,0,1000,1000)`, C=`linux-box` `(0,1000,500,1000)`.
Both adjacent pairs have exactly touching edges. A/B share the whole Y span
`[0,1000]`. A/C share X `[0,500]`: `round(100*500/1000)=50` for A and
100 for C. The four links are:

```text
studio-pc.right(0,100) = macbook(0,100)
macbook.left(0,100) = studio-pc(0,100)
studio-pc.down(0,50) = linux-box(0,100)
linux-box.up(0,100) = studio-pc(0,50)
```

## Synergy 3 editor converter: import only

`derive_synergy3_links` is a separate crate-private function used only by the
`db.json` importer. It retains the independently implemented service converter
from `docs/re/02-service.md` B5:

1. Expand original editor bounds by 0.5 on each side, with left/top clamped
   to zero and the sizes adjusted for clipping.
2. For every ordered pair and direction, probe the source: left/up move
   the origin by -1 without changing its size; right/down grow the size by 1.
3. Require strictly positive intersection of the probe and expanded target.
4. Use the **unprobed expanded** perpendicular overlap for percentages,
   rounded independently for source and target, with no extra clamp.
5. Later identical `(source,edge,source-range)` keys replace earlier keys.
   Omit rounded-empty ranges with a warning.

Import reports warn when expanded editor rects already overlap, because the
historical converter can then emit extra directions. Those service results
are retained as explicit imported links for fidelity; native derivation never
uses this converter. The normal db fixture's PC left range remains `[22,91]`,
whereas unexpanded native points would give `[23,91]`. The verification script
also imports a zero-spacing editor fixture and confirms its legacy extra
right link survives **only** in the imported explicit graph.

## Explicit overrides and additive model fields

Existing `links.left/right/top/bottom: Option<String>` keep their signatures.
`None` derives that edge. A nonempty name maps its whole edge to the whole
target edge. `Some("")` disconnects that edge explicitly. An empty `links`
table alone overrides no edges.

WP2 adds `LinksCfg.ranges: Vec<RangedLink>` (default empty). A range list
for an edge replaces derivation for that edge. A nonempty whole-edge target
and ranges on the same edge are an error. An empty-string target can coexist
with ranges; this makes imported disconnected-edge declarations unambiguous.

```rust
pub struct RangedLink {
    pub edge: Edge,
    pub from_range: (u8, u8),
    pub to: String,
    pub to_range: (u8, u8),
}
```

`ScreenCfg.options: Option<ScreenOptions>` retains half-duplex toggles,
per-screen dead corners/size, Xinerama and preserveFocus settings, with
optional values to preserve global fallback. `Config.options: SynergyOptions`
retains protocol=barrier, keep-foreground, lock controls, and modifier gates.
Default global options are omitted in serialization. These are additions
inside the owned crate; original fields and functions keep their types.
Consumers constructing Rust struct literals must supply these new fields.
The current workspace builds without changes outside this crate.

`ImportReport.trusted_servers: Vec<String>` contains normalized SHA-256 hex
candidates from `tls/trusted-servers`. Importing never changes a trust store.
The human CLI reports only the count. Explicit `--json` returns candidates
for callers to route into WP4's approval flow.

## Import and export boundaries

A Synergy 3 directory reads only `db.json`, `local.json`, `synergy.conf`,
`synergy-core.conf`, and `tls/trusted-servers`. It never opens certificate
keys, follows configured file paths, or contacts a server. File reads are
bounded to 16 MiB. IDs are transient join keys and wire-name suffix inputs;
serial, license, telemetry, Sentry and account fields are ignored. Warning
messages contain field paths and kinds, not their values.

Precedence: db v17 maps global/screen settings; local v7 identifies the local
record and role and maps local input settings; a present `synergy.conf`
replaces screen membership/links and overlays its options while retaining
matching db metadata; the core INI finally supplies the actual role/name,
port, TLS, remote host, preventSleep, languageSync and invertYScroll.
Unsupported INI enum spellings fail instead of guessing. A false Synergy
`checkPeerFingerprints` never disables Openergy's default client-trust policy.

Editor rects keep their original uniform units rather than inventing physical
display dimensions. Service links are computed on those rects and retained as
explicit ranges, so later physical rescaling cannot change imported ranges.
Different dead-corner sizes use the last positive value in original JSON
insertion order, matching the service; the report warns about the collapse.
`performance.limit` is KiB (4000 → 4,096,000 bytes). `performance.sync` has
no documented service/core effect and gets a warning; directional clipboard
permissions retain their defaults. Disabled `local-network.keep_alive` omits
heartbeat in Synergy, so the 3000 ms default remains, with a warning.

Only toggle lock hotkeys map to the current Openergy hotkey field. Other
press/release actions produce warnings. Keyboard and mouse triggers both
survive. Synergy's ignored `action.enabled` flag gets a warning when false.
Fractional imported link endpoints round to integers with a warning.
A plain `synergy.conf` cannot identify the local machine; selecting its first
screen as server is explicitly warned. Confirm that choice before use.

Export preserves the representable core view: screen names, aliases, screen
options, ranged links, global core options and the toggle lock binding.
It cannot carry role, TLS, remote host, physical rects, local input settings
or directional clipboard permissions. The CLI warns about those limits.
Export→import→export is byte-identical for both committed fixtures.
Full Openergy config equality is not possible through the Synergy language.

CLI output files use `cli::write_atomic(&Path, &str) -> Result<(), ConfigError>`:
an exclusive same-directory 0600 temp file, full write, sync, then rename.
Failure removes only that writer's temp file. WP6 owns daemon persistence;
Step 0's daemon writer is separate and does not yet enforce 0600.
