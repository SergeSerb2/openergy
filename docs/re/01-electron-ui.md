# Synergy 3.7.0-beta: Electron UI reverse-engineering report

Static analysis date: 2026-10-21. Target: `/Applications/Synergy.app/Contents/Resources/app.asar`. Scope: Electron renderer, preload, and main process only. This is a clean-room behavior specification, not an implementation or a source-code reproduction.

## Evidence, scope, and confidence

- The archive is **67,474,000 bytes**. Its SHA-256 is `4c29acbe4e48c5120444f866b1f8ecc2f9a4200fc594602936eeb7713a43c713`. The root package identifies version `3.7.0-beta`, entry point `./build/index.cjs`, and Node `^24.0.0`. **Evidence:** `package.json` → `version`, `main`, `engines`; REA extraction manifest and direct byte count.
- All target analysis was static. No Synergy process or binary was started, stopped, signaled, attached to, or driven. No Synergy HTTP, discovery, licensing, update, or telemetry endpoint was exercised. The archive was copied to `/tmp/openergy-re/ui/app.asar`; extracted and recovered files are under that scratch directory. User configuration was read only to produce a redacted shape snapshot. No raw credentials or identities were copied to this report. Tool acquisition used the explicitly requested `npx` workflow.
- REA `6.7.0` was used for `doctor`, `extract-artifact`, and `analyze-javascript-application` on the three application bundles. Doctor found the artifact readable but reported missing native analysis engines and registration drift; no setup changes were made. The JavaScript graph recovered five browser-window constructions and 35 literal IPC channels, but renderer analysis exhausted its 2 GiB worker heap. Its graph is **partial**, not an exhaustive negative result. Manual bundle and inline-source-map inspection supplies the renderer findings below. **Evidence:** scratch `rea-doctor.json`, `rea-extract.json`, `rea-analysis.json` → `limitations`, `normalized_result.summary`.
- The renderer contains an inline source map with 888 sources, including 206 application/common source files. These are embedded evidence, not separate source files in the archive. Main/preload references to `index.cjs.map` do not have matching application map files in this archive. **Evidence:** `build/assets/index.js` → final `sourceMappingURL`, `sources`, `sourcesContent`; `build/index.cjs` and `build/preload/index.cjs` → final map comments; archive file inventory.

### How to read evidence references

Every abbreviated reference below names a file **inside `app.asar`** and a unique source region or symbol:

| Prefix | Actual archive file | Locator rule |
|---|---|---|
| `R:` | `build/assets/index.js` | The named source basename/region and symbol/string. Source regions normally map to `apps/gui-react/src/…`; settings types map to `libs/common/src/settings/…`; reusable controls map to `libs/gui-components/src/components/…`. Both region comments and inline-map source names are searchable. |
| `M:` | `build/index.cjs` | The named `// apps/gui-electron/src/…` or `// libs/gui-common/src/…` region and symbol/string. |
| `P:` | `build/preload/index.cjs` | The named `apps/gui-electron/src/preload/…` region and symbol. |
| `CSS:` | `build/assets/index.css` | Exact selector or declaration. |
| `CFG:` | Read-only configuration snapshot, outside the archive | Only a supplement to archive evidence; never proof of a factory default. |

Useful bundle anchors: renderer `GlobalSettingsData.ts` near line 56339; renderer application IPC near 88414–88788; renderer screens near 93417–106102; main IPC near 71893–72326; main service connection near 75884; main windows near 79464–80157. Line numbers refer to the original extracted bundles, not reformatted scratch copies.

**Default notation:** `D=?` means no factory default was found in the Electron artifact. `O=…` is a benign value observed in the existing configuration, and may reflect past edits or migrations. `U=…` is a UI fallback or new-item initial value. These must not be interchanged. Unless stated otherwise, persisted numeric/string fields have a TypeScript shape but no recovered database-level range validator. Consumer labels `S→C*` mean the UI sends the setting to the service and its label/type implies a core effect; actual service-to-core translation is outside this report and remains to be confirmed by the other research agent.

## A. Feature inventory and settings

### A1. Navigation and screen inventory

The app uses a React/Redux `HashRouter`. The main shell has an icon rail, a settings sidebar, and one content pane. Open utility routes remain available without a service connection. Other routes show a service-status page until the main process reports a connection. Protected content can instead show licensing status, reset status, permission checking, or first-computer setup. **Evidence:** `R:App.tsx#App`, `Protected.tsx#ProtectedRoute`, `MainContainer.tsx#MainContainer`, `layout/Navigation.tsx#Navigation`.

| Screen/route | Controls and behavior | Evidence |
|---|---|---|
| `/layout`, default `/` redirect | Computer canvas; Auto discover switch; per-card options; link to local details; round plus button whose popup opens Manage computers; tutorial/error/rejection overlays. | `R:ScreenLayoutPage.tsx#ScreenLayoutPage`; `MainContainer.tsx` → `/layout` |
| `/layout/computer-details` | Active interface IP addresses and hostname; missing-connection warning; explanation for manual setup; warning when a specific interface rather than Auto is selected. Refreshes adapter data every second. | `R:ComputerDetailsPage.tsx#componentDidMount`, `render` |
| `/keyboard-mouse` | Hotkey configuration/upgrade action; relative mouse moves; Alt Gr; lock to screen. | `R:KeyboardMousePage.tsx#render` |
| `/keyboard-mouse/hotkeys` | Empty-state illustration or sorted shortcut list; create; per-shortcut Edit, Disable/Enable, Remove. | `R:ListHotkeys.tsx#ListHotKeysComponent` |
| `/keyboard-mouse/hotkeys/:hotkeyid` | Shortcut capture control; ordered action list; add/edit action; move up/down; per-action Enable/Disable and Remove. | `R:HotkeyPage.tsx`; `HotkeyActions.tsx#HotkeyActions`, `ActionRow` |
| `/copy-paste` | Enable text/image clipboard sharing; size limit in MB; static warning that large images/clipboards can slow sharing. | `R:CopyPastePage.tsx#render` |
| `/manage-computers` | Detected computer cards, OS logo, primary keyboard/mouse marker, service-reachability styling, retry spinner, standard card menu, help and manual-add action. | `R:ManageComputer.tsx#DetectedComputer`, `render` |
| `/add-new-computer` | Multiple address rows; IPv4/hostname mode; automatic readiness check; add/delete/clear rows; connect all successful rows. | `R:ManualConnectPage.tsx#ManualConnectPageLegacy`; `AddressInput.tsx#render` |
| `/local-settings` | Login-screen access, automatic reconnect, beta updates, read-only config location, reset modal. Login control omitted for Flatpak; guest-account branch shows a warning. | `R:LocalSettingsPage.tsx#render`, `loginScreenPermissions` |
| `/local-networking` | Interface dropdown and missing-interface badge; keep-alive toggle; conditional interval field. | `R:LocalNetworkingPage.tsx#render` |
| `/security` | TLS toggle or upgrade action; conditional key-length dropdown and certificate-path readout. | `R:SecurityPage.tsx#render` |
| `/miscellaneous` | Windows system/UAC mode; foreground capture; switch delay; double-tap enable/time; Meta visibility; core port. | `R:MiscellaneousPage.tsx#render` |
| `/troubleshooting` | Log visibility/filter/level/path; reporting consent; critical/core error dialog controls; launch diagnostics. | `R:TroubleshootingPage.tsx#render` |
| `/custom-config` | Text configuration editor/import/revert; local core name; server-role switch; client server IP or server's own IP list; Windows system mode; command arguments; explicit Save/start/connect action. Other sidebar entries become disabled. | `R:CustomConfigPage.tsx#render`; `Navigation.tsx#NavMenuItem`; `CSS:.nav-bar-button-disabled` |
| `/auth` | Welcome; reporting opt-in; account-login or manual-key choice. Optional beta-user and invalid-stored-key banners come from query parameters. | `R:WelcomePage.tsx#WelcomePage` |
| `/auth/initial-login`, `/auth/update-login` | Account identifier/password fields, forgot-password link, inline errors. Update form offers this computer or all computers. | `R:AuthContainer.tsx`; `AuthForm.tsx#AccountAuthInput`, `Buttons` |
| `/auth/initial-serial`, `/auth/update-serial` | Monospace serial input; account/help links; offline-license guidance; initial Continue or update-this/update-all actions. | `R:AuthPage.tsx#SubHeader`; `AuthForm.tsx#SerialAuthInput`, `Buttons` |
| `/auth/update` | Choose account-based or manual license change; Cancel. | `R:UpdateLicensePage.tsx#render` |
| Setup state, without its own route | Name-computer illustration, required unique name field, Next; POST creates the local computer record, then navigate to layout. | `R:setup/Setup.tsx`; `NameComputerPage.tsx#onSubmit` |
| `/open/status` | Branded loading/install/service/version/reset/permission status, log toggle, context-specific help action. Also used as fallback when service is disconnected. | `R:status/Status.tsx#StatusMessage`; `Service.tsx`; `ActionLink.tsx` |
| `/open/log` | Separate window with filter and log-level controls; ANSI-rendered log lines; follows the tail only when the reader is at the bottom. | `R:Log.tsx#render`, `onNewLogLines` |
| `/open/about` | App/core/build information, registration/license area, Change license, legal and project credits. Existing UI renders the full serial, so native screenshots must redact it. | `R:AboutPage.tsx#render` |
| `/open/mac-perm` | Permission steps, looping muted local video for pre/post macOS 13, Open privacy settings. | `R:MacPermissionsPage.tsx#MacPermissionsPage` |
| `/open/diagnostic` | Live diagnostic text, privacy notice, Show file location, Dismiss, Reset settings, reinstall help. | `R:DiagnosticPage.tsx#DiagnosticPage` |
| Bad route / fatal renderer fallback | Unknown route throws; ErrorBoundary/fatal-start path replaces app content with a short failure message. | `R:RoutesUtils.ts#BadRoute`; `ErrorBoundary.tsx`; `App.tsx#startSafe` |

The icon rail has Pause/Resume, logs with an error badge, and Help. The sidebar groups User, Connections, and Advanced. An update button appears in its footer only when a newer version is reported. “Tray” in these React component names means the in-window rail; it is not the native `synergy-tray` process. **Evidence:** `R:layout/Navigation.tsx#Navigation`, `NavMenu`, `UpdateSection`; `libs/gui-components/src/components/Tray.tsx`.

### A2. Dialogs, menus, and conditional views

| Surface | Complete recovered choices or state variants | Evidence |
|---|---|---|
| Computer card menu | Make this primary, unless already primary; Computer preferences; Retry connection, unless self; Remove computer, including self. No confirmation is implemented in this renderer menu. | `R:editor/ScreenMenu.tsx#ScreenMenu` |
| Computer preferences | Four tabs: Name, Modifier keys, Dead corners, Miscellaneous. OS image, current name, Close, Save. Name and dead-corner inputs also save on blur; modifier/fix changes use Save. Save is disabled for unchanged active-tab data, name errors, or duplicate modifier outputs. | `R:ScreenSettings.tsx#render`, `onSave`, `getUpdateValue`, `saveButtonDisabled` |
| Unknown peer address | “Can't connect yet”; Try again, Connect manually, close. Retry requests settings, waits 700 ms, then rechecks the address. | `R:PeerNotReadyDialog.tsx`; `editor/Screen.tsx#onRetryFromDialog`; `ManageComputer.tsx#retryUnknownAddress` |
| Rejected incoming peer | Dismissible warning for serial/signature mismatch or incompatible version. Unknown rejection reasons throw rather than showing a generic message. | `R:editor/PeerRejectedOverlay.tsx#getMessage` |
| Shortcut recorder | “Record new hotkey”; capture button changes from “Record keystroke” to “Recording keystrokes”; close overlay. | `R:ListHotkeys.tsx#NewHotkeyModal`; `HotkeyCaptureButton.tsx#render` |
| Action editor | Dispatch when; action type; conditional captured stroke, target-computer multiselect, single computer, direction, or lock behavior; Cancel/Save. | `R:EditHotkeyAction.tsx#render` |
| Reset modal | Warning that settings will be deleted; Reset all computers / Reset this computer; close. Local-settings flow relaunches on success. | `R:ResetModal.tsx`; `ResetSettings.tsx` |
| Diagnostic reset | Connected service: soft-reset dialog with same this/all choices. Disconnected: Hard reset with Reset. Separate Reset successful / Reset failed dialogs, Restart app, reinstall help. | `R:DiagnosticPage.tsx#DiagnosticModal`; `SoftResetDialog.tsx`; `HardResetDialog.tsx`; `ResetSuccessDialog.tsx`; `ResetFailedDialog.tsx` |
| Core process conflict | Fork (Deskflow/Barrier/Input Leap), legacy Synergy core, or another current core variants. Cancel / Yes. Yes sends process IDs to the service to reclaim; the GUI does not implement the actual stopping operation. | `R:CoreAlreadyRunningDialog.tsx#CoreAlreadyRunningDialog`; `IPCLegacy.ts#reclaimCore` |
| License status | Access expired/warning, maintenance expired/warning, invalid, remote grace/disabled, server activation contested, offline challenge/response. Details in A8. | `R:status/LicenseStatus.tsx#LicenseStatus` |
| Native application menu | Synergy 3: About, License, Exit. View: Reload, Toggle Developer Tools. Edit: Undo, Redo, Cut, Copy, Paste, Select All. Help: Show Log (`CmdOrCtrl+L`), Contact Support, Run Diagnostic Tool. | `M:windows/WindowManager.ts#createMenus` |
| Native context menu | Cut, Copy, Paste, Select All; development builds add Toggle Developer Tools. It is installed for main/about/log/diagnostic windows. | `M:windows/AppWindow.ts#setContextMenu`; each window's `create` |
| Native error dialog | Copy, Exit, Dismiss, Ignore, Ignore all; development adds Sentry. Copy can open a second support-ticket prompt. Ignore suppresses a matching error for this process session; Ignore all clears the queue and disables critical-error dialogs in UI settings. | `M:error/ErrorHandler.ts#getErrorDialogButtons`, `onErrorDialogClick`, `onCopyButtonClick` |
| Install-location dialogs | Copy-to-Applications Decline/Accept; already-running OK; existing app No (exit)/Yes; move failure Exit. Describe only: none were invoked. | `M:App.ts#moveToApplicationsFolder` |
| Linux Wayland dialog | How to switch to Xorg / Continue with Wayland; Don't remind me again persists suppression. This is a cross-platform code branch in the Mac artifact. | `M:utils/Wayland.ts#showWaylandDialog` |
| Optional Test menu | Root; license-page shortcuts; all main-page shortcuts; status/log/about/permission/diagnostic windows; Config file mode Enable/Disable; service error, error storm, GUI front/back errors, service exceptions/rejections. Hidden unless `SYNERGY_ENABLE_TEST_MENU`; production default false. | `M:utils/TestMenu.ts#getTestMenuTemplate`, `licensePages`, `mainPages`; `config/EnvConfig.ts#ENABLE_TEST_MENU` |

No native tray/menubar status-item implementation was recovered from Electron window/menu code. Its menu contents belong to the separate tray-binary investigation. Likewise, no dedicated certificate-trust/fingerprint comparison dialog was found in the recovered React sources; see A6 and uncertainty. **Evidence:** `M:windows/WindowManager.ts#createMenus`, window inventory; `R:SecurityPage.tsx#render`; recovered application-source searches for `fingerprint`, `trusted`, `certificate`.

