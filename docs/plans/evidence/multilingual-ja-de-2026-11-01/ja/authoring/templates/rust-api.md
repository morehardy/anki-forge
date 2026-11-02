# Rust API

よく使う値は crate のルートにあります。`Project`、`Note`、`NoteType`、`Field`、`Template`、`Content`、`Media`、`BuildOptions`、`BuildOutput` です。公開ドメインモジュールは `note`、`schema`、`media`、`build`、`update`、`diagnostics` です。リポジトリのツール群は、feature で有効にする別の API 群です。利用側のアプリケーションに必要ありません。

<!-- section: authoring -->
## コンテンツの作成

| API | 役割 |
| --- | --- |
| `Project::new(namespace)` | 配布物の安定した名前空間を検証する |
| `project.default_deck(name)` | 既定の配置先を設定する |
| `project.add(key, note)` | ノート、モデル、メディアをアトミックに収集する |
| `project.add_asset(media)` | 直接記述した HTML・CSS・スクリプトの明示的アセットを追加する |
| `Note::basic(front, back)` / `Note::cloze(text)` | 組み込みモデルを保持するノートを作る |
| `model.note().field(key, content)` | 不変モデルを保持する独自のノートを作る |
| `note.deck(name).tag(tag)` / `.tags(iterable)` | 追加前にノートを設定する |
| `NoteType::builder(key)...build()` | 不変モデルを検証して完成させる |
| `NoteType::from_bundle(path)` | bundle-v2 からモデルと、データを所有するアセットを読み込む |

`Field::new(key).name(display)` と `Template::new(key).name(display)` は、識別情報と表示を分けます。テンプレートは、フィルター、条件セクション、ブラウザーテンプレートを含めてフィールドキーを参照します。`Field::required()` は内容が空でないことを検査し、`Field::sort()` は並べ替え用フィールドを選択します。`schema::GenerationRule::all` と `any` はフィールドキーを受け取ります。`builder.cloze_field(key)` は Cloze モデルを宣言します。

`schema::FieldKey` と `TemplateKey` は別の型です。文字列リテラル、所有する文字列、借用する文字列、`Cow<str>`、借用・所有するキーを組み合わせて使えます。変換は元の値を保持し、キーの検証はスキーマ完成時に行います。テンプレートキーをフィールドキーとして暗黙に受け付けることはありません。

<!-- section: content-and-media -->
## コンテンツとメディア

通常の文字列はすべて `Content::text` になります。HTML には `Content::html` が必要です。`Content::sequence` は意味を持つ値を組み合わせ、ビルド時に描画します。`Media::file(path)` と `Media::bytes(bytes, mime)` はスナップショットを所有します。`with_limits` 版は読み取り前に `media::MediaLimits { max_bytes }` の上限を適用します。既定値はアセットごとに256 MiBです。`with_export_name(name)` は、同じスナップショットを共有する改名済みの値を返します。既存のクローンは名前を保持します。

`media.image()` と `media.sound()` はリソースの依存関係を保持します。固定ファイル名は移植可能で、衝突しない名前である必要があります。明示的アセットは、静的に見える参照がなくても同梱されます。[メディア](/docs/media/)を参照してください。

`Note::image_occlusion(media)` は `note::ImageOcclusionBuilder` を返します。`.mask(Mask::rect(key, x, y, width, height)).mode(mode).build()` は、デコード後の画像サイズ、矩形、一意の安定キーを検証します。`HideAllGuessOne` と `HideOneGuessOne` の両方に対応しています。[画像穴埋め](/docs/image-occlusion/)を参照してください。

<!-- section: build-options -->
## ビルドのオプション

`project.build(BuildOptions::to(path))` は永続 APKG を公開します。`BuildOptions::temporary()` は所有権で寿命を管理する一時出力を返します。どちらも `.update_from(previous_apkg)`、`.inspect_limits(limits)`、`.update_policy(policy)` を受け付けます。基準パッケージなしでポリシーを指定すると設定エラーになります。setter の順番はこの規則に影響しません。

`build::InspectLimits::default()` は有限の上限を設定します。公開フィールドは変更可能です。値はすべて `u64` で、0は無制限ではなく0を意味します。

| フィールド | 既定値 |
| --- | --- |
| `max_archive_bytes` | 2 GiB |
| `max_entries` | 100,000 |
| `max_central_directory_bytes` | 16 MiB |
| `max_zip_entry_bytes` / `max_zip_total_bytes` | 1 GiB / 4 GiB |
| `max_meta_bytes` / `max_media_map_bytes` | 64 KiB / 16 MiB |
| `max_identity_bytes` | 64 MiB |
| `max_collection_bytes` / `max_media_bytes` | 512 MiB / 256 MiB |
| `max_decoded_total_bytes` / `max_zstd_window_bytes` | 4 GiB / 64 MiB |

上限は基準パッケージと候補の検査に独立して適用されます。検査に失敗すると、リソース、上限、観測量、取得可能な場合は関連エントリーを報告します。上限を増やすには、呼び出し側で明示的に指定します。メディアのインポートには、それより前の段階で別の上限を適用します。

