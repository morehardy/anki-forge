"""Sequential diagnostic measurements with full output checks."""
from pathlib import Path
import json,os,subprocess,sys
W=Path(__file__).resolve().parent;R=W.parents[2]
sys.path.insert(0,str(R/'benchmarks'))
import bench,verify
name=sys.argv[1]; binary=W/'binaries'/name
profiles=sys.argv[2:] or ['basic-mixed-text-v1','basic-image-unique-v2','basic-audio-unique-v2','basic-mixed-unique-v2','basic-mixed-shared-v2']
rows=[]
for repeat in range(3):
    for profile in profiles:
        case=W/f'{name}-{profile}-{repeat}';case.mkdir()
        inp=R/'benchmarks/.work/latest-20261002/fixtures'/profile/'inputs/1000.json'
        document=json.loads(inp.read_text());output=case/'output.apkg'
        env=dict(os.environ,TMPDIR=str(case),TMP=str(case),TEMP=str(case))
        p=subprocess.run([str(bench.COLLECTOR),'60',str(case/'stdout.log'),str(case/'stderr.log'),str(binary),str(inp),str(output)],capture_output=True,text=True,check=True,env=env)
        measurement=json.loads(p.stdout)
        assert measurement['exit_code']==0 and not measurement['signal'] and not measurement['leftover_descendants']
        stderr=(case/'stderr.log').read_text();line=next(l for l in stderr.splitlines() if l.startswith('[DEBUG-perf-20261002] '));stages=json.loads(line.split(' ',1)[1])
        checked=verify.verify_artifact(output,document,bench.INSPECTOR);assert checked['status']=='passed',checked
        bench.save(case/'verification.json',checked);bench.save(case/'measurement.json',measurement)
        row={'repeat':repeat,'profile':profile,'elapsed_ms':measurement['elapsed_ns']/1e6,'rss_mib':measurement['peak_rss_bytes']/2**20,'stages':{k:{'count':v[0],'ms':v[1]/1e6} for k,v in stages.items()}}
        rows.append(row);bench.save(W/f'{name}-results.json',{'binary_sha256':bench.sha256(binary),'rows':rows})
        print(json.dumps(row),flush=True);output.unlink()
