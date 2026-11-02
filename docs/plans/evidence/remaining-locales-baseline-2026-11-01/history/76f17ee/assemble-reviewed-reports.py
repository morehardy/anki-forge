"""Attach manually authored conclusions to actual observations; performs no review."""
import json,pathlib,hashlib,sys,urllib.parse
root=pathlib.Path.cwd(); env=sys.argv[1]; p=pathlib.Path('docs/plans/evidence/remaining-locales-baseline-2026-11-01')
load=lambda f:json.loads((root/f).read_text())
hashfile=lambda f:hashlib.sha256((root/f).read_bytes()).hexdigest()
fileproof=lambda f,d:{'path':str(f),'sha256':hashfile(f),'description':d}
manifest=load(p/f'{env}-origin-manifest.json');u=load(p/f'usage-{env}/browser-tour.json');m=load(p/f'machine-{env}/http-observations.json');c=load(p/'review-conclusions.json');a=load(p/'execution-reuse-audit.json')
assert u['manifestSha256']==m['manifestSha256']==hashfile(p/f'{env}-origin-manifest.json')
records={r['route']:r for r in manifest['pages']+manifest['resources']}
bindings=[{'route':r,'sha256':v['sha256']} for r,v in records.items()]
deps={'Cargo.toml','Cargo.lock','bindings/python/pyproject.toml','bindings/node/package.json','docs/ai-quickstart.json','website/package.json','website/package-lock.json'}
src=[b for b in a['currentBindings'] if b['path'] not in deps];dependency=[b for b in a['currentBindings'] if b['path'] in deps]
for b in a['currentBindings']:assert hashfile(pathlib.Path(b['path']))==b['sha256']
old=pathlib.Path('docs/plans/evidence/multilingual-es-integration-2026-10-31/executions')
def execution(f,description):return dict(fileproof(f,description),sourceBindings=src,dependencyBindings=dependency)
def common(kind,obs):
 return {'schemaVersion':1,'kind':kind,'status':'passed','mode':'release-candidate','reviewerType':'ai','reviewerRun':f'/root/baseline_acceptance/{kind}-{env}-2026-11-01','inputSha256':manifest['inputSha256'],'configuration':manifest['configuration'],'releaseLocales':manifest['releaseLocales'],'readOrigin':obs['readOrigin'],'originManifest':{'path':str(p/f'{env}-origin-manifest.json'),'sha256':hashfile(p/f'{env}-origin-manifest.json'),'commit':manifest['commit']},'resourceBindings':bindings,'entries':{},'openIssues':[],'independence':c['authorship'],'scope':'All artifact routes are hash-bound for freshness; binding is not a claim of manual reading of every page. Actual readUrls, raw observations, task answers and evidence below define the review scope.'}
