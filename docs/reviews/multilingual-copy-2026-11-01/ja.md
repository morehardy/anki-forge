# 日语文案审阅

首页简洁，使用「同じノートとして識別」解释 identity 的处理值得保留。正文的「準備済み公開」「任意の成果物」等名词组合比较机械；有些句子省略了动作对象或把英文修饰关系直接搬了过来。

保留 ノート／カード／デッキ、画像穴埋め 和「手元のデータから」。Basic、Cloze、Rust 标识符和示例稳定键不应强行改成日语。

P2 表示应优先处理的事实范围或理解歧义；P3 表示可选的自然度、清晰度和术语优化。编号按位置统计，同一原因可能影响多处文字。本页建议已应用到产品源文件，并补齐相关术语的一致性。表中原文和行号保留审阅时快照，便于追溯；不作为正式发布的独立审阅凭据。

## 界面文案

| 编号与位置 | 原文 | 建议替换 | 问题 |
| --- | --- | --- | --- |
| ja-01 P2<br>[ai.language](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/ja.json:144) | アプリケーションの言語 | プログラミング言語 | アプリケーションの言語 容易指应用显示语言，选项实际是编程语言。 |
| ja-02 P3<br>[ai.description](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/ja.json:137) | 言語を選び、作りたいものを記入して、依頼文を AI コーディングアシスタントに貼り付けます。 | プログラミング言語を選び、作りたいものを記入して、依頼文を AI コーディングアシスタントに貼り付けます。 | 与语言选项卡使用同一术语。 |
| ja-03 P3<br>[ai.copy](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/ja.json:146) | AI 用プロンプトをコピー | AI への依頼文をコピー | 标题叫依頼文，按钮却叫プロンプト；建议统一到较易懂的依頼文。 |
| ja-04 P3<br>[ai.copied](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/ja.json:147) | プロンプトをコピーしました。AI コーディングアシスタントに貼り付けてください。 | 依頼文をコピーしました。AI コーディングアシスタントに貼り付けてください。 | 保持复制前后术语一致。 |
| ja-05 P3<br>[ai.fallback](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/ja.json:148) | プロンプトを選択しました。キーボードでコピーし、AI コーディングアシスタントに貼り付けてください。 | 依頼文を選択しました。キーボードでコピーし、AI コーディングアシスタントに貼り付けてください。 | 保留恢复操作，统一称谓。 |
| ja-06 P3<br>[home.evolveDescription](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/ja.json:20) | 定義を訂正したり例を追加したりできます。安定した ID と以前のビルドを使い、Anki Forge は再エクスポート時に既存のノートを照合します。 | 定義の訂正や例の追加ができます。ノート ID を維持し、前回配布したオリジナルの APKG を使うことで、再エクスポート時に既存のノートを照合できます。 | 以前のビルド 是过程，不是明确的基准文件；改为前次分发的原始包。 |
| ja-07 P3<br>[home.validateDescription](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/ja.json:24) | 未入力のフィールド、メディア、テンプレートの問題を診断結果で確認します。検証をビルドの手順に組み込めます。 | 必須フィールドの未入力や、メディア・テンプレートの問題を診断結果で確認できます。検証をビルド手順に組み込めます。 | 并非所有空字段都违规；补上必須，贴近校验规则。 |
| ja-08 P3<br>[workbench.source](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/ja.json:49) | 完全なソース ↗ | ソースコード全体 ↗ | 完全なソース 是 full source 的直译；日本技术界面通常直接说源码全体。 |
| ja-09 P3<br>[examples.fullSource](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/ja.json:124) | 完全なソースを読む | ソースコード全体を読む | 与示例工作台统一。 |
| ja-10 P3<br>[examples.intro](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/ja.json:122) | ソースを読み、カードを裏返し、生成されたデッキを Anki で開いてみてください。ウェブのプレビューは、ダウンロードと同じフィールドを使っています。 | ソースコードを読み、カードを裏返して、生成されたデッキを Anki で開いてみてください。ウェブのプレビューとダウンロードできるデッキには、同じフィールドの内容を使っています。 | ダウンロード 指下载动作，不能作为“相同字段”的另一方；应说可下载的牌组。 |
| ja-11 P3<br>[examples.combined](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/ja.json:123) | 4枚のカードのデッキをダウンロード | 4枚入りのデッキをダウンロード | 4枚のカードのデッキ 连续の，简化更自然。 |
| ja-12 P3<br>[update.none](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/ja.json:83) | 検出事項なし | 変更は検出されませんでした | 検出事項なし 是审计式名词，状态消息用完整自然句。 |
| ja-13 P3<br>[performance.description](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/ja.json:172) | 各処理には Basic ノートが{notes}件あり、テキストのみ、個別のメディア、共有する複数種類のメディアを扱います。Rust は genanki よりエクスポート時間が{minimum}〜{maximum}%短くなりました。 | 各テストでは Basic ノートを{notes}件使います。5種類のテストで、テキストのみ、ノートごとに異なるメディア、ノート間で共有するメディアを比較しました。Anki Forge の Rust API は genanki よりエクスポート時間が{minimum}〜{maximum}%短くなりました。 | 把“各处理”改成测试场景，明确个别媒体与共享媒体，以及比较的是具体 API。 |
| ja-14 P3<br>[performance.basic-image-unique-v2](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/ja.json:177) | 個別の画像 | ノートごとに異なる画像 | 個別の画像 没说明个别的单位；明确是每条笔记不同。 |
| ja-15 P3<br>[performance.basic-audio-unique-v2](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/ja.json:178) | 個別の音声 | ノートごとに異なる音声 | 与图片标签使用同一关系。 |

