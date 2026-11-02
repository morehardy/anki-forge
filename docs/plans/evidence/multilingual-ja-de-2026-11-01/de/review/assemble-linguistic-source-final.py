from pathlib import Path
import json,hashlib,subprocess,copy,re
root=Path.cwd();base=Path('docs/plans/evidence/multilingual-ja-de-2026-11-01/de');review=base/'review'
sha=lambda b:hashlib.sha256(b).hexdigest()
proof=lambda p:{'path':str(p),'sha256':sha(p.read_bytes())}
load=lambda p:json.loads(p.read_text())
baseline='270f26aa5653a0f56297f46fd35e586da444622b'
priorpath=Path('docs/plans/evidence/multilingual-ja-de-2026-11-01/ja/linguistic-source-review-final.json')
priorraw=subprocess.check_output(['git','show',baseline+':'+str(priorpath)]);prior=json.loads(priorraw)
assert sha(priorraw)=='98ec8f2327db813eacad3b47394dbe555cff7e6ac1f316b4f7872f5ac5d98fcb'
candidatepath=base/'source-candidates-final.json';candidate=load(candidatepath)
checks=load(review/'production-mapping-and-five-locale-delta-final-check.json');model=load(review/'shared-model-and-prior-lineage-preview.json')
assert not checks['issues'] and not model['issues'];assert checks['previewCandidate']['sha256']==sha(candidatepath.read_bytes())
assert all(x['matchesCurrent'] for x in checks['currentCandidateBindings']) and len(checks['currentCandidateBindings'])==204
notespath=review/'linguistic-blind-read.json';notes=load(notespath);deltapath=review/'linguistic-delta-tools-persistence.json';delta=load(deltapath);vocabpath=review/'search-vocabulary-linguistic-delta.json';vocab=load(vocabpath)
assert all(x['status']=='resolved' for x in notes['findings']);assert not delta['openNewFindings'] and not vocab['openIssues']
for filename,expected in vocab['inputHashes'].items():assert sha(Path(filename).read_bytes())==expected,filename
for mapping in checks['germanProductionMappings']:assert sha(Path(mapping['production']).read_bytes())==mapping['reviewedSha256'],mapping['production']
for check in checks['oldInputFileChecks']:assert sha(Path(check['path']).read_bytes())==check['currentSha256'],check['path']
assert sha(Path('website/site-model.mjs').read_bytes())==model['siteModel']['currentSha256']
priorproof={'path':str(priorpath),'sha256':sha(priorraw),'reviewerRun':prior['reviewerRun'],'baselineCommit':baseline}
notemap={x['topic']:x for x in notes['pageNotes']+notes['articleNotes']}
shared_files=['website/src/i18n/de.json','website/src/content/i18n/de.json','website/site-model.mjs','website/language-briefs.json','docs/ai-quickstart.json','website/search-terms.json','website/search-concepts.json']
common={
 'home':('home.hero1','The complete German homepage, AI brief controls, copy-recovery messages, benchmark qualifiers and machine summaries were read. Direct Du instructions remain consistent; Anki-APKG generation is distinct from client-side learning behavior.'),
 'examples':('examples.intro','Read the complete German example descriptions and UI, including Basic/Cloze/media/occlusion, original APKG update warning and Notiz/Karte/Mediendatei one/other forms. Preview/download and update limitations are understandable.'),
 'blog':('blog.heading','Read the complete blog-list labels, article categories, navigation and feed copy. Gedanken zur Kartenerstellung is idiomatic and avoids a literal translation of the English wordplay.')
}
catalog=load(Path('website/src/i18n/de.json'))
entries=copy.deepcopy(candidate['entries'])
for topic,locales in entries.items():
 for locale,entry in locales.items():
  if locale!='de':
   previous=prior['entries'][topic][locale]['linguistic'];assert previous['status']=='reviewed'
   entry['linguistic']=copy.deepcopy(previous)
   entry['linguistic']['evidence'].append({
    'criterion':'six-locale-shared-delta-with-independent-prior-lineage',
    'priorEvidence':{**priorproof,'entry':f'{topic}/{locale}/linguistic'},
    'deltaEvidence':proof(review/'production-mapping-and-five-locale-delta-final-check.json'),
    'modelAndLineageEvidence':proof(review/'shared-model-and-prior-lineage-preview.json'),
    'judgement':'Compared every old-locale review dependency to immutable 270f26a and recomputed all 170 baseline signatures against the sealed independent final-v2 report. Existing bodies, UI/framework catalogs, rendering/search code, metadata/sidebar/navigation projections, briefs, prompts and locale vocabulary remain unchanged. Independently read the sole new shared language-menu choice Deutsch with its de tag and /de topic route, and the German-only shared data additions. Retain the prior topic-specific scores, full evidence lineage and existing minor observations; this is an incremental shared-data review, not a fresh read of unchanged prose.',
    'japaneseIndependenceBoundary':('This reviewer authored the original Japanese translation and does not newly approve it. Its unchanged prose is covered by the other reviewer’s complete Japanese read and media/internal-tools delta in the prior independent final report; the current judgment is limited to the independently authored German/menu additions.' if locale=='ja' else None)
   })
   continue
  if topic in common:
   key,judgment=common[topic];file='website/src/i18n/de.json';span=catalog[key];text=Path(file).read_text();line=next(i for i,s in enumerate(text.splitlines(),1) if '"'+key+'"' in s)
  else:
   item=notemap[topic];file=item['path'].split('/staging/',1)[1];text=Path(file).read_text();body=re.sub(r'^---\n.*?\n---\n','',text,flags=re.S)
   span=next(s for s in body.splitlines() if s.strip() and not s.startswith(('#','import ','<!--','<span','{/*')))
   line=text[:text.index(span)].count('\n')+1;judgment=item['note']
   if topic=='anki-forge-vs-genanki':judgment='Balanced comparison with limited historical performance claim. The complete paragraph with vergleichen/preparieren was independently reread after the corrected verb construction; the two operations now have clear complements.'
   elif topic=='rust-api':judgment='Method tables and error domains are coherent. The complete numeric-limit paragraph was independently reread after numeric 0 replaced the ambiguous written null.'
   elif topic=='node-api':judgment='Report versus JSON field naming and asynchronous ownership are clear. The changed numeric-limit paragraph now uses literal 0 and cannot be mistaken for the JavaScript null value.'
   elif topic=='troubleshooting':judgment='Symptom/action tables are direct and actionable. The complete final persistence paragraph was independently reread; Fehlern beim dauerhaften Speichern removes the memory-fault ambiguity.'
  evidence=[{
   'criterion':'full-german-topic-language-read',
   'file':file,'line':line,'span':span,'targetFileSha256':sha(Path(file).read_bytes()),
   'judgement':judgment,
   'readEvidence':proof(notespath),
   'reviewMethod':'Full independent German prose/table or UI reading in isolated staging, followed by exact production-byte mapping and independent rereading of all subsequent changes. Protected code was not rewritten or treated as linguistic approval.'
  },{
   'criterion':'complete-german-shared-language-surface',
   'files':shared_files,
   'judgement':'Read all 210 business messages, 11 framework messages, 29 document title/description pairs, five sidebar groups, three navigation labels, full terminology/ownership brief, parameterized prompt and production vocabulary. Notiz/Karte/Stapel remain distinct. Du register, singular/plural forms, copy/search recovery, original distribution APKG and source-only boundaries are consistent. The final search-only additions were separately read against their German body sections; no search-ranking conclusion is claimed.',
   'originalFullRead':proof(notespath),
   'finalVocabularyDelta':proof(vocabpath)
  }]
  if topic in ['anki-forge-vs-genanki','rust-api','node-api','custom-notetypes']:
   evidence.append({'criterion':'independent-reread-of-integrator-corrections','evidence':proof(notespath),'judgement':'The closed-findings records and subsequentClarification contain the exact reread hashes and judgments for the corrected grammar, numeric zero and precise whitespace-key wording. Production bytes match those independently reread inputs.'})
  if topic in ['media','troubleshooting']:
   evidence.append({'criterion':'independent-internal-tools-and-persistence-delta','evidence':proof(deltapath),'judgement':next(x['assessment'] for x in delta['entries'] if x['topic']==topic)})
  entry['linguistic']={'status':'reviewed','score':copy.deepcopy(notes['score']),'evidence':evidence,'openIssues':[]}
