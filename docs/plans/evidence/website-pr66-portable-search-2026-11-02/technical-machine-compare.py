import pathlib, json, hashlib

batch = pathlib.Path('docs/plans/evidence/website-pr66-portable-search-2026-11-02')
load = lambda path: json.loads(path.read_bytes())
digest = lambda value: hashlib.sha256(value).hexdigest()
manifests = {env:load(batch/f'{env}-site-manifest.json') for env in ['prefix','root']}
assert manifests['prefix']['inputSha256'] == manifests['root']['inputSha256']
assert manifests['prefix']['releaseLocales'] == manifests['root']['releaseLocales']
origin = {env:m['configuration']['site'] + m['configuration']['base'].rstrip('/') for env,m in manifests.items()}
captures = {env:load(batch/f'technical-machine-{env}/observations.json') for env in ['prefix','root']}
resources = {env:{x['route']:x for x in captures[env]['inventory']} for env in ['prefix','root']}
records = []
for route, resource in resources['prefix'].items():
    root_route = route.removeprefix(manifests['prefix']['configuration']['base'])
    other = resources['root'][root_route]
    filename = root_route.lstrip('/')
    prefix = pathlib.Path('target/website-pr66-portable-search/prefix-dist',filename).read_bytes()
    root = pathlib.Path('target/website-pr66-portable-search/root-dist',filename).read_bytes()
    assert digest(prefix) == resource['sha256']
    assert digest(root) == other['sha256']
    assert prefix.decode('utf-8').replace(origin['prefix'],origin['root']).encode('utf-8') == root, route
    records.append({'prefixRoute':route,'rootRoute':root_route,'prefixSha256':digest(prefix),'rootSha256':digest(root),'equalAfterOnlyConfiguredUrlSubstitution':True})
assert len(records) == 192
comparison = {'schemaVersion':1,'reviewerRun':'/root/standards_review/pr66-portable-search-dual-machine-2026-11-02','inputSha256':manifests['prefix']['inputSha256'],'soleReplacement':origin,'method':'Compare every current archived Markdown/llms byte array; the only allowed transformation is the complete configured HTTPS site+base string. No whitespace, prose, metadata or escaping normalization.','records':records}
(batch/'technical-machine-dual-configuration-comparison.json').write_text(json.dumps(comparison,ensure_ascii=False,indent=2)+'\n')
excerpts=[]
for locale,tasks in captures['root']['readings'].items():
    entry=pathlib.Path(tasks['python-install'][0]['bodyPath']).read_text()
    excerpts.append({'locale':locale,'task':'llms-entry','url':tasks['python-install'][0]['url'],'excerpt':'\n'.join(entry.splitlines()[:10])})
    for task,items in tasks.items():
        header=pathlib.Path(items[1]['bodyPath']).read_text().splitlines()[:5]
        excerpts.append({'locale':locale,'task':task,'url':items[1]['url'],'header':header})
(batch/'technical-machine-root-reading-excerpts.json').write_text(json.dumps({'scope':'Actual current root HTTP entry and primary header excerpts for independent fresh reading; full paragraphs share prefix reading only via the recorded exact paired-byte comparison.','records':excerpts},ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'pairedResources':len(records),'rootEntryAndTaskExcerpts':len(excerpts),'inputSha256':manifests['prefix']['inputSha256']}))
