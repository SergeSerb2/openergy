"""Freeze review metadata only. Does not import or execute the runtime controller."""
import hashlib
import json
import os
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
fixture = json.loads((HERE / 'fixture.json').read_text())
ROOT = Path(fixture['root'])
ID = fixture['identifier']
RAW = HERE / 'raw'
public = json.loads((RAW / 'public-fixtures.json').read_text())
trees = json.loads((RAW / 'bundle-inventories.json').read_text())
driver = json.loads((RAW / 'driver-inventory.json').read_text())


def sha(p):
    with p.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


source_hashes = {row['path']: sha(REPO / row['path']) for row in json.loads((RAW / 'app-source-binding.json').read_text())}
source_hashes.update(json.loads((RAW / 'daemon-source-binding.json').read_text()))
for path in ['tools/release/sparkle.py', 'tools/release/priority.py']:
    source_hashes[path] = sha(REPO / path)
private_hashes = {str(ROOT / path): sha(ROOT / path) for path in ['fixture.json', 'launch.lock',
    'state/config.toml', 'public-key.txt', 'public/appcast.xml', 'public/openergy-fixture.zip']}
cache_parent = Path('/var/folders/s6/_07cr4w13vsg_myjkv9ldpqc0000gn/C').resolve(strict=True)
caches = [str(parent / identifier) for parent in [Path('/Users/serge/Library/Caches'), cache_parent,
          ROOT / 'home/Library/Caches', ROOT / 'driver-home/Library/Caches'] for identifier in [ID, ID + '.driver']]
target_env = dict(HOME=str(ROOT / 'home'), CFFIXED_USER_HOME=str(ROOT / 'home'), TMPDIR=str(ROOT / 'tmp') + '/',
    XDG_CONFIG_HOME=str(ROOT / 'state'), XDG_CACHE_HOME=str(ROOT / 'home'),
    OPENERGY_CONFIG=str(ROOT / 'state/config.toml'), OPENERGY_CONFIG_DIR=str(ROOT / 'state'),
    OPENERGY_TLS_DIR=str(ROOT / 'state/tls'), OPENERGY_SOCKET=str(ROOT / 'state/control.sock'),
    OPENERGY_DAEMON=str(ROOT / 'target/Openergy.app/Contents/Helpers/openergy'),
    OPENERGY_DAEMON_ARGS='--platform virtual', OPENERGY_DISABLE_DISCOVERY='1', OPENERGY_ALLOW_PORT_24800='0')
scope = ('One initial no-argument running private target launch; real private AppModel and virtual Off daemon startup; '
    'one pinned external Sparkle CLI download/validation/Apple quit/replacement and NSWorkspace no-argument bundle relaunch; '
    'new private model/daemon readiness; one normal final quit; exact owned helper/job/cache/endpoint cleanup; '
    'normal Nice=0 external Sparkle helpers, LaunchServices registration/timestamp notifications and normal '
    'Gatekeeper/gktool/XProtect assessment. Persistent shared registration/assessment state is unknown; '
    'no full restoration promise. All explicit fixture network requests use only literal 127.0.0.1:56327. '
    'No bypass or suppression of normal OS security checks, native input/clipboard/live sharing, installed-app '
    'or production config/TLS/default IPC changes, Xcode registration build, shared identity unregister, or publication.')
