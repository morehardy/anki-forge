"""Check actual final capture bytes before any independent semantic reuse."""
from pathlib import Path
from urllib.parse import urlparse
import json,hashlib,sys
root=Path.cwd();base=root/'docs/plans/evidence/multilingual-ja-de-2026-11-01/de';env=sys.argv[1];assert env in ('prefix','root')
sha=lambda b:hashlib.sha256(b).hexdigest()
def load(p):return json.loads(p.read_text())
def proof(p):return {'path':str(p.relative_to(root)),'sha256':sha(p.read_bytes())}
currentpath=base/f'usage-final-{env}/browser-tour.json';priorpath=base/('usage-prefix/browser-tour.json' if env=='prefix' else 'usage-final-prefix/browser-tour.json');current=load(currentpath);prior=load(priorpath)
assert current['commit']=='7b5f539ec2b8c34a0a9ac052531cda9571e97725' and current['inputSha256']=='c1c3b90c31ad677b850f62e53bb1634d8a1696034faa7543fc877246ee6c0695'
manifestpath=base/f'final-{env}-origin-manifest.json';manifest=load(manifestpath);assert sha(manifestpath.read_bytes())==current['manifestSha256']
records={x['route']:x for x in manifest['pages']+manifest['resources']}
checks=[];copies=downloads=identical_screens=0
for locale,tasks in current['entries'].items():
 for task in tasks:
  before=next(x for x in prior['entries'][locale] if x['id']==task['id'])
  for field in ['body','codeBlocks','copiedPrograms','h1','steps']:assert task[field]==before[field],(locale,task['id'],field)
  assert [x['sha256']for x in task['downloadedResources']]==[x['sha256']for x in before['downloadedResources']]
  for c in task['copiedPrograms']:
   assert sha((root/c['source']).read_bytes())==c['sourceSha256'];assert (root/c['source']).read_text().strip()==c['copiedCode'].strip();assert sha(c['copiedCode'].strip().encode())==c['clipboardSha256'];copies+=1
  for d in task['downloadedResources']:
   assert d['httpStatus']==200 and sha((root/d['path']).read_bytes())==d['sha256'];assert records[urlparse(d['url']).path]['sha256']==d['sha256'];downloads+=1
  for url in task['readUrls']+[task['finalUrl']]+[x['url']for x in task['downloadedResources']]:
   parsed=urlparse(url);assert parsed.scheme+'://'+parsed.netloc==current['readOrigin'];assert parsed.path in records
   assert parsed.path.startswith('/anki-forge/') if env=='prefix' else not parsed.path.startswith('/anki-forge/')
  assert sha((root/task['screenshot']).read_bytes())==task['screenshotSha256']
  identical=task['screenshotSha256']==before['screenshotSha256'];identical_screens+=identical
  checks.append({'locale':locale,'task':task['id'],'bodySha256':sha(task['body'].encode()),'priorBodySha256':sha(before['body'].encode()),'bodyIdentical':True,'codeBlocksIdentical':True,'clipboardProgramsIdentical':True,'downloadBytesIdentical':True,'currentReadUrls':task['readUrls'],'currentScreenshot':proof(root/task['screenshot']),'screenshotByteIdentical':identical})
for record in current['loadedResources']:assert record['status']==200 and records[record['route']]['sha256']==record['sha256']
assert len(checks)==36 and copies==36 and downloads==24
machine=[x for x in manifest['resources']if x['route'].endswith(('/index.md','/llms.txt'))];assert len(machine)==192
layoutchecks=[]
if env=='prefix':
 lp=base/'layout-final-prefix/observations.json';layout=load(lp);old=load(base/'layout-prefix/observations.json');assert layout['manifestSha256']==current['manifestSha256'] and not layout['pageErrors']
 for x in layout['observations']:
  before=next(o for o in old['observations']if (o['id'],o['width'],o['scale'])==(x['id'],x['width'],x['scale']))
  assert x['geometry']==before['geometry'] and x['fonts']==before['fonts'];assert x['geometry']['scrollWidth']==x['geometry']['viewport'] and x['geometry']['lang']=='de'
  assert records[x['route']]['sha256']==x['sha256'];assert sha((root/x['screenshot']).read_bytes())==x['screenshotSha256']
  layoutchecks.append({'id':x['id'],'width':x['width'],'scale':x['scale'],'geometryUnchanged':True,'fontsUnchanged':True,'screenshotByteIdentical':x['screenshotSha256']==before['screenshotSha256'],'screenshot':proof(root/x['screenshot'])})
 assert len(layoutchecks)==16
result={'schemaVersion':1,'reviewerRun':'/root/ja_full_author/independent-final-de-usage-2026-11-01','status':'actual-byte-checks-completed','scope':'Actual fresh final browser action and resource evidence; byte comparison enables explicit semantic reuse only after independent reviewer assessment. No new SDK execution or full unchanged-prose reread claimed.','environment':env,'sourceCommit':current['commit'],'inputSha256':current['inputSha256'],'currentTour':proof(currentpath),'priorTour':proof(priorpath),'currentManifest':proof(manifestpath),'counts':{'tasks':len(checks),'clipboardOperations':copies,'downloads':downloads,'machineResources':len(machine),'taskScreenshotsByteIdenticalToPrior':identical_screens},'bodyAndActionChecks':checks,'layoutChecks':layoutchecks,'issues':[]}
if env=='prefix':result['ogImage']={'current':proof(base/'layout-final-prefix/og-de.png'),'byteIdenticalToPrior':(base/'layout-final-prefix/og-de.png').read_bytes()==(base/'layout-prefix/og-de.png').read_bytes()}
out=base/f'review/actual-final-{env}-body-and-action-checks.json';assert not out.exists();out.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n');print(json.dumps({'path':str(out.relative_to(root)),'sha256':sha(out.read_bytes()),'counts':result['counts'],'identicalLayoutScreenshots':sum(x['screenshotByteIdentical']for x in layoutchecks),'og':result.get('ogImage'),'issues':[]},ensure_ascii=False,indent=2))
