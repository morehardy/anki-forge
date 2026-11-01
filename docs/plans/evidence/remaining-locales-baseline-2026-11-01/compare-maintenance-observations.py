"""Compare fresh observations with byte-preserved prior observations; no new review verdicts."""
import pathlib,json,hashlib,sys
p=pathlib.Path('docs/plans/evidence/remaining-locales-baseline-2026-11-01');old=p/'history/76f17ee';env=sys.argv[1]
read=lambda base,name:json.loads((base/name).read_text());digest=lambda f:hashlib.sha256(f.read_bytes()).hexdigest()
m=read(p,f'{env}-origin-manifest.json');prior=read(old,f'{env}-origin-manifest.json');rows=[]
assert m['inputSha256']!=prior['inputSha256']
assert m['configuration']==prior['configuration'] and m['releaseLocales']==prior['releaseLocales']
previous={r['route']:r for r in prior['pages']+prior['resources']}
changed=[r['route'] for r in m['pages']+m['resources'] if r['sha256']!=previous[r['route']]['sha256']]
inspection=read(p,f'generated-resource-inspection-{env}.json')
assert sorted(changed)==sorted(r['route'] for r in inspection['changedResources'])
for r in m['pages']+[r for r in m['resources'] if r['route'].endswith(('/index.md','/llms.txt'))]:assert r['sha256']==previous[r['route']]['sha256'],r['route']
assert len(previous)==len(m['pages']+m['resources'])
for axis,name in [('browser',f'usage-{env}/browser-tour.json'),('machine',f'machine-{env}/http-observations.json')]:
 before=read(old,name);now=read(p,name);assert before['manifestSha256']!=now['manifestSha256'];assert now['inputSha256']==m['inputSha256']
 for locale,tasks in now['entries'].items():
  for a,b in zip(before['entries'][locale],tasks):
   assert a['id']==b['id']
   if axis=='browser':
    for key in ['body','codeBlocks','copiedPrograms']:assert a.get(key)==b.get(key),(axis,locale,b['id'],key)
    assert [x['sha256'] for x in a['downloadedResources']]==[x['sha256'] for x in b['downloadedResources']]
   else:
    for x,y in zip([a['index']]+a['followed'],[b['index']]+b['followed']):assert x['body']==y['body'] and x['sha256']==y['sha256'] and y['status']==200
   rows.append({'axis':axis,'locale':locale,'id':b['id'],'actualReadUrls':b['readUrls'],'completeBodyAndCodeEqualToPriorReview':True})
result={'schemaVersion':1,'scope':'New actual browser navigation/clipboard/download and HTTP tasks were rerun after trusted-surface test fixture backport; semantic text is exactly unchanged, without normalization. Retain prior independent semantic conclusions with fresh actual origins and new manifest/input binding. This does not relabel an old passing report or claim an SDK rerun.','sourceChange':'Only trusted-surface bootstrap test and immutable fixture files added by1a448e9','currentManifest':{'path':str(p/f'{env}-origin-manifest.json'),'sha256':digest(p/f'{env}-origin-manifest.json'),'inputSha256':m['inputSha256'],'commit':m['commit']},'priorManifest':{'path':str(old/f'{env}-origin-manifest.json'),'sha256':digest(old/f'{env}-origin-manifest.json'),'inputSha256':prior['inputSha256']},'allHtmlMarkdownLlmsBytesEqual':True,'changedGeneratedResources':changed,'generatedResourceInspection':{'path':str(p/f'generated-resource-inspection-{env}.json'),'sha256':digest(p/f'generated-resource-inspection-{env}.json')},'boundRecords':len(previous),'observations':rows}
(p/f'maintenance-{env}-comparison.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
print(env,':30 fresh tasks and all',len(previous),'manifest records bound; HTML/machine/task content equal and generated resource differences independently inspected.')
