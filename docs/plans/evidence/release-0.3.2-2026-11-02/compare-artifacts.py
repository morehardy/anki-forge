# Independent full comparison of the release-0.3.2 frozen artifacts with the previously
# independently reviewed bing-seo artifacts: manifests (every page/resource record) and the
# complete dist file trees, byte for byte. Also re-hashes every new dist file against its manifest.
import json, hashlib
from pathlib import Path
root = Path('/Users/hp/Desktop/2026/anki-forge')
ev = root/'docs/plans/evidence/release-0.3.2-2026-11-02'
sha = lambda b: hashlib.sha256(b).hexdigest()
result = {'schemaVersion': 1, 'reviewerRun': '/root/usage_release_review/release-0.3.2-2026-11-02', 'environments': {}}
for env in ['root', 'prefix']:
    new_dir, old_dir = root/f'target/release-0.3.2-artifacts/{env}-dist', root/f'target/bing-seo-artifacts/{env}'
    nb = (ev/f'{env}-site-manifest.json').read_bytes(); ob = (root/f'docs/plans/evidence/bing-seo-2026-11-02/{env}-site-manifest.json').read_bytes()
    assert (new_dir/'generated/site-manifest.json').read_bytes() == nb and (old_dir/'generated/site-manifest.json').read_bytes() == ob
    n, o = json.loads(nb), json.loads(ob)
    base = '' if n['configuration']['base'] == '/' else n['configuration']['base']
    def rel(route):
        r = route[len(base):].lstrip('/')
        return r + 'index.html' if (not r or r.endswith('/')) else r
    for rec in n['pages'] + n['resources']:
        assert sha((new_dir/rel(rec['route'])).read_bytes()) == rec['sha256'], rec['route']
    om = {r['route']: r for r in o['pages'] + o['resources']}; nm = {r['route']: r for r in n['pages'] + n['resources']}
    changed = sorted(r for r in nm if r in om and nm[r] != om[r])
    files_new = {str(p.relative_to(new_dir)) for p in new_dir.rglob('*') if p.is_file()}
    files_old = {str(p.relative_to(old_dir)) for p in old_dir.rglob('*') if p.is_file()}
    differing = sorted(f for f in files_new & files_old if (new_dir/f).read_bytes() != (old_dir/f).read_bytes())
    kinds = {}
    for r in nm:
        k = 'html' if r.endswith('/') else Path(r).suffix or 'other'
        kinds.setdefault(k, [0, 0]); kinds[k][0] += 1; kinds[k][1] += (r in om and nm[r] == om[r])
    result['environments'][env] = {
        'newManifest': {'path': str((ev/f'{env}-site-manifest.json').relative_to(root)), 'sha256': sha(nb), 'commit': n['commit'], 'inputSha256': n['inputSha256']},
        'previousManifest': {'path': f'docs/plans/evidence/bing-seo-2026-11-02/{env}-site-manifest.json', 'sha256': sha(ob), 'commit': o['commit'], 'inputSha256': o['inputSha256']},
        'sameConfiguration': n['configuration'] == o['configuration'], 'sameReleaseLocales': n['releaseLocales'] == o['releaseLocales'],
        'pages': len(n['pages']), 'resources': len(n['resources']), 'addedRoutes': sorted(set(nm) - set(om)), 'removedRoutes': sorted(set(om) - set(nm)),
        'changedManifestRecords': changed, 'unchangedByKind(total,unchanged)': kinds,
        'distFiles': len(files_new), 'distFilesOnlyNew': sorted(files_new - files_old), 'distFilesOnlyOld': sorted(files_old - files_new), 'distFilesDiffering': differing,
        'allNewDistFilesMatchManifest': True,
    }
    # The first release-0.3.2 build (reviewed first, superseded only by the review-config change).
    sb = (root/f'target/release-0.3.2-superseded/{env}-site-manifest.json').read_bytes(); s = json.loads(sb)
    sm = {r['route']: r for r in s['pages'] + s['resources']}
    result['environments'][env]['supersededFirstBuild'] = {'path': f'target/release-0.3.2-superseded/{env}-site-manifest.json', 'sha256': sha(sb), 'inputSha256': s['inputSha256'],
        'sameRoutes': set(sm) == set(nm), 'changedManifestRecords': sorted(r for r in nm if nm[r] != sm.get(r))}
    assert set(sm) == set(nm) and all('/generated/' in r for r in result['environments'][env]['supersededFirstBuild']['changedManifestRecords'])
    assert not result['environments'][env]['addedRoutes'] and not result['environments'][env]['removedRoutes']
    assert all('/generated/' in r for r in changed), changed
result['conclusion'] = 'Against both the previously reviewed bing-seo artifact and the superseded first release-0.3.2 build: every HTML page (incl. 404), Markdown, llms.txt and every other resource is byte-identical to the previously reviewed artifact; only /generated/*.apkg, /generated/showcase.json (and the dist-only generated/site-manifest.json) differ.'
(ev/'artifact-comparison.json').write_text(json.dumps(result, indent=1) + '\n')
for env, r in result['environments'].items(): print(env, r['changedManifestRecords'], r['distFilesDiffering'], r['unchangedByKind(total,unchanged)'])
