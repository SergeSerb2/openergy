# Openergy reverse engineering: Synergy 3 service

Research date: 2026-10-21. Target: `/Applications/Synergy.app/Contents/MacOS/synergy-service`, Synergy **3.7.0-beta**, macOS arm64, Node **24.16.0**, core **1.21.1-beta**. This is a clean-room behavioral specification. No replacement code was written.

**Method and limits.** The installed app and user configuration were read only. Binary and analysis copies are in `/tmp/openergy-re/service/`. No Synergy executable was launched, attached to, signalled, stopped, or restarted. No service endpoint was contacted. Runtime evidence consists of `ps`, `lsof`, `launchctl print`, file metadata, and existing logs. The only package commands run were research tools. Sensitive user values are omitted or written as `<redacted>`. Synthetic names in examples are not live computer identifiers.

**Evidence notation.** `S:Lx–y` means LF-based lines in `/tmp/openergy-re/service/extracted/index.cjs`. Its retained comments identify the original bundle module. Function names below are search anchors, not claims that original TypeScript files were shipped. `P` means `/tmp/openergy-re/service/passive-summary.json`; `C` means the read-only user configuration under `~/Library/Preferences/Synergy/`; `R` means REA output in the scratch directory. Findings marked **inference** follow from static code and were not exercised against the live service.

Evidence: embedded build config `S:L73–85`; `C/versions.json` contains only the listed version/platform metadata; `P` records the passive process and socket observations.

## A. Extraction, bundle structure, dependencies, and size

### A1. Exact extraction for this artifact

The executable is **127,473,952 bytes** (121.57 MiB), with one arm64 Mach-O image. The SEA resource is segment **`NODE_SEA`**, section **`__NODE_SEA_BLOB`**. The segment is not named `__NODE_SEA` in this file. Its file offset is **91,734,016** and section size is **7,062,789 bytes** (`0x6bc505`).

Evidence: scratch `load-commands.txt`, `LC_SEGMENT_64` / section fields; `rea-macho.json` reports arm64 and the same section. Artifact SHA-256: `b5f9a851569f9b0aa99f87f784006c48ddf62f0be735f33117c531cd79b93f9a`.

Extraction is a byte-range operation; it does not require Node to load the target:

1. Copy the installed executable to scratch. Read its Mach-O load commands with `otool -l`.
2. Slice the section above into `sea.blob`.
3. In this blob, bytes 0–3 decode as little-endian magic `0x0143da20`. The following observed five bytes are `01 00 00 00 01`. Their complete version-specific meaning was not needed for extraction and is not asserted here.
4. At blob offset **9**, a little-endian unsigned 64-bit length is **28**. At offset **17**, its UTF-8 string is **`build/apps/service/index.cjs`**.
5. At offset **45**, another little-endian unsigned 64-bit length is **7,062,736**. Slice that many bytes from blob offset **53**, or executable offset **91,734,069**.
6. Save that slice as `extracted/index.cjs`. It is readable UTF-8 JavaScript. The slice ends exactly at the section end: **zero trailing bytes**. There is no additional snapshot or code-cache payload in this observed resource.

Evidence: direct inert byte parsing, `sea.blob`, `extracted/index.cjs`; extracted source SHA-256 `5710ae08d0a6f2a59dde7996dc61c1469de1cd3897a7963388163d1b6d648110`. This is an extraction recipe for the inspected file, not a universal SEA format parser.

The source has an esbuild-style CommonJS bootstrap (`__commonJS`, `__esm`, `__toESM`), readable names, module-path comments, and an `index.cjs.map` reference. No source map was present in the SEA resource. The application ends by calling `App.startSafe()`. Shared libraries and Windows/Linux implementations remain in the macOS bundle.

Evidence: `S:L1–90`, `S:L145964` onward, `S:L177018–177022`, final `sourceMappingURL`; `sections.json` records **1,000 path markers / 935 distinct marked paths**. Marked paths include split declarations and repeated initialization blocks, so this is not a count of independent runtime modules.

### A2. REA use and its coverage

Used REA **6.7.0** with `--help`, `doctor`, `inspect-macho`, `inspect-artifact`, and `analyze-javascript-application` on the extracted directory. Large outputs were retained in scratch. No command executed the target payload. `inspect-binary-layout --help` describes an ELF input, so Mach-O inspection used `inspect-macho` instead.

Evidence: `rea-help.txt`, `rea-doctor.json`, `rea-macho.json`, `rea-artifact.json`, `rea-javascript.json`. The JavaScript result reports **1 parsed file, 760,701 AST nodes, 712 recovered modules, 1,533 findings, and 0 parse failures**. REA explicitly reports incomplete semantic coverage and a primitive-string budget limit; its counts are not proof that every semantic relationship was recovered. `doctor` reports missing optional native analysis engines; the static inspections and JavaScript parser completed.

### A3. What takes space

The JS resource is only **5.54%** of the executable. `__TEXT` is **90,144,768 bytes**, and `__LINKEDIT` is **28,662,048 bytes**. Thus a native replacement can remove much more than the application JS alone. Segment sizes describe file contents; they are not resident-memory measurements.

Evidence: Mach-O segment sizes in `load-commands.txt` / `rea-macho.json`; file size and section slice above.

Within the JS slice, the following sizes sum text between module-path markers. Nested dependencies are attributed to the innermost package, and repeated bundled copies are included. These are shipped source-byte estimates, not install sizes or heap sizes.

| Package or family | Source bytes | Main explanation |
|---|---:|---|
| All Sentry namespaces | 1,413,628 | Node SDK, integrations, utilities, three copies of `@sentry/core` |
| `iconv-lite` | 1,109,195 | Two nested copies, under `body-parser` and `raw-body` |
| `@sentry/core` alone | 757,256 | Three copies of roughly 252 KB each; included in Sentry total above |
| `systeminformation` | 714,620 | Broad cross-platform system inspection library |
| `mime-db` | 658,784 | Three databases: root, Socket.IO, Engine.IO |
| `node-forge` | 582,692 | Broad cryptography library used for PEM/certificate work |
| `zod` | 544,898 | Schema validation |
| `ws` | 378,187 | Root plus nested Engine.IO and Socket.IO-adapter copies |
| Service-owned modules | 357,188 | Lifecycle, config, sync, API, platform installers |
| Shared `libs/common` | 24,278 | License/settings/env/layout support |

Evidence: `sections.json`, `dependency-sizes.json`; package boundaries are retained source comments. Own service/common/default-data text totals **383,048 bytes**, about **5.42% of the JS slice**. Sentry, charset tables, MIME tables, and WebSocket duplication are specific size-reduction opportunities.

### A4. Complete observed package inventory and version confidence

The marked source identifies **119 distinct innermost third-party package names**. The table below is exhaustive for those markers, including transitive and nested packages. It does not enumerate Node's built-in libraries. A package can be bundled without its version metadata.

Version labels:

- **B**: exact version is present in the service bundle itself.
- **M**: version range declared by the companion app's root `package.json`; this is not a resolved service lockfile.
- **C**: exact version found in a corresponding companion `app.asar` package manifest; this is supporting metadata, not proof of the service's resolved copy.
- **U**: no version established. Do not replace this with a guessed current npm version.

The companion metadata was read directly from ASAR byte ranges, without running Electron. It is imperfect evidence: its `@sentry/node` manifest is **7.112.0**, while the service's retained SDK banner is **7.120.4** and the app declaration is `~7.120.4`. The service's B evidence takes precedence. Multiple nested copies can also have different versions even when C exposes only one.

Evidence: service `systeminformation/package.json` at `S:L35079`, `socket.io/package.json` at `S:L144419`, Zod version object at `S:L149059–149065`, Sentry SDK constants at `S:L5870,L12652,L26210` and banner at `S:L177025–177026`, localForage license metadata at `S:L177028–177034`; scratch `packaged-dependencies.json` retains only package names, versions, ranges, and metadata paths.

