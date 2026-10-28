import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { siteConfig } from '../site.config.mjs';
import { checkSurface } from './public-surface.mjs';
import { loadHeadingMigrations } from './heading-migrations.mjs';

const website = fileURLToPath(new URL('../', import.meta.url));
const dist = process.argv[2] ? path.resolve(process.argv[2]) : path.join(website, 'dist');
const baselineFile = process.argv[3] || process.env.PUBLIC_SURFACE_BASELINE || path.join(website, 'public-surface-baseline.json');
if (process.env.CI && !process.env.PUBLIC_SURFACE_BASELINE && !process.argv[3]) throw new Error('CI must supply a trusted public surface baseline from the base revision or independently verified artifact');
const baseline = JSON.parse(await readFile(baselineFile, 'utf8'));
const migrations = await loadHeadingMigrations(path.resolve(website, '..'));
const result = await checkSurface(dist, baseline, siteConfig, migrations);
console.log(`Historic public surface preserved: ${result.pages} pages, ${result.machineResources} machine resources from ${result.source}.`);
const integration = JSON.parse(await readFile(process.env.MACHINE_SURFACE_BASELINE || path.join(website, 'machine-surface-baseline.json'), 'utf8'));
const inherited = await checkSurface(dist, integration, siteConfig, migrations);
console.log(`Inherited machine integration preserved: ${inherited.machineResources} resources from ${inherited.source}; production status is recorded separately.`);
