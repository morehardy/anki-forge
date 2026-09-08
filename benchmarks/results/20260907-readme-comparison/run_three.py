"""Predeclare and run three complete unchanged media matrices sequentially."""
import datetime,json,os,subprocess,sys,time
from pathlib import Path
REPO=Path(__file__).resolve().parents[3]
sys.path.insert(0,str(REPO/'benchmarks'))
import bench,media_workload
ROOT=Path(__file__).resolve().parent

def save(path,data):
 path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n')

def hardware():
 return {k:bench.command(args,optional=True) for k,args in {
  'cpu':['sysctl','-n','machdep.cpu.brand_string'],
  'memory_bytes':['sysctl','-n','hw.memsize'],
  'logical_cpus':['sysctl','-n','hw.logicalcpu'],
  'filesystem':['df','-h',str(REPO)],
 }.items()}

if __name__=='__main__':
 if (ROOT/'plan.json').exists():
  raise RuntimeError('refusing to overwrite a predeclared experiment')
 prefix=datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%SZ-readme-system')
 names=[f'{prefix}-r{i}' for i in (1,2,3)]
 plan={'schema':'readme-media-comparison-plan-v1','created_utc':bench.utc(),
       'source_commit':bench.command(['git','rev-parse','HEAD']),
       'git_status':bench.command(['git','status','--short']),
       'rounds':names,'round_count':3,'sizes':[100,200,500,1000],
       'profiles':list(media_workload.PROFILES),'timing_per_round':10,'rss_per_round':5,
       'warmups_per_phase_per_round':3,'schedule_seed':20260907,
       'schedule_policy':'Repeat the unchanged balanced, interleaved schedule in three sequential sessions.',
       'aggregate':'Pool all 30 timing and 15 separate RSS samples per exporter/cell; median and linear inclusive Q1/Q3. Keep all three per-session medians and same-session ratios.',
       'chart_time_saved':'100 * (1 - pooled Rust median / pooled genanki median)',
       'stability':'Report max-minus-min of all three same-session time-saved percentages per cell; flag spread above 5 percentage points. Diagnostic only; no significance claim.',
       'failure_policy':'Retain every attempt; stop on a failed full round. No best-run selection or automatic extra rounds.',
       'environment':hardware(),'host_before':bench.host_state(),
       'oracle_patch':bench.command(['git','diff','--binary','HEAD'],cwd=REPO/'docs/source/anki'),
       'oracle_revision':bench.command(['git','rev-parse','HEAD'],cwd=REPO/'docs/source/anki')}
 if plan['git_status']:
  raise RuntimeError('requires a clean package source checkout before measurement')
 save(ROOT/'plan.json',plan)
 print('PLAN',ROOT/'plan.json',flush=True)
 media_workload.generate(ROOT/'fixtures')
 print('Fixtures frozen and verified. Waiting 30 seconds after preparation.',flush=True)
 time.sleep(30)
 for index,name in enumerate(names,1):
  print(f'ROUND {index}/3 {name}',flush=True)
  with (ROOT/f'round-{index}.log').open('w') as log:
   proc=subprocess.Popen([sys.executable,str(REPO/'benchmarks/media_bench.py'),
                          '--name',name,'--inputs',str(ROOT/'fixtures')],cwd=REPO,
                          stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,bufsize=1)
   for line in proc.stdout:
    log.write(line);log.flush();print(line,end='',flush=True)
   code=proc.wait()
  if code:
   save(ROOT/'experiment.json',{'status':'failed','round':index,'returncode':code,'completed_utc':bench.utc()})
   raise SystemExit(code)
 save(ROOT/'experiment.json',{'status':'completed','rounds':names,'completed_utc':bench.utc(),'host_after':bench.host_state()})
 print('ALL THREE ROUNDS COMPLETE',ROOT,flush=True)
