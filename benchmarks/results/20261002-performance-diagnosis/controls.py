"""Predeclared repeated same-binary sync ablation; diagnostic results only."""
from pathlib import Path
import json,os,random,statistics,subprocess,sys
W=Path(__file__).resolve().parent;R=W.parents[2]
sys.path.insert(0,str(R/'benchmarks'))
import bench,verify
binary=W/'binaries/controls'
profiles=['basic-mixed-text-v1','basic-image-unique-v2','basic-audio-unique-v2','basic-mixed-unique-v2','basic-mixed-shared-v2']
modes={'baseline':{},'skip_cas':{'ANKIFORGE_DIAG_SKIP_CAS_SYNC':'1'},'skip_staging':{'ANKIFORGE_DIAG_SKIP_STAGE_SYNC':'1'},'skip_both':{'ANKIFORGE_DIAG_SKIP_CAS_SYNC':'1','ANKIFORGE_DIAG_SKIP_STAGE_SYNC':'1'}}
bench.save(W/'control-plan.json',{'profiles':profiles,'notes':1000,'modes':modes,'warmups_per_combination':1,'timing_repeats':3,'expected_exports':80,'binary_sha256':bench.sha256(binary),'source_commit':bench.command(['git','rev-parse','HEAD']),'purpose':'Diagnose two independent intermediate sync calls; final artifact synchronization and checks unchanged; not a production repair.'})
rows=[];rng=random.Random(20261002)
for role,repeats in [('warmup',1),('timing',3)]:
    for repeat in range(repeats):
        cases=[(p,m) for p in profiles for m in modes];rng.shuffle(cases)
        for profile,mode in cases:
            case=W/f'control-{role}-{repeat}-{profile}-{mode}';case.mkdir()
            inp=R/'benchmarks/.work/latest-20261002/fixtures'/profile/'inputs/1000.json'
            document=json.loads(inp.read_text());output=case/'output.apkg'
            env=dict(os.environ,TMPDIR=str(case),TMP=str(case),TEMP=str(case),**modes[mode])
            p=subprocess.run([str(bench.COLLECTOR),'60',str(case/'stdout.log'),str(case/'stderr.log'),str(binary),str(inp),str(output)],capture_output=True,text=True,check=True,env=env)
            measurement=json.loads(p.stdout)
            bench.save(case/'measurement.json',measurement)
            assert measurement['exit_code']==0 and not measurement['signal'] and not measurement['leftover_descendants']
            line=next(l for l in (case/'stderr.log').read_text().splitlines() if l.startswith('[DEBUG-perf-20261002] '))
            stages=json.loads(line.split(' ',1)[1])
            checked=verify.verify_artifact(output,document,bench.INSPECTOR);bench.save(case/'verification.json',checked);assert checked['status']=='passed',checked
            oracle=None
            if role=='timing' and repeat==0 and mode in ('baseline','skip_both'):
                with (case/'oracle.stdout.log').open('w') as stdout,(case/'oracle.stderr.log').open('w') as stderr:
                    subprocess.run([str(bench.ORACLE),str(inp),str(output),str(case/'anki.json')],check=True,stdout=stdout,stderr=stderr,timeout=120)
                oracle=json.loads((case/'anki.json').read_text());assert oracle['status']=='passed'
            row={'role':role,'repeat':repeat,'profile':profile,'mode':mode,'elapsed_ms':measurement['elapsed_ns']/1e6,'rss_mib':measurement['peak_rss_bytes']/2**20,'artifact_sha256':checked['artifact_sha256'],'logical_sha256':checked['physical']['logical_sha256'],'oracle_passed':oracle is not None,'stages':{k:{'count':v[0],'ms':v[1]/1e6} for k,v in stages.items()}}
            rows.append(row);bench.save(W/'control-results.json',{'status':'running','rows':rows})
            print(json.dumps({'completed':len(rows),**{k:row[k] for k in ('role','repeat','profile','mode','elapsed_ms')}}),flush=True);output.unlink()
        print(f'{role} round {repeat+1}/{repeats} complete',flush=True)
assert len(rows)==80
for profile in profiles: assert len({r['logical_sha256'] for r in rows if r['profile']==profile})==1
bench.save(W/'control-results.json',{'status':'completed','binary_sha256':bench.sha256(binary),'rows':rows})
summary=[]
for profile in profiles:
    c={'profile':profile}
    for mode in modes:
        selected=[r for r in rows if r['profile']==profile and r['mode']==mode and r['role']=='timing']
        c[mode]={'median_ms':statistics.median(r['elapsed_ms'] for r in selected),'n':len(selected),'samples_ms':[r['elapsed_ms'] for r in selected],'stages':{k:statistics.median(r['stages'][k]['ms'] for r in selected) for k in selected[0]['stages']}}
    summary.append(c)
bench.save(W/'control-summary.json',summary)
print(json.dumps(summary),flush=True)
