import pathlib,json,hashlib,zipfile,sqlite3,compression.zstd,sys
p=pathlib.Path('docs/plans/evidence/remaining-locales-baseline-2026-11-01');env=sys.argv[1];old=pathlib.Path(f'target/remaining-locales-baseline/{env}-artifact');new=pathlib.Path(f'target/remaining-locales-baseline-maintenance/{env}-artifact')
a=json.loads((old/'generated/site-manifest.json').read_text());b=json.loads((new/'generated/site-manifest.json').read_text());prev={r['route']:r for r in a['pages']+a['resources']};changed=[r for r in b['pages']+b['resources'] if r['sha256']!=prev[r['route']]['sha256']]
base=b['configuration']['base'].rstrip('/');sha=lambda x:hashlib.sha256(x).hexdigest();checks=[]
def identity_content(x):
 if isinstance(x,dict):return {k:identity_content(v) for k,v in x.items() if k not in ['mtime_secs','collection_blake3','identity_blake3']}
 if isinstance(x,list):return [identity_content(v) for v in x]
 return x
def inspect(f):
 with zipfile.ZipFile(f) as z:
  db=sqlite3.connect(':memory:');db.deserialize(compression.zstd.decompress(z.read('collection.anki21b')));assert db.execute('pragma integrity_check').fetchone()[0]=='ok'
  notes=db.execute('select guid,mid,flds from notes order by guid').fetchall();cards=db.execute('select notes.guid,cards.ord from cards join notes on cards.nid=notes.id order by notes.guid,cards.ord').fetchall();db.close()
  media={n:sha(compression.zstd.decompress(z.read(n))) for n in z.namelist() if n.isdigit()}
  identity=json.loads(z.read('ankiforge-identity.json'));return {'notes':notes,'cards':cards,'decodedMediaHashes':media,'decodedMediaMapSha256':sha(compression.zstd.decompress(z.read('media'))),'identityExceptRevisionTimesAndPayloadDigests':identity_content(identity)}
def showcase_content(x):
 if isinstance(x,dict):return {k:showcase_content(v) for k,v in x.items() if k not in ['bytes','sha256','sourceCommit']}
 if isinstance(x,list):return [showcase_content(v) for v in x]
 return x
for record in changed:
 rel=record['route'][len(base):].lstrip('/');before=old/rel;after=new/rel
 proof={'route':record['route'],'previousSha256':prev[record['route']]['sha256'],'currentSha256':record['sha256']}
 if rel.endswith('.apkg'):
  first=inspect(before);second=inspect(after);assert first==second,rel;proof.update(kind='regenerated-showcase-apkg',observations=second,conclusion='SQLite integrity passes; GUID/model/content/card ordinals, decoded media and identity keys exactly equal. Revision-time and collection/identity payload digests intentionally excluded from semantic comparison; binary hashes are not claimed equal.')
 elif rel=='generated/showcase.json':
  first=json.loads(before.read_text());second=json.loads(after.read_text());assert showcase_content(first)==showcase_content(second);proof.update(kind='showcase-provenance',previousCommit=first['sourceCommit'],currentCommit=second['sourceCommit'],conclusion='Only generated APKG byte sizes/hashes and actual sourceCommit differ; displayed content, counts, comparison and source digest exactly equal.')
 elif rel=='pagefind/pagefind-entry.json':
  assert json.loads(before.read_text())==json.loads(after.read_text());proof.update(kind='json-key-order-only',conclusion='Parsed JSON exactly equal, including language index hashes; object key serialization order differs.')
 else:raise AssertionError('Unexplained delivered resource change: '+rel)
 checks.append(proof)
(p/f'generated-resource-inspection-{env}.json').write_text(json.dumps({'schemaVersion':1,'scope':'Independent semantic inspection of actual changed generated resources after maintenance backport; no broad byte equality claim.','changedResources':checks},ensure_ascii=False,indent=2)+'\n');print(env,'explained changed generated resources:',len(checks))
