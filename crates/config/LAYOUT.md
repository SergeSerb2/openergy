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
Coordinates and rect extents must fit positive i32 space. Move the layout's
origin before supplying negative coordinates: the frozen service expansion
clips at zero, so negative origins do not preserve geometry.

## Exact converter algorithm

The frozen architecture §2.8 uses the Synergy service algorithm in
`docs/re/02-service.md` B5, not nearest-center selection. It is implemented
independently from those documented rules.

1. For each original rect `(x,y,w,h)`, compute expanded bounds:
   `left=max(0,x-0.5)`, `top=max(0,y-0.5)`, `right=x+w+0.5`,
   `bottom=y+h+0.5`. Width and height follow from those bounds.
2. Examine each source/target pair and each direction. Left shifts the
   expanded source's x by -1 without changing width. Up shifts y by -1
   without changing height. Right grows width by 1. Down grows height by 1.
3. Require strictly positive intersection of this probe and the expanded
   target on both axes. Boundary-only contact does not collide. For normally
   separated rectangles this admits gaps smaller than two layout units.
4. Intersect the **unprobed expanded** perpendicular spans. Left/right use
   Y; up/down use X. For shared span `[a,b]`, source origin `o`, length `d`,
   use `(round(100*(a-o)/d), round(100*(b-o)/d))`. Compute the target's
   range independently, with its own origin and length. No extra clamp.
5. A later identical `(source,edge,source-range)` key replaces the earlier
   one, as the service does. Different intersecting source ranges fail
   validation. Output order is screen order, left/right/top/bottom order,
   then target order.

The Step 0 `DerivedLink` range fields are `(u8,u8)`. An overlap that rounds
to an empty interval cannot be exported as valid Synergy config. It is omitted.
Warnings call out expanded-rect collisions and this integer-precision limit.
The public `derive_links` signature has no warning return; patches and
import/export CLI paths surface warnings. Input validation rejects invalid
explicit ranges rather than dropping them.

At runtime each percentage interval is start-inclusive and end-exclusive.
For source fraction `p` inside `[s0,s1)`, map to
`t0 + ((p-s0)/(s1-s0))*(t1-t0)`. A point exactly at `s1` belongs to the next
interval. Export writes these integer endpoints directly, avoiding another
rounding pass.

## Partial overlap: 1512×982 Mac beside 2560×1440 PC

The original points are Mac `(0,300,1512,982)` and PC
`(1512,0,2560,1440)`. Expanded Mac bounds are
`x=[0,1512.5], y=[299.5,1282.5]`, with height 983.
Expanded PC bounds are `x=[1511.5,4072.5], y=[0,1440.5]`.
Their shared Y interval is `[299.5,1282.5]`:

- Mac: `round(100*0/983)=0`, `round(100*983/983)=100`.
- PC: `round(100*299.5/1440.5)=21`,
  `round(100*1282.5/1440.5)=89`.

The useful pair is `Mac.right(0,100) → PC(21,89)` and
`PC.left(21,89) → Mac(0,100)`.

**Frozen-contract edge case:** with zero original spacing, expansion itself
already overlaps by one unit. The PC's right probe also collides with the Mac.
The literal converter therefore adds `PC.right(21,89) → Mac(0,100)`.
Up/down collisions here round to empty X ranges and are omitted. This
three-link result is recorded through the real daemon in VERIFICATION.md.
WP2 preserves the specified converter rather than silently replacing its
rules. This behavior needs an architecture decision before the layout UI
assumes that all derived directions face the target.

Two supported ways to obtain just the useful pair are demonstrated by the
real verification script:

- Set the PC's `[screen.links] right = ""`; left still derives normally.
- Move PC x to 1513, the service's one-unit layout spacing. Expanded X spans
  now only meet; just the outward left/right probes collide. Y percentages
  stay 21–89.

A future architecture amendment could require an original-rect direction
check before each probe. No signature change is needed for that rule.

## Three-screen L

`fixtures/layout/l-shape.toml` has A=`studio-pc` `(0,0,1000,1000)`,
B=`macbook` `(1001,0,1000,1000)`, C=`linux-box` `(0,1001,500,1000)`.
The one-unit spacing follows the original editor. Expanded bounds are:

- A: X and Y `[0,1000.5]`.
- B: X `[1000.5,2001.5]`, Y `[0,1000.5]`.
- C: X `[0,500.5]`, Y `[1000.5,2001.5]`.

A/B have full Y overlap. A/C share X `[0,500.5]`:
`round(100*500.5/1000.5)=50` for A and 100 for C. Thus exactly:

```text
studio-pc.right(0,100) = macbook(0,100)
macbook.left(0,100) = studio-pc(0,100)
studio-pc.down(0,50) = linux-box(0,100)
linux-box.up(0,100) = studio-pc(0,50)
```

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
