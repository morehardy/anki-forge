"""Archive the completed round without changing source or measurement records."""
from pathlib import Path
import hashlib,json,shutil,tarfile
W=Path(__file__).resolve().parent; R=W.parents[2]
D=R/'benchmarks/results/20261002-active-writer-genanki'
RUN=R/'benchmarks/.work/runs/20261002-active-writer-genanki'
D.mkdir(exist_ok=False)
def digest(p):
 with p.open('rb') as s:return hashlib.file_digest(s,'sha256').hexdigest()
def read(p):return json.loads(p.read_text())
for p in W.iterdir():
 if p.is_file() and p.suffix in {'.json','.py','.log','.csv','.patch','.md','.svg','.png'}:
  shutil.copy2(p,D/p.name)
shutil.copy2(W/'source-and-inputs.tar.gz',D/'source-and-inputs.tar.gz')
shutil.copy2(RUN/'manifest.json',D/'run-manifest.json')
shutil.copy2(R/'benchmarks/results/20261002-reader-cache-genanki/verify-archives.py',D/'verify-archives.py')
manifest={'schema':'archive-sha256-v1','archives':{}}
def archive(name, entries):
 expected={}
 with tarfile.open(D/name,'w:gz') as tar:
  for p,member in sorted(entries,key=lambda e:e[1]):
   assert p.is_file() and not p.is_symlink(),p
   assert member not in expected,member
   expected[member]=digest(p)
   tar.add(p,arcname=member,recursive=False)
 manifest['archives'][name]=expected

def tree(root,prefix):
 return [(p,prefix+'/'+str(p.relative_to(root))) for p in root.rglob('*') if p.is_file()]
entries=tree(RUN,'run')
profiles=set()
for stem in ('import','fixed','budget','hash','confirmation'):
 for suffix in ('config','plan','results','summary'):
  p=W/f'{stem}-{suffix}.json';entries.append((p,'diagnostic/'+p.name))
 for row in read(W/f'{stem}-results.json')['rows']:
  name=f"{stem}-{row['role']}-{row['repeat']}-{row['profile']}-{row['mode']}"
  assert (W/name).is_dir(),name
  entries.extend(tree(W/name,'diagnostic/'+name))
  profiles.add(row['profile'])
for profile in sorted(profiles):
 p=W/'fixtures'/profile/'inputs/1000.json';entries.append((p,'diagnostic/fixtures/'+profile+'/inputs/1000.json'))
for p in W.glob('build-*.json'):entries.append((p,'diagnostic/'+p.name))
archive('measurements-and-validation.tar.gz',entries)
entries=tree(W/'baseline-source','source/baseline-source')
for name in ('import-probe','sync','digest','budget','hash'):
 source=W/f'{name}-source'; files=read(W/f'build-{name}.json')['source_files']
 assert {str(p.relative_to(source)) for p in source.rglob('*') if p.is_file()}==set(files)
 for name,d in files.items():
  p=source/name;assert digest(p)==d;entries.append((p,'diagnostic/'+source.name+'/'+name))
for name,d in read(W/'source-snapshot.json')['source_files'].items():
 if name.startswith('bindings/') and d!='missing':
  p=R/name;assert digest(p)==d;entries.append((p,'source/'+name))
for p in W.iterdir():
 if p.is_file() and p.suffix in {'.py','.log'}:entries.append((p,'scripts-and-logs/'+p.name))
archive('source-and-probes.tar.gz',entries)
with tarfile.open(D/'source-and-inputs.tar.gz') as tar:
 expected={}
 for entry in tar.getmembers():
  assert entry.isfile() and entry.name not in expected
  with tar.extractfile(entry) as s:expected[entry.name]=hashlib.file_digest(s,'sha256').hexdigest()
 manifest['archives']['source-and-inputs.tar.gz']=expected
(D/'archive-manifest.json').write_text(json.dumps(manifest,indent=2,sort_keys=True)+'\n')
print(json.dumps({'status':'written','destination':str(D),'members':{k:len(v) for k,v in manifest['archives'].items()}},indent=2))
