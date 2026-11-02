"""Serialize independently read current-artifact observations; no old usage pass is reused."""
import hashlib,json,pathlib,subprocess,urllib.parse
B=pathlib.Path('docs/plans/evidence/website-pr66-final-2026-11-02')
def digest(p): return hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
def load(p): return json.loads(pathlib.Path(p).read_text())
def bind(p,description=None):
 r={'path':str(p),'sha256':digest(p)}
 if description: r['description']=description
 return r
def save(name,value): (B/name).write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n')
source=load(B/'linguistic-source-review.json')
assert digest(B/'linguistic-source-review.json')=='65716db6e5eed257017a069a49f1d922a90e5cbe12d2b28b1c4f9cb9f6dcfa2e'
prior_execution=load(B/'executions/basic/execution-inputs.json')
for x in prior_execution['inputs']: assert digest(x['path'])==x['sha256'],x['path']
programs={'rust':'anki_forge/examples/target_api_basic.rs','python':'bindings/python/examples/quickstart.py','node':'bindings/node/examples/quickstart.mjs'}
kept={'rust':'rust/src/main.rs','python':'python/main.py','node':'node/main.mjs'}
for sdk,p in programs.items():assert pathlib.Path(p).read_bytes()==(B/'executions/basic'/kept[sdk]).read_bytes()
sources=list(programs.values())+['scripts/packaged_consumer/src/inspection.rs','scripts/packaged_consumer/src/bin/verify_apkg.rs','scripts/packaged_consumer/src/bin/verify_json_recipe.rs','anki_forge/examples/json_to_anki.rs','bindings/python/examples/json_to_anki.py','bindings/node/examples/json_to_anki.mts','docs/recipes/json-to-anki/v1.json','docs/recipes/json-to-anki/v2.json','docs/recipes/json-to-anki/cell.png','docs/recipes/json-to-anki/tone.wav','docs/troubleshooting.md','docs/zh-cn/troubleshooting.md','website/scripts/verify-public-recipe.mjs']
deps=['docs/ai-quickstart.json','bindings/python/pyproject.toml','bindings/node/package.json','Cargo.toml','Cargo.lock','website/package.json','website/package-lock.json']
# Current public-recipe source is immutable in the reviewed HEAD; the basic runner also records its actual input hashes.
for p in sources+deps: assert subprocess.check_output(['git','show','09d431830568345a9e6ae2bd1cda2154af5a08e1:'+p])==pathlib.Path(p).read_bytes(),p
sb=[bind(p) for p in sources];db=[bind(p) for p in deps]
receipts=[]
for sdk in ['rust','python','node']:
 p=B/f'executions/public-json/{sdk}/installation.json';x=load(p);s=x['source']
 assert s.get('version',s.get('metadata',{}).get('version'))=='0.3.1'
 if sdk=='rust': assert 'registry+https://github.com/rust-lang/crates.io-index' in s['resolution']
 if sdk=='python': assert urllib.parse.urlparse(s['download_info']['url']).hostname=='files.pythonhosted.org'
 if sdk=='node': assert urllib.parse.urlparse(s['resolved']).hostname=='registry.npmjs.org' and s['native'][0][1]['version']=='0.3.1'
 receipts.append(bind(p,'Fresh public-package installation receipt, read and checked for version 0.3.1 and official registry provenance.'))
 for n in ['comparison','review']:
  q=load(B/f'executions/public-json/{sdk}/{n}.json')
  assert q['policy']['allows_publication'] is True
  assert [f['code'] for f in q['findings']]==['RISK.NOTE_CHANGED']
