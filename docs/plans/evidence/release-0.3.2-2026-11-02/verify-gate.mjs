// Validates this review's reports with the real release gate, without editing (read-only use of)
// website/post-build-reviews.config.json (a temporary config under target/ selects them).
import {readFile,writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {verifyPostBuildReviews} from '../../../../website/scripts/post-build-reviews.mjs';
const configPath='website/post-build-reviews.config.json';
const out={};
for(const env of ['root','prefix']){
  const bytes=await readFile(`target/release-0.3.2-artifacts/${env}-dist/generated/site-manifest.json`);
  const result=await verifyPostBuildReviews(JSON.parse(bytes),createHash('sha256').update(bytes).digest('hex'),{configPath});
  out[env]=result;console.log(JSON.stringify(result,null,2));
}
await writeFile('docs/plans/evidence/release-0.3.2-2026-11-02/gate-verification.json',JSON.stringify({schemaVersion:1,configPath,verifiedAt:new Date().toISOString(),results:out},null,2)+'\n');
