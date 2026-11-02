from pathlib import Path
import subprocess,os,json,hashlib,shutil,datetime
root=Path('/Users/hp/Desktop/2026/anki-forge')
work=root/'target/bing-seo-basic-execution'
evidence=root/'docs/plans/evidence/bing-seo-2026-11-02/executions/basic'
old=root/'docs/plans/evidence/website-pr66-portable-search-2026-11-02/executions/basic'
env=os.environ.copy();env['PATH']='/Users/hp/.nvm/versions/node/v24.19.0/bin:/Users/hp/.cargo/bin:'+env['PATH'];env['CARGO_TARGET_DIR']=str(root/'target/bing-seo-public-consumers')
for lang in ['rust','python','node']:(work/lang).mkdir(parents=True,exist_ok=True);(evidence/lang).mkdir(parents=True,exist_ok=True)
def copy(src,dst):dst.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(src,dst)
def run(args,lang,log):
    out=subprocess.run(args,cwd=work/lang,env=env,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    (evidence/lang/log).write_text('$ '+' '.join(args)+'\n'+out.stdout+'\nexit='+str(out.returncode)+'\n')
    if out.returncode:raise RuntimeError(f'{lang}: {args}: {out.stdout[-2000:]}')
copy(old/'rust/Cargo.toml',work/'rust/Cargo.toml');copy(old/'rust/Cargo.lock',work/'rust/Cargo.lock')
copy(root/'anki_forge/examples/target_api_basic.rs',work/'rust/src/main.rs')
copy(root/'scripts/packaged_consumer/src/inspection.rs',work/'rust/src/inspection.rs')
copy(root/'scripts/packaged_consumer/src/bin/verify_apkg.rs',work/'rust/src/bin/verify_apkg.rs')
run(['cargo','run','--offline','--locked','--quiet','--bin','locale_data_basic_task'],'rust','basic.log')
copy(root/'bindings/python/examples/quickstart.py',work/'python/main.py');copy(old/'python/duplicate-key.py',work/'python/duplicate-key.py')
run(['python3','-m','venv','.venv'],'python','venv.log')
py=str(work/'python/.venv/bin/python')
run([py,'-m','pip','install','--index-url','https://pypi.org/simple','--only-binary=:all:','--report','pip-report.json','ankiforge==0.3.1'],'python','install.log')
run([py,'main.py'],'python','basic.log');run([py,'duplicate-key.py'],'python','troubleshooting.log')
copy(old/'node/package.json',work/'node/package.json');copy(old/'node/package-lock.json',work/'node/package-lock.json');copy(root/'bindings/node/examples/quickstart.mjs',work/'node/main.mjs')
run(['npm','ci','--registry=https://registry.npmjs.org','--include=optional','--ignore-scripts'],'node','install.log')
run(['node','main.mjs'],'node','basic.log')
for lang in ['rust','python','node']:
    run(['cargo','run','--offline','--locked','--quiet','--manifest-path',str(work/'rust/Cargo.toml'),'--bin','verify_apkg','--',str(work/lang)],'rust',f'inspection-{lang}.log')
    for file in (work/lang).iterdir():
        if file.is_file() and (file.suffix in ['.apkg','.json','.mjs','.py','.toml','.lock']):copy(file,evidence/lang/file.name)
copy(work/'rust/src/main.rs',evidence/'rust/src/main.rs')
oldReport=json.loads((root/'docs/plans/evidence/website-pr66-portable-search-2026-11-02/usage-execution-audit.json').read_text())
record={'schemaVersion':1,'reviewerRun':'/root/usage_release_review/bing-seo-2026-11-02','executedAt':datetime.datetime.now(datetime.timezone.utc).isoformat(),'description':'Fresh public package basic programs and duplicate-key recovery executed by this independent reviewer; all generated APKG files decoded by the public consumer inspector.'}
for kind in ['sourceBindings','dependencyBindings']:
    record[kind]=[{'path':b['path'],'sha256':hashlib.sha256((root/b['path']).read_bytes()).hexdigest()} for b in oldReport[kind]]
(evidence/'execution-inputs.json').write_text(json.dumps(record,indent=2)+'\n')
print('Fresh basic Rust, Python, Node and duplicate-key recovery passed; APKG content independently decoded.')
