from pathlib import Path
import hashlib,json,tarfile
R=Path(__file__).resolve().parent;result={}
for name,record in json.loads((R/'archive-manifest.json').read_text()).items():
 p=R/name;assert hashlib.sha256(p.read_bytes()).hexdigest()==record['sha256']
 with tarfile.open(p) as archive:
  members=archive.getmembers();assert len(members)==record['member_count'];assert len({m.name for m in members})==len(members)
  for m in members:assert m.isfile() and hashlib.sha256(archive.extractfile(m).read()).hexdigest()==record['members'][m.name]['sha256']
 result[name]={'members':len(members),'verified':True}
print(json.dumps(result,indent=2))