### A3. Computer layout editor and preference records

The canvas stores one abstract rectangle per **computer**, not one pixel-accurate record per physical display. Initial rectangle size is 200×110; editor design size is 622×440 with padding 5, centered at left 211/top 165. Actual editor width is read from the content element. Persisted coordinates are canvas units. `screenCount`/`screens` exist as metadata but do not cause this editor to render separate cards per monitor. **Evidence:** `R:libs/common/src/Layout.ts#defaultScreenProperties`, `defaultScreenEditorProperties`; `editor/ScreenLayoutEditor.tsx#getScreenArray`; `editor/LayoutComputer.ts#serviceToLayoutFormat`.

Drag uses the pointer offset, clamps to the canvas, snaps borders and corners within 20 units, rejects overlapping snap candidates, and leaves a one-unit separation between adjacent stored rectangles. Resize works on edges/corners, minimum dimension 100, does not lock aspect ratio, attempts nearby-edge alignment, and shrinks away from intersections with a 1000-iteration guard. Coordinates/dimensions are rounded **up** before a save. Drag saves on mouse-up; resize saves on stop; rendering-only movement does not write every frame. A changed layout PATCH sends the full visible layout list to `settings/computers/batch`. **Evidence:** `R:editor/ScreenPositioner.ts#screenPositioner`; `Interactions.ts#horizontalTransforms`, `verticalTransforms`; `Resizer.ts#resizer`, `sizeClamp`; `LayoutComputer.ts#ceilScreenDimensions`, `createPatchList`; `ScreenLayoutEditor.tsx#handleScreenLayoutChange`; `Screen.tsx` → `onResizeStop`.

Visual links are **not** the core configuration's text links. Rendering shrinks rectangles by 3 units on each edge, then uses 7-unit strips to join nearby right/bottom neighbors across the visible gutter. Link spans are interval intersections: vertical overlap for left/right neighbors; horizontal overlap for above/below neighbors. The editor itself never emits strings such as `left(0,100)`. **Evidence:** `R:editor/Interactions.ts#remapScreensForLinker`, `getScreenSpacing`; `editor/Linker.ts#linker`, `currentRight`, `currentBottom`; batch payload in `ScreenLayoutEditor.tsx`.

For rebuilding, the conceptual normalized edge mapping is: if A and B share a vertical interval `[a,b]`, A's span is `100×(a−A.top)/A.height` through `100×(b−A.top)/A.height`, and B's span is independently normalized to B's height. Two equal-height, fully aligned neighbors imply 0–100 on both sides; a partial-height neighbor does not. This is a **geometric inference**, not a recovered service formatting rule. Exact contact tolerance, rounding, direction syntax, and conversion to `left(0,100)` must come from the service/core report. **Evidence:** `R:editor/Linker.ts#currentRight` overlap formula; absence of config-text generation in `createPatchList`.

| Persisted computer field (`db.json.data.computers[]`) | Type / allowed input | Default evidence | Meaning / consumer | Evidence |
|---|---|---|---|---|
| `id` | string, opaque identity | D=?; O redacted | Record key and role/address/status join key; S/UI. | `R:GlobalSettingsData.ts#Computer`; `LayoutComputer.ts#toScreenMap` |
| `name` | string; UI rejects empty and exact duplicate; no other name restriction found | D=? | Display name; hotkey targets use names; S→C*. | `R:NameValidator.ts#isComputerNameValid`; `ScreenSettings.tsx`; `HotkeyActions.tsx#computerNames` |
| `hostname` | string; not edited in preferences | D=? | Metadata; producer S, UI carries record. | `R:GlobalSettingsData.ts#Computer` |
| `isRemoved` | boolean | D=?; O=false | Soft-removal marker; layout excludes removed records. Manage list maps all returned records, so returned filtering matters. | `R:LayoutComputer.ts#validComputers`; `ManageComputer.tsx#render` |
| `dead_corners.top-left`, `top-right`, `bottom-left`, `bottom-right` | each integer or empty string; UI strips nondigits; no upper bound | D=?; O=empty each | Pixel-size areas that block leaving at each corner; S→C*. | `R:ScreenSettingTabs.tsx#DeadCorner`, `getNumber`; `ScreenSettings.tsx#setDeadCorners` |
| `scroll_sensitivity` | number; no control/range found | D=?; O=0.5 | Stored sensitivity; service/core behavior unverified. | `R:GlobalSettingsData.ts#Computer`; CFG shape |
| `fix.invert_scroll` | boolean | D=?; U=false when missing; O=false | Reverse scrolling while this computer is controlled; S→C*. | `R:ScreenSettingTabs.tsx#Miscellaneous` → `invert_scroll` |
| `fix.sync_language` | boolean | D=?; U=false; O=false | Use primary computer's keyboard layout; S→C*. | Same → `sync_language` |
| `fix.prevent_sleep` | boolean | D=?; U=false; O=false | Keep controlled computer awake; S→C*. | Same → `prevent_sleep` |
| `fix.capslock` | boolean | D=?; U=false; O=false | UI's half-duplex Caps Lock control; S→C*. | Same → `capslock` |
| `fix.num_lock` | boolean | D=?; U=false; O=false | UI's half-duplex Num Lock control; S→C*. | Same → `num_lock` |
| `fix.scr_lock` | boolean | D=?; U=false; O=false | UI's half-duplex Scroll Lock control; S→C*. | Same → `scr_lock` |
| `fix.xtest` | boolean | D=?; U=false; O=false | XTest workaround for Xinerama; S→C*. | Same → `xtest` |
| `hdk.caps`, `hdk.scr_lock`, `hdk.num_lock` | boolean each | D=?; O=false each | Separate legacy-looking fields in the type and stored records. No current React control writes them. Do not equate them with `fix.*` without service evidence. | `R:GlobalSettingsData.ts#HalfDuplexKey`; `ScreenSettingTabs.tsx#Miscellaneous`; CFG shape |
| `modifier.Shift`, `Ctrl`, `Alt`, `Super`, `Meta` | string target from same five names; Meta hidden unless `developers.showMeta`; outputs must be unique across all five | D=?; O=identity mapping | Remap modifiers on this computer when controlled; the primary has an explanatory warning. S→C*. | `R:ScreenSettingTabs.tsx#ModifierKeys`; `ScreenSettings.tsx#isUniqueModifier` |
| `misc.left`, `top` | number, canvas bounded on drag; upward-rounded on save | D=?; initial placement constants above | Position; UI consumes; S derives links*. | `R:ScreenPositioner.ts#screenPositioner`; `LayoutComputer.ts#ceilScreenDimensions` |
| `misc.width`, `height` | number; UI minimum 100 per dimension | New rectangle constants 200×110; database D=? | Edge proportions, not display resolution; UI/S→C*. | `R:Resizer.ts#sizeClamp`; `Layout.ts#defaultScreenProperties` |
| `misc.os` | `linux` / `windows` / `macos` | D=?; machine dependent | OS logo; S supplies/UI consumes. | `R:GlobalSettingsData.ts#OS`; `Screen.tsx#osImages` |
| `misc.connected` | boolean | D=?; O=false | Retained data field, explicitly **not authoritative** for present UI status. | `R:ManageComputer.tsx#initStates`; `CombinedSettingsData.ts#screenStatus` |
| `misc.customImage` | declared `null`; layout/view accepts an image string when present | D=?; O=null | Optional card icon data; no image-picker UI found. | `R:GlobalSettingsData.ts#ComputerMisc`; `Screen.tsx#targetIcon` |
| `misc.logo` | string | D=?; O=empty | Layout-carried metadata, no editor control. | `R:LayoutComputer.ts#layoutToServiceFormat` |
| `misc.screenCount` | number | D=?; O=1 | Metadata; not a multi-monitor card control. | `R:GlobalSettingsData.ts#ComputerMisc`; `ScreenLayoutEditor.tsx#getScreenArray` |
| `misc.screens` | declared nested object map; editor model declares string array | D=?; O={} | Type disagreement; pass-through layout metadata. Do not choose an array-only native format on this evidence. | `R:GlobalSettingsData.ts#ComputerMisc`; `LayoutComputer.ts#LayoutComputer`, `layoutToServiceFormat` |

### A4. Global behavior settings

These keys live under `db.json.data`. The “Local networking” page includes globally synced keep-alive values; the “Local settings” page edits globally stored `system.auto_restart`. Page copy is not a reliable guide to storage scope. **Evidence:** `R:GlobalSettingsData.ts#GlobalSettingsData`; `LocalSettingsPage.tsx#patchAutoRestartCore`; `LocalNetworkingPage.tsx#patchGlobalNetworkSettings`.

| Key | Type / allowed UI values | Default evidence | Effect / consumer | Evidence |
|---|---|---|---|---|
| `mainID.id` | string; selected computer ID, empty means none | D=? | Primary/server selection; S, UI, inferred C role. | `R:ScreenMenu.tsx#onMakePrimaryClick`; `CoreConfig.ts#getCoreModeStandard` |
| `autoDiscover.discovery` | boolean | D=?; O=true | Automatic peer discovery; service owns discovery; UI toggle. | `R:ScreenLayoutPage.tsx#onAutoDiscoverChange` |
| `keyboard.altgr` | boolean | D=?; O=true | PC Alt Gr behavior; S→C*. | `R:KeyboardMousePage.tsx#render`, `patchKeyboardSettings` |
| `keyboard.lock_to_screen` | boolean | D=?; O=true | Enable lock-to-screen behavior; label does not specify the actual lock key. S→C*. | Same → `scroll-lock-switch`, `Enable lock to screen` |
| `mouse.relative` | boolean | D=?; O=true | Relative mouse moves; S→C*. | Same → `relative-mouse-switch` |
| `performance.allow_copy` | boolean | D=?; O=true | Text/image clipboard transfer; S→C*. | `R:CopyPastePage.tsx#componentDidUpdate` |
| `performance.limit` | number, persisted in KB; UI integer MB field 1–1000 | D=?; O=4000 (display 4 MB) | UI divides by 1000 to display and saves `parseInt(MB)×1000`; S→C*. Decimal MB input is truncated on save. | `R:CopyPastePage.tsx#constructor`, `componentDidUpdate`, `sync-clipboard-input` |
| `performance.sync` | boolean; no UI control | D=?; O=false | Declared but no renderer behavior located; S/C semantics unknown. | `R:GlobalSettingsData.ts#PerformanceSettings`; CFG shape |
| `security.encryption` | boolean; license feature gate | D=?; O=true | TLS sharing; S→C*. Not the same as local GUI HTTP transport. | `R:SecurityPage.tsx#render`; `M:EnvConfig.ts#SERVICE_URL` |
| `security.key_length` | number; UI 2048 or 4096 | D=?; O=4096 | Certificate key size; service/certificate producer, core TLS consumer inferred. | `R:SecurityPage.tsx#keyLengths`; `GlobalSettingsData.ts#EncryptionLevel` |
| `security.fingerprint` | string, sensitive; never reproduced | D=? | Stored fingerprint; no React display/edit/control recovered. | `R:GlobalSettingsData.ts#GlobalSecurityData`; CFG shape |
| `security.path` | string in declared type; absent in observed global file | D=? | Stale/optional-looking path used by refresh handler; initial UI uses `local_security.path`. | `R:SecurityPage.tsx#constructor`, `updateState`; `GlobalSettingsData.ts#GlobalSecurityData` |
| `local-network.keep_alive` | boolean | D=?; O=true | Keep-alive control; S→C*, exact wire scope pending service report. | `R:LocalNetworkingPage.tsx#render`, `patchGlobalNetworkSettings` |
| `local-network.interval` | number; UI 0–65535 ms | D=?; O=5000 | Keep-alive interval, conditional field, saved on blur; S→C*. | Same → `keep-alive-interval` |
| `system.auto_restart` | boolean | D=?; O=true | Automatic connection restart; service manager consumer. | `R:LocalSettingsPage.tsx#patchAutoRestartCore` |
| `system.admin_daemon` | boolean; no current UI control | D=?; O=false | Declared/stored only here; exact service/core behavior unresolved. | `R:GlobalSettingsData.ts#SystemSettings`; CFG shape |
| `system.no_daemon` | boolean; no current UI control | D=?; O=false | Declared/stored only here; exact service/core behavior unresolved. | Same |
| `developers.switchDelay` | number; UI 0–30000 ms | D=?; O=0 | Edge-switch delay; saved on blur; S→C*. | `R:MiscellaneousPage.tsx#render`, `patchGlobalDeveloperSettings` |
| `developers.shouldSwitchDoubleTap` | boolean | D=?; O=false | Enables double-tap condition; S→C*. | Same → `Enable switch with double tap` |
| `developers.switchDoubleTap` | number; UI 0–30000 ms | D=?; O=0 | Maximum double-tap interval; conditional field; S→C*. | Same → `switchDoubleTapMillis` |
| `developers.dragAndDrop` | boolean; no current UI control | D=?; O=false | Stored feature flag; no UI/file-transfer flow found. Its actual support must be checked in service/core. | `R:GlobalSettingsData.ts#DeveloperSettings`; CFG shape |
| `developers.corePort` | number; UI 0–65535 | D=?; O=24800 | Core server/client port; field filters exponent/sign/decimal keys; S→C*. | `R:MiscellaneousPage.tsx#defaultCorePort` |
| `developers.showMeta` | boolean | D=?; O=false | Adds Meta to the modifier editor; UI consumer. Does not mean Command/Windows key: UI warns about that distinction. | Same → `Enable meta key`; `ScreenSettingTabs.tsx#availableModifiers` |
| `override.shouldOverride` | boolean | D=?; O=false | Switch between graph editor and custom text configuration; UI/S. | `R:Navigation.tsx#onCustomConfigSwitch`; `MainContainer.tsx` |
| `override.configFile` | string, **file contents**, not a path | D=?; O=empty | Shared text config; import uses FileReader; service supplies default generated text. S→C*. | `R:CustomConfigPage.tsx#setDefault`, `updateScreen`, `readFileData`, `onSave` |
| `override.serverName` | string; empty/`.` treated as unknown by role helper; save payload can send null | D=? | Core server name; compared with local override name for role. UI/S→C*. | `R:CoreConfig.ts#getCoreModeOverridden`; `CustomConfigPage.tsx#onSave` |
| `hotkeys` | string-keyed shortcut map | D=?; O={} | Enabled shortcuts with ordered actions; detailed in A7/F. S→C*. | `R:GlobalSettingsData.ts#HotKey`; `ListHotkeys.tsx` |

### A5. Node-local and GUI-only settings

Local records live in `local.json`, not in the synced global schema. `UISetting.json` is read/written by Electron main. Screen reachability and `coreRunning` are runtime response fields, not persisted configuration. **Evidence:** `R:LocalSettingsData.ts`; `CombinedSettingsData.ts`; `M:utils/GUISettings.ts#GuiSettings.load`, `save`.

