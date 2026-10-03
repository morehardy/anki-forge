from pathlib import Path
import hashlib,json,tarfile
R=Path(__file__).resolve().parent
result={}
for name,record in json.loads((R/'archive-manifest.json').read_text()).items():
 p=R/name;assert hashlib.sha256(p.read_bytes()).hexdigest()==record['sha256']
 with tarfile.open(p) as archive:
  members=archive.getmembers();assert len(members)==record['member_count']
  assert len({m.name for m in members})==len(members)
  for m in members:
   assert m.isfile() and hashlib.sha256(archive.extractfile(m).read()).hexdigest()==record['members'][m.name]
 result[name]={'verified':True,'members':len(members)}
manifest=json.loads((R/'archive-manifest.json').read_text())['source-and-probes.tar.gz']['members']
for stem,folder in [('probe','probe-source'),('detail','detail-source'),('historical','historical-source')]:
 record=json.loads((R/f'build-{stem}.json').read_text())
 for path,digest in record['source_files'].items():assert manifest['source/'+folder+'/'+path]==digest
 result[stem+'-source']={'build_source_hashes_match':True,'files':len(record['source_files'])}
print(json.dumps(result,indent=2))
