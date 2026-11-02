"""Verify retained actual execution inputs and inspect APKG bytes; never run SDKs."""
import hashlib,json,pathlib,sqlite3,zipfile,subprocess
from compression import zstd
root=pathlib.Path.cwd(); out=pathlib.Path('docs/plans/evidence/multilingual-ja-de-2026-11-01/ja')
old=pathlib.Path('docs/plans/evidence/multilingual-es-integration-2026-10-31/executions')
prior=pathlib.Path('docs/plans/evidence/multilingual-pt-br-2026-11-01/execution-reuse-audit.json')
historic_path=pathlib.Path('docs/plans/evidence/remaining-locales-baseline-2026-11-01/execution-reuse-audit.json')
baseline_commit='231bcd56b71765f71b6700141e7237d8561f20bc'
load=lambda p:json.loads((root/p).read_text())
sha=lambda b:hashlib.sha256(b).hexdigest()
binding=lambda p:{'path':str(p),'sha256':sha((root/p).read_bytes())}
a=load(prior);historic=load(historic_path);current=[];changes=[]
for b in a['currentBindings']:
 n=binding(pathlib.Path(b['path']));current.append(n)
 if n['sha256']!=b['sha256']:
  assert b['path']=='docs/ai-quickstart.json'
  previous_bytes=subprocess.check_output(['git','show',f"{baseline_commit}:{b['path']}"],cwd=root)
  prev=json.loads(previous_bytes); now=load(pathlib.Path(b['path']))
  assert sha(previous_bytes)==b['sha256']
  for k in ('prompts','requestPlaceholders','promptContracts'):
   assert list(now[k])==list(prev[k])+['ja']
   del now[k]['ja']
  assert now==prev
  changes.append({'path':b['path'],'previousSha256':b['sha256'],'currentSha256':n['sha256'],'reason':'Only Japanese editorial prompt/placeholder/contract entries were added. Compared directly to immutable baseline 231bcd56b71765f71b6700141e7237d8561f20bc: publicVersion and all runtime/install commands plus twelve earlier locale/SDK combinations are exactly unchanged. The Japanese prompt is separately semantically reviewed; these additions do not alter any executed consumer program or package dependency.'})
assert len(changes)==1
for sdk,row in historic['executionSources'].items():
 assert binding(pathlib.Path(row['current']['path']))==row['current']
 assert binding(pathlib.Path(row['executed']['path']))==row['executed']
 assert row['current']['sha256']==row['executed']['sha256']
outputs=[]
for prior_output in historic['outputs']:
 f=pathlib.Path(prior_output['file']['path']);assert binding(f)==prior_output['file']
 with zipfile.ZipFile(root/f) as z:
  assert z.testzip() is None
  db=zstd.decompress(z.read('collection.anki21b'));c=sqlite3.connect(':memory:');c.deserialize(db)
  assert c.execute('pragma integrity_check').fetchone()[0]=='ok'
  notes=[list(r)for r in c.execute('select guid,flds from notes order by guid')]
  cards=[r[0]for r in c.execute('select ord from cards order by nid,ord')]
  identity=json.loads(z.read('ankiforge-identity.json'))
  media=[]
  for name in z.namelist():
   if name.isdigit():
    payload=z.read(name)
    if payload.startswith(bytes.fromhex('28b52ffd')):payload=zstd.decompress(payload)
    media.append(sha(payload))
  assert notes==prior_output['notes'];assert sorted(cards)==sorted(prior_output['cardOrdinals']);assert sorted(media)==sorted(prior_output['mediaSha256'])
  assert identity==prior_output['identity'];assert identity['format_version']=='ankiforge-identity-v1'
  logical=identity['identity'];assert {n['guid']for n in logical['notes'].values() if n['active']}=={n[0]for n in notes}
  if 'public-json' in str(f):
   assert len(notes)==len(cards)==3 and len(media)==2
   assert set(media)=={binding(pathlib.Path('docs/recipes/json-to-anki/cell.png'))['sha256'],binding(pathlib.Path('docs/recipes/json-to-anki/tone.wav'))['sha256']}
   assert logical['namespace']=='biology-json'
  elif f.name=='spanish.apkg':assert len(notes)==len(cards)==1 and notes[0][1]=='hola\x1fhello' and not media
  else:assert f.name=='recovered.apkg' and len(notes)==len(cards)==2 and not media
  outputs.append({'file':binding(f),'notes':notes,'cards':len(cards),'cardOrdinals':cards,'mediaSha256':sorted(media),'namespace':logical['namespace'],'noteIdentity':logical['notes'],'sqliteIntegrity':'ok'})
