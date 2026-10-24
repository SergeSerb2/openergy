import errno,json,os,pathlib,socket,sys,tempfile,time
root=pathlib.Path(tempfile.mkdtemp(prefix='oe-refusal-owner-',dir='/tmp'));root.chmod(0o700)
records=[]
for family,host in [(socket.AF_INET,'127.0.0.1'),(socket.AF_INET6,'::1')]:
 for label,how in [('read',socket.SHUT_RD),('write',socket.SHUT_WR),('both',socket.SHUT_RDWR)]:
  owner=socket.socket(family,socket.SOCK_STREAM);other=None;client=None
  r={'family':family.name,'host':host,'shutdown':label}
  try:
   owner.setsockopt(socket.SOL_SOCKET,socket.SO_REUSEADDR,0)
   if hasattr(socket,'SO_REUSEPORT'):owner.setsockopt(socket.SOL_SOCKET,socket.SO_REUSEPORT,0)
   if family==socket.AF_INET6:owner.setsockopt(socket.IPPROTO_IPV6,socket.IPV6_V6ONLY,1)
   owner.bind((host,0));owner.listen(1);address=owner.getsockname();r['port']=address[1];assert address[1] not in (24800,5353)
   try:owner.shutdown(how);r['shutdown_errno']=0
   except OSError as e:r['shutdown_errno']=e.errno
   other=socket.socket(family,socket.SOCK_STREAM)
   if family==socket.AF_INET6:other.setsockopt(socket.IPPROTO_IPV6,socket.IPV6_V6ONLY,1)
   try:other.bind(address);r['competing_bind_errno']=0
   except OSError as e:r['competing_bind_errno']=e.errno
   other.close();other=None
   if r['competing_bind_errno']!=errno.EADDRINUSE:
    r['qualification']='UNVERIFIED: endpoint exclusivity not established; no connect attempted';records.append(r);continue
   client=socket.socket(family,socket.SOCK_STREAM);client.settimeout(.5);begin=time.monotonic()
   try:client.connect(address);r['connect_result']='Connected';r['connect_errno']=0
   except OSError as e:r['connect_result']=type(e).__name__;r['connect_errno']=e.errno
   r['connect_elapsed_seconds']=time.monotonic()-begin
   r['qualification']='QUALIFIED_ACTUAL_REFUSAL' if r['connect_errno']==errno.ECONNREFUSED else 'UNVERIFIED: actual refusal not observed'
   records.append(r)
  except OSError as e:
   r['setup_error_errno']=e.errno;records.append(r)
  finally:
   if client:client.close()
   if other:other.close()
   owner.close()
result={'scope':'real OS endpoint ownership/refusal qualification only; no product/installer run','os':os.uname().sysname,'records':records,'all_owned_sockets_closed':True,'product_source_changed':False}
(root/'result.json').write_text(json.dumps(result,indent=2)+'\n');pathlib.Path('/tmp/oe-owned-refusal-qualification-20261023-state.json').write_text(json.dumps({'root':str(root)},indent=2)+'\n');print(json.dumps(result,indent=2));print('Private record:',root,flush=True)