| Key / file | Type / allowed input | Default evidence | Effect / consumer | Evidence |
|---|---|---|---|---|
| `local.json.version` | number | O=7; factory D=? | Local schema version; S migration ownership. | `R:LocalSettingsData.ts#version`; CFG shape |
| `myId` | string, sensitive | D=? | This node identity; role and local joins; S/UI. | `R:CoreConfig.ts#getCoreModeStandard`; `LocalSettingsData.ts` |
| `ips` | string→boolean map, address keys sensitive | D=?; O={} | Local stored data; no direct React setting found. | `R:LocalSettingsData.ts#ips` |
| `local_computers` | computer-ID→address string map | D=? | Local address lookup for retry/custom-mode default server address; S/UI. | `R:ManageComputer.tsx#connectToPeer`; `CustomConfigPage.tsx#setDefault` |
| `send-report` | boolean | D=?; U=false on welcome before ready; O=true | Consent enables Sentry after settings arrive; UI/main and service own their own reporting. | `R:WelcomePage.tsx`; `TroubleshootingPage.tsx#onSendReportChange`; `SentryUtils.ts#updateSettings` |
| `serial` | string, secret | D=? | Local license/peer signature input; flows only, no values in this report. | `R:LocalSettingsData.ts`; `M:IPCHandlers.ts#serialKey` |
| `license.graceStartEpochSecs` | number, epoch seconds; 0 means no active grace | D=?; O=0 | Local online-license grace bookkeeping; main produces, service persists. | `R:LocalSettingsData.ts#LocalLicenseData`; `M:LicenseCheck.ts#handleFailed` |
| `license.offlineActivationResponse` | string, sensitive | D=?; O=empty | Machine/license-bound offline response; main verifies, service persists. | Same; `M:LicenseCheck.ts#applyOfflineResponse` |
| `license.holdsServerActivation` | boolean | D=?; O=false | Cache of server-activation ownership; online server remains authoritative. | Same; `R:App.tsx#shouldActivateForRoleChange` |
| `diagnostic.level` | `FATAL/ERROR/WARNING/INFO/DEBUG/VERBOSE` in type; UI maps 0 Info→INFO, 1 Debug→DEBUG, 2 Debug 1→VERBOSE; fourth visible Debug 2 has no mapping | D=?; O=INFO; log U=INFO before ready | Core logging level; S→C*. Mapping inconsistency is a bug candidate. | `R:LocalSettingsData.ts#CoreLogLevel`; `LogLevels.ts#logLevelsByIndex`; `Log.tsx#render` |
| `diagnostic.dir` | string, read-only UI | D=? | Full shared log **file** path despite name `dir`; main tail reader/UI. | `M:LogHandler.ts#setFile`, `tailRead`; `R:TroubleshootingPage.tsx` |
| `local_security.path` | string, read-only UI | D=? | This node's certificate path; S produces/UI reads. | `R:SecurityPage.tsx#constructor`; `LocalSettingsData.ts#LocalSecurityData` |
| `local_system.path` | string, read-only UI | D=? | Config-location readout; S produces/UI reads. | `R:LocalSettingsPage.tsx#constructor`, `config-location-input` |
| `local_system.run_on_login_screen` | boolean | D=?; O=true | Access on login screen. Control uses service `allow-on-login` operation rather than a raw patch; permission result can revert the switch. | `R:LocalSettingsPage.tsx#componentDidUpdate`; `IPCLegacy.ts#setAllowOnLogin` |
| `local_system.core_enabled` | boolean | Declared default true; O=true | Persisted intent to run sharing; Pause/Resume is node-local. UI fallback also assumes running until refresh. S manager/UI. | `R:LocalSettingsData.ts#LocalDataSystem` source-map documentation; `Navigation.tsx#coreRunning` |
| `local_developers.keepForeground` | boolean | D=?; O=true | More reliable foreground input capture; S→C*. | `R:MiscellaneousPage.tsx#takeForegroundChecked` |
| `local_developers.elevate_privileges` | enum 0 AsNeeded, 1 Always, 2 Never; current Windows toggle exposes only 1/2 | D=?; O=1 | Run core as system for UAC; service process policy. | `R:LocalSettingsData.ts#CoreElevateMode`; `MiscellaneousPage.tsx#elevate-privileges-switch` |
| `local_override.computerName` | string, nonempty in custom-mode save validation | D=?; O=empty | Core screen name; generated fallback removes non-alphanumeric display-name characters, lowercases, appends last eight ID characters. | `R:CustomConfigPage.tsx#getComputerName`, `updateScreen`; `Config.ts#validate` |
| `local_override.serverIP` | string; custom mode validates IPv4 with optional port 1–65535 | D=?; O=empty | Explicit server address; shown only in client mode; S→C*. | `R:Config.ts#checkIPValid`; `CustomConfigPage.tsx#render` |
| `local_override.elevate_privileges` | same 0/1/2 enum; Windows UI exposes 1/2 | D=?; O=1 | Custom-config core privilege mode; S manager. | `R:CustomConfigPage.tsx#onSave`, Windows switch |
| `local_override.args` | string array | D=?; O=[] | Command arguments; editor joins/splits on literal spaces, without shell quoting support; S→C*. | `R:CustomConfigPage.tsx#updateScreen`, `onSave` |
| `network.interface` | string from adapter IDs, including `Auto` | D=?; UI falls back to active interface only when nullish | Local interface selection affects both service settings-sync and core input links, per recovered comment. | `R:LocalNetworkingPage.tsx#constructor`, `render`; `M:LocalHost.ts#load` |
| `tutorial.finished` | boolean | D=?; O=true | Dismiss final placement tutorial; local UI state plus service-persisted patch. | `R:TutorialOverlay.tsx#endTutorial` |
| `UISetting.json.showCriticalErrors` | boolean | **D=true** | Permit native error dialogs; Ignore all sets false. | `M:GuiSettings.load`; `ErrorHandler.ts#displayUniqueErrorDialog` |
| `UISetting.json.showCoreErrors` | boolean | **D=false** | Permit dialogs tagged `component: core`; does not filter all log lines. | `M:GuiSettings.load`; `ErrorHandler.ts#displayErrorDialog` |
| `UISetting.json.filterLevel` | `All/Core/Service` | **D=All** | Log display filter; Electron persists, renderer filters text prefixes. | `M:GuiSettings.load`; `R:LogLevels.ts#getLogPrefix` |
| `UISetting.json.suppressWaylandDialog` | boolean | **D=false** | Suppress experimental-Wayland notice; main only. | `M:GuiSettings.load`; `Wayland.ts#showWaylandDialog` |
| `UISetting.json.updateTrack` | `stable/beta` | **D=stable**; O=beta | Release-check channel; Receive beta updates maps boolean to enum. | `M:GuiSettings.load`; `R:LocalSettingsPage.tsx#componentDidUpdate` |
| Ephemeral log visibility | boolean, not stored in these schemas | U=false | Opens/closes log window; error badge cleared when opened. | `R:redux/global/slice.ts#initialState`; `M:LogHandler.ts#constructor` |

### A6. Security and trust UX

The only dedicated React security controls are TLS enable, key length, and a read-only path. TLS access is gated by `license.hasTLSAccess()`; unsupported licenses see “Upgrade for TLS”. Encryption-enabled state determines whether key/path rows are shown, even if the license gate replaces the toggle. On initial construction the path comes from `local_security.path`; on settings refresh it incorrectly comes from `security.path`, which was absent in the observed global file. **Evidence:** `R:SecurityPage.tsx#constructor`, `updateState`, `render`.

A stored `security.fingerprint` exists, but no dedicated fingerprint display, comparison, allow/deny trust prompt, trusted-server editor, or trust-accept IPC endpoint was found in the recovered UI code. The generic `displayErrorDialog` event can show text supplied by the service, but the recovered generic dialog returns no trust decision to it. This is a bounded negative finding: do not assume the product lacks trust management in its other binaries. The `tls/trusted-servers` contents were not read. **Evidence:** `R:GlobalSettingsData.ts#GlobalSecurityData`; `M:ServiceConnection.ts` → `displayErrorDialog`; `App.ts#parseDisplayErrorDialogMessage`; `ErrorHandler.ts#onErrorDialogClick`; application-source identifier searches.

### A7. Keyboard shortcuts and actions

A shortcut consists of an ID, enabled flag, input stroke, and ordered actions. Input capture uses DOM `KeyboardEvent.code`, not localized character text. Pure modifier presses are ignored; Control/Alt/Meta/Shift flags accompany a non-modifier code. Mouse capture maps left→`"1"`, right→`"2"`, middle→`"3"`, button index 3→`"4"`, index 4→`"5"`. Release submits the captured combination and ends capture. The UI prevents default keyboard/mouse/context-menu behavior only while recording. **Evidence:** `R:HotkeyCaptureButton.tsx#modifierKeys`, `recordKeys`, `recordMouse`, `sendOnChange`, `startRecording`, `endRecording`.

| Action identifier | Label / payload-specific fields | Initial/allowed values | Evidence |
|---|---|---|---|
| `MAP_TO_KEYSTROKE` | Press and release key; `keystroke`, `targets` | Captured stroke required; target names multiselect; empty selection shown as All. | `R:EditHotkeyAction.tsx#actionOptions`, `makeAction`, `isReadyToSubmit` |
| `MAP_TO_KEYDOWN` | Press key; same fields | Same. | Same |
| `MAP_TO_KEYUP` | Release key; same fields | Same. | Same |
| `SWITCH_TO_COMPUTER` | Switch to computer; `target` | Display-name string from current computers, not ID; first option initially selected. | Same; `HotkeyActions.tsx#computerNames` |
| `SWITCH_IN_DIRECTION` | Switch in direction; `target` | `left/right/up/down`; initial Left. | `R:EditHotkeyAction.tsx#directionOptions`, `makeAction` |
| `LOCK_CURSOR` | Lock cursor to current computer; `lockType` | `toggle/on/off`; initial Toggle. | Same → `lockOptions` |
| `RESTART_CORE` | Restart connection | No extra target field. | Same → `actionTypes`; `R:GlobalSettingsData.ts#HotKeyActionRestartCoreConnection` |

All actions carry `enabled:boolean` and `trigger:0|1` (Pressed/Released). New shortcut/action UI defaults to enabled; new action trigger defaults to Pressed; no action type initially selected. Shortcut list sorts IDs, not human-visible strokes. Action order is explicit and moved with a swap endpoint. Enable toggles use JSON Patch test+replace; edits use JSON Patch diffs. No default hotkeys are defined in this UI artifact; the observed map was empty. **Evidence:** `R:ListHotkeys.tsx#constructor`, `onHotkeyChange`, `enabledPatch`; `EditHotkeyAction.tsx#constructor`; `HotkeyActions.tsx#swapUp`, `swapDown`; `GlobalSettingsData.ts#Triggered`.

Feature access: the main keyboard page links to hotkeys only when the license grants it; otherwise it offers “Upgrade for Hotkeys”. UI route protection is not itself proof of service enforcement. **Evidence:** `R:KeyboardMousePage.tsx#render`; `MainContainer.tsx` routes.

The recovered license helpers deny both hotkey and TLS controls only for Lite (`S3L`). Main configuration defaults computer-count limits to Lite=3, Plus=5, Max=15, and Business=null (no UI count cap), with environment overrides. These are product-policy defaults, not database defaults or evidence of core enforcement. **Evidence:** `R:libs/common/src/License.ts#hasHotkeysAccess`, `hasTLSAccess`, `checkComputerLimit`; `M:libs/common/src/EnvConfig.ts#computerLimits`.

### A8. Onboarding, account/licensing, updates, logs, and diagnostics

**Onboarding:** welcome/credential choice → validate/activate/save license → wait for a settings/license round trip → name this computer if absent/removed/unnamed → layout tutorial. Tutorial steps are derived, not an arbitrary multi-page wizard: one computer asks the user to add another; multiple computers with no primary ask for primary selection; then unfinished tutorial asks to position/resize; closing that last step persists `tutorial.finished=true`. Its primary-choice instruction uses the older phrase “Use this keyboard and mouse”, while the actual menu says “Make this primary”. **Evidence:** `R:Protected.tsx`; `AuthPage.tsx` two effects; `redux/global/utils.ts#checkSelfComputerSetup`; `TutorialOverlay.tsx#getStep`, `Step1–Step3`; `ScreenMenu.tsx`.

**Account flow only:** login submits account identifier/password to main; main calls a remote login endpoint, parses returned licenses, and chooses the first parsed license. UI handles invalid credentials, no valid licenses, local form errors, and activation/save errors. There is no license picker in this flow. Initial manual entry trims and validates the key, activates/checks as appropriate, then saves it. Update flows can apply locally or across computers using `SerialKeyUpdate` (All=0, This=1, None=2). UI waits for a settings round trip before navigating away to avoid a race. Password storage is not implemented in the inspected flow; a returned serial is persisted through the service. **Evidence:** `M:utils/Auth.ts#authFetch`, `login`; `R:utils/Auth.ts#accountLogin`, `saveLoginSerialKey`, `validateAndSaveSerialKey`, `getUpdateType`; `AuthForm.tsx#handleSubmit`; `AuthPage.tsx`; `libs/common/src/types/ControlRoutes.ts#SerialKeyUpdate`.

**Runtime licensing:** personal licenses activate at entry and routine checks skip remote verification. Online business licenses check immediately after relevant role/key/computer-count changes and then every 24 hours. Picking this machine as primary can auto-take over the server activation; a former server demoted to client releases ownership through activation. The main process starts a 14-day local grace period on check failure. Offline activation is required only on an offline-license server when more than one active computer exists; UI displays a challenge and accepts a response, then resumes sharing on success. No challenge/response/key values are recorded here. **Evidence:** `R:App.tsx#LICENSE_CHECK_INTERVAL_MS`, `shouldActivateForRoleChange`, license effect; `M:LicenseCheck.ts#run`, `activate`, `handleFailed`, `runOffline`; `R:LicenseStatus.tsx#OfflineActivation`.

| License state / screen | User flow | Evidence |
|---|---|---|
| `NoSerialKey` | Redirect to welcome/auth. | `R:Protected.tsx` |
| `Invalid`, `AccessExpired` | Apply new license; no Continue. | `R:LicenseStatus.tsx#LicenseStatus` |
| `AccessWarning`, `MaintenanceExpired`, `MaintenanceWarning` | Apply new license or Continue; expiry messages are computed for business SKUs. | Same; `redux/global/utils.ts#getLicenseStateFromSerialKey` |
| `RemoteWarning` | Failure explanation and remaining grace days; Apply new license or Continue. | `R:LicenseStatus.tsx`; `slice.ts#setRemoteLicenseState` |
| `RemoteDisabled` | Explain disabled verification; Apply new license; text promises automatic recovery after reinstatement. Exact core-stop behavior is not established by this text. | Same; `App.tsx` check effect |
| `Deactivated` | License limit reached; team/single-primary copy; No dismisses while paused, Yes requests takeover and resumes on success. | `R:LicenseStatus.tsx#Deactivated` |
| `OfflineActivationRequired` | Read-only activation challenge, editable response, Cancel/Activate; empty/invalid response errors; missing hardware identity has a Close-only variant. | `R:LicenseStatus.tsx#OfflineActivation` |

**Updates:** this is a remote version check plus a download-page link, not a recovered in-app binary installer. Check runs when the shell mounts and when the persisted channel changes. Stable omits the channel query; beta adds `?track=beta`. Request includes app version, language, and an OS/kernel/architecture User-Agent. Clicking Update available invokes `getLatest`, which opens the external download page. No periodic update timer or updater module was found in these application call sites. **Evidence:** `R:LayoutContainer.tsx` effect; `M:Version.ts#fetchRemote`, `checkForUpdate`; `IPCHandlers.ts` → `getLatest`; `R:Navigation.tsx#UpdateSection`.

