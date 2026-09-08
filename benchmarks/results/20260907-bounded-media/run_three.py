"""Predeclare and run three complete unchanged media matrices sequentially."""
import datetime,hashlib,json,os,subprocess,sys,time
from pathlib import Path
REPO=Path(__file__).resolve().parents[4]
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
 prefix=datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%SZ-bounded-media')
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
 snapshot = {'source_commit': plan['source_commit'], 'git_status': plan['git_status'],
             'source_files': bench.identity_snapshot(bench.registry())['source_files']}
 save(ROOT/'source-snapshot.json', snapshot)
 patch = subprocess.check_output(['git', 'diff', '--binary', 'HEAD'], cwd=REPO)
 tracked = set(subprocess.check_output(['git', 'ls-files', '-z'], cwd=REPO, text=True).split('\0'))
 for path in bench.source_paths():
  relative = str(path.relative_to(REPO))
  if relative not in tracked and path.is_file():
   added = subprocess.run(['git', 'diff', '--no-index', '--binary', '--', '/dev/null', relative], cwd=REPO, capture_output=True)
   if added.returncode not in (0, 1):
    raise RuntimeError(added.stderr.decode())
   patch += added.stdout
 (ROOT/'source.patch').write_bytes(patch)
 plan['source_snapshot'] = {name: bench.sha256(ROOT/name) for name in ('source-snapshot.json', 'source.patch')}
 plan['expected_power_source'] = plan['host_before']['power'].splitlines()[0]
 plan['power_label'] = 'Battery power' if "'Battery Power'" in plan['expected_power_source'] else 'AC power'
 plan['power_policy'] = 'Record native power immediately before and after every export, outside timing; stop on any source change.'
 save(ROOT/'plan.json',plan)
 print('PLAN',ROOT/'plan.json',flush=True)
 inputs = REPO/'benchmarks/.work/readme-comparison/fixtures'
 import media_bench
 if not inputs.exists():
  media_workload.generate(inputs)
 media_bench.suite_inputs(inputs)
 print('Fixtures frozen and verified. Waiting 30 seconds after preparation.',flush=True)
 time.sleep(30)
 for index,name in enumerate(names,1):
  print(f'ROUND {index}/3 {name}',flush=True)
  with (ROOT/f'round-{index}.log').open('w') as log:
   proc=subprocess.Popen([sys.executable,str(ROOT/'guarded_media_bench.py'),
                          '--name',name,'--inputs',str(inputs)],cwd=REPO,
                          stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,bufsize=1)
   for line in proc.stdout:
    log.write(line);log.flush();print(line,end='',flush=True)
   code=proc.wait()
  if code:
   save(ROOT/'experiment.json',{'status':'failed','round':index,'returncode':code,'completed_utc':bench.utc()})
   raise SystemExit(code)
 save(ROOT/'experiment.json',{'status':'completed','rounds':names,'completed_utc':bench.utc(),'host_after':bench.host_state()})
 print('ALL THREE ROUNDS COMPLETE',ROOT,flush=True)