assert 'NOTE.KEY_DUPLICATE' in (B/'executions/basic/python/troubleshooting.log').read_text()
locks=[B/'executions/basic/rust/Cargo.lock',B/'executions/basic/python/pip-report.json',B/'executions/basic/node/package-lock.json',B/'executions/public-json/rust/Cargo.lock']
assert 'version = "0.3.1"' in locks[0].read_text()
assert load(locks[1])['install'][0]['metadata']['version']=='0.3.1'
assert load(locks[2])['packages']['node_modules/ankiforge']['version']=='0.3.1'
audit={'schemaVersion':1,'reviewerRun':'/root/spec_review/pr66-final-usage-2026-11-02','scope':'Fresh executions performed by the coordinating agent in this batch; independent reviewer read the runner, actual logs, exact retained basic programs, public installation receipts, policy JSON, and independently decoded every retained APKG. No old batch SDK execution is rebound. SDK examples were not executed again by this reviewer.','actualBasicInputRecord':bind(B/'executions/basic/execution-inputs.json'),'allRecordedBasicInputsMatchCurrent':True,'basicProgramCopiesMatchCurrent':True,'publicRecipeInputsMatchImmutableCommit':'09d431830568345a9e6ae2bd1cda2154af5a08e1','sourceBindings':sb,'dependencyBindings':db,'receipts':receipts,'locksAndInstallReports':[bind(p) for p in locks],'independentInspection':[bind(B/'usage-apkg-inspection.log','Independently reran existing ZIP/protobuf/SQLite decoder on all 10 actual APKG files: basic 1/1/0; duplicate-key recovery 2/2/0; all six JSON packages 3/3/2.'),bind(B/'usage-json-inspection.log','Independently compiled the current JSON inspector offline with the retained public execution lockfile and decoded all three SDK v1/v2 pairs. Verified two exact fixture media files, stable GUID/model/note mappings, exactly one changed answer and compare/review policy allowed.')],'limits':'No new native Anki client import or cross-platform SDK execution is claimed.'}
save('usage-execution-audit.json',audit)
judgments={
'rust-basic':'The actual localized page provides Rust 1.92+, public package 0.3.1, a complete src/main.rs, cargo run and persistent spanish.apkg with hola → hello. It distinguishes exporting without Anki from importing into Anki, and retains namespace spanish and key es:hola. The actual copied full program matches current source.',
'python-basic':'The actual localized search led to this language Python quickstart. Its virtual-environment setup, Windows activation alternative, ordinary CPython 3.11+ / matching wheel condition, public 0.3.1, main.py and persistent spanish.apkg are coherent. The actual copied complete program matches source; installation coverage is stated as recorded release checks, not newly claimed tests on every runtime.',
'node-basic':'The actual localized sidebar led to Node quickstart. It states Node.js 22.13+, the public npm package with matching native optional dependency, complete ESM main.mjs and persistent output after artifact closure. TypeScript instructions are linked separately. The actual copied program matches current source.',
'json-media':'The actual page supplies all four inputs in recipe/, complete Rust/Python/Node programs and matching runtime/install/run steps. I read the 3 notes / 3 cards / 2 media expectation, one-answer-only update, explicit original v1 baseline, separate output directory and repeated policy check. All three complete programs were copied and all four input URLs downloaded. It keeps client import, local-edit, scheduling and non-deletion limits explicit.',
'original-baseline-update':'The actual localized update text requires the previous original distribution APKG with identity evidence and unchanged namespace/note/model/field/template keys. It distinguishes successful compare from policy permission, retains hard-error and risk-category limits, explains that Anki re-export is unusable, and says preparation does not publish. The accepted candidate remains the reviewed file; client import checks are still required. Fresh three-SDK JSON execution demonstrates this same original-baseline workflow without claiming a new native Anki import.',
 'troubleshooting':'The actual localized table directs NOTE.KEY_DUPLICATE to one unique key per logical source record; it distinguishes operation failure from completed comparison that blocks publication. Atomic add/retry, explicit structured kind/code and original distribution baseline remain clear. The fresh Python log shows AddError NOTE.KEY_DUPLICATE followed by successful corrected output, independently decoded as two notes/two cards.'}
