"""Audit diagnostic before/after cases from retained raw records, without reruns."""
import argparse,json,statistics
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--work-dir',type=Path,default=Path(__file__).resolve().parent);W=p.parse_args().work_dir
out={};content={}
for stem,total,repeats,warmups,anki_expected in [('pilot',40,3,1,10),('pilot2',24,3,1,6),('confirmation',130,10,3,10)]:
 data=json.loads((W/f'{stem}-results.json').read_text());assert data['status']=='completed';rows=data['rows'];assert len(rows)==total;seen=set();oracles=0
 for r in rows:
  key=(r['role'],r['repeat'],r['profile'],r['mode']);assert key not in seen;seen.add(key);case=W/(stem+'-'+'-'.join(map(str,key)))
  m=json.loads((case/'measurement.json').read_text());v=json.loads((case/'verification.json').read_text())
  assert m['reaped'] and not any(m[k] for k in ['spawn_error','exit_code','signal','interrupted_signal','leftover_descendants'])
  assert v['status']==v['physical']['status']==v['semantic']['status']=='passed';assert r['elapsed_ms']==m['elapsed_ns']/1e6 and r['rss_mib']==m['peak_rss_bytes']/2**20
  assert r['logical_sha256']==v['physical']['logical_sha256'];content.setdefault(r['profile'],set()).add(r['logical_sha256'])
  assert r['power_before'].splitlines()[0]==r['power_after'].splitlines()[0]=="Now drawing from 'AC Power'"
  assert r['oracle_passed']==(case/'anki.json').exists()
  if r['oracle_passed']:assert r['role']=='timing' and r['repeat']==0 and json.loads((case/'anki.json').read_text())['status']=='passed';oracles+=1
 assert oracles==anki_expected
 summary=json.loads((W/f'{stem}-summary.json').read_text())
 for c in summary:
  for mode in ['before','after']:
   selected=[r for r in rows if r['role']=='timing' and r['profile']==c['profile'] and r['mode']==mode];warm=[r for r in rows if r['role']=='warmup' and r['profile']==c['profile'] and r['mode']==mode]
   assert len(selected)==repeats and len(warm)==warmups
   assert c[mode]['samples_ms']==[r['elapsed_ms'] for r in selected]
   assert c[mode]['median_ms']==statistics.median(r['elapsed_ms'] for r in selected)
   assert c[mode]['rss_mib']==statistics.median(r['rss_mib'] for r in selected)
 if stem=='confirmation':
  counts={}
  for i in range(0,len(rows),2):
   a,b=rows[i:i+2];assert all(a[k]==b[k] for k in ['role','repeat','profile']);assert {a['mode'],b['mode']}=={'before','after'}
   if a['role']=='timing':counts[(a['profile'],a['mode'])]=counts.get((a['profile'],a['mode']),0)+1
  assert len(counts)==10 and set(counts.values())=={5}
 assert json.loads((W/f'{stem}-source-before.json').read_text())==json.loads((W/f'{stem}-source-after.json').read_text())
 out[stem]={'exports':total,'anki_checks':oracles,'status':'passed','summaries_recomputed':True}
assert all(len(s)==1 for s in content.values())
print(json.dumps({'status':'passed','exports':194,'anki_checks':26,'learning_content_equal':True,'groups':out},indent=2))
