from transport import *
import tarfile,io
commit='8281eb79f54725ddc8fe8bae83dbd0f44fa5d885'
assert subprocess.check_output(['git','rev-parse','HEAD']).decode().strip()==commit
assert subprocess.check_output(['git','show',commit+':.gitattributes'])==subprocess.check_output(['git','show','HEAD:.gitattributes'])
archive=ROOT/'source.tar';subprocess.run(['git','archive','--format=tar','--output',str(archive),commit],check=True)
items=[]
for row in subprocess.check_output(['git','ls-tree','-r','-z',commit]).split(b'\0'):
 if not row:continue
 head,name=row.split(b'\t',1);mode,kind,oid=head.decode().split();assert kind=='blob' and mode in ['100644','100755'];items.append((name.decode(),oid))
batch=subprocess.run(['git','cat-file','--batch'],input=''.join(oid+'\n' for _,oid in items).encode(),stdout=subprocess.PIPE,check=True).stdout
stream=io.BytesIO(batch);blobs={}
for path,oid in items:
 header=stream.readline().decode().split();assert header[0]==oid and header[1]=='blob';data=stream.read(int(header[2]));assert stream.read(1)==b'\n';blobs[path]=(oid,data)
files=[];conversions=[]
with tarfile.open(archive) as tar:
 for member in tar:
  if member.isdir():continue
  assert member.isfile() and not member.name.startswith('/') and '..' not in pathlib.PurePosixPath(member.name).parts
  data=tar.extractfile(member).read();oid,blob=blobs[member.name]
  if data!=blob:
   assert member.name.endswith('.cmd') and data.replace(b'\r\n',b'\n')==blob
   clean=subprocess.check_output(['git','hash-object','--stdin','--path='+member.name],input=data).decode().strip();assert clean==oid
   conversions.append({'path':member.name,'rule':'*.cmd text eol=crlf','git_bytes':len(blob),'archive_bytes':len(data),'git_sha256':hashlib.sha256(blob).hexdigest(),'archive_sha256':hashlib.sha256(data).hexdigest(),'git_clean_blob':clean,'matches':True})
  files.append({'path':member.name,'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest(),'git_blob':oid,'git_blob_bytes':len(blob),'git_blob_sha256':hashlib.sha256(blob).hexdigest()})
assert sorted(x['path'] for x in files)==sorted(blobs)
manifest={'schema_version':1,'commit':commit,'included_parent':'9a4ac2eed77a8ed8bddd61e060be14a506df570a','archive_sha256':sha(archive),'regular_files':len(files),'git_exact_byte_matches':len(files)-len(conversions),'checkout_conversions':conversions,'files':files}
(ROOT/'source-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