**Logs:** Electron initially reads and converts the full current log synchronously, then tails it, batching new lines every 100 ms. Renderer retains only the newest 1000 lines in its pending buffer. It updates displayed lines only when already near the bottom (20-unit threshold), preserving a user's current reading position. Filters are All/Core/Service by rendered line prefix. Log-level choices have the mapping defect noted in A5. No search, export, clear-log, or explicit pause button is implemented in this viewer. **Evidence:** `M:LogHandler.ts#tailRead`, `tailDestruct`; `R:Log.tsx#onNewLogLines`, `isScrollAtBottom`, `render`; `LogLevels.ts`.

**Diagnostics:** opening the window starts the separate diagnostics helper, streams stdout/stderr as text, recognizes a saved zip path from its output, and can reveal it in Finder. The UI warns that the report contains addresses, names, and file paths. Closing the diagnostic window stops that helper in the product's normal code, but no such flow was invoked for this research. Hard reset delegates to the service helper; soft reset uses this/all service controls. **Evidence:** `M:diagnostic/DiagnosticTool.ts#start`, `startProcess`, `initRevealEvent`, `initResetEvents`; `R:DiagnosticPage.tsx`; reset dialog components.

## B. Electron and service IPC contract

### B1. Transport stack, trust boundary, and response envelopes

The renderer has no direct Node service client. Preload exposes `window.ELECTRON` with plain `buildConfig`, `envConfig`, `globalConst`, a small `process` description, and an `ipcBridge` with `on`, `send`, `sendSync`, `invoke`. The bridge accepts arbitrary channel strings; no application-channel allowlist is present. Listener callbacks receive `(event,arg)` and `on` returns an unsubscribe closure. A rejected preload `invoke` is caught/reported without rethrowing, so callers can receive `undefined`. **Evidence:** `P:preload/index.ts#setupContextBridge`; `preload/IpcBridge.ts#_ipcBridge`.

Renderer→main uses Electron IPC. Main→local service uses Axios HTTP at **`http://127.0.0.1:24803/v1/`** in production. The service URL can be overridden in development. `serviceFetch` joins the supplied URI to that base; unspecified method becomes GET via Axios. `serviceControl` prefixes `controls/`. No Unix socket, node-ipc transport, or direct renderer Socket.IO transport is constructed in these UI paths, even though other modules may be listed as package dependencies. **Evidence:** `M:config/EnvConfig.ts#SERVICE_URL`; `IPCHandlers.ts#register`; `libs/gui-common/src/Fetch.ts#initFetch`, `fetchFromService`, `fetchData`; `R:IPCLegacy.ts#_get`, `_post`, `_patch`, `_delete`.

Main separately opens a Socket.IO connection to the base with the `/v1/` suffix removed: `http://127.0.0.1:24803/`. No custom namespace/path/transport or auth options are supplied, so bundled Socket.IO defaults apply, including `/socket.io`. It listens for service events and forwards notifications/refetched state to renderer windows. **Evidence:** `M:service/ServiceConnection.ts#constructor`, `connect`; bundled `socket.io-client` manager → `opts.path || "/socket.io"`.

**No local auth/token handshake is visible:** neither `fetchFromService` nor `ServiceConnection.connect` supplies a bearer token, cookie, signature header, or Socket.IO auth object. This establishes what the GUI sends, not whether the service independently limits bind address/origin/clients. `LoginResult` declares optional `token` and `email`, but the actual `login` handler returns result type and serial only. Peer-signature checking and remote account login below are separate boundaries. **Evidence:** `M:Fetch.ts#fetchData`, `fetchFromService`; `ServiceConnection.ts#connect`; `utils/Auth.ts#login`; `R:libs/gui-common/src/types/Auth.ts#LoginResult`.

Main returns the HTTP response body, not the full Axios response. Most settings call sites expect `{data:T}`; the renderer wraps failures as/recognizes `{error:SerializedError}`. A failed service HTTP response can contain `{error,serviceSentryId}`; main turns this into `ServiceBadResponseFetchError` and serializes it for IPC. Do not assume a `success` field is mandatory in every service response; that is not proven by these callers. **Evidence:** `M:Fetch.ts#fetchData`, `createErrorFromResponse`; `IPCHandlers.ts` → `serviceFetch`; `R:IPCLegacy.ts#InvokeResponse`, `invoke`; settings helpers reading `.data`.

### B2. Renderer→main request/reply channels

Direction is renderer→main with a Promise result unless marked synchronous. Optional fields are shown with `?`. All sensitive values below are types/placeholders, never captured data.

| Channel | Input shape | Result / action | Evidence |
|---|---|---|---|
| `serviceFetch` | `{uri:string,method?:string,data?:unknown,...AxiosOptions}` | Service body, or `{error}`. After success main refreshes its settings cache via `serviceFetchDone`. URI is a relative path. | `M:IPCHandlers.ts#register`; `R:IPCLegacy.ts` |
| `serviceControl` | `{endpoint:string,method:string,data?:unknown}` | Fetch `controls/${endpoint}`; service body or `{error}`. | Same |
| `ping` | `{address:string}` | `{pingStatus:string}` or `{error}`; trims address, performs peer HTTP ping from main. | `M:IPCHandlers.ts`; `Pinger.ts#ping` |
| `login` | `{email:string,password:string}` | `{resultType:"success",serialKey:string}` or `invalid` / `no license` result; failure `{error}`. | `M:utils/Auth.ts#login`; `IPCHandlers.ts` |
| `authFetch` | `{form:{email,password,...},...}` | Registered legacy path returning `{resultType,licenses?}`. Recovered source wrapper adds `rejectUnauthorized:false`, but the handler only consumes `form`; do not infer TLS behavior from that unused field. Current compiled renderer flow uses `login`. | `M:IPCHandlers.ts` → `authFetch`; `R:inline map IPCLegacy.ts#authFetch`; `AuthForm.tsx` |
| `versionCheck` | `{}` / no meaningful input | boolean update availability, or `{error}`. | `M:IPCHandlers.ts`; `Version.ts#checkForUpdate` |
| `licenseCheck` | `{}` | `ok`, `grace`, `disabled`, `deactivated`, or `offlineActivation` discriminated result; possible `{error}`. | `M:IPCHandlers.ts`; `LicenseCheck.ts#run` |
| `licenseActivate` | `{serialKey?:string,takeover?:boolean,autoTakeover?:boolean}` | `ok`, `failed`+message, `deactivated`+message, or `error`; optional key overrides current key. Both takeover flags default false. | `M:IPCHandlers.ts`; `LicenseCheck.ts#activate` |
| `applyOfflineActivation` | `{response:string}` | `ok` or `failed`+message; generic `{error}` on exception. | `M:IPCHandlers.ts`; `LicenseCheck.ts#applyOfflineResponse` |
| `getLatest` | `{}` | Opens external download page; no app-update payload returned. | `M:IPCHandlers.ts` → `getLatest` |
| `checkFile` | file-path string | boolean regular-file existence. Registered handler and source-map wrapper; not retained as a call in the compiled renderer application region. | `M:IPCHandlers.ts` → `checkFile`; `R:inline map IPCLegacy.ts#checkFile` |
| `relaunch` | none | Emits main-app relaunch request; no useful payload. | `M:IPCHandlers.ts`; `App.ts#relaunch` |
| `updateSerial` | none | Navigates main window to license-change page, closes About if open. | `M:IPCHandlers.ts`; `WindowManager.ts#showLicensePage` |
| `getSettings` **sync** | none | Cached combined settings or `{ready:false}`; no immediate service call in handler. | `M:IPCListeners.ts#getSettings`; `R:IPCLegacy.ts#getSettings` |
| `diagInfo` **sync** | none | `{displayLog:boolean,filterLog:LogFilter,showCriticalErrors:boolean,showCoreErrors:boolean}` | `M:IPCListeners.ts` → `diagInfo` |
| `updateTrack` **sync** | none | `stable` or `beta` from UI settings. | Same → `updateTrack` |
| `networkAdapter` **sync** | none | `{hostname,interfaces,active_interface}`; active interface includes `id`, `ips`. Interfaces map names to non-internal IPv4 address arrays, plus synthetic Auto. | `M:IPCListeners.ts`; `LocalHost.ts#load`; `R:ComputerDetailsPage.tsx` |
| `osRelease` **sync** | none | OS release string; permission-video selector uses Darwin major >21 for macOS 13+. | `M:IPCListeners.ts`; `R:MacPermissionsPage.tsx` |

### B3. Renderer→main one-way channels

| Channel | Argument / behavior | Evidence |
|---|---|---|
| `requestSettings` | none; if cache ready, calls `sendSettings` (which fetches again), broadcasts `updateSettings` | `M:IPCListeners.ts`; `App.ts#sendSettings` |
| `requestServiceConnectionStatus` | none; broadcasts `serviceConnectionStatusChanged:boolean` | `M:App.ts#connectToService` |
| `appendFrontendLog` | serializable console record; append to GUI frontend log | `R:utils/ConsoleCapture.ts`; `M:IPCListeners.ts`; `GuiLog.ts#writeFrontendLog` |
| `reportErrorSafe` | `{error?:SerializedError,sentryReport?:object,friendlyMessage?:string,handled?:boolean,display?:boolean,component?:string,...}` | Main reconstructs error and reports/displays it. | `R:IPCSenders.ts#reportErrorSafe`; `M:IPCListeners.ts`; `ErrorHandler.ts` |
| `displayErrorDialog` | `{friendlyMessage:string,technicalMessage?:string}` | Native generic error UI. | `R:IPCSenders.ts`; `M:IPCListeners.ts` |
| `openExternalUrl` | `{uri:string}` | Opens supplied URL through OS shell. | Same |
| `openDevTools` | none | Opens developer tools. | Same |
| `openHelp` | none | Opens external help page. | Same |
| `openUpgradePage` | none | Opens external purchase/upgrade page. | Same |
| `openPrivacyMenu` | none | Resets accessibility permission state through helper, then opens privacy settings. Not used during research. | `M:IPCListeners.ts`; `mac/Accessibility.ts#reset`, `openSettings` |
| `openDiagnosticTool` | none | Opens diagnostic window and starts helper in normal app flow. | `M:IPCListeners.ts`; `WindowManager.ts#showDiagnosticWindow` |
| `toggleLog` | none | Opens/closes log window. | `M:IPCListeners.ts`; `LogHandler.ts#toggle` |
| `setFilterLog` | `All/Core/Service` string | Persist `filterLevel`; does not set service log level. | `M:IPCListeners.ts`; `LogHandler.ts#filterLevel` |
| `setShowCriticalErrors` | boolean | Persist UI dialog setting. | `M:IPCListeners.ts` |
| `setShowCoreErrors` | boolean | Persist UI core-dialog setting. | Same |
| `setUpdateTrack` | `stable/beta` | Persist channel, emit UI settings changed. | Same; `GUISettings.ts#setUpdateTrack` |
| `revealDiagnosticFile` | none | Reveal saved report zip, else open Documents; handler exists while diagnostic window is open. | `M:DiagnosticTool.ts#initRevealEvent` |
| `resetService` | none | Run reset helper and stream output/completion; handler exists with diagnostic window. | `M:DiagnosticTool.ts#initResetEvents` |

### B4. Main→renderer notifications

| Channel | Payload / receiving window | Evidence |
|---|---|---|
| `connectedToService`, `disconnectedFromService` | no payload; broadcast existing windows | `M:App.ts#connectToService`; `R:IPCListeners.ts` |
| `serviceConnectionStatusChanged` | boolean; broadcast response to status request | Same |
| `serviceInstallerStatusChanged` | enum `0 Idle,1 InProgress,2 Complete,3 Failed`; main window | `M:App.ts#onReady`; `R:ServiceInstallStatus.ts`; `IPCListeners.ts` |
| `updateSettings` | full combined settings response; broadcast | `M:App.ts#sendSettings`; `R:App.tsx#onUpdateSettingsData` |
| `macPermissionStatusSet` | `Unknown/Authorized/Denied/Error`; main | `M:App.ts#setupEventListeners`; `R:IPCListeners.ts` |
| `errorOccurred` | no argument supplied by main; renderer callback type permits a number, which is not a reliable wire contract | `M:App.ts#connectToService`; `R:IPCListeners.ts#onErrorOccurred` |
| `showPeerRejected` | `{ip:string,reason:"invalidSignature"|"incompatibleVersion"}`; main | `M:App.ts`; `R:PeerRejectedOverlay.tsx#getMessage` |
| `showCoreAlreadyRunning` | `{name:string,pid:number}[]`; main | `M:App.ts`; `R:CoreAlreadyRunningDialog.tsx` |
| `logWindowOpened`, `logWindowClosed` | none; main | `M:WindowManager.ts#openLogWindow`; `R:App.tsx` |
| `newLogLines` | newline-delimited string; log window | `M:App.ts#setupEventListeners`; `LogHandler.ts#tailRead` |
| `uiSettingsChanged` | object containing persisted GUI settings; main window | `M:WindowManager.ts#setupEventListeners`; `R:LayoutContainer.tsx`, `TroubleshootingPage.tsx` |
| `diagnostic-data` | output string; diagnostics | `M:DiagnosticWindow.ts#sendData`; `R:DiagnosticPage.tsx` |
| `reset-finished` | boolean success; diagnostics | `M:DiagnosticWindow.ts#sendResetFinished`; `R:HardResetDialog.tsx` |
| `testError` | none; optional debug flow | `M:TestMenu.ts#testReactError`; `R:App.tsx` |
| `filterCore` | `LogFilter` string; **renderer listeners exist but no main sender was found** | `R:Log.tsx#filterCoreListener`; `TroubleshootingPage.tsx#setFilter`; main sender search |

Bundled Sentry also defines `sentry-electron.renderer-start` (no application payload), `.event` (serialized event JSON), `.scope` (serialized scope JSON), `.envelope` (SDK envelope), `.status` (renderer status string), and `.add-metric` (SDK metric object). These are SDK telemetry channels to main, not `synergy-service` commands. The SDK can select its own protocol-based transport (`sentry-ipc`) instead of classic IPC; presence of handlers alone does not prove each channel is active. **Evidence:** `M:node_modules/@sentry/electron/esm/common/ipc.js#IPCChannel`; `esm/main/ipc.js#configureClassic`, `configureIPC`.

### B5. Main→service HTTP endpoint inventory

All paths in this table are relative to `/v1/`. `{id}` and `{index}` are path parameters; records and values are types. These are the endpoints **visible from Electron**, not an assertion that the service has no other API.

