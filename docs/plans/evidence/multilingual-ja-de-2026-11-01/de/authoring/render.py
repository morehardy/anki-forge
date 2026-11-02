from pathlib import Path
import json,re,sys
ROOT=Path(__file__).resolve().parents[6]
BASE=Path(__file__).resolve().parent.parent
SOURCES=json.loads((BASE/'authoring/sources.json').read_text())
FENCE=re.compile(r'^```[^\n]*\n[\s\S]*?^```',re.M)
def source(name):
 d=next(d for d in SOURCES if d['name']==name)
 return (ROOT/d['source']).read_text()
def show(names):
 for n in names:
  s=source(n); i=iter(range(1000)); print('\n=== '+n+' ===\n'+FENCE.sub(lambda m:'[[CODE'+str(next(i))+']]',s))
def write(name,body):
 blocks=FENCE.findall(source(name))
 for i,b in enumerate(blocks):
  assert body.count('[[CODE'+str(i)+']]')==1,(name,i)
  body=body.replace('[[CODE'+str(i)+']]',b)
 assert '[[CODE' not in body
 p=BASE/'staging/docs/de'/('index.md' if name=='docs' else name+'.md'); p.parent.mkdir(parents=True,exist_ok=True); p.write_text(body.strip()+'\n')
if __name__=='__main__':
 if sys.argv[1]=='show': show(sys.argv[2:])
 elif sys.argv[1]=='render':
  for p in (BASE/'authoring/drafts').glob('*.md'):write(p.stem,p.read_text())