| Package | Version / confidence | Bundled text bytes | Evidence anchor in S |
|---|---|---:|---|
| `@leichtgewicht/ip-codec` | Unknown U | 6,422 | L103771 `node_modules/@leichtgewicht/ip-codec/index.cjs` |
| `@logdna/tail-file` | ~4.0.2 M | 10,010 | L71739 `node_modules/@logdna/tail-file/lib/tail-file.js` |
| `@sentry-internal/tracing` | 7.120.4 C | 144,577 | L17938 `node_modules/@sentry-internal/tracing/cjs/common/debug-build.js` |
| `@sentry/core` | 7.120.4 B | 757,256 | L4374 `node_modules/@sentry/node/node_modules/@sentry/core/cjs/debug-build.js` |
| `@sentry/integrations` | 7.120.4 C | 38,631 | L31496 `node_modules/@sentry/integrations/cjs/captureconsole.js` |
| `@sentry/node` | 7.120.4 B | 327,751 | L21494 `node_modules/@sentry/node/cjs/tracing/index.js` |
| `@sentry/utils` | ~7.120.4 M | 145,413 | L413 `node_modules/@sentry/utils/cjs/is.js` |
| `@socket.io/component-emitter` | 3.1.2 C | 2,575 | L136748 `node_modules/@socket.io/component-emitter/lib/cjs/index.js` |
| `@vscode/sudo-prompt` | ~9.3.2 M | 31,580 | L71145 `node_modules/@vscode/sudo-prompt/index.js` |
| `accepts` | Unknown U | 8,744 | L97618 `node_modules/accepts/index.js` |
| `async-mutex` | ~0.5.0 M | 9,446 | L168786 `node_modules/async-mutex/index.mjs` |
| `base64id` | Unknown U | 2,302 | L120009 `node_modules/base64id/lib/base64id.js` |
| `bencode` | Unknown U | 17,904 | L107256 `node_modules/bencode/lib/encode.js` |
| `bittorrent-dht` | Unknown U | 26,324 | L109184 `node_modules/bittorrent-dht/peer-store.js` |
| `body-parser` | Unknown U | 14,977 | L91291 `node_modules/body-parser/lib/utils.js` |
| `buffer-equals` | Unknown U | 706 | L107835 `node_modules/buffer-equals/index.js` |
| `buffer-from` | Unknown U | 1,969 | L110423 `node_modules/buffer-from/index.js` |
| `bytes` | Unknown U | 2,736 | L73504 `node_modules/bytes/index.js` |
| `call-bind-apply-helpers` | 1.0.2 C | 2,019 | L92644 `node_modules/call-bind-apply-helpers/functionCall.js` |
| `call-bound` | Unknown U | 822 | L93099 `node_modules/call-bound/index.js` |
| `commander` | ~15.0.0 M | 110,480 | L163875 `node_modules/commander/index.js` |
| `content-disposition` | Unknown U | 7,715 | L98004 `node_modules/content-disposition/index.js` |
| `content-type` | Unknown U | 3,736 | L81510 `node_modules/content-type/index.js` |
| `cookie` | Unknown U | 5,389 | L98234 `node_modules/cookie/index.js` |
| `cookie-signature` | Unknown U | 1,164 | L98214 `node_modules/cookie-signature/index.js` |
| `cors` | Unknown U | 6,920 | L134828 `node_modules/cors/lib/index.js` |
| `date-fns` | ~4.4.0 M | 7,341 | L169614 `node_modules/date-fns/constructFrom.js` |
| `debug` | 4.4.3 C | 52,469 | L72146 `node_modules/debug/src/common.js` |
| `depd` | Unknown U | 10,060 | L72803 `node_modules/depd/index.js` |
| `discovery-channel` | ~5.5.1 M | 7,928 | L110474 `node_modules/discovery-channel/index.js` |
| `dns-discovery` | Unknown U | 26,209 | L106447 `node_modules/dns-discovery/store.js` |
| `dns-packet` | Unknown U | 61,260 | L103277 `node_modules/dns-packet/types.js` |
| `dns-socket` | Unknown U | 7,095 | L105395 `node_modules/dns-socket/index.js` |
| `dotenv` | ~17.4.2 M | 11,816 | L92 `node_modules/dotenv/lib/main.js` |
| `dunder-proto` | 1.0.1 C | 1,096 | L92702 `node_modules/dunder-proto/get.js` |
| `ee-first` | Unknown U | 1,614 | L81348 `node_modules/ee-first/index.js` |
| `emittery` | ~2.0.0 M | 27,831 | L161804 `node_modules/emittery/index.js` |
| `encodeurl` | Unknown U | 715 | L94332 `node_modules/encodeurl/index.js` |
| `engine.io` | Unknown U | 98,791 | L120358 `node_modules/engine.io/build/parser-v3/utf8.js` |
| `engine.io-parser` | 5.2.3 C | 10,542 | L120073 `node_modules/engine.io-parser/build/cjs/commons.js` |
| `env-paths` | ~4.0.0 M | 2,617 | L146103 `node_modules/env-paths/index.js` |
| `es-define-property` | 1.0.1 C | 476 | L92440 `node_modules/es-define-property/index.js` |
| `es-errors` | 1.3.0 C | 1,581 | L91626 `node_modules/es-errors/type.js` |
| `es-object-atoms` | 1.1.2 C | 246 | L92271 `node_modules/es-object-atoms/index.js` |
| `escape-html` | Unknown U | 1,266 | L94347 `node_modules/escape-html/index.js` |
| `etag` | Unknown U | 1,829 | L94703 `node_modules/etag/index.js` |
| `express` | ~5.2.1 M | 38,093 | L94608 `node_modules/express/lib/view.js` |
| `fast-json-patch` | ~3.1.1 M | 27,674 | L145266 `node_modules/fast-json-patch/commonjs/helpers.js` |
| `finalhandler` | Unknown U | 4,187 | L94480 `node_modules/finalhandler/index.js` |
| `forwarded` | Unknown U | 1,337 | L94746 `node_modules/forwarded/index.js` |
| `fresh` | Unknown U | 2,173 | L97700 `node_modules/fresh/index.js` |
| `function-bind` | 1.1.2 C | 2,631 | L92557 `node_modules/function-bind/implementation.js` |
| `get-intrinsic` | 1.3.0 C | 15,128 | L92767 `node_modules/get-intrinsic/index.js` |
| `get-proto` | 1.0.1 C | 1,438 | L92538 `node_modules/get-proto/Reflect.getPrototypeOf.js` |
| `gopd` | 1.2.0 C | 594 | L92414 `node_modules/gopd/gOPD.js` |
| `has-flag` | Unknown U | 537 | L72497 `node_modules/has-flag/index.js` |
| `has-symbols` | 1.1.0 C | 2,468 | L92457 `node_modules/has-symbols/shams.js` |
| `hasown` | 2.0.4 C | 368 | L92755 `node_modules/hasown/index.js` |
| `http-errors` | Unknown U | 5,820 | L73339 `node_modules/http-errors/index.js` |
| `iconv-lite` | Unknown U | 1,109,195 | L73663 `node_modules/raw-body/node_modules/iconv-lite/lib/bom-handling.js` |
| `inherits` | Unknown U | 1,431 | L73275 `node_modules/inherits/inherits_browser.js` |
| `ipaddr.js` | Unknown U | 21,978 | L94794 `node_modules/ipaddr.js/lib/ipaddr.js` |
| `is-promise` | Unknown U | 423 | L95784 `node_modules/is-promise/index.js` |
| `is-safe-filename` | 0.1.1 C | 789 | L146109 `node_modules/is-safe-filename/index.js` |
| `k-bucket` | Unknown U | 13,596 | L107872 `node_modules/k-bucket/index.js` |
| `k-rpc` | Unknown U | 10,399 | L108749 `node_modules/k-rpc/index.js` |
| `k-rpc-socket` | Unknown U | 6,679 | L108383 `node_modules/k-rpc-socket/index.js` |
| `localforage` | 1.10.0 B | 84,084 | L31868 `node_modules/localforage/dist/localforage.js` |
| `lru` | Unknown U | 7,448 | L106343 `node_modules/dns-discovery/node_modules/lru/index.js` |
| `math-intrinsics` | 1.1.0 C | 2,106 | L92334 `node_modules/math-intrinsics/abs.js` |
| `media-typer` | Unknown U | 2,338 | L91125 `node_modules/media-typer/index.js` |
| `merge-descriptors` | Unknown U | 866 | L94307 `node_modules/merge-descriptors/index.js` |
| `mime-db` | 1.52.0 C | 658,784 | L81615 `node_modules/mime-db/db.json` |
| `mime-types` | 2.1.35 C | 11,296 | L90972 `node_modules/mime-types/mimeScore.js` |
| `ms` | ~2.1.3 M | 8,439 | L72028 `node_modules/ms/index.js` |
| `multicast-dns` | Unknown U | 6,172 | L105696 `node_modules/multicast-dns/index.js` |
| `negotiator` | Unknown U | 49,388 | L97100 `node_modules/negotiator/lib/charset.js` |
| `network-address` | Unknown U | 1,333 | L105607 `node_modules/network-address/index.js` |
| `node-forge` | ~1.4.0 M | 582,692 | L53236 `node_modules/node-forge/lib/forge.js` |
| `object-assign` | Unknown U | 2,211 | L134757 `node_modules/object-assign/index.js` |
| `object-inspect` | Unknown U | 19,423 | L91635 `node_modules/object-inspect/util.inspect.js` |
| `on-finished` | ~2.4.1 M | 3,243 | L81405 `node_modules/on-finished/index.js` |
| `on-headers` | ~1.1.0 M | 3,197 | L110701 `node_modules/on-headers/index.js` |
| `once` | Unknown U | 1,285 | L95738 `node_modules/once/once.js` |
| `parseurl` | Unknown U | 2,288 | L94395 `node_modules/parseurl/index.js` |
| `path-to-regexp` | Unknown U | 12,964 | L95797 `node_modules/path-to-regexp/dist/index.js` |
| `pretty-hash` | Unknown U | 455 | L110408 `node_modules/pretty-hash/index.js` |
| `proxy-addr` | Unknown U | 5,096 | L95418 `node_modules/proxy-addr/index.js` |
| `qs` | Unknown U | 35,173 | L93294 `node_modules/qs/lib/formats.js` |
| `ramda` | ~0.32.0 M | 4,089 | L167272 `node_modules/ramda/es/index.js` |
| `randombytes` | Unknown U | 257 | L107863 `node_modules/randombytes/index.js` |
| `range-parser` | Unknown U | 2,343 | L97773 `node_modules/range-parser/index.js` |
| `raw-body` | Unknown U | 6,004 | L77430 `node_modules/raw-body/index.js` |
| `router` | Unknown U | 20,928 | L96167 `node_modules/router/lib/layer.js` |
| `safe-buffer` | Unknown U | 1,843 | L107196 `node_modules/safe-buffer/index.js` |
| `safer-buffer` | Unknown U | 2,395 | L73594 `node_modules/safer-buffer/safer.js` |
| `send` | Unknown U | 17,073 | L98401 `node_modules/send/index.js` |
| `serve-static` | Unknown U | 3,783 | L99430 `node_modules/serve-static/index.js` |
| `setprototypeof` | Unknown U | 636 | L73111 `node_modules/setprototypeof/index.js` |
| `side-channel` | Unknown U | 1,335 | L93254 `node_modules/side-channel/index.js` |
| `side-channel-list` | Unknown U | 2,529 | L92177 `node_modules/side-channel-list/index.js` |
| `side-channel-map` | Unknown U | 1,719 | L93123 `node_modules/side-channel-map/index.js` |
| `side-channel-weakmap` | Unknown U | 2,601 | L93180 `node_modules/side-channel-weakmap/index.js` |
| `simple-sha1` | Unknown U | 541 | L109164 `node_modules/simple-sha1/index.js` |
| `socket.io` | 4.8.3 B | 109,357 | L137260 `node_modules/socket.io/dist/client.js` |
| `socket.io-adapter` | Unknown U | 39,750 | L139467 `node_modules/socket.io-adapter/dist/contrib/yeast.js` |
| `socket.io-parser` | 4.2.6 C | 15,116 | L136827 `node_modules/socket.io-parser/build/cjs/is-binary.js` |
| `statuses` | Unknown U | 4,181 | L73132 `node_modules/statuses/codes.json` |
| `supports-color` | Unknown U | 3,246 | L72511 `node_modules/supports-color/index.js` |
| `systeminformation` | 5.31.7 B | 714,620 | L35079 `node_modules/systeminformation/package.json` |
| `thunky` | Unknown U | 2,262 | L105646 `node_modules/multicast-dns/node_modules/thunky/index.js` |
| `toidentifier` | Unknown U | 445 | L73325 `node_modules/toidentifier/index.js` |
| `type-is` | Unknown U | 3,003 | L91196 `node_modules/type-is/index.js` |
| `unordered-set` | Unknown U | 983 | L106305 `node_modules/unordered-set/index.js` |
| `unpipe` | Unknown U | 1,061 | L77391 `node_modules/unpipe/index.js` |
| `vary` | Unknown U | 2,240 | L98885 `node_modules/vary/index.js` |
| `wrappy` | Unknown U | 928 | L95706 `node_modules/wrappy/wrappy.js` |
| `ws` | ~8.21.0 M | 378,187 | L99585 `node_modules/ws/lib/constants.js` |
| `zod` | 4.4.3 B | 544,898 | L146773 `node_modules/zod/index.js` |

Evidence: every inventory row comes from the named bundle marker; version provenance follows B/M/C/U above. Counts include code that the service does not activate, such as BitTorrent DHT, which discovery explicitly disables.

## B. Service responsibilities and configuration

### B1. Startup, ownership, and lifecycle

The macOS service is a **per-user LaunchAgent**, not a root daemon. The installed plist uses label `com.symless.synergy3`, executable `Contents/MacOS/synergy-service`, argument `-d`, and the app's MacOS directory as working directory. `KeepAlive.SuccessfulExit=false` requests relaunch after an unsuccessful exit. Exit 0 intentionally stops the service; exit 1 indicates failure; exit 2 requests a restart. The passive launchctl observation showed PID **1229**, one run, and no previous exit.

Evidence: read-only `~/Library/LaunchAgents/com.symless.synergy3.plist`; `P.launchctl`; constants `S:L161487–161498`; `MacInstaller.createAgentFile`, `S:L174509–174538`.

Startup sequence is: install error/exit handlers; load environment configuration; initialize log and Sentry support; check Node version (skipped in production); ensure directories; write `versions.json`; select CLI mode. `-d` enters daemon mode, `-r` resets, `-u` uninstalls, and the default path installs. Merely launching this executable without `-d` would mutate installation state. None of these paths was run during research.

Evidence: `App.start/startSafe`, `S:L176943–177019`; `RuntimeController`, `S:L176876–176942`; `NodeVersion.assertNodeVersion`, `S:L146745–146759`.

Daemon startup first claims a single-instance socket, then watches `/Applications`, creates the config/UI controllers, starts and waits for the tray, creates the core process controller, loads databases, creates peer sync and REST servers, starts permission observation, enables discovery, and starts the REST listener. The single-instance endpoint is `$TMPDIR/synergy-service.sock` on macOS; a connecting second instance writes `connection-attempt` and exits successfully. It is a lock/notification socket, not the UI settings API.

Evidence: `Daemon.run`, `S:L176559–176592`; `SingleInstance`, `S:L175848–175913`; `P.service_sockets` confirms the lock socket. `Tray.startReadyWatch` allows 10 seconds, `S:L175945–175964`.

### B2. Core launch, restart, environment, and temporary path

For macOS and Unix, the generated core command is exactly the executable plus either **`server --settings <path>`** or **`client --settings <path>`**. The service does not append its older stored custom-argument array here. The child uses Node `spawn` defaults: inherited process environment and working directory, with piped standard streams. macOS adds `QT_PLUGIN_PATH` when `../Frameworks/QtPlugins` exists.

Evidence: `UnixCoreProcess.serverArgs/clientArgs/spawnChild`, `S:L170076–170119`; `spawnProcess`, `S:L163133–163140`; `MacCoreProcess` constructor, `S:L170511–170521`. `extraCoreArgs` is read from `SYNERGY_CORE_EXTRA_ARGS` at `S:L161736`, but has no use in the launch path. `local_override.args` is saved at `S:L174910` and is not appended by these getters.

**Temporary-path correction:** this bundle does not copy the core binary. Before a normal production launch, `MacCoreProcess.getBinaryPath` unlinks `$TMPDIR/synergy-core`, creates a symlink to the installed `synergy-core`, and returns that symlink path. The code does not explain the original reason for this indirection. Do not infer elevation or a performance gain from it.

Evidence: `S:L170523–170538`, calls to `unlinkSync` and `symlinkSync`; no `copyFile` in that method.

Live `ps` names the temporary launch path, while `lsof` shows the executable mapped from the installed app. A passive `lstat` found that the temporary path is now a **Unix socket**, not a regular copied executable or symlink. Existing logs record successful core IPC connections. **Inference:** the core opens its local IPC socket at the same pathname after execution, replacing the launch symlink. This explains why the current pathname's file type cannot establish how the executable was launched. The native core agent should verify its socket-creation implementation.

Evidence: `P.tmp_core_path_type=socket`; `P.core_ipc_recent` records `core ipc connected`; retained `passive-lsof-redacted.txt` shows installed core mapping and service/core Unix endpoints. The exact random temporary directory component is omitted.

The base controller also runs the installed core with `--version` during normal app startup and caches the result. It extracts the token after ` v` and before the optional parenthesized hash / `, protocol`, then writes `versions.json`. This research read that file and did not run the version command. The IPC hello uses the parsed core version rather than the app version.

