# 英语文案审阅

整体准确、可读，可以保留主要表达。优先清理已过时的源码限定，再把 identity、publication、media closure 等内部术语写得具体。英文源文的问题会被其他语言继承。

保留 “Notes on making notes.”：它在英语里成立，不必为了跨语言对齐而删掉双关。更新说明中关于 Anki 导入设置和本地编辑的限制也应保留。

P2 表示应优先处理的事实范围或理解歧义；P3 表示可选的自然度、清晰度和术语优化。编号按位置统计，同一原因可能影响多处文字。本页建议已应用到产品源文件，并补齐相关术语的一致性。表中原文和行号保留审阅时快照，便于追溯；不作为正式发布的独立审阅凭据。

## 界面文案

| 编号与位置 | 原文 | 建议替换 | 问题 |
| --- | --- | --- | --- |
| en-01 P3<br>[home.choose](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/en.json:9) | Choose your language. | Choose your programming language. | language 在多语言网站上也可指界面语言；这里选择的是 Rust、Python 或 Node.js。 |
| en-02 P3<br>[ai.language](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/en.json:144) | Application language | Programming language | 这是语言选项卡的无障碍标签；Application language 容易让人想到界面语言。 |
| en-03 P3<br>[ai.description](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/en.json:137) | Choose a language, describe your task, then paste the brief into your AI coding assistant. | Choose a programming language, describe your task, then paste the prompt into your AI coding assistant. | 说明选择对象，并统一 brief 与 prompt，减少同一内容的称谓切换。 |
| en-04 P3<br>[ai.skillQuestion](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/en.json:141) | Your coding assistant supports Skills? | Does your coding assistant support Skills? | 原句是陈述句倒装缺失的口语问句；常规产品界面用完整疑问句更自然。 |
| en-05 P3<br>[ai.request](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/en.json:138) | What should the deck do? | What kind of deck would you like to create? | 牌组是生成结果，What should the deck do 更像是在询问软件行为。 |
| en-06 P3<br>[home.install](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/en.json:10) | Export spanish.apkg with the public {version} package. Normal export needs no Anki installation. | Export spanish.apkg with Anki Forge {version}. You do not need to install Anki to export a deck. | public {version} package 名词顺序生硬；normal export 也没有在此定义。 |
| en-07 P3<br>[home.evolve2](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/en.json:19) | Keep their identity. | Keep the same note IDs. | 首屏 identity 较抽象；直接指出保持不变的是笔记 ID。 |
| en-08 P3<br>[home.evolveDescription](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/en.json:20) | Correct a definition or add an example. Stable IDs and a previous build help Anki Forge match existing notes when you export again. | Correct a definition or add an example. Keep stable note IDs and use the original APKG from the previous release so Anki Forge can match existing notes when you export again. | previous build 没说明要保存哪个文件；补齐原始 APKG 的含义，保持现有导入限制。 |
| en-09 P3<br>[update.comparison](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/en.json:82) | Actual comparison | Comparison results | Actual comparison 没交代“实际”相对于什么；结果标签更直接。 |
| en-10 P3<br>[update.none](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/en.json:83) | No findings | No changes detected | 该字段仅在比较 findings 为空时显示；用比较上下文的表达代替抽象 findings。 |
| en-11 P3<br>[performance.title](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/en.json:165) | The export, measured. | Export performance | 原文省略句刻意，作为性能模块标题不如名词短语清楚。 |
| en-12 P3<br>[performance.description](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/en.json:172) | Each workload contains {notes} Basic notes, covering text-only, unique media and shared mixed media. Rust used {minimum}–{maximum}% less export time than genanki. | Each test uses {notes} Basic notes. The five tests cover text-only notes, separate media files and shared media. In these tests, the Anki Forge Rust API took {minimum}–{maximum}% less time to export than genanki. | 明确五种场景是整组测试的覆盖范围；比较对象是 Anki Forge 的 Rust API，不是 Rust 语言本身。 |
| en-13 P3<br>[machine.constraints](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/en.json:195) | Normal export does not require Anki. Published package APIs differ from source builds. Keep the original previous distribution APKG for updates; Anki import settings and newer local edits can affect the result. | Exporting a deck does not require Anki. The APIs available in published packages differ from those in source builds. For updates, keep the original APKG from the previous release; Anki import settings and newer local edits can affect the result. | API 与 source builds 原句比较层级不同，original previous distribution 名词堆叠。 |

## 正文与元数据

### en-14 P2

[docs/build-guarantees.md](/Users/hp/Desktop/2026/anki-forge/docs/build-guarantees.md:82)

标题暗示需源码构建，紧接着正文却说明已包含在公开 0.3.1 包；保留旧锚点，只改可见标题。

原文片段：

> ## Prepared publication from source

建议替换：

> ## Prepare, review and publish

### en-15 P2

[docs/python/api.md](/Users/hp/Desktop/2026/anki-forge/docs/python/api.md:82)

本节讲公开包中的批量媒体和预备发布；Source 暗示仅源码可用。

原文片段：

> ## Source API details

建议替换：

> ## Batch media and prepared publication

### en-16 P3

[docs/updates.md](/Users/hp/Desktop/2026/anki-forge/docs/updates.md:1)

publication 既指动作又指产物，标题绕口；用户任务是更新牌组。 此项是仓库源文标题；生成的网站 H1 由 site-model.mjs 提供，不应把它报成网页标题错误。

原文片段：

> # Update and distribute a publication

建议替换：

> # Update and distribute a deck

### en-17 P3

[docs/template-bundles.md](/Users/hp/Desktop/2026/anki-forge/docs/template-bundles.md:9)

media closure 是内部概念；说明包含哪些依赖即可。

原文片段：

> A bundle is a reusable model declaration, template text, CSS and explicit media
> closure.

建议替换：

> A bundle contains a reusable note type definition, template text, CSS and all explicitly declared media dependencies.

### en-18 P3

[docs/cards.md](/Users/hp/Desktop/2026/anki-forge/docs/cards.md:4)

笔记存储字段，卡片才有显示的正反面；给初学者补清二者关系。

原文片段：

> A Basic note has a front
> and back.

建议替换：

> A Basic note stores front and back fields, which its template uses to generate a question-and-answer card.

### en-19 P3

[website/site-model.mjs](/Users/hp/Desktop/2026/anki-forge/website/site-model.mjs:90) · `page:installation.title.en`

网页标题由注册表生成；正文实际只讲 Rust，标题应让 Python/Node 用户在点入前辨认。

原文片段：

> Add to your application

建议替换：

> Add Anki Forge to a Rust application
