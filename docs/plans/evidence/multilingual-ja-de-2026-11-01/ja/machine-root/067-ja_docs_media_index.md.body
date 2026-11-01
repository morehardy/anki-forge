# Anki カードに画像と音声を追加する

Canonical page: https://ankiforge.dev/ja/docs/media/
Language: ja
Description: Rust、Python、Node.js で画像・音声をインポートしてカードを組み立て、エクスポート名、MIME タイプ、メディア上限を管理します。

発音音声付きの語彙カードや図を使った問題など、生成する Anki カードに画像と音声を同梱します。Anki Forge は、指定されたファイルを APKG にコピーします。音声合成などによる音声・画像の作成には別のツールが必要です。例では Rust を使います。後半の一括インポートでは Python と Node.js の呼び出しも示します。

`Media` の値は不変のスナップショットを所有します。ファイルは `Media::file`、ネイティブのバイト列は MIME タイプを明示した `Media::bytes` でインポートします。インポート成功後に元ファイルを変更・削除してもアセットは変わりません。クローンはストレージを共有し、複数のプロジェクトで使えます。

## メディアの作成手順を実行する

リポジトリのルートから実行します。

```sh
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
cargo run --locked -p ankiforge --example target_api_media
```

この手順はリポジトリの PNG・WAV フィクスチャーを使い、ノート1件、カード1枚、アセット3個を含む `media.apkg` を書き出します。2つ目の例は小さな画像と音声を自ら生成し、CSS から参照するメディアと明示的なメディアを含めた `spanish-media.apkg` を書き出します。

```rust
fn media(output: &Path, image: &Path, sound: &Path) -> anyhow::Result<()> {
    let image = Media::file(image)?.with_export_name("cell.png")?;
    let sound = Media::file(sound)?;
    let mut project = Project::new("docs-media")?;
    project.add("cell", Note::basic(image.image(), sound.sound()))?;
    let css = Media::bytes(b".card { color: navy; }".to_vec(), "text/css")?
        .with_export_name("course.css")?;
    project.add_asset(css)?;
    let built = project.build(BuildOptions::to(output.join("media.apkg")))?;
    verify(&built, 1, 1, 3)?;
    Ok(())
}
```

## コンテンツと明示的なアセットを組み合わせる

次の完全なプログラムは `fixtures/pixel.png`、`fixtures/silence.wav`、`fixtures/labels.woff` を使います。自分のアプリケーションでは使用するファイルのパスを渡してください。フィクスチャーのフォントは、リポジトリの再配布可能なテスト用フォントです。

```rust
use ankiforge::{BuildOptions, Content, Field, Media, NoteType, Project, Template};

fn main() -> anyhow::Result<()> {
    let image = Media::file("fixtures/pixel.png")?.with_export_name("badge.png")?;
    let sound = Media::file("fixtures/silence.wav")?;
    let font = Media::file("fixtures/labels.woff")?.with_export_name("labels.woff")?;
    let model = NoteType::builder("media-card")
        .field(Field::new("front"))
        .field(Field::new("back"))
        .template(Template::new("card")
            .front(r#"{{front}}<img src="badge.png">"#)
            .back("{{FrontSide}}<hr>{{back}}"))
        .css("@font-face { font-family: labels; src: url('labels.woff'); } .card { font-family: labels; }")
        .asset(font)
        .asset(image.clone())
        .build()?;
    let mut project = Project::new("media-guide")?;
    project.add("cell", model.note()
        .field("front", Content::sequence([Content::text("Cell "), image.image()]))
        .field("back", sound.sound()))?;
    let output = project.build(BuildOptions::to("media-guide.apkg"))?;
    assert_eq!(output.report().counts().media, 3);
    Ok(())
}
```

型付きの `image()` と `sound()` の値は、エクスポートまで依存関係を保持します。手書きの HTML・CSS・スクリプトで参照する場合は、`builder.asset(media)` または `project.add_asset(media)` が必要です。ライブラリは任意の文字列からファイルの内容を推測しません。明示的なアセットは、静的に検出可能な参照がなくても含まれます。

`with_export_name` には実際のファイル名をそのまま渡してください。型付き音声参照は描画時に HTML エンティティをエスケープするため、`tone&copy;.mp3` のような名前も、その名前のアセットを正確に参照します。ファイル名を事前にエスケープしないでください。音声ファイル名のパーセント記号やフラグメント記号は文字どおり扱い、URL パスとは解釈しません。

## 名前、MIME、上限

既定のエクスポート名は内容から導かれ、元のパスには依存しません。固定のファイル名が必要なら、参照を作成する前に `with_export_name` を使います。クローンの名前を変えても、それ以前のクローンやコンテンツノードの名前は変わりません。

