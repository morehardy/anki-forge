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

const manifestBytes = await readFile(path.join(directory, 'generated/site-manifest.json'));
const manifest = JSON.parse(manifestBytes);
const base = manifest.configuration.base === '/' ? '' : manifest.configuration.base;
const hashBytes = bytes => createHash('sha256').update(bytes).digest('hex');
await mkdir(output, { recursive: true });
await writeFile(path.join(output, 'origin-manifest.json'), manifestBytes);
const artifactRecords = [...manifest.pages, ...manifest.resources];
for (const record of artifactRecords) {
  let relative = record.route.slice(base.length).replace(/^\//, '');
  if (!relative || relative.endsWith('/')) relative += 'index.html';
  assert.equal(hashBytes(await readFile(path.join(directory, relative))), record.sha256, `Artifact bytes changed: ${record.route}`);
}
await writeFile(path.join(output, 'artifact-integrity.json'), JSON.stringify({ schemaVersion: 1, observation: 'Compared every archived artifact page/resource byte digest with its actual manifest before browser observation.', manifestSha256: hashBytes(manifestBytes), pages: manifest.pages.length, resources: manifest.resources.length, matched: artifactRecords.length }, null, 2) + '\n');
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
const context = await browser.newContext({ viewport: { width: 320, height: 900 }, reducedMotion: 'reduce' });
await context.route('https://static.cloudflareinsights.com/**', route => route.abort());
const page = await context.newPage();
page.setDefaultTimeout(15000);
const home = manifest.pages.find(p => p.id === 'home' && p.locale === 'pt-BR');
const quickstart = manifest.pages.find(p => p.id === 'python-quickstart' && p.locale === 'pt-BR');
assert.ok(home && quickstart);
const observations = { schemaVersion: 1, scope: 'actual-browser-layout-and-input-observations-only', mode, manifestSha256: hashBytes(manifestBytes), commit: manifest.commit, inputSha256: manifest.inputSha256, configuration: manifest.configuration, releaseLocales: manifest.releaseLocales, helper: {path: path.relative(root,process.argv[1]),sha256:hashBytes(await readFile(process.argv[1]))}, readOrigin: server.origin, layout: [], shortViewport: [], keyboard: null, noScript: null, pageErrors: [] };
page.on('pageerror',error=>observations.pageErrors.push({url:page.url(),message:error.message}));
const save = () => writeFile(path.join(output,'accessibility-observations.json'), JSON.stringify(observations,null,2)+'\n');
const shot = async (target,name) => { const file=path.join(output,name+'.png');await target.screenshot({path:file,animations:'disabled'});return{path:path.relative(root,file),sha256:hashBytes(await readFile(file))};};
try {
  for(const target of [home,quickstart]) for(const width of [320,390,768,1440]) for(const scale of [1,2]) {
    await page.setViewportSize({width,height:900});await page.goto(server.origin+target.route);
    await page.evaluate(()=>document.fonts.ready);
    const originalFont=await page.locator('html').evaluate(el=>parseFloat(getComputedStyle(el).fontSize));
    if(scale===2) await page.addStyleTag({content:`html { font-size: ${originalFont*2}px !important; }`});
    const summary=page.locator('.site-header details.language-menu > summary').first();assert.equal(await summary.innerText(),'Português do Brasil');
    const measure=()=>page.evaluate(()=>{
      const visible=el=>{const r=el.getBoundingClientRect();return el.checkVisibility({checkOpacity:true,checkVisibilityCSS:true})&&r.width>0&&r.height>0;};
      const describe=el=>{const r=el.getBoundingClientRect();return{tag:el.tagName,text:el.textContent.trim(),x:r.x,y:r.y,width:r.width,height:r.height,right:r.right,bottom:r.bottom};};
      const summary=document.querySelector('.site-header details.language-menu > summary');const sr=summary.getBoundingClientRect();const summaryOcclusion=[0.1,0.5,0.9].map(f=>{const x=sr.left+sr.width*f,y=sr.top+sr.height/2,top=document.elementFromPoint(x,y);return{x,y,visible:!!top&&(top===summary||summary.contains(top)),topTag:top?.tagName,topClass:top?.className,topLabel:top?.getAttribute('aria-label')};});
      return {summaryOcclusion,mobileNavigationButtons:[...document.querySelectorAll('.sl-menu-button')].filter(visible).map(describe),viewportWidth:innerWidth,documentWidth:document.documentElement.scrollWidth,bodyWidth:document.body.scrollWidth,htmlFontSize:getComputedStyle(document.documentElement).fontSize,bodyFontSize:getComputedStyle(document.body).fontSize,languageSummary:describe(document.querySelector('.site-header details.language-menu > summary')),visibleHeaderControls:[...document.querySelectorAll('header a,header button,header select,header summary')].filter(visible).map(describe),languageOptions:[...document.querySelectorAll('.site-header details.language-menu a')].filter(visible).map(describe)};
    });
    const closed=await measure();await summary.click();const expanded=await measure();
    const screenshot=await shot(page,`pt-BR-${target.id}-${width}-text-${scale*100}-menu`);
    observations.layout.push({pageId:target.id,url:page.url(),width,textScale:scale,textScaleMethod:'Measured original html font-size and multiplied it by two with an injected style; this observes CSS text enlargement, not claimed browser UI zoom.',closed,expanded,screenshot});
    await save();console.log(`${target.id} ${width}px ${scale*100}% text: closed width ${closed.documentWidth}, open width ${expanded.documentWidth}; summary right ${expanded.languageSummary.right}`);
  }
  for(const target of [home,quickstart])for(const width of [320,390]){
    await page.setViewportSize({width,height:360});await page.goto(server.origin+target.route);await page.evaluate(()=>document.fonts.ready);const font=await page.locator('html').evaluate(e=>parseFloat(getComputedStyle(e).fontSize));await page.addStyleTag({content:`html{font-size:${font*2}px!important}`});
    await page.locator('.site-header details.language-menu > summary').first().click();
    const before=await page.evaluate(()=>{const ul=document.querySelector('.site-header .language-menu ul'),r=ul.getBoundingClientRect(),summary=document.querySelector('.site-header .language-menu summary'),sr=summary.getBoundingClientRect();return{documentWidth:document.documentElement.scrollWidth,panel:{x:r.x,y:r.y,right:r.right,bottom:r.bottom,scrollHeight:ul.scrollHeight,clientHeight:ul.clientHeight},summary:{x:sr.x,y:sr.y,right:sr.right,bottom:sr.bottom}}});
    for(let i=0;i<3;i++)await page.keyboard.press('Tab');const lastFocus=await page.evaluate(()=>{const e=document.activeElement,r=e.getBoundingClientRect(),ul=e.closest('ul'),p=ul.getBoundingClientRect();return{text:e.textContent,href:e.href,rect:{x:r.x,y:r.y,right:r.right,bottom:r.bottom},panel:{y:p.y,bottom:p.bottom,scrollTop:ul.scrollTop}}});
    assert.equal(lastFocus.text,'Español');assert.ok(lastFocus.rect.bottom<=360&&lastFocus.rect.y>=lastFocus.panel.y);assert.ok(before.panel.bottom<=360);
    const screenshot=await shot(page,`pt-BR-${target.id}-${width}x360-text-200-last-option`);await page.keyboard.press('Enter');const destination=manifest.pages.find(p=>p.id===target.id&&p.locale==='es');await page.waitForURL(server.origin+destination.route);
    observations.shortViewport.push({pageId:target.id,startUrl:server.origin+target.route,width,height:360,textScale:2,before,lastFocus,followedLastOptionUrl:page.url(),screenshot});await save();
  }
  await page.setViewportSize({width:320,height:900});await page.goto(server.origin+home.route);
  const focusTrace=[];let reached=false;
  for(let i=0;i<32;i++){
    await page.keyboard.press('Tab');
    const f=await page.evaluate(()=>({tag:document.activeElement.tagName,text:document.activeElement.textContent?.trim().slice(0,160),target:document.activeElement.matches('details.language-menu > summary')}));focusTrace.push(f);
    if(f.target){reached=true;break;}
  }
  assert.ok(reached,'Could not reach pt-BR language summary with Tab');await page.keyboard.press('Enter');
  assert.equal(await page.locator('.site-header details.language-menu').first().getAttribute('open'),'');
  await page.keyboard.press('Tab');
  const targetBeforeEnter=await page.evaluate(()=>({tag:document.activeElement.tagName,text:document.activeElement.textContent.trim(),href:document.activeElement.href}));
  assert.equal(targetBeforeEnter.text,'English');await page.keyboard.press('Enter');await page.waitForURL(server.origin+manifest.pages.find(p=>p.id==='home'&&p.locale==='en').route);
  observations.keyboard={startUrl:server.origin+home.route,focusTrace,openedViaEnter:true,targetBeforeEnter,finalUrl:page.url(),finalLang:await page.locator('html').getAttribute('lang'),screenshot:await shot(page,'keyboard-language-switch')};await save();
  const nojs=await browser.newContext({viewport:{width:320,height:900},javaScriptEnabled:false,reducedMotion:'reduce'});const np=await nojs.newPage();
  await np.goto(server.origin+home.route);const nojsBody=await np.locator('main').innerText();
  const prompts=await np.locator('[data-ai-panel] [data-ai-prompt]').allTextContents();
  assert.equal(prompts.length,3);assert.ok(prompts.every(t=>t.includes('pacote publicado')&&t.includes('APKG original distribuído')));
  const nojsShot=await shot(np,'no-js-home-320');
  const nojsTrace=[];let nojsReached=false;
  for(let i=0;i<32;i++){await np.keyboard.press('Tab');const f=await np.evaluate(()=>({tag:document.activeElement.tagName,text:document.activeElement.textContent?.trim().slice(0,160),target:document.activeElement.matches('details.language-menu > summary')}));nojsTrace.push(f);if(f.target){nojsReached=true;break;}}
  assert.ok(nojsReached);await np.keyboard.press('Enter');assert.equal(await np.locator('.site-header details.language-menu').first().getAttribute('open'),'');
  const menuLabels=await np.locator('.site-header details.language-menu a').allTextContents();
  await np.locator('.site-header details.language-menu > summary').first().click();
  const rustRoute=manifest.pages.find(p=>p.id==='quickstart'&&p.locale==='pt-BR').route;
  await np.locator(`main a[href="${rustRoute}"]`).first().click();await np.waitForURL(server.origin+rustRoute);
  const docBody=await np.locator('main').innerText();assert.ok(docBody.includes('cargo run')&&docBody.includes('spanish.apkg'));
  observations.noScript={startUrl:server.origin+home.route,homeBody:nojsBody,visibleCompletePrompts:prompts,languageFocusTrace:nojsTrace,menuOpenedViaEnter:true,menuLabels,quickstartUrl:np.url(),quickstartLang:await np.locator('html').getAttribute('lang'),quickstartBody:docBody,homeScreenshot:nojsShot,quickstartScreenshot:await shot(np,'no-js-rust-quickstart-320')};await save();await nojs.close();
  console.log('Captured pt-BR layout/text enlargement, keyboard menu navigation and no-JavaScript prompt/quickstart access.');
} catch(error){observations.failure={message:error.message,url:page.url()};await save();throw error;}
finally{await context.close();await browser.close();await server.close();}
