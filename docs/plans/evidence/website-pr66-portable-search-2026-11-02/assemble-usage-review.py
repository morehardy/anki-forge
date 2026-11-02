"""Serialize actual portable-index browser usage and fresh public-SDK execution evidence."""
import copy,json,hashlib,pathlib,urllib.parse
B=pathlib.Path('docs/plans/evidence/website-pr66-portable-search-2026-11-02');P=pathlib.Path('docs/plans/evidence/website-pr66-rebuild-2026-11-02');F=pathlib.Path('docs/plans/evidence/website-pr66-final-2026-11-02')
def load(p):return json.loads(pathlib.Path(p).read_text())
def sha(p):return hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
def bind(p,description=None):
 r={'path':str(p),'sha256':sha(p)}
 if description:r['description']=description
 return r
def save(n,x):(B/n).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
source=load(B/'linguistic-input-audit.json')
for x in source['inputFiles']:assert sha(x['path'])==x['currentSha256']
execution=load(B/'usage-execution-audit.json')
for x in execution['sourceBindings']+execution['dependencyBindings']+execution['receipts']+execution['locksAndInstallReports']+execution['independentInspection']:assert sha(x['path'])==x['sha256']
assert sha(B/'usage-benchmark-expectations.json')==sha(P/'usage-benchmark-expectations.json')
audit={'schemaVersion':1,'reviewerRun':'/root/spec_review/pr66-portable-search-usage-2026-11-02','scope':'New browser runs on the portable sorted-index artifact, with new actual queries, clipboard and downloads. Reuse prior independent natural-language reading only after exact visible task body/code equality and unchanged non-Pagefind artifact bytes. Search runtime/result order is newly exercised, not inherited. Public SDK programs were genuinely executed again in this batch, and new APKGs independently decoded.','freshExecutionAudit':bind(B/'usage-execution-audit.json'),'sourceDelta':bind(B/'linguistic-input-audit.json'),'configurations':{},'newIssues':[]}
tours={};reports={}
for mode in ['prefix','root']:
 now=load(B/f'usage-{mode}/browser-tour.json');old=load(P/f'usage-{mode}/browser-tour.json');prior=load(P/f'usage-tasks-{mode}.json');m=load(B/f'{mode}-site-manifest.json')
 assert not now['pageErrors'] and not now['responseReadErrors'] and not now.get('failure')
 assert now['manifestSha256']==sha(B/f'{mode}-site-manifest.json') and now['inputSha256']==m['inputSha256']
 assert now['configuration']==prior['configuration'] and now['releaseLocales']==prior['releaseLocales']
 records={x['route']:x for x in m['pages']+m['resources']};wanted={x['route'] for x in now['loadedResources']};machines={x['route'] for x in m['resources'] if x['route'].endswith('/index.md') or x['route'].endswith('/llms.txt')};assert len(machines)==192;wanted.update(machines)
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
 previousBindings={x['route']:x['sha256'] for x in prior['resourceBindings']};nonSearch={p for p in wanted if '/pagefind/' not in p}
 for p in nonSearch:assert previousBindings.get(p)==records[p]['sha256'],(mode,p,'non-Pagefind resource changed')
 oldNonSearch={p for p in previousBindings if '/pagefind/' not in p};assert oldNonSearch==nonSearch
 search=[{'route':p,'sha256':records[p]['sha256'],'previousSha256':previousBindings.get(p),'status':'same' if previousBindings.get(p)==records[p]['sha256'] else 'changed' if p in previousBindings else 'new'} for p in sorted(wanted-nonSearch)]
 entry=load(f'target/website-pr66-portable-search/{mode}-dist/pagefind/pagefind-entry.json');assert list(entry['languages'])==sorted(entry['languages']);assert all(v['page_count']==34 for v in entry['languages'].values())
 runtime=load('website/vendor/pagefind/1.5.2/runtime.json')
 for name,expected in runtime['files'].items():assert sha(f'target/website-pr66-portable-search/{mode}-dist/pagefind/{name}')==expected['sha256']
 audit['configurations'][mode]={'previousReview':bind(P/f'usage-tasks-{mode}.json'),'currentTour':bind(B/f'usage-{mode}/browser-tour.json'),'currentManifest':bind(B/f'{mode}-site-manifest.json'),'inputSha256':m['inputSha256'],'tasks':taskChecks,'nonPagefindBoundResourceBytesIdentical':True,'nonPagefindCount':len(nonSearch),'actualCurrentPagefindResources':search,'removedOldSearchRoutes':sorted(p for p in previousBindings if '/pagefind/' in p and p not in wanted),'homeAndEditorialCapturedTextEqual':True,'actualNewSearchRegressionResults':now['repairObservations']['wordForms'],'counts':{'tasks':36,'copies':sum(len(t['copiedPrograms']) for es in now['entries'].values() for t in es),'downloads':sum(len(t['downloadedResources']) for es in now['entries'].values() for t in es),'loadedResources':len(now['loadedResources']),'requiredMachineResources':192,'resourceBindings':len(wanted)}}
 tours[mode]=now;reports[mode]=(prior,m,records,wanted)