Evidence: `CoreProcess` constructor and `resolveCoreVersion`, `S:L169307–169329,L169466–169489`; `Versions.writeVersionsFile`, `S:L163848–163869`. The output file has `{app,core,node,platform,arch}`.

Unix supervision rules are precise:

| Condition | Service behavior | Evidence |
|---|---|---|
| First stdout or stderr data | Emit running status and begin status-IPC connection | `setupProcessEvents`, L170110–170118 |
| Explicit stop | Disable automatic restart, send SIGTERM; after 300 ms send SIGKILL if no clean exit | `stop`, L169985–170015 |
| Unexpected exit, including exit 0 | Restart if `restartOnExit` is still true | L170120–170142 |
| Run duration below 5 seconds | Delay restart by 1 second | L169933–169934,L170128–170138 |
| Exit code 5 | Report a duplicate instance and stop retrying | L169935,L170123–170126 |
| Config update | Serialize restart with a mutex; stop then start | L170017–170029; daemon updated listener L176384–176411 |
| User pause | Persist `local_system.core_enabled=false`; stop core | `Daemon.pauseCore`, L176531–176539 |
| User resume | Check known conflicting core names, persist true, re-evaluate config | `Daemon.resumeCore`, L176540–176549 |

Conflict detection covers `synergy-core`, `synergys`, `synergyc`, `deskflow-core`, `barriers`, `barrierc`, `input-leaps`, and `input-leapc`. Resume can send the UI `{name,pid}[]` as `coreAlreadyRunning`. The explicit reclaim route accepts positive integer PIDs, sends SIGTERM, waits up to five 100 ms polls, then SIGKILL and up to ten more polls. This is described only; it was not invoked.

Evidence: `CoreTypes`, `S:L167623–167631`; `UnixCoreProcess.findConflictingCores/stopProcesses`, `S:L170145–170178`; control schema `S:L175188–175190`; daemon `S:L176542–176557`.

### B3. Elevation and login-screen support

`local_developers.elevate_privileges` is the normal elevation mode. Only numeric **1** yields `[daemon] elevate=true`; other values yield false. In custom mode, the source is `local_override.elevate_privileges`. On macOS the normal core is spawned as the user. This service does not turn `system.admin_daemon` into a root process. `system.auto_restart`, `system.admin_daemon`, and `system.no_daemon` occur in the bundled defaults but are not used to control this launch/restart path.

Evidence: `SettingsReader.elevateMode`, `S:L171275–171276`; `CoreConfig.save*SettingsFile`, `S:L168361–168392`; overrides `S:L168202,L168281`; whole-bundle identifier search found the three legacy system keys only in default data, `S:L171378–171381`.

macOS login-screen access is a separate **LoginWindow LaunchAgent**. Enabling it uses `@vscode/sudo-prompt` to create a writable `/Library/Application Support/Synergy` directory for the current user, link its plist into `/Library/LaunchAgents/com.symless.synergy-agent.plist`, and link the installed core into `/Library/PrivilegedHelperTools/synergy-core`. The generated plist has label `com.symless.synergy-agent`, `LimitLoadToSessionType=LoginWindow`, the helper executable plus the current core arguments, unsuccessful-exit keepalive, and `/Library/Logs/Synergy/synergy-agent.log` for both output streams.

Evidence: `PreLogin.enablePreLoginAccess/hasPreLoginAccess`, `S:L170408–170482`; `PreLoginAgentConfig.generateConfig`, `S:L170360–170384`; `ControlRoutes` allow-on-login, `S:L175357–175375`.

The access check requires a writable data directory, plist, symlink, writable plist, and helper link. If access exists, each core launch saves or removes the login plist according to `local_system.run_on_login_screen`. Disabling removes the data-directory plist; the code does not remove the helper link or outer plist symlink here. For TLS clients, it copies `tls/trusted-servers` to the system data directory. Argument rewriting changes `--profile-dir` only if that argument already exists. The current normal command uses `--settings`, so it retains that settings path; this code does not copy the settings INI, layout config, or certificate into a separate system profile.

Evidence: `S:L170410–170428,L170350–170355,L170394–170405,L170484–170489,L170540–170554`; `ClientTLSConfig.copyTLSFingerprintFile`, `S:L167856–167873`. Login-window operation was not enabled or exercised.

Windows support, although present in this bundle, uses a separate privileged daemon through a named pipe. It sends the settings-file path and a start command; the daemon reads core mode and elevation from that INI. Linux inherits Unix supervision and has a separate pre-login config branch. Neither platform was exercised here.

Evidence: `WindowsCoreProcess.start`, `S:L170882–170889`; `DaemonIpcClient.sendConfigFile`, `S:L170682–170698`; `LinuxCoreProcess`, `S:L170241–170276`. Detailed daemon IPC is in E.

### B4. Database boundaries and effective settings

`db.json` is the shared database: `{version:{syncVersion:number,schemaVersion:number},data:{...}}`. Its current shared schema is **17**. `local.json` is node-local schema **7**. Shared changes are synchronized; local changes, IDs, IP maps, license storage, and paths are not sent as part of the shared database. Explicit peer commands can separately change all computers' serials.

Evidence: default templates `S:L171358–171437`; `SettingsReader.read`, `S:L171123–171140`; `DatabaseSyncer.broadcastDatabaseSend`, `S:L174142–174155`; local schema inspection in scratch `config-shapes.json`. The live files contain two computer records; their values are not reproduced.

Effective settings are a shallow spread of global `data`, then all local fields, plus runtime metadata. `security` merges shared `security` with `local_security`; `override` merges shared `override` with `local_override`. Only computers with `isRemoved != true` are used for core generation. The normal server is `mainID.id`; it is this node when it equals `local.myId`. Its peer address comes from `local_computers[mainID.id]`, not from the computer's display name.

Evidence: `SettingsReader`, `S:L171123–171140,L171188–171192,L171226–171230,L171244–171248,L171281–171295`; `CoreConfig.getNormalCoreMode`, `S:L168401–168408`.

| Setting source | Meaning / use | Evidence |
|---|---|---|
| `local.myId` | 32 random bytes encoded as 64 hex characters, created if absent | `Database.generateSelfID`, L172217–172221 |
| `local.local_system.path` | Layout file path; initialized to default `synergy.conf` when empty | L171197–171198,L172735–172740 |
| `local.local_security.path` | PEM path; initialized to `synergyCert.pem` when empty | L172741–172743 |
| `local.diagnostic.dir` | Initialized to log path; actual file logger uses environment `LOG_PATH` | L172744–172746,L162781–162782 |
| `local.diagnostic.level` | Service log level and normal core log-level input | L172720–172723,L171262–171267 |
| `data.developers.corePort` | Core port, default 24800 | L171269–171270,L171388 |
| `local.network.interface` | Adapter name; `Auto` has no matching adapter and leaves default bind selection | L171272–171273,L171083–171087; template L171433–171435 |
| `local.local_computers` | Node ID to locally learned peer address; self maps to 127.0.0.1 | L172095–172097,L172310–172320 |
| `local.local_system.core_enabled` | Persisted intent to run/pause the core; not synced | L172154–172159,L172527–172529 |
| `data.override.shouldOverride` | Select custom naming/config branch; it still requires self/main records and a valid license | L168078–168115,L168127–168146 |

A normal screen name is **all ASCII alphanumeric characters from the display name, lowercased, then `-` and the final eight characters of the full computer ID**. If sanitization produces an empty name, use just that ID suffix. Empty input names throw. Thus an OS-style display name can produce `macos-<redacted>`; the suffix is not derived from the MAC address or hostname. The separate `hostname` field is populated from the operating system and is not used by this naming function.

Evidence: `generateReadableComputerName`, `S:L167604–167614`; `getDefaultComputer`, `S:L174796–174801`; `Database.generateSelfID`, `S:L172217–172221`.

### B5. Exact `synergy.conf` mapping

The writer produces three sections, in order: `screens`, `links`, `options`. Each starts with `section: <name>` and ends with `end`; blank lines separate sections. Screen entries use a colon and tab indentation. Screen-option keys and scalar values have their first character lowercased. Links are written verbatim. An option whose value is an array becomes one repeated assignment per item. No aliases section is emitted, despite an unused aliases structure being built.

Evidence: `CoreConfigFile.toString/safeToString`, `S:L167554–167600`; `CoreConfig.createConfig`, `S:L168423–168431,L168463–168466,L168529–168531`.

| Input | Output under each generated screen | Evidence |
|---|---|---|
| Every `computer.modifier` entry | Same mapping, first character lowercased on output; default Alt/Meta/Shift/Super/Ctrl map to themselves | L168430–168433; defaults L174823–174828 |
| `fix.capslock` | `halfDuplexCapsLock` | L168437–168439 |
| `fix.scr_lock` | `halfDuplexScrollLock` | L168440–168442 |
| `fix.num_lock` | `halfDuplexNumLock` | L168443–168445 |
| `fix.xtest` | `xtestIsXineramaUnaware` | L168446–168448 |
| Positive nonempty `dead_corners[corner]` | Append `+<corner>` to initial `switchCorners = none `; assign `switchCornerSize` | L168451–168458 |
| Global `keyboard.altgr=true` | `altgr = alt` | L168460–168462 |

`switchCornerSize` is one scalar: when corners have different positive sizes, the last enumerated corner wins. `fix.sync_language`, `fix.invert_scroll`, and `fix.prevent_sleep` go to the INI instead. `hdk`, `scroll_sensitivity`, screen images, screen count, and nested monitor rectangles have no further direct mapping in this layout writer.

Evidence: complete loops in `S:L168423–168467`; `computersToScreens`, `S:L168738–168754`; INI mappings below.

| Input | `section: options` assignment | Condition |
|---|---|---|
| `local-network.interval` | `heartbeat` | Only when `local-network.keep_alive` is true |
| `mouse.relative` | `relativeMouseMoves` | Always |
| `performance.allow_copy` | `clipboardSharing` | Always |
| `performance.limit` | `clipboardSharingSize` | Always; default 4000; this service applies no unit conversion |
| `local_developers.keepForeground` | `win32KeepForeground` | Always, including on macOS |
| `developers.switchDelay` | `switchDelay` | Only if greater than zero |
| `developers.switchDoubleTap` | `switchDoubleTap` | Only if `shouldSwitchDoubleTap` is true |
| `keyboard.lock_to_screen` | `disableLockToScreen` | Boolean negation |
| Enabled hotkeys | Trigger/action assignments described below | Only `hotkey.enabled` gates the hotkey |

Evidence for every row: `CoreConfig.createConfig`, `S:L168468–168528`. `performance.sync`, `developers.dragAndDrop`, and `developers.showMeta` are stored but not used by this core config writer.

**Layout-to-links algorithm.** Layout uses each active computer's editor rectangle `{left,top,width,height}` in `misc`. Default rectangles are 200 × 110 editor units in a 622 × 440 editor; a new computer starts centered, then is positioned one unit outside the bounding rectangle of existing computers. If that bound is wider than tall, put the new computer below it; otherwise put it to the right. This is not a direct physical-display pixel map.

Evidence: `Layout` defaults, `S:L146718–146731`; `SettingsUtils.getPositionedComputer`, `S:L174844–174868`; `computersToScreens`, `S:L168738–168754`.

For link generation, each rectangle is expanded by **0.5 units** on every side, except that top and left are clamped to zero. The resulting width/height account for the amount clipped by that clamp. Every ordered pair is examined. Collision means strict positive intersection on both axes. For the up probe, move the first rectangle's top one unit upward without increasing height. For down, increase height by one. For left, move left one unit without increasing width. For right, increase width by one. A colliding probe produces a direction entry. These asymmetric probes matter for exact compatibility, especially with overlapping or boundary-clamped rectangles.

Evidence: `Converter.expand/detectCollision/getTouching/exportLinks`, `S:L167419–167550`; the call supplies `screenSpacing=1` at `S:L168530`.

For up/down links, take the intersection of the **unprobed expanded rectangles** on X. For left/right, use Y. For interval `[start,end]` and rectangle origin `o`, length `d`, calculate endpoint percentages as `round(100*(start-o)/d)` and `round(100*(end-o)/d)`. Use that calculation separately for the source and target. The output key is `direction(sourceStart,sourceEnd)` and value is `targetName(targetStart,targetEnd)`. There is no extra percentage clamp. Later entries overwrite an identical direction/range key.

Evidence: `getOverlapX/getOverlapY/getOverlapPercentage/detectLink`, `S:L167437–167488`, and direction assignments `S:L167490–167542`.

For equal-height adjacent synthetic computers A and B, A to the right of B produces `left(0,100) = B(0,100)` and the reverse produces `right(0,100) = A(0,100)`. Partial overlap produces different source/target percentages. The live config contains two reciprocal full-height links; names are intentionally omitted.

Evidence: same converter functions; passive read of `C/synergy.conf` confirms full-height ranges and reciprocal directions.

