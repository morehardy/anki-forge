import {readFile,writeFile} from 'node:fs/promises';
import {createRequire} from 'node:module';
import {serveBuiltSite} from '../../../../../../website/scripts/serve-built-site.mjs';
const root=process.cwd(),require=createRequire(`${root}/website/package.json`),{chromium}=require('playwright');
const output='docs/plans/evidence/remaining-locales-2026-11-01/samples/probes';
const server=await serveBuiltSite({directory:'target/remaining-locales/sample-probe-artifact'});
const browser=await chromium.launch({executablePath:'/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',headless:true});
const observations=[];
const matrix=[...['en'].flatMap(locale=>[320,390].flatMap(width=>[1,2].flatMap(textScale=>[true,false].map(javaScriptEnabled=>({locale,width,height:360,textScale,javaScriptEnabled}))))),{locale:'de',width:320,height:360,textScale:2,javaScriptEnabled:false}];
async function resizeText(page,factor){if(factor===1)return;await page.evaluate(f=>{const values=[...document.querySelectorAll('*')].map(el=>[el,parseFloat(getComputedStyle(el).fontSize)]);for(const [el,size] of values)el.style.setProperty('font-size',`${size*f}px`,'important');},factor);}
const metrics=()=>{
 const menu=[...document.querySelectorAll('details.language-menu')].find(el=>el.getBoundingClientRect().width>0 && el.getBoundingClientRect().height>0),ul=menu.querySelector('ul');
 const measure=el=>{const b=el.getBoundingClientRect(),cx=Math.max(0,Math.min(innerWidth-1,b.x+b.width/2)),cy=Math.max(0,Math.min(innerHeight-1,b.y+b.height/2)),hit=document.elementFromPoint(cx,cy);return {text:el.textContent.trim(),href:el.getAttribute('href'),x:b.x,y:b.y,width:b.width,height:b.height,fullyInViewport:b.x>=-1&&b.y>=-1&&b.right<=innerWidth+1&&b.bottom<=innerHeight+1,hasVisibleClickPoint:b.width>0&&b.height>0&&b.bottom>0&&b.top<innerHeight&&b.right>0&&b.left<innerWidth&&(hit===el||el.contains(hit)),fontSize:getComputedStyle(el).fontSize};};
 return {menuOpen:menu.open,viewport:{width:innerWidth,height:innerHeight},windowScroll:{x:scrollX,y:scrollY},documentWidth:document.documentElement.scrollWidth,summary:measure(menu.querySelector('summary')),ul:{...measure(ul),scrollHeight:ul.scrollHeight,clientHeight:ul.clientHeight,scrollTop:ul.scrollTop,overflowY:getComputedStyle(ul).overflowY},items:[...ul.querySelectorAll('a')].map(measure),active:document.activeElement?measure(document.activeElement):null};
};
try{
 for(const scenario of matrix){
  const name=`${scenario.locale}-${scenario.width}-text${scenario.textScale*100}-js${Number(scenario.javaScriptEnabled)}`;
  const ctx=await browser.newContext({viewport:{width:scenario.width,height:scenario.height},javaScriptEnabled:scenario.javaScriptEnabled,reducedMotion:'reduce'});await ctx.route('https://static.cloudflareinsights.com/**',r=>r.abort());const page=await ctx.newPage();page.setDefaultTimeout(5000);
  const url=server.origin+`/anki-forge/_locale-probe/${scenario.locale}/`,record={...scenario,name,url,tabSequence:[],activations:[]};
  try{
   await page.goto(url);await resizeText(page,scenario.textScale);await page.locator('.language-menu:visible summary').focus();await page.keyboard.press('Enter');record.initial=await page.evaluate(metrics);
   await page.screenshot({path:`${output}/${name}-open.png`});
   for(let i=0;i<7;i++){await page.keyboard.press('Tab');record.tabSequence.push(await page.evaluate(metrics));}
   record.afterTab=await page.evaluate(metrics);
   const rect=record.initial.ul;await page.mouse.move(Math.max(2,Math.min(scenario.width-2,rect.x+rect.width/2)),Math.max(2,Math.min(330,rect.y+20)));await page.mouse.wheel(0,600);await page.waitForTimeout(150);record.afterWheel=await page.evaluate(metrics);
   await page.screenshot({path:`${output}/${name}-after-scroll.png`});
   for(let i=0;i<7;i++){
    await page.goto(url);await resizeText(page,scenario.textScale);await page.locator('.language-menu:visible summary').focus();await page.keyboard.press('Enter');for(let j=0;j<=i;j++)await page.keyboard.press('Tab');
    const focused=await page.evaluate(metrics),href=focused.active.href;await Promise.all([page.waitForURL(u=>u.pathname!==new URL(url).pathname),page.keyboard.press('Enter')]);
    record.activations.push({index:i,focused:focused.active,href,arrived:page.url(),matches:page.url()===new URL(href,url).href});
   }
   record.allSevenKeyboardActivatable=record.activations.length===7&&record.activations.every(r=>r.matches);
   record.allTabTargetsVisible=record.tabSequence.every(r=>r.active.fullyInViewport&&r.active.hasVisibleClickPoint);
   record.allItemsInitiallyFit=record.initial.items.every(r=>r.fullyInViewport);
   record.noHorizontalOverflow=record.initial.documentWidth<=scenario.width+1&&record.initial.items.every(r=>r.x>=-1&&r.x+r.width<=scenario.width+1);
  }catch(error){record.error=error.stack;}
  observations.push(record);console.log(name,JSON.stringify({keyboard:record.allSevenKeyboardActivatable,visibleFocus:record.allTabTargetsVisible,initialFit:record.allItemsInitiallyFit,horizontal:record.noHorizontalOverflow,error:record.error?.split('\n')[0]}));
  await ctx.close();
 }
 const page=await browser.newPage({viewport:{width:390,height:680}});for(const locale of ['ja','de']){await page.goto(server.origin+`/anki-forge/_locale-probe/${locale}/`);await page.screenshot({path:`${output}/sample-${locale}-390.png`});}await page.close();
 await writeFile(`${output}/menu-observations.json`,JSON.stringify({schemaVersion:1,scope:'Copied frozen artifact with eight configured languages and actual ja/de sample text; no product change or release claim. 200% means computed CSS font-size doubled, not native browser page zoom.',browser:await browser.version(),origin:server.origin,observations},null,2)+'\n');
}finally{await browser.close();await server.close();}
