import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

const root = new URL('../../', import.meta.url);
const workloads = [
  ['basic-mixed-text-v1', 'Text only'],
  ['basic-image-unique-v2', 'Unique images'],
  ['basic-audio-unique-v2', 'Unique audio'],
  ['basic-mixed-unique-v2', 'Mixed, unique media'],
  ['basic-mixed-shared-v2', 'Mixed, shared media'],
];

export function presentComparison(csv, notes) {
  const [header, ...records] = csv.trim().split(/\r?\n/).map(row => row.split(','));
  const results = workloads.map(([profile, label]) => {
    const matches = records.filter(row => row[header.indexOf('profile')] === profile && Number(row[header.indexOf('notes')]) === notes);
    assert.equal(matches.length, 1, `Expected one benchmark row: ${profile}/${notes}`);
    const value = name => {
      const number = Number(matches[0][header.indexOf(name)]);
      assert.ok(Number.isFinite(number) && number > 0, `Invalid ${name} for ${profile}`);
      return number;
    };
    const rust = value('rust_time_ms_median');
    const genanki = value('genanki_time_ms_median');
    return { profile, label, rust, genanki, saved: 100 * (1 - rust / genanki), speedup: genanki / rust,
      rustRSS: value('rust_rss_mib_median'), genankiRSS: value('genanki_rss_mib_median') };
  });
  return { results, maxTime: Math.max(...results.flatMap(row => [row.rust, row.genanki])),
    minimumSaved: Math.min(...results.map(row => row.saved)), maximumSaved: Math.max(...results.map(row => row.saved)) };
}

export async function loadBenchmarkPresentation() {
  const config = JSON.parse(await readFile(new URL('docs/benchmark-presentation.json', root), 'utf8'));
  assert.match(config.dataset, /^benchmarks\/results\/[\w-]+$/);
  for (const key of ['measurementCommit', 'evidenceCommit']) assert.match(config[key], /^[a-f0-9]{40}$/);
  assert.ok(Number.isInteger(config.notes) && config.notes > 0);
  return { ...config, ...presentComparison(await readFile(new URL(`${config.dataset}/comparison.csv`, root), 'utf8'), config.notes) };
}
