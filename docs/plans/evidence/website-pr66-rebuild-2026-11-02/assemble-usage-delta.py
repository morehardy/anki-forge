"""After actual new browser tours, assemble narrowly justified incremental usage proof."""
import copy,json,hashlib,pathlib,urllib.parse
B=pathlib.Path('docs/plans/evidence/website-pr66-rebuild-2026-11-02')
P=pathlib.Path('docs/plans/evidence/website-pr66-final-2026-11-02')
def load(p):return json.loads(pathlib.Path(p).read_text())
def sha(p):return hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
def bind(p,description=None):
 r={'path':str(p),'sha256':sha(p)}
 if description:r['description']=description
 return r
def save(n,x):(B/n).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
source=load(B/'linguistic-delta-input-audit.json');assert sum(x['changed'] for x in source['inputFiles'])==1
for x in source['inputFiles']:assert sha(x['path'])==x['currentSha256']
# Every original program/dependency and installed-package receipt is retained, not silently rebound.
execution=load(P/'usage-execution-audit.json')
for x in execution['sourceBindings']+execution['dependencyBindings']:assert sha(x['path'])==x['sha256']
for x in execution['receipts']+execution['locksAndInstallReports']+execution['independentInspection']:assert sha(x['path'])==x['sha256']
expected=load(B/'usage-benchmark-expectations.json')
assert sha(B/'usage-benchmark-expectations.json')==sha(P/'linguistic-benchmark-expectations.json')
audit={'schemaVersion':1,'reviewerRun':'/root/spec_review/pagefind-stability-usage-2026-11-02','scope':'New real browser runs for each new artifact. Reuse prior independently read page/task semantics only after every current body/heading/code/copy/download and previously bound artifact resource is compared. Reuse previous SDK executions explicitly, after all original source/dependency and retained evidence bytes match. No SDK execution or APKG decoding is rerun in this delta.','previousExecutionAudit':bind(P/'usage-execution-audit.json'),'sourceAndDependencyBindingsRechecked':execution['sourceBindings']+execution['dependencyBindings'],'receiptAndInspectionBindingsRechecked':execution['receipts']+execution['locksAndInstallReports']+execution['independentInspection'],'sourceDelta':bind(B/'linguistic-delta-input-audit.json'),'configurations':{},'newIssues':[]}
tours={};reports={}
for mode in ['prefix','root']:
 now=load(B/f'usage-{mode}/browser-tour.json');old=load(P/f'usage-{mode}/browser-tour.json');prior=load(P/f'usage-tasks-{mode}.json');m=load(B/f'{mode}-site-manifest.json')
 assert not now['pageErrors'] and not now['responseReadErrors'] and not now.get('failure')
 assert now['manifestSha256']==sha(B/f'{mode}-site-manifest.json') and now['inputSha256']==m['inputSha256']
 assert now['configuration']==prior['configuration'] and now['releaseLocales']==prior['releaseLocales']
 records={x['route']:x for x in m['pages']+m['resources']};wanted={x['route'] for x in now['loadedResources']};machine={x['route'] for x in m['resources'] if x['route'].endswith('/index.md') or x['route'].endswith('/llms.txt')};assert len(machine)==192;wanted.update(machine)
 taskChecks=[]
 for l,es in now['entries'].items():
  assert len(es)==6
  for e,z in zip(es,old['entries'][l]):
   for field in ['id','h1','body','codeBlocks','copiedPrograms']:assert e[field]==z[field],(mode,l,e['id'],field)
   assert sorted(x['sha256'] for x in e['downloadedResources'])==sorted(x['sha256'] for x in z['downloadedResources'])
   for x in e['copiedPrograms']:assert sha(x['source'])==x['sourceSha256'] and pathlib.Path(x['source']).read_text().strip()==x['copiedCode'].strip()
   wanted.update(urllib.parse.urlparse(u).path for u in e['readUrls'])
   for x in e['downloadedResources']:assert sha(x['path'])==x['sha256'];wanted.add(urllib.parse.urlparse(x['url']).path)
   taskChecks.append({'locale':l,'id':e['id'],'actualProseHeadingRenderedCodeAndClipboardEqual':True,'downloadsEqual':True})
  for field in ['mainText','heading','description','rows','performanceText','primaryMetric','rangeMetric']:assert now['homeObservations'][l][field]==old['homeObservations'][l][field],(mode,l,field)
  wanted.add(urllib.parse.urlparse(now['homeObservations'][l]['url']).path)
  for e,z in zip(now['editorialObservations'][l],old['editorialObservations'][l]):assert e['mainText']==z['mainText'] and e['heading']==z['heading'];wanted.add(urllib.parse.urlparse(e['url']).path)
 for x in now['repairObservations']['japaneseClipboard']:assert x['deniedSelectionMatches'];wanted.add(urllib.parse.urlparse(x['url']).path)
 for x in now['repairObservations']['wordForms']:assert any(urllib.parse.urlparse(r['url']).path==x['expected'] for r in x['results']);wanted.add(urllib.parse.urlparse(x['pageUrl']).path)
 assert not any(p.endswith('/generated/showcase.json') for p in wanted)
 assert len(wanted)==397 and wanted=={x['route'] for x in prior['resourceBindings']}
 changed=[]
 for x in prior['resourceBindings']:
  if records[x['route']]['sha256']!=x['sha256']:changed.append({'route':x['route'],'beforeSha256':x['sha256'],'currentSha256':records[x['route']]['sha256']})
 assert len(changed)==1 and changed[0]['route'].endswith('/pagefind/pagefind-entry.json'),changed
 entry=load(f'target/website-pr66-rebuild/{mode}-dist/pagefind/pagefind-entry.json');oldentry=load(f'target/website-pr66-final/{mode}-dist/pagefind/pagefind-entry.json');assert entry==oldentry
 assert list(entry['languages'])==sorted(entry['languages'])
 audit['configurations'][mode]={'previousReview':bind(P/f'usage-tasks-{mode}.json'),'currentTour':bind(B/f'usage-{mode}/browser-tour.json'),'currentManifest':bind(B/f'{mode}-site-manifest.json'),'inputSha256':m['inputSha256'],'tasks':taskChecks,'other396BoundResourceBytesIdentical':True,'changedResources':changed,'changedEntryParsedEqual':True,'same397ResourceScope':True,'homeAndEditorialCapturedTextEqual':True,'counts':{'tasks':36,'copies':sum(len(t['copiedPrograms']) for es in now['entries'].values() for t in es),'downloads':sum(len(t['downloadedResources']) for es in now['entries'].values() for t in es),'loadedResources':len(now['loadedResources']),'requiredMachineResources':192,'resourceBindings':397}}
 tours[mode]=now;reports[mode]=(prior,m,records,wanted)
