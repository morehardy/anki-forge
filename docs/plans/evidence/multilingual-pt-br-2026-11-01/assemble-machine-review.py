"""Attach this reviewer's written answers to verified final HTTP observations."""
import hashlib,json,pathlib,sys
root=pathlib.Path.cwd();p=pathlib.Path('docs/plans/evidence/multilingual-pt-br-2026-11-01');env=sys.argv[1];label=sys.argv[2]
load=lambda f:json.loads((root/f).read_text())
sha=lambda b:hashlib.sha256(b).hexdigest()
proof=lambda f,d:{'path':str(f),'sha256':sha((root/f).read_bytes()),'description':d}
m=load(p/f'machine-{label}/http-observations.json');manifest_path=p/f'machine-{label}/origin-manifest.json';manifest=load(manifest_path);c=load(p/'machine-conclusions.json');audit=load(p/'execution-reuse-audit.json')
assert m['manifestSha256']==sha((root/manifest_path).read_bytes());assert manifest['inputSha256']==m['inputSha256'];assert len(m['entries'])==4
for b in audit['currentBindings']:assert sha((root/b['path']).read_bytes())==b['sha256']
previous=load(p/'machine-prefix/http-observations.json')
oldbase=previous['configuration']['site']+(''if previous['configuration']['base']=='/'else previous['configuration']['base'])
newbase=m['configuration']['site']+(''if m['configuration']['base']=='/'else m['configuration']['base'])
comparisons=[]
for locale,tasks in m['entries'].items():
 for t in tasks:
  old=next(x for x in previous['entries'][locale]if x['id']==t['id'])
  for before,after in zip([old['index']]+old['followed'],[t['index']]+t['followed'],strict=True):
   assert before.get('topic')==after.get('topic')
   assert before['body'].replace(oldbase,newbase)==after['body'],f"Semantic delta requires rereview: {locale}/{t['id']}/{after['route']}"
   comparisons.append({'locale':locale,'task':t['id'],'previousUrl':before['url'],'previousSha256':before['sha256'],'currentUrl':after['url'],'currentSha256':after['sha256'],'equalAfterExactDeploymentUrlReplacement':True})
comparison_path=p/f'machine-{label}/content-comparison.json'
(root/comparison_path).write_text(json.dumps({'scope':'Fresh final HTTP responses compared to actual responses read by this reviewer. Only the exact configured deployment URL is normalized; no words, code, labels or restrictions are removed. Each final task still starts from a fresh same-locale llms request.','from':oldbase,'to':newbase,'comparisons':comparisons},ensure_ascii=False,indent=2)+'\n')
records={r['route']:r for r in manifest['pages']+manifest['resources']};bindings=[{'route':r['route'],'sha256':r['sha256']}for r in manifest['resources']if r['route'].endswith(('/index.md','/llms.txt'))]
report={'schemaVersion':1,'kind':'machine-read','status':'passed','mode':'release-candidate','reviewerType':'ai','reviewerRun':f'/root/pt_br_technical/machine-read-{env}-2026-11-01','inputSha256':manifest['inputSha256'],'configuration':manifest['configuration'],'releaseLocales':manifest['releaseLocales'],'readOrigin':m['readOrigin'],'originManifest':dict(proof(manifest_path,'Original bytes of the actual final artifact manifest.'),commit=manifest['commit']),'resourceBindings':bindings,'entries':{},'openIssues':[],'method':'Four fresh tasks per locale begin at the actual locale llms.txt over HTTP and follow real links to that locale Markdown. Response status, UTF-8 content type, no download disposition, byte count and SHA-256 match the archived final manifest. Answers were written by this independent reviewer after reading the captured task-relevant instructions and limitations, not copied from earlier passing reports. Final response bodies were compared exactly to those read, allowing only the exact deployment URL replacement. Hash binding of all machine resources establishes freshness, not a claim of manually reading every unvisited page. SDK executions are explicitly reused after a separate fresh input and APKG audit.'}
for locale,tasks in m['entries'].items():
 results=[]
 for t in tasks:
  reads=[t['index']]+t['followed'];proofs=[proof(p/f'machine-{label}/http-observations.json','Actual final HTTP requests, response URLs, headers, bodies, bytes and manifest hashes.'),proof(comparison_path,'Exact comparison to the task-relevant responses this reviewer read; permits only configured deployment URL replacement.'),proof(p/'machine-conclusions.json','Manually authored current four-language answers from actual retrieved content.'),proof(p/'execution-reuse-audit.json','Explicit execution reuse and fresh inspection of ten retained APKGs; no SDK rerun is claimed.')]
  for r in reads:
   assert sha((root/r['path']).read_bytes())==r['sha256']==records[r['route']]['sha256'];proofs.append(proof(pathlib.Path(r['path']),f"Raw UTF-8 HTTP 200 response from {r['url']} with matching artifact resource hash."))
  answer=c['answers'][locale][t['id']]+' Sources: '+', '.join(r['url']for r in t['followed'])
  results.append({'id':t['id'],'status':'passed','readUrls':t['readUrls'],'answer':answer,'evidence':proofs,'openIssues':[]})
 report['entries'][locale]={'aiMachineReadReview':results,'openIssues':[]}
(root/p/f'machine-read-{env}.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
print(f'{env}: 16 current-manifest-bound machine conclusions assembled from real HTTP requests; {len(bindings)} machine resources bound.')
