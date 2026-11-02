// Independent current-artifact observations. Produces no passing review conclusions.
import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { pathToFileURL } from 'node:url';
const [directory, output, repositoryRoot, mode = 'editorial-preview'] = process.argv.slice(2);
assert.ok(directory && output && repositoryRoot, 'Usage: helper artifact-directory output-directory repository-root editorial-preview|release-candidate');
assert.ok(['editorial-preview', 'release-candidate'].includes(mode));
const root = path.resolve(repositoryRoot);
const { chromium } = await import(pathToFileURL(path.join(root, 'website/node_modules/playwright/index.mjs')));

const base='/anki-forge';
const hashBytes=bytes=>createHash('sha256').update(bytes).digest('hex');
await mkdir(output,{recursive:true});
const types = { '.html': 'text/html', '.md': 'text/markdown; charset=utf-8', '.txt': 'text/plain; charset=utf-8', '.js': 'text/javascript', '.css': 'text/css', '.json': 'application/json', '.wasm': 'application/wasm', '.svg': 'image/svg+xml', '.png': 'image/png', '.wav': 'audio/wav', '.woff2': 'font/woff2' };
const httpServer = createServer(async (request, response) => {
  try {
    const requestUrl = new URL(request.url, 'http://localhost');
    assert.ok(requestUrl.pathname.startsWith(base + '/'));
    let relative = decodeURIComponent(requestUrl.pathname.slice(base.length));
    if (relative.endsWith('/')) relative += 'index.html';
    const file = path.resolve(directory, '.' + relative);
    assert.ok(file.startsWith(path.resolve(directory) + path.sep));
    const bytes = await readFile(file);
    response.writeHead(200, { 'Content-Type': types[path.extname(file)] ?? 'application/octet-stream' }); response.end(bytes);
  } catch {
    response.writeHead(404, { 'Content-Type': 'text/html' }); response.end(await readFile(path.join(directory, '404.html')));
  }
});
await new Promise((resolve, reject) => { httpServer.once('error', reject); httpServer.listen(0, '127.0.0.1', resolve); });
const server = { origin: `http://127.0.0.1:${httpServer.address().port}`, close: () => new Promise(resolve => httpServer.close(resolve)) };
const browser = await chromium.launch({ executablePath: '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome', headless: true });
const context=await browser.newContext({javaScriptEnabled:false,viewport:{width:320,height:360}});const page=await context.newPage();
const choices=[['zh-CN','简体中文'],['es','Español'],['pt-BR','Português do Brasil'],['ja','日本語'],['de','Deutsch'],['fr','Français'],['ru','Русский']];const observations=[];
try{
for(const width of [320,390])for(const scale of [1,2]){
 await page.setViewportSize({width,height:360});await page.goto(server.origin+base+'/docs/quickstart/');console.log('Loaded fixture '+width+'/'+scale);await page.evaluate(()=>document.fonts.ready);
 await page.locator('.site-header .language-menu ul').evaluate((list,choices)=>{const original=list.firstElementChild;list.replaceChildren(...choices.map(([id,label])=>{const item=original.cloneNode(true),a=item.querySelector('a');a.textContent=label;a.hreflang=id;return item;}));},choices);
 if(scale===2)await page.evaluate(()=>{const sizes=[...document.querySelectorAll('*')].map(e=>[e,parseFloat(getComputedStyle(e).fontSize)]);for(const [e,size]of sizes)e.style.fontSize=`${size*2}px`;});
 await page.evaluate(css=>{const style=document.createElement('style');style.textContent=css;document.head.append(style);},'.language-menu ul a{display:block;line-height:1.25}@media(max-height:500px){.header-tools .language-menu ul{position:fixed!important;top:auto!important;bottom:16px;right:16px!important;max-width:calc(100vw - 32px);max-height:25dvh}}');
 const summary=page.locator('.site-header .language-menu summary');await summary.focus();await summary.press('Enter');const states=[];
 for(const [id]of choices){await page.keyboard.press('Tab');const state=await page.evaluate(()=>{const e=document.activeElement,r=e.getBoundingClientRect(),top=document.elementFromPoint(r.x+r.width/2,r.y+r.height/2),ul=e.closest('ul'),u=ul.getBoundingClientRect();return{locale:e.hreflang,text:e.textContent,rect:{x:r.x,y:r.y,width:r.width,height:r.height,right:r.right,bottom:r.bottom},panel:{x:u.x,y:u.y,width:u.width,height:u.height,bottom:u.bottom,scrollTop:ul.scrollTop,scrollHeight:ul.scrollHeight},top:{tag:top?.tagName,class:top?.className,text:top?.textContent.trim().slice(0,60)},hit:e.contains(top),inViewport:r.x>=0&&r.right<=innerWidth+1&&r.y>=0&&r.bottom<=innerHeight+1,clientRects:[...e.getClientRects()].map(r=>({x:r.x,y:r.y,width:r.width,height:r.height}))}});const shot=`${width}-${scale}-${id}.png`;await page.screenshot({path:path.join(output,shot)});state.screenshot=path.relative(root,path.join(output,shot));states.push(state);console.log(JSON.stringify({width,scale,...state}));}
 observations.push({width,scale,states});await writeFile(path.join(output,'fixture-observations.json'),JSON.stringify({scope:'Actual copied preview; exact seven-language no-JS fixture and per-element computed-font doubling, observations only',observations},null,2)+'\n');
}
}finally{await context.close();await browser.close();await server.close();}