**Hotkeys.** `hotkeys` is an object keyed by hotkey ID. Each value has `{id,enabled,keystroke,actions}`. A keyboard stroke is `{type:"keystroke",control,alt,meta,shift,key}`; a mouse stroke uses `type:"mousebutton"` and `button`. All four modifiers are booleans. The emitted modifier order is Control, Alt, Meta, Shift, followed by the mapped key or mouse button, joined by `+`. A trigger is `keystroke(...)` or `mousebutton(...)`.

Evidence: schema `S:L161391–161407,L161461–161465`; `strokeToKeystroke`, `S:L168723–168736`.

| Action `type` | Other required fields | Emitted action |
|---|---|---|
| `MAP_TO_KEYSTROKE` | `keystroke`, `targets:string[]` | Stroke's own type with mapped stroke and optional colon-joined target names |
| `MAP_TO_KEYDOWN` | Same | `keyDown` or `mouseDown` |
| `MAP_TO_KEYUP` | Same | `keyUp` or `mouseUp` |
| `SWITCH_TO_COMPUTER` | `target:string` display name | `switchToScreen(generatedName)` |
| `SWITCH_IN_DIRECTION` | `target:up/down/left/right` | `switchInDirection(direction)` |
| `LOCK_CURSOR` | `lockType:toggle/on/off` | `lockCursorToScreen(value)` |
| `RESTART_CORE` | No additional field | `restartServer(restart)` |

Every action schema also has `enabled:boolean` and `trigger:0|1` (pressed/released). Released actions get a leading semicolon. The generator **does not inspect `action.enabled`**. Target references use display names via a reverse-name map; duplicate display names overwrite that map entry. An empty target list emits the one-argument form. Each action is a separate assignment for the same trigger.

Evidence: schemas `S:L161408–161459`; generation `S:L168483–168525`; `getActionTrigger`, `S:L168757–168767`.

The key map passes unknown names through. It maps `KeyA`–`KeyZ` to lowercase letters, `Digit0`–`Digit9` to digits, `Numpad0`–`Numpad9` to `KP_0`–`KP_9`, arrows to Down/Left/Right/Up, and standard function keys to themselves. Nonidentity mappings include Backquote→Grave, Backspace→BackSpace, BracketLeft/Right→BracketL/R, ContextMenu→Menu, Enter→Linefeed, Quote→DoubleQuote, IntlBackslash→Backslash, IntlRo→`\u30ED`, IntlYen→`\u00A5`, Convert→`\uEF23`, KanaMode→`\uEF7e`, Lang1–5→`\uEF31,\uEF34,\uFF26,\uFF25,\uEF2A`, and PrintScreen→`\uFD1D`. Browser action names become `WWW` plus their action suffix. Audio/Volume down/up become AudioDown/AudioUp; mute becomes AudioMute. LaunchApp1/2/Mail become AppUser1/AppUser2/AppMail. MediaPlayPause/Stop/TrackNext/TrackPrevious become AudioPlay/AudioStop/AudioNext/AudioPrev. Numpad Add/Comma/Decimal/Divide/Enter/Equal/Multiply/Subtract become KP_Add/KP_Separator/KP_Decimal/KP_Divide/KP_Enter/KP_Equal/KP_Multiply/KP_Subtract. Other explicitly listed ordinary keys retain their names.

Evidence: complete `KEY_MAP_TO_CORE` and fallback `keyMap`, `S:L168562–168722`. Backslash-u values above describe the literal strings in the bundle mapping.

### B6. Exact `synergy-core.conf` mapping

This is a Qt-style INI. Existing contents are parsed into ordered section/key maps, splitting at the first `=`. Recognized generated keys are overwritten; undefined generated values are deleted; other keys and sections survive. Comments and non-assignment lines do not survive. Changing mode does not remove a stale opposite-mode section. Booleans are lowercase, numbers use decimal strings. Strings matching `[A-Za-z0-9_.:\-/]+` are bare; other strings are double quoted, with backslash and quote escaping. Empty strings are quoted. The writer uses LF line endings and a blank line after each nonempty section.

Evidence: `CoreSettingsFile.parseCoreSettings/mergeCoreSettings/formatValue`, `S:L167666–167679,L167725–167785`.

| INI section/key | Normal source / value | Evidence |
|---|---|---|
| `core/coreMode` | Client 1, server 2 | L167680–167690 |
| `core/computerName` | Generated self name | L167691; CoreConfig L168164,L168241 |
| `core/port` | `developers.corePort` | L167692,L171269–171270 |
| `core/preventSleep` | Self `fix.prevent_sleep` | L167693,L168165,L168242 |
| `core/interface` | Selected interface's IPv4 address; delete key if no selected interface | L167694,L171272–171273 |
| `log/level` | Normalized diagnostic level or forced core log level | L167696–167697,L171262–171267 |
| `security/tlsEnabled` | License permits TLS AND shared `security.encryption` | L167699–167700,L168115 |
| `security/certificate` | Effective PEM path if TLS on; delete if off | L167701,L168371,L168388 |
| `security/checkPeerFingerprints` | **false when TLS on**; delete if off | L167702–167705 |
| `daemon/elevate` | Elevation mode equals numeric 1 | L167707–167709,L168368,L168385 |
| `server/externalConfig` | Whether a server layout path exists | L167711–167714 |
| `server/externalConfigFile` | `local_system.path` / CoreConfig config path | L167714,L168372 |
| `client/remoteHost` | `local_computers[mainID.id]` | L167718,L171281–171283 |
| `client/languageSync` | Self `fix.sync_language` | L167719,L168246 |
| `client/invertYScroll` | Self `fix.invert_scroll` | L167720,L168247 |

Log normalization accepts FATAL/ERROR/WARNING/INFO/DEBUG/VERBOSE. NOTE becomes INFO; other strings beginning with DEBUG become VERBOSE; absent or unrecognized values become INFO. Custom mode takes explicit local computerName/serverIP/elevation, compares shared `override.serverName` with local computerName to choose role, and forces preventSleep/runOnLoginScreen/languageSync/invertScroll false. Shared custom `configFile`, when truthy and override is enabled, is written verbatim as `synergy.conf`.

Evidence: `S:L167642–167665`; `CoreConfig.update*Override/getOverrideCoreMode/getConfigFileContents`, `S:L168191–168226,L168271–168307,L168394–168421`.

The live INI is client mode, port 24800, TLS enabled, fingerprint checking false, elevation true, language sync false, and inverted scroll false. Its remote host and generated name are redacted. This confirms the relevant static mapping, without calling a local endpoint.

Evidence: passive read of `C/synergy-core.conf`; no write or connection was made.

### B7. TLS certificate and trust-file lifecycle

When TLS is enabled, **both server and client paths persist a local certificate**. On first creation, node-forge generates an RSA key pair using effective `security.key_length` (**4096 bits** by default). It creates a self-issued certificate with serial field `01`, validity from the creation time to the same calendar date one year later, and subject/issuer **CN=symless.com, C=UK, L=IoM, O=Symless**. No extensions are set by this generator. `cert.sign(privateKey)` omits the digest argument; this bundled node-forge implementation defaults to **SHA-1 with RSA**. The separately computed certificate fingerprint uses **SHA-256**, so these are different algorithms serving different purposes.

Evidence: `ServerTLSConfig.generateAndSavePem`, `S:L167966–167992`; subject constants `S:L167898–167901`; node-forge `cert.sign`, `S:L63621–63633`; default key size `S:L171370–171374`.

The file contains the RSA private-key PEM followed by the certificate PEM. Default path: `~/Library/Preferences/Synergy/synergyCert.pem`, overridden by `local_security.path`. If a PEM already exists, the service recomputes its fingerprint and inspects the private key's actual RSA modulus length. A desired-length mismatch triggers regeneration. It does **not** check certificate expiration in this load path. A malformed or missing certificate/key throws instead of automatically repairing it.

Evidence: `S:L167908–167935,L167938–167964,L167986–167991`; path defaults `S:L161730–161731,L172741–172743`.

The fingerprint is SHA-256 over the certificate's DER bytes, obtained by decoding its base64 PEM body, then rendered as uppercase colon-separated hex byte pairs. The server returns that fingerprint to `SettingsWriter.updateTLSFingerprint`, which changes the shared `security.fingerprint` only if different, then broadcasts the updated shared database. The client obtains the shared fingerprint and overwrites its trusted-server file.

Evidence: `S:L167938–167955`; `CoreConfig.updateServerNormal/updateServerOverride`, `S:L168168–168172,L168203–168207`; daemon `S:L176523–176526`; `SettingsWriter.updateTLSFingerprint`, `S:L174915–174920`; client `S:L168309–168337`.

`tls/trusted-servers` is one text line: **`v2:sha256:<64 lowercase hex characters>\n`**. The converter removes colons, lowercases, requires exactly 64 hex characters, and rejects invalid input. The method overwrites the file rather than appending a set of historical servers. On macOS it sits beside the settings INI under `tls/`; optional pre-login handling copies it to `/Library/Application Support/Synergy/tls/trusted-servers`. No trusted-clients writer exists in this service code.

Evidence: `ClientTLSConfig`, `S:L167830–167849,L167856–167890`; current file is 75 bytes with one conforming record. Its value is not retained in this report.

Passive certificate inspection confirms RSA 4096, SHA1-with-RSA signing, the listed subject, and one-year validity. **Both the combined private-key PEM and trust file currently have mode 0644.** The generator supplies no explicit restrictive file mode. Generated INI currently sets `checkPeerFingerprints=false`, so writing the pinned fingerprint does not mean the active core enforces it.

Evidence: `stat` metadata; `openssl x509 -noout -text` was fed only the certificate block, with output restricted to public algorithm/subject/date fields. No private key or fingerprint was printed in that inspection. `S:L167991` and `S:L167702–167705` prove the mode omission and generated check flag.

### B8. Logging, diagnostics, telemetry, updates, and recovery

Unified logs default to `~/Library/Logs/Synergy/synergy.jsonl`. Each line is JSON with exactly the normal fields `{ts,source,level,msg}`; `ts` is an ISO UTC timestamp. Source names are service/core/tray/security. The passive current log sample had 736 parseable records and those four keys. Core text timestamps are parsed from `[YYYY-MM-DDTHH:mm:ss] LEVEL: message`; unparsable text gets current time and INFO. Core output is logged with force=true, independent of the service's chosen level, while the core's own INI limits what it emits.

Evidence: `formatJson`, `S:L162790–162796`; `LogSource`, `S:L162799–162807`; `CoreLogProcessor`, `S:L169860–169925`; `P.log`.

Every file log line synchronously checks whether rotation is needed and synchronously appends. Rotation occurs when current size is **greater than 10 MiB**, before the next append. Backups shift `.4→.5` through `.1→.2`, then current becomes `.1`; there are five backups plus the active file. Rotation errors are printed to console and append is still attempted. A write failure disables this endpoint for the rest of the process. LaunchAgent stdout/stderr files are separate; this rotation code does not rotate them.

Evidence: `FileEndpoint`, `S:L162677–162678,L162723–162771`; installed plist stdout/stderr paths. `SYNERGY_LOGGER` selects comma-separated file/console endpoints; production defaults to file, development to file,console (`S:L161723–161730,L162774–162787`).

The service launches the tray, translates its command lines, forwards core state and error dialogs to Electron, observes macOS permission state, and watches app removal. Every **2 seconds**, `MacPermissionWatcher` launches `synergy-security`; it does not reuse a persistent helper. Exit 0 means Authorized; other exits mean Denied. A helper launch failure preserves the last known state, or sets Error if no state is known. Loss of authorization stops the core and can open the UI with `--permissions`. Permission restoration re-evaluates core config. These paths were inspected but not triggered.

Evidence: `MacPermissionWatcher`, `S:L174606–174703`; `Daemon.initPermissionWatch`, `S:L176334–176382`; `UIHandler.run/getConfigAppArguments`, `S:L163750–163780`. Tray protocol is in D.

Sentry 7.x is initialized early. It removes the SDK's OnUncaughtException, OnUnhandledRejection, and Http integrations; installs its own error handlers; sets app release/environment, OS version, hostname, architecture, and memory context; and sets the license email as Sentry user when available. Reporting is then gated by effective `send-report` and the environment configuration's `sentryEnabled`. In this production build, the constructor computes `isProduction OR envFlag`, so `SYNERGY_SENTRY_ENABLED=false` does not itself disable reporting; the local send-report switch does. The SDK initializes enabled, immediately has telemetry disabled, then is updated from settings. The DSN value is deliberately omitted.

Evidence: `SentryUtility.init/updateSettings/setTelemetry`, `S:L163302–163357,L163404–163418`; `SettingsReader.sendErrorReport`, `S:L171194–171195`; production flag calculation `S:L161734`; sample-rate constant 1 at `S:L146712`.

