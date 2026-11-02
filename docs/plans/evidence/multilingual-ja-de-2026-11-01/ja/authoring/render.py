import pathlib,re,json
ROOT=pathlib.Path(__file__).resolve().parents[6]
HERE=pathlib.Path(__file__).parent
inventory=json.loads((HERE/'source-inventory.json').read_text())
for file in (HERE/'templates').glob('*.md'):
 name=file.stem; data=inventory[name]; body=file.read_text()
 seen=[]
 def insert(m):
  i=int(m[1]);seen.append(i);return data['blocks'][i]
 body=re.sub(r'@@CODE_(\d+)@@',insert,body)
 assert sorted(seen)==list(range(len(data['blocks']))),(name,seen,len(data['blocks']))
 markers=re.findall(r'<!-- section: ([^>]+) -->',body)
 assert markers==data['markers'],(name,markers,data['markers'])
 (ROOT/'docs/ja'/('index.md' if name=='docs' else name+'.md')).write_text(body)
 print(name,len(body.encode()))
