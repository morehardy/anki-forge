"""Apply an authored locale package once, preserving unrelated locale data."""
import json
import re
import shutil
import sys
from pathlib import Path

root = Path.cwd()
evidence = Path(__file__).resolve().parent
locale = sys.argv[1]
assert locale in ('ja', 'de')
load = lambda p: json.loads(p.read_text())
dump = lambda p, data: p.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n')
patch = load(evidence / locale / 'authoring' / f'{locale}-data-patch.json')
release_file = root / 'website/release-locales.json'
release = load(release_file)
assert release['locales'] == (['en', 'zh-CN', 'es', 'pt-BR'] if locale == 'ja' else ['en', 'zh-CN', 'es', 'pt-BR', 'ja'])
assert len(patch['documents']) == 29 and len(patch['docGroups']) == 5
if locale == 'de':
    staging = evidence / 'de/staging'
    for source in staging.rglob('*'):
        if source.is_file():
            dest = root / source.relative_to(staging)
            assert not dest.exists(), dest
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, dest)
model_path = root / 'website/site-model.mjs'
model = model_path.read_text()
match = re.search(r'const documents = (\[.*?\n\]);', model, re.S)
documents = json.loads(match[1])
assert set(patch['documents']) == {d['name'] for d in documents}
for document in documents:
    for key in ('title', 'description'):
        old = json.dumps(document[key], ensure_ascii=False, separators=(', ', ': '))
        assert old in model and locale not in document[key]
        document[key][locale] = patch['documents'][document['name']][key]
        model = model.replace(old, json.dumps(document[key], ensure_ascii=False, separators=(', ', ': ')), 1)
groups = iter(patch['docGroups'])
model, count = re.subn(r'(\{ labels: \{[^\n]+?)( \}, ids:| \}, collapsed:)', lambda m: m[1] + ', ' + json.dumps(locale) + ': ' + json.dumps(next(groups), ensure_ascii=False) + m[2], model)
assert count == 5
for ident, label in patch['navigation'].items():
    model, count = re.subn(r"(\{ id: '" + re.escape(ident) + r"', label: \{[^\n]+?)( \} \})", lambda m: m[1] + ', ' + json.dumps(locale) + ': ' + json.dumps(label, ensure_ascii=False) + m[2], model)
    assert count == 1
model_path.write_text(model)
for filename, key in [('search-terms.json', 'searchTerms'), ('search-concepts.json', 'searchConcepts')]:
    p = root / 'website' / filename
    data = load(p)
    assert locale not in data['locales']
    data['locales'][locale] = patch[key]
    dump(p, data)
p = root / 'website/language-briefs.json'
data = load(p)
assert locale not in data
data[locale] = patch['languageBrief']
dump(p, data)
p = root / 'docs/ai-quickstart.json'
data = load(p)
for key in ('prompts', 'requestPlaceholders', 'promptContracts'):
    assert locale not in data[key]
    data[key][locale] = patch[key][locale]
dump(p, data)
p = root / 'website/search-queries.json'
data = load(p)
frozen = load(evidence / 'queries' / f'{locale}-queries-frozen.json')
queries = frozen['queries'] if isinstance(frozen, dict) else frozen
assert len(queries) == 30 and locale not in data['locales']
data['locales'][locale] = queries
if locale == 'ja':
    data['provenanceNote'] += ' Japanese and German fixtures were independently frozen from the existing task intents and reviewed language briefs before their new production vocabulary or retrieval results were read; the content/vocabulary authors did not read these queries. See docs/plans/evidence/multilingual-ja-de-2026-11-01/queries/authorship.json for exact frozen-file hashes. They remain AI-authored acceptance cases, not measured user demand.'
dump(p, data)
release['locales'].append(locale)
dump(release_file, release)
dump(root / 'website/post-build-reviews.config.json', {'schemaVersion': 1, 'environments': {env: {'taskReview': f'docs/plans/evidence/multilingual-ja-de-2026-11-01/{locale}/usage-tasks-{env}.json', 'machineReview': f'docs/plans/evidence/multilingual-ja-de-2026-11-01/{locale}/machine-read-{env}.json'} for env in ('root', 'prefix')}})
print(f'Integrated {locale}; release snapshot: {release["locales"]}')
