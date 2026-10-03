"""Seal one full matrix plus separately labelled exploratory and confirmation evidence."""
from pathlib import Path
import hashlib,json,shutil,tarfile
W=Path(__file__).resolve().parent;R=W.parents[2];D=R/'benchmarks/results/20261002-spool-genanki';D.mkdir(exist_ok=True);assert not (D/'SHA256SUMS').exists(), 'sealed evidence cannot be overwritten';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
# Compact plans, records and logs only; source copies stay inside this deliberate archive.
for p in sorted(W.iterdir()):
 if p.is_file() and p.suffix in {'.py','.json','.log','.patch','.md','.csv','.png','.svg','.gz'} and p.name not in {'archive.log'}:
  shutil.copy2(p,D/p.name)
run=R/'benchmarks/.work/runs/20261002-spool-genanki';smoke=R/'benchmarks/.work/runs/20261002-spool-smoke';shutil.copy2(run/'manifest.json',D/'run-manifest.json')
manifest={}
def pack(name,entries):
 hashes={}
 with tarfile.open(D/name,'w:gz') as archive:
  for p,key in entries:
   assert p.is_file() and key not in hashes and p.suffix not in {'.apkg','.sqlite','.db'}
   hashes[key]={'sha256':sha(p),'bytes':p.stat().st_size};archive.add(p,arcname=key,recursive=False)
 with tarfile.open(D/name) as archive:
  members=archive.getmembers();assert len(members)==len(hashes)
  for m in members:assert m.isfile() and hashlib.sha256(archive.extractfile(m).read()).hexdigest()==hashes[m.name]['sha256']
 manifest[name]={'sha256':sha(D/name),'members':hashes,'member_count':len(hashes),'members_verified':True,'archive_bytes':(D/name).stat().st_size}
entries=[]
for prefix,folder in [('run',run),('smoke',smoke)]:
 for p in sorted(folder.rglob('*')):
  if p.is_file():entries.append((p,prefix+'/'+str(p.relative_to(folder))))
pack('measurements-and-validation.tar.gz',entries)
entries=[]
for stem in ['pilot','pilot2','confirmation']:
 for case in sorted(W.glob(stem+'-*')):
  if case.is_dir():
   for p in sorted(case.rglob('*')):
    if p.is_file():entries.append((p,'diagnostic/'+str(p.relative_to(W))))
 for p in sorted(W.glob(stem+'-*.json')):entries.append((p,'diagnostic/'+p.name))
for name in ['pilot.py','pilot2.py','confirmation.py','audit-confirmation.py','pilot-version1.patch','spool-reopen.rs','spool-active-writer.rs','assets-before.rs','snapshot-before.rs']:
 entries.append((W/name,'diagnostic/'+name))
pack('confirmation-and-probes.tar.gz',entries)
# Validate the already frozen source archive, including the untracked new module.
source=json.loads((D/'source-snapshot.json').read_text())['source_files'];members={}
with tarfile.open(D/'source-and-inputs.tar.gz') as archive:
 for m in archive:
  assert m.isfile();digest=hashlib.sha256(archive.extractfile(m).read()).hexdigest();members[m.name]={'sha256':digest,'bytes':m.size}
  if m.name.startswith('source/'):assert source[m.name.removeprefix('source/')]==digest
assert 'source/anki_forge/src/media/snapshot/spool.rs' in members
manifest['source-and-inputs.tar.gz']={'sha256':sha(D/'source-and-inputs.tar.gz'),'members':members,'member_count':len(members),'members_verified':True,'archive_bytes':(D/'source-and-inputs.tar.gz').stat().st_size}
(D/'archive-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps({k:{'members':v['member_count'],'bytes':v['archive_bytes'],'verified':v['members_verified']} for k,v in manifest.items()},indent=2))