# Specific phrases were read in the captured bodies, not copied from source files or old reports.
needles={
'en':['The program writes','On Windows','Use Node.js','Read three biology','Keep the project namespace','NOTE.KEY_DUPLICATE'],
'zh-CN':['程序写出','Windows 使用','使用 Node.js','从 JSON 读取','保持项目命名空间','NOTE.KEY_DUPLICATE'],
'es':['El programa escribe','En Windows','Usa Node.js','Lee tres preguntas','Mantén estables','NOTE.KEY_DUPLICATE'],
'pt-BR':['O programa grava','No Windows','Use Node.js','Leia três perguntas','Mantenha estáveis','NOTE.KEY_DUPLICATE'],
'ja':['プログラムは永続ファイル','Windows では','Node.js 22.13','JSON にある生物学','プロジェクトの名前空間','NOTE.KEY_DUPLICATE'],
'de':['Das Programm schreibt','Unter Windows','Verwende Node.js','Lies drei Biologiefragen','Behalte den Projektnamensraum','NOTE.KEY_DUPLICATE']}
P=load(B/'usage-prefix/browser-tour.json');R=load(B/'usage-root/browser-tour.json')
conclusions={'schemaVersion':1,'reviewerRun':'/root/spec_review/pr66-final-usage-2026-11-02','scope':'Independent actual current HTML reading for six tasks per locale. Full visible task prose was read via usage-reading-prefix packets; code blocks were separately captured and complete quickstart/JSON programs verified through actual clipboard. The second configuration independently navigated and captured equal body/code/download bytes, established by the exact comparison record. Home performance text and extra examples/blog-list headings were read; supplemental full-page snapshots are retained without claiming a new full editorial audit of all their text.','comparison':bind(B/'usage-dual-configuration-comparison.json'),'sourceIssuePolicy':'All 32 unresolved source minor/preference observations remain in the frozen source report and are carried in report openIssues. A usage pass means no task-blocking defect found, not that all wording preferences were removed.','locales':{}}
for l,es in P['entries'].items():
 c=[]
 for i,e in enumerate(es):
  assert e['id']==R['entries'][l][i]['id'] and e['body']==R['entries'][l][i]['body']
  matching=[line for line in e['body'].splitlines() if needles[l][i] in line]
  assert matching,(l,e['id'],needles[l][i])
  c.append({'id':e['id'],'actualHeading':e['h1'],'actualQuote':matching[0],'judgment':judgments[e['id']]})
 h=P['homeObservations'][l]
 conclusions['locales'][l]={'tasks':c,'performance':{'actualDescription':h['description'],'primaryMetric':h['primaryMetric'],'rangeMetric':h['rangeMetric'],'judgment':'Read current five-workload Rust/API comparison, all five timing rows, 1,000 Basic-note scope and hardware/Anki-import limitation. Displayed decimals agree with current locale-specific expected values; comma locales es/pt-BR/de, decimal-point locales en/zh-CN/ja.'},'extraHeadings':[{'pageId':e['pageId'],'heading':e['heading']} for e in P['editorialObservations'][l]]}
save('usage-independent-conclusions.json',conclusions)
allissues=[]
for page,locs in source['entries'].items():
 for l,entry in locs.items():
  for issue in entry['linguistic']['openIssues']: allissues.append({**issue,'sourcePageId':page,'sourceLocale':l})
