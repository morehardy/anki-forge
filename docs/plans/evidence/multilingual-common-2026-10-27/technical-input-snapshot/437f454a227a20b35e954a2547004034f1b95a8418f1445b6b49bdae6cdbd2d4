import { writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { pages, localeIds } from '../site-model.mjs';
import { reviewInputs, reviewInputFiles, rubricVersion, glossaryVersion } from './content-review-inputs.mjs';

const root = fileURLToPath(new URL('../../', import.meta.url));
const entries = {};
for (const page of pages) {
  entries[page.id] = {};
  for (const locale of localeIds) entries[page.id][locale] = {
    ...(await reviewInputs(page, locale, root)), inputFiles: await reviewInputFiles(page, locale, root),
    technical: { status: 'pending', evidence: [], openIssues: [] },
    linguistic: { status: 'pending', score: {}, evidence: [], openIssues: [] },
  };
}
const report = { schemaVersion: 1, rubricVersion, glossaryVersion, reviewerType: 'ai', reviewerRun: null, entries };
if (process.argv[2]) await writeFile(path.resolve(process.argv[2]), JSON.stringify(report, null, 2) + '\n');
else process.stdout.write(JSON.stringify(report, null, 2) + '\n');
