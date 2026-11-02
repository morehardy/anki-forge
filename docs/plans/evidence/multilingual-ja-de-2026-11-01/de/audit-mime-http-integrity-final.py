"""Independently parse actual MIME HTTP responses; no semantic pass is created."""
import hashlib
import json
from html.parser import HTMLParser
from pathlib import Path
import sys

ROOT = Path.cwd()
AREA = Path('docs/plans/evidence/multilingual-ja-de-2026-11-01/de')
ENV = sys.argv[1]
assert ENV in ('prefix', 'root')
sha = lambda data: hashlib.sha256(data).hexdigest()
load = lambda filename: json.loads((ROOT / filename).read_text())
proof = lambda filename: {'path': str(filename), 'sha256': sha((ROOT / filename).read_bytes())}
manifest_path = AREA / f'final-{ENV}-origin-manifest.json'
capture_path = AREA / f'mime-green-final-{ENV}.json'
manifest, capture = load(manifest_path), load(capture_path)
assert manifest['commit'] == '7b5f539ec2b8c34a0a9ac052531cda9571e97725'
assert capture['manifestSha256'] == proof(manifest_path)['sha256']
assert capture['inputSha256'] == manifest['inputSha256']
assert capture['configuration'] == manifest['configuration']
assert capture['readOrigin'].startswith('http://127.0.0.1:')
manifest_records = {record['route']: record for record in manifest['pages'] + manifest['resources']}
expected_pages = ['python-api', 'node-api', 'rust-api', 'media', 'rust-guide']
assert [record['id'] for record in capture['records']] == expected_pages


class Paragraphs(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.main = False
        self.paragraph = None
        self.code = None
        self.result = []

    def handle_starttag(self, tag, attrs):
        if tag == 'main':
            self.main = True
        if self.main and tag == 'p':
            self.paragraph = {'text': '', 'code': []}
        if self.paragraph is not None and tag == 'code':
            self.code = ''

    def handle_data(self, data):
        if self.paragraph is not None:
            self.paragraph['text'] += data
        if self.code is not None:
            self.code += data

    def handle_endtag(self, tag):
        if tag == 'code' and self.code is not None:
            self.paragraph['code'].append(self.code)
            self.code = None
        if tag == 'p' and self.paragraph is not None:
            self.result.append(self.paragraph)
            self.paragraph = None
        if tag == 'main':
            self.main = False


records = []
for record in capture['records']:
    assert len(record['responses']) == 2
    contents = []
    for suffix, response in zip(['', 'index.md'], record['responses']):
        route = record['route'] + suffix
        data = (ROOT / response['path']).read_bytes()
        assert response['url'] == capture['readOrigin'] + route
        assert response['status'] == 200
        assert len(data) == response['bytes']
        assert sha(data) == response['sha256'] == manifest_records[route]['sha256']
        if suffix:
            assert 'charset=utf-8' in response['headers']['content-type'].lower()
            assert 'content-disposition' not in response['headers']
        else:
            assert response['headers']['content-type'].startswith('text/html')
        contents.append(data.decode('utf-8'))
    parser = Paragraphs()
    parser.feed(contents[0])
    paragraph = next(item for item in parser.result if 'NOTE.MEDIA_USAGE_INVALID' in item['text'] and 'image/' in item['text'])
    markdown = next(line for line in contents[1].splitlines() if 'NOTE.MEDIA_USAGE_INVALID' in line and 'image/' in line)
    for mime in ['image/*', 'audio/*', 'video/*']:
        assert mime in paragraph['text'] and mime in paragraph['code']
        assert f'`{mime}`' in markdown
    assert paragraph['text'] == record['text']
    assert markdown == record['markdown']
    records.append({'id': record['id'], 'route': record['route'], 'actualHtmlParagraph': paragraph['text'], 'actualCodeValues': paragraph['code'], 'actualMarkdownParagraph': markdown, 'responses': [{**proof(item['path']), 'url': item['url'], 'status': item['status'], 'headers': item['headers']} for item in record['responses']]})
output = AREA / f'mime-http-integrity-final-{ENV}.json'
assert not (ROOT / output).exists(), 'Do not overwrite recorded integrity observations'
report = {'schemaVersion': 1, 'reviewerRun': f'/root/de_technical_review/de-mime-final-{ENV}-2026-11-01', 'scope': 'Independent parser and hash corroboration for actual five-page/ten-response MIME observations. Reviewer must separately read these exact paragraphs before closing the finding.', 'manifest': proof(manifest_path), 'capture': proof(capture_path), 'records': records}
(ROOT / output).write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
print(json.dumps({'path': str(output), 'pages': 5, 'actualHttpResponses': 10, 'mimeLiterals': 15}))