assert len(allissues)==32
visual=[('prefix','es-home-numbers.png','Spanish decimal commas and five workload labels fit; original-method limitation remains visible.'),('prefix','de-home-numbers.png','German decimal commas and scoped benchmark explanation fit without clipping.'),('root','zh-CN-home-numbers.png','Chinese note-count and shared/separate-media labels, decimal points and benchmark limitation visible.'),('root','en-home-numbers.png','English scoped five-test wording, decimal points, rows and limitation visible.'),('prefix','pt-BR-home-numbers.png','Brazilian Portuguese decimal commas, media-per-note labels and limitation visible.'),('root','ja-home-numbers.png','Japanese benchmark scope and decimal-point timing rows remain readable; narrow heading wraps but is not clipped.'),('root','ja-stable-ids-and-deck-updates-clipboard-denied.png','Actually inspected Japanese copy button, selected complete Rust snippet and Japanese manual-copy recovery message.')]
visualEvidence=[{**bind(B/f'usage-{m}'/f), 'observation':o} for m,f,o in visual]
pageids={'rust-basic':'quickstart','python-basic':'python-quickstart','node-basic':'node-quickstart','json-media':'json-to-anki','original-baseline-update':'updates','troubleshooting':'troubleshooting'}
def execution(paths,what):
 return [{**bind(B/p,what),'sourceBindings':sb,'dependencyBindings':db} for p in paths]
executions={}
for sdk in ['rust','python','node']:
 executions[sdk+'-basic']=execution([f'executions/basic/{sdk}/basic.log',f'executions/basic/{sdk}/spanish.apkg',f'executions/basic/{sdk}/inspection.log'],'Fresh current-batch public 0.3.1 execution by coordinating agent; independently checked input hashes, exact program copies, package provenance, log and decoded APKG. See usage-execution-audit.json; no old execution reuse.')
