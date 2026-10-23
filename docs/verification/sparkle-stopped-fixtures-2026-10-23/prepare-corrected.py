from pathlib import Path
import base64,hashlib,importlib.util,json,os,plistlib,subprocess,tempfile,uuid,xml.etree.ElementTree as ET
os.umask(0o077)
__import__("sys").dont_write_bytecode=True
state=json.loads(Path('/tmp/oe-sparkle-combined-20261023-state.json').read_text());historical=Path(state['root']);source=historical/'bundle-final/Openergy.app';tool=historical/'swift/artifacts/sparkle/Sparkle/bin/sign_update'
root=Path(tempfile.mkdtemp(prefix='oe-stopped-update-',dir='/private/tmp'));root.chmod(0o700)
Path('/tmp/oe-stopped-update-fixtures-20261023-state.json').write_text(json.dumps({'root':str(root),'source_app':str(source),'installer_executed':False},indent=2)+'\n')
for n in ['old','candidate','config','public','module-cache']:(root/n).mkdir(mode=0o700)
identifier='dev.openergy.updater-fixture.'+uuid.uuid4().hex
feed='http://127.0.0.1:25600/valid.xml';archive_url='http://127.0.0.1:25600/openergy-fixture.zip'
records=[]
env={k:v for k,v in os.environ.items() if not any(x in k.upper() for x in ['TOKEN','SECRET','PASSWORD','PRIVATE_KEY','API_KEY'])};env.update(TMPDIR=str(root)+'/',CLANG_MODULE_CACHE_PATH=str(root/'module-cache'),SWIFT_MODULECACHE_PATH=str(root/'module-cache'))
def run(label,args,expected=0):
 p=subprocess.run(['nice','-n','15',*map(str,args)],env=env,capture_output=True,text=True,timeout=180)
 records.append({'label':label,'arguments':list(map(str,args)),'exit':p.returncode});(root/'commands.json').write_text(json.dumps(records,indent=2)+'\n')
 if p.returncode!=expected:raise RuntimeError(label+' failed; captured tool diagnostics suppressed')
 return p.stdout.strip()
