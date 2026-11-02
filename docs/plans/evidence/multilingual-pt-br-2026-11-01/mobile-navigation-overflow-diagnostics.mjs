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
const context=await browser.newContext({viewport:{width:390,height:600},reducedMotion:'reduce'});await context.route('https://static.cloudflareinsights.com/**',r=>r.abort());const page=await context.newPage();const observations=[];
try{for(const variant of ['actual','relative-to-tools'])for(const width of [320,390])for(const route of ['/docs/quickstart/','/zh-cn/docs/quickstart/','/pt-br/docs/quickstart/']){
 await page.setViewportSize({width,height:600});await page.goto(server.origin+base+route);await page.evaluate(()=>document.fonts.ready);await page.evaluate(()=>document.documentElement.style.fontSize='200%');
 if(variant==='relative-to-tools')await page.evaluate(()=>{const s=document.createElement('style');s.textContent='.header-tools .mobile-menu{position:static!important}.header-tools .mobile-menu nav{max-width:100%;top:100%;right:0}';document.head.append(s)});
 const menu=page.locator('.site-header .mobile-menu summary');await menu.focus();await menu.press('Enter');const geometry=await page.evaluate(()=>{const r=e=>{const x=e.getBoundingClientRect();return{x:x.x,y:x.y,right:x.right,bottom:x.bottom,width:x.width}};return{summary:r(document.querySelector('.site-header .mobile-menu summary')),nav:r(document.querySelector('.site-header .mobile-menu nav')),firstLink:r(document.querySelector('.site-header .mobile-menu nav a')),tools:r(document.querySelector('.site-header .header-tools'))}});const filename=`${variant}-${route.split('/')[1]==='docs'?'en':route.split('/')[1]}-${width}.png`;await page.screenshot({path:path.join(output,filename)});const o={variant,width,route,geometry,screenshot:path.relative(root,path.join(output,filename))};observations.push(o);console.log(JSON.stringify(o));await writeFile(path.join(output,'navigation-observations.json'),JSON.stringify({scope:'Copied actual preview; mobile nav width diagnosis and CSS injection only',observations},null,2)+'\n');
}}finally{await context.close();await browser.close();await server.close();}