| Method / endpoint | Payload | Expected response use / purpose | Evidence |
|---|---|---|---|
| GET `settings` | none | `{data:CombinedSettings}`; refresh main cache and license bookkeeping | `M:Settings.ts#update`; `LicenseCheck.ts#fetchLicenseData` |
| POST `settings/computers` | `{name:string}` | Create/setup local computer; body not needed beyond success | `R:IPCLegacy.ts#setupThisComputer` |
| PATCH `settings/computers/{id}` | full `Computer` object | Save preference changes | `R:ScreenSettings.tsx#onSave`; `IPCLegacy.ts#_patch` |
| DELETE `settings/computers/{id}` | none | Remove computer | `R:IPCLegacy.ts#removeScreen` |
| PATCH `settings/computers/batch` | array of `{id,misc:{os,height,width,left,top,logo,customImage,screenCount,screens}}` | Layout update; no status/name/address fields in this projection | `R:ScreenLayoutEditor.tsx#handleScreenLayoutChange`; `LayoutComputer.ts#layoutToServiceFormat` |
| PATCH `settings/mainID/{id}` | none | Choose primary | `R:ScreenMenu.tsx#onMakePrimaryClick` |
| PATCH `settings/autoDiscover/{true|false}` | none | Enable/disable discovery | `R:ScreenLayoutPage.tsx#onAutoDiscoverChange` |
| PATCH `settings/keyboard` | `{altgr?:boolean,lock_to_screen?:boolean}` | Settings response | `R:KeyboardMousePage.tsx#patchKeyboardSettings` |
| PATCH `settings/mouse` | `{relative:boolean}` | Settings response | Same → `patchMouseSettings` |
| PATCH `settings/performance` | `{allow_copy?:boolean,limit?:number}` | Clipboard controls | `R:CopyPastePage.tsx#patchPerformanceSettings` |
| PATCH `settings/security` | `{encryption?:boolean,key_length?:number}` | Security controls | `R:SecurityPage.tsx#patchSecuritySettings` |
| PATCH `settings/system` | `{auto_restart:boolean}` | Reconnect control | `R:LocalSettingsPage.tsx#patchAutoRestartCore` |
| PATCH `settings/local-network` | `{keep_alive?:boolean,interval?:number}` | Global keep-alive values | `R:LocalNetworkingPage.tsx#patchGlobalNetworkSettings` |
| PATCH `settings/network` | `{interface:string}` | Local network device | Same → `patchLocalNetworkSettings` |
| PATCH `settings/developers` | subset of `{switchDelay,shouldSwitchDoubleTap,switchDoubleTap,corePort,showMeta}` | Global advanced values | `R:MiscellaneousPage.tsx#patchGlobalDeveloperSettings` |
| PATCH `settings/local_developers` | `{keepForeground?:boolean,elevate_privileges?:0|1|2}` | Local capture/privilege values | Same → `patchLocalDeveloperSettings` |
| PATCH `settings/tutorial` | `{finished:true}` | Tutorial dismissal | `R:TutorialOverlay.tsx#endTutorial` |
| POST `settings/send-report` | `{"send-report":boolean}` | Reporting preference | `R:WelcomePage.tsx`; `TroubleshootingPage.tsx#onSendReportChange` |
| PATCH `settings/diagnostic/level` | `{level:CoreLogLevel}` | `{data:{level,dir}}` read back | `R:Log.tsx`; `TroubleshootingPage.tsx#onLogLevelChange` |
| POST `settings/hotkeys` | `{id:string,keystroke:HIDStroke,enabled:true}` | `{data:HotKey}` including service-added actions | `R:ListHotkeys.tsx#onHotkeyChange` |
| PATCH `settings/hotkeys/{id}` | JSON Patch array; enable test/replace or stroke diff | Updated shortcut | `R:ListHotkeys.tsx#onSwitchClick`; `HotkeyActions.tsx#hotkeyChange` |
| DELETE `settings/hotkeys/{id}` | none | Delete shortcut | `R:ListHotkeys.tsx#onRemoveClick` |
| POST `settings/hotkeys/{id}/actions` | `HotkeyAction` | Add action | `R:EditHotkeyAction.tsx#makeAction` |
| PATCH `settings/hotkeys/{id}/actions/{index}` | JSON Patch array | Edit/enable action | Same; `HotkeyActions.tsx#toggleEnabled` |
| DELETE `settings/hotkeys/{id}/actions/{index}` | none | Remove action | `R:HotkeyActions.tsx#removeItem` |
| POST `settings/hotkeys/{id}/actions/swap` | `{a:number,b:number}` | Reorder actions by index | `R:HotkeyActions.tsx#swapUp`, `swapDown` |
| GET `custom/settings` | none | `{data:string}` generated config text | `R:CustomConfigPage.tsx#setDefault` |
| POST `custom/settings` | toggle-only `{shouldOverride:boolean}`, or `{shouldOverride:true,hasConfigData:true,isServer:boolean,configFile:string,serverName:string|null,computerName:string,serverIP?:string,elevate_privileges?:number,args:string[]}` | Apply config mode/text and local role/arguments | `R:Navigation.tsx#onCustomConfigSwitch`; `CustomConfigPage.tsx#onSave` |
| POST `controls/connect-peer` | `{type:"ip",ipAddress:string,hostname?:string|null}` or `{type:"hostname",hostname:string,ipAddress?:string|null}` | Body's `.data`; truthy reply is not core-link success | `R:IPCLegacy.ts#connectPeer`; `types/computer.ts`; `ManageComputer.tsx#connectToPeer` |
| POST `controls/pauseCore` | none | Stop sharing intent on this node | `R:IPCLegacy.ts#pauseCore` |
| POST `controls/resumeCore` | none | Resume sharing intent | Same → `resumeCore` |
| POST `controls/reclaimCore` | `{pids:number[]}` | Reclaim conflicting core processes | Same → `reclaimCore` |
| POST `controls/allow-on-login` | `{allow:boolean}` | `{data:{requiredPermissions:boolean,receivedPermission?:boolean}}` | Same → `setAllowOnLogin`; `LocalSettingsPage.tsx` |
| POST `controls/resetThis`, `controls/resetAll` | none | Reset selected scope | `R:IPCLegacy.ts#resetThis`, `resetAll` |
| POST `controls/stopService` | none | Legacy restart-service recovery action | `R:IPCLegacy.ts#stopService`; `Service.tsx#WrongServiceVersion` |
| POST `controls/saveSerialKey` | `{serialKey:string,updateMethod:0|1|2}` | Save local/all license; renderer requests settings afterwards | `R:IPCLegacy.ts#saveSerialKey` |
| POST `controls/saveLicenseGrace` | `{graceStartEpochSecs:number}` | Main's license state persistence | `M:LicenseCheck.ts#postGrace` |
| POST `controls/saveOfflineActivation` | `{offlineActivationResponse:string}` | Persist validated offline response | Same → `postOfflineResponse` |
| POST `controls/saveServerActivation` | `{holdsServerActivation:boolean}` | Persist role-activation cache | Same → `postServerActivation` |
| GET debug-only `testError`, `testException`, `testDelayedExceptions`, `testRejection`, `testDelayedRejections` | none | Optional Test menu; not normal product operations | `M:TestMenu.ts#testServiceError`, `testServiceExceptions`, `testServiceRejections` |

### B6. Service→main Socket.IO events and state refresh

| Socket event | Payload / main handling | Evidence |
|---|---|---|
| `connect` | Socket lifecycle; emit internal connected, broadcast `connectedToService` | `M:ServiceConnection.ts#connect`; `App.ts#connectToService` |
| `disconnect` | Socket lifecycle; broadcast disconnected | Same |
| `event` | arbitrary service message; value itself mainly triggers a settings fetch, then broadcast | `M:ServiceConnection.ts`; `App.ts#onServiceMessage` |
| `errorOccurred` | no consumed payload; main marks error badge via renderer notification | Same |
| `peerRejected` | `{ip,reason}`; forwarded as `showPeerRejected` | Same |
| `closeElectron` | `{restart:boolean}`; main quits or relaunches | Same |
| `displayErrorDialog` | **JSON string** of `{friendlyMessage,technicalMessage}`; main parses, shows native dialog | `M:ServiceConnection.ts`; `App.ts#parseDisplayErrorDialogMessage` |
| `reportErrorSafe` | **JSON string** of error/report options; main parses, reconstructs error, reports | `M:ServiceConnection.ts`; `App.ts#onServiceReportErrorSafe` |
| `coreAlreadyRunning` | `{name,pid}[]` or nullish→[]; forwarded to main window | `M:ServiceConnection.ts`; `App.ts` |

The generic event path does `settings.update()` and then `sendSettings()`, which itself does another `settings.update()`. Thus one event can cause two full GETs. Main's `serviceFetchDone` also refreshes settings, and several renderer mutations explicitly request a further refresh. There is no recovered incremental settings-patch push or version-based deduplication at this UI boundary. **Evidence:** `M:App.ts#onServiceMessage`, `sendSettings`, `setupEventListeners`; `R:IPCLegacy.ts#saveSerialKey`; `ScreenSettings.tsx#onSave`.

### B7. Peer check and remote account/auth shapes

Manual-add **peer readiness** is not sent to the local service: Electron main itself performs an IPv4 HTTP GET to `http://{address}:24802/ping`, with a 5-second timeout. Expected body wraps `data` (object, or JSON string to parse). Headers are `x-synergy-message-id` and `x-synergy-signature`. Main compares the signature against base64 SHA-256 of the concatenation of message-ID text, `JSON.stringify(data)`, and the current serial string. This is a keyed digest construction in this code, **not HMAC**. No secret values are needed to specify its shape. Missing headers or malformed data are errors. A single Pinger instance cancels its previous timeout/request when a new ping starts. **Evidence:** `M:Pinger.ts#request`, `parsePingResponse`, `_setupTimeout`, `abortActivePing`; `Signatures.ts#makeSerialKeySignature`; `IPCHandlers.ts#pinger`.

Remote account login uses a form with `email`, `password`, and `getLicense:"true"` to `/auth/login` at the configured legacy API base; an unsuccessful response maps to invalid credentials. Successful response expects `data.license[]` entries containing `licenseKey`. The UI consumes a chosen serial, not a session-token handshake. Remote license check/activation use `/synergy/api/product/check` and `/synergy/api/product/activate` with `{machineSignature,hostnameSignature,serialKey,appVersion,osName,isServer}`; activation adds `takeover:boolean`. Identity signatures are hashes; all values remain redacted. These endpoints were read as strings only and were never called. **Evidence:** `M:utils/Auth.ts#authFetch`, `login`; `LicenseApiClient.ts#checkLicense`, `activateLicense`; `LicenseCheck.ts#buildApiData`; `Signatures.ts#makeAnonymousSignature`.

## C. Roles, discovery, connection flow, and status copy

### C1. Primary/server selection

Normal role is unknown if `mainID.id` is empty or this computer has no computer record. It is server when `mainID.id===myId`; otherwise client. The GUI does not have a separate initial Server/Client radio wizard. It calls the input source the **primary** computer. “Make this primary” PATCHes `settings/mainID/{id}`. Discovery connects a **service peer**; it does not inherently promote that peer to primary. Actual core role/config generation belongs to the service. **Evidence:** `R:CoreConfig.ts#getCoreModeStandard`; `ScreenMenu.tsx#onMakePrimaryClick`; `IPCLegacy.ts#connectPeer`.

Custom-config mode uses a primary/server toggle. In its role helper, an empty server name or `.` is unknown; server name equal to local override name means server; other names mean client. The custom editor has its own `isServer` transport field and reads `settings.customConfig.isServer`. Save emits shared text plus local name/address/privilege/arguments. For a client it validates IPv4 with optional port, not arbitrary hostname/IPv6. The text is only minimally validated by the renderer; it does not parse or validate full core configuration syntax. **Evidence:** `R:CoreConfig.ts#getCoreModeOverridden`; `CustomConfigPage.tsx#onSave`, `updateScreen`; `Config.ts#validate`, `checkIPValid`.

### C2. Discovery and manual connection sequence

1. Enable `autoDiscover.discovery` to let the service discover peers. The UI has no independent discovery engine, scanner progress page, scan button, or discovery packet format. Peers arrive in refreshed `computers`/`local_computers`/`screenStatus`. **Evidence:** `R:ScreenLayoutPage.tsx#onAutoDiscoverChange`; `LayoutComputer.ts#toScreenMap`; `M:App.ts#onServiceMessage`.
2. Manage computers presents returned records with names and OS images; layout omits `isRemoved` records. Manage computer's own render loop does not filter that flag. Missing-computer help advises the same network/firewall checks and leads to manual add. **Evidence:** `R:ManageComputer.tsx#render`; `LayoutComputer.ts#validComputers`.
3. Manual add begins with one empty IPv4 row. Its menu can switch to hostname, clear, or delete (delete only with multiple rows). More rows can be added. Address edits trigger a debounced ping after 400 ms once the input looks valid. Its first IPv4 regex checks four one-to-three-digit groups; the reusable segmented input separately constrains octets to 0–255. Hostname nonempty is enough to start checking. **Evidence:** `R:ManualConnectPage.tsx#constructor`, `isValidAddress`, `connectRow`; `AddressInput.tsx#invertInput`, `render`; `libs/gui-components/src/components/IpInput.tsx` → `255`.
4. The button is enabled when at least one nonempty row exists and every nonempty row reports success. Empty rows are ignored. Expected failure display waits 1000 ms, while actual application errors are immediate. Connect performs rows sequentially through `controls/connect-peer`; it returns to layout only when these calls complete without thrown failure. This is not proof that input sharing is connected. **Evidence:** `R:ManualConnectPage.tsx#allowAddComputers`, `setRowStatusDelayed`, `connectComputers`; `ManageComputer.tsx#connectToPeer` comment and state behavior.
5. Retry on a known card sends its locally cached address through the service and shows a transient spinner. If no address is known, the dialog can refresh after 700 ms or open manual add. The result does not directly set authoritative reachability/core status; a settings refresh does. **Evidence:** `R:editor/Screen.tsx#connect`, `onRetryConnectionClick`; `ManageComputer.tsx#initStates`, `onRetryConnectionClick`; `PeerNotReadyDialog.tsx`.

### C3. Connection state model and exact short labels

| State | Visible short copy / rendering | Evidence |
|---|---|---|
| Manual row `none` | No status message/icon | `R:AddressInput.tsx#render`; `types/computer.ts#PingStatus` |
| `loading` | “One moment...” with rotating icon | Same |
| `success` | “Ready” with tick | Same |
| `failed` | “Failed”; tooltip `Result: {pingStatus}` | Same |
| `error` | “Error” | Same |
| `serial-key-mismatch` | “Serial key mismatch” | Same |
| Service reachable/unreachable | Badge tooltip “Service reachable” / “Service unreachable” | `R:FauxScreen.tsx#StatusBadges` |
| Core connected | “Keyboard/mouse connected” tooltip; green badge | `R:FauxScreen.tsx#coreStatusLabel`; `CSS:.status-badge.connected` |
| Core disconnected | “Keyboard/mouse not connected” tooltip; gray badge | Same |
| Core unknown | Core badge omitted. Label constant explains status is only visible on server, but no unknown badge is rendered in this component. | `R:FauxScreen.tsx#StatusBadges`, `coreStatusLabel` |
| Paused | Rail uses resume icon, active orange style, title “Paused”; running state title “Pause” | `R:Navigation.tsx#coreRunning`; `CSS:.tray-button-active` |
| No primary | “No primary computer selected.” | `R:editor/ErrorOverlay.tsx#ScreenLayoutError` |
| Overlap | “Computers overlap.” | Same |
| Duplicate name | “Computers have the same name.” | Same |
| License computer-count excess | Removal/upgrade/support guidance, parameterized count | Same → `checkComputerLimit` |
| Unknown address | “Can't connect yet”; “Try again” / “Connect manually” | `R:PeerNotReadyDialog.tsx` |
| Rejected signature/version | Explains serial mismatch or incompatible versions; links support; close button | `R:PeerRejectedOverlay.tsx#getMessage` |

