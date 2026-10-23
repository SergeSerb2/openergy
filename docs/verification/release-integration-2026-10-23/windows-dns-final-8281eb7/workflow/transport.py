import base64,datetime,hashlib,json,pathlib,subprocess,time
ROOT=pathlib.Path('<LOCAL_PRIVATE>')
def sha(p): return hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
def remote(label,script,timeout=55,exe='powershell.exe'):
    source=ROOT/(label+'.input.ps1');source.write_text(script)
    argv=['ssh','-o','BatchMode=yes','-o','ConnectTimeout=8','<SSH_ALIAS>',exe,'-NoLogo','-NoProfile','-NonInteractive','-EncodedCommand',base64.b64encode(script.encode('utf-16le')).decode()]
    t=time.monotonic();started=datetime.datetime.now(datetime.timezone.utc).isoformat()
    try:
        p=subprocess.run(argv,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=timeout)
        out,err,code,unknown=p.stdout,p.stderr,p.returncode,p.returncode==255
    except subprocess.TimeoutExpired as e: out,err,code,unknown=e.stdout or b'',e.stderr or b'',None,True
    (ROOT/(label+'.stdout')).write_bytes(out);(ROOT/(label+'.stderr')).write_bytes(err)
    result={'started_utc':started,'elapsed_ms':round((time.monotonic()-t)*1000),'transport_exit':code,'remote_exit_unknown':unknown,'script_sha256':sha(source),'stdout_sha256':sha(ROOT/(label+'.stdout')),'stderr_sha256':sha(ROOT/(label+'.stderr'))}
    (ROOT/(label+'.transport.json')).write_text(json.dumps(result,indent=2)+'\n')
    return result,out.decode('utf-8-sig',errors='replace')
