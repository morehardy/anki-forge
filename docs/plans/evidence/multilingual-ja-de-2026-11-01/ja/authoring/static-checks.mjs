import fs from 'node:fs/promises';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { sectionMapping, headings } from '../../../../../../website/scripts/content-sections.mjs';
import { messageParameters } from '../../../../../../website/src/i18n/format-message.mjs';
const here=path.dirname(fileURLToPath(import.meta.url));
const root=path.resolve(here,'../../../../../..');
const read=async p=>fs.readFile(path.join(root,p),'utf8');
const json=async p=>JSON.parse(await read(p));
const inventory=JSON.parse(await fs.readFile(path.join(here,'source-inventory.json'),'utf8'));
const patch=JSON.parse(await fs.readFile(path.join(here,'ja-data-patch.json'),'utf8'));
const issues=[];const outputs={};const sourceHashes={};const documentChecks=[];
const assert=(test,message)=>{if(!test)issues.push(message);};
const hash=s=>createHash('sha256').update(s).digest('hex');
const fences=s=>[...s.matchAll(/^```[^\n]*\n[\s\S]*?^```[^\S\n]*$/gm)].map(m=>m[0]);
for(const [topic,item] of Object.entries(inventory)){
 const file=`docs/ja/${topic==='docs'?'index':topic}.md`;
 const body=await read(file);outputs[file]=hash(body);sourceHashes[item.source]=hash(await read(item.source));
 let expected=item.blocks;
 if(topic==='quickstart')expected=[item.blocks[0],`\x60\x60\x60rust\n${(await read('anki_forge/examples/target_api_basic.rs')).trimEnd()}\n\x60\x60\x60`,item.blocks[1]];
 const actual=fences(body);const same=JSON.stringify(actual)===JSON.stringify(expected);
 assert(same,`${topic}: code block differs`);assert(!body.includes('@@CODE_'),`${topic}: placeholder remains`);
 const delimiterCount=(body.match(/^```/gm)||[]).length;assert(delimiterCount===actual.length*2,`${topic}: unbalanced code fences`);
 let sections;try{sections=sectionMapping(topic,'ja',body);}catch(error){issues.push(error.message);}
 documentChecks.push({topic,file,source:item.source,codeBlocks:actual.length,codeBytesUnchanged:same,sectionMapping:sections??null,headingCount:headings(body).length});
}
const blogChecks=[];
for(const [key,code] of [['your-first-anki-deck','anki_forge/examples/target_api_basic.rs'],['stable-ids-and-deck-updates','website/src/generated/updates.rs']]){
 const file=`website/src/content/blog/ja/${key}.mdx`;const body=await read(file);outputs[file]=hash(body);sourceHashes[code]=hash(await read(code));
 const expected=`\x60\x60\x60rust\n${(await read(code)).trimEnd()}\n\x60\x60\x60`;
 assert(fences(body).length===1&&fences(body)[0]===expected,`${key}: expanded code differs`);
 assert(body.includes('locale: ja')&&body.includes(`translationKey: ${key}`)&&body.includes('date: 2026-09-22'),`${key}: article identity mismatch`);
 let sections;try{sections=sectionMapping(`blog:${key}`,'ja',body);}catch(error){issues.push(error.message);}
 blogChecks.push({file,codeSource:code,codeBytesUnchanged:fences(body)[0]===expected,sectionMapping:sections??null});
}
const dictionaries=[];
for(const directory of ['website/src/i18n','website/src/content/i18n']){
 const en=await json(`${directory}/en.json`),ja=await json(`${directory}/ja.json`);outputs[`${directory}/ja.json`]=hash(await read(`${directory}/ja.json`));
 assert(JSON.stringify(Object.keys(en).sort())===JSON.stringify(Object.keys(ja).sort()),`${directory}: keys differ`);
 for(const key of Object.keys(en)){
  const source=typeof en[key]==='string'?[en[key]]:Object.values(en[key].forms);
  const target=typeof ja[key]==='string'?[ja[key]]:Object.values(ja[key].forms);
  const parameters=messageParameters(source[0]);
  for(const text of target){assert(text.trim().length>0,`${directory}/${key}: empty`);assert(JSON.stringify(messageParameters(text))===JSON.stringify(parameters),`${directory}/${key}: interpolation differs`);
   const bracketTokens=t=>[...t.matchAll(/\[[A-Z_]+\]/g)].map(m=>m[0]).sort();assert(JSON.stringify(bracketTokens(source[0]))===JSON.stringify(bracketTokens(text)),`${directory}/${key}: search parameter differs`);
  }
  if(typeof ja[key]!=='string')for(const cat of new Intl.PluralRules('ja').resolvedOptions().pluralCategories)assert(!!ja[key].forms[cat],`${key}: plural form ${cat} missing`);
 }
 dictionaries.push({file:`${directory}/ja.json`,keys:Object.keys(ja).length,keyParity:true,parametersChecked:true,pluralCategories:new Intl.PluralRules('ja').resolvedOptions().pluralCategories});
}
const sampleReview=(await json('docs/plans/evidence/remaining-locales-2026-11-01/samples/independent-review.json')).locales.ja;
const reusedSamples=[];
for(const key of ['quickstart','json-to-anki','updates','python-api']){
 const file=`docs/ja/${key}.md`;const original=`docs/plans/evidence/remaining-locales-2026-11-01/samples/ja/${key}.md`;
 assert((await read(file))===(await read(original)),`${key}: reused sample changed`);
 const src=inventory[key].source;assert(sourceHashes[src]===sampleReview.sourceInputHashes[src],`${key}: English source changed since sample review`);
 reusedSamples.push({topic:key,targetByteIdentical:true,englishSourceHashUnchanged:true});
}
assert(Object.keys(patch.documents).length===29,'metadata count');assert(Object.keys(patch.searchTerms).length===31,'search terms count');
assert(Object.keys(inventory).every(key=>patch.documents[key]&&patch.searchTerms[key]),'document metadata coverage');
assert(patch.docGroups.length===5&&Object.keys(patch.navigation).length===3,'navigation coverage');
const promptEn=(await json('docs/ai-quickstart.json')).prompts.en;
assert(JSON.stringify(messageParameters(promptEn))===JSON.stringify(messageParameters(patch.prompts.ja)),'prompt parameter mismatch');
for(const required of patch.promptContracts.ja.required)assert(patch.prompts.ja.includes(required),`prompt missing ${required}`);
assert(patch.prompts.ja.length<=patch.promptContracts.ja.maxCharacters,'prompt too long');
assert((await fs.readdir(path.join(root,'docs/ja'))).filter(f=>f.endsWith('.md')).length===29,'doc file count');
assert((await fs.readdir(path.join(root,'website/src/content/blog/ja'))).filter(f=>f.endsWith('.mdx')).length===2,'blog count');
// Validate internal document topic paths and their stable anchors without building shared output.
const routes=new Map(Object.keys(inventory).map(topic=>[topic==='docs'?'/docs/':`/docs/${topic}/`,topic]));
for(const file of Object.keys(outputs).filter(p=>p.endsWith('.md')||p.endsWith('.mdx'))){
 const body=(await read(file)).replace(/^```[^\n]*\n[\s\S]*?^```[^\S\n]*$/gm,'');
 for(const match of body.matchAll(/(?:\]\(|sitePath\(')(\/(?:ja\/)?docs\/[^)'\s]*)/g)){
  const [pathname,anchor]=match[1].replace(/^\/ja\//,'/').split('#');const topic=routes.get(pathname);
  assert(!!topic,`${file}: unknown internal path ${pathname}`);
  if(topic&&anchor){const doc=documentChecks.find(d=>d.topic===topic);const aliases=new Set(Object.values(doc.sectionMapping??{}));const target=await read(doc.file);const ids=[...target.matchAll(/\bid="([^"]+)"/g)].map(m=>m[1]);assert(aliases.has(anchor)||ids.includes(anchor),`${file}: unknown anchor ${pathname}#${anchor}`);}
 }
}
outputs['docs/plans/evidence/multilingual-ja-de-2026-11-01/ja/authoring/ja-data-patch.json']=hash(await fs.readFile(path.join(here,'ja-data-patch.json'),'utf8'));
const result={schemaVersion:1,date:'2026-11-01',actor:'ja-full-author',status:issues.length?'author-checks-failed':'author-static-checks-complete',independentReview:false,releaseReady:false,scope:'Author-only static verification; no content-review approval or build acceptance.',documents:documentChecks,blogs:blogChecks,dictionaries,reusedSamples,patch:{documents:29,searchTerms:31,docGroups:5,navigation:3,promptCharacters:patch.prompts.ja.length,briefStatus:patch.languageBrief.status},sourceHashes,outputs,issues,limitations:['No independent technical or linguistic approval.','No SDK execution, browser, font, Pagefind ranking, complete site build or deployment checks performed by this author.','The new independent query set was not read or used to author vocabulary.']};
await fs.writeFile(path.join(here,'author-static-checks.json'),JSON.stringify(result,null,2)+'\n');
console.log(JSON.stringify({status:result.status,documents:documentChecks.length,blogs:blogChecks.length,dictionaries,issues},null,2));
if(issues.length)process.exitCode=1;