for sdk in ('rust','python','node'):
 v1=next(x for x in outputs if x['file']['path'].endswith(f'public-json/{sdk}/v1.apkg'));v2=next(x for x in outputs if x['file']['path'].endswith(f'public-json/{sdk}/v2.apkg'))
 assert {r[0]for r in v1['notes']}=={r[0]for r in v2['notes']}
 changed=[k for k in v1['noteIdentity'] if v1['noteIdentity'][k]['content_hash']!=v2['noteIdentity'][k]['content_hash']]
 assert changed==['bio:cell'];assert next(r[1]for r in v2['notes']if r[0]==v2['noteIdentity']['bio:cell']['guid']).endswith('\x1fThe basic structural and functional unit of life')
reports=[]
for r in a['comparisonReports']:
 f=pathlib.Path(r['file']['path']);assert binding(f)==r['file'];j=load(f)
 assert j['policy']['allows_publication'] is True and j['highest_risk']=='low'
 assert len(j['findings'])==1 and j['findings'][0]['code']=='RISK.NOTE_CHANGED'
 reports.append({'file':binding(f),'highestRisk':'low','policyAllowsPublication':True,'findings':['RISK.NOTE_CHANGED']})
logs=[]
for sdk in ('rust','python','node'):
 for name in ('basic.log','install.log','inspection.log'):
  f=old/'basic'/sdk/name;body=(root/f).read_text();assert 'exit 0' in body;logs.append(binding(f))
 assert '0.3.1' in (root/old/'basic'/sdk/'install.log').read_text()
f=old/'basic/python/troubleshooting.log';body=(root/f).read_text();assert 'NOTE.KEY_DUPLICATE' in body and 'exit 0' in body;logs.append(binding(f))
f=old/'public-recipe.log';body=(root/f).read_text();assert body.count('Verified 3 notes, 3 cards, 2 exact media assets; stable identities; one answer change; compare and publication allowed.')==3;logs.append(binding(f))
for sdk in ('rust','python','node'):
 f=old/'public-json'/sdk/'installation.json';j=load(f);version=j['source'].get('version',j['source'].get('metadata',{}).get('version'));assert version=='0.3.1';logs.append(binding(f))
prior_logs={row['path']:row['sha256'] for row in a['executionLogAndInstallationBindings']}
for row in logs: assert row['sha256']==prior_logs[row['path']]
deps={'Cargo.toml','Cargo.lock','bindings/python/pyproject.toml','bindings/node/package.json','docs/ai-quickstart.json','website/package.json','website/package-lock.json'}
source=[b for b in current if b['path']not in deps];dependency=[b for b in current if b['path']in deps]
def execution(f,d):return dict(binding(f),description=d,sourceBindings=source,dependencyBindings=dependency)
bytask={}
for sdk in ('rust','python','node'):
 bytask[sdk+'-basic']=[execution(old/'basic'/sdk/'basic.log','Explicit reuse of actual successful public 0.3.1 execution; source and runtime/package inputs re-audited.'),execution(old/'basic'/sdk/'spanish.apkg','Retained actual APKG, freshly decoded with Python ZIP/Zstandard/SQLite and checked against current inputs.')]
for task in ('json-media','original-baseline-update'):
 bytask[task]=[execution(old/'public-recipe.log','Explicit reuse of three real public SDK recipe executions. All six APKGs were freshly inspected for exact media, stable GUIDs, one-answer change and publication policy.')]
bytask['troubleshooting']=[execution(old/'basic/python/troubleshooting.log','Explicit reuse of real duplicate-key error and corrected two-note output; code and recovered APKG rechecked.')]
result={'schemaVersion':1,'reviewerRun':'/root/ja_de_infra_audit/ja-execution-reuse-2026-11-01','status':'passed','scope':'Actual earlier SDK executions are explicitly reused, not rerun. This reviewer freshly checked inputs, logs, package resolutions, ten retained ZIP/Zstandard/SQLite outputs and six comparison/review reports. Only AI prompt editorial locale data changed and its execution-relevant values are exactly equal.','priorAudit':binding(prior),'immutableBaseline':baseline_commit,'originalInspectionLineage':binding(historic_path),'currentBindings':current,'explainedNonExecutableChanges':changes,'outputs':outputs,'comparisonReports':reports,'executionLogAndInstallationBindings':logs,'executionEvidenceByTask':bytask,'openIssues':[]}
(root/out/'execution-reuse-audit.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
print(f'Checked {len(current)} current bindings, {len(outputs)} APKGs, {len(reports)} comparisons, {len(logs)} logs/resolutions; actual execution explicitly reused.')
