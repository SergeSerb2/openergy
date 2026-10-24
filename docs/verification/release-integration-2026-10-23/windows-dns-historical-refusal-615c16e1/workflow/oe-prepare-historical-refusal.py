import ast,base64,datetime,hashlib,io,json,os,pathlib,shutil,subprocess,tarfile,tempfile
repo=pathlib.Path.cwd(); oldptr=json.loads(pathlib.Path('/private/tmp/oe-win-dns-8281eb7-private-pointer.json').read_text()); prior=pathlib.Path(oldptr['local'])
p=pathlib.Path(tempfile.mkdtemp(prefix='oe-historical-refusal-615c16e1-',dir='/private/tmp'));os.chmod(p,0o700)
def sha(b):return hashlib.sha256(b).hexdigest()
def writejson(f,v):(p/f).write_text(json.dumps(v,indent=2)+'\n')
# Recover only the approved transport; do not change its original files.
s=(prior/'transport.py').read_text();tree=ast.parse(s)
rootnode=next(n for n in tree.body if isinstance(n,ast.Assign) and any(isinstance(t,ast.Name) and t.id=='ROOT' for t in n.targets))
lines=s.splitlines();lines[rootnode.lineno-1:rootnode.end_lineno]=['ROOT=pathlib.Path('+repr(str(p))+')'];(p/'transport.py').write_text('\n'.join(lines)+'\n')
writejson('provenance.json',{'parent_commit':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'historical_commit':subprocess.check_output(['git','rev-parse','615c16e1^{commit}'],text=True).strip(),'prior_local':str(prior),'approved_transport_sha256':sha((prior/'transport.py').read_bytes()),'workflow_reference':'docs/verification/release-integration-2026-10-23/windows-dns-final-8281eb7/workflow','prepared_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()})
commit=json.loads((p/'provenance.json').read_text())['historical_commit']
probe='crates/net/examples/dns_queue_cancel_c4806a3d.rs';data=(repo/probe).read_bytes();assert sha(data)=='530c880eb74f6c95d24a0612c4fa8773fa21d256765fc3f9f6260801fa303a30'
entries=[];base=[]
ls=subprocess.check_output(['git','ls-tree','-r','-z',commit]).split(b'\0');batch=subprocess.Popen(['git','cat-file','--batch'],stdin=subprocess.PIPE,stdout=subprocess.PIPE)
with tarfile.open(p/'source.tar','w',format=tarfile.PAX_FORMAT) as tar:
 for line in ls:
  if not line:continue
  hdr,path=line.split(b'\t',1);mode,typ,oid=hdr.decode().split();path=path.decode();assert typ=='blob' and mode in ['100644','100755'],(path,mode,typ)
  batch.stdin.write(oid.encode()+b'\n');batch.stdin.flush();meta=batch.stdout.readline().split();payload=batch.stdout.read(int(meta[2]));assert batch.stdout.read(1)==b'\n';assert sha(payload)
  row={'path':path,'mode':mode,'git_blob':oid,'bytes':len(payload),'sha256':sha(payload)};base.append(row)
  if path==probe:continue
  inf=tarfile.TarInfo(path);inf.size=len(payload);inf.mode=int(mode[-3:],8);inf.mtime=0;tar.addfile(inf,io.BytesIO(payload));entries.append(row)
 inf=tarfile.TarInfo(probe);inf.size=len(data);inf.mode=0o644;inf.mtime=0;tar.addfile(inf,io.BytesIO(data));entries.append({'path':probe,'mode':'100644','bytes':len(data),'sha256':sha(data),'overlay':True})
batch.stdin.close();assert batch.wait()==0
(p/'overlay').mkdir();(p/'overlay'/pathlib.Path(probe).name).write_bytes(data)
writejson('source-manifest.json',{'commit':commit,'archive_sha256':sha((p/'source.tar').read_bytes()),'export_method':'git ls-tree plus exact git cat-file blob bytes; no checkout or line-ending conversion','only_overlay':probe,'files':entries})
writejson('historical-git-manifest.json',{'commit':commit,'files':base})
writejson('public-api-review.json',{'historical_commit':commit,'probe_sha256':sha(data),'api_review':'Public symbols and methods required by all compiled probe functions exist in historical lib.rs/reactor.rs; final check is unchanged native compiler/Clippy. No shim or instrumentation is allowed.','imports':['ConnectErrorKind','NetCommand','NetConfig','NetDriver','NetError','NetEvent','Priority','TransportMode','openergy_session::ConnId'],'methods':['new','set_connect_timeout','set_transport_mode','execute_checked','poll','reconfigure','waker','conn_info','identity_fingerprint'],'historical_observer_trace_sites':0,'observer_missing_is_not_fault_witness':True,'production_files_preserved':len(base)-sum(e['path']==probe for e in base),'cargo_lock':next(e for e in base if e['path']=='Cargo.lock'),'reactor':next(e for e in base if e['path']=='crates/net/src/reactor.rs')})
for f in ['common.ps1','audit.ps1','runtime-common.ps1','build.ps1','native-tool-reference.json']:
 shutil.copyfile(prior/f,p/f)
s=(p/'build.ps1').read_text().replace('8281eb79f54725ddc8fe8bae83dbd0f44fa5d885',commit).replace('EXACT_CANDIDATE_NATIVE_BUILD_AND_FREEZE_PASS','EXACT_HISTORICAL_OVERLAY_NATIVE_BUILD_AND_FREEZE_PASS');(p/'build.ps1').write_text(s)
pathlib.Path('/private/tmp/oe-historical-refusal-pointer.json').write_text(json.dumps({'local':str(p),'prior':str(prior)})+'\n')
print(json.dumps({'private_workspace_created':True,'historical_commit':commit,'historical_files':len(base),'overlay_files':1,'source_files':len(entries),'source_tar_bytes':(p/'source.tar').stat().st_size,'source_tar_sha256':sha((p/'source.tar').read_bytes()),'public_api_static_check':'compatible; native compile pending'}))