Log messages at DEBUG or a more severe level become Sentry breadcrumbs even if suppressed by the configured file log level. Duplicate error tracking hashes error text; the local log/error sets also suppress repeated messages. Some core error messages receive limited name/path anonymization before error reporting, but their original messages are still written to the local log. There is no general secret scrubber here. This service logs parsed `.env` contents to console and has debug statements for serial changes and full sync messages. A replacement should retain diagnostic structure without retaining those sensitive payloads.

Evidence: `Logger.log`, `S:L162925–162974`; `SentryUtility.recordReported`, `S:L163434–163463`; `ErrorReporter`, `S:L169513–169569`; anonymizer `S:L169828–169856`; `.env` console logging `S:L146642–146660`; serial debug logging `S:L176159–176162,L176210–176212`; sync debug logging `S:L173803–173805,L173911–173912`.

Licensing in this service is a **local parse and feature-gating flow**. It validates serial text shape, decodes its record version, obtains edition/SKU, type, optional expiry/seat metadata, and creates a License object. It gates TLS and hotkeys by SKU, computer count by configured limits (defaults 3/5/15/unlimited), and rejects expired nonperpetual business licenses before core generation. Save operations clear a stored offline response when the serial changes and can propagate a serial update to peers. Grace-start time, offline activation response, and server-activation state are persisted from the UI. No live key was decoded for this research.

Evidence: `License`/parser, `S:L146245–146538`; limits `S:L146613–146618`; `CoreConfig.checkLicense`, `S:L168537–168559`; restrictions `S:L172228–172243`; save path `S:L172340–172379`; control routes `S:L175237–175292`.

There is **no service-owned HTTP license activation/check loop or update-download loop in the recovered application code**. Shared environment config defines API and version URL bases, but the service does not use them to call activation or download endpoints. This is a bounded static absence claim, not proof about Electron or the native binaries. The macOS service owns installation/uninstallation and watches `/Applications`: a rename event for `Synergy.app` invokes uninstall. It does not establish whether that rename was removal, replacement, or update.

Evidence: application-module inventory in A, source call search for fetch/axios/update functions, `RuntimeController` and `MacInstaller`, `S:L176876–176940,L174366–174582`; watcher `S:L174584–174603`; unused shared URL configuration `S:L146542–146608`.

Reset handling removes this node from shared data, removes local/global database files and UI settings when requested, stops sync, and stops the core. Each step has a 3-second timeout; errors are reported and later steps still run. Network reset and serial-update receivers then exit with code 2. This documents destructive product behavior; no reset was performed.

Evidence: `ResetComputer`, `S:L175083–175142`; `DatabaseSyncer.receiveReset/receiveUpdateSerial`, `S:L174039–174049`; daemon handlers `S:L176154–176162`.

## C. Peer discovery and database synchronization

### C1. Discovery transport and advertisement

`autoDiscover.discovery` controls a `discovery-channel` instance constructed with **`dht:false`**. It uses `dns-discovery` and `multicast-dns`, not a Synergy-specific Bonjour `_service._tcp` name. The default transport is IPv4 mDNS on **UDP 5353**, using the standard IPv4 mDNS multicast group `<redacted>`. Port **24804** is an advertised peer-port value passed to `join`, not the actual mDNS socket bind port. Live `lsof` confirms UDP `*:5353`, with no service listener on 24804 in the sample.

Evidence: `Discovery` constructor/start, `S:L174714–174718,L174741–174751`; `discovery-channel`, `S:L110489–110494`; `multicast-dns` defaults/bind, `S:L105708–105755`; `P.service_sockets`. IPv4 addresses other than 127.0.0.1 are redacted even when they are well-known protocol constants.

The discovery channel is the **serial string converted to bytes, SHA-1 hashed, and rendered as lowercase hex**. The DNS name is `<40 hex characters>.dns-discovery.local`. This exposes a deterministic channel tag, not a plaintext serial. Do not treat it as a computer ID. There are no display names, hostnames, TLS certificates, or database contents in this discovery advertisement. Service identity is learned later through signed peer-sync messages.

Evidence: `Discovery.serialKey/start`, `S:L174723–174749`; default hash and `join`, `S:L110507,L110553–110590,L110691–110692`; DNS domain `S:L106608`; TXT query `S:L106953–106960`.

For normal mDNS discovery, the library sends a DNS **TXT question** for that name. It stores its own advertisement as an address/port tuple and answers with TXT records containing a `token` and a base64 `peers` field. `peers` decodes to consecutive **6-byte tuples: four IPv4 octets, then a 16-bit big-endian port**. An all-zero IPv4 tuple means “use the sender address” in this library; the actual address literal is omitted here. Thus a peer advertises discovery port 24804 in the tuple, while the wrapper emits only the discovered host to the daemon. The daemon connects that host on configured **sync port 24802**, ignoring the advertised port.

Evidence: `dns-discovery._onquestion/_visit/_parsePeers/encodePeer/decodePeer`, `S:L106824–106839,L106948–106960,L106982–106993,L107147–107170`; `Discovery.onPeerFound`, `S:L174729–174739`; daemon discovery handler `S:L176324–176332`; `DatabaseSyncer.connectPeer`, `S:L174157–174166`.

The library can also process SRV answers for the same channel name. An SRV answer supplies target/port; a zero target uses sender address. TXT values use standard DNS TXT length-delimited byte strings containing `key=value`, split at the first equals sign. The token is base64 SHA-256 of a rotating random secret followed by requester address. It supports echo suppression and the library's optional unicast DNS announce/push scheme. The service supplies no external DNS servers; the library's default server list is empty, and DHT is disabled, so the service's configured discovery path is local multicast.

Evidence: `S:L106596,L106724–106780,L106802–106818,L107134–107135,L107172–107188`; `Discovery` passes only `{dht:false}`. The library's unused unicast TXT fields include `subscribe`, `announce`, and `unannounce` (`S:L106889–106918`); these are not a separate product API.

Discovery repeats its announce/lookup after **60–70 seconds** with jitter. The mDNS helper updates multicast interface membership every **5 seconds**; its multicast TTL defaults to 255. The DNS discovery helper rotates secrets every **5 minutes**. The wrapper keeps a set of seen host:port pairs for logging, but still emits `peerFound` for repeated discoveries, allowing another connection attempt. Discovery itself is gated on a serial; manual/configured peer sync is distinct from the discovery flag.

Evidence: `S:L110625–110634,L105743–105749,L106617,L174729–174739`; daemon settings/discovery handlers `S:L176210–176232,L176243–176258` and sync setup `S:L176131–176163`.

### C2. Sync transport, identity, and authentication

Peer synchronization is a plain **HTTP + WebSocket server on TCP 24802** by default. It binds all IPv4 interfaces when no named adapter is selected, or that adapter's address otherwise. Outgoing URLs are `ws://<peer>:<port>` and use a selected local address when set; WebSocket handshake timeout is **1 second**. The server attaches `ws` to a normal HTTP server, without TLS or an authentication check on the upgrade. JSON message signatures are checked after connection. The constructor's fallback 33726 is overridden by `Daemon` with `SYNC_PORT`, default 24802.

Evidence: `WSClient`, `S:L172997–173005`; `WSServer`, `S:L173129–173152`; `SyncSettings.ip/networkInterface`, `S:L173654–173683`; `DatabaseSyncer` constructor/activateNetwork, `S:L173766–173784,L174192–174200`; `Daemon.initDatabaseSyncer`, `S:L176131–176139`; live wildcard listener and established sync connection in `P`.

Peers are identified by their full random computer ID. Connections are grouped by IP, with at most **two** sockets per IP in a Multiplexer; its error text incorrectly refers to three. The multiplexer sends over the first open connection. An `addpeer` message carries all advertised local IPv4 addresses. The first accepted message also links its header computer ID to the observed source address. Local `local_computers` then remembers that ID/address mapping. The service filters its own addresses from outgoing peer attempts.

Evidence: `Multiplexer`, `S:L172800–172840,L172863–172879`; `Peers`, `S:L173592–173640`; `SyncSettings`, `S:L173669–173698`; `DatabaseSyncer.onConnected/addActivePeer`, `S:L173914–173920,L173984–173998`; `Database.updateComputerIP`, `S:L172310–172320`. See F for update limitations.

### C3. Exact sync message envelope

A WebSocket message is UTF-8 JSON with this shape; all identity, signature, and secret examples are placeholders:

| Field | Type and meaning |
|---|---|
| `header.id` | String; newly generated UUID for a locally created message |
| `header.type` | String command name |
| `header.dbversion` | Integer schema compatibility version, distinct from `body.version.syncVersion` |
| `header.signature` | Base64 SHA-256 digest defined below |
| `header.computerID` | Sender computer ID string |
| `header.receivers` | Optional string array; internally a set accumulated by setters |
| `body` | Any JSON value at the envelope validator; individual handlers interpret it |

Evidence: `MessageParser/Message.serialize/deserialize`, `S:L173464–173485,L173557–173582`.

The digest input is the direct concatenation, in this order, of UTF-8 **message ID**, **`JSON.stringify(body)`**, and the **shared serial string**. The digest is base64. There are no separators and no HMAC construction. The signature does **not** cover type, dbversion, computerID, or receivers. Validation recomputes it using the locally configured serial and compares strings with `==`. A cached signature survives serialization/cloning while header fields change. This explains header rewriting during forwarding. No encryption protects the sync body.

Evidence: `Message.sign/checkSignature/signature`, `S:L173531–173555`; clone/forward `S:L173878–173889,L173961–173964`; transport above. This describes observed authentication behavior; it is not a license-bypass procedure.

The daemon constructs sync initially with a fixed placeholder channel string, then starts it with the parsed local serial after `licenseChanged`. Discovery also receives that serial. The report does not reproduce either an actual serial or a derived live channel tag.

Evidence: `Daemon.initDatabaseSyncer`, `S:L176064–176065,L176131–176139`; license handler `S:L176210–176236`; `DatabaseSyncer.start`, `S:L174112–174116`.

### C4. Every sync command and the additional HTTP route

| `header.type` | Body | Sending / receiving behavior | Evidence |
|---|---|---|---|
| `addpeer` | `{id:string,ips:string[]}` | Sent after connection, after dbcheck. Update known peer addresses, or initiate connections to the supplied addresses. | L173914–173920,L174026–174037 |
| `ipbroadcast` | `string[]` addresses | Receive handler connects each address. No application send call found for this type. | L174006–174007,L174169–174172 |
| `dbcheck` | `{syncVersion:number,schemaVersion:number}` | Exchange version state and request/send newer shared database. Fresh setup can send both values as zero. | L174051–174094 |
| `senddb` | Complete shared `{version,data}` database | Accept newer syncVersion; reject older by broadcasting current data; do nothing when equal. | L173802–173818,L174142–174155 |
| `reset` | Empty string on normal sender | Reset local product state, stop sync/core, close/restart UI, exit 2. | L174039–174043,L174178–174180; daemon L176154–176157 |
| `updateSerial` | New serial string, value always redacted | Save new serial, stop/reinitialize state without deleting databases, exit 2. | L174045–174049,L174182–174184; daemon L176159–176162 |

Unknown types are logged as errors. A valid outer signature does not add a command-specific body schema; malformed bodies can reach handlers. Invalid signatures emit `peerRejected:{ip,reason:"invalidSignature"}`; incompatible schema emits the same event with reason `"incompatibleVersion"`. These become UI notifications. Invalid messages are ignored; this handler does not close the socket.

Evidence: `processMessage/checkMessage`, `S:L174000–174024,L173969–173982`; network listener returns at `S:L173941–173947`; UI forwarding `S:L176151–176153`.

The same 24802 listener also exposes unauthenticated **GET `/ping`**. Its success body is `{data:{synergy_version,hostname,system}}`. Response headers are `X-Synergy-Message-ID`, `X-Synergy-Computer-ID`, `X-Synergy-Signature`, and `X-Synergy-DB-Version`; the signature is calculated from the inner body object, not the `{data:...}` response wrapper. This endpoint can reveal host/system metadata even before a signed WebSocket message is accepted. It was not called.

Evidence: `DatabaseSyncer.setupExpress`, `S:L174096–174110`; common `sendRestSuccess`, `S:L173402–173410`.

### C5. Version, conflict, forwarding, and persistence rules

Shared updates do **whole-database replacement**, not field-level merge or JSON Patch between machines. A local broadcast-producing save sets syncVersion to current **epoch milliseconds** if that is greater than the old value; otherwise it increments the old value by one. Thus the rule is `max(Date.now(),old+1)`. Receive accepts strictly greater versions, rebroadcasts current data for strictly older versions, and ignores equal versions without comparing content. Clock skew can dominate another computer's edits; simultaneous equal versions can leave different content unresolved.

Evidence: `Database.saveGlobalFileAndEmit`, `S:L172595–172621`; `DatabaseSyncer.receiveDatabase/receiveDatabaseCheck/broadcastDatabaseSend`, `S:L173802–173818,L174051–174060,L174142–174155`. Conflict consequences are static inferences, not forced experiments.

