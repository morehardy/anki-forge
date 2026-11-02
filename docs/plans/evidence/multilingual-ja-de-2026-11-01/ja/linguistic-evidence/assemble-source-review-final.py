"""One-sentence linguistic delta over the sealed full independent source review."""
from pathlib import Path
import hashlib, json, copy
root=Path.cwd(); base=root/'docs/plans/evidence/multilingual-ja-de-2026-11-01/ja'
sha=lambda b:hashlib.sha256(b).hexdigest()
proof=lambda p:{'path':str(p.relative_to(root)), 'sha256':sha(p.read_bytes())}
priorpath=base/'linguistic-source-review.json'; prior=json.loads(priorpath.read_text())
assert sha(priorpath.read_bytes())=='442a9ba0698b625bc7b33868aa1fd97bb84f0dab5bca3a012fefef026214087e'
oldpath=base/'source-candidates-final.json'; newpath=base/'source-candidates-final-v2.json'
old=json.loads(oldpath.read_text()); new=json.loads(newpath.read_text())
changed=[]
for topic, locales in old['entries'].items():
 for locale, entry in locales.items():
  if entry != new['entries'][topic][locale]:
   changed.append((topic,locale))
   assert [k for k in entry if entry[k]!=new['entries'][topic][locale][k]]==['targetHash']
assert changed==[('media','ja')]
filename='docs/ja/media.md'; text=(root/filename).read_text()
sentence=' `tools::load_project` は `internal-tools` feature が必要なリポジトリ内部向けの API であり、利用側アプリケーション向けのサポート対象 API には含まれません。'
assert text.count(sentence)==1
readhashes=json.loads((base/'linguistic-evidence/read-input-hashes.json').read_text())['files']
assert sha(text.replace(sentence,'').encode())==readhashes[filename]
unchanged=[]
for name,expected in readhashes.items():
 if name != filename:
  assert sha((root/name).read_bytes())==expected,name
  unchanged.append(name)
report=copy.deepcopy(prior)
report['reviewerRun']=prior['reviewerRun']+'/media-internal-tools-delta'
report['reviewMethod']['snapshot']=proof(newpath)
report['reviewMethod']['incrementalReview']={
 'priorFullReview':proof(priorpath),
 'priorCandidate':proof(oldpath),
 'newCandidate':proof(newpath),
 'method':'Independently reread the single appended Japanese sentence in its full media paragraph and compared the exact feature/support boundary with anki_forge/src/lib.rs lines 36–40. The sealed complete 170-entry review remains the source of all earlier judgments. Removing exactly this sentence reconstructs the full-read Japanese media file hash. All remaining recorded read-input files are byte-identical, and all other 169 candidate entries are unchanged; no fresh full reread or German approval is claimed.',
 'changedEntries':['media/ja'], 'exactReusedEntries':169,
 'unchangedReadFiles':unchanged,
 'deltaEvidence':str((base/'linguistic-evidence/media-language-delta.json').relative_to(root))
}
entry=report['entries']['media']['ja']; entry['targetHash']=new['entries']['media']['ja']['targetHash']
entry['linguistic']['evidence'].append({
 'criterion':'incremental-feature-boundary-language-read',
 'file':filename, 'line':text[:text.index(sentence)].count('\n')+1,
 'span':sentence.strip(), 'sourceSha256':sha(text.encode()),
 'judgement':'The appended sentence clearly attaches the internal-tools prerequisite and unsupported-consumer-API limitation to tools::load_project alone. リポジトリ内部向け and 利用側アプリケーション向け make the two audiences explicit; the wording does not incorrectly restrict NoteType::from_bundle. Rust feature terminology and protected identifiers are preserved. Reread with the preceding validation recommendation and following MIME paragraph; no new linguistic issue.',
 'technicalBoundarySource':{**proof(root/'anki_forge/src/lib.rs'),'lines':'36–40'},
 'priorFullReview':{**proof(priorpath),'entry':'media/ja/linguistic'}
})
for topic,locales in new['entries'].items():
 for locale,c in locales.items():
  for k in ['sourceHash','targetHash','technicalDependencyHash','inputFiles']:assert report['entries'][topic][locale][k]==c[k]
  if (topic,locale)!=('media','ja'):assert report['entries'][topic][locale]==prior['entries'][topic][locale]
delta={'kind':'single-sentence-independent-linguistic-delta','priorFullReview':proof(priorpath),'currentCandidate':proof(newpath),'file':filename,'beforeFileSha256':readhashes[filename],'afterFileSha256':sha(text.encode()),'onlyAddedText':sentence,'reconstructionVerified':True,'otherCandidateEntriesIdentical':169,'otherRecordedReadFilesIdentical':len(unchanged),'linguisticConclusion':'No new issue; original scores and three Japanese minor findings retained.','technicalBoundarySource':proof(root/'anki_forge/src/lib.rs')}
for path,obj in [(base/'linguistic-evidence/media-language-delta.json',delta),(base/'linguistic-source-review-final.json',report)]:
 assert not path.exists(),f'Refuse overwriting sealed evidence {path}'
 path.write_text(json.dumps(obj,ensure_ascii=False,indent=2)+'\n')
 print(json.dumps(proof(path)))