report={
 'schemaVersion':1,'rubricVersion':candidate['rubricVersion'],'glossaryVersion':candidate['glossaryVersion'],'reviewerType':'ai','reviewerRun':'/root/ja_full_author/independent-de-full-and-five-locale-shared-delta-final-2026-11-01',
 'reviewMethod':{
  'axis':'linguistic source','scope':'34 topics × 6 release locales (204 records): 34 German topics from independent full-read evidence plus 170 old-locale judgments retained through verified immutable lineage and an actual shared-data delta review.',
  'independence':'Reviewer did not author or edit German content. Reviewer authored original Japanese translations; this report does not self-approve them. Existing Japanese prose is unchanged from 270f26a and retains the separate /root/de_full_author independent full-read and media-delta evidence. The present Japanese-locale judgment is only the new German-menu/shared-data delta.',
  'snapshot':proof(candidatepath),'baselineCommit':baseline,'priorIndependentReport':priorproof,
  'method':[
   'Read all 29 German documentation bodies and two articles as German, plus all 210 UI and 11 framework messages and the complete author data patch, before consulting English for the proposed clarity fixes.',
   'Independently reread the integrator changes to the genanki workflow sentence, Rust/Node numeric-zero wording, exact whitespace-key rule, internal-tools qualification and persistence-error wording.',
   'Compare every one of the 33 production German files with the exact reviewed staging and delta hashes. All match; map the same 31 longforms to stable section contracts.',
   'Read the final German-only vocabulary delta against concepts, compatibility, updates and the authoring guide; confirm ordinary German inflections/compound words and limit-oriented client-history terms without reading the frozen query set.',
   'Reconstruct all 226 old-locale review dependencies from immutable 270f26a and recompute 170 signatures against the Japanese final-v2 snapshot and independent final report. Verify seven cited report/source hashes and all 38 retained minor spans.',
   'Compare whole site-model syntax after removing only de properties; existing projections and implementation are identical. Read the unchanged LanguageSwitcher component and its new selected Deutsch entry. Old prompts, briefs, metadata, sidebar labels and vocabulary remain unchanged.',
   'Recompute every current final-candidate signature and bind the language judgments to those 204 sourceHash/targetHash/technicalDependencyHash values.'
  ],
  'fullReadEvidence':proof(notespath),'stagingDeltaEvidence':proof(deltapath),'vocabularyDeltaEvidence':proof(vocabpath),
  'productionMappingEvidence':proof(review/'production-mapping-and-five-locale-delta-final-check.json'),'modelAndPriorLineageEvidence':proof(review/'shared-model-and-prior-lineage-preview.json'),
  'independentSearchQueriesRead':False,'exclusions':'No technical-axis approval, SDK execution, browser/layout acceptance, independent Pagefind ranking evaluation or deployment approval. Final post-build usage tasks remain separate.'
 },
 'closedFindings':copy.deepcopy(notes['findings']),
 'additionalRereads':{'customNotetypeKeyClarification':notes['subsequentClarification'],'toolsAndPersistenceDelta':proof(deltapath),'optionalPersistenceObservation':delta['closedOptionalObservation']},
 'entries':entries,
 'summary':{'topics':34,'localeEntries':204,'fullGermanLongforms':31,'germanBusinessKeys':210,'germanFrameworkKeys':11,'existingLocaleEntriesWithVerifiedPriorEvidence':170,'retainedMinorIssues':{locale:sum(len(v[locale]['linguistic']['openIssues']) for v in entries.values()) for locale in ['en','zh-CN','es','pt-BR','ja','de']},'germanOpenMinorIssues':0,'majorIssues':0,'criticalIssues':0}
}
for topic,locales in candidate['entries'].items():
 for locale,entry in locales.items():
  target=report['entries'][topic][locale]
  for k in ['sourceHash','targetHash','technicalDependencyHash','inputFiles']:assert target[k]==entry[k],(topic,locale,k)
  assert target['linguistic']['status']=='reviewed' and target['linguistic']['evidence']
  assert all(4<=x<=5 for x in target['linguistic']['score'].values())
  assert not any(x['severity'] in ['major','critical'] for x in target['linguistic']['openIssues'])
out=base/'linguistic-source-review-final.json';assert not out.exists(),'Refusing to overwrite sealed final evidence'
out.write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'report':proof(out),'summary':report['summary']},ensure_ascii=False,indent=2))
