"""Same-host remeasurement of frozen historical and current instrumented sources."""
from pathlib import Path
import json,os,statistics,subprocess,sys
W=Path(__file__).resolve().parent;R=W.parents[2]
sys.path.insert(0,str(R/'benchmarks'))
import bench,verify
profiles=['basic-mixed-text-v1','basic-image-unique-v2'];modes=['historical','current']
bench.save(W/'history-plan.json',{'historical_source':'20260921-readme-genanki/source-and-inputs.tar.gz','current_commit':bench.command(['git','rev-parse','HEAD']),'profiles':profiles,'notes':1000,'warmup_rounds':1,'timing_repeats':3,'expected_exports':16,'instrumented':True})
rows=[]
for role,repeats in [('warmup',1),('timing',3)]:
    for repeat in range(repeats):
        for profile in profiles:
            order=modes if repeat%2==0 else list(reversed(modes))
            for mode in order:
                binary=W/'binaries'/('historical' if mode=='historical' else 'controls')
                case=W/f'history-{role}-{repeat}-{profile}-{mode}';case.mkdir()
                inp=R/'benchmarks/.work/latest-20261002/fixtures'/profile/'inputs/1000.json'
                doc=json.loads(inp.read_text());out=case/'output.apkg'
                env=dict(os.environ,TMPDIR=str(case),TMP=str(case),TEMP=str(case));env.pop('ANKIFORGE_DIAG_SKIP_CAS_SYNC',None);env.pop('ANKIFORGE_DIAG_SKIP_STAGE_SYNC',None)
                p=subprocess.run([str(bench.COLLECTOR),'60',str(case/'stdout.log'),str(case/'stderr.log'),str(binary),str(inp),str(out)],capture_output=True,text=True,check=True,env=env)
                m=json.loads(p.stdout);bench.save(case/'measurement.json',m);assert m['exit_code']==0 and not m['signal'] and not m['leftover_descendants']
                line=next(l for l in (case/'stderr.log').read_text().splitlines() if l.startswith('[DEBUG-perf-20261002] '));stages=json.loads(line.split(' ',1)[1])
                checked=verify.verify_artifact(out,doc,bench.INSPECTOR);bench.save(case/'verification.json',checked);assert checked['status']=='passed',checked
                if role=='timing' and repeat==0:
                    with (case/'oracle.stdout.log').open('w') as stdout,(case/'oracle.stderr.log').open('w') as stderr: subprocess.run([str(bench.ORACLE),str(inp),str(out),str(case/'anki.json')],check=True,stdout=stdout,stderr=stderr,timeout=120)
                    assert json.loads((case/'anki.json').read_text())['status']=='passed'
                row={'role':role,'repeat':repeat,'profile':profile,'mode':mode,'elapsed_ms':m['elapsed_ns']/1e6,'logical_sha256':checked['physical']['logical_sha256'],'stages':{k:{'count':v[0],'ms':v[1]/1e6} for k,v in stages.items()}}
                rows.append(row);bench.save(W/'history-results.json',{'status':'running','rows':rows});print(json.dumps(row),flush=True);out.unlink()
assert len(rows)==16
for p in profiles: assert len({r['logical_sha256'] for r in rows if r['profile']==p})==1
summary=[]
for p in profiles:
    c={'profile':p}
    for mode in modes:
        samples=[r for r in rows if r['role']=='timing' and r['profile']==p and r['mode']==mode]
        c[mode]={'median_ms':statistics.median(r['elapsed_ms'] for r in samples),'samples_ms':[r['elapsed_ms'] for r in samples],'stages':{k:statistics.median(r['stages'][k]['ms'] for r in samples) for k in samples[0]['stages']}}
    summary.append(c)
bench.save(W/'history-results.json',{'status':'completed','rows':rows});bench.save(W/'history-summary.json',summary)
print(json.dumps(summary),flush=True)