On receiving a newer database, `SettingsWriter` applies two preservation rules before replacement:

1. If incoming data lacks this node's computer record altogether but the previous data had one, re-add the previous record and place it outside the incoming layout's outer bound. A present soft-removed record counts as present and is not automatically revived.
2. If the previous data had a valid main computer and incoming data has no valid main computer, restore the previous `mainID`. It **does not** keep the old primary when incoming data has another valid primary, despite broader wording in comments.

Every incoming `misc.connected` is reset to false because it is not authoritative. Incoming auto-discovery state is applied. If a preservation rule changed incoming data, save it with a new version and rebroadcast; otherwise persist the received version without a new broadcast-producing version. Rebuilding config/UI status still occurs. The shared file is persisted with synchronous `writeFileSync`, not an atomic temp-file rename.

Evidence: `SettingsWriter.replaceGlobalFromNetwork/forceRestoreData/forceAddSelf/forcePrimary/neutralizeConnected`, `S:L174922–174946,L175012–175073`; `Database.replaceGlobal/saveGlobalFileAndEmit`, `S:L172567–172576,L172595–172621`; positioning `S:L174844–174868`.

Schema acceptance for current schema 17 is **16 through 17 inclusive**: the current migration is backward compatible; no migration 18 is present. Outbound headers use the lower of known peer version and local schema; for an unknown peer, use local schema minus one (16). **This only changes the envelope header. The code does not transform the database body into an older schema for that send.** Local disk loading runs the bundled migrations up to 17; the network replacement path does not call that loader.

Evidence: migrations metadata `S:L171800–171820`; compatibility and outbound version selection `S:L173820–173868,L173878–173902`; load path `S:L171970–171980`; network replacement above.

The service holds up to **10,000 seen message IDs**. However, receive processing calls `processMessage` and `addActivePeer` **before** checking whether the ID is already seen. Deduplication therefore limits forwarding, not first-order handler execution. Forwarding also deserves care: it obtains IPs whose peer IDs **are in** `message.receivers`, rewrites the sender ID to this node, adds current peers to receivers, and sends to that computed list. The variable is called `filter`, but it is not passed as an exclusion list in that loop. This is the actual implementation, not the apparent intended “all unseen peers” rule.

Evidence: `S:L173787–173790,L173949–173965`; `Peers.ips`, `S:L173600–173608`; `Message` receiver setters, `S:L173487–173508`; `UniqueQueue`, `S:L173702–173757`.

### C6. Keepalive and reconnect behavior

When `local-network.keep_alive` is true, the shared interval controls both generated core `heartbeat` and peer-WebSocket keepalive. The database getter changes intervals below **1000 ms** to **5000 ms**; false returns zero. A sync keepalive tick sends a WebSocket ping and marks the socket not alive. Any ping, pong, or application message marks it alive. If the next tick still sees not alive, terminate the socket. A zero interval does not start the timer. Changing the interval restarts the sync network and connections.

Evidence: `Database.keepAliveInterval`, `S:L172172–172181`; `KeepAlive`, `S:L172950–172988`; `DatabaseSyncer.setKeepAliveInterval`, `S:L174202–174206`.

Stored peer addresses are attempted on sync activation. Discovery can trigger repeated attempts after its 60–70 second announcements. There is no separate application-level exponential reconnect scheduler for failed manual peers in this code. Core status IPC has its own independent one-second retry loop while the core is running; daemon IPC on Windows also retries after a second. Socket.IO has a separate UI heartbeat, documented below.

Evidence: `DatabaseSyncer.activateNetwork/connectPeer`, `S:L174157–174172,L174192–174200`; `Network.connect`, `S:L173312–173328`; `CoreProcess.connectStatusIpc`, `S:L169445–169464`; `DaemonIpcClient.connect`, `S:L170614–170627`.

## D. Local API used by Electron and tray

### D1. Transport, security, and response envelope

The UI API is **HTTP on `127.0.0.1:24803`**, default prefix **`/v1/`**, plus Socket.IO on the same HTTP server. It uses Express JSON parsing and Socket.IO `{serveClient:false}`. No bearer token, cookie/session validator, client secret, custom origin validator, or Socket.IO authentication middleware is installed in these application handlers. The server's loopback bind restricts its network reach; it does not authenticate another local process. `lsof` confirms the loopback listener. No endpoint was probed.

Evidence: `RestServer` constructor/initiate, `S:L175729–175780,L175791–175814`; prefix and port supplied by daemon `S:L176458–176472`; `P.service_sockets`.

A normal success is HTTP 200 with JSON **`{data:<value>}`**. When no data argument is given, JSON serialization omits it, yielding **`{}`**. Errors default to HTTP 500; several validation paths use 400. An error object contains `{type,name,message,stack,cause}`, recursively converting Error causes. **Error responses are double-encoded:** `createErrorJson` returns a JSON string containing `{serviceSentryId,error}`, then `res.json` serializes that string again. Callers need a second parse to get the object. Undefined serviceSentryId is omitted. The common error middleware can report after headers have already been sent.

Evidence: `RestUtils`, `S:L173402–173458`; `errorToObject`, `S:L163485–163500`; schema-specific errors in routes below.

The local API registers **40 normal method/path combinations**, plus five gated test routes. The settings response is the combined global/local settings object from B, with `ready`, `databaseVersions:{globalVersion,localVersion}`, `customConfig:{isServer}`, `serviceInfo:{permissionStatus,preLoginAccess,serviceVersion,coreVersion}`, `coreRunning`, and `screenStatus`. `screenStatus` maps computer IDs to `{serviceReachable:boolean,core:"connected"|"disconnected"|"unknown"}`. It reflects live sync/core state; it does not trust synced `misc.connected`.

Evidence: complete registration functions `S:L175208–175216,L175311–175377,L175382–175407,L175425–175448,L175451–175652`; `Database.getSettingsReaderWithServiceInfo/computeScreenStatus`, `S:L172654–172686`; `CoreStatusTracker`, `S:L170946–171039`.

### D2. All control routes

Paths below include the default prefix. “Empty” means no body field is read by that handler. Responses list the inner `data` value; absent means the `{}` success described above.

| Method and path | Request / action | Success `data` | Evidence in S |
|---|---|---|---|
| POST `/v1/controls/saveSerialKey` | `{serialKey:string,updateMethod:number}`. 0 propagates to all, 1 selects this computer; parse/save locally; restart running core. | For method 1, the prior stored serial is read into response data; otherwise usually absent. All values are sensitive. | L175237–175259 |
| POST `/v1/controls/saveLicenseGrace` | `{graceStartEpochSecs:integer>=0}`; merge into local license. | Absent | L175261–175270 |
| POST `/v1/controls/saveOfflineActivation` | `{offlineActivationResponse:string}`; merge local license. | Absent | L175272–175281 |
| POST `/v1/controls/saveServerActivation` | `{holdsServerActivation:boolean}`; merge local license. | Absent | L175283–175292 |
| POST `/v1/controls/connect-peer` | `{type:"ip",ipAddress:IPv4 string}` or `{type:"hostname",hostname:string}`. | Boolean: address accepted/connection already pending; **not** proof the peer connected. | L175170–175178,L175294–175308; Network L173305–173328 |
| POST `/v1/controls/pauseCore` | Empty; persist paused state and stop core. | Absent | L175218–175222 |
| POST `/v1/controls/resumeCore` | Empty; conflict check, persist enabled and reconfigure. | Absent; a conflict is a separate event. | L175223–175227 |
| POST `/v1/controls/reclaimCore` | `{pids:positive integer[]}`; stop those processes and resume. | Absent | L175228–175235 |
| POST `/v1/controls/restart` | Empty; restart only if persisted core_enabled is true. | Absent; restart is not awaited. | L175313–175320 |
| POST `/v1/controls/stopService` | Empty; exit 0 after response close. | Absent | L175322–175327 |
| POST `/v1/controls/resetThis` | Empty; reset this computer, then exit 2 after response closes. | Absent | L175329–175341 |
| POST `/v1/controls/resetAll` | Empty; broadcast reset and reset self, then exit 2. | Absent | L175343–175355 |
| POST `/v1/controls/allow-on-login` | `{allow:<truthy/falsy>}`; no boolean schema here. Enable permission/setup flow or remove login plist. | Enabling: `{requiredPermissions:false}` if already accessible, otherwise `{requiredPermissions:true,receivedPermission:boolean}`; refusal throws. Disabling: absent. | L175357–175375 |

### D3. All settings and custom-config routes

| Method and path | Request | Success `data` / semantics | Evidence in S |
|---|---|---|---|
| GET `/v1/settings` | None | Combined settings and runtime fields above; includes sensitive local data. | L175457–175463 |
| POST `/v1/settings/computers` | `{name:string}` | Active computers array. Adds this node using its existing self ID; refuses if self already exists. Picks it as main if no valid main. | L175428–175448 |
| DELETE `/v1/settings/computers/:id` | No body | Computers array. Soft-removes record (`isRemoved=true`, name cleared). | L175514–175516,L174948–174958 |
| PATCH `/v1/settings/computers/:id` | Partial computer object | Computers array. Shallow merge; `misc` receives a separate shallow merge. | L175518–175534,L172487–172507 |
| PATCH `/v1/settings/computers/batch` | Array of partial computer objects with IDs | Same dynamic route. Existing records are shallow-merged; unknown IDs skipped. This batch path does not separately deep-merge misc. | L175522–175524,L172550–172562 |
| POST `/v1/settings/send-report` | `{"send-report":value}` | Absent; stores local reporting flag. | L175465–175467 |
| PATCH `/v1/settings/local_developers` | Partial `{keepForeground:boolean,elevate_privileges:number}` | Local developer table; optional field type checks. | L175469–175472,L174891–174895 |
| PATCH `/v1/settings/diagnostic/level` | `{level:value}` | Local diagnostic table; the route has no enum schema. | L175474–175477 |
| PATCH `/v1/settings/network` | Partial local network object, notably `{interface:string}` | Network table; restarts sync selection and updates core config. | L175479–175482,L172534–172537 |
| PATCH `/v1/settings/tutorial` | `{finished:value}` | Tutorial table. | L175484–175487 |
| PATCH `/v1/settings/mainID/:id` | ID in path | `{id:...}` read before async save, whose underlying object is replaced; can return the old primary. Nonexistent ID gives 400. | L175489–175506,L174964–174973 |
| PATCH `/v1/settings/autoDiscover/:shouldAutoDiscover` | Exact string `true` enables; any other string disables. | `{discovery:boolean}` | L175508–175512 |
| POST `/v1/settings/hotkeys` | Strict `{id,keystroke,enabled}`; actions field excluded. | Created hotkey with `actions:[]`. Duplicate ID rejected. | L175536–175545,L172392–172395 |
| DELETE `/v1/settings/hotkeys/:id` | ID | Absent | L175547–175550,L172410–172413 |
| PATCH `/v1/settings/hotkeys/:id` | RFC 6902 JSON Patch operation array | Updated hotkey; actions excluded from patch target, then preserved. | L175551–175563 |
| POST `/v1/settings/hotkeys/:id/actions` | Action union from B5 | Whole updated hotkey. | L175565–175575 |
| DELETE `/v1/settings/hotkeys/:id/actions/:actionId` | Integer-coerced action index | Whole updated hotkey. | L175577–175587,L172431–172435 |
| POST `/v1/settings/hotkeys/:id/actions/swap` | `{a:integer-coercible,b:integer-coercible}` | Whole updated hotkey; swaps action array positions. | L175589–175599 |
| PATCH `/v1/settings/hotkeys/:id/actions/:actionId` | JSON Patch array applied to selected action | Whole updated hotkey; parses patched action union. | L175601–175619 |
| PATCH `/v1/settings/security` | Partial `{encryption:boolean,key_length:number,fingerprint:string}` | Effective merged security object. | L175621–175629,L174884–174889 |
| PATCH `/v1/settings/performance` | Partial `{limit:number,sync:boolean,allow_copy:boolean}` | Performance table. | L175631–175639,L174877–174882 |
| PATCH `/v1/settings/keyboard` | Partial shared keyboard object | Updated table; shallow merge. | L175641–175648 |
| PATCH `/v1/settings/mouse` | Partial shared mouse object | Updated table; shallow merge. | L175641–175649 |
| PATCH `/v1/settings/local-network` | Partial shared keepalive object | Updated table; shallow merge. | L175641–175650 |
| PATCH `/v1/settings/system` | Partial legacy shared system object | Updated table; does not imply those legacy launch flags are active. | L175641–175651 |
| PATCH `/v1/settings/developers` | Partial shared developer object | Updated table; shallow merge. | L175641–175652 |
| GET `/v1/custom/settings` | None | Generated layout config **string**, not the saved custom settings object. | L175396–175405 |
| POST `/v1/custom/settings` | `{shouldOverride,hasConfigData?,configFile?,serverName?,computerName?,serverIP?,elevate_privileges?,args?}` | Absent. `args`, if nonnull, must be an array. shouldOverride must be nonnull. hasConfigData gates other field writes. Shared and local fields split as in B. Save is not awaited by route. | L175388–175394,L174897–174913 |

