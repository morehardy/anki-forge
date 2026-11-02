# 画像穴埋め

プログラムから Anki の画像穴埋めカードを生成します。画像と矩形の座標を渡すと、Anki Forge はマスクごとに1枚のカードを持つ、インポート可能なノートを作ります。Anki のエディターでマスクを描かずに、図の問題をまとめて作成できます。画像内のラベルを自動検出する機能はありません。座標は呼び出し側で指定します。

データを所有する画像と矩形から、構造化された画像穴埋めノートを作成します。各マスクには、後続バージョンでも同じ領域を識別し続ける安定キーが必要です。ノートは通常の `add(key, note)` でプロジェクトに追加します。例では Rust を使います。Python と Node.js にも同じビルダーがあり、[Python API](/docs/python-api/#media-and-occlusion) と [Node API](/docs/node-api/#media-and-image-occlusion) で確認できます。

<!-- section: run-the-example -->
## サンプルを実行する

```sh
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
```

`target/docs-examples/diagram.apkg` を Anki で開きます。完全な実行プログラムは、リポジトリの画像穴埋め用画像を読み込み、ノート1件、カード1枚、アセット1個であることを確認します。

<!-- source: anki_forge/examples/docs_workflow.rs#occlusion -->
```rust
fn occlusion(output: &Path, diagram: &Path) -> anyhow::Result<()> {
    let image = Media::file(diagram)?;
    let note = Note::image_occlusion(image)
        .mask(Mask::rect("region-1", 10, 10, 40, 25))
        .build()?;
    let mut project = Project::new("docs-diagram")?.default_deck("Diagram");
    project.add("diagram", note)?;
    let built = project.build(BuildOptions::to(output.join("diagram.apkg")))?;
    verify(&built, 1, 1, 1)?;
    Ok(())
}
```
<!-- /source -->

<!-- section: multiple-masks-and-modes -->
## 複数のマスクとモード

この単独実行できる例では、デコード後の画像が80×80ピクセル以上である `fixtures/occlusion.png` を用意します。

```rust
use ankiforge::{BuildOptions, Media, Note, Project};
use ankiforge::note::{Mask, OcclusionMode};

fn main() -> anyhow::Result<()> {
    let note = Note::image_occlusion(Media::file("fixtures/occlusion.png")?)
        .mask(Mask::rect("nucleus", 10, 10, 20, 20))
        .mask(Mask::rect("wall", 50, 50, 20, 20))
        .mode(OcclusionMode::HideOneGuessOne)
        .build()?
        .field("header", "Cell structure")
        .field("back_extra", "Review the diagram.");
    let mut project = Project::new("cell-diagram")?;
    project.add("cell", note)?;
    let output = project.build(BuildOptions::to("cell-diagram.apkg"))?;
    assert_eq!(output.report().counts().cards, 2);
    Ok(())
}
```

既定のモードは `HideAllGuessOne` で、`HideOneGuessOne` も使えます。キー付きの各マスクは、それぞれの穴埋めカードを生成します。座標は EXIF の向きを適用した表示画像上の有限なピクセル値です。ライブラリは PNG、JPEG、GIF、WebP、BMP を完全にデコードします。デコード済みバッファーの上限は256 MiBで、圧縮済みメディアのインポート上限とは独立しています。矩形の幅と高さは正で、画像内に収まり、空でない一意のキーを使う必要があります。

ビルダーは `Note` を返す前に画像とマスクを検証します。`header`、`back_extra`、`comments` は通常の `field` で設定します。自動生成される `image` と `occlusion` のフィールドは予約済みで、上書きできません。

前回の配布物から更新する場合、マスクキーは順番が変わってもカードの序数を保持します。削除したマスクの序数は予約され、同じキーを復元すると同じ序数を再使用します。新しいキーが別のマスクの識別情報を引き継ぐことはありません。割り当て可能な範囲は1〜500で、使い切ると明示的に失敗します。マスクの削除・追加は[更新リスクの解析](/docs/updates/)で確認できます。APKG から除外しても、学習者の既存カードは削除されません。
