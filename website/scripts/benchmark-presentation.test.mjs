import test from 'node:test';
import assert from 'node:assert/strict';
import { loadBenchmarkPresentation, presentComparison } from './benchmark-presentation.mjs';

test('the presentation selects the five specified 1,000-note measurements', async () => {
  const { results, maxTime, minimumSaved, maximumSaved } = await loadBenchmarkPresentation();
  assert.deepEqual(results.map(row => [row.rust.toFixed(1), row.genanki.toFixed(1), row.saved.toFixed(1), row.speedup.toFixed(2), row.rustRSS.toFixed(2), row.genankiRSS.toFixed(2)]), [
    ['52.7', '105.0', '49.8', '1.99', '21.91', '32.25'],
    ['150.7', '320.2', '52.9', '2.12', '92.45', '35.59'],
    ['128.7', '275.4', '53.3', '2.14', '62.66', '35.97'],
    ['110.5', '239.5', '53.9', '2.17', '65.19', '35.03'],
    ['61.4', '112.9', '45.7', '1.84', '29.36', '32.66'],
  ]);
  assert.equal(maxTime.toFixed(1), '320.2');
  assert.equal(minimumSaved.toFixed(1), '45.7');
  assert.equal(maximumSaved.toFixed(1), '53.9');
});

test('a slower Rust measurement scales both bars and reverses the time advantage', async () => {
  const { readFile } = await import('node:fs/promises');
  const csv = await readFile(new URL('../../benchmarks/results/20261008-latest-commit-genanki/comparison.csv', import.meta.url), 'utf8');
  const [header, ...rows] = csv.trim().split('\n').map(row => row.split(','));
  const record = rows.find(row => row[0] === 'basic-mixed-text-v1' && row[1] === '1000');
  record[header.indexOf('rust_time_ms_median')] = '1000';
  record[header.indexOf('genanki_time_ms_median')] = '500';
  const presentation = presentComparison([header, ...rows].map(row => row.join(',')).join('\n'), 1000);
  assert.equal(presentation.maxTime, 1000);
  assert.equal(presentation.results[0].saved, -100);
});

test('both README headlines, alt text and all chart variants agree with the selected CSV', async () => {
  const { readFile } = await import('node:fs/promises');
  const { results, minimumSaved, maximumSaved } = await loadBenchmarkPresentation();
  for (const filename of ['README.md', 'README.zh-CN.md']) {
    const markdown = await readFile(new URL(`../../${filename}`, import.meta.url), 'utf8');
    const alt = markdown.match(/alt="([^"]+)"[^>]*width="1000"/)[1];
    for (const row of results) assert.ok(alt.includes(`${row.rust.toFixed(1)} / ${row.genanki.toFixed(1)}`), `${filename}: stale chart alt`);
    assert.ok(markdown.includes(`${minimumSaved.toFixed(1)}–${maximumSaved.toFixed(1)}`), `${filename}: stale headline`);
    for (const row of results) {
      assert.ok(markdown.includes(`${row.rustRSS.toFixed(2)} | ${row.genankiRSS.toFixed(2)} | ${row.speedup.toFixed(2)}×`), `${filename}: stale method table`);
    }
  }
  for (const variant of ['light', 'dark', 'light-mobile', 'dark-mobile']) {
    const svg = await readFile(new URL(`../../docs/assets/readme/export-times-${variant}.svg`, import.meta.url), 'utf8');
    for (const row of results) for (const time of [row.rust, row.genanki]) assert.ok(svg.includes(`<!-- ${time.toFixed(1)} -->`), `Stale ${variant} chart`);
  }
});
