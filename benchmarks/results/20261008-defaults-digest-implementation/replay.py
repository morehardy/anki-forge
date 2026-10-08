"""Verify retained evidence and recalculate medians; does not rerun exporters."""
from pathlib import Path
import hashlib,json,statistics,tarfile
D=Path(__file__).resolve().parent
def sha(data):return hashlib.sha256(data).hexdigest()
checks=json.loads((D/'evidence-sha256.json').read_text())
for name,digest in checks.items():assert sha((D/name).read_bytes())==digest,name
manifest=json.loads((D/'archive-manifest.json').read_text())['files']
with tarfile.open(D/'evidence.tar.gz','r:gz') as tar:
 members={m.name:m for m in tar.getmembers()};documents={}
 for name,item in manifest.items():
  member=members['evidence/'+name];assert member.isfile(),name
  data=tar.extractfile(member).read();assert len(data)==item['bytes'] and sha(data)==item['sha256'],name
  if name.endswith('.json') or name.endswith('stdout.log'):
   try:documents[name]=json.loads(data)
   except (ValueError,UnicodeDecodeError):pass
 groups={name.rsplit('/',1)[0]:data for name,data in documents.items() if name.startswith('records/') and name.endswith('/results.json')}
 counts={'exports':0,'anki_imports':0,'timing':0,'rss':0,'warmup':0,'smoke':0,'trace':0};logical={}
 for folder,result in groups.items():
  assert result['status']=='passed',folder
  plan=documents[folder+'/plan.json'];assert len(result['rows'])==len(plan['schedule'])
  for case,row in zip(plan['schedule'],result['rows']):
   assert all(row[k]==v for k,v in case.items())
   path=row['directory'];measurement=documents[path+'/measurement.json'];collector=documents[path+'/collector.json']
   assert collector['returncode']==0 and json.loads(collector['stdout'])==measurement
   assert measurement['elapsed_ns']/1e6==row['elapsed_ms']
   assert measurement['peak_rss_bytes']/2**20==row['rss_mib']
   verified=documents[path+'/verification.json'];assert verified['status']=='passed'
   assert verified['physical']['logical_sha256']==row['logical_sha256']
   logical.setdefault(row['profile'],set()).add(row['logical_sha256'])
   if row['oracle']:
    assert documents[path+'/oracle-process.json']['returncode']==0
    assert documents[path+'/anki.json']['status']=='passed';counts['anki_imports']+=1
   counts['exports']+=1;counts[row['role']]+=1
 assert all(len(h)==1 for h in logical.values())
 assert counts==documents['audit.json']['counts']
 cells=0
 for group,profiles in documents['analysis.json'].items():
  rows=groups['records/'+group]['rows']
  for profile,data in profiles.items():
   for mode,cell in data['cells'].items():
    measured=[r for r in rows if r['profile']==profile and r['mode']==mode]
    for metric in ['elapsed_ms','input_ms','operation_ms','artifact_bytes']:
     values=[r[metric] for r in measured if r['role']=='timing']
     assert values==cell[metric]['samples']
     assert statistics.median(values)==cell[metric]['median']
    values=[r['rss_mib'] for r in measured if r['role']=='rss']
    assert values==cell['rss_mib']['samples'] and statistics.median(values)==cell['rss_mib']['median']
    if mode!=data['baseline']:
     baseline=data['cells'][data['baseline']]['elapsed_ms']['median']
     assert abs(100*(1-cell['elapsed_ms']['median']/baseline)-cell['elapsed_reduction_percent'])<1e-10
    cells+=1
 for name,source in [('source-manifest.json','source'),('baseline-source-manifest.json','baseline-source')]:
  for path,digest in documents[name].items():assert manifest[source+'/'+path]['sha256']==digest,path
 out={'status':'passed','archive_files':len(manifest),'reconciled_counts':counts,'recalculated_cells':cells,'scope':'Archive hashes, recorded collector/verification/Anki results, frozen sources and statistics. No new export, Anki import or check of omitted binaries/media/APKG bytes.'}
(D/'replay-result.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps(out,indent=2))
