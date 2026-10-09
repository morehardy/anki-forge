"""One predeclared, randomized same-binary diagnostic matrix; every output checked."""
from pathlib import Path
import json,os,random,statistics,subprocess,sys
W=Path(__file__).resolve().parent;R=W.parents[2]
sys.path.insert(0,str(R/'benchmarks'));import bench,verify
profiles=['basic-mixed-text-v1','basic-image-unique-v2','basic-audio-unique-v2','basic-mixed-unique-v2','basic-mixed-shared-v2']
modes={'default':{},'positional':{'ANKIFORGE_DIAG_POSITIONAL':'1'},'bulk_cards':{'ANKIFORGE_DIAG_BULK_CARDS':'1'},'budget16':{'ANKIFORGE_DIAG_SNAPSHOT_BUDGET':str(16*1024*1024)}}
def power():return subprocess.check_output(['pmset','-g','batt'],text=True).strip()
bench.save(W/'control1-plan.json',dict(profiles=profiles,notes=1000,modes=modes,warmup_repeats=1,timing_repeats=5,expected_exports=120,expected_anki_checks=20,schedule_seed=20261003,binary_sha256={'official':bench.sha256(W/'binaries/current'),'probe':bench.sha256(W/'binaries/probe')},purpose='Locate residual cost. Instrumented diagnostic stages alter only one variable from default; official assesses instrumentation overhead. No production changes.'))
rows=[];rng=random.Random(20261003)
for role,repeats in [('warmup',1),('timing',5)]:
 for repeat in range(repeats):
  cases=[(p,m) for p in profiles for m in modes];rng.shuffle(cases)
  for profile,mode in cases:
   case=W/f'control1-{role}-{repeat}-{profile}-{mode}';case.mkdir()
   inp=R/'benchmarks/.work/spool-20261002/fixtures'/profile/'inputs/1000.json';doc=json.loads(inp.read_text());out=case/'output.apkg'
   binary=W/'binaries'/('current' if mode=='official' else 'probe')
   env=dict(os.environ,TMPDIR=str(case),TMP=str(case),TEMP=str(case))
   for key in ['ANKIFORGE_DIAG_SNAPSHOT_BUDGET','ANKIFORGE_DIAG_SERIAL_MEDIA','ANKIFORGE_DIAG_SKIP_RETAIN_SYNC']:env.pop(key,None)
   env.update(modes[mode]);before=power();assert before.splitlines()[0]=="Now drawing from 'AC Power'"
   p=subprocess.run([str(bench.COLLECTOR),'60',str(case/'stdout.log'),str(case/'stderr.log'),str(binary),str(inp),str(out)],capture_output=True,text=True,check=True,env=env)
   after=power();assert after.splitlines()[0]==before.splitlines()[0]
   m=json.loads(p.stdout);bench.save(case/'measurement.json',m);assert m['exit_code']==0 and not m['signal'] and not m['leftover_descendants']
   stages={}
   if mode!='official':
    line=next(l for l in (case/'stderr.log').read_text().splitlines() if l.startswith('[DEBUG-residual-20261002] '));stages={k:{'count':v[0],'ms':v[1]/1e6} for k,v in json.loads(line.split(' ',1)[1]).items()}
   checked=verify.verify_artifact(out,doc,bench.INSPECTOR);bench.save(case/'verification.json',checked);assert checked['status']=='passed',checked
   oracle=False
   if role=='timing' and repeat==0:
    with (case/'oracle.stdout.log').open('w') as stdout,(case/'oracle.stderr.log').open('w') as stderr:subprocess.run([str(bench.ORACLE),str(inp),str(out),str(case/'anki.json')],check=True,stdout=stdout,stderr=stderr,timeout=120)
    assert json.loads((case/'anki.json').read_text())['status']=='passed';oracle=True
   row=dict(role=role,repeat=repeat,profile=profile,mode=mode,elapsed_ms=m['elapsed_ns']/1e6,rss_mib=m['peak_rss_bytes']/2**20,logical_sha256=checked['physical']['logical_sha256'],artifact_sha256=checked['artifact_sha256'],power_before=before,power_after=after,oracle_passed=oracle,stages=stages)
   rows.append(row);bench.save(W/'control1-results.json',dict(status='running',rows=rows));print(json.dumps({k:row[k] for k in ['role','repeat','profile','mode','elapsed_ms']}),flush=True);out.unlink()
  print(role,'round',repeat+1,'complete',flush=True)
assert len(rows)==120 and sum(r['oracle_passed'] for r in rows)==20
for profile in profiles:assert len({r['logical_sha256'] for r in rows if r['profile']==profile})==1
bench.save(W/'control1-results.json',dict(status='completed',rows=rows))
summary=[]
for profile in profiles:
 c={'profile':profile}
 for mode in modes:
  selected=[r for r in rows if r['profile']==profile and r['mode']==mode and r['role']=='timing']
  keys=set().union(*(r['stages'] for r in selected))
  c[mode]=dict(n=len(selected),median_ms=statistics.median(r['elapsed_ms'] for r in selected),samples_ms=[r['elapsed_ms'] for r in selected],rss_mib=statistics.median(r['rss_mib'] for r in selected),stages={k:dict(ms=statistics.median(r['stages'].get(k,{'ms':0})['ms'] for r in selected),count=statistics.median(r['stages'].get(k,{'count':0})['count'] for r in selected)) for k in keys})
 summary.append(c)
bench.save(W/'control1-summary.json',summary)
print(json.dumps([{ 'profile':c['profile'],**{m:round(c[m]['median_ms'],3) for m in modes}} for c in summary]),flush=True)
