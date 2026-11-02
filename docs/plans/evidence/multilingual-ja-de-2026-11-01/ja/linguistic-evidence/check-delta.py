from pathlib import Path
import json,subprocess,hashlib,re
ROOT=Path.cwd();base=ROOT/'docs/plans/evidence/multilingual-ja-de-2026-11-01/ja';oldloc=['en','zh-CN','es','pt-BR']
C=json.loads((base/'source-candidates-final.json').read_text());P=json.loads((ROOT/'docs/plans/evidence/multilingual-pt-br-2026-11-01/linguistic-source-review-final.json').read_text())
def sha(b):return hashlib.sha256(b).hexdigest()
def old(p):return subprocess.check_output(['git','show','231bcd5:'+p])
files=sorted({p for values in C['entries'].values() for l,e in values.items() if l in oldloc for p in e['inputFiles']})
checks=[]
for p in files:
 before=old(p);after=(ROOT/p).read_bytes();checks.append({'path':p,'baselineSha256':sha(before),'currentSha256':sha(after),'unchanged':before==after})
changed=[r['path'] for r in checks if not r['unchanged']]
projections=[]
for p in ['docs/ai-quickstart.json','website/language-briefs.json','website/search-terms.json','website/search-concepts.json']:
 before=json.loads(old(p));after=json.loads((ROOT/p).read_text())
 for l in oldloc:
  if p.endswith('ai-quickstart.json'):a={k:before[k][l] for k in ['prompts','requestPlaceholders','promptContracts']};b={k:after[k][l] for k in a}
  elif 'search-' in p:a=before['locales'][l];b=after['locales'][l]
  else:a=before[l];b=after[l]
  projections.append({'path':p,'locale':l,'unchanged':a==b})
def docs(s):return json.loads(re.search(r'const documents = (\[.*?\]);',s,re.S).group(1))
before=old('website/site-model.mjs').decode();after=(ROOT/'website/site-model.mjs').read_text();oldmeta=docs(before);newmeta=docs(after)
for l in oldloc:
 a={d['name']:{k:d[k][l] for k in ['title','description']} for d in oldmeta};b={d['name']:{k:d[k][l] for k in ['title','description']} for d in newmeta};projections.append({'path':'website/site-model.mjs','locale':l,'part':'29 title/description pairs','unchanged':a==b})
 labels=re.compile(r'(en|\x27zh-CN\x27|es|\x27pt-BR\x27):\s*([\x22\x27])(.*?)\2')
 for label,start,end in [('docGroups','export const docGroups','export const navigation'),('navigation','export const navigation','export const footerNavigation')]:
  a=labels.findall(before[before.index(start):before.index(end)]);b=labels.findall(after[after.index(start):after.index(end)]);projections.append({'path':'website/site-model.mjs','part':label,'unchanged':a==b})
assert all(p['unchanged'] for p in projections)
oldissues=[]
for topic,locales in P['entries'].items():
 for l,e in locales.items():
  for issue in e['linguistic']['openIssues']:
   p=issue['file'];content=(ROOT/p).read_text();span=issue['span'];assert span in content,(topic,l,span)
   oldissues.append({'topic':topic,'locale':l,'file':p,'span':span,'stillPresent':True})
result={'kind':'independent-linguistic-delta-evidence','baselineCommit':'231bcd5','oldLocales':oldloc,'inputFileChecks':checks,'changedInputFiles':changed,'unchangedLocalizedProjections':projections,'retainedPriorMinorIssues':oldissues,'priorReportSha256':sha((ROOT/'docs/plans/evidence/multilingual-pt-br-2026-11-01/linguistic-source-review-final.json').read_bytes()),'candidateSha256':sha((base/'source-candidates-final.json').read_bytes()),'notes':['Compared every current old-locale source dependency against immutable 231bcd5 bytes.','Independently read the actual changed locale menu, prompt, metadata, vocabulary, search-normalization and social-font changes.','Prior minor issue spans verified still present, not silently closed.','No old-locale full reread or artifact/browser pass is claimed.']}
(base/'linguistic-evidence/baseline-delta-checks.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n');print(json.dumps({'inputFiles':len(checks),'changed':changed,'localizedProjections':len(projections),'priorMinorIssues':len(oldissues)},ensure_ascii=False,indent=2))
