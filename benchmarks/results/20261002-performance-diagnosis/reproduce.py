"""Diagnostic feedback loop: fixed notes, varying unique image count."""
from pathlib import Path
import json,os,statistics,subprocess,sys
ROOT=Path(__file__).resolve().parents[3]
sys.path.insert(0,str(ROOT/'benchmarks'))
import bench,verify
WORK=Path(__file__).resolve().parent
source=ROOT/'benchmarks/.work/latest-20261002/fixtures/basic-image-unique-v2/inputs/100.json'
original=json.loads(source.read_text())
rows=[]
for repeat in range(3):
    for count in (0,1,10,100):
        document=json.loads(json.dumps(original))
        document['media']=document['media'][:count]
        ids={m['id'] for m in document['media']}
        for m in document['media']: m['path']=str(source.parent/m['path'])
        for n in document['notes']:
            for key in ('front_media','back_media'):
                if key in n: n[key]=[m for m in n[key] if m in ids]
        case=WORK/f'repro-{repeat}-{count}'
        case.mkdir()
        inp=case/'input.json'; inp.write_text(json.dumps(document))
        env=dict(os.environ,TMPDIR=str(case),TMP=str(case),TEMP=str(case))
        adapter=next(a for a in bench.registry() if a['id']=='rust')['command']
        output=case/'output.apkg'
        p=subprocess.run([str(bench.COLLECTOR),'60',str(case/'stdout.log'),str(case/'stderr.log'),*adapter,str(inp),str(output)],env=env,capture_output=True,text=True,check=True)
        measurement=json.loads(p.stdout)
        assert measurement['exit_code']==0 and not measurement['signal'] and not measurement['leftover_descendants']
        checked=verify.verify_artifact(output,document,bench.INSPECTOR)
        assert checked['status']=='passed',checked
        bench.save(case/'measurement.json',measurement);bench.save(case/'verification.json',checked)
        row=dict(repeat=repeat,notes=100,media=count,ms=measurement['elapsed_ns']/1e6,rss_mib=measurement['peak_rss_bytes']/2**20)
        rows.append(row);print(json.dumps(row),flush=True)
        output.unlink()
bench.save(WORK/'reproduction.json',{'adapter_sha256':bench.sha256(Path(adapter[0])),'rows':rows,'medians_ms':{str(c):statistics.median(r['ms'] for r in rows if r['media']==c) for c in (0,1,10,100)}})
