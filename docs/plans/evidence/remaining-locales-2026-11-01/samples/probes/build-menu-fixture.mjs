import { cp, readFile, writeFile, mkdir } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { createHash } from 'node:crypto';
import { configuredLocales } from '../../../../../../website/locale-model.mjs';
const root=process.cwd(),require=createRequire(`${root}/website/package.json`);
const {load}=require('cheerio'),{micromark}=require('micromark');
const {gfmTable,gfmTableHtml}=require('micromark-extension-gfm-table');
const source='target/remaining-locales/pt-br/prefix-artifact',target='target/remaining-locales/sample-probe-artifact',evidence='docs/plans/evidence/remaining-locales-2026-11-01/samples/probes';
await cp(source,target,{recursive:true,errorOnExist:true,force:false});
const inputPaths=[`${source}/generated/site-manifest.json`,`${source}/docs/quickstart/index.html`,'website/src/components/LanguageSwitcher.astro','website/locale-model.mjs'];
const template=await readFile(`${source}/docs/quickstart/index.html`,'utf8');const fixturePaths=[];
for(const [locale,config] of Object.entries(configuredLocales)){
 const $=load(template);$('html').attr('lang',locale);$('title').text(`${config.label} isolated eight-language fixture`);
 const menu=$('details.language-menu'),original=menu.find('li').first().clone();menu.find('summary').attr('lang',locale).text(config.label);menu.find('ul').empty();
 for(const [id,entry] of Object.entries(configuredLocales))if(id!==locale){const li=original.clone();li.find('a').attr({href:`/anki-forge/_locale-probe/${id}/`,lang:id,hreflang:id,'data-section-map':'{}','data-section-ids':'[]'}).text(entry.label);menu.find('ul').append(li);}
 if(['ja','de'].includes(locale)){
  const sampleBase=`docs/plans/evidence/remaining-locales-2026-11-01/samples/${locale}`;
  const md=await readFile(`${sampleBase}/quickstart.md`,'utf8'),api=await readFile(`${sampleBase}/python-api.md`,'utf8'),ui=JSON.parse(await readFile(`${sampleBase}/home-ui.json`,'utf8'));
  inputPaths.push(`${sampleBase}/quickstart.md`,`${sampleBase}/python-api.md`,`${sampleBase}/home-ui.json`);
  $('h1').first().text(md.match(/^# (.*)$/m)[1]);
  $('.sl-markdown-content').first().html(micromark(md.replace(/^# .*\n/,''),{extensions:[gfmTable()],htmlExtensions:[gfmTableHtml()]})+micromark(api,{extensions:[gfmTable()],htmlExtensions:[gfmTableHtml()]})+`<p data-probe-ui></p>`);
  $('[data-probe-ui]').text(`${ui['home.updates']} · ${ui['copy.install']} · ${ui['copy.noScript']}`);
 }
 $('main').prepend('<p style="font-size:12px;border:1px solid;padding:4px" data-probe-notice>Isolated sample fixture: menu has eight configured languages. Only Japanese/German sample main text is translated. These routes are not released.</p>');
 const filename=`${target}/_locale-probe/${locale}/index.html`;await mkdir(`${target}/_locale-probe/${locale}`,{recursive:true});await writeFile(filename,$.html());fixturePaths.push(filename);
}
const hash=b=>createHash('sha256').update(b).digest('hex');const inputs=[];for(const p of [...new Set(inputPaths)])inputs.push({path:p,sha256:hash(await readFile(p))});
await writeFile(`${evidence}/fixture-provenance.json`,JSON.stringify({schemaVersion:1,source,target,method:'Copy frozen prefix artifact; preserve actual compiled header/menu CSS and classes. Replace only fixture menu list and current language; add real ja/de sample main text rendered through micromark. Production source/dist unchanged.',inputs,fixtureFiles:await Promise.all(fixturePaths.map(async path=>({path,sha256:hash(await readFile(path))})))},null,2)+'\n');
console.log('Created isolated eight-language menu/sample fixture:',target);
