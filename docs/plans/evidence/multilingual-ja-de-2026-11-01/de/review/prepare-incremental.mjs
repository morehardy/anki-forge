import fs from 'node:fs/promises';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { pages } from '../../../../../../website/site-model.mjs';
const here=path.dirname(fileURLToPath(import.meta.url)),root=path.resolve(here,'../../../../../..');
const sha=async p=>createHash('sha256').update(await fs.readFile(path.join(root,p))).digest('hex');
const active=['en','zh-CN','es','pt-BR','ja'];
const files=new Set();
for(const page of pages)for(const locale of active)if(page.sources?.[locale])files.add(page.sources[locale]);
for(const locale of active){files.add(`website/src/i18n/${locale}.json`);files.add(`website/src/content/i18n/${locale}.json`);}
const shared=['website/locale-model.mjs','website/src/components/LanguageSwitcher.astro','website/site-model.mjs','docs/ai-quickstart.json','website/language-briefs.json'];
for(const file of shared)files.add(file);
const hashes={};for(const file of files)hashes[file]=await sha(file);
const quickstart=JSON.parse(await fs.readFile(path.join(root,'docs/ai-quickstart.json'),'utf8'));
const promptSnapshot=Object.fromEntries(active.map(locale=>[locale,{prompt:quickstart.prompts[locale],placeholder:quickstart.requestPlaceholders[locale],contract:quickstart.promptContracts[locale]}]));
const result={schemaVersion:1,status:'preparation-only-no-approval',date:'2026-11-01',reviewerTask:'/root/ja_full_author',activeLocales:active,releaseBeforeGerman:JSON.parse(await fs.readFile(path.join(root,'website/release-locales.json'),'utf8')),fileHashes:hashes,promptSnapshot,plannedScope:['Compare finalized DE candidate against this pre-DE five-locale snapshot.','Read actual differences in shared language-menu labels, old-locale metadata/UI/body inputs, and AI prompt dependencies before issuing any incremental judgment.','The added native language label Deutsch is appropriate in every existing locale; navigation still needs final-candidate inspection.','Do not rereview unchanged full bodies by hash alone; preserve their prior independent evidence and describe the true delta.','Any body/engine mutation expands the affected reading scope.','This reviewer authored the Japanese full candidate and does not approve the original Japanese prose. Any later Japanese delta review must be explicitly limited to independently authored shared additions and refer to the prior independent Japanese approval.'],awaiting:['Final six-locale candidate and exact source hashes','Finalized DE fixes and shared integration','Actual shared diff, including any post-build modifications'],limitations:['No final source approval has been issued by this preparatory record.','No independent search-query file was opened.']};
await fs.writeFile(path.join(here,'incremental-five-locale-before-de.json'),JSON.stringify(result,null,2)+'\n');
console.log(JSON.stringify({files:files.size,locales:active,status:result.status}));
