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
const context=await browser.newContext({viewport:{width:320,height:900},reducedMotion:'reduce'});
await context.route('https://static.cloudflareinsights.com/**',r=>r.abort());const page=await context.newPage();
const variants={actual:''};
const results=[];
try{
for(const [name,css]of Object.entries(variants))for(const width of [320,390])for(const height of [600])for(const slug of ['/','/docs/quickstart/','/pt-br/','/pt-br/docs/quickstart/']){
 await page.setViewportSize({width,height});await page.goto(server.origin+base+slug);await page.evaluate(()=>document.fonts.ready);await page.addStyleTag({content:'html{font-size:32px!important}'+css});
 const summary=page.locator('.site-header details.language-menu > summary').first();
 const measurement=await page.evaluate(()=>{
  const rect=e=>{if(!e)return null;const r=e.getBoundingClientRect(),s=getComputedStyle(e);return{x:r.x,y:r.y,width:r.width,height:r.height,right:r.right,bottom:r.bottom,position:s.position,padding:s.padding,maxWidth:s.maxWidth,whiteSpace:s.whiteSpace,text:e.textContent.trim().slice(0,90)}};
  const summary=document.querySelector('.site-header details.language-menu > summary'),r=summary.getBoundingClientRect();
  const samples=[0.1,0.5,0.9].map(f=>{const x=r.left+f*r.width,y=r.top+r.height/2,el=document.elementFromPoint(x,y);return{x,y,covered:!el||!(el===summary||summary.contains(el)),topTag:el?.tagName,topClass:el?.className}});
  const controls=[...document.querySelectorAll('.site-header .wordmark, .site-header .header-tools > *')].filter(e=>e.checkVisibility()).map(e=>{const r=e.getBoundingClientRect(),top=document.elementFromPoint(r.x+r.width/2,r.y+r.height/2);return{...rect(e),tag:e.tagName,class:e.className,hit:e.contains(top),hitTag:top?.tagName,hitClass:top?.className,hitText:top?.textContent.trim().slice(0,80)}});return {controls,documentWidth:document.documentElement.scrollWidth,summary:rect(summary),tools:rect(document.querySelector('.site-header .header-tools')),siteHeader:rect(document.querySelector('.site-header')),outerHeader:rect(document.querySelector('.page > .header')),sidebarButton:rect(document.querySelector('.sl-menu-button')),menu:rect(document.querySelector('.site-header .language-menu ul')),samples,options:[...document.querySelectorAll('.site-header .language-menu a')].map(rect)};
 });
 const filename=`${name}-${slug.startsWith('/pt-br')?'pt':'en'}-${slug.includes('/docs/')?'docs':'home'}-${width}x${height}-200.png`;await page.screenshot({path:path.join(output,filename),animations:'disabled'});results.push({name,width,height,slug,url:page.url(),measurement,screenshot:path.relative(root,path.join(output,filename))});console.log(JSON.stringify(results.at(-1)));await writeFile(path.join(output,'prototype-results.json'),JSON.stringify({scope:'Editorial preview CSS-injection experiment; not release acceptance',variants,results},null,2)+'\n');
}
}finally{await context.close();await browser.close();await server.close();}
