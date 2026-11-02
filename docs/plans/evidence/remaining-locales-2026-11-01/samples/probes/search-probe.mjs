import { readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { rankSearchDocuments } from '../../../../../../website/src/lib/search.mjs';
const root=process.cwd(), base='docs/plans/evidence/remaining-locales-2026-11-01/samples', out=`${base}/probes`;
const bytes=async p=>readFile(`${root}/${p}`), hash=b=>createHash('sha256').update(b).digest('hex');
const inputs=[];const read=async p=>{const b=await bytes(p);inputs.push({path:p,sha256:hash(b)});return b.toString('utf8');};
const queries=JSON.parse(await read(`${out}/queries.json`));
await read(`${out}/intentions.md`); await read('website/src/lib/search.mjs');
const observations=[];
for(const locale of ['ja','de']){
 const brief=JSON.parse(await read(`${base}/${locale}/language-brief.json`));
 const ui=JSON.parse(await read(`${base}/${locale}/home-ui.json`));
 const documents=[{id:'home',type:'home',url:`/fixture/${locale}/`,title:ui['home.title'],description:ui['home.description'],terms:[],body:Object.values(ui).join('\n')}];
 for(const id of ['quickstart','json-to-anki','updates','python-api']){
  const body=await read(`${base}/${locale}/${id}.md`);const title=body.match(/^# (.*)$/m)[1];
  const description=body.split(/\n\s*\n/).find(s=>s.trim()&&!s.startsWith('#')&&!s.startsWith('<!--'));
  documents.push({id,type:'docs',url:`/fixture/${locale}/${id}/`,title,description,terms:[],body});
 }
 const glossary=Object.fromEntries(['note','card','deck','namespace','baseline'].map(id=>[id,{terms:[brief.terms[id]]}]));glossary.api={terms:['API']};
 for(const mode of ['lexical','brief-terms-only'])for(const test of queries[locale]){
  const result=rankSearchDocuments(test.query,documents,locale,{concepts:mode==='lexical'?{}:glossary,pagefindResults:documents.map(d=>({url:d.url,content:d.body,excerpt:d.description}))});
  const top=result.slice(0,3).map(r=>({id:r.id,score:r.score,matchedTerms:r.matchedTerms}));
  observations.push({locale,mode,...test,segments:[...new Intl.Segmenter(locale,{granularity:'word'}).segment(test.query)].filter(s=>s.isWordLike).map(s=>s.segment),top3:top,expectedTop3:test.expected===null?result.length===0:top.some(r=>r.id===test.expected)});
 }
}
await writeFile(`${out}/search-observations.json`,JSON.stringify({schemaVersion:1,scope:'Small actual-sample corpus passed through current ranking API; the pagefindResults parameter carries explicitly synthetic body inputs, NOT a real Pagefind index or results. No production locale catalog, concepts or 30-query release acceptance exists here.',node:process.version,icu:process.versions.icu,inputs,observations},null,2)+'\n');
for(const o of observations.filter(o=>o.mode==='lexical'))console.log(o.locale,o.expectedTop3?'target found':'diagnostic miss',JSON.stringify(o.query),'=>',o.top3.map(r=>r.id).join(', '));
