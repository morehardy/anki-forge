"""Record actual root-task evidence with exact reuse of sealed prefix semantic reads."""
from pathlib import Path
import json,hashlib
root=Path.cwd();base=root/'docs/plans/evidence/multilingual-ja-de-2026-11-01/ja'
sha=lambda b:hashlib.sha256(b).hexdigest()
def proof(p,description):return {'path':str(p.relative_to(root)),'sha256':sha(p.read_bytes()),'description':description}
prefixpath=base/'usage-prefix/browser-tour.json';prefix=json.loads(prefixpath.read_text());tourpath=base/'usage-root/browser-tour.json';tour=json.loads(tourpath.read_text());priorpath=base/'usage-task-conclusions-prefix.json';prior=json.loads(priorpath.read_text());reviewpath=base/'usage-tasks-prefix.json'
assert sha(reviewpath.read_bytes())=='0e460dbe3d97e6a495392bf80c79c494974f101d7eca7b64501ec29828a87e3e'
assert sha(priorpath.read_bytes())=='23fb275fc0d0e411f04742627813e9b4df24629fc18ebd72692b76fdf5aa7bd5'
assert tour['configuration']['base']=='/' and tour['mode']=='release-candidate'
assert tour['inputSha256']==prefix['inputSha256'] and tour['commit']==prefix['commit']
entries={};comparisons=[];copies=downloads=0
for locale,tasks in tour['entries'].items():
 entries[locale]={}
 for task in tasks:
  ident=task['id'];previous=next(x for x in prefix['entries'][locale] if x['id']==ident)
  assert task['body']==previous['body'] and task['codeBlocks']==previous['codeBlocks']
  assert task['copiedPrograms']==previous['copiedPrograms']
  for copied in task['copiedPrograms']:
   assert (root/copied['source']).read_text().strip()==copied['copiedCode'].strip();copies+=1
  assert [x['sha256']for x in task['downloadedResources']]==[x['sha256']for x in previous['downloadedResources']]
  for download in task['downloadedResources']:
   assert download['httpStatus']==200 and '/anki-forge/' not in download['url']
   assert sha((root/download['path']).read_bytes())==download['sha256'];downloads+=1
  old=prior['observations']['prefix']['entries'][locale][ident]
  entries[locale][ident]={'status':'passed','openIssues':[],'bodySha256':sha(task['body'].encode()),'method':'exact-prefix-body-reuse-after-independent-root-comparison','conclusion':'For this root-deployment task, individually compared the entire actual main text and rendered code with the sealed independently reviewed five-language prefix task: byte-identical. The root observer separately reran localized navigation/search, real clipboard operations and linked HTTP downloads against the root artifact and its own manifest. Reuse the following prior semantic judgment unchanged in meaning; this is not a new full prose read or SDK execution: '+old['conclusion'],'reusedBody':{'tour':proof(prefixpath,'Actual five-language prefix bodies, individually compared with the root run.'),'review':proof(reviewpath,'Sealed independently interpreted prefix usage review; its full-read/reuse boundary applies.')}}
  comparisons.append({'locale':locale,'task':ident,'prefixBodySha256':sha(previous['body'].encode()),'rootBodySha256':sha(task['body'].encode()),'byteIdentical':True,'renderedCodeBlocksIdentical':True,'clipboardProgramsIdentical':True,'downloadBytesIdentical':True})
assert len(comparisons)==30 and copies==30 and downloads==20
out={'schemaVersion':1,'reviewerRun':'/root/de_full_author/independent-ja-rendered-usage-2026-11-01','scope':'Root/integrator performed 30 fresh browser tasks against the formal root-deployment artifact, including 30 complete-program clipboard operations and 20 HTTP downloads. This independent reviewer examined every root task action/URL record, compared all 30 actual main bodies and rendered code blocks exactly with the sealed five-language prefix read, checked clipboard and downloaded bytes, and visually inspected current Japanese Python/JSON screenshots. Reuse preserves the prefix boundary: Japanese six-task semantics were independently fully read there; unchanged four-language semantics were reused only after exact prior-body comparison. Actual earlier SDK runs/APKGs remain explicitly reused through the current independent audit. No SDK rerun, native Anki import UI run, new root full-prose reread or root narrow-layout sweep is claimed.','executionAudit':prior['executionAudit'],'localizedDuplicateKeyRows':prior['localizedDuplicateKeyRows'],'observations':{'root':{'status':'passed','openIssues':[],'browserTour':proof(tourpath,'Actual formal five-language root browser actions and resources.'),'entries':entries}},'prefixReview':proof(reviewpath,'Sealed prefix semantic review reused after exact-body checks.'),'priorBodyComparisons':comparisons,'visualReview':{'scope':'Two current root-deployment Japanese desktop screenshots actually visually examined; broader prefix visual findings are retained only in the separate prefix review.','conclusion':'The root Japanese Python quickstart and JSON recipe screenshots preserve clear Japanese glyphs, visible titles/instructions/code and complete input anchors. No visual task blocker appears in these two current desktop views. No root narrow-layout or browser-UI zoom conclusion is inferred.','evidence':[proof(base/'usage-root'/n,'Current root Japanese screenshot visually inspected by the independent task reviewer.')for n in ['ja-python-basic.png','ja-json-media.png']]}}
p=base/'usage-task-conclusions-root.json';assert not p.exists();p.write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n');print('root conclusions SHA',sha(p.read_bytes()))
