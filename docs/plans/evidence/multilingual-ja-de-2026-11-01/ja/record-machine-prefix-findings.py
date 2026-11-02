"""Preserve explicit old-locale lineage and this reviewer's newly read JA answers."""
import hashlib
import json
from pathlib import Path

directory = Path('docs/plans/evidence/multilingual-ja-de-2026-11-01/ja')
previous = Path('docs/plans/evidence/multilingual-pt-br-2026-11-01')
current_path = directory / 'machine-prefix/http-observations.json'
prior_path = previous / 'machine-final-prefix/http-observations.json'
prior_report_path = previous / 'machine-read-prefix.json'
read = lambda path: json.loads(path.read_text())
sha = lambda data: hashlib.sha256(data).hexdigest()
proof = lambda path: {'path': str(path), 'sha256': sha(path.read_bytes())}
current, old, prior_report = read(current_path), read(prior_path), read(prior_report_path)
assert prior_report['originManifest']['sha256'] == old['manifestSha256']
assert sha(Path(prior_report['originManifest']['path']).read_bytes()) == old['manifestSha256']
rows = []
for locale in ['en', 'zh-CN', 'es', 'pt-BR']:
    for task in current['entries'][locale]:
        before = next(item for item in old['entries'][locale] if item['id'] == task['id'])
        for left, right in zip([before['index']] + before['followed'], [task['index']] + task['followed'], strict=True):
            assert left.get('topic') == right.get('topic')
            for response in [left, right]:
                assert sha(Path(response['path']).read_bytes()) == response['sha256']
                assert Path(response['path']).read_text() == response['body']
            removed = []
            normalized = right['body']
            if right['route'].endswith('/llms.txt'):
                removed = [line for line in normalized.splitlines(keepends=True) if line.startswith('- [日本語](https://morehardy.github.io/anki-forge/ja/llms.txt): ')]
                assert len(removed) == 1
                normalized = normalized.replace(removed[0], '')
            assert normalized == left['body']
            rows.append({'locale': locale, 'task': task['id'], 'topic': right.get('topic', 'llms'), 'previousUrl': left['url'], 'previousSha256': left['sha256'], 'currentUrl': right['url'], 'currentSha256': right['sha256'], 'exactAddedLinesRead': removed, 'allOtherBytesEqual': True})
assert len(rows) == 56
audit_path = directory / 'machine-prefix/prior-response-comparison.json'
audit = {'schemaVersion': 1, 'currentCapture': proof(current_path), 'priorCapture': proof(prior_path), 'priorReview': proof(prior_report_path),
         'scope': 'Actual old-four-locale response comparison and explicit retention of prior authored answers. Forty followed Markdown bodies are byte-identical; each of sixteen actual llms starts adds only the displayed Japanese-language index link. Those four localized link descriptions were read; they do not alter task instructions or switch language fallback. No fresh full reading of unchanged old-four-locale prose is claimed.', 'rows': rows}
audit_path.write_text(json.dumps(audit, ensure_ascii=False, indent=2) + '\n')
answers = {locale: {task['id']: task['answer'].split(' Sources: ')[0] for task in entry['aiMachineReadReview']} for locale, entry in prior_report['entries'].items()}

