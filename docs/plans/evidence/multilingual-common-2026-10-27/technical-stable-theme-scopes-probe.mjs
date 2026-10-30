import fs from 'node:fs/promises';import vm from 'node:vm';import assert from 'node:assert/strict';import ts from '/Users/hp/Desktop/2026/anki-forge/website/node_modules/typescript/lib/typescript.js';
const source=await fs.readFile('/Users/hp/Desktop/2026/anki-forge/website/src/scripts/site.ts','utf8');
const prefix=source.slice(source.indexOf('\n')+1,source.indexOf("document.querySelectorAll<HTMLElement>('[data-workbench]')"));
const code=ts.transpileModule(prefix,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.ES2022}}).outputText;
let cases=0;
for(const locale of ['en','zh-CN'])for(const stored of ['','light','dark','invalid']){
 const controls=[{dataset:{}},{dataset:{}}],radios=[];for(const scope of ['header','docs'])for(const value of ['light','dark','auto']){let checked=false;const o={name:`theme-${scope}-${locale}`,value,disabled:true,events:{},addEventListener(e,fn){this.events[e]=fn;}};Object.defineProperty(o,'checked',{get(){return checked;},set(v){checked=!!v;if(checked)for(const other of radios)if(other!==o&&other.name===o.name)other.checked=false;}});radios.push(o);}
 const media={matches:false,addEventListener(e,fn){this.change=fn;}},html={lang:locale,dataset:{}},saved={};
 const doc={documentElement:html,querySelector(){return{textContent:'{}'};},querySelectorAll(sel){return sel==='[data-theme-control]'?controls:radios;}};
 vm.runInNewContext(code,{document:doc,window:{matchMedia(){return media;}},localStorage:{getItem(){return stored;},setItem(k,v){saved[k]=v;}},translatorFor(){return()=>'';}});
 const verify=value=>{for(const scope of ['header','docs']){const selected=radios.filter(o=>o.name===`theme-${scope}-${locale}`&&o.checked);assert.equal(selected.length,1);assert.equal(selected[0].value,value);}for(const c of controls)assert.equal(c.dataset.themeChoice,value);assert.ok(radios.every(o=>!o.disabled));};
 verify(['light','dark'].includes(stored)?stored:'auto');cases++;
 for(const selected of radios){selected.checked=true;selected.events.change();verify(selected.value);assert.equal(saved['starlight-theme'],selected.value==='auto'?'':selected.value);assert.equal(html.dataset.theme,selected.value==='dark'?'dark':'light');cases++;}
 const auto=radios.find(o=>o.value==='auto');auto.checked=true;auto.events.change();media.matches=true;media.change();verify('auto');assert.equal(html.dataset.theme,'dark');cases++;
}
console.log(JSON.stringify({actualSitePrefixExecuted:true,environment:'Node vm with minimal radio-group DOM double; not browser/build/HTML execution',localeCount:2,initialStorageStates:4,checks:cases,passed:true}));
