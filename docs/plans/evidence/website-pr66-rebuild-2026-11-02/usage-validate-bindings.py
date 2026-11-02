import json,pathlib,hashlib,urllib.parse
b=pathlib.Path('docs/plans/evidence/website-pr66-rebuild-2026-11-02');out={}
for mode in ['prefix','root']:
 p=b/f'usage-tasks-{mode}.json';r=json.loads(p.read_text());m=json.loads((b/f'{mode}-site-manifest.json').read_text());mr={x['route']:x['sha256'] for x in m['pages']+m['resources']};n=0
 def visit(x):
  global n
  if isinstance(x,dict):
   if set(['path','sha256']).issubset(x):
    assert hashlib.sha256(pathlib.Path(x['path']).read_bytes()).hexdigest()==x['sha256'],x['path'];n+=1
   for v in x.values(): visit(v)
  elif isinstance(x,list):
   for v in x:visit(v)
 visit(r)
 for x in r['resourceBindings']: assert mr[x['route']]==x['sha256']
 bound={x['route'] for x in r['resourceBindings']}
 for l,en in r['entries'].items():
  assert len(en['taskResults'])==6
  for task in en['taskResults']:
   for u in task['readUrls']:assert urllib.parse.urlparse(u).path in bound
   assert task['status']=='passed'
   assert all(x['severity'] not in ['major','critical'] for x in task['openIssues'])
 out[mode]={'reportSha256':hashlib.sha256(p.read_bytes()).hexdigest(),'allEvidenceAndSourceBindingsMatch':True,'fileBindingsVerifiedIncludingRepeatedTaskAttachments':n,'resourceBindings':len(bound),'allTaskReadUrlsBound':True,'tasks':36,'nonblockingSourceIssuesPreserved':len(r['openIssues'])}
(b/'usage-final-binding-validation.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out,indent=2))
