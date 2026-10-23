import pathlib,json,re,hashlib
p=pathlib.Path(json.loads(pathlib.Path('/private/tmp/oe-win-dns-8281eb7-private-pointer.json').read_text())['local'])
raw=p/'raw'
def sha(f):return hashlib.sha256(f.read_bytes()).hexdigest()
def analyze(name):
 f=raw/'logs'/f'{name}.stdout';lines=f.read_text().splitlines();obs={};mode=None;base_events={'queue':[],'lifecycle':[]}
 for line in lines:
  if line.startswith('BASE_TRANSCRIPT_START driver='):mode=line.split('=',1)[1]
  if line.startswith('BASE_EVENT ') and mode:base_events[mode].append(line)
  if line.startswith('BASE_TRANSCRIPT driver='):mode=None
  if line.startswith('TIMING '):
   key=re.search(r'case=(\S+)',line)[1];obs[key]={'timing_line':line,'calls':[],'events':[],'traces':[]}
   for field in ['lower_deadline_ns','upper_deadline_ns','observation_ns','peer_bytes']:obs[key][field]=int(re.search(rf'{field}=(\d+)',line)[1])
   obs[key]['trace_overflow']='trace_overflow=true' in line
  m=re.fullmatch(r'CALL case=(\S+) index=(\d+) op=(\S+) enter_ns=(\d+) return_ns=(\d+)',line)
  if m:obs[m[1]]['calls'].append(dict(index=int(m[2]),op=m[3],enter_ns=int(m[4]),return_ns=int(m[5])))
  m=re.fullmatch(r'EVENT case=(\S+) call=(\d+) value=(.*)',line)
  if m:obs[m[1]]['events'].append(dict(call=int(m[2]),literal_event=m[3]))
  m=re.fullmatch(r'TRACE case=(\S+) at_ns=(\d+) (.*)',line)
  if m:obs[m[1]]['traces'].append(dict(at_ns=int(m[2]),literal_trace=m[3]))
 for o in obs.values():
  c=o['calls'];assert c and [x['index'] for x in c]==list(range(len(c)))
  assert c[0]['op']=='execute_checked' and all(x['op']=='poll' for x in c[1:])
  assert all(x['enter_ns']<=x['return_ns'] for x in c)
  assert all(a['return_ns']<=b['enter_ns'] for a,b in zip(c,c[1:]))
  o['last_call_return_ns']=c[-1]['return_ns'];o['full_observation_retained']=c[-1]['return_ns']>=o['observation_ns']==6000000000
  o['event_count']=len(o['events']);o['call_count']=len(c);o['trace_count']=len(o['traces'])
  eligible=next((x['index'] for x in c[1:] if x['enter_ns']>=o['upper_deadline_ns']),None);o['first_poll_known_after_upper_deadline']=eligible
  for e in o['events']:
   call=c[e['call']];e['receipt_interval']={k:call[k] for k in ['enter_ns','return_ns']}
   e['before_lower_deadline']=call['return_ns']<o['lower_deadline_ns'];e['enters_at_or_after_upper_deadline']=call['enter_ns']>=o['upper_deadline_ns'];e['after_first_eligible_poll']=eligible is not None and e['call']>eligible
  del o['calls']
 queue=[l for l in lines if l.startswith('QUEUE ')];frames=[l for l in lines if l.startswith('BASE_FRAME ')];recoveries=[l for l in lines if l.startswith('RECOVERY ')];matrix=[l for l in lines if l.startswith('REFUSAL ')];os_controls=[l for l in lines if l.startswith('OS_CONTROL ')]
 return {'name':name,'raw_stdout_sha256':sha(f),'literal_result':[l for l in lines if l.startswith('RESULT ')],'queue_pass_batches':sum('verdict=PASS' in l for l in queue),'queue_skipped_batches':sum('executed=false' in l for l in queue),'obsolete_failure_events_in_queue_transcript':sum('value=ConnectFailed ' in l and ('error: "shutdown"' in l or 'error: "connect_timeout:' in l) for l in base_events['queue']),'base_frame_exchanges':len(frames),'same_caller_recovery_exchanges':len(recoveries),'base_event_counts':{k:len(v) for k,v in base_events.items()},'refusal_matrix_verdict_counts':{v:sum(f'verdict={v}' in l for l in matrix) for v in ['PASS','FAIL','UNVERIFIED']},'os_control_groups':{label:{'attempts':sum(f'label={label}' in l for l in os_controls),'actual_in_budget_refusals':sum(f'label={label}' in l and 'actual_refusal=true within_budget=true' in l for l in os_controls)} for label in ['strict_short_observation','bounded_fixture_reference']},'literal_flow_lines':[l for l in lines if re.match(r'^(RESULT|QUEUE|BASE_CASE|BASE_FRAME|LIFECYCLE|CONTRACTS|RECOVERY|REFUSAL|DEADLINE|FIXTURE |FIXTURE_SETUP|EXCLUSIVE_BIND|TLS_HANDOFF|OS_CONTROL|BASE_OWNER|BASE_RESOLVER|RESOLVER)',l)],'observations':obs}
results=[analyze(n) for n in ['normal-full','owned-conflict-full']]
(p/'derived-runtime.json').write_text(json.dumps(results,indent=2)+'\n')
for d in results:
 print(json.dumps({k:v for k,v in d.items() if k not in ['observations','literal_flow_lines']},indent=2))
 print('Observation count/full six-second windows:',len(d['observations']),sum(o['full_observation_retained'] for o in d['observations'].values()))
