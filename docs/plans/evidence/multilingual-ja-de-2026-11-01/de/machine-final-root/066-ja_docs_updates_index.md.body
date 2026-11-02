# 安定したノート ID で Anki デッキを更新する

Canonical page: https://ankiforge.dev/ja/docs/updates/
Language: ja
Description: ノート ID を維持し、前回配布時のオリジナル APKG を基準にビルドして、比較を確認してから新バージョンを配布します。

すでに配布した Anki デッキに訂正や新しいカードを届けます。ノートの ID を安定させると、Anki Forge と Anki は既存のノートを新規ノートとして扱わずに識別できます。ただし、実際に何が更新されるかには、Anki のインポート設定や、それより新しいローカルでの編集も影響します。

プロジェクトの名前空間と、ノート・モデル・フィールド・テンプレートのキーは変更しないでください。新しいリリースは、完全なバージョン付き識別情報を含む、前回の**配布時のオリジナル APKG**を基準に作成します。そのパッケージを対応するソースのリビジョンと一緒に保管してください。

## 2つの版を生成して比較する

```sh
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
```

このコマンドは `spanish-v1.apkg` と `spanish-v2.apkg` を書き出し、変更した回答を比較して、更新版のパッケージ内の件数を確認します。実行されるソースコードは次のとおりです。

```rust
fn vocabulary(answer: &str) -> anyhow::Result<Project> {
    let mut project = Project::new("docs-spanish-updates")?.default_deck("Spanish");
    project.add("hola", Note::basic("hola", answer))?;
    Ok(project)
}

fn updates(output: &Path) -> anyhow::Result<()> {
    let previous = output.join("spanish-v1.apkg");
    let next = output.join("spanish-v2.apkg");
    vocabulary("hello")?.build(BuildOptions::to(&previous))?;
    let project = vocabulary("hello; hi")?;
    let comparison = project.compare(CompareOptions::against(&previous))?;
    ensure!(comparison.policy().allows_publication());
    let built = project.build(BuildOptions::to(&next).update_from(&previous))?;
    verify(&built, 1, 1, 0)?;
    ensure!(built.report().comparison().is_some());
    Ok(())
}
```

`compare` は、公開を妨げる検出事項があっても正常に完了します。元のリスクレベルとその根拠に加えて、`comparison.policy().allows_publication()` を別途確認してください。`build(...update_from(...))` も同じ分析を行い、ポリシーが候補の公開を認めない場合は、公開前に `BuildError` を返します。

## ポリシーと検査の上限

次の完全なプログラムでは、より厳しいしきい値を選び、比較と更新に同じ設定を使います。

```rust
use ankiforge::{BuildOptions, Note, Project};
use ankiforge::build::InspectLimits;
use ankiforge::update::{CompareOptions, RiskLevel, UpdatePolicy};

fn main() -> anyhow::Result<()> {
    let mut first = Project::new("policy-example")?;
    first.add("cell", Note::basic("Cell?", "Unit of life"))?;
    first.build(BuildOptions::to("policy-v1.apkg"))?;
    let mut next = Project::new("policy-example")?;
    next.add("cell", Note::basic("Cell?", "The basic unit of life"))?;
    let mut limits = InspectLimits::default();
    limits.max_collection_bytes = 1 << 30;
    let policy = UpdatePolicy::default().fail_on(RiskLevel::Medium);
    let comparison = next.compare(CompareOptions::against("policy-v1.apkg")
        .inspect_limits(limits.clone()).update_policy(policy.clone()))?;
    assert!(comparison.policy().allows_publication());
    next.build(BuildOptions::to("policy-v2.apkg").update_from("policy-v1.apkg")
        .inspect_limits(limits).update_policy(policy))?;
    Ok(())
}
```

デフォルトのポリシーは High と Critical の検出事項がある場合に公開を止めます。特定のカテゴリを確認したうえで、呼び出し側が意図的に `UpdatePolicy::default().allow(RiskCode::NoteRemoved)` を選ぶこともできます。この指定は、元のレベルと根拠を保持したまま、**そのカテゴリの検出事項をすべて**許容します。特定のノートだけを個別に承認する指定ではありません。該当する検出事項のない許容指定には警告が出ます。不明なリスクコードは拒否されます。比較で返されたコードをすべて自動的に許容しないでください。

識別情報の欠落や破損、名前空間の不一致、不正な候補データ、検査上限への抵触はエラーであり、許容指定できるリスクカテゴリではありません。分析を完了できない場合、比較はそれまでに得られた検査結果を含むエラーを返します。`update_from` のない新規作成に明示的なポリシーを指定すると、setter の呼び出し順にかかわらず設定エラーになります。