名前にパス、制御文字、移植性のない予約名を含めることはできません。競合検査は Unicode NFC と大文字・小文字の正規化を使います。そのため、大文字・小文字または正規化形式だけが異なる名前も競合します。名前と内容が完全に一致する組み合わせの登録は冪等です。同じ名前で異なる内容を登録すると、部分的に登録せず失敗します。

拡張子を含めたエクスポート名全体の上限は UTF-8 で255バイトです。プロジェクトとテンプレートバンドルの JSON Schema は、ファイル名の構文と文字数を検査します。ローダーはバイト数も検査します。たとえば `é` を128回繰り返すとスキーマの文字数上限は通りますが、256バイトになるため、読み込み時に `MEDIA.EXPORT_NAME_INVALID` を返します。ビルド前に `tools::load_project` または `NoteType::from_bundle` で入力全体を検証してください。 `tools::load_project` は `internal-tools` feature が必要なリポジトリ内部向けの API であり、利用側アプリケーション向けのサポート対象 API には含まれません。

MIME の構文と、判別可能な内容は一致する必要があります。ネイティブのバイト列には64 KiBのインライン上限はありません。インポートの既定上限はアセットごとに256 MiBです。`file_with_limits` と `bytes_with_limits` は、読み取り前に `MediaLimits { max_bytes }` を受け取ります。大きなスナップショットは一時ストレージに退避される場合があり、最後の所有者が解放されると削除されます。スナップショットを所有していても、インポート中に同時変更される元ファイルについて、ファイルシステムのアトミックな読み取り像を保証するものではありません。

プロジェクト JSON では、インラインの `bytes` 配列をデコードするときにも同じ既定のアセット単位の上限を適用します。最初の超過バイトで停止し、配列の残りは収集しません。エラーには上限と観測バイト数を持つ `MediaError` が保持されます。拒否時のバイト数は下限であり、未読部分を含む配列全体のサイズではありません。

有効なメディアパッケージであっても、すべての Anki クライアントでコーデックが再生できるとは限りません。対象クライアントで画像、音声、フォントをテストしてください。[トラブルシューティング](https://ankiforge.dev/ja/docs/troubleshooting/#media-and-templates)を参照してください。

### 型付きの画像・音声としての使用

コンストラクター自体は引き続き失敗しません。`Project::add` はインポート時に保持した MIME を検査します。画像には `image/*`、音声参照には `audio/*` と `video/*` を受け付けます。`application/octet-stream`、テキスト、フォント、PDF の型付き使用は `NOTE.MEDIA_USAGE_INVALID` になります。直接記述した HTML・CSS 用の明示的アセットとして登録してください。正しい形式で明示した未知の画像・動画 MIME は種類の検査を通りますが、デコーダーやプレーヤーがそのバイト列に対応することを保証するものではありません。改名によって保持済みの MIME が変わることはありません。ビルドでは配置済みのバイト列を判別し、エクスポート拡張子を独立して検証します。`wrong.mp3` という名前の PNG は画像としての追加に成功しても、ビルドで `MEDIA.DECLARED_MIME_MISMATCH` になることがあります。

## メディアの一括インポート

これらのメソッドは公開 `0.3.1` パッケージに含まれています。

同時実行数を制限して複数のファイルをインポートし、入力順を保ちながら、後のエクスポートに使うメディアスナップショットを所有します。

```rust
use ankiforge::Media;
let images = Media::files(["first.png", "second.png"])?;
```

Node では `await Media.files(['first.png', 'second.png'])`、Python では `Media.files(['first.png', 'second.png'])` を使います。

### 一括処理の動作と上限

Rust では `Media::files(paths)`、Node では `await Media.files(paths)`、Python では `Media.files(paths)` を使います。呼び出しごとに最大4ワーカーで、入力順にファイルをインポートします。16ファイル未満のバッチは逐次実行します。Rust には `files_with_limits` もあります。Node と Python は `file` と同じ上限指定を受け付けます。上限はアセットごとに適用します。インポートに失敗すると、開始済みの処理を待ってからバッチのスナップショットを破棄し、入力順で最初のエラーを返します。既存のメディア所有者は引き続き有効です。Python はインポート中に GIL を解放し、Node はイベントループの外で処理します。単一ファイルのインポートは、従来どおりその場でデータを所有するスナップショットを取得します。同時実行を使うには、一括メソッドを明示的に選んでください。

生存中のスナップショットは、プロセス全体で **64 MiB** のメモリ枠を共有します。1 MiBを超える個々のアセットは、引き続き一時ファイルへストリーミングします。エンコード済みメディアは、準備処理ごとに遅延確保する **16 MiB** のバッファープールを共有し、いっぱいになると一時ストレージへ退避します。これらの上限は他の作業用メモリを含まず、RSS の上限ではありません。同時ビルドはそれぞれ独自のエンコード用プールを所有できます。