<!-- section: build-compare-and-output -->
## ビルド、比較、出力

`BuildOutput::artifact()` は、データを所有する `build::ApkgArtifact` への参照を返します。`BuildOutput::report()` は観測結果を返します。成果物のクローンは一時ファイルの生存期間を共有します。`persist_to` は永続所有者を返すか、公開状況と元の I/O エラーを持つ `build::PersistError` を返します。

`project.compare(update::CompareOptions::against(path))` は、高リスクの検出事項によりポリシーが公開を阻止しても、完了した `ComparisonReport` を返します。`findings()`、`highest_risk()`、`policy()`、`diagnostics()`、`snapshot()` で内容を参照できます。`UpdatePolicy::default()` は High/Critical を拒否します。`.fail_on(level)` はしきい値を変更し、`.allow(RiskCode)` は登録済みのリスク分類全体を許可します。識別情報の証拠の欠落・破損などのハードエラーは許可できません。

<!-- section: reports-and-errors -->
## レポートとエラー

`BuildReport::snapshot()` は `build::json::ReportSnapshot` を返します。これは成功・失敗やファイルの所有権を持たない観測結果です。`BuildOutput::snapshot()` と `BuildError::snapshot()` は、実際の結果とレポートを含む `build::json::BuildSnapshot` を返します。`update::json::ComparisonSnapshot` は、元の検出事項とポリシー評価を分離します。すべてのスナップショット DTO は `serde::Serialize` を実装し、ファイルを所有しません。拡張トレイトは不要です。

スナップショットのパスフィールドは、Rust ではネイティブの `PathBuf` 値を保持します。JSON では、有効な Unicode なら文字列、それ以外は損失のないネイティブ表現です。UTF-8 でない Unix 名は `{"encoding":"unix_bytes","bytes":[...]}`、対応しない UTF-16 サロゲートを含む Windows 名は `{"encoding":"windows_wide","units":[...]}` になります。成功時の成果物パスと失敗時の公開パスの両方に適用されます。元のプラットフォームで JSON のパスを復元するには、`build::json::PathSnapshot` としてデシリアライズし、`into_path_buf()` を呼び出します。別のプラットフォームのエンコード済みパスは拒否されます。エラーのネイティブパスを自分でシリアライズする際は、`PathSnapshot::new(path)` が同じ表現を提供します。

エラーは操作と同じモジュールにあります。スキーマ・バンドルのエラーは `schema`、ノート追加・画像穴埋めのエラーは `note`、メディアのエラーは `media`、ビルド・永続化のエラーは `build`、比較・ポリシーのエラーは `update` です。各エラーは `kind()` と `code()` を提供し、`std::error::Error + Send + Sync + 'static` を実装して、実際の原因エラーを保持します。

<!-- section: addition-context -->
## 追加時のコンテキスト

追加に失敗してもプロジェクトは変わりません。構造化コンテキストには元のノート・モデルのキーと対象を記録し、必要に応じて競合やメディア使用の詳細を付けます。Rust では `AddError::context()` / `detail()` を通じて `note::{AddContext, AddTarget, AddDetail}` を取得できます。SDK では `AddError.details` が型付きの context/detail を提供します。Node のフィールドは camelCase で、深く凍結されています。Python は snake_case の TypedDict を使います。タグは挿入時のインデックスを保持します。フィールドの位置は、元のシーケンスインデックス（フィールド自体は null/None、ルートは []）と任意の UTF-8 バイト範囲を保持します。ビルド時の既定デッキのエラーはノート・モデルのキーを持たず、原因チェーンから確認できます。

型付き画像を追加する際は image/* MIME が必要です。音声参照には audio/* または video/* を受け付けます。他の種類は `NOTE.MEDIA_USAGE_INVALID` になります。コンストラクター自体は失敗せず、明示的アセットと直接記述した HTML には影響しません。検査はインポート時に保持した MIME を使い、再生の可否は保証しません。ビルド時の独立した MIME・拡張子検証も引き続き適用されます。対象と詳細の完全な対応表は[設計文書](https://github.com/morehardy/anki-forge/blob/main/docs/plans/2026-09-28-rust-api-validation-and-errors-design.md)を参照してください。

<!-- section: review-once-publish-once -->
## 一度確認した候補をそのまま公開する

準備済み公開 API は、公開 `0.3.1` パッケージに含まれています。

@@CODE_0@@

準備処理では、上限付きの完全な検査と比較を実行します。`publish` までは保存先に公開しません。`publish` は同じ非公開候補を使い、基準パッケージとのパスの別名関係を再検査します。相対パスは準備呼び出し時に確定します。プロジェクトを編集しても候補は変わりません。レポートの所要時間には確認に費やした時間を含みません。未公開の所有者をドロップすると候補が削除されます。公開された一時出力は、通常どおり最後の成果物所有者の解放時に削除されます。JSON からの復元や、消費済み所有者での再試行はできません。ネイティブビルドは使用しないステージング用マニフェストを省略しますが、埋め込みの識別情報の証拠とメディア検証は保持します。内部のステージングツールには、引き続き完全なマニフェストとフィンガープリントを渡します。
