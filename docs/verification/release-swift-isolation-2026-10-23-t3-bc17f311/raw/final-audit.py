"""Finalize retained read-only failure evidence; no compiler, package or product launch."""
import gzip,hashlib,json,os,stat,struct,subprocess
from pathlib import Path
from prepare import P,REPO,sha,save,load,preservation,source_inventory,meta,inventory
DEST=REPO/'docs/verification/release-swift-isolation-2026-10-23-t3-bc17f311'
def digest(data):return hashlib.sha256(data).hexdigest()
def export(path,relative=None):
 original=path.read_bytes();relative=Path('raw')/(relative or path.name);payload=original
 if path.suffix not in ['.py','.sb']:payload=gzip.compress(original,mtime=0);relative=Path(str(relative)+'.gz')
 out=DEST/relative;out.parent.mkdir(parents=True,exist_ok=True)
 with out.open('xb') as f:f.write(payload)
 return dict(path=str(relative),private_original=str(path),original_sha256=digest(original),original_bytes=len(original),payload_sha256=digest(original),exported_sha256=digest(payload),exported_bytes=len(payload),transform='exact original bytes'+('; gzip mtime 0' if path.suffix not in ['.py','.sb'] else ''))

def main():
 assert load('preservation-initial.json')==preservation()==load('preservation-after-package.json')
 assert len(list(P.glob('manifest-*.guard')))==1 and (P/'package-once.guard').exists()
 assert load('package-result.json')['exit']==1 and not load('package-result.json')['timeout']
 assert not list((P/'output').iterdir())
 stderr=(P/'package.stderr').read_text();assert 'output-file-map.json' in stderr and 'sandbox_apply' not in stderr and "couldn't create cache" not in stderr
 assert 'rust-objcopy' in stderr and 'libLLVM.dylib' in stderr
 binary=P/'cargo-target/aarch64-apple-darwin/release/openergy';header=binary.read_bytes()[:32];magic,cpu,subtype,filetype=struct.unpack('<IIII',header[:16]);assert magic==0xfeedfacf and cpu==0x0100000c and filetype==2
 limits=dict(overall='FAILED',package_invocations=1,manifest_invocations=1,manifest_budget=3,original_failure_resolved='Canonical Swift plus explicit private paths/native engine/--disable-sandbox successfully compiled and executed the actual project manifest under unchanged outer filesystem/network isolation.',new_failure='Real package exited 1 during native Swift app build planning: encountered an I/O error (code: 1) while reading private Openergy.build/output-file-map.json.',new_failure_path_inspection='The named output-file-map.json was absent after failure. Read-only path/metadata inspection retained. Cause of failure to generate/read this file remains unresolved; no additional compiler/build/package command ran.',real_manifest_limit='The successful describe preflight evaluated the real project manifest. It did not exercise native app output-file-map generation. Full app build isolation is not established.',compiled_daemon=dict(path=str(binary),sha256=sha(binary),bytes=binary.stat().st_size,metadata=meta(binary),mach_o=dict(magic=hex(magic),cpu_type=hex(cpu),filetype=filetype),executed=False),not_reached=['whole-app compilation/link completion','bundle assembly','ad hoc or nested signatures','ZIP/mode/link roundtrip','archive/BUILD/source binding','staged version/absent daemon/status/invalid method/stop flow','extracted version/absent daemon/status/invalid method/stop flow','extracted signature verification','App UI, public nil-plist entry runtime and private fixture/launcher execution'],preflight_recipe_sha256=sha(P/'manifest-recipe-1.json'),package_plan_sha256=sha(P/'package-plan.json'),outer_policy_sha256=sha(P/'package.sb'),priority='All observed owned identities at absolute NI15; no Swift frontend/codesign/product daemon observed in package attempt. Manifest preflight has separate compiler observations.',signing='Identity and purpose unset; ordinary ad hoc policy validated before build. Signing not reached.',rust_warning='Pinned 1.99.1 rust-objcopy cannot load @rpath/libLLVM.dylib; raw warning unchanged; no toolchain substitution.',review='New explicit operator opt-in requires independent source review and full native build validation before general release use. Normal default engine and sandbox flags unchanged. No independent review performed in this task.',old_runs='Both old failed attempts remain FAILED and immutable; byte/link recovery remains recorded; original metadata restoration remains UNVERIFIED.',excluded='No GUI, app/launcher/updater/installer/installed target/relaunch, key/Keychain/GK/LS, service/account/task/global config/TLS/IPC/DNS/toolchain default, native input/clipboard, live sharing, 24800/5353, Docker, push/publication or external network fixture. Three provider-stopped tasks untouched. No tests/mocks/delegation/computer use.')
 save('failure-limits.json',limits)
 swiftstate={}
 for p in sorted((P/'swift').iterdir()):
  if p.is_file():swiftstate[p.name]=dict(path=str(p),sha256=sha(p),bytes=p.stat().st_size,metadata=meta(p))
 save('fresh-swift-state.json',swiftstate)
 samples=load('manifest-1-samples.json');rows=[r for sample in samples for r in sample['owned']]
 save('manifest-priority-coverage.json',dict(samples=len(samples),observations=len(rows),owned_pids=len({r['pid']for r in rows}),nice_values=sorted({r['nice']for r in rows}),swift_frontends=[r for r in rows if Path(r['comm']).name.strip('()')=='swift-frontend'],limits='Manifest compiler observations only; separate from failed package app build.'))
 save('final-preservation-audit.json',dict(all_initial_baselines_match=True,source_inputs=len(source_inventory()),old_cache_entries={n:len(v)for n,v in preservation()['old_cache'].items()},old_failed_guards_originals_and_ledgers_unchanged=True,package_guard_sha256=sha(P/'package-once.guard'),manifest_guard_sha256=sha(P/'manifest-1.guard'),package_controller_sha256=sha(P/'controller.py'),profile_sha256=sha(P/'package.sb'),scope='read-only postfailure audit; no new build or product execution'))
 ledger=json.loads((DEST/'copy-ledger.json').read_text())
 additions=['copy-ledger-first.json','collect-first.py','collect-stdout-before-completion','first-export-audit.json','failure-limits.json','fresh-swift-state.json','manifest-priority-coverage.json','final-preservation-audit.json','final-audit.py']
 for name in additions:ledger.append(export(P/name))
 ledger.append(export(P/'collect.stdout','collect-final.stdout'))
 # Retain exact fresh build-state files as evidence. These are new, not old clones.
 for name in swiftstate:ledger.append(export(P/'swift'/name,'fresh-swift/'+name))
 with (DEST/'copy-ledger.json').open('w') as f:json.dump(ledger,f,indent=2,sort_keys=True);f.write('\n')
 checks=[]
 for i in ledger:
  original=Path(i['private_original']).read_bytes();data=(DEST/i['path']).read_bytes()
  assert digest(original)==i['original_sha256'] and len(original)==i['original_bytes'],i['path']
  assert digest(data)==i['exported_sha256'] and len(data)==i['exported_bytes'],i['path']
  payload=gzip.decompress(data) if i['path'].endswith('.gz') else data
  assert digest(payload)==i['payload_sha256'],i['path']
  checks.append(i['path'])
 audit=dict(checked_exports=len(checks),all_private_originals_match=True,all_exported_hashes_match=True,all_payload_hashes_match=True,ledger_sha256=sha(DEST/'copy-ledger.json'),first_export_audit_issue_preserved='An active stdout prefix export was rebound to an immutable exact prefix original; first collector/ledger and final stdout retained. No product output or failure changed.')
 save('export-audit.json',audit)
 with (DEST/'export-audit.json').open('x') as f:json.dump(audit,f,indent=2,sort_keys=True);f.write('\n')
 print(json.dumps(audit),flush=True)
if __name__=='__main__':
 os.umask(0o077)
 main()