`screenStatus[id]` is `{serviceReachable:boolean,core:"connected"|"disconnected"|"unknown"}`. It is local, derived, never synced or persisted. A card is visually active when service is reachable and core is not explicitly disconnected; unknown core can still look active. Manage-computer card highlighting uses **only service reachability**. `misc.connected` is retained in stored records but not used as present truth. This distinction should remain explicit in the native design. **Evidence:** `R:CombinedSettingsData.ts#ScreenStatus`; `Screen.tsx` → `active`; `ManageComputer.tsx#initStates`; `LayoutComputer.ts#toScreenMap`.

Peer ping result strings are `success`, `serial key mismatch`, `invalid address`, `timeout`, `cancelled`, `connection refused`, `connection reset`, `host unreachable`, `network unreachable`, `dns lookup timeout`, `dns lookup not found`, `unknown error`. Renderer recognizes success and serial mismatch by lowercasing English text; other results become Failed with a tooltip. Exceptions in renderer ping utility are caught and return `{success:false}`, so not every underlying error will produce the row's Error branch. **Evidence:** `M:Pinger.ts#PingStatusBase`, `PingStatus`; `libs/gui-common/src/AxiosUtils.ts#ConnectionErrorType`, `getConnectionErrorType`; `R:utils/Ping.ts#ping`; `ManualConnectPage.tsx#connectRow`.

### C4. Local service/startup status copy

| Condition | Short text / action | Evidence |
|---|---|---|
| Generic loading | “Please wait, loading...” | `R:status/Status.tsx#LoadingStatus` |
| Installer pending, enabled | “Waiting to install background service...” | `R:status/Service.tsx#InstallPending` |
| Installer pending, disabled | “No connection to background service (install disabled).” | Same |
| Installer running | “Installing background service...” | Same → `InstallInProgress` |
| Installer failed | Installation-problem copy and reinstall link | Same → `InstallationFailed` |
| GUI/service mismatch | “Service version is out of date.”; restart local service, restart computer, reinstall suggestions | Same → `WrongServiceVersion` |
| Reset in progress | “Resetting...” | `R:Status.tsx#StatusMessage` |
| Unknown Mac permission | “Checking permissions...” | Same |
| Service disconnected or permission unknown | Run diagnostic tool; Show log/Hide log remains available | `R:ActionLink.tsx#ActionLink`; `Status.tsx#DisplayLogButton` |
| Preload has no body/root yet | “One moment please...”; later “GUI front-end hasn't loaded yet, retrying...” | `P:Loading.ts#loop` |
| Preload exception | GUI pre-load failed fallback | `P:preload/index.ts` startup catch |
| Renderer startup exception | GUI front-end failed to load fallback | `R:App.tsx#startSafe` |

The recovered installation-status enum is Idle=0, InProgress=1, Complete=2, Failed=3. Permission statuses are Unknown/Authorized/Denied/Error. Authorized closes the separate permission window; Denied or Error opens it; Unknown waits. No runtime claims about the current installation state are made. **Evidence:** `R:ServiceInstallStatus.ts`; `CombinedSettingsData.ts#MacPermissionStatus`; `M:MacPermissionWindow.ts#showOrHide`.

## D. Visual design inventory for a native replacement

This section describes declared layout/styles, not a live screenshot. No app window was opened or manipulated. Exact rendering under a particular OS scale factor, text size, or accessibility configuration remains unverified.

### D1. Window and layout dimensions

| Surface | Declared geometry and behavior | Evidence |
|---|---|---|
| Main window | 960×650 **content** units; fixed size, no fullscreen, ordinary native frame; main title includes product tier, version, beta stage. | `M:windows/MainWindow.ts#create`, `getTitle`; `utils/Title.ts#getWindowTitle`, `getProductName` |
| About | 600×580 content, fixed size, no fullscreen, parented. | `M:windows/AboutWindow.ts#create` |
| Logs | 1000×600 content, parented if possible, ordinary resizable default. Closing stops log reader. | `M:windows/LogWindow.ts#create` |
| Diagnostics | 750×550 window, minimum 450×300, parented; resizable default. | `M:windows/DiagnosticWindow.ts#create` |
| Mac permission window | 800×650 window; frame true, not resizable, empty title. | `M:windows/MacPermissionWindow.ts#create` |
| Main icon rail | Width/min-width 80; navy background; 40×40 logo, 50×50 icon buttons with 24×24 glyphs; top padding 24. | `CSS:.tray`, `.tray-button`, `.tray > img` |
| Settings sidebar | Width 200 including box sizing; blue background; 24 left/16 right padding; vertical scroll. Header 24/32; nav rows height 32, 6-radius corners. | `CSS:.nav-bar`, `.nav-bar-title`, `.nav-bar-button`, universal `box-sizing` |
| Content pane | Remaining width; horizontal padding 24 each side; standard-page padding 5; h2 starts 41 units from top. | `CSS:.content`, `.standard-page`, `.standard-page > h2` |
| Canvas | Nominal 622×440; runtime width measured; white, radius 5, subtle `#e8e8e8` shadow. Coordinates are not physical screen pixels. | `R:Layout.ts`; `ScreenLayoutEditor.tsx`; `CSS:.screen-layout .edit-screen-container` |
| General overlay | Full parent cover, `rgba(32,59,126,0.9)` curtain; white 664×560 card, radius 6, light shadow. Preferences uses this system, inner tab/content width 616. | `CSS:.overlay-curtain`, `.overlay-content`, `.screen-settings > .tab-container`, `.content-container`; `R:Overlay.tsx` |
| Compact dialogs | Inner dialog max-width 420, padding 15; stacked content/actions, or row action variant. | `CSS:.dialog`, `.dialog__content`, `.dialog__actions` |
| Computer card | 6-unit border, gray fill; OS logo centered; name at bottom; ellipsis top-right; primary mouse badge bottom-left; connection badges bottom-right. | `CSS:.faux-screen` and descendants; `R:FauxScreen.tsx` |

The 960-wide main layout dedicates 280 units to two navigation columns before content padding. Large form rows and custom fixed-width modals then compete for a relatively narrow area. This is a design constraint, not measured user latency. **Evidence:** `M:MainWindow.ts#create`; `CSS:.tray`, `.nav-bar`, `.content`, `.overlay-content`.

### D2. Palette

| Role | Declared color | Evidence |
|---|---|---|
| Rail/nav selection/active card border | `#1f2d5d` | `CSS:.tray`, `.nav-bar-button.active`, `.faux-screen.active` |
| Sidebar | `#1b46b1` | `CSS:.nav-bar` |
| Sidebar hover | `rgba(32,59,126,0.5)` | `CSS:.nav-bar-button:hover` |
| Primary accent/update/paused indicator | `#ff7c00` | `CSS:.bg-primary`, `.nav-bar-update`, `.tray-button-active` |
| Secondary blue | `#3b67d3` | `CSS:.text-tertiary`, `.bg-tertiary` |
| Links/info/tab text | `#4285f4` | `CSS:a`, `.text-info`, `.tab-button` |
| Main text | `#19171d` | `CSS:body`, `h1`–`h4` |
| Active card text | `#4a4a4a` | `CSS:.faux-screen.active` |
| Gray body/helper/inactive | `#828282` | `CSS:.text-default`, `.faux-screen.inactive` |
| Inactive borders | `#bdbdbd` | `CSS:.faux-screen.inactive`, `.primary-badge` |
| Card background | `#ececec` | `CSS:.faux-screen` later override |
| Up/connected badges | `#2aa327` | `CSS:.status-badge.up`, `.status-badge.connected` |
| Down/disconnected badges | `#828282` | `CSS:.status-badge.down`, `.status-badge.disconnected` |
| Danger | `#ec4c47`, deeper `#cb1b15` variants | `CSS:.text-danger`, danger button/alert rules |
| Warning | `#d68233` | `CSS:.peer-rejected-overlay .alert.alert-warn`, warning utility rules |
| Active OS graphic cyan | `#2bbaef` | `CSS:.faux-screen.active svg path` |
| Teal accents | `#00ada8`, `#33b2cc` | `CSS:` literal color declarations and `.buttonBase:active` |
| Surface and foreground on dark | `#ffffff` / `#fff` | `CSS:.overlay-content`, `.nav-bar-button` |

These are product CSS colors. A SwiftUI redesign can preserve primary-role and connection-state meaning without reproducing every utility color. **Evidence:** semantic class use in `R:Navigation.tsx`, `FauxScreen.tsx`, `PeerRejectedOverlay.tsx` and selectors above.

### D3. Typography, assets, and motion

| Item | Static design facts | Evidence |
|---|---|---|
| Body | Bundled Roboto regular, 16 units; nominal 24 line height; paragraph rule later overrides line height to 1.2em. Bold/italic/bold-italic files present. | `CSS:@font-face`, `body`, `p`; `build/assets/Roboto-*.ttf` |
| Headings | Bundled Manrope; h1 36/44 bold, h2 32/40 weight 600, h3 24/32 bold, h4 18/24 weight 600. Only regular/bold face files declared, so intermediate weights may be synthesized. | `CSS:@font-face`, `h1`–`h4`; `build/assets/Manrope-*.ttf` |
| Buttons | Manrope 16 bold, minimum width 95, 10×24 padding, radius 5. Many settings pages use compact variants. | `CSS:.buttonBase`, variants; `R:Button.tsx` |
| Input sizes | 14/22, 16/24, 18/32, 24/42 type utilities; associated input heights 30/32/40/40. | `CSS:.text-sm` through `.text-xl`, `input.text-*` |
| Code/log/shortcut glyphs | SourceCodePro for monospace; NotoEmoji for symbols; shortcut display can include arrows/media/mouse glyphs. | `CSS:@font-face`, `.log-text`, hotkey styles; `R:utils/Hotkeys.ts#displayCodes`; `build/assets/SourceCodePro.ttf`, `NotoEmoji.ttf` |
| Icon set | OS marks, logo, ellipsis, keyboard/mouse, edit/remove/retry wand, log/help/network, pause/resume, user, add, arrows, tick/danger/warning/info. Mix of inline SVG JSX, data-URI SVG/PNG, and external files. No uniform native-symbol system. | `R:assets/...` source regions; `ScreenMenu.tsx` imports; `Navigation.tsx` imports; `libs/gui-components/src/components/svgs/*` |
| Large illustrations | Naming, computer details, shortcut creation, reset; authored vector assets rather than live native controls. | `build/assets/name-computer.svg`, `computer_name.svg`, `press_key.svg`, `ResetImage.svg`; corresponding R regions |
| Permission guidance | Two local MP4 assets autoplay, loop, muted, playsInline. | `R:MacPermissionsPage.tsx`; `build/assets/macos-permission.mp4`, `macos-permission-13.mp4` |
| Loading motion | Rotating icon, linear 2-second infinite rotation. | `CSS:@keyframes rotating`, `.rotating` |
| Control motion | Button hover transitions 0.15 seconds ease; round-add hover 0.15 ease-in-out; switch thumb 0.15 ease. No reduced-motion branch was found in CSS. | `CSS:.buttonBase`, `.cir-btn`, switch thumb transition; full CSS `prefers-reduced-motion` search |
| Layout movement | Direct pointer-driven card updates; no spring or easing in editor code. Touch resize events are ignored. | `R:ScreenLayoutEditor.tsx#dragScreen`, `onScreenResize`; `Screen.tsx` |
| Log movement | Programmatic scroll-to-bottom with duration 0. | `R:Log.tsx#scrollToBottom` |

**Light/dark:** main explicitly forces `nativeTheme.themeSource="light"`. CSS uses fixed light surfaces and dark navigation; no `prefers-color-scheme` handling or theme setting was found. This is not a system-dark-aware design. **Evidence:** `M:App.ts#start`; `CSS:body`, `.overlay-content`, `.tray`, `.nav-bar`; full CSS theme-query search.

### D4. What to keep and improve

- **Keep explicit role and link distinctions.** A small primary marker plus separate service/core status can reduce false “connected” claims. Tooltips carry useful meaning. Native UI should make it accessible without hover. **Evidence:** `R:FauxScreen.tsx#StatusBadges`, primary badge; `CombinedSettingsData.ts#ScreenStatus`.
- **Keep direct layout manipulation and local detail access.** Drag/resize is a compact model for edge switching, and the local-address page is useful when discovery fails. Make edge spans visible if the native replacement supports partial links. **Evidence:** `R:ScreenLayoutEditor.tsx`; `ComputerDetailsPage.tsx`; geometry in A3.
- **Reduce navigation and terminology friction.** Settings are split across local, globally synced, miscellaneous, and network pages with misleading scope copy. Tutorial primary-action text is stale. Use one storage-scope model and one primary/server term. **Evidence:** `R:GlobalSettingsData.ts`; `LocalSettingsPage.tsx` → “not synced”; `TutorialOverlay.tsx#Step2`; `ScreenMenu.tsx`.
- **Replace fixed layout and hover-only state.** The main window cannot resize; canvas cards can become small; several icon buttons remove focus outlines. `role="grid"`/`tabIndex` alone does not supply keyboard drag/resize. No such keyboard operation handler was found. **Evidence:** `M:MainWindow.ts#create`; `CSS:.tray-button:focus`, `.nav-bar-button:focus`; `R:Screen.tsx`, `ScreenLayoutEditor.tsx`.
- **Make saves consistent.** Some fields write immediately, some on blur, and preferences also have Save. Active-tab equality controls whether the whole computer record is saved. A native model should avoid hidden cross-tab edits and provide clear write/error state. **Evidence:** `R:ScreenSettings.tsx#getUpdateValue`, `onSave`; `CopyPastePage.tsx#componentDidUpdate`; `MiscellaneousPage.tsx`.
- **Use bounded, relevant error UI.** The generic modal has five choices and can launch another support prompt; Ignore all changes a persistent global dialog preference. Better inline recovery can reduce interruptions while preserving logs. **Evidence:** `M:ErrorHandler.ts#getErrorDialogButtons`, `onCopyButtonClick`, `onErrorDialogClick`.

## E. Bloat, likely slow paths, telemetry, and fragility

### E1. Measured archive composition

The ASAR is 67.47 MB decimal (64.35 MiB). The independently extracted regular files total **64,942,452 bytes across 9736 files**. These are logical file sizes, not process memory or app startup cost. The whole installed app's 524 MB figure was supplied in the task; this report did not independently attribute the external Electron framework, Qt core, or other binaries. **Evidence:** archive byte inventory; `package.json`; file-size census of extracted archive entries.

The renderer is 14,081,552 bytes. Its final inline source-map comment occupies **10,301,074 bytes (73.2%)**; content before the comment is 3,780,478 bytes. Separately packaged `.map` files total **21,148,811 bytes**, excluding that inline map. Sentry/Sentry-internal module directories occupy **33,129,275 bytes**, about 51.0% of extracted file bytes. Font files total **1,872,976 bytes**. These groups overlap (for example Sentry includes maps), so do not add them as disjoint savings. **Evidence:** `build/assets/index.js` → final map-comment byte boundary; `node_modules/**/*.map`; `node_modules/@sentry/**`, `@sentry-internal/**`; `build/assets/*.ttf` measured sizes.