plan = dict(
    schema=1, status='DO-NOT-RUN', approval_received=False,
    runtime_accepted=False, independent_runtime_reproduction=False,
    starting_root='ad280644a96905d9831825bb4f87d1388dbdca73',
    reviewed_bootstrap_candidate='7287e9aa4caf6570fc4116a89908d548a258f302',
    reviewed_bootstrap_root='c62bfdb', independent_R4='f190240318f4e2fca68a869ad0bb78225dbb4e7b', R4_root='616b1ad',
    daemon_source_root='ad280644a96905d9831825bb4f87d1388dbdca73',
    later_stop_fix='User reported root3183a9e/f19cfc0 after daemon, both signed bundles, archive and signatures were frozen. Retain declared old daemon source; no rebuild or imported timing claim.',
    approval_scope=scope,
    previous_approval=dict(time='2026-10-23T17:16:55-07:00', case='5c2ed82',
        scope='completed stopped-target negative and positive cases only', applies_to_this_operation=False),
    missing_prerequisites=['Independent review of this exact frozen controller, proposal and fixture bindings',
        'New explicit user approval of approval_scope bound to proposal/controller SHA256',
        'At execution preflight: live marker with at least240s remaining; all immutable hashes/modes unchanged; '
        'no fixture process/job/cache/socket/receipt and exclusive ownership of the exact loopback port'],
    packaging_blockers=[],
    fixture=fixture, marker=fixture['marker'], target_environment=target_env,
    source_hashes=source_hashes, private_file_hashes=private_hashes,
    evidence_hashes={p.name: sha(p) for p in [RAW / 'bundle-inventories.json', RAW / 'driver-inventory.json',
        RAW / 'common-code-identity.json', RAW / 'public-fixtures.json', RAW / 'daemon-artifact.json',
        HERE / 'static/audit.json']},
    controller=dict(path=str(HERE / 'controller-DO-NOT-RUN.py.txt'), sha256=sha(HERE / 'controller-DO-NOT-RUN.py.txt'),
        preparation='text and AST parse only; no import, execution, simulation, unit test or mock'),
    public_key=public['public_key'], archive_signature=public['archive_signature'],
    http_files={'/' + name: row for name, row in public['files'].items()},
    feed_url=public['feed_url'], download_url=public['download_url'],
    official_sparkle=dict(version='2.10.0', revision='eef1a539a373c1f1a320624b1130fc5de7b2e100',
        binary_artifact_checksum='17e28312b8e18ab7cdbbe09a6fb28cc55a5479ec6c371dbc07cdecd2a14fd959',
        signer_sha256=public['signer_sha256'],
        driver=str(ROOT / 'driver/sparkle.app'),
        driver_executable_sha256=driver['Contents/MacOS/sparkle']['sha256'],
        old_repaired_driver_retained='/private/tmp/oe-stopped-update-llil939f/driver/sparkle.app',
        source_review='raw/driver-source-binding.json; exact 602-entry official source export and complete retained tree',
        original_Xcode_build='Historical host-limit breach remains unaccepted; not repeated or hidden'),
    external_cli_command=[str(ROOT / 'driver/sparkle.app/Contents/MacOS/sparkle'), str(ROOT / 'target/Openergy.app'),
        '--check-immediately', '--feed-url', public['feed_url'], '--user-agent-name', 'Openergy-private-verification', '--verbose'],
    initial_command=[str(ROOT / 'target/Openergy.app/Contents/MacOS/OpenergyPrivateLauncher')],
    initial_entry='Frozen CFBundleExecutable, zero external arguments. Launcher execve preserves PID and supplies private environment before protected Swift image load.',
    relaunch='Only pinned external Sparkle progress agent may call NSWorkspace.openURL on the exact target bundle, with no argument/environment repair. Controller never launches a second target.',
    final_quit='NSRunningApplication.runningApplication(withProcessIdentifier:).terminate() for independently matched new PID/start/UID/executable only; no native input or AppleScript.',
    limits=dict(initial_launches=1, updater_invocations=1, explicit_normal_quits_at_most=1,
        loopback_port=56327, socket_binds=1, reuse_address=False, fallback_ports=0,
        initial_readiness_seconds=20, bootstrap_internal_readiness_seconds=15, update_and_relaunch_seconds=90,
        normal_final_exit_seconds=10, cleanup_SIGTERM_not_before_seconds=10,
        cleanup_SIGKILL_or_job_bootout_not_before_seconds=15, whole_runtime_seconds=240,
        owned_active_processes_max=12, owned_total_process_identities_max=32,
        target_app_PIDs_max=2, daemon_PIDs_max=2, process_sample_period_seconds=.05,
        HTTP_methods=['GET', 'HEAD'], HTTP_exact_routes=['/appcast.xml', '/openergy-fixture.zip'],
        HTTP_total_requests_max=4, HTTP_per_method_route_max=1, HTTP_header_bytes_max=8192,
        HTTP_body_request_bytes=0, HTTP_connection_timeout_seconds=2, HTTP_backlog=2,
        HTTP_sent_body_bytes_max=sum(x['bytes'] for x in public['files'].values()),
        IPC_methods=['status.get', 'config.get'], IPC_requests_max=4, IPC_request_bytes_max=256,
        IPC_response_bytes_max=65536, IPC_timeout_seconds=2,
        CLI_stdout_bytes_max=8388608, CLI_stderr_bytes_max=8388608,
        initial_stdout_bytes_max=1048576, initial_stderr_bytes_max=1048576,
        readonly_OS_command_seconds_max=5, readonly_OS_capture_bytes_max=33554432,
        inventory_entries_max=8192, inventory_file_bytes_max=268435456),
    supervisor_stop_limit='Unchanged Swift DaemonSupervisor sends SIGTERM, waits up to1.5s, then can sendSIGKILL without a join. Nil PID, exit receipt, shutdown return or CLI success is not child-exit proof. Independently observe old daemon PID/start/UID/path absence and ESRCH, and reject late restarts.',
    owned_jobs=[ID + '-sparkle-updater', ID + '-sparkle-progress'], owned_cache_roots=caches,
    allowed_helper_sha256=sorted({v['sha256'] for tree in [trees['candidate'], driver] for k, v in tree.items()
        if 'sha256' in v and k.endswith(('/Autoupdate', '/MacOS/Updater', '/MacOS/Downloader', '/MacOS/Installer'))}),
    owned_cleanup='After normal app quit and independent app/daemon/helper exit, remove only initially absent exact cache roots with unchanged observed device/inode/currentUID, no symlink, bounded inventory. '
        'If exact expected launchd job remains after15s, require unique program path inside owned root/cache and exact frozen helper hash before one bootout of gui/UID/exactlabel. '
        'No identity-wide unregister, no shared gktool/lsregister signal, no unknown process adoption, no stale socket unlink, no automatic retry. '
        'Unexpected ownership, remaining process/job or unknown cache path is a failed cleanup with retained evidence. Keep fixture bundles/config/preferences/build state/evidence for review; these are declared retained files, not leaks claimed removed.',
    readonly_pre_post_evidence=['libproc BSD PID/PPID/UID/start_sec/start_usec/nice and independent proc_pidpath; ESRCH corroborates exits',
        'Target/child KERN_PROCARGS2 only after exact ownership binding; expected allowlist paths and noDYLD overrides',
        'private socket device/inode/UID and child-owned lsof path; two bounded real framed IPC reads per launch',
        'exact build/plist/config/whole bundle trees and nested/deep all-architecture codesign checks',
        'fixture-filtered lsregister -dump before and after; never save full shared database',
        'exact launchctl print gui/UID/fixture-updater and progress labels before/after and at cleanup',
        'lsof -nP -iTCP:56327 before/after; no query or use of production24800/5353',
        'one bounded log show --last4m --style ndjson with exact fresh root/ID predicate; preserve normal Gatekeeper/gktool/XProtect and LS diagnostics; no trust inference from raw result codes',
        'old/new ready and exit receipt bytes plus independent identities; receipts never substitute for process observations',
        'per-request exact route/content-length/fileSHA/sent body count and first full CLI streams; no requests during preparation'],
    host_effects=dict(target='Initial running target, real AppModel startup/quit and supervised virtual Off daemon, replacement and no-argument relaunch are explicitly in the future approval scope.',
        helper_priority='Pinned Sparkle launchd helper jobs normally Nice=0. Targets and daemons must actually beNI15. Do not claim helpers areNI15 or renice any unrelated process.',
        launchservices='Private driver/nested helpers and target may register; installer timestamps notify LaunchServices. Normal notifications remain enabled.',
        assessment='Normal Gatekeeper/gktool/XProtect assessment remains enabled. No bypass, quarantine removal, policy exception or forced trust clearance.',
        persistence='Registration/assessment state can persist. Exact state and reversibility are unknown. Owned job/cache cleanup is not full shared-state restoration.',
        network='All intentional fixture HTTP and Unix IPC are exact owned loopback/private endpoints. No remote URL/fetch is authorized. PrivateHOME is not proof of external helper or OS-service confinement.'),
    safety_refusals='Preflight only: missing/expired/mismatched marker, wrong nonce, unsafe modes/links, source/artifact drift, existing receipt/socket/process/job/cache, occupied exact port or missing exact approval/review => refuse before target launch. '
        'No extra runtime invalid-bundle cases or installer negative case. Existing stopped negative proof is not repeated. No environment/config refresh to make a failure pass.',
    runtime_unverified=['Bundle.main after POSIX exec', 'initial no-argument entry and protected-image environment',
        'real AppModel/private daemon readiness', 'normal Apple quit and independent old child exit',
        'real externalCLI download/validation/replacement', 'NSWorkspace no-argument relaunch and new identities',
        'same private config/environment and new virtual daemon', 'orderly final exit and exact owned cleanup'],
    proof_limits='This future path can establish external official CLI/harness acceptance only. Require independent review of first raw observations. '
        'Source order plus noarg launch command/samePID/procpath/environment is not a kernel dyld trace. Process sampling can miss short entry/helper events; '
        'missing LS/NSWorkspace/exec evidence remains UNVERIFIED. Log byte ceilings are sampled and OS-command capture limits are checked after capture; these do not prove a strict peak resource bound. '
        'No standard visual/manual AppUpdater controller, native input, DeveloperID, notarization, trusted release or publication acceptance.',
    signing=dict(mode='ad-hoc options0, no disable-library-validation entitlement; no credential',
        fixture_key='New disposable Ed25519 seed only, mode0600 inside0700root, deleted after official archive and full-feed verification. No key material in repository/evidence; only public metadata retained.',
        production_archive_contract='tools/release/sparkle.py retains dev.openergy.app/HTTPS/universal gates. This separate private fixture packaging path never calls its production archive command.'),
    static_validation='Both package-entry invocations and all59 preparation commands passed after retained pre-command preflight failures. Additional static metadata audit passed. '
        'Daemon compile46.44s exited0 with preserved rust-objcopy SIGABRT/libLLVM stripping warning; no rebuild/fix for numbers. '
        'Compiled/packaged Swift is reviewed arm64DEBUG; final fixture is not a final release candidate.')
with (HERE / 'operation-DO-NOT-RUN.json').open('x') as f:
    json.dump(plan, f, indent=2, sort_keys=True)
    f.write('\n')
print(json.dumps({'proposal_sha256': sha(HERE / 'operation-DO-NOT-RUN.json'),
                  'controller_sha256': plan['controller']['sha256'], 'approval_received': False}))