`computers/batch` is a special value of the `:id` route, not an additional registration. It is included separately above to specify its distinct payload. Thus this table has one extra displayed row relative to the method/path registration count.

Evidence: dispatch branch `S:L175518–175535`.

Most legacy update routes do not strictly reject unknown properties; table updates use shallow merge. Security/performance/local developer checks only test the listed present fields' types. The hotkey create and patched hotkey schemas are strict; action discriminated unions have their own schemas. Several route handlers fire async saves or restarts without waiting, so HTTP 200 can precede resulting config, sync, or core state.

Evidence: `assertType`, `S:L175076–175080`; `Database.updateGlobalData`, `S:L172509–172512`; route bodies above; hotkey schemas `S:L161391–161465`.

Five additional GET routes exist only when development mode or `SYNERGY_TEST_ERRORS` is true: `/v1/testError`, `/v1/testException`, `/v1/testRejection`, `/v1/testDelayedExceptions`, `/v1/testDelayedRejections`. They report/display or trigger immediate/delayed test failures; some intentional exception/rejection branches do not send a success response. They were not called. No health endpoint or log-download endpoint is registered by this REST server.

Evidence: `TestRoutes`, `S:L175655–175714`; registration gate `S:L175769–175772`; bounded complete normal registration inventory above. `/ping` is on the separate sync listener, not this server.

### D4. All service-to-Electron Socket.IO events

There are no application-specific UI→service Socket.IO commands in this server. UI mutations use REST. The application handles Socket.IO `connection` and `disconnect` to retain active sockets; sends below are broadcast to all retained sockets. The default Engine.IO/Socket.IO path is `/socket.io/`, separate from `/v1/`. The bundled defaults are Engine.IO protocol **4**, Socket.IO parser protocol **5**, ping interval **25,000 ms**, ping timeout **20,000 ms**, max HTTP buffer **1,000,000 bytes**, and `allowEIO3=false`. These are library defaults, not active packet-capture observations.

Evidence: `RestServer.initiate/notifyElectron`, `S:L175791–175825`; Engine.IO defaults `S:L135078–135089`, protocol `S:L120354`; Socket.IO parser `S:L136972`; server created without a custom path or heartbeat override at `S:L175780`.

| Event | Payload sent by service | Trigger / meaning | Evidence in S |
|---|---|---|---|
| `event` | String `connected: <count>` | A UI socket connected. | L175799 |
| `event` | Boolean true | Sync peer list changed. | L176147–176149 |
| `event` | String `globalDataUpdated` | Shared settings changed. | L176264–176266 |
| `event` | String `screenStatus` | Core running/connection status changed; client should refresh settings. | L176423,L176432 |
| `newLogLines` | Boolean true | Log level changed. This is not a stream of log text. | L176259–176263 |
| `peerRejected` | `{ip:string,reason:"invalidSignature"|"incompatibleVersion"}` | Peer-sync validation failed. | L176151–176153; C4 |
| `closeElectron` | `{restart:boolean}` | False for ordinary UI close; true for reset/restart. | L176156,L176479–176480 |
| `displayErrorDialog` | **JSON string** of `{friendlyMessage,technicalMessage}` | Display dialog. | L176482–176484,L163706–163714 |
| `reportErrorSafe` | **JSON string** of error-report options | Contains serialized error/string plus handled/display/log/component/friendlyMessage and sentryReport when present. | L176485–176489,L163716–163728,L163502–163538 |
| `errorOccurred` | No meaningful argument | Logger saw ERROR/FATAL. Queue facility exists, but current caller does not request queue=true. | L175828–175832,L176473–176475,L162934–162939 |
| `coreAlreadyRunning` | `{name:string,pid:number}[]` | Conflicting sharing processes found. | L176551–176557 |
| `connectionState` | `Disconnected`, `Connecting`, `Connected`, or `Listening` | Forwarded core IPC state. | L176449; E1 |
| `connectedClients` | `string[]` core screen names | Forwarded core IPC list. | L176450; E1 |
| `connectionRefused` | String | Forwarded core IPC reason. | L176451; E1 |
| `retryIn` | Integer parsed from text | Retry delay in seconds. | L176452,L169421–169423 |
| `secureSocket` | String, not converted to boolean | Forwarded core IPC value. | L176453; E1 |
| `peerFingerprint` | Sensitive string; always `<redacted>` in research output | Forwarded core IPC value. | L176454; E1 |
| `unrecognisedClient` | String | Forwarded core IPC name. | L176455; E1 |
| `missingKeyboardLayouts` | String | Forwarded core IPC value. | L176456; E1 |

The event names are the complete statically observed `notifyElectron` call set, including the forwarded type list. Socket.IO's own protocol errors, connection acknowledgements, and heartbeats are library transport messages rather than additional product commands.

Evidence: whole-bundle `notifyElectron` call inventory; `Daemon.forwardCoreStatusToElectron`, `S:L176441–176457`.

### D5. Service–tray interface

The native tray does **not** use this REST API for its primary control interface. The service spawns `synergy-tray` and communicates over its stdin/stdout pipes. Service→tray lines are **`SHOW_START`**, **`SHOW_STOP`**, and **`EXIT`**, followed by OS newline. Tray→service recognized stdout values are **`READY`**, **`EXIT`**, **`CONFIG`**, **`START`**, **`STOP`**, **`RESTART`**, and **`EXITED`**. There is no JSON or authentication layer on these parent-owned pipes.

Evidence: `Tray` constructor/sendMessage/showStart/showStop/sendExitMessage/onStdoutData`, `S:L175925–175944,L175995–176028,L176042–176059`.

READY satisfies the 10-second startup wait. CONFIG opens UI. START/STOP resume/pause core. RESTART re-evaluates config. EXIT closes UI and stops core before requesting tray exit; tray exit completes service shutdown with code 0. Tray stderr becomes an error log. An unexpected stdout value throws an error. `onStdoutData` treats an entire Node data chunk, trimmed, as one command; it does not buffer/split multiple newline-delimited commands. The service's chunk handling is therefore a compatibility hazard even if the tray writes one line at a time.

Evidence: `Daemon.initTray`, `S:L176286–176323`; `Tray.startReadyWatch/onStderrData/onStdoutData/onParentExit`, `S:L175945–175964,L175992–176040`. The native tray's internal implementation is outside this report.

## E. Service-to-core and daemon IPC

### E1. Core status channel on macOS, Linux, and Windows

The service uses **`node:net` streams with newline-delimited text**, not the input-sharing TCP protocol, JSON, or a WebSocket. On Unix it connects to **`path.join(os.tmpdir(),"synergy-core")`**. On Windows the corresponding endpoint is **`\\.\pipe\synergy-core`**. It reads using Node readline. This is distinct from the service's single-instance socket and from the Windows daemon pipe.

Evidence: `CoreIpcClient` imports/path resolution, `S:L169106–169112,L169174–169189,L169294–169298`. Live logs in `P` confirm this core-status channel has connected, without this research opening a connection.

Handshake and stream rules:

| Direction | Line / shape | Service interpretation | Evidence in S |
|---|---|---|---|
| Service→core | `hello=<coreVersion>\n` | Exact parsed core version token, cached from the normal version probe | L169175,L169209–169211,L169471–169489 |
| Core→service, first line | `hello=<value>` | Accept any line with this prefix | L169195–169205,L169244–169245 |
| Core→service, first line | `versionMismatch=<coreVersion>` | Warn and accept, keeping connection | L169239–169242 |
| Core→service, first line | Any other text | Destroy connection as rejected handshake | L169203,L169247–169248 |
| Service→core | `stop\n` | A method exists to request stop | L169165–169172 |
| Core→service | `bye` | Destroy local socket and emit disconnected | L169250–169255 |
| Core→service | `command=value` | Split at **first equals**; remaining equals stay in value | L169257–169259 |
| Core→service | Bare `command` | Treat value as empty string | L169257–169259 |

The code comments say exact version matching is needed for the core to replay queued pre-connection state. The service accepts a mismatch but cannot establish from this code whether replay then occurs. The actual version matcher is the token regex, not a reconstruction from app version or build SHA. The native agent should establish the core's precise replay policy.

Evidence: `S:L169115–169118,L169231–169242,L169466–169489`; no native server code was used to turn comments into a stronger claim.

| Incoming command | Value and emitted application event | Evidence in S |
|---|---|---|
| `connectionState` | One of `Disconnected/Connecting/Connected/Listening`; unknown values warn and are discarded | L169261–169267 |
| `connectedClients` | Comma-separated screen names → string array; empty → `[]` | L169269–169271 |
| `connectionRefused` | Raw string reason | L169272–169274 |
| `retryIn` | `parseInt(value,10)` → numeric retry seconds; malformed text is not separately validated | L169275–169277,L169421–169423 |
| `secureSocket` | Raw string | L169278–169280 |
| `peerFingerprint` | Raw sensitive string, omitted from this report | L169281–169283 |
| `unrecognisedClient` | Raw string | L169284–169286 |
| `missingKeyboardLayouts` | Raw string | L169287–169289 |
| Any other command | Ignore, debug-log command name | L169290–169291 |

These values are forwarded through `CoreProcess` to the Socket.IO events listed in D. No escaping scheme or structured list encoding exists here; readable generated computer names avoid commas/equals. This service does not parse keyboard or mouse input events on this channel. The status tracker debounces changes for **100 ms**. On servers it compares generated names against the connected-client set; on clients it knows the self/main core link and returns unknown for other peers.

Evidence: forwarding `S:L169408–169440,L176445–176456`; `CoreStatusTracker`, `S:L170948–171029`; generated naming in B4.

Core IPC connection attempts have a **2-second connection timeout** and a mutex with a **3-second acquisition timeout**. The connection timeout is cleared as soon as the stream connects, before waiting for hello. There is no distinct handshake-response timer in `CoreIpcClient`. While core is running, failed/disconnected attempts retry after **1 second** with a guard against simultaneous retry loops. Explicit Unix process stop currently sends signals through `UnixCoreProcess.stop`; the core client's `sendStop` method is not called by that stop path.

Evidence: `S:L169128–169131,L169182–169189,L169445–169464`; Unix stop `S:L169985–170015`; whole-bundle `sendStop`/call-site inspection. No stop was sent.

### E2. Windows privileged-daemon control channel

The bundled Windows controller connects to **`\\.\pipe\synergy-daemon`** using `node:net` and newline-delimited text. The `CORE_IPC_PORT` environment property defaults to 24801, but this implementation does **not** use a TCP port for daemon IPC. Its path resolver rejects non-Windows platforms. This code is present in the macOS SEA but is selected only for Windows by the process factory.

Evidence: `DaemonIpcClient.createSocket/resolvePath`, `S:L170636–170645,L170779–170784`; `CoreProcessFactory`, `S:L170933–170945`; environment declaration `S:L161637–161643`.

| Service→daemon line | Expected daemon response | Use | Evidence in S |
|---|---|---|---|
| `hello=<appVersion>` | Prefix `hello=` or `versionMismatch=` | Initial handshake; version mismatch warns but succeeds | L170754–170771 |
| `noop` | `ok` | Liveness check before each operation | L170773–170777 |
| `configFile=<settingsPath>` | `ok` | Select Qt INI containing coreMode and daemon/elevate | L170682–170694 |
| `start` | `ok` | Launch/relaunch configured core | L170695–170698 |
| `stop` | `ok` | Stop core | L170700–170703 |
| `logLevel=<level>` | `ok` | Change daemon/core log level | L170705–170710 |
| `logPath` | `logPath=<path>` | Obtain file to tail | L170712–170719 |

All lines get `\n`; one request waits for one readline response. `logPath` splits response on all equals and requires exactly two pieces, so an equals in the path would be rejected. Normal acknowledgement is exact `ok`. There is no product token/credential exchange in this client; effective access control depends on the native named pipe, whose ACL is outside this report.

Evidence: `DaemonIpcClient.sendMessage/requestLogPath`, `S:L170712–170751`. The code neither constructs a TCP authentication protocol nor inspects pipe ACLs.

Daemon connections retry every second after failure; the connect timeout is 2 seconds. Sending is serialized by a **15-second mutex acquisition timeout**, and read response timeout is **10 seconds**. The response timer is created before acquiring the send mutex and is not cancelled on a successful response. On connection, the service restores log level and obtains/tails the log path with `pollFileIntervalMs=100`, unlimited poll failures, and 1-second retry delay. Windows `isRunning` is a local boolean after command acknowledgements. Restarts are guarded and throttled to one second; the restart method invokes start without first invoking stop.

Evidence: `S:L170586–170596,L170614–170627,L170643–170645,L170721–170751`; `WindowsCoreProcess`, `S:L170814–170839,L170840–170879,L170882–170918`. Windows behavior was not run on this Mac.

### E3. Other IPC must remain distinct

| Channel | Endpoint / transport | Payload |
|---|---|---|
| Service instance lock | `$TMPDIR/synergy-service.sock` on Mac | `connection-attempt`; no settings exchange |
| Core status | `$TMPDIR/synergy-core` / Windows core named pipe | Hello plus command/value lines |
| Windows daemon control | Windows daemon named pipe | Hello/noop/configFile/start/stop/logLevel/logPath |
| Tray control | Child stdin/stdout | Uppercase command lines from D5 |
| Electron UI | Loopback HTTP + Socket.IO 24803 | REST JSON and events from D |
| Peer configuration sync | Plain WebSocket + HTTP 24802 | Signed JSON envelope from C |
| Input/clipboard data plane | Core's configured port, default 24800 | Native core protocol; out of service scope |

Evidence: B–E channel-specific anchors; passive service/core sockets. This separation prevents a replacement from confusing the status socket with the core data protocol or the tray pipe with Electron's API.

## F. Performance evidence, fragile behavior, and opportunities

### F1. Passive runtime measurements

Two `ps -axo pid,ppid,pcpu,rss,etime,comm` / selected-PID samples taken during this research showed the following. macOS `ps` RSS is KiB; it includes resident shared pages. These samples are not a CPU trace, peak-memory measurement, or launch benchmark.

| Process | PID / parent | RSS samples | `%CPU` samples |
|---|---|---|---|
| Service | 1229 / launchd | 39,824 → 34,592 KiB (38.9 → 33.8 MiB) | 0.0 → 0.0 |
| Native tray | 1400 / service | 47,984 → 48,016 KiB (about 46.9 MiB) | 0.0 → 0.0 |
| Native core | 76958 / service | 74,928 → 50,080 KiB (73.2 → 48.9 MiB) | 0.3 → 2.0 |

The later sample, retained in `P`, is dated **2026-10-21 21:00:59 UTC**. At that point the service had run for 10h30m and the core for 3h42m. Summed RSS for these three processes is roughly **130–159 MiB** across the samples, but is not their unique physical-memory footprint. Electron was not included. No claim about measurable end-to-end latency or startup speed follows from these observations.

Evidence: initial command-output summary and `P.ps`; `P.launchctl` independently confirms service PID and run state. A separate user helper with “Synergy” in its name was not included in these totals.

### F2. Specific opportunities with supporting evidence

| Area | Observed behavior | What Openergy can improve | Evidence |
|---|---|---|---|
| Runtime size | 121.57 MiB executable; only 6.74 MiB is JS, and about 0.37 MiB is service/common/default data | Use a small native daemon and focused OS APIs | A1–A3; Mach-O segment sizes |
| Bundled duplication | Three Sentry cores, two iconv-lite copies, three MIME databases, three ws copies | Avoid broad duplicated runtime/dependency graphs | A3 / `dependency-sizes.json` |
| Permission checks | Launch a new helper every 2 seconds, without an in-flight guard in the interval callback | Persistent or event-driven permission observation; avoid overlapping helper launches | `MacPermissionWatcher`, S:L174624–174649 |
| Config edits | Every valid config update writes layout/INI and emits updated; Unix handler restarts core even if content is unchanged | Compare effective config and apply changes with minimal input interruption | S:L168339–168355,L176384–176411,L170017–170029 |
| Logging I/O | Per-line synchronous exists/stat/rotation/append; core messages forced through | Buffered structured logging, bounded rotation work outside the input path | S:L162723–162771,L169914–169921 |
| JSON storage | Synchronous whole-file writes; frequent structuredClone getters; no atomic rename | Transactional/atomic persistence and explicit ownership | S:L172119–172140,L172574–172583,L172595–172621 |
| Conflict resolution | Whole database chosen by clock-derived integer; equal versions ignored | Deterministic per-record conflicts and a defined tie rule | C5 |
| Sync confidentiality/auth | `ws://`; digest omits command and identity headers; upgrade unauthenticated | Authenticated encrypted peer channel, integrity over the complete envelope | C2–C3 |
| Trust enforcement | Trust file written, but generated core setting disables peer-fingerprint checks | Make encryption and identity verification consistent | B7 / S:L167702–167705 |
| Certificate storage | Private key and certificate share a 0644 PEM; default signing digest is SHA-1 | Restrict private-key storage and use a current signing design | B7; stat and node-forge sign path |
| Cert renewal | Existing PEM is checked for RSA length, not expiry | Defined renewal and trust-transition policy | S:L167908–167935 |
| Discovery overhead | Multicast membership refresh every 5 s, helper secret rotation every 5 min, discovery every 60–70 s; DHT code shipped but disabled | Smaller local discovery implementation and explicit interface change handling | C1 |
| UI control contract | No local authentication; error JSON double-encoded; many 200 responses precede actual work | Authenticated local endpoint and completion/status semantics | D1–D3 |
| Platform bloat | Windows/Linux installers and broad system information APIs remain in Mac payload | Platform-focused packaging | A module inventory / B3 |
| Legacy settings | Several stored flags and extra args have no launch effect | Remove unsupported knobs or make their effect explicit | B2–B6 |

