# Python API

この SDK は、ネイティブ拡張を通じて標準の Rust 公開 API を呼び出します。`py.typed` と検証済みの公開シグネチャを含みます。まずは[クイックスタート](/docs/python-quickstart/)を参照してください。

<!-- section: project-and-note-values -->
## プロジェクトとノートの値

| API | 動作 |
| --- | --- |
| `Project(namespace, *, default_deck=None)` | 明示した安定した名前空間を検証し、デフォルトデッキを設定します |
| `project.add(key, note)` | ノート、モデル、そのノートが所有するメディアを不可分に取り込みます。ノートキーが重複すると失敗します |
| `project.add_asset(media)` | 生の HTML／CSS／スクリプトが参照する依存ファイルを明示的に追加します |
| `len(project)` | 正常に追加されたノートの件数です |
| `Note.basic(front, back)` | 組み込みの Basic モデルを使います。文字列は Text として扱われます |
| `Note.cloze(text)` | 組み込みの Cloze モデルを使います。文字列は Text として扱われ、穴埋め構文は保持されます |
| `note.field(key, content)` | フィールドを設定した新しいノートを返します。キーは追加時に検証されます |
| `note.deck(name)` / `.tag(tag)` / `.tags(tags)` | 指定した設定を持つノートを返します |
| `note.note_type` | ノートが所有する不変のモデルです |

通常の文字列は Text になります。明示的な生成には `Content.text(value)` と `Content.html(value)` を使います。`Content.sequence(iterable)` は、レンダリングせず、メディアへの依存関係も失わずに、文字列とコンテンツを組み合わせます。ファイル名をそのままフィールドに渡してもメディア参照としては扱われません。

<!-- section: models-and-templates -->
## モデルとテンプレート

`NoteType.builder(key)` は不変のビルダーを返します。`.name`、`.field`、`.template`、`.css`、`.cloze_field`、`.asset` の各メソッドは、変更を反映した新しいビルダーを返します。`.build()` は定義全体を検証し、不変の NoteType を返します。`model.note()` は、そのモデルを所有するノートを作成します。`NoteType.from_bundle(path, limits=MediaLimits())` は、現在のバンドル形式から同じモデルと関連ファイルを読み込みます。任意の上限指定は、各ファイルを読み込む前に個別に適用されます。

`Field(key, name=None, required=False, sort=False)` は、安定キーと表示名を分けて扱います。`Template(key, front, back, name=None, browser_front=None, browser_back=None, target_deck=None, generation=GenerationRule())` は、すべてのテンプレートソース文字列でフィールドキーを使います。`GenerationRule.all(keys)` と `.any(keys)` でカード生成条件を設定できます。デフォルトでは Anki のフィールド存在条件を推定します。ビルダーの `.cloze_field(key)` は、独自の穴埋めカード生成を宣言します。

<!-- section: media-and-occlusion -->
## メディアと画像穴埋め

| API | 動作 |
| --- | --- |
| `Media.file(path, *, limits=MediaLimits())` | 直ちにファイルを読み込み、内容のスナップショットを所有します |
| `Media.bytes(data, media_type, *, limits=MediaLimits())` | 明示した MIME タイプとともにバイト列を所有します |
| `media.with_export_name(name)` | 同じスナップショットを共有する、別のファイル名の値を返します。以前の参照の名前は変わりません |
| `media.filename`, `media.media_type`, `len(media)` | スナップショットの読み取り専用メタデータです |
| `media.image()` / `.sound()` | スナップショットを保持する型付きコンテンツを返します |

`MediaLimits(max_bytes=256 << 20)` は、スナップショットを作成する前に、個々のインポートを制限します。大きなメディアは、所有権を持つ一時ストレージに退避される場合があります。名前の衝突は、明示的に追加したファイルと自動収集されたファイルの全体で検証されます。

`Note.image_occlusion(media)` は ImageOcclusionBuilder を返します。`.mask(Mask.rect(key, x, y, width, height))` を連結し、必要なら `.mode(OcclusionMode.HIDE_ONE_GUESS_ONE)` を指定して、最後に `.build()` を呼びます。デフォルトのモードは `HIDE_ALL_GUESS_ONE` です。このビルダーの `.build()` は画像をデコードし、マスクの安定キーと、有限かつ画像内に収まるピクセル座標を検証します。返された Note では `header`、`back_extra`、`comments` を通常どおり設定できます。生成される `image` フィールドと `occlusion` フィールドは手動では設定できません。

<!-- section: operations-and-results -->
## 操作と結果

| API | 結果 |
| --- | --- |
| `project.build(BuildOptions.to(path))` | 永続ファイルが保証された BuildOutput を返します |
| `project.build(BuildOptions.temporary())` | 所有権付きの一時ファイルを持つ BuildOutput を返します |
| `options.update_from(path)` | 完全な基準パッケージの識別情報を使って更新するオプションを返します |
| `project.compare(CompareOptions.against(path))` | 完了した ComparisonReport を返します。公開ポリシーによる判断は検出事項とは別に含まれます |
| `options.update_policy(policy)` | Update／Compare に明示的なリスクポリシーを適用します。Create に指定すると設定エラーになります |
| `options.inspect_limits(limits)` | 基準パッケージと候補に、それぞれ有限の上限を適用します |

BuildOutput は `.artifact`、`.report`、`.snapshot()` を持ちます。BuildReport は `.counts`（ノート数・カード数・メディア数を持つ BuildCounts）、`.diagnostics`、`.comparison`、`.snapshot()` を公開します。レポート自体は処理結果の outcome やファイルの所有者を持ちません。ComparisonReport は `.findings`、`.policy`、`.allows_publication`、`.snapshot()` を公開します。スナップショットは JSON にシリアライズできる通常の辞書であり、成果物の所有権は含みません。

