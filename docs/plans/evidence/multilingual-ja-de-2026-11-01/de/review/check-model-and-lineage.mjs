import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';
import ts from '../../../../../../website/node_modules/typescript/lib/typescript.js';
const here=path.dirname(fileURLToPath(import.meta.url)),root=path.resolve(here,'../../../../../..');
const baseline='270f26aa5653a0f56297f46fd35e586da444622b';
const old=p=>execFileSync('git',['show',`${baseline}:${p}`],{cwd:root,maxBuffer:16*1024*1024});
const hash=s=>createHash('sha256').update(s).digest('hex');
const read=p=>fs.readFile(path.join(root,p),'utf8');
const prior=JSON.parse(old('docs/plans/evidence/multilingual-ja-de-2026-11-01/ja/linguistic-source-review-final.json'));
const refs=new Map();
function visit(value){if(!value||typeof value!=='object')return;if(typeof value.path==='string'&&typeof value.sha256==='string')refs.set(value.path,value.sha256);for(const child of Object.values(value))visit(child);}
visit(prior);
const lineage=[];for(const [file,expected]of refs){const actual=hash(old(file));lineage.push({file,expectedSha256:expected,immutableSha256:actual,matches:expected===actual});}
function parse(source){return ts.createSourceFile('site-model.mjs',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.JS);}
function normalizedWithoutDe(source){const tree=parse(source);const result=ts.transform(tree,[ctx=>node=>ts.visitNode(node,function walk(child){if(ts.isObjectLiteralExpression(child))return ts.factory.updateObjectLiteralExpression(child,child.properties.filter(p=>!(ts.isPropertyAssignment(p)&&p.name.text==='de')).map(p=>ts.visitEachChild(p,walk,ctx)));return ts.visitEachChild(child,walk,ctx);})]);const output=ts.createPrinter({removeComments:true}).printFile(result.transformed[0]);result.dispose();return output;}
function literal(node){if(ts.isStringLiteral(node)||ts.isNumericLiteral(node))return node.text;if(node.kind===ts.SyntaxKind.TrueKeyword)return true;if(node.kind===ts.SyntaxKind.FalseKeyword)return false;if(ts.isArrayLiteralExpression(node))return node.elements.map(literal);if(ts.isObjectLiteralExpression(node))return Object.fromEntries(node.properties.map(p=>[p.name.text,literal(p.initializer)]));throw Error('Nonliteral '+node.getText());}
function array(source,name){const declarations=parse(source).statements.filter(ts.isVariableStatement).flatMap(s=>s.declarationList.declarations);return literal(declarations.find(d=>d.name.getText()===name).initializer);}
const file='website/site-model.mjs',before=old(file).toString(),after=await read(file),patch=JSON.parse(await read('docs/plans/evidence/multilingual-ja-de-2026-11-01/de/authoring/de-data-patch.json'));
const oldNormalized=normalizedWithoutDe(before),newNormalized=normalizedWithoutDe(after);
const metadataChecks=array(after,'documents').map(d=>({topic:d.name,title:d.title.de,description:d.description.de,matchesReadPatch:d.title.de===patch.documents[d.name].title&&d.description.de===patch.documents[d.name].description}));
const groupLabels=array(after,'docGroups').map(d=>d.labels.de),navigation=Object.fromEntries(array(after,'navigation').map(n=>[n.id,n.label.de]));
const result={schemaVersion:1,date:'2026-11-01',status:'pre-final-shared-model-and-lineage-check',finalSourceReport:false,releaseReady:false,baselineCommit:baseline,siteModel:{baselineSha256:hash(before),currentSha256:hash(after),wholeAstEqualAfterRemovingGermanProperties:oldNormalized===newNormalized,interpretation:'When true, all existing metadata, sidebar/navigation labels, route definitions and implementation statements remain identical; only literal de data properties were added.'},germanMetadata:metadataChecks,germanSidebar:{labels:groupLabels,matchesReadPatch:JSON.stringify(groupLabels)===JSON.stringify(patch.docGroups)},germanNavigation:{labels:navigation,matchesReadPatch:JSON.stringify(navigation)===JSON.stringify(patch.navigation)},priorReportReferenceChecks:lineage,menuReading:{file:'website/src/components/LanguageSwitcher.astro',label:'Deutsch',registry:{prefix:'/de',lang:'de',og:'de_DE',dir:'ltr'},judgment:'Read the actual unchanged component: it takes labels and locale routes from the selected release set, renders the Deutsch autonym with matching lang/hreflang, and builds the same-topic destination. Existing five labels remain intact. This is a source-language delta assessment, not a browser/layout result.'},issues:[]};
if(!result.siteModel.wholeAstEqualAfterRemovingGermanProperties)result.issues.push('Site model has changes other than de literal properties');
if(metadataChecks.some(c=>!c.matchesReadPatch)||!result.germanSidebar.matchesReadPatch||!result.germanNavigation.matchesReadPatch)result.issues.push('German model text differs from independently read patch');
if(lineage.some(c=>!c.matches))result.issues.push('Prior report reference hash differs at immutable baseline');
await fs.writeFile(path.join(here,'shared-model-and-prior-lineage-preview.json'),JSON.stringify(result,null,2)+'\n');
console.log(JSON.stringify({wholeModelUnchangedAfterRemovingDe:result.siteModel.wholeAstEqualAfterRemovingGermanProperties,metadata:metadataChecks.length,sidebar:result.germanSidebar,navigation:result.germanNavigation,lineageReferences:lineage.length,lineageAllMatch:lineage.every(c=>c.matches),issues:result.issues},null,2));