## 識別情報とその根拠

パッケージには `ankiforge-identity-v1` 形式の `ankiforge-identity.json` が含まれます。モデル／設定の ID、ノートの GUID、過去のフィールド／テンプレートのスロット、カード／マスクの通し番号、リビジョン、メディアを、実際のコレクションと格納データに結び付けます。Anki からの再エクスポートではこの情報が保持されないため、更新の基準には使えません。欠落した情報が暗黙に再構築されることもありません。

表示名や内容を編集しても、識別用のキーは保持されます。内容が変わらない場合は基準パッケージのリビジョンを維持し、変更した場合はリビジョンを進めます。フィールドやテンプレートを並べ替えても、過去の対応関係は保持されます。使われなくなった識別子も予約されたままなので、同じキーを復元すると再利用できます。ノートキーを別のモデルへ移すことはできず、モデルキーの種類を通常型と Cloze 型の間で変更することもできません。

## クライアントのインポート動作と公開前の確認

APKG は配布用パッケージであり、削除を同期する仕組みではありません。ノート、フィールド、テンプレート、マスクを省いても、学習者の既存データは削除されません。フィールド／テンプレートのスキーマ変更と並べ替えフィールドの変更は、Anki のマージやインポートの設定が動作に影響するため、高リスクです。スキーマ変更によって、ノート内容の比較より先にモデルが複製されたり、対象ノートのタイムスタンプが更新されたりする場合があります。

サポートするクライアントで、初回インポートと既存コレクションへの更新をテストしてください。予定しているマージ設定とノート更新設定を使い、フィールド、GUID、カードの集合、ローカルでの編集、既存の学習スケジュールを確認します。識別情報が安定しているだけでは、新しいローカル編集の上書きや、削除された履歴の復元は保証されません。

候補は、保存済みの基準パッケージとは別のパスに書き出してください。確認を終えたら承認したパッケージを配布し、次回のリリースの根拠として保管します。ファイルの所有権と公開時の保証については[ビルドの保証](https://ankiforge.dev/ja/docs/build-guarantees/)を参照してください。

## 一度だけビルドし、確認してから公開する

この API は公開パッケージ `0.3.1` に含まれています。

一度ビルドしたらレポートを確認し、その同じ候補ファイルを公開します。確認後に通常のビルドをもう一度行うと、別の候補が生成されます。準備済み公開では、実際の APKG と確認対象のレポートの対応を保てます。

```rust
use ankiforge::{BuildOptions, Note, Project};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut project = Project::new("spanish")?.default_deck("Spanish");
    project.add("es:hola", Note::basic("hola", "hello"))?;
    project.build(BuildOptions::to("spanish-original.apkg"))?;

    let mut next = Project::new("spanish")?.default_deck("Spanish");
    next.add("es:hola", Note::basic("hola", "hello; hi"))?;
    let prepared = next.prepare_publication(
        BuildOptions::to("spanish-reviewed.apkg").update_from("spanish-original.apkg"),
    )?;
    println!("{:?}", prepared.report().comparison());
    if prepared
        .report()
        .comparison()
        .is_some_and(|report| report.policy().allows_publication())
    {
        let output = prepared.publish()?;
        println!("{}", output.artifact().path().display());
    }
    Ok(())
}
```

固定したソースのチェックアウトで、上の完全なプログラムを `anki_forge/examples/docs_publication.rs` に保存します。この例は本ガイドで追加するもので、`1199196` には含まれていません。その後、次を実行します。

```sh
cargo run --locked -p ankiforge --example docs_publication
```

この例はリポジトリのルートに `spanish-original.apkg` を書き出し、ポリシーが公開を認めた場合は `spanish-reviewed.apkg` も書き出します。準備の段階では公開先は変更されません。配布を承認する前に検出事項を確認し、クライアントへのインポートをテストしてください。この例が自動的に公開するのは、ポリシーが許容する単純な内容変更だけです。ベンチマークはこのワークフローを測定していません。

[ビルドのライフサイクル](https://ankiforge.dev/ja/docs/build-guarantees/#prepared-publication-from-source)、[Rust API](https://ankiforge.dev/ja/docs/rust-api/#review-once-publish-once)、[Node API](https://ankiforge.dev/ja/docs/node-api/#prepare-review-and-publish)、[Python API](https://ankiforge.dev/ja/docs/python-api/#prepare-review-and-publish)も参照してください。