ApkgArtifact は `.path`、`.persist_to(path)`、`.close()` を公開します。`copy.copy` でコピーすると所有者が1つ増えます。コンテキストマネージャーは終了時にその所有権を解放します。一時ファイルは最後の所有者が解放されると消えます。永続パスのファイルは、ハンドルを破棄しても残ります。永続化に失敗しても、元のハンドルは引き続き利用できます。

`UpdatePolicy()` は High と Critical の検出事項をブロックします。`.fail_on(RiskLevel.MEDIUM)` はしきい値を変更します。`.allow('RISK.NOTE_REMOVED')` は、根拠を保持したまま、登録済みカテゴリ全体を明示的に許容します。不明なコードは即座に拒否されます。`InspectLimits` は `max_identity_bytes` を含む、すべての有限のバイト数／件数上限を公開します。必須の検査を無効にすることはできません。

<!-- section: errors-and-concurrency -->
## エラーと並行処理

失敗時には SchemaError、AddError、MediaError、ImageOcclusionError、TemplateBundleError、CompareError、PolicyError、BuildError、PersistError のいずれかが送出されます。それぞれ `kind`、`code`、`details`、`source_details` 内の構造化された検査結果、`causes` 内の原因チェーンのテキストを公開し、ネイティブ例外を `__cause__` として保持します。取得可能な場合は、I/O の原因も連鎖した OSError として保持されます。BuildError は `.report` と `.snapshot()`、CompareError は部分的な `.report` を持ち、PersistError の details には実際に起きた公開処理の情報が含まれます。

ネイティブの Project 操作は GIL を解放し、同じオブジェクトの同時利用を `BINDING.PROJECT_BUSY` で拒否します。独立したプロジェクトは並行して実行できます。プロセスを fork した後は、すべてのネイティブ値とハンドルを作り直してください。引き継がれたハンドルは `BINDING.FORKED_OBJECT` で操作を拒否し、クリーンアップでも親プロセス所有のファイルを削除しません。子プロセスで作った値は通常どおり動作し、クリーンアップされます。[診断](/docs/python-diagnostics/)も参照してください。

<!-- section: addition-context -->
## ノート追加時のコンテキスト

追加に失敗した場合、プロジェクトは変更されません。構造化されたコンテキストには、元のノート／モデルキーと対象が記録され、必要に応じて衝突やメディアの使用箇所の詳細も含まれます。Rust は `note::{AddContext, AddTarget, AddDetail}` を `AddError::context()`／`detail()` で公開し、SDK は型付きのコンテキストと詳細を `AddError.details` で公開します。Node のフィールドは camelCase で、入れ子も含めて不変です。Python は snake_case の TypedDict を使います。タグは挿入時のインデックスを保持します。フィールドの位置は元のシーケンスのインデックス（フィールド自体には `null`／`None`、ルートには `[]`）と、任意の UTF-8 バイト範囲を保持します。ビルド時のデフォルトデッキのエラーにはノート／モデルキーがなく、原因チェーンから確認できます。

型付き画像を追加する場合は image/* MIME が必要です。音声参照には audio/* または video/* を使えます。それ以外のカテゴリでは `NOTE.MEDIA_USAGE_INVALID` が発生します。コンストラクターは引き続き失敗せず、明示的なファイル追加と生の HTML には影響しません。この検査はインポート時に保持した MIME を使うもので、再生できることを保証しません。ビルド時の独立した MIME／拡張子検証も引き続き適用されます。対象と詳細の全項目は[設計文書](https://github.com/morehardy/anki-forge/blob/main/docs/plans/2026-09-28-rust-api-validation-and-errors-design.md)を参照してください。

<!-- section: source-api-details -->
## メディアの一括処理と準備済み公開

メディアの一括処理と準備済み公開は、公開パッケージ `0.3.1` に含まれています。

<!-- section: prepare-review-and-publish -->
### 準備、確認、公開

```python
from ankiforge import Project, Note, BuildOptions, BuildError
project = Project('review-once').add('one', Note.basic('Question', 'Answer'))
previous = project.build(BuildOptions.to('previous.apkg'))
previous.artifact.close()
prepared = project.prepare_publication(
    BuildOptions.to('next.apkg').update_from('previous.apkg')
)
try:
    report = prepared.report  # observations; does not own the candidate
    if report.comparison is None or report.comparison.allows_publication:
        output = prepared.publish()
        output.artifact.close()  # persistent next.apkg remains
except BuildError as error:
    print(error.snapshot()['result'])  # actual publication/durability facts
    raise
finally:
    prepared.close()  # idempotent; deletes an unused candidate
```

公開を一度試みると、ポリシー違反や I/O エラーで失敗した場合も含めて所有者を消費します。その後の呼び出しは `PreparedPublicationStateError` となり、コードは `BUILD.PREPARED_UNAVAILABLE`、理由は `closed` または `consumed` です。close は、すでに実行中の公開処理を取り消しません。相対パスは準備の呼び出し時に確定し、後から Project を変更しても確認対象の APKG は変わりません。レポートは close 後も利用でき、ファイルは保持しません。オプションの変更や再試行には、もう一度準備を行ってください。所要時間には確認待ちの時間は含まれません。遅い段階のエラーでは、置換に成功していても永続性が未確認の場合があります。対応を決める前に、公開処理の実際の結果を確認してください。ネイティブ binding protocol 1 と contract bundle 2.1.0 が必要です。同じバージョン番号でも古いバイナリは拒否されます。
