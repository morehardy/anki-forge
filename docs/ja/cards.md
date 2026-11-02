# Basic と Cloze のカード

Anki で1枚ずつ追加する代わりに、コードから Basic（質問と解答）と Cloze（穴埋め）のカードを生成します。Basic ノートは表面と裏面を持ちます。Cloze ノートは、サポートされる穴埋め番号ごとに1枚のカードを生成します。この Rust の例では、2件のノートから3枚のカードを作成します。Python と Node.js にも同じ `Note.basic` と `Note.cloze` のコンストラクターがあります。[Python API](/docs/python-api/#project-and-note-values) と [Node API](/docs/node-api/#project-and-notes) を参照してください。

リポジトリのルートから実行します。

```sh
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
```

`target/docs-examples/cards.apkg` を Anki で開きます。Basic の質問は **hola** です。Cloze のカードは **Madrid** と **Spain** をそれぞれ隠します。

<!-- section: author-the-notes -->
## ノートを作成する

次の関数は[完全な実行プログラム](https://github.com/morehardy/anki-forge/blob/main/anki_forge/examples/docs_workflow.rs)から抜き出したものです。`verify` ヘルパーは、成功した出力のノート・カード・メディア数を確認します。

<!-- source: anki_forge/examples/docs_workflow.rs#cards -->
```rust
fn cards(output: &Path) -> anyhow::Result<()> {
    let mut project = Project::new("docs-spanish")?.default_deck("Spanish");
    project.add("hola", Note::basic("hola", "hello"))?;
    project.add(
        "capital",
        Note::cloze("{{c1::Madrid}} is in {{c2::Spain}}.")
            .field("back_extra", "A city and its country."),
    )?;
    let built = project.build(BuildOptions::to(output.join("cards.apkg")))?;
    verify(&built, 2, 3, 0)?;
    Ok(())
}
```
<!-- /source -->

`Project::new` の第1引数は安定した名前空間です。各 `add` 呼び出しには、ソースレコードの安定キーが必要です。同じキーを別のノートに再使用すると失敗します。内容から自動でキーを割り当てることはありません。同じキーのまま解答を変えた場合、前回の配布物を基準に更新をビルドすると、同じ論理ノートとして扱われます。

<!-- section: text-fields-and-card-identity -->
## テキスト、フィールド、カードの識別情報

Basic と Cloze の文字列はどちらも Text です。意図的な HTML には `Content::html` を使います。Cloze の穴埋め構文は Text 内でも認識されます。`back_extra` は組み込み Cloze の追加フィールドキーです。テキスト、HTML、画像、音声のいずれも、同じ `Note::field` メソッドで設定します。

`{{c1::answer}}` または `{{c1::answer::hint}}` を使います。通常の文だけでは Cloze の問題は生成されません。対応する穴埋め番号は1〜500です。番号を変えるとカードの識別情報が変わる場合があります。このような変更は[比較と更新ポリシー](/docs/updates/)で確認してください。

次に、[メディア](/docs/media/)、[独自のモデル](/docs/custom-notetypes/)、[画像穴埋め](/docs/image-occlusion/)を追加できます。