The renderer source map contains approximately 4,852,016 UTF-8 bytes of dependency source text versus 467,092 bytes of app/common source text, before map mappings/encoding overhead. This supports substantial packaging headroom but does not by itself prove that removing a map will proportionally improve JavaScript execution time. **Evidence:** `build/assets/index.js` → decoded map `sources`/`sourcesContent`, grouped by app/common vs dependency source paths.

#### Top 20 largest archive files/assets

Sizes are uncompressed bytes. Each path is the direct archive evidence for its row.

| Rank | Archive path / evidence | Bytes |
|---|---|---:|
| 1 | `build/assets/index.js` | 14,081,552 |
| 2 | `build/index.cjs` | 3,001,639 |
| 3 | `node_modules/@sentry/replay/cjs/index.js.map` | 1,455,481 |
| 4 | `node_modules/@sentry/replay/esm/index.js.map` | 1,451,945 |
| 5 | `build/assets/NotoEmoji.ttf` | 878,928 |
| 6 | `build/preload/index.cjs` | 595,725 |
| 7 | `node_modules/@sentry/replay/esm/index.js` | 314,665 |
| 8 | `node_modules/@sentry/replay/cjs/index.js` | 314,593 |
| 9 | `node_modules/socket.io-client/dist/socket.io.js.map` | 310,154 |
| 10 | `build/assets/macos-permission-13.mp4` | 267,655 |
| 11 | `build/assets/macos-permission.mp4` | 251,963 |
| 12 | `node_modules/async/dist/async.js` | 224,884 |
| 13 | `node_modules/axios/dist/axios.min.js.map` | 220,272 |
| 14 | `node_modules/@sentry-internal/replay-canvas/cjs/index.js.map` | 217,309 |
| 15 | `node_modules/@sentry-internal/replay-canvas/esm/index.js.map` | 217,170 |
| 16 | `node_modules/axios/dist/esm/axios.min.js.map` | 210,895 |
| 17 | `node_modules/socket.io-client/dist/socket.io.msgpack.min.js.map` | 207,448 |
| 18 | `node_modules/async/dist/async.mjs` | 200,300 |
| 19 | `node_modules/socket.io-client/dist/socket.io.min.js.map` | 196,142 |
| 20 | `node_modules/engine.io-client/dist/engine.io.js.map` | 193,195 |

#### Top 20 archive directory/module groups

These groups aggregate all files under the stated archive prefix, including maps, types, and alternate module builds. They are not a module-load or runtime-coverage ranking.

| Rank | Archive prefix / evidence | Bytes |
|---|---|---:|
| 1 | `build/assets` | 16,691,650 |
| 2 | `node_modules/@sentry/replay` | 6,022,350 |
| 3 | `node_modules/@sentry/node` | 5,077,907 |
| 4 | `node_modules/zod` | 4,557,611 |
| 5 | `node_modules/@sentry-internal/tracing` | 3,634,738 |
| 6 | `node_modules/@sentry/core` | 3,194,140 |
| 7 | `build/index.cjs` | 3,001,639 |
| 8 | `node_modules/@sentry/browser` | 2,983,393 |
| 9 | `node_modules/@sentry-internal/replay-canvas` | 2,743,082 |
| 10 | `node_modules/@sentry-internal/feedback` | 2,602,172 |
| 11 | `node_modules/@sentry/integrations` | 2,514,454 |
| 12 | `node_modules/@sentry/electron` | 2,451,316 |
| 13 | `node_modules/axios` | 1,650,572 |
| 14 | `node_modules/@sentry/utils` | 1,592,473 |
| 15 | `node_modules/socket.io-client` | 1,416,419 |
| 16 | `node_modules/engine.io-client` | 869,939 |
| 17 | `node_modules/async` | 807,120 |
| 18 | `build/preload` | 681,993 |
| 19 | `node_modules/localforage` | 466,889 |
| 20 | `node_modules/@sentry/types` | 313,250 |

Both CJS and ESM builds, source maps, declaration maps/types, and full browser distributions of dependencies are present. Sentry replay code/assets are bundled even though the application initialization examined here does not explicitly enable a replay integration. Do not confuse packaged capability with active session recording. **Evidence:** file inventory under `node_modules/@sentry/replay/{cjs,esm}`, `@sentry-internal/replay-canvas`, `socket.io-client/dist`; `R:utils/Sentry.ts#init`; `M:utils/Sentry.ts#init3`.

### E2. Startup work and likely slow paths

| Static observation | Practical consequence / rebuild opportunity | Evidence |
|---|---|---|
| A single renderer module imports React, React DOM, Redux, router, Zod, utility libraries, all screens and telemetry. HTML loads one script and one stylesheet. | Main UI eagerly parses/initializes a broad bundle rather than loading screens on demand. A native view tree and narrow dependencies can reduce this work. No measured launch time is claimed. | `build/index.html`; `R:App.tsx` imports; `build/assets/index.js` dependency regions |
| Main eagerly includes a ~3 MB bundle; preload ~0.6 MB; each browser window uses the same preload/application page. | Extra process and per-window preload/renderer initialization for log/about/permission/diagnostic utility windows. | `M:AppWindow.ts#preloadFilePath`, `loadStartPage`; all window constructors |
| Startup loads environment/config, log handling, error/Sentry integration, GUI settings; checks app location and single-instance lock; can run service installation/launch before creating main window. | Startup depends on service-management and platform steps, rather than a very small immediately available control window. | `M:App.ts#start`, `onReady`; `ServiceProcess.ts#runInstallOrLaunch` |
| Many component constructors call synchronous `getSettings`, `diagInfo`, `networkAdapter`, or `updateTrack`. | Main-process stalls can block renderer construction. The recovered comment explicitly warns about a blank screen if main takes too long. | `R:IPCLegacy.ts#getSettings` documentation; `CopyPastePage.tsx`, `SecurityPage.tsx`, `LocalSettingsPage.tsx`, `Log.tsx` constructors |
| Generic service events cause two full settings GETs before a broadcast; mutating requests and explicit requests cause more. | Redundant work and complete-state replacement rather than narrow subscriptions. Prefer one authoritative state feed with revision/correlation. | `M:App.ts#onServiceMessage`, `sendSettings`, `serviceFetchDone` |
| Local-details page synchronously asks for network adapter every second; main enumerates interfaces each time. | Repeated OS enumeration and IPC while that view is open. Refresh on meaningful changes or user request. | `R:ComputerDetailsPage.tsx#componentDidMount`; `M:LocalHost.ts#load` |
| Log opening reads/parses **all** existing log text synchronously before tailing; renderer later caps at 1000 lines. | Large logs can stall main and transfer needless history. Read a bounded tail off the UI-critical path. | `M:LogHandler.ts#tailRead`; `R:Log.tsx#onNewLogLines` |
| Log batches run every 100 ms while open; ANSI parsing/rendering maps lines into DOM nodes. | Continuous renderer work scales with line rate and 1000-line repaint. Native bounded list/tail can reduce overhead. | `M:LogHandler.ts#tailRead`; `R:Log.tsx#render` |
| Editor uses Ramda transformations, cross-products of nearby snap transforms, JSON stringify equality, and overlap loops during pointer movement. | Potential scaling limits with many computers; optimize geometry without changing edge semantics. | `R:ScreenPositioner.ts#screenPositioner`; `LayoutComputer.ts#areComputersEqual`; `Resizer.ts#resizer` |
| Four heading/body families/variants plus emoji and videos are shipped. | Native system typography/icons and concise static permission guidance can reduce resources. Whether to retain video is a product decision. | `CSS:@font-face`; `build/assets/*.ttf`, `macos-permission*.mp4` |
| Preload polls for root every 100 ms, waits 2 seconds, then schedules reload after another 3 seconds if it still cannot find root. It stops as soon as `#root` exists, even if React never painted useful content. | Watchdog tests DOM scaffolding, not app readiness; a blank React root can escape it. | `P:Loading.ts#begin`, `loop`; `build/index.html` → `#root` |

### E3. Telemetry and update behavior

Renderer uses `@sentry/react`; main uses `@sentry/electron` plus Sentry common helpers. Production setup initializes SDKs, immediately disables telemetry, then enables it according to `send-report` when settings arrive. Sample rate constant is 1. The helpers set component/core-role tags and assign a user identity derived from the license; no identity value is retained here. Automatic session tracking is initially false and is toggled with reporting consent. **Evidence:** `R:utils/Sentry.ts#init`; `libs/gui-common/src/SentryUtils.ts#updateSettings`, `updateEmailFromSerialKey`, `updateCoreMode`, `setTelemetry`; `M:utils/Sentry.ts#init3`; `GlobalConst` → `sentrySampleRate`.

The main `beforeSend` branch for disabled/unknown reporting state returns the event rather than null, while the renderer equivalent returns null. Other SDK enable flags may still block transmission, so this is a **defense-in-depth inconsistency**, not evidence that an unwanted report was sent. No traffic was observed or generated to test it. **Evidence:** `M:utils/Sentry.ts#beforeSend`; `R:utils/Sentry.ts#beforeSend`; `SentryUtils.ts#setTelemetry`.

Several application logging sites print full IPC arguments, full settings/license objects, serial-validation input, remote-license strings, or ping response headers in the product. This presents a secret-exposure risk in the existing logs/diagnostic bundle. The research did not read those live logs and does not reproduce the values. A native replacement should redact at the logging boundary. **Evidence:** `R:IPCLegacy.ts#invoke`; `utils/Auth.ts#validateAndSaveSerialKey`; `redux/global/utils.ts#getLicenseStateFromSerialKey`; `M:utils/Auth.ts#authFetch`; `Pinger.ts#request`; `R:utils/ConsoleCapture.ts`.

Update behavior is summarized in A8: channel-aware remote version comparison and external download link, no recovered in-app automatic installer. This is less complex than a full updater, but synchronous OS information gathering/network availability still lies on the check path. **Evidence:** `M:Version.ts#fetchRemote`; `OSInfo.ts#getOSInfo`; `IPCHandlers.ts#getLatest`; `R:LayoutContainer.tsx` effect.

### E4. Concrete static fragility findings

These are source-supported candidates, not live reproductions. Research scope forbids launching or testing the target.

| Finding | Specific problem | Evidence |
|---|---|---|
| Certificate path changes source on refresh | Initial path is `local_security.path`; updates read `security.path`. Observed global record lacks the latter. | `R:SecurityPage.tsx#constructor`, `updateState`; CFG shape |
| Fourth log-level option has no mapping | Four labels are rendered, but index table only has 0–2; selecting Debug 2 can produce undefined. | `R:Log.tsx#render`; `TroubleshootingPage.tsx#render`; `LogLevels.ts#logLevelsByIndex` |
| Editing shortcut stroke may do nothing | Hotkey-actions header passes an `onChange` callback that returns `hotkeyChange` rather than calling it with the captured stroke. The compiled bundle retains this form. | `R:HotkeyActions.tsx#HotkeyActions` → header `HotkeyCaptureButton` |
| Blur-save flags can cause duplicate writes | `componentDidUpdate` patches on any flag inequality, then clears the flag. The clear transition also meets the inequality condition. | `R:CopyPastePage.tsx#componentDidUpdate`; `MiscellaneousPage.tsx`; `LocalNetworkingPage.tsx` |
| Per-computer edit state aliases settings record | State takes the record from settings and mutates nested fix/modifier/corner objects; only the original comparison snapshot is cloned. | `R:ScreenSettings.tsx#stateFromSettings`, `setFix`, `setModifier`, `setDeadCorners` |
| Per-tab comparison can miss pending cross-tab changes | Save compares the active tab's field, but sends the whole record. Switching tabs changes save eligibility without a per-tab draft model. | `R:ScreenSettings.tsx#getUpdateValue`, `computersEqual`, `onSave` |
| Role/status gate consistency uncertain | Exact service version is compared and a wrong-version view exists, but `ProtectedRoute` does not itself check `service.isValidVersion`. Reachability is not compatibility. | `R:redux/global/utils.ts#checkIsServiceVersionValid`; `Protected.tsx`; `Service.tsx#WrongServiceVersion` |
| Unknown rejection reason crashes the overlay | New service reason values are not handled by a generic fallback. | `R:PeerRejectedOverlay.tsx#getMessage` default throw |
| Multirow checks share one cancellable pinger | Main owns one Pinger whose new request aborts the prior one; manual page accepts multiple address rows. Requests can interfere even though renderer tracks row request IDs. | `M:IPCHandlers.ts#pinger`; `Pinger.ts#_ping`; `R:ManualConnectPage.tsx#connectRow` |
| One debounce/status-delay object per multirow page | All rows share a Debouncer and a single status timeout; one row's pending failure can cancel another's display. | `R:ManualConnectPage.tsx#pingDebouncer`, `statusTimeoutId`, `cancelStatusTimeout` |
| IPC rejection swallowed in preload | Catch reports and returns no explicit error result; callers may distinguish null poorly. | `P:IpcBridge.ts#invoke`; `R:IPCLegacy.ts#invoke` |
| Wide IPC surface, weak sender scoping in app handlers | Generic channel forwarding, dynamic `serviceFetch` URI/options, external URL open, filesystem existence query, reset/service controls; application handlers do not validate sender frames/origin here. | `P:IpcBridge.ts`; `M:IPCHandlers.ts#register`; `IPCListeners.ts#registerLegacy` |
| Node-enabled windows despite context bridge | Main/About/Log explicitly enable Node integration and context isolation; Diagnostics enables Node integration without an explicit contextIsolation setting. Actual Electron defaults apply where omitted. | `M:MainWindow.ts#create`, `AboutWindow.ts#create`, `LogWindow.ts#create`, `DiagnosticWindow.ts#create` |
| Diagnostic reset listener registered during render | `HardResetDialog` registers `reset-finished` outside an effect and never unsubscribes. Re-renders can add listeners. Diagnostic output effect also re-registers every render but does return cleanup. | `R:HardResetDialog.tsx`; `DiagnosticPage.tsx` effect |
| Application subscription cleanup incomplete | App stores log-open/log-close unsubscribes but omits them from the returned cleanup function. StrictMode makes lifecycle correctness particularly relevant. | `R:App.tsx#init`, cleanup; `start` → StrictMode |
| Custom argument splitting is naive | Splits on literal spaces; quoted paths and repeated spaces cannot be represented reliably by the editor's round trip. | `R:CustomConfigPage.tsx#onSave`, `updateScreen` |
| Numeric limits are mostly HTML hints | Several fields directly parse input; UI min/max do not prove server rejection of out-of-range or NaN/null values. | `R:MiscellaneousPage.tsx`, `LocalNetworkingPage.tsx`, `CopyPastePage.tsx` change/blur handlers |

A native rebuild can beat this architecture with a small typed local control interface, one authoritative status stream, separate service/core states, bounded log loading, explicit asynchronous saves, system typography, accessible controls, and no account/telemetry/update dependency on the basic sharing loop. These are design conclusions from the observations, not measured promises about a future implementation. **Evidence:** B1–B6 boundaries; E1–E4 findings.

## F. Global schema v17, local schema v7, and visible migrations

### F1. Persistence boundary

