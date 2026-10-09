"""Archive the completed session and supporting records, without rerunning exports."""
from pathlib import Path
import hashlib,json,shutil,tarfile
W=Path(__file__).resolve().parent;R=W.parents[2]
D=R/'benchmarks/results/20261008-media-defaults-genanki';D.mkdir(exist_ok=True)
RUN=R/'benchmarks/.work/runs/20261008-media-defaults-genanki'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
assert json.loads((W/'completed.json').read_text())['status']=='completed'
assert json.loads((W/'verification-summary.json').read_text())['status']=='passed'
for p in W.iterdir():
 if p.is_file() and p.suffix in {'.py','.json','.csv','.md','.svg','.png','.patch','.gz'}:shutil.copy2(p,D/p.name)
archives={}
with tarfile.open(D/'source-and-inputs.tar.gz') as archive:
 archives['source-and-inputs.tar.gz']={m.name:hashlib.file_digest(archive.extractfile(m),'sha256').hexdigest() for m in archive if m.isfile()}
records={}
with tarfile.open(D/'measurements-and-validation.tar.gz','w:gz') as archive:
 for prefix,base in [('run',RUN),('smoke',R/'benchmarks/.work/runs/20261008-media-defaults-genanki-smoke')]:
  for p in sorted(base.rglob('*')):
   if p.is_file() and p.suffix!='.apkg':
    assert not p.is_symlink()
    name=prefix+'/'+str(p.relative_to(base));records[name]=sha(p);archive.add(p,arcname=name,recursive=False)
 for p in sorted(W.glob('*.log')):
  name='logs/'+p.name;records[name]=sha(p);archive.add(p,arcname=name,recursive=False)
archives['measurements-and-validation.tar.gz']=records
(D/'archive-manifest.json').write_text(json.dumps({'schema':'archive-sha256-v1','archives':archives,'scope':{'exports':840,'anki_checks':40,'binaries_omitted':True,'media_bytes_omitted':True,'apkg_deleted_after_validation':True}},indent=2)+'\n')
print(json.dumps({'destination':str(D),'archives':{name:len(entries) for name,entries in archives.items()}},indent=2))