save('usage-incremental-audit.json',audit)
for mode,now in tours.items():
 prior,m,records,wanted=reports[mode];entries={}
 for l,es in now['entries'].items():
  tasks=[]
  for e,oldtask in zip(es,prior['entries'][l]['taskResults']):
   assert e['id']==oldtask['id'];route=urllib.parse.urlparse(e['finalUrl']).path
   reusedExecutions=copy.deepcopy(oldtask['executionEvidence'])
   for ex in reusedExecutions:
    assert sha(ex['path'])==ex['sha256']
    for x in ex['sourceBindings']+ex['dependencyBindings']:assert sha(x['path'])==x['sha256']
    ex['description']='Explicit reuse of the previous final-batch public SDK execution after exact original source/dependency/evidence SHA checks. The prior independent APKG decoding remains valid. No SDK program execution or decoding is rerun in this producer-only delta.'
   evidence=[{'route':route,'sha256':records[route]['sha256'],'observation':'Actually navigated and captured this current localized HTML. Its complete task body, heading, code and clipboard bytes are identical to the previously independently read page; task interpretation is explicitly inherited after this exact check.'},bind(B/f'usage-{mode}/browser-tour.json',f'New actual {mode} six-language browsing, clipboard/download actions and Pagefind regression queries. Inspect entries[{l}]/{e["id"]}.'),bind(B/'usage-incremental-audit.json','Every task reading/code/copy/download and all 397 bound resources compared with prior final review; only Pagefind entry bytes differ and parsed metadata is equal.'),bind(P/'usage-independent-conclusions.json','Retained independent actual full task reading, exact localized quote and judgment, reused only for proven unchanged content.'),bind(P/f'usage-tasks-{mode}.json','Immutable previous completed usage review and its original source/dependency/evidence bindings.'),bind(P/'usage-execution-audit.json','Previous public registry receipts, complete execution inputs and actual independent APKG decoder results, all rechecked unchanged.'),bind(e['screenshot'],'Actual new task screenshot retained; no new full visual audit claimed for unchanged content.')]
   evidence.extend(bind(d['path'],'Newly downloaded task input bytes; exact equality with prior independently verified input.') for d in e['downloadedResources'])
   tasks.append({'id':e['id'],'status':'passed','openIssues':oldtask['openIssues'],'readUrls':e['readUrls'],'steps':e['steps']+['Compared every current visible task paragraph, heading, rendered program and copied code with the previous independently read final capture: exact equality. Reused its semantic judgment and existing nonblocking observations.','Reused previous public SDK execution only after exact source, dependency, package receipt, output and inspection evidence checks; did not rerun it.'],'executionEvidence':reusedExecutions,'evidence':evidence})
  entries[l]={'taskResults':tasks,'openIssues':prior['entries'][l].get('openIssues',[])}
 report={'schemaVersion':1,'kind':'usage-tasks','status':'passed','mode':'release-candidate','reviewerType':'ai','reviewerRun':f'/root/spec_review/pagefind-stability-usage-{mode}-2026-11-02','inputSha256':m['inputSha256'],'configuration':m['configuration'],'releaseLocales':m['releaseLocales'],'readOrigin':now['readOrigin'],'originManifest':{**bind(B/f'{mode}-site-manifest.json'),'commit':m['commit']},'resourceBindings':[{'route':p,'sha256':records[p]['sha256']} for p in sorted(wanted)],'resourceScope':'All actual new browser resources, read URLs and downloads plus required 192 machine texts: exact same 397-resource scope as prior final review. Canonical Pagefind entry is included; unused showcase.json and social images are excluded. Machine semantic review is separate.','candidateScope':'New local PR66 rebuild candidate only; no deployment, fresh native Anki import, new SDK execution or new reading of byte-identical prose claimed.','previousReview':bind(P/f'usage-tasks-{mode}.json'),'independentIncrementalAudit':bind(B/'usage-incremental-audit.json'),'sourceReviews':[bind(B/'linguistic-source-review.json'),bind(P/'linguistic-source-review.json')],'repairRegressions':now['repairObservations'],'performanceObservations':{l:{k:v for k,v in h.items() if k!='mainText'} for l,h in now['homeObservations'].items()},'openIssues':prior['openIssues'],'entries':entries}
 save(f'usage-tasks-{mode}.json',report);print(mode,'36 actual tasks; 396 unchanged bindings + canonical entry; explicit exact-input SDK/reading reuse.')
