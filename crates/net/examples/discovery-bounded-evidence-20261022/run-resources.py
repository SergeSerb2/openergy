import pathlib, subprocess, time, json
root=pathlib.Path('/tmp/oe-bounded-discovery-20261022-5a69')
with (root/'patched-probe-final.log').open('w') as out:
 child=subprocess.Popen(['/usr/bin/time','-l','nice','-n','15','target/debug/examples/discovery_recovery','probe',str(root/'patched-probe-final'),'25090','25091'],stdout=out,stderr=subprocess.STDOUT)
 try:
  time.sleep(10)
  processes=subprocess.check_output(['/bin/ps','-axo','pid=,ppid=,comm='],text=True)
  candidates=[int(line.split()[0]) for line in processes.splitlines() if len(line.split())>=3 and line.split()[1]==str(child.pid) and 'discovery_recovery' in line]
  if len(candidates)!=1: raise RuntimeError(f'probe PID mapping failed: {candidates}')
  pid=candidates[0]
  threads=subprocess.check_output(['/bin/ps','-M','-p',str(pid)],text=True)
  stats=subprocess.check_output(['/bin/ps','-p',str(pid),'-o','pid,rss,time,%cpu,command'],text=True)
  udp=subprocess.run(['/usr/sbin/lsof','-w','-nP','-a','-p',str(pid),'-iUDP','-Fpn'],capture_output=True,text=True,check=True).stdout
  (root/'retained-process-resources.txt').write_text(threads+'\n'+stats+'\nUDP:\n'+udp)
  report={'pid':pid,'threads':len(threads.strip().splitlines())-1,'udp_descriptors':len([line for line in udp.splitlines() if line.startswith('n')]),'rss_kib':int(stats.strip().splitlines()[-1].split()[1]),'sample_seconds_after_start':10}
  (root/'retained-process-resources.json').write_text(json.dumps(report,indent=2)+'\n')
  print(json.dumps(report),flush=True)
  code=child.wait(timeout=70);print('probe_exit_status='+str(code),flush=True)
  raise SystemExit(code)
 finally:
  if child.poll() is None:
   child.kill();child.wait()
