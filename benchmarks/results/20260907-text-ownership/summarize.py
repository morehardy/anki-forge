"""Aggregate all declared samples; instrumented and interrupted runs never supply headline timings."""
from pathlib import Path
import json,re,statistics
W=Path(__file__).resolve().parent
load=lambda p:json.loads(p.read_text())
def rows(name): return [json.loads(s) for s in (W/name/'attempts.jsonl').read_text().splitlines()]
def stats(v):
    v=sorted(v)
    def q(p):
        x=(len(v)-1)*p; a=int(x); return v[a]+(v[min(a+1,len(v)-1)]-v[a])*(x-a)
    return dict(n=len(v),median=statistics.median(v),q1=q(.25),q3=q(.75),minimum=min(v),maximum=max(v))
t=rows('before-after-timing'); m=rows('before-after-rss')
result={'method':{'timing_repeats':7,'rss_repeats':5,'warmups_per_phase':2,'power_source':'AC Power','timing_scope':'Fresh process startup, JSON parsing, Deck construction/registration, default export and verification inside the library.','comparison':'Previous shared-buffer implementation versus consuming temporary Project text; system allocator, Rust release/default features.','time_change_convention':'Positive means slower; negative means faster.','rss_scope':'Median of independent native process peaks; Rust allocator probe reported separately.'},'cases':[]}
for case in load(W/'cases.json'):
    c=dict(case); c['variants']={}
    for v in ['current','accepted']:
        ts=[r for r in t if r['case']==case['name'] and r['variant']==v and r['sample']>=0]
        ms=[r for r in m if r['case']==case['name'] and r['variant']==v and r['sample']>=0]
        assert len(ts)==7 and len(ms)==5
        rss_key=next(k for k in ms[0]['measurement'] if k.startswith('peak_rss'))
        c['variants'][v]={'time_ms':stats([r['measurement']['elapsed_ns']/1e6 for r in ts]),'rss_mib':stats([r['measurement'][rss_key]/2**20 for r in ms]),'artifact_sha256':ts[0]['artifact_sha256'],'artifact_bytes':ts[0]['artifact_bytes']}
    a,b=c['variants']['current'],c['variants']['accepted']
    c.update(time_change_pct=100*(b['time_ms']['median']/a['time_ms']['median']-1),rss_change_pct=100*(b['rss_mib']['median']/a['rss_mib']['median']-1),rss_change_mib=b['rss_mib']['median']-a['rss_mib']['median'])
    result['cases'].append(c)
result['allocations']=[]
for case in load(W/'allocation-check/plan.json')['cases']:
    c={'case':case['name'],'variants':{}}
    for v in ['current-memory','accepted-memory']:
        vals=[]
        for i in range(3):
            p=W/'allocation-check'/f'{case["name"]}-{v}-{i}'/'stderr.log'
            b,a=[json.loads(s) for s in re.findall(r'\[DEBUG-post-audit-memory\] (.*)',p.read_text())]
            vals.append({'peak_live_mib':a['peak_rust_bytes']/2**20,'export_allocated_mib':(a['cumulative_rust_bytes']-b['cumulative_rust_bytes'])/2**20,'export_allocation_calls':a['allocation_calls']-b['allocation_calls']})
        c['variants'][v]={k:stats([x[k] for x in vals]) for k in vals[0]}
    result['allocations'].append(c)
result['stages']=[]
r=rows('stage-check')
for case in load(W/'stage-check/plan.json')['cases']:
    c={'case':case['name'],'variants':{}}
    for v in ['current-stages','accepted-stages']:
        s=[x['stages_ns'] for x in r if x['case']==case['name'] and x['variant']==v and x['sample']>=0]
        total=[((x['normalize.total']+x['identity.after_normalize']) if v=='current-stages' else x['normalize.with_identity'])/1e6 for x in s]
        c['variants'][v]={'normalization_and_identity_ms':stats(total),'individual_stages_ms':{k:stats([x[k]/1e6 for x in s]) for k in s[0]}}
    result['stages'].append(c)
result['quality']=load(W/'quality-summary.json')
result['runs']={p.parent.name:{'attempts':len(p.read_text().splitlines()),'excluded_from_performance':(p.parent/'excluded.json').exists(),'plan':load(p.parent/'plan.json')} for p in sorted(W.glob('*/attempts.jsonl'))}
(W/'summary.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
print('Aggregated',len(result['cases']),'cases;',sum(x['attempts'] for x in result['runs'].values()),'retained attempts')
