import pathlib,json,subprocess,hashlib
B=pathlib.Path('docs/plans/evidence/website-pr66-portable-search-2026-11-02')
def load(p):return json.loads(pathlib.Path(p).read_text())
def bind(p,description=None):
 r={'path':str(p),'sha256':hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()}
 if description:r['description']=description
 return r
record=load(B/'executions/basic/execution-inputs.json')
for x in record['inputs']:assert bind(x['path'])['sha256']==x['sha256']
programs={'rust':('anki_forge/examples/target_api_basic.rs','rust/src/main.rs'),'python':('bindings/python/examples/quickstart.py','python/main.py'),'node':('bindings/node/examples/quickstart.mjs','node/main.mjs')}
for s,(p,kept) in programs.items():assert pathlib.Path(p).read_bytes()==(B/'executions/basic'/kept).read_bytes()
source=['anki_forge/examples/target_api_basic.rs','bindings/python/examples/quickstart.py','bindings/node/examples/quickstart.mjs','scripts/packaged_consumer/src/inspection.rs','scripts/packaged_consumer/src/bin/verify_apkg.rs','scripts/packaged_consumer/src/bin/verify_json_recipe.rs','anki_forge/examples/json_to_anki.rs','bindings/python/examples/json_to_anki.py','bindings/node/examples/json_to_anki.mts','docs/recipes/json-to-anki/v1.json','docs/recipes/json-to-anki/v2.json','docs/recipes/json-to-anki/cell.png','docs/recipes/json-to-anki/tone.wav','docs/troubleshooting.md','docs/zh-cn/troubleshooting.md','website/scripts/verify-public-recipe.mjs']
deps=['docs/ai-quickstart.json','bindings/python/pyproject.toml','bindings/node/package.json','Cargo.toml','Cargo.lock','website/package.json','website/package-lock.json']
receipts=[]
for sdk in programs:
 x=load(B/f'executions/public-json/{sdk}/installation.json');s=x['source'];assert s.get('version',s.get('metadata',{}).get('version'))=='0.3.1'
 receipts.append(bind(B/f'executions/public-json/{sdk}/installation.json','Actual current-batch public package 0.3.1 receipt.'))
 if sdk=='rust':assert 'registry+https://github.com/rust-lang/crates.io-index' in s['resolution']
 if sdk=='python':assert s['download_info']['url'].startswith('https://files.pythonhosted.org/')
 if sdk=='node':assert s['resolved'].startswith('https://registry.npmjs.org/') and s['native'][0][1]['version']=='0.3.1'
 for n in ['comparison','review']:
  q=load(B/f'executions/public-json/{sdk}/{n}.json');assert q['policy']['allows_publication'] and [f['code'] for f in q['findings']]==['RISK.NOTE_CHANGED']
assert 'NOTE.KEY_DUPLICATE' in (B/'executions/basic/python/troubleshooting.log').read_text()
# Independently decode the new execution outputs with the exact existing inspector source.
w=pathlib.Path(record['workDirectory'])/'rust'
assert (w/'src/inspection.rs').read_bytes()==pathlib.Path('scripts/packaged_consumer/src/inspection.rs').read_bytes()
assert (w/'src/bin/verify_apkg.rs').read_bytes()==pathlib.Path('scripts/packaged_consumer/src/bin/verify_apkg.rs').read_bytes()
logs=[]
for group in ['basic','public-json']:
 for sdk in programs:
  args=[str(w/'target/debug/verify_apkg'),str((B/f'executions/{group}/{sdk}').resolve())];r=subprocess.run(args,capture_output=True,text=True);assert r.returncode==0,r.stderr;logs.append('$ '+' '.join(args)+'\nexit 0\n'+r.stdout+r.stderr)
(B/'usage-apkg-inspection.log').write_text('\n'.join(logs)+'\n')
(w/'src/bin/verify_json_recipe.rs').write_bytes(pathlib.Path('scripts/packaged_consumer/src/bin/verify_json_recipe.rs').read_bytes())
r=subprocess.run(['cargo','build','--offline','--locked','--quiet','--manifest-path',str(w/'Cargo.toml'),'--bin','verify_json_recipe'],capture_output=True,text=True);assert r.returncode==0,r.stderr
logs=[]
for sdk in programs:
 args=[str(w/'target/debug/verify_json_recipe'),str((B/f'executions/public-json/{sdk}').resolve()),str(pathlib.Path('docs/recipes/json-to-anki').resolve())];r=subprocess.run(args,capture_output=True,text=True);assert r.returncode==0,r.stderr;logs.append('$ '+' '.join(args)+'\nexit 0\n'+r.stdout+r.stderr)
(B/'usage-json-inspection.log').write_text('\n'.join(logs)+'\n')
locks=[B/'executions/basic/rust/Cargo.lock',B/'executions/basic/python/pip-report.json',B/'executions/basic/node/package-lock.json',B/'executions/public-json/rust/Cargo.lock']
assert load(locks[1])['install'][0]['metadata']['version']=='0.3.1';assert load(locks[2])['packages']['node_modules/ankiforge']['version']=='0.3.1'
out={'schemaVersion':1,'reviewerRun':'/root/spec_review/pr66-portable-search-usage-2026-11-02','scope':'Fresh coordinating-agent public SDK executions in this portable-search batch, independently checked against actual copied programs and input record. No earlier SDK execution is reused because website/package*.json changed. This reviewer independently reread logs/official receipts and actually decoded all 10 new APKG files and all three JSON v1/v2 pairs.','actualBasicInputRecord':bind(B/'executions/basic/execution-inputs.json'),'sourceBindings':[bind(p) for p in source],'dependencyBindings':[bind(p) for p in deps],'receipts':receipts,'locksAndInstallReports':[bind(p) for p in locks],'independentInspection':[bind(B/'usage-apkg-inspection.log','Fresh independent ZIP/protobuf/SQLite inspection: basic 1/1/0, duplicate-key recovery 2/2/0, all six JSON packages 3/3/2.'),bind(B/'usage-json-inspection.log','Current inspector compiled offline with retained new execution lock; independently verifies exact two media payloads, unchanged GUID/model/identity, one answer change and allowed compare/review policy in all three SDKs.')],'limits':'No new native Anki import or deployment claim.'}
(B/'usage-execution-audit.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n');print('New SDK input/receipt audit and independent APKG decoding passed.')
