"""Offline audit of all retained diagnostic records; no exporters are launched."""
from pathlib import Path
import argparse,json,hashlib,statistics

def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def audit(W):
 records=[]; oracle=0;digests={};summaries={}
 for stem,expected,expected_anki in [('baseline',24,4),('control',180,30),('detail',12,2)]:
  data=json.loads((W/f'{stem}-results.json').read_text());assert data['status']=='completed';rows=data['rows'];assert len(rows)==expected
  seen=set();summary=[]
  for r in rows:
   key=(r['role'],r['repeat'],r['profile'],r['mode']);assert key not in seen;seen.add(key)
   p=W/(stem+'-'+ '-'.join(map(str,key)));m=json.loads((p/'measurement.json').read_text());v=json.loads((p/'verification.json').read_text())
   assert m['exit_code']==0 and not m['signal'] and not m['leftover_descendants'] and not m['spawn_error'] and m['reaped']
   assert v['status']=='passed' and r['elapsed_ms']==m['elapsed_ns']/1e6 and r['rss_mib']==m['peak_rss_bytes']/2**20
   assert r['logical_sha256']==v['physical']['logical_sha256'];digests.setdefault(r['profile'],set()).add(r['logical_sha256'])
   assert r['power_before'].splitlines()[0]==r['power_after'].splitlines()[0]=="Now drawing from 'AC Power'"
   if (p/'anki.json').exists():
    assert r['role']=='timing' and r['repeat']==0 and json.loads((p/'anki.json').read_text())['status']=='passed';oracle+=1
   if stem!='baseline' and r['mode']!='official':
    line=next(l for l in (p/'stderr.log').read_text().splitlines() if l.startswith('[DEBUG-residual-20261002] '));stages={k:{'count':v[0],'ms':v[1]/1e6} for k,v in json.loads(line.split(' ',1)[1]).items()};assert r['stages']==stages
  assert len(list(W.glob(f'{stem}-*/measurement.json')))==expected
  count=sum((W/(stem+'-'+'-'.join(map(str,(r['role'],r['repeat'],r['profile'],r['mode']))))/'anki.json').exists() for r in rows);assert count==expected_anki
  for profile in dict.fromkeys(r['profile'] for r in rows):
   c={'profile':profile}
   for mode in dict.fromkeys(r['mode'] for r in rows):
    selected=[r for r in rows if r['profile']==profile and r['mode']==mode and r['role']=='timing'];warm=[r for r in rows if r['profile']==profile and r['mode']==mode and r['role']=='warmup'];assert len(selected)==5 and len(warm)==1
    if stem=='baseline':c[mode]=statistics.median(r['elapsed_ms'] for r in selected)
    else:
     keys=set().union(*(r['stages'] for r in selected));c[mode]=dict(n=5,median_ms=statistics.median(r['elapsed_ms'] for r in selected),samples_ms=[r['elapsed_ms'] for r in selected],rss_mib=statistics.median(r['rss_mib'] for r in selected),stages={k:dict(ms=statistics.median(r['stages'].get(k,{'ms':0})['ms'] for r in selected),count=statistics.median(r['stages'].get(k,{'count':0})['count'] for r in selected)) for k in keys})
   summary.append(c)
  saved=json.loads((W/f'{stem}-summary.json').read_text());computed={c.pop('profile'):c for c in summary} if stem=='baseline' else {c['profile']:c for c in summary};saved=saved if stem=='baseline' else {c['profile']:c for c in saved};assert computed==saved
  records.extend(rows);summaries[stem]={'exports':expected,'anki_checks':count}
 assert oracle==36 and len(records)==216 and all(len(s)==1 for s in digests.values())
 assert json.loads((W/'source-before.json').read_text())==json.loads((W/'source-after.json').read_text())
 assert json.loads((W/'runtime-match.json').read_text())['runtime_sources_match']
 result=dict(status='passed',exports=216,warmups=36,timings=180,anki_checks=36,profiles=5,logical_content_equal=True,production_identity_unchanged=True,raw_stages_reparsed=True,summaries_recomputed=True,groups=summaries)
 return result
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--work-dir',type=Path,default=Path('benchmarks/.work/residual-20261002'));a=p.parse_args();result=audit(a.work_dir);print(json.dumps(result,ensure_ascii=False,indent=2))
