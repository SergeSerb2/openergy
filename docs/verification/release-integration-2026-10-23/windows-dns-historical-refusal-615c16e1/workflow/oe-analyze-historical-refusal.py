import collections,hashlib,json,pathlib,re
p=pathlib.Path(json.loads(pathlib.Path('/private/tmp/oe-historical-refusal-pointer.json').read_text())['local']);raw=p/'raw';f=raw/'logs/historical-refusal.stdout'
if not f.exists():
 result={'matching_historical_refusal_gate':'UNVERIFIED','historical_fault_witness':False,'reason':'No historical refusal output; build/setup failure is not a behavior witness.'}
else:
 text=f.read_text(encoding='utf-8-sig');lines=text.splitlines();observations={}
 for line in lines:
  m=re.match(r'(TIMING|CALL|EVENT|TRACE) case=(\S+) (.*)',line)
  if not m:continue
  typ,case,body=m.groups();o=observations.setdefault(case,{'calls':[],'events':[],'traces':[]})
  if typ=='TIMING':o['timing']=dict(re.findall(r'(\w+)=(\S+)',body))
  elif typ=='CALL':o['calls'].append({k:int(v) if v.isdigit() else v for k,v in re.findall(r'(\w+)=(\S+)',body)})
  elif typ=='EVENT':
   em=re.match(r'call=(\d+) value=(.*)',body);o['events'].append({'call':int(em[1]),'value':em[2]})
  else:o['traces'].append(body)
 fixtures={int(m[1]):m[2]=='true' for line in lines if (m:=re.match(r'FIXTURE port=(\d+) qualified=(true|false)',line))}
 matrix=[dict(re.findall(r'(\w+)=(\S+)',line)) for line in lines if line.startswith('REFUSAL ')]
 for row in matrix:
  if row['case'] in observations:observations[row['case']]['literal_refusal']=row
 for case,o in observations.items():
  calls=o['calls'];timing=o['timing'];o['full_six_second_window']=int(timing['observation_ns'])==6000000000 and calls[-1]['return_ns']>=6000000000
  o['call_count']=len(calls);o['event_count']=len(o['events']);o['trace_count']=len(o['traces'])
  for e in o['events']:e['receipt_call']=calls[e['call']]
 wrong=[{'case':case,'event':e,'strict_verdict':o.get('literal_refusal',{}).get('verdict'),'qualified':o.get('literal_refusal',{}).get('qualified')} for case,o in observations.items() for e in o['events'] if e['value'].startswith('Connected {') and o.get('literal_refusal',{}).get('qualified')=='true' and o.get('literal_refusal',{}).get('verdict')=='FAIL']
 run=json.loads((raw/'historical-refusal-result.json').read_text(encoding='utf-8-sig'))
 witness=bool(wrong) and run['exit']==1 and not run['timed_out']
 result={'matching_historical_refusal_gate':'CLOSED_REAL_WRONG_BEHAVIOR_REJECTED' if witness else 'UNVERIFIED','historical_fault_witness':witness,'raw_stdout_sha256':hashlib.sha256(f.read_bytes()).hexdigest(),'exit':run['exit'],'elapsed_ms':run['elapsed_ms'],'timed_out':run['timed_out'],'forced_cleanup':run['forced_cleanup'],'fixture_qualification':fixtures,'refusal_matrix_counts':dict(collections.Counter(row['verdict'] for row in matrix)),'false_connected_count':len(wrong),'false_connected_witnesses':wrong,'literal_result':[line for line in lines if line.startswith('RESULT ')],'os_controls':[line for line in lines if line.startswith('OS_CONTROL ')],'exclusive_owners':[line for line in lines if line.startswith('EXCLUSIVE_BIND ')],'observation_count':len(observations),'full_six_second_observations':sum(o['full_six_second_window'] for o in observations.values()),'product_trace_count':sum(len(o['traces']) for o in observations.values()),'missing_trace_is_fault_witness':False,'observations':observations,'scope':'Historical refusal controls only. No strict TCP/fallback/TLS timing or full acceptance.'}
(p/'derived-runtime.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({k:v for k,v in result.items() if k not in ['observations','false_connected_witnesses','os_controls','exclusive_owners']},indent=2))