These are evidence-based design opportunities, not measured promises that a particular replacement architecture already meets them.

### F3. Concrete correctness and error-handling gaps

1. **Duplicate sync commands execute before deduplication.** The check happens after `processMessage`. A repeated reset/updateSerial can reach its handler again; a repeated senddb still incurs parsing/checking. Deduplication should precede side effects in a replacement. Evidence: `S:L173949–173965`.
2. **Header integrity gap.** Signature verification covers ID/body/serial but not command type, sender ID, receiver list, or schema header. The configured local listener accepts an untrusted connection before message validation. Evidence: `S:L173547–173555,L173557–173568,L173969–173982`; C2.
3. **Forwarding direction is suspect.** It forwards to peers listed as receivers rather than taking their complement. This is a static control-flow observation; no multicast flood or replay was generated. Evidence: `S:L173957–173964,L173600–173608`.
4. **Ready-event ordering can strand a promise.** `DatabaseSyncer.start` awaits `activateNetwork`, which itself awaits emitting ready, and only then waits for `once("ready")`. A future ready event may be required to resolve that wait. The listener and socket can still be active. Evidence: `S:L174112–174116,L174192–174200`. This is an inference from await/event order, not a forced live failure.
5. **Core handshake can wait indefinitely after connection.** The 2-second socket-connect timer is cleared before the first response line; there is no separate handshake timer. The mutex only limits later acquisition attempts. Evidence: `S:L169182–169205,L169128–169131`.
6. **Network-listen errors need better handling.** REST and peer HTTP listen methods catch synchronous exceptions but do not install a matching asynchronous HTTP-server error rejection in those methods. An asynchronous bind error can leave the startup promise unresolved or reach the global error handler. Evidence: `S:L175805–175813,L173141–173151`; global handlers `S:L163589–163597`.
7. **Stale adapter cache.** `LocalHost._loadInterfaces` adds/updates entries but does not clear removed adapters. `isLocalAddress` can therefore retain old addresses. Evidence: `S:L171054–171087`.
8. **Incomplete peer-address refresh.** `addActivePeer` updates an existing peer's dbVersion but does not add a newly observed source IP. Address sets depend on a separate addpeer message; its behavior changes if a peer is already known. Evidence: `S:L173984–173998,L174026–174037`.
9. **Pipe/log chunk boundaries are not line boundaries.** Tray parsing treats one stdout chunk as one command. Core log parsing splits each chunk independently and keeps no remainder, so split lines can be logged with wrong timestamp/severity and coalesced tray commands can become “unexpected”. Evidence: `S:L175995–176028,L169874–169912`.
10. **Disabled hotkey actions still generate.** Only the parent hotkey's enabled flag is inspected. Duplicate display names also collide in the reverse-name map for hotkey targets. Evidence: `S:L168463–168466,L168483–168525`.
11. **Config writes can retain obsolete INI keys/sections.** Merge preserves unrecognized keys and the previous mode's section. That can preserve old behavior if a newer core reads those values. Evidence: `S:L167748–167772`.
12. **Unexpected exceptions do not force a clean restart.** Custom uncaughtException and unhandledRejection handlers report and continue; they do not exit or roll back state. This can leave partially completed operations. Evidence: `S:L163589–163597`.
13. **Single-instance lock races are possible.** Any connect error, not just a confirmed absent/stale server, causes macOS socket unlink and immediate server creation; lock resolves true without awaiting server listen completion. Evidence: `S:L175894–175909`.
14. **Stop timer targets a mutable child handle.** The 300 ms fallback accesses `this.childProcess` later. A race between exit-listener processing and timeout could throw or act on a changed child; current restart mutex reduces but does not prove elimination of all such races. Evidence: `S:L169991–170014,L170017–170029`. This is an untested race inference.
15. **Windows IPC timers persist after success.** Read-timeout creation occurs before lock acquisition and timers are not cancelled; a slow send queue consumes timeout budget before its own write. Evidence: `S:L170721–170751`.
16. **Installer start check is inverted.** After launchctl start, `runLaunchControlStartService` throws when `isServiceRunning()` returns true. Static evidence supports a logic error; the install/retry path was not run. Evidence: `S:L174556–174564`.
17. **App rename triggers uninstall without presence checking.** A replacement/update-style rename can reach the same watcher action as removal. Evidence: `S:L174597–174602`.
18. **Sentry's apparent 1000-error cap does not stop sends.** Above the threshold, `recordReported` returns an empty object; `send` interprets only isDuplicate and continues capture. Its set then stops growing, rather than enforcing a send limit. Evidence: `S:L163369–163397,L163448–163463`.
19. **Missing optional subsystems can leak into settings handlers.** The global discovery flag can skip `initDiscovery`, while the database's autoDiscoverChanged listener accesses the discovery getter unconditionally. Toggling that setting can throw when discovery was disabled at environment level. Evidence: `S:L176077–176081,L176243–176258,L176578–176581`.
20. **Private data can enter diagnostics.** `.env` dumps, full sync-message debug strings, serial-change logs, hostname/email Sentry context, and original core messages are not generally scrubbed. Only a few core error patterns are anonymized for reporting. Evidence: B8. No actual values are included here.

## Open questions / uncertainty

- **Exact dependency versions:** B entries are proven from this service; M and C entries are deliberately weaker. Many transitive versions are not present in the payload. A build lockfile or matching service SBOM is needed to resolve U entries and reconcile multiple copies. Optional external native-module requires include `bufferutil`, `utf-8-validate`, `osx-temperature-sensor`, and `macos-temperature-sensor`; their versions/load success were not established. Evidence: `S:L99670,L100312,L41249,L41266` and repeated ws copies.
- **SEA prefix:** byte ranges and source extraction are verified. The complete semantics of the five prefix bytes after the magic were not independently established from the exact Node 24.16 serializer. No extra resource bytes remain after source; no cached snapshot was needed or executed.
- **Why the temporary symlink:** source proves creation, and passive state proves that its pathname now hosts a socket. The motive is undocumented here. Native socket code must confirm whether it intentionally replaces that symlink; this report does not claim a core copy.
- **Loaded versus current disk image:** passive logs, version file, generated files, and socket state agree with major bundle behavior. Without attaching to the process or restarting it, this study does not prove every byte of the current on-disk SEA matches the image loaded 10 hours earlier.
- **Login-window behavior:** setup paths and plist contents are reconstructed, but login-screen input, FileVault/preboot support, permissions under LoginWindow, and certificate/settings access were not tested. This code alone does not prove FileVault unlock support.
- **Native transport policies:** TLS cipher versions, actual native peer-verification behavior, core IPC socket permissions, queued-state replay matching, and Windows pipe ACLs belong to native-core/security analysis. The generated false fingerprint-check setting is proven; stronger runtime TLS conclusions require native evidence.
- **Wire behavior:** discovery/sync packet layouts follow the shipped library code. No packet capture was taken, no peer was contacted, and no crafted message was sent. Timeouts, race conditions, forwarding defects, and conflict consequences are static inferences unless tied to passive observations above.
- **UI ownership of remote services:** no service-owned activation/update HTTP flow was found. Electron may own those operations; the UI agent should document it. The service's local licensing and sync flow is documented here without actual license values.
- **Performance:** no startup benchmark, CPU profile, memory allocation trace, forced crash, or reconnect test was allowed. RSS/CPU samples do not prove a memory leak, steady-state cost, or user-perceived latency. The 2-second helper schedule is proven code, while per-launch helper cost remains unmeasured.
- **Validation scope:** this change is one Markdown research report. No TypeScript or implementation package was changed; compilation/typechecking is therefore not applicable. No unit test, integration server, or browser flow was created/run. The user's passive-only rule prevents active end-to-end API/protocol validation; verification instead used raw section bounds, static REA parsing, source anchors, existing generated files, and existing process/socket/log state.