publicpaths=['executions/public-recipe.log']+[f'executions/public-json/{sdk}/{name}' for sdk in ['rust','python','node'] for name in ['v1.apkg','v2.apkg','comparison.json','review.json','installation.json']]
executions['json-media']=execution(publicpaths,'Fresh public-package JSON/media execution in all three SDKs. Independently checked public receipt, source/dependency identity and actual paired APKG contents, stable identities, two exact media files, one answer change and allowed policy.')
executions['original-baseline-update']=executions['json-media']
executions['troubleshooting']=execution(['executions/basic/python/troubleshooting.log','executions/basic/python/recovered.apkg'],'Fresh controlled duplicate-key error and correction; original log read, corrected APKG independently decoded. Unique-key correction agrees with all six actual localized pages.')
for mode,tour in [('prefix',P),('root',R)]:
 manifestPath=B/f'{mode}-site-manifest.json';m=load(manifestPath)
 assert tour['manifestSha256']==digest(manifestPath) and tour['inputSha256']==m['inputSha256']
 assert not tour['pageErrors'] and not tour['responseReadErrors'] and not tour.get('failure')
 resources={x['route']:x for x in m['pages']+m['resources']}
 wanted={x['route'] for x in tour['loadedResources']}
 machines={x['route'] for x in m['resources'] if x['route'].endswith('/index.md') or x['route'].endswith('/llms.txt')}
 assert len(machines)==192
 wanted.update(machines)
 for es in tour['entries'].values():
  for e in es:
   wanted.update(urllib.parse.urlparse(u).path for u in e['readUrls'])
   for d in e['downloadedResources']:
    assert digest(d['path'])==d['sha256'];wanted.add(urllib.parse.urlparse(d['url']).path)
   for p in e['copiedPrograms']:
    assert digest(p['source'])==p['sourceSha256']
    assert pathlib.Path(p['source']).read_text().strip()==p['copiedCode'].strip()
 for h in tour['homeObservations'].values():wanted.add(urllib.parse.urlparse(h['url']).path)
 for es in tour['editorialObservations'].values():
  for e in es:wanted.add(urllib.parse.urlparse(e['url']).path)
 for x in tour['repairObservations']['japaneseClipboard']:wanted.add(urllib.parse.urlparse(x['url']).path)
 for x in tour['repairObservations']['wordForms']:wanted.add(urllib.parse.urlparse(x['pageUrl']).path)
 assert not any(u.endswith('/generated/showcase.json') for u in wanted)
 bindings=[{'route':u,'sha256':resources[u]['sha256']} for u in sorted(wanted)]
 entries={}
 for l,es in tour['entries'].items():
  tasks=[]
  for i,e in enumerate(es):
   identifier=e['id'];route=urllib.parse.urlparse(e['finalUrl']).path
   observed=conclusions['locales'][l]['tasks'][i]
   ev=[{'route':route,'sha256':resources[route]['sha256'],'observation':observed['judgment']+' Actual wording: '+observed['actualQuote']},bind(B/f'usage-{mode}/browser-tour.json',f'Actual {mode} browser navigation, main text, rendered code, copied bytes, downloaded files and local URL records for {l}/{identifier}.'),bind(B/'usage-independent-conclusions.json','Independent current task-specific reading, exact localized quote and interpretation.'),bind(B/'usage-execution-audit.json','Fresh independent input/receipt audit and actual APKG inspection scope.'),bind(B/'usage-dual-configuration-comparison.json','Actual separately executed tours match on all 36 task bodies, code and download bytes.'),bind(B/f'usage-reading-prefix-{l}.txt','Full current visible task prose reading packet; rendered code omitted here but bound in the browser capture.'),bind(e['screenshot'],'Actual task screenshot retained for reproducibility; only explicitly listed visualReview images were separately visually inspected.')]
   ev.extend(bind(B/name,'Independent actual current APKG decoding output.') for name in ['usage-apkg-inspection.log','usage-json-inspection.log'])
   ev.extend(receipts)
   ev.extend(bind(p,'Actual retained dependency lock/install report for the fresh execution.') for p in locks)
   ev.extend(bind(d['path'],'Downloaded actual input bytes from this localized task.') for d in e['downloadedResources'])
   issues=source['entries'][pageids[identifier]][l]['linguistic']['openIssues']
   tasks.append({'id':identifier,'status':'passed','openIssues':issues,'readUrls':e['readUrls'],'steps':e['steps']+[observed['judgment'],'Used the fresh same-batch public-package execution and independently decoded output; old execution reports are not reused.'],'executionEvidence':executions[identifier],'evidence':ev})
  entries[l]={'taskResults':tasks,'openIssues':[]}
 report={'schemaVersion':1,'kind':'usage-tasks','status':'passed','mode':'release-candidate','reviewerType':'ai','reviewerRun':f'/root/spec_review/pr66-final-usage-{mode}-2026-11-02','inputSha256':m['inputSha256'],'configuration':m['configuration'],'releaseLocales':m['releaseLocales'],'readOrigin':tour['readOrigin'],'originManifest':{**bind(manifestPath),'commit':m['commit']},'resourceBindings':bindings,'resourceScope':{'description':'Union of all actual browser response resources, read URLs, downloads and the full required machine-text contract. Whole initial artifact integrity is recorded separately and does not claim every unrelated file was used. Machine semantic reading belongs to the independent machine report.','actualLoadedResources':len(tour['loadedResources']),'requiredMachineResources':len(machines),'boundUniqueResources':len(bindings),'showcaseJsonActuallyRequested':False,'unusedShowcaseAndSocialImagesBound':False},'candidateScope':'Local frozen PR66 final candidate only; both build configurations were independently browsed. No deployment or new native Anki import is claimed.','executionScope':audit['scope'],'sourceReviews':[bind(B/'linguistic-source-review.json'),bind(B/'linguistic-heading-review.json')],'independentConclusions':bind(B/'usage-independent-conclusions.json'),'crossConfigurationComparison':bind(B/'usage-dual-configuration-comparison.json'),'repairRegressions':tour['repairObservations'],'performanceObservations':{l:{k:v for k,v in h.items() if k!='mainText'} for l,h in tour['homeObservations'].items()},'editorialScope':'Opened and captured examples and blog-list pages in all six locales; checked headings and language. Full source editorial assessment remains in the separate frozen linguistic report.','visualReview':visualEvidence,'openIssues':allissues,'entries':entries}
 save(f'usage-tasks-{mode}.json',report)
 print(mode,': 36 actual tasks,',len(bindings),'bound resources; 32 existing nonblocking source observations preserved')
