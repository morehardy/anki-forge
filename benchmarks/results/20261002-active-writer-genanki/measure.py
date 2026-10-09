"""Predeclare a balanced diagnostic run and validate every resulting package."""
from pathlib import Path
import argparse,hashlib,json,os,random,statistics,subprocess,sys
W=Path(__file__).resolve().parent;R=W.parents[2]
sys.path.insert(0,str(R/'benchmarks'));import bench,verify
p=argparse.ArgumentParser();p.add_argument('group');p.add_argument('--warmups',type=int,default=2);p.add_argument('--repeats',type=int,default=6);args=p.parse_args()
config=json.loads((W/f'{args.group}-config.json').read_text());profiles=config['profiles'];modes=config['modes']
def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def power():return subprocess.check_output(['pmset','-g','batt'],text=True).strip()
def identity():
 files={str(W/'binaries'/v['binary']):sha(W/'binaries'/v['binary']) for v in modes.values()}
 files.update({str(path):sha(path) for path in [bench.COLLECTOR,bench.INSPECTOR,bench.ORACLE]})
 for profile in profiles:
  path=W/'fixtures'/profile/'inputs/1000.json';files[str(path)]=sha(path)
  for media in json.loads(path.read_text()).get('media',[]):
   source=(path.parent/media['path']).resolve();files[str(source)]=sha(source)
 return files
plan=dict(config,group=args.group,warmups=args.warmups,repeats=args.repeats,seed=20261004,expected_exports=len(profiles)*len(modes)*(args.warmups+args.repeats),expected_anki_checks=len(profiles)*len(modes),created_utc=bench.utc(),identity_before=identity())
assert not (W/f'{args.group}-plan.json').exists();bench.save(W/f'{args.group}-plan.json',plan)
rows=[];rng=random.Random(plan['seed']);names=list(modes)
for role,repeats in [('warmup',args.warmups),('timing',args.repeats)]:
 for repeat in range(repeats):
  order=profiles.copy();rng.shuffle(order)
  for profile in order:
   mode_order=names[repeat%len(names):]+names[:repeat%len(names)]
   for mode in mode_order:
    case=W/f'{args.group}-{role}-{repeat}-{profile}-{mode}';case.mkdir()
    inp=W/'fixtures'/profile/'inputs/1000.json';doc=json.loads(inp.read_text());out=case/'output.apkg'
    env={k:v for k,v in os.environ.items() if not k.startswith(('ANKIFORGE_DIAG_','ANKIFORGE_ROUND4_'))}
    env.update(TMPDIR=str(case),TMP=str(case),TEMP=str(case));env.update(modes[mode].get('env',{}))
    before=power();assert before.splitlines()[0]=="Now drawing from 'AC Power'"
    result=subprocess.run([str(bench.COLLECTOR),'60',str(case/'stdout.log'),str(case/'stderr.log'),str(W/'binaries'/modes[mode]['binary']),str(inp),str(out)],env=env,capture_output=True,text=True,check=True)
    after=power();assert after.splitlines()[0]==before.splitlines()[0]
    measurement=json.loads(result.stdout);bench.save(case/'measurement.json',measurement)
    assert measurement['reaped'] and not any(measurement[k] for k in ['spawn_error','exit_code','signal','interrupted_signal','leftover_descendants'])
    checked=verify.verify_artifact(out,doc,bench.INSPECTOR);bench.save(case/'verification.json',checked);assert checked['status']=='passed'
    oracle=role=='timing' and repeat==0
    if oracle:
     with (case/'oracle.stdout.log').open('w') as stdout,(case/'oracle.stderr.log').open('w') as stderr:
      subprocess.run([str(bench.ORACLE),str(inp),str(out),str(case/'anki.json')],check=True,stdout=stdout,stderr=stderr,timeout=120)
     assert json.loads((case/'anki.json').read_text())['status']=='passed'
    traces=[line.split('] ',1)[1] for line in (case/'stderr.log').read_text().splitlines() if line.startswith('[DEBUG-import-round4] ')]
    stages={key:{'count':value[0],'ms':value[1]/1e6} for key,value in json.loads(traces[0]).items()} if traces else {}
    row=dict(role=role,repeat=repeat,profile=profile,mode=mode,elapsed_ms=measurement['elapsed_ns']/1e6,rss_mib=measurement['peak_rss_bytes']/2**20,artifact_bytes=checked['artifact_bytes'],logical_sha256=checked['physical']['logical_sha256'],artifact_sha256=checked['artifact_sha256'],power_before=before,power_after=after,oracle_passed=oracle,stages=stages)
    rows.append(row);bench.save(W/f'{args.group}-results.json',dict(status='running',rows=rows));out.unlink()
  print(role,repeat+1,'complete',flush=True)
assert len(rows)==plan['expected_exports'] and sum(row['oracle_passed'] for row in rows)==plan['expected_anki_checks']
assert identity()==plan['identity_before']
for profile in profiles:assert len({row['logical_sha256'] for row in rows if row['profile']==profile})==1
bench.save(W/f'{args.group}-results.json',dict(status='completed',rows=rows,identity_unchanged=True,completed_utc=bench.utc()))
summary=[]
for profile in profiles:
 cell={'profile':profile}
 for mode in modes:
  selected=[row for row in rows if row['role']=='timing' and row['profile']==profile and row['mode']==mode]
  keys=set().union(*(row['stages'] for row in selected))
  cell[mode]=dict(n=len(selected),samples_ms=[row['elapsed_ms'] for row in selected],median_ms=statistics.median(row['elapsed_ms'] for row in selected),rss_mib=statistics.median(row['rss_mib'] for row in selected),artifact_bytes=statistics.median(row['artifact_bytes'] for row in selected),stages={key:dict(ms=statistics.median(row['stages'][key]['ms'] for row in selected),count=statistics.median(row['stages'][key]['count'] for row in selected)) for key in keys})
 summary.append(cell)
bench.save(W/f'{args.group}-summary.json',summary)
print(json.dumps([{'profile':cell['profile'],**{mode:round(cell[mode]['median_ms'],3) for mode in modes}} for cell in summary],indent=2))