The existing global file is an envelope with `version:{schemaVersion:17,syncVersion:number}` and `data`. Local file has top-level `version:7` and settings directly at the root. Renderer receives a **merged transport object**, not either file verbatim. Its imported global type omits the disk envelope; its local type includes `version:number`. `databaseVersions` supplies global/local schema information separately when present. A SwiftUI model should not accidentally sync local identities, paths, licensing state, addresses, tutorial state, or runtime reachability just because they occur in the merged response. **Evidence:** `R:libs/common/src/settings/GlobalSettingsData.ts#GlobalSettingsData`; `LocalSettingsData.ts#LocalSettingsData`; `CombinedSettingsData.ts#CombinedSettingsData`, `DatabaseVersions`; CFG redacted shape showing global 17/local 7.

The following is an independently organized, TypeScript-like specification of the recovered fields. It is documentation only. It combines the source-map type declarations, runtime uses, and observed disk shape. Optional/widened fields are explicitly explained below. Identifiers are kept for compatibility; organization and prose are original.

### F2. Global file and computer records

```ts
// Primitive strings that carry identities or secrets are never example values.
type ComputerId = string;
type Modifier = "Shift" | "Ctrl" | "Alt" | "Meta" | "Super";
type Platform = "macos" | "windows" | "linux";
type CornerSize = "" | number; // UI accepts digits, then parseInt; unit: px

type DbV17 = {
  version: { schemaVersion: 17; syncVersion: number };
  data: SharedSettings;
};

type SharedSettings = {
  computers: ComputerRecord[];
  mainID: { id: ComputerId }; // empty string means no primary
  autoDiscover: { discovery: boolean };
  hotkeys: Record<string, Shortcut>;
  keyboard: { altgr: boolean; lock_to_screen: boolean };
  mouse: { relative: boolean };
  performance: { allow_copy: boolean; limit: number; sync: boolean };
  security: {
    encryption: boolean;
    key_length: number; // UI: 2048 | 4096
    fingerprint: string; // sensitive; <redacted>
    path?: string; // declared by old type; absent in observed global file
  };
  "local-network": { keep_alive: boolean; interval: number };
  system: { auto_restart: boolean; admin_daemon: boolean; no_daemon: boolean };
  developers: {
    corePort: number;
    dragAndDrop: boolean;
    showMeta: boolean;
    shouldSwitchDoubleTap: boolean;
    switchDelay: number;
    switchDoubleTap: number;
  };
  override: {
    shouldOverride: boolean;
    configFile: string; // configuration text
    serverName: string; // edit request may temporarily send null
  };
};

type ComputerRecord = {
  id: ComputerId;
  name: string;
  hostname: string;
  isRemoved: boolean;
  scroll_sensitivity: number;
  dead_corners: Record<"top-left" | "top-right" | "bottom-left" | "bottom-right", CornerSize>;
  modifier: Record<Modifier, string>; // UI restricts outputs to Modifier and uniqueness
  fix: Record<
    "capslock" | "num_lock" | "scr_lock" | "xtest" |
    "sync_language" | "invert_scroll" | "prevent_sleep",
    boolean
  >;
  hdk: Record<"caps" | "scr_lock" | "num_lock", boolean>;
  misc: CanvasMetadata;
};

type CanvasMetadata = {
  left: number;
  top: number;
  width: number;
  height: number;
  os: Platform;
  connected: boolean; // legacy persisted value, not authoritative runtime status
  logo: string;
  customImage: null | string; // persisted type says null; rendering accepts string
  screenCount: number;
  screens: Record<string, Record<string, unknown>>; // editor's array type conflicts
};
```

**Evidence:** `R:GlobalSettingsData.ts#GlobalSettingsData`, `Computer`, `ComputerMisc`, `ComputerFixes`, `HalfDuplexKey`, `GlobalSecurityData`, `DeveloperSettings`; `LayoutComputer.ts#LayoutComputer`; `Screen.tsx#targetIcon`; CFG envelope/field presence. `CanvasMetadata.customImage` is intentionally widened for UI behavior; `security.path` is intentionally optional for the observed file. `screens` follows the persisted/object declaration, not the inconsistent editor array annotation.

### F3. Shortcut records and actions

```ts
type StrokeModifiers = {
  control: boolean;
  alt: boolean;
  meta: boolean;
  shift: boolean;
};
type HIDStroke = StrokeModifiers & (
  | { type: "keystroke"; key: string } // DOM code, e.g. KeyX, not a character
  | { type: "mousebutton"; button: string }
);
type Shortcut = {
  id: string;
  enabled: boolean;
  keystroke: HIDStroke;
  actions: ShortcutAction[];
};
type ActionBase = { enabled: boolean; trigger: 0 | 1 }; // pressed / released

type ShortcutAction = ActionBase & (
  | {
      type: "MAP_TO_KEYSTROKE" | "MAP_TO_KEYDOWN" | "MAP_TO_KEYUP";
      keystroke: HIDStroke;
      targets: string[]; // computer names; UI shows empty array as All
    }
  | { type: "SWITCH_TO_COMPUTER"; target: string }
  | { type: "SWITCH_IN_DIRECTION"; target: "left" | "right" | "up" | "down" }
  | { type: "LOCK_CURSOR"; lockType: "toggle" | "on" | "off" }
  | { type: "RESTART_CORE" }
);
```

These are the records actively used by the current shortcut editor and Zod discriminated unions. The common source also contains older, unused-looking `HotKeyActionOriginal` types with PascalCase action names, array-based key strokes, and ordinal enums. They are **not** the current renderer endpoint payloads, and no migration between them is shown here. **Evidence:** `R:GlobalSettingsData.ts#HIDStroke`, `HotKey`, `HotKeyAction`, `HotKeyActionOriginal`; `EditHotkeyAction.tsx#makeAction`; `HotkeyCaptureButton.tsx`.

### F4. Local file

```ts
type ElevationPolicy = 0 | 1 | 2; // AsNeeded / Always / Never
type CoreLogLevel = "FATAL" | "ERROR" | "WARNING" | "INFO" | "DEBUG" | "VERBOSE";

type LocalV7 = {
  version: 7;
  myId: ComputerId; // sensitive identity; <redacted>
  ips: Record<string, boolean>;
  local_computers: Record<ComputerId, string>;
  serial: string; // secret; <redacted>
  "send-report": boolean;
  license: {
    graceStartEpochSecs: number;
    offlineActivationResponse: string; // sensitive; <redacted>
    holdsServerActivation: boolean;
  };
  diagnostic: { level: CoreLogLevel; dir: string };
  local_security: { path: string };
  local_system: {
    path: string;
    run_on_login_screen: boolean;
    core_enabled: boolean;
  };
  local_developers: {
    keepForeground: boolean;
    elevate_privileges: ElevationPolicy;
  };
  local_override: {
    computerName: string;
    serverIP: string;
    elevate_privileges: ElevationPolicy;
    args: string[];
  };
  network: { interface: string };
  tutorial: { finished: boolean };
};
```

**Evidence:** `R:LocalSettingsData.ts#LocalSettingsData`, `LocalLicenseData`, `CoreElevateMode`, `CoreLogLevel`, `LocalDataSystem`; CFG version/shape. Literal local version 7 is an observation, not a constant enforced by the renderer's `number` annotation. `core_enabled` default true is documented in the embedded source; other local factory defaults remain unproven here.

### F5. GUI settings and merged runtime response

```ts
type GuiSettingsFile = {
  showCriticalErrors: boolean; // default true
  showCoreErrors: boolean; // default false
  filterLevel: "All" | "Core" | "Service"; // default All
  suppressWaylandDialog: boolean; // default false
  updateTrack: "stable" | "beta"; // default stable
};

type LinkStatus = {
  serviceReachable: boolean;
  core: "connected" | "disconnected" | "unknown";
};
type CombinedSettings = SharedSettings & Omit<LocalV7, "version"> & {
  version: number; // local schema number in imported merged type
  ready: boolean;
  customConfig: { isServer: boolean };
  databaseVersions?: { localVersion: number; globalVersion?: number };
  screenStatus?: Record<ComputerId, LinkStatus>;
  coreRunning?: boolean;
  serviceInfo?: {
    serviceVersion: string;
    coreVersion?: string;
    permissionStatus: "Unknown" | "Authorized" | "Denied" | "Error";
    preLoginAccess: boolean;
  };
};
```

**Evidence:** `M:utils/GUISettings.ts#GuiSettings.load`, `fromObject`; `R:CombinedSettingsData.ts#CombinedSettingsData`, `ScreenStatus`, `ServiceInfo`; `LocalSettingsData.ts#version`. GUI settings loader accepts arbitrary file keys into its map rather than applying a strict recovered object validator; the block lists known keys, not a proven rejection policy for extensions. Main `Settings.update` requires ready/serviceInfo/diagnostic, so a transport partial object should not be treated as a fully usable settings response.

`screenStatus` and `coreRunning` are explicitly described in the embedded types as node-local, derived, recomputed on read, and not persisted/synced. `customConfig.isServer` is a response field used for licensing and custom mode, not a substitute disk schema for `mainID` or override values. **Evidence:** `R:CombinedSettingsData.ts` field documentation; `App.tsx` license effect; `M:Settings.ts#update`.

### F6. Migration evidence and limits

| Topic | What is established | What is not established | Evidence |
|---|---|---|---|
| Current global version | Existing file says schema 17; has sync version plus data envelope. | Schema-17 constructor/defaults and from-version transforms. | CFG shape; `R:GlobalSettingsData.ts` |
| Current local version | Existing local file says 7; merged type allows number. | Local-version history or upgrade algorithm. | CFG shape; `R:LocalSettingsData.ts` |
| UI settings upgrade | Loader seeds five known defaults then overlays stored keys; unreadable/missing file falls back to defaults and reports read errors. | No numbered GUI schema/migration chain. | `M:GuiSettings.load`, `fromObject` |
| Runtime status split | Current UI uses derived `screenStatus` instead of persisted `misc.connected`; field is retained in records. | Which service version first migrated behavior, and whether old stored flags are deleted anywhere. | `R:CombinedSettingsData.ts`; `ManageComputer.tsx#initStates`; `LayoutComputer.ts#toScreenMap` |
| Half-duplex fields | Both `hdk.*` and `fix.*` exist; active controls write `fix.*`. | A historical `hdk`→`fix` migration or aliasing rule. | `R:GlobalSettingsData.ts`; `ScreenSettingTabs.tsx` |
| Legacy shortcut model | Old type family and current discriminated unions coexist in embedded source. | A migration between their wire/storage formats. | `R:GlobalSettingsData.ts#HotKeyActionOriginal`, `HotKeyAction` |
| Security path discrepancy | Old global path type is present; observed file uses local path; refresh uses the old path. | A confirmed formal migration that moved the path. | `R:SecurityPage.tsx`; `GlobalSettingsData.ts`; `LocalSettingsData.ts`; CFG shape |

No numbered global/local database migrations, complete database initializer, schema-17 default table, or database-level validator for every settings section was recovered in the UI application sources. The UI depends on `GET settings` from the service and sends mutations back; migration authority therefore needs the service analysis. Dependency-library “migration” strings are not evidence of Synergy settings migrations. **Evidence:** `M:Settings.ts#update`; `R:IPCLegacy.ts`; source-map application/common inventory; application source searches for migration/schema initialization.

## Open questions / uncertainty

1. **Factory defaults:** all `D=?` values require the service's initializer/migrations or a disposable clean installation. The current user's values are not a baseline. This work did not reset or modify anything. **Evidence:** UI reads service values in page constructors; no initializer in recovered UI regions.
2. **Exact core config links:** geometry and batch payloads are recovered; service generation of `left(0,100)`/destination ranges, ordering, quantization, names, and edge tolerance is not. Use A3 as a geometric model only. **Evidence:** `R:Linker.ts`, `LayoutComputer.ts#createPatchList`; config text supplied by `GET custom/settings`.
3. **Actual consumers of stored-only fields:** `performance.sync`, `system.admin_daemon`, `system.no_daemon`, `developers.dragAndDrop`, `scroll_sensitivity`, `hdk.*`, `misc.screens` need service/core tracing. No current UI control proves that they work. **Evidence:** `R:GlobalSettingsData.ts`; screen render/control inventory.
4. **Trust and fingerprints:** dedicated certificate trust UX is not in recovered React screens. Other binaries may own it. Trusted-server contents and key material were deliberately not inspected. **Evidence:** A6 bounded search and generic-dialog contract.
5. **Local API security:** the GUI supplies no auth options. Service bind addresses, request validation, CORS/origin policy, socket admission, and local attacker access are not established here. This report makes no tested vulnerability claim. **Evidence:** `M:Fetch.ts`, `ServiceConnection.ts`, `IPCHandlers.ts`.
6. **Status and pause semantics:** UI clearly separates service/core status and stores local core-run intent. Actual synchronization frequency, core startup policy, network failures, license-disabled enforcement, and login-screen behavior require service/core evidence. **Evidence:** `R:CombinedSettingsData.ts`; `Navigation.tsx`; `App.tsx`; service endpoints in B5.
7. **Native tray and auxiliary menus:** Electron menu is recovered. `synergy-tray`, security, launcher, diagnostic helper internals are outside this artifact. **Evidence:** `M:WindowManager.ts#createMenus`; `DiagnosticTool.ts` only launches helper.
8. **Performance:** no live launch/RSS/CPU/network profiling was permitted. Size and static complexity are measured; speed conclusions are opportunities, not benchmarks. **Evidence:** E1 file census; E2 code paths.
9. **UI rendering/accessibility:** design is derived from CSS/component structure. No screenshots, browser flow, pointer interaction, permission prompts, account flow, or target runtime tests were performed, as required by the static-only safety boundary. Contrast/layout at user display scaling and keyboard accessibility need a future isolated evaluation. **Evidence:** D selectors and component handlers.
10. **REA coverage:** full-bundle graph has a renderer heap-limit warning; focused static REA on the recovered IPC/layout/settings source subset also completed. That subset has unresolved external imports by design and includes source-map declarations that can be tree-shaken from executable code. Manual compiled-region checks distinguish current call sites from source-only legacy wrappers. **Evidence:** scratch `rea-analysis.json#limitations`, `rea-focused.json#limitations`; original bundle region comparisons.
11. **Type discrepancies:** global security path, customImage, screens, nullable custom serverName, and source-only legacy wrappers cannot be resolved by trusting one exported type. Wire-level schema must accommodate observed reads/writes and should be finalized with the service report. **Evidence:** F2 notes; A3/A4; B2/B5.

## Research verification

- Verified the copied archive digest against the installed archive at the end of research; no target writes were performed.
- Checked A–F coverage against the requested feature list, enumerated application IPC registrations and renderer call sites, and separated defaults, observations, runtime-only fields, and inferences.
- Scanned the report against sensitive strings from the live configuration without printing those strings. The only substring match was the public loopback constant `127.0.0.1`, independently present in application code. No actual credential, fingerprint, machine identity, computer name, hostname, or email value appears in the report. All sensitive examples use types or `<redacted>`.
- Documentation only: no implementation, dependency, lockfile, or test file was written by this research task. No git mutation command was run; the only repository file written was this requested report. No build/typecheck applies to this Markdown artifact. Runtime/e2e tests were not run because they would violate the explicit static-only target restriction.
