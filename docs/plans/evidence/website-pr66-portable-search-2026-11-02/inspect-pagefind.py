import pathlib,json,gzip,hashlib,sys

def decode(b):
 def read(i):
  h=b[i];i+=1;m=h>>5;a=h&31
  if a<24:n=a
  elif a in [24,25,26,27]:
   size=1<<(a-24);n=int.from_bytes(b[i:i+size],'big');i+=size
  else:raise ValueError((i,h))
  if m==0:return n,i
  if m==1:return -1-n,i
  if m in [2,3]:return (b[i:i+n] if m==2 else b[i:i+n].decode()),i+n
  if m==4:
   out=[]
   for _ in range(n):v,i=read(i);out.append(v)
   return out,i
  if m==5:
   out={}
   for _ in range(n):k,i=read(i);v,i=read(i);out[k]=v
   return out,i
  if m==7:return {20:False,21:True,22:None}.get(n,n),i
  raise ValueError((i,m,n))
 return read(0)[0]
def payload(p):
 b=gzip.decompress(p.read_bytes());assert b.startswith(b'pagefind_dcd');return b[12:]
def inspect(root):
 root=pathlib.Path(root);entry=json.loads((root/'pagefind-entry.json').read_text());result={}
 for lang,desc in entry['languages'].items():
  meta=decode(payload(root/f"pagefind.{desc['hash']}.pf_meta"));pages=[]
  for p in meta[1]:
   fragment=json.loads(payload(root/'fragment'/f'{p[0]}.pf_fragment'));pages.append({'hash':p[0],'url':fragment['url'],'words':p[1]})
  result[lang]={'meta':meta,'pages':pages,'chunks':meta[2]}
 return result
if __name__=='__main__':
 a,b=map(inspect,sys.argv[1:3])
 for lang in sorted(a):
  left,right=a[lang],b[lang]
  print(lang,'page order same',left['pages']==right['pages'],'chunks same',left['chunks']==right['chunks'])
  if left['pages']!=right['pages']:
   for i,(x,y) in enumerate(zip(left['pages'],right['pages'])):
    if x!=y:print(i,x,y)
  if left['chunks']!=right['chunks']:print(left['chunks'],right['chunks'])
