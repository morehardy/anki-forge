# Python の診断と更新ポリシー

ビルドに成功すると、成果物の存在が保証された BuildOutput を返します。失敗すると、観測結果と公開状況を保持する BuildError を送出します。診断だけで成功・失敗を判断することはありません。

```python
import json
from ankiforge import Project, Note, BuildOptions, BuildError

project = Project('course').add('term', Note.basic('Question', 'Answer'))
try:
    output = project.build(BuildOptions.to('course.apkg'))
except BuildError as error:
    print(error.kind, error.code)
    print(json.dumps(error.snapshot(), ensure_ascii=False))
else:
    print(output.artifact.path)
    print(json.dumps(output.report.snapshot(), ensure_ascii=False))
```

判断にはメッセージの解析ではなく、機械処理用の `kind` と `code` を使います。公開 API の各種例外はすべて `causes` に原因チェーンのテキストを保持し、ネイティブ例外をチェーンします。`details` は操作ごとの証拠を保持します。テンプレートのソース位置、メディアパスと上限の測定値、未完了の比較で得られた観測結果、公開状況などです。可能な場合、I/O の原因はチェーンされた OSError になります。`source_details` は入れ子の型付きエラーを保持します。バンドルのテキスト上限の原因には `limit` と `observed`、メディアの原因には `code`、`path`、`limit_exceeded` が含まれます。

BuildReport は観測結果だけを保持します。そのスナップショットには件数、診断、経過時間、完了した比較が含まれます。Output と BuildError のスナップショットは、実際の Success/Failure の結果を含みます。失敗のスナップショットは、一度も公開されなかった保存先と、公開後に耐久性の確認が失敗した保存先を区別します。JSON のパスはファイルを所有せず、一時成果物の生存期間も延ばしません。

<!-- section: compare-before-publication -->
## 公開前に比較する

```python
from ankiforge import CompareOptions, UpdatePolicy

comparison = project.compare(CompareOptions.against('course-v1.apkg'))
print(comparison.findings)
print(comparison.policy)
```

完了した比較は、High リスクを検出しても通常どおり返ります。ポリシーの判断は `comparison.allows_publication` で確認します。基準パッケージを読み取れない、完全な識別情報の証拠がない、候補が無効、上限を超えた場合は CompareError を送出します。その `.report` は失敗前までの観測結果を保持します。

既定のポリシーは High と Critical の検出事項を拒否します。特定のリスク分類を確認した後、許可を明示できます。

```python
policy = UpdatePolicy().allow('RISK.NOTE_REMOVED')
comparison = project.compare(CompareOptions.against('course-v1.apkg').update_policy(policy))
output = project.build(BuildOptions.to('course-v2.apkg')
    .update_from('course-v1.apkg').update_policy(policy))
```

許可は、指定した分類に属するすべての検出事項に適用されます。検出事項は元のリスクと証拠を保持します。一致する検出事項がない許可は警告になりますが、警告によって成功した公開が失敗になることはありません。不明なコード、証拠の欠落、その他のハードエラーは許可できません。比較で返ったすべての分類を自動許可しないでください。

`update_from` は、完全な証拠を内部に持つ前回配布時のオリジナルパッケージを必要とします。Anki から再エクスポートしたファイルには、その証拠が残りません。APKG からノート・カードを省いても、学習者のコレクションから自動削除されることはありません。構造の変更は、想定する Anki のインポート設定に照らして確認してください。

<!-- section: budgets-and-artifact-ownership -->
## 上限と成果物の所有権

```python
from ankiforge import InspectLimits

limits = InspectLimits(max_archive_bytes=3 << 30)
options = BuildOptions.to('course-v2.apkg').update_from('course-v1.apkg').inspect_limits(limits)
```

上限は、検査対象の基準パッケージと候補それぞれに適用します。メディアのインポートには、それより前の段階で別の MediaLimits が適用されます。上限の超過では `details['limit_exceeded']` にリソース、上限、観測値が保持されます。該当する上限を意図的に増やして再試行してください。バンドルでは `NoteType.from_bundle(path, limits=MediaLimits(...))` を渡して各アセットの読み取り上限を調整します。マニフェストとテンプレートのテキストには、別の固定上限が適用されます。必要な検査を無効にはできません。

永続パスが不要な場合は `BuildOptions.temporary()` を使います。ファイルを利用している間は出力または成果物ハンドルを保持してください。`artifact.persist_to(path)` は永続コピーを作成します。永続化に失敗すると、PersistError の詳細は保存先の公開段階を記録します。元のハンドルは引き続き使用できます。

スナップショットとエラー詳細のパスは、有効な Unicode なら文字列です。Python が `surrogateescape` で表現する Unix のバイト名は、JSON では `{"encoding": "unix_bytes", "bytes": [...]}` になります。`Path(os.fsdecode(bytes(value["bytes"])))` で復元できます。対になっていない UTF-16 サロゲートを含む Windows の名前は `{"encoding": "windows_wide", "units": [...]}` となり、各配列要素には元の16ビットコード単位が1つ入ります。エンコードされたパスは元のプラットフォームのパスを表します。実行時の `artifact.path` は引き続き `pathlib.Path` です。`BuildOptions.to`、`update_from`、`CompareOptions.against` はバインディングを通してネイティブのパス値を保持します。

ネイティブ拡張の欠落・不一致は、`BINDING.EXTENSION_UNAVAILABLE` または `BINDING.VERSION_MISMATCH` を示すインポートエラーになります。対応する wheel をインストールするか、maturin でソースを再ビルドしてください。1つの Project での同時操作は拒否されます。並行処理には独立したプロジェクトを使ってください。