save('usage-incremental-audit.json',audit)
def ex(paths,description):return [{**bind(B/p,description),'sourceBindings':execution['sourceBindings'],'dependencyBindings':execution['dependencyBindings']} for p in paths]
executions={}
for sdk in ['rust','python','node']:executions[sdk+'-basic']=ex([f'executions/basic/{sdk}/basic.log',f'executions/basic/{sdk}/spanish.apkg',f'executions/basic/{sdk}/inspection.log'],'Fresh public 0.3.1 SDK execution in this portable-search batch. Independently verified actual source copies, recorded current inputs and output APKG. Previous execution is not reused because website dependency bindings changed.')
public=['executions/public-json-run.log']+[f'executions/public-json/{s}/{n}' for s in ['rust','python','node'] for n in ['v1.apkg','v2.apkg','comparison.json','review.json','installation.json']]
executions['json-media']=ex(public,'Fresh current-batch public-package execution in all three SDKs; exact paired APKG payloads and identity/policy independently decoded and verified. No old execution reuse.')
executions['original-baseline-update']=executions['json-media']
executions['troubleshooting']=ex(['executions/basic/python/troubleshooting.log','executions/basic/python/recovered.apkg'],'Fresh current-batch NOTE.KEY_DUPLICATE and unique-key recovery; corrected APKG independently decoded as 2 notes and 2 cards.')
for mode,now in tours.items():
 prior,m,records,wanted=reports[mode];entries={}
 for l,es in now['entries'].items():
  tasks=[]
  for e,oldtask in zip(es,prior['entries'][l]['taskResults']):
   assert e['id']==oldtask['id'];route=urllib.parse.urlparse(e['finalUrl']).path
   evidence=[{'route':route,'sha256':records[route]['sha256'],'observation':'Actually navigated and captured this new localized HTML. Complete task prose, heading, code and clipboard bytes equal the previously independently read page; its natural-language interpretation is explicitly inherited. New portable Pagefind search is exercised separately, not assumed equal.'},bind(B/f'usage-{mode}/browser-tour.json',f'New actual {mode} six-language browsing, clipboard/download actions and portable Pagefind queries; inspect entries[{l}]/{e["id"]}.'),bind(B/'usage-incremental-audit.json','Exact unchanged natural-language evidence and actual complete current search-resource scope; no old Pagefind execution claim.'),bind(F/'usage-independent-conclusions.json','Immutable original full task reading and localized quote, reused only for proven unchanged content.'),bind(P/f'usage-tasks-{mode}.json','Previous completed usage report supplying unchanged reading lineage and unresolved observations.'),bind(B/'usage-execution-audit.json','New same-batch SDK input/receipt audit and actual independent APKG decoder results.'),bind(e['screenshot'],'Actual new task screenshot retained; no new full visual audit claimed for unchanged wording.')]
   evidence.extend(execution['receipts']+[{**x,'description':'Actual new execution dependency lock or installation receipt.'} for x in execution['locksAndInstallReports']]+execution['independentInspection'])
   evidence.extend(bind(d['path'],'Newly downloaded actual task input bytes.') for d in e['downloadedResources'])
   tasks.append({'id':e['id'],'status':'passed','openIssues':oldtask['openIssues'],'readUrls':e['readUrls'],'steps':e['steps']+['Compared full current task body/heading/rendered program/copy bytes with previous independently read capture: exact equality. Existing wording judgments and nonblocking observations remain.','Used genuinely new public SDK execution evidence and independently decoded new outputs. Previous SDK execution was not rebound.'],'executionEvidence':executions[e['id']],'evidence':evidence})
  entries[l]={'taskResults':tasks,'openIssues':prior['entries'][l].get('openIssues',[])}
 report={'schemaVersion':1,'kind':'usage-tasks','status':'passed','mode':'release-candidate','reviewerType':'ai','reviewerRun':f'/root/spec_review/pr66-portable-search-usage-{mode}-2026-11-02','inputSha256':m['inputSha256'],'configuration':m['configuration'],'releaseLocales':m['releaseLocales'],'readOrigin':now['readOrigin'],'originManifest':{**bind(B/f'{mode}-site-manifest.json'),'commit':m['commit']},'resourceBindings':[{'route':p,'sha256':records[p]['sha256']} for p in sorted(wanted)],'resourceScope':'Union of every actual browser-loaded resource, read URL, downloaded input and the required 192 machine texts. All new Pagefind metadata/shards/fragments/WASM that the browser loaded are bound. No unused showcase.json or social-image binding; machine semantic reading is separate.','candidateScope':'Local new portable-search candidate. New actual search/browser actions and genuinely new SDK executions with independent APKG inspection; natural-language reading alone is reused after exact unchanged-content proof. No deployment or native Anki import claimed.','previousReview':bind(P/f'usage-tasks-{mode}.json'),'independentIncrementalAudit':bind(B/'usage-incremental-audit.json'),'sourceReviews':[bind(B/'linguistic-source-review.json'),bind(F/'linguistic-source-review.json')],'visualReview':bind(B/'usage-visual-observations.json'),'repairRegressions':now['repairObservations'],'performanceObservations':{l:{k:v for k,v in h.items() if k!='mainText'} for l,h in now['homeObservations'].items()},'openIssues':prior['openIssues'],'entries':entries}
 save(f'usage-tasks-{mode}.json',report);print(mode,'36 actual tasks;',len(wanted),'bound resources; new Pagefind run + new SDK execution; unchanged wording lineage retained.')
