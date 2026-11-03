// Re-derives the manifest inputSha256 exactly as website/scripts/site-manifest.mjs does, for the
// current tree and for the same tree with only website/post-build-reviews.config.json replaced by its
// HEAD blob. Proves the rebuild's only input difference is that config file. Read-only.
import {execFileSync} from 'node:child_process';
import {readFile,writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
const sha=b=>createHash('sha256').update(b).digest('hex');
const git=(a,o={})=>execFileSync('git',a,{encoding:o.buffer?undefined:'utf8',maxBuffer:1<<26});
const inputs=git(['ls-files','--cached','--others','--exclude-standard','--','website','docs','anki_forge','bindings','contracts','contract_tools','scripts','skills/anki-forge','.cargo','rust-toolchain','rust-toolchain.toml','Cargo.toml','Cargo.lock','README.md','README.zh-CN.md','.github/workflows/website.yml']).trim().split('\n').filter(Boolean).sort();
const current=[];
for(const f of inputs){if(f.startsWith('docs/plans/')||f.startsWith('docs/superpowers/'))continue;try{current.push([f,sha(await readFile(f))]);}catch(e){if(e.code!=='ENOENT')throw e;}}
const cfg='website/post-build-reviews.config.json';
const headBlob=git(['show',`HEAD:${cfg}`],{buffer:true});
const reverted=current.map(([f,h])=>[f,f===cfg?sha(headBlob):h]);
const head=git(['rev-parse','HEAD']).trim();
const changedVsHead=git(['status','--porcelain','--untracked-files=all','--','website','docs','anki_forge','bindings','contracts','contract_tools','scripts','skills/anki-forge','.cargo','Cargo.toml','Cargo.lock','README.md','README.zh-CN.md','.github']).trim().split('\n').filter(l=>l&&!l.slice(3).startsWith('docs/plans/'));
const manifests={};
for(const env of ['root','prefix']){const b=await readFile(`docs/plans/evidence/release-0.3.2-2026-11-02/${env}-site-manifest.json`);const s=await readFile(`target/release-0.3.2-superseded/${env}-site-manifest.json`);manifests[env]={current:{sha256:sha(b),inputSha256:JSON.parse(b).inputSha256,commit:JSON.parse(b).commit},superseded:{sha256:sha(s),inputSha256:JSON.parse(s).inputSha256,commit:JSON.parse(s).commit}};}
const out={schemaVersion:1,head,inputFiles:current.length,currentInputSha256:sha(JSON.stringify(current)),inputSha256WithHeadConfig:sha(JSON.stringify(reverted)),configCurrentSha256:current.find(([f])=>f===cfg)[1],configHeadSha256:sha(headBlob),workingTreeChangesOutsideEvidence:changedVsHead,manifests};
out.onlyConfigDiffers=out.currentInputSha256===manifests.root.current.inputSha256&&out.inputSha256WithHeadConfig===manifests.root.superseded.inputSha256&&changedVsHead.length===1&&changedVsHead[0].endsWith(cfg);
await writeFile('docs/plans/evidence/release-0.3.2-2026-11-02/rebuild-input-verification.json',JSON.stringify(out,null,2)+'\n');
console.log(JSON.stringify(out,null,2));
if(!out.onlyConfigDiffers)process.exit(1);