# Written only after reading this candidate's Japanese llms and actual followed
# HTTP Markdown, including setup code, Node API, Python recipe and boundaries.
answers['ja'] = {
    'python-install': '通常の CPython 3.11 以降と対応プラットフォームの wheel を使います。記録済みの検証範囲は3.11〜3.14です。python -m venv .venv で環境を作り、. .venv/bin/activate（Windows は .venv\\Scripts\\Activate.ps1）で有効にして、python -m pip install ankiforge==0.3.1 を実行します。公開例を main.py に保存して python main.py を実行すると、Spanish デッキに hola → hello の Basic カードを1枚含む永続 spanish.apkg が生成されます。名前空間 spanish とノートキー es:hola は後の編集でも維持します。エクスポートには Anki は不要で、インポート結果は Anki で確認します。通常の wheel 利用と Maturin によるソースビルドを混同せず、対象ネイティブプラットフォームを互換性表で確認します。',
    'node-api': 'Node.js 22.13 以降で npm install --include=optional ankiforge@0.3.1 を使い、対応するネイティブ依存パッケージもインストールします。new Project(namespace) に安定キーで Note を追加し、await project.build(BuildOptions.to(path)) を呼びます。BuildOutput は artifact/report/snapshot を持ちますが、BuildReport は観測結果のみで、成否やファイル所有権は持ちません。comparison の null を確認し、オブジェクトの highestRisk と JSON の highest_risk を区別します。build/compare は呼び出し時の所有付きスナップショットを非同期処理します。一時ファイルを使う間は所有者を保持し、最後の close 後は消えます。永続ファイルは残ります。準備済み公開は失敗を含む全試行で所有者を消費するため、再試行には再準備が必要です。TypeScript は NodeNext の設定で npx tsc、.mts の出力は node dist/main.mjs で実行します。',
    'json-update': '実際に読んだ Python の完全プログラムを使う場合、recipe/ に v1.json・v2.json・cell.png・tone.wav を保存し、公開 ankiforge==0.3.1 をインストールします。main.py に例を保存して python main.py recipe output/python を実行します。呼び出し側が JSON を読み、各版にノート3件・カード3枚・元のメディア2個を同梱します。音声は短い合成音で、読み上げ機能ではありません。biology-json と3つの業務 ID を維持し、配布時のオリジナル v1.apkg を残して別パスに v2 を生成します。Anki の再エクスポートは基準に使えません。comparison.json と review.json の許可を確認し、同じ準備済み候補を公開します。この解答だけの変更は RISK.NOTE_CHANGED で、出力ディレクトリに既存の版があれば停止します。両版をインポートして解答と画像・音声を確認します。省略したノートの削除同期や全復習状態の保持は保証しません。tools::load_project は internal-tools 向けで、利用側の公開 API ではありません。',
    'compatibility-constraints': '互換性表は2026-10-13時点の公開0.3.1を記録し、Rust 1.92、Node 22.13および検証時の22/24/26、通常のCPython 3.11〜3.14の範囲を示します。ネイティブパッケージはmacOS arm64/x64、Linux x64 GNU/glibc、Windows x64です。全ホストの成功を意味せず、ブラウザー・Electron・Bun・Alpine/musl・ARM64 Linux/Windowsは記録済みNode検証範囲外です。他のPython実装、free-threaded、サブインタープリターも別途検証が必要です。通常の出力はAnki不要ですが、描画・コーデック・インポート・復習履歴は対象クライアントで確認します。更新はオリジナルAPKGを使い、省略は削除同期になりません。検査上限は基準と候補に個別適用され、0は無制限ではありません。原子的な置換はAPKG単位で、耐久性の保証ではなく、遅いエラーでは公開済みかを確認します。レポートやJSONは一時ファイルを所有しません。',
}
conclusions = {'schemaVersion': 1, 'reviewerRun': '/root/ja_de_infra_audit/ja-machine-prefix-2026-11-01',
    'reviewedCaptures': {'prefix': proof(current_path)},
    'method': 'Twenty current HTTP tasks start at their same-locale llms entry. The four Japanese answers were newly authored after independent reading of actual Japanese responses and complete task-relevant setup/Node/Python-recipe code. The other sixteen answers explicitly retain the preceding independent reviewer’s conclusions after rereading those conclusions and comparing every current task response to the archived prior HTTP bytes. Their forty followed Markdown responses are identical; the sixteen llms starts only add a Japanese index link, whose exact localized text was inspected. This is an incremental judgment for the old four languages, not a new full prose reading or automatic promotion of a previous pass.',
    'supportingEvidence': [{'path': str(audit_path), 'description': 'Actual response comparison: unchanged prior-four-language task instructions and inspected new Japanese llms links.'}, {'path': str(prior_report_path), 'description': 'Explicit immutable lineage for the sixteen retained old-language answers, originally authored by an independent reviewer.'}],
    'answers': answers, 'openIssues': []}
(directory / 'machine-conclusions-prefix.json').write_text(json.dumps(conclusions, ensure_ascii=False, indent=2) + '\n')
print('Recorded four newly read Japanese answers plus sixteen explicitly retained old-language answers, after 56 exact prior/current response comparisons.')