spec=importlib.util.spec_from_file_location('openergy_sparkle',Path('tools/release/sparkle.py'));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
key=root/'ephemeral-seed';verifier=root/'public-verifier';gen=root/'generate-key.swift'
gen.write_text('''import Foundation
import CryptoKit
let root = URL(fileURLWithPath: CommandLine.arguments[1])
let key = Curve25519.Signing.PrivateKey()
try Data(key.rawRepresentation.base64EncodedString().utf8).write(to: root.appendingPathComponent("ephemeral-seed"))
try Data(key.publicKey.rawRepresentation.base64EncodedString().utf8).write(to: root.appendingPathComponent("public-key.txt"))
''')
try:
 run('generate-ephemeral-key',['swift','-module-cache-path',root/'module-cache',gen,root])
 assert key.stat().st_mode&0o777==0o600
 public=(root/'public-key.txt').read_text();pub=base64.b64decode(public,validate=True);assert len(pub)==32
 verifier.write_bytes(base64.b64encode(bytes(64)+pub));verifier.chmod(0o600)
 trees={}
 for folder,build in [('old','1'),('candidate','2')]:
  app=root/folder/'Openergy.app';run('copy-'+folder,['ditto',source,app]);p=app/'Contents/Info.plist';info=plistlib.loads(p.read_bytes());info.update(CFBundleIdentifier=identifier,CFBundleVersion=build,SUFeedURL=feed,SUPublicEDKey=public,SUDefaultsDomain=str(root/'config/sparkle-preferences'),SUEnableAutomaticChecks=False,SUAutomaticallyUpdate=False);p.write_bytes(plistlib.dumps(info))
  run('sign-'+folder,['codesign','--force','--options','0','--timestamp=none','--sign','-',app]);m.verify_app(app,['arm64','x86_64']);trees[folder]=m.tree_identity(app)
 payload=root/'public/openergy-fixture.zip';intermediate=root/'candidate.zip';run('archive-candidate',['ditto','-c','-k','--sequesterRsrc','--keepParent',root/'candidate/Openergy.app',intermediate]);m.stable_zip(intermediate,payload)
 unpacked=root/'roundtrip';run('extract-roundtrip',['ditto','-x','-k',payload,unpacked]);assert m.tree_identity(unpacked/'Openergy.app')==trees['candidate'];m.verify_app(unpacked/'Openergy.app',['arm64','x86_64'])
 signature=run('sign-archive',[tool,'--ed-key-file',key,'-p',payload]);raw=base64.b64decode(signature,validate=True);assert len(raw)==64
 run('verify-archive',[tool,'--verify','--ed-key-file',verifier,payload,signature]);wrong=bytearray(raw);wrong[0]^=1;wrong=base64.b64encode(wrong).decode()
 run('reject-wrong-archive-signature',[tool,'--verify','--ed-key-file',verifier,payload,wrong],expected=1)
 ET.register_namespace('sparkle',m.NS)
 for name,sig in [('valid.xml',signature),('signature-negative.xml',wrong)]:
  rss=ET.Element('rss',{'version':'2.0'});channel=ET.SubElement(rss,'channel');ET.SubElement(channel,'title').text='Openergy private installation fixture';ET.SubElement(channel,'link').text=feed;item=ET.SubElement(channel,'item');ET.SubElement(item,'title').text='Private candidate';ET.SubElement(item,'{'+m.NS+'}version').text='2';ET.SubElement(item,'{'+m.NS+'}shortVersionString').text='0.1.0';ET.SubElement(item,'{'+m.NS+'}minimumSystemVersion').text='15.0';ET.SubElement(item,'enclosure',{'url':archive_url,'length':str(payload.stat().st_size),'type':'application/octet-stream','{'+m.NS+'}edSignature':sig});ET.indent(rss);p=root/'public'/name;p.write_bytes(ET.tostring(rss,encoding='utf-8',xml_declaration=True)+b'\n');run('sign-feed-'+name,[tool,'--ed-key-file',key,p]);run('verify-feed-'+name,[tool,'--verify','--ed-key-file',verifier,p])
 (root/'bundle-trees.json').write_text(json.dumps(trees,indent=2)+'\n')
 manifest={'fixture_root':str(root),'bundle_identifier':identifier,'old_build':'1','candidate_build':'2','public_key':public,'fixture_key_source':'new ephemeral CryptoKit key; production signing key never read','expected_jobs':[identifier+'-sparkle-updater',identifier+'-sparkle-progress'],'planned_loopback':'127.0.0.1:25600','server_started':False,'installer_executed':False,'candidate_or_old_app_executed':False,'relaunch_supported':False,'relaunch_limit':'stopped target only; bundles have no argument-free private bootstrap and must not be launched','download_validation':'offline official verifier accepts archive signature, rejects changed signature; both full feeds verify','source_app':str(source),'files':{(str(p.relative_to(root)) if p.is_relative_to(root) else str(p)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'bytes':p.stat().st_size} for p in [payload,root/'public/valid.xml',root/'public/signature-negative.xml',root/'old/Openergy.app/Contents/Info.plist',root/'candidate/Openergy.app/Contents/Info.plist',tool]}}
 (root/'preparation.json').write_text(json.dumps(manifest,indent=2)+'\n')
 print(json.dumps({'root':str(root),'bundle_identifier':identifier,'archive_bytes':payload.stat().st_size,'archive_sha256':hashlib.sha256(payload.read_bytes()).hexdigest(),'offline_signature_checks':'PASS: archive valid and altered signature rejected; both feeds valid','installer_executed':False}),flush=True)
finally:
 key.unlink(missing_ok=True);verifier.unlink(missing_ok=True)
 (root/'key-cleanup.json').write_text(json.dumps({'ephemeral_seed_removed':not key.exists(),'public_verifier_removed':not verifier.exists(),'production_key_read':False},indent=2)+'\n')
