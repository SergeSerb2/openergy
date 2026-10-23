from pathlib import Path
import hashlib,importlib.util,json,os,plistlib,subprocess
os.umask(0o077)
root=Path(json.loads(Path('/tmp/oe-stopped-update-fixtures-20261023-state.json').read_text())['root'])
state=json.loads(Path('/tmp/oe-sparkle-combined-20261023-state.json').read_text());historical=Path(state['root']);tool=historical/'swift/artifacts/sparkle/Sparkle/bin/sign_update';official=historical/'swift/artifacts/sparkle/Sparkle/Sparkle.xcframework/macos-arm64_x86_64/Sparkle.framework'
records=json.loads((root/'commands.json').read_text());assert len(records)==14 and all(x['exit']==(1 if x['label']=='reject-wrong-archive-signature' else 0) for x in records)
assert not(root/'ephemeral-seed').exists();assert json.loads((root/'key-cleanup.json').read_text())['ephemeral_seed_removed']
info=plistlib.loads((root/'old/Openergy.app/Contents/Info.plist').read_bytes());identifier=info['CFBundleIdentifier']
manifest={'fixture_root':str(root),'bundle_identifier':identifier,'old_build':'1','candidate_build':'2','public_key':info['SUPublicEDKey'],'fixture_key_source':'new ephemeral CryptoKit key; production signing key never read','expected_jobs':[identifier+'-sparkle-updater',identifier+'-sparkle-progress'],'planned_loopback':'127.0.0.1:25600','server_started':False,'installer_executed':False,'candidate_or_old_app_executed':False,'relaunch_supported':False,'relaunch_limit':'stopped target only; no argument-free private bootstrap; neither target may be launched','archive_validation':'offline official verifier accepts archive signature, rejects altered signature; both full feeds verify','source_app':str(historical/'bundle-final/Openergy.app'),'metadata_recovery':'Initial controller exited1 only while naming the external signer in final metadata. All14 preparation commands finished with their expected exits. Key deletion ran in finally. No signing or packaging command repeated.','files':{str(p.relative_to(root)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'bytes':p.stat().st_size} for p in [root/'public/openergy-fixture.zip',root/'public/valid.xml',root/'public/signature-negative.xml',root/'old/Openergy.app/Contents/Info.plist',root/'candidate/Openergy.app/Contents/Info.plist']},'signer':{'path':str(tool),'sha256':hashlib.sha256(tool.read_bytes()).hexdigest()}}
(root/'preparation.json').write_text(json.dumps(manifest,indent=2)+'\n')
cli=root/'driver/sparkle.app';original=Path('/private/tmp/oe-sparkle-cli-build-njko0i6t/products/sparkle.app');log=[]
def run(label,args):
 p=subprocess.run(['nice','-n','15',*map(str,args)],capture_output=True,text=True,timeout=60);log.append({'label':label,'arguments':list(map(str,args)),'exit':p.returncode});(root/'driver-preparation-commands.json').write_text(json.dumps(log,indent=2)+'\n')
 if p.returncode:raise RuntimeError(label+' failed; tool diagnostics suppressed')
 return p.stdout
run('copy-unaccepted-cli',['ditto',original,cli])
framework=cli/'Contents/Frameworks/Sparkle.framework'
import shutil
assert framework.resolve().is_relative_to(root.resolve());shutil.rmtree(framework)
run('copy-official-framework',['ditto',official,framework])
p=cli/'Contents/Info.plist';ci=plistlib.loads(p.read_bytes());ci['CFBundleIdentifier']=identifier+'.driver';ci['SUDefaultsDomain']=str(root/'config/driver-preferences');p.write_bytes(plistlib.dumps(ci))
run('local-adhoc-cli-sign',['codesign','--force','--options','0','--timestamp=none','--sign','-',cli]);run('verify-cli-bundle-seal',['codesign','--verify','--deep','--strict',cli]);run('verify-arm64',['lipo','-verify_arch','arm64',cli/'Contents/MacOS/sparkle'])
loads=run('inspect-cli-linkage',['otool','-L',cli/'Contents/MacOS/sparkle']);assert '@rpath/Sparkle.framework' in loads
binary=cli/'Contents/MacOS/sparkle';original_binary=original/'Contents/MacOS/sparkle'
# codesign changes the Mach-O signature; the code payload provenance stays in the original build manifest.
driver={'source_cli':str(original),'source_cli_binary_sha256':hashlib.sha256(original_binary.read_bytes()).hexdigest(),'prepared_cli':str(cli),'prepared_cli_binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'framework_source':str(official),'framework_executable_sha256':hashlib.sha256((framework/'Versions/B/Sparkle').read_bytes()).hexdigest(),'bundle_identifier':ci['CFBundleIdentifier'],'resource_seal_verify_exit':0,'framework_changed_in_private_copy_only':True,'xcode_invoked':False,'cli_executed':False,'installer_executed':False,'trust':'local ad-hoc, not DeveloperID/notarized or runtime accepted'}
(root/'driver-preparation.json').write_text(json.dumps(driver,indent=2)+'\n')
future={'status':'DO NOT RUN: fixture helper/cache effects need scope approval; no running-target/relaunch support','negative_command':[str(binary),str(root/'old/Openergy.app'),'--check-immediately','--feed-url','http://127.0.0.1:25600/signature-negative.xml','--user-agent-name','Openergy-private-verification','--verbose'],'positive_command':[str(binary),str(root/'old/Openergy.app'),'--check-immediately','--feed-url','http://127.0.0.1:25600/valid.xml','--user-agent-name','Openergy-private-verification','--verbose'],'required_preflight':'Fresh non-running fixture ID; exclusive owned loopbackserver; exact file hashes and private target parent; inventory fixture job/cache paths; no installed app or global settings touched','future_environment':{'CFFIXED_USER_HOME':str(root/'config'),'TMPDIR':str(root/'config')+'/'},'scope':'complete download and stopped-bundle signature rejection/replacement only; no target app execution or relaunch'}
(root/'operation-DO-NOT-RUN.json').write_text(json.dumps(future,indent=2)+'\n')
print(json.dumps({'root':str(root),'fixture_identifier':identifier,'archive_bytes':manifest['files']['public/openergy-fixture.zip']['bytes'],'offline_archive_and_feed_signature_checks':'PASS','prepared_cli_resource_seal':'PASS','cli_or_installer_executed':False,'ephemeral_seed_removed':True}),flush=True)