## 正文与元数据

### ja-16 P3

[docs/ja/build-guarantees.md](/Users/hp/Desktop/2026/anki-forge/docs/ja/build-guarantees.md:46)

準備済み公開 是机械组合，像“公开已准备好”；本节实际是分两步生成和写入同一候选包。

原文片段：

> ## 準備済み公開

建议替换：

> ## 候補ファイルを準備し、確認後に保存する

### ja-17 P3

[docs/ja/updates.md](/Users/hp/Desktop/2026/anki-forge/docs/ja/updates.md:94)

用具体动作解释 API，避免继续扩散未自然成立的名词。

原文片段：

> 準備済み公開では、実際の APKG と確認対象のレポートの対応を保てます。

建议替换：

> `prepare_publication` で候補を準備し、確認後に `publish` で保存すると、レポートで確認したものと同じ APKG を書き出せます。

### ja-18 P2

[docs/ja/python-migration.md](/Users/hp/Desktop/2026/anki-forge/docs/ja/python-migration.md:17)

optional 在此是“可能不存在”，任意 在日语中常读成“任意选择的”，会误解旧返回值语义。

原文片段：

> 任意の成果物と ensure_success

建议替换：

> 成果物がない場合もある戻り値と ensure_success

### ja-19 P3

[docs/ja/media.md](/Users/hp/Desktop/2026/anki-forge/docs/ja/media.md:3)

把“语音合成”明确挂到音频，避免修饰音声・画像整体。

原文片段：

> 音声合成などによる音声・画像の作成には別のツールが必要です。

建议替换：

> 音声や画像の作成には別のツールが必要です。たとえば、読み上げ音声は音声合成ツールで用意します。

### ja-20 P2

[docs/ja/node-api.md](/Users/hp/Desktop/2026/anki-forge/docs/ja/node-api.md:3)

Project のみ 容易被读成只导出 Project；实际入口也导出 Note、Media 等。

原文片段：

> `ankiforge` エントリーポイントは、作成用モデルとして Project のみを公開します。

建议替换：

> `ankiforge` の公開 API では、`Project` を中心にノートやメディアを組み立てます。

### ja-21 P2

[website/src/content/blog/ja/stable-ids-and-deck-updates.mdx](/Users/hp/Desktop/2026/anki-forge/website/src/content/blog/ja/stable-ids-and-deck-updates.mdx:53)

not a supported baseline 应直接说不能用于更新基准，而不是“不对应的包”。

原文片段：

> Anki から再エクスポートしたファイルは、対応する基準パッケージではありません。

建议替换：

> Anki から再エクスポートしたファイルは、更新の基準パッケージとして使用できません。

### ja-22 P3

[docs/ja/json-to-anki.md](/Users/hp/Desktop/2026/anki-forge/docs/ja/json-to-anki.md:3)

配布元 是分发来源；原意是首次原始包。回答/解答在学习题场景统一为解答。

原文片段：

> 配布元の APKG と、回答だけを変更して確認済みの更新版

建议替换：

> 初回配布用の APKG と、解答だけを変更した検証済みの更新版

### ja-23 P3

[website/site-model.mjs](/Users/hp/Desktop/2026/anki-forge/website/site-model.mjs:90) · `page:installation.title.ja`

本页只讲 Rust，避免语言指南入口把所有 SDK 都引到这里。

原文片段：

> アプリケーションに組み込む

建议替换：

> Rust アプリケーションに組み込む

### ja-24 P3

[website/site-model.mjs](/Users/hp/Desktop/2026/anki-forge/website/site-model.mjs:28) · `page:quickstart.description.ja`

入门摘要中的“永続 APKG”偏内部类型术语；说明保存后可继续使用即可。

原文片段：

> Rust の公開パッケージをインストールし、スペイン語のカードを作成して、永続 APKG ファイルを書き出します。

建议替换：

> Rust の公開パッケージをインストールし、スペイン語のカードを作成して、後から使える APKG ファイルとして保存します。