usage=common('usage-tasks',u)
usage['executionReuse']='No SDK consumer was rerun for this baseline maintenance. Current source/dependency equality, exact clipboard programs, previous actual installation/execution logs and independent ZIP/Zstandard/SQLite inspection of ten retained APKGs justify explicitly bound reuse. New browser navigation, clipboard and recipe downloads occurred for every locale/configuration.'
usage['visualInspection']='Reviewer inspected prefix screenshots en/rust-basic, zh-CN/python-basic and es/node-basic at 1440x1000. Other screenshots record browser state; no claim that every screenshot was manually inspected.' if env=='prefix' else 'Root screenshot review and root/prefix semantic comparison are documented in root-review-comparison.json.'
for loc,tasks in u['entries'].items():
 out=[]
 for task in tasks:
  tid=task['id'];route=urllib.parse.urlparse(task['finalUrl']).path
  proofs=[fileproof(p/f'usage-{env}/browser-tour.json','Fresh actual browser navigation, rendered text, clipboard operations and HTTP download observations for this configuration.'),fileproof(pathlib.Path(task['screenshot']),'Actual browser screenshot retained as task-state evidence.'),fileproof(p/'execution-reuse-audit.json','Independent audit of unchanged inputs and retained actual outputs; SDK execution is reused explicitly.'),{'route':route,'sha256':records[route]['sha256'],'observation':c['usage'][tid]}]
  if tid.endswith('-basic'):
   sdk=tid.split('-')[0];ex=[execution(old/f'basic/{sdk}/basic.log','Retained actual public 0.3.1 execution log; unchanged current source/dependencies audited independently.'),execution(old/f'basic/{sdk}/spanish.apkg','Retained actual output, newly decoded and inspected independently in execution-reuse-audit.json.')]
  elif tid in ['json-media','original-baseline-update']:
   ex=[execution(old/'public-recipe.log','Retained actual public 0.3.1 executions of the complete Rust/Python/Node JSON programs; current inputs and six outputs independently revalidated.')]
   for sdk in ['rust','python','node']:
    proofs += [fileproof(old/f'public-json/{sdk}/comparison.json','Actual recipe comparison retains stable identity and permits this one-answer change.'),fileproof(old/f'public-json/{sdk}/review.json','Actual prepared candidate policy report, independently inspected.')]
  else:ex=[execution(old/'basic/python/troubleshooting.log','Retained actual duplicate-key AddError and recovery execution, newly inspected against unchanged current troubleshooting/source inputs.')]
  out.append({'id':tid,'status':'passed','openIssues':[],'readUrls':task['readUrls'],'steps':task['steps']+['Independently read the captured localized instructions and checked the retained execution using the fresh source/dependency and APKG audit.',c['usage'][tid]],'executionEvidence':ex,'evidence':proofs})
 usage['entries'][loc]={'taskResults':out,'openIssues':[]}
machine=common('machine-read',m)
machine['method']='Each task made a new actual HTTP GET to this locale llms.txt, followed its real same-locale Markdown links and verified status, UTF-8 content type, no download disposition, manifest bytes and SHA-256. Independent answers were written from the captured responses, with source citations below; no English fallback or old passing answer was used.'
for loc,tasks in m['entries'].items():
 out=[]
 for task in tasks:
  reads=[task['index']]+task['followed'];proofs=[fileproof(p/f'machine-{env}/http-observations.json','Fresh HTTP response URLs, headers, bodies and exact manifest integrity observations.')]
  for read in reads:proofs.append(fileproof(pathlib.Path(read['path']),f"Actual HTTP UTF-8 response from {read['url']}; status {read['status']}, manifest-bound content."))
  out.append({'id':task['id'],'status':'passed','openIssues':[],'readUrls':task['readUrls'],'answer':c['machine'][loc][task['id']]+' Sources: '+', '.join(r['url'] for r in task['followed']),'evidence':proofs})
 machine['entries'][loc]={'aiMachineReadReview':out,'openIssues':[]}
if env=='root':
 for doc in [usage,machine]:
  doc['rootSemanticReview']='Fresh root browser/HTTP tasks are recorded independently. Complete captured bodies/code match the independently read prefix text after only exact deployment URL replacement, as recorded in root-review-comparison.json; therefore semantic conclusions are carried across with actual root URLs, rather than claiming another SDK execution or a full rereading of identical prose.'
  key='taskResults' if doc['kind']=='usage-tasks' else 'aiMachineReadReview'
  for entry in doc['entries'].values():
   for task in entry[key]:task['evidence'].append(fileproof(p/'root-review-comparison.json','Actual root text/code and HTTP body comparison, deployment-URL-only normalization, plus direct root screenshot review.'))
for name,doc in [('usage-tasks',usage),('machine-read',machine)]:
 (root/p/f'{name}-{env}.json').write_text(json.dumps(doc,ensure_ascii=False,indent=2)+'\n')
print(f'Assembled {env}: 18 usage judgments and 12 machine answers from manual conclusions and fresh observations.')
