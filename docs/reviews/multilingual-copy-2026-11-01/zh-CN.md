# 简体中文文案审阅

首页已有适合中文的改写；技术正文的直译感明显更重。“身份证据”“拥有所有权的快照”“消费者应用”等表达增加理解成本，部分源码限定也已过时。建议围绕“谁持有数据、保持什么不变、文件什么时候写入”重新组织句子。

保留“从你的数据批量生成 Anki 卡片”“Anki 牌组制作与开发实践”等已经改写过的表达。保留笔记／卡片／牌组的区分。技术语境中的“所有权”本身不是禁用词，应解释持有者和生命周期。

P2 表示应优先处理的事实范围或理解歧义；P3 表示可选的自然度、清晰度和术语优化。编号按位置统计，同一原因可能影响多处文字。本页建议已应用到产品源文件，并补齐相关术语的一致性。表中原文和行号保留审阅时快照，便于追溯；不作为正式发布的独立审阅凭据。

## 界面文案

| 编号与位置 | 原文 | 建议替换 | 问题 |
| --- | --- | --- | --- |
| zh-CN-01 P3<br>[home.evolve2](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/zh-CN.json:19) | 保留原有身份。 | 保留原有笔记 ID。 | “保留身份”是 identity 的直译，首页应说清保持什么不变。 |
| zh-CN-02 P3<br>[home.evolveDescription](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/zh-CN.json:20) | 修正定义或补充示例。稳定 ID 与上一次构建的产物，帮助 Anki Forge 在再次导出时匹配已有笔记。 | 修正定义或补充示例时，保留稳定的笔记 ID，并使用上一版分发时的原始 APKG，让 Anki Forge 在再次导出时识别原有笔记。 | “上一次构建的产物”过泛，应点明更新所需的原始包。 |
| zh-CN-03 P3<br>[home.install](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/zh-CN.json:10) | 使用公开版本 {version} 导出 spanish.apkg。普通导出不需要安装 Anki。 | 使用 Anki Forge {version} 导出 spanish.apkg。导出牌组无需安装 Anki。 | “公开版本”“普通导出”偏内部术语。 |
| zh-CN-04 P3<br>[home.try](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/zh-CN.json:12) | 阅读代码，试用卡片。 | 查看代码，预览卡片。 | “试用卡片”不符合此处翻转预览的操作。 |
| zh-CN-05 P3<br>[update.comparison](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/zh-CN.json:82) | 实际比较 | 比较结果 | “实际比较”不自然，标签后面展示的是结果。 |
| zh-CN-06 P3<br>[update.none](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/zh-CN.json:83) | 无发现项 | 未检测到变化 | “无发现项”是 findings 的机械翻译；此处只在 findings 为空时显示。 |
| zh-CN-07 P3<br>[examples.updateDescription](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/zh-CN.json:129) | 第二次构建把答案从“hello”改为“hello; hi”，与第一个包比较并检查笔记身份是否保留。 | 第二次构建将答案从“hello”改为“hello; hi”。程序会与第一个包比较，确认笔记 ID 保持不变。 | 把“检查笔记身份是否保留”改成读者能直接理解的动作。 |
| zh-CN-08 P3<br>[examples.limitations](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/zh-CN.json:131) | Anki 的导入设置和本地编辑会影响更新方式。预览展示内容与身份；实际卡片样式和复习行为由 Anki 决定。 | Anki 的导入设置和本地编辑会影响更新结果。网页预览仅展示内容和笔记标识；实际卡片样式及复习方式由 Anki 决定。 | “内容与身份”“复习行为”过于抽象。 |
| zh-CN-09 P3<br>[ai.brief](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/zh-CN.json:145) | AI 提示 | 给 AI 的任务说明 | prompt 在这里包含任务、安装和约束；“AI 提示”不够明确。 |
| zh-CN-10 P3<br>[ai.copy](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/zh-CN.json:146) | 复制 AI 提示 | 复制给 AI 的任务说明 | 与展示标题使用同一称谓。 |
| zh-CN-11 P3<br>[ai.description](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/zh-CN.json:137) | 选择开发语言、描述任务，再把提示复制给 AI 编码助手。 | 选择开发语言，描述任务，再把任务说明粘贴给 AI 编程助手。 | 统一“任务说明”和“编程助手”；编码助手在中文中较生硬。 |
| zh-CN-12 P3<br>[ai.copied](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/zh-CN.json:147) | 提示已复制，可以粘贴给 AI 编码助手。 | 任务说明已复制，请粘贴给 AI 编程助手。 | 与按钮及标题保持同一术语。 |
| zh-CN-13 P3<br>[ai.fallback](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/zh-CN.json:148) | 提示已选中，请用键盘复制，再粘贴给 AI 编码助手。 | 任务说明已选中，请用键盘复制，再粘贴给 AI 编程助手。 | 保留失败后的恢复操作，统一术语。 |
| zh-CN-14 P3<br>[ai.skillQuestion](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/zh-CN.json:141) | 编码助手支持 Skill？ | 你的 AI 编程助手支持 Skill 吗？ | 问句补完整，“编码助手”改为更常见的“编程助手”。 |
| zh-CN-15 P3<br>[performance.description](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/zh-CN.json:172) | 每组 {notes} 条 Basic 笔记，覆盖纯文本、不重复媒体和共享混合媒体。Rust 导出耗时比 genanki 低 {minimum}–{maximum}%。 | 每组测试包含 {notes} 条 Basic 笔记。五组测试覆盖纯文本、独立媒体和共享媒体等场景。Anki Forge Rust API 的导出耗时比 genanki 低 {minimum}–{maximum}%。 | 避免把五种场景写成每一组都包含的内容，并明确测的是库的 Rust API。 |
| zh-CN-16 P3<br>[performance.basic-mixed-unique-v2](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/zh-CN.json:179) | 不重复混合媒体 | 图片与音频，不共享文件 | “不重复混合媒体”名词堆叠；强调是否在笔记间复用文件。 |
| zh-CN-17 P3<br>[performance.basic-mixed-shared-v2](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/zh-CN.json:180) | 共享混合媒体 | 图片与音频，共享文件 | “共享混合媒体”缺少共享对象；与上一条成对。 |

## 正文与元数据

### zh-CN-18 P3

[docs/zh-cn/index.md](/Users/hp/Desktop/2026/anki-forge/docs/zh-cn/index.md:1)

“将……生成”搭配不当，应为“根据……生成”或“将……打包为”。

原文片段：

> Anki Forge 将笔记、模板与媒体生成 Anki `.apkg` 文件。

建议替换：

> Anki Forge 根据笔记、模板和媒体文件生成 Anki `.apkg` 文件。

### zh-CN-19 P3

[docs/zh-cn/concepts.md](/Users/hp/Desktop/2026/anki-forge/docs/zh-cn/concepts.md:23)

“快照拥有所有权”主语含混。说明哪个对象持有数据，并保留 Rust 所有权含义。

原文片段：

> `Media` 是拥有所有权的快照，文件名由内容确定。

建议替换：

> `Media` 持有媒体数据的不可变快照，并管理其生命周期；导出文件名由内容确定。

### zh-CN-20 P3

[docs/zh-cn/updates.md](/Users/hp/Desktop/2026/anki-forge/docs/zh-cn/updates.md:5)

“身份证据”像证件，“源代码修订”也缺少对象；保留 identity evidence 的映射及版本含义。

原文片段：

> 每次新版本基于之前的**原始分发 APKG** 构建，该包携带完整、带版本的身份证据。将包和源代码修订一起保存。

建议替换：

> 每个新版本都以上一版分发时的**原始 APKG** 为基准构建。包内包含完整的标识映射和修订信息，且这些信息采用带版本号的格式。请将原始包与对应的源码版本一起保存。

### zh-CN-21 P3

[docs/zh-cn/templates.md](/Users/hp/Desktop/2026/anki-forge/docs/zh-cn/templates.md:5)

“显式媒体依赖闭包”是实现术语，应直接说明用户要准备的内容。

原文片段：

> 模板包是一组可复用的模型声明、模板文本、CSS 和显式媒体依赖闭包。

建议替换：

> 模板包包含可复用的笔记类型定义、模板文本、CSS，以及显式声明的全部媒体依赖。

### zh-CN-22 P2

[docs/zh-cn/build-guarantees.md](/Users/hp/Desktop/2026/anki-forge/docs/zh-cn/build-guarantees.md:41)

标题与下一句“已包含在公开包中”冲突；也应点明 publish 在这里是写入文件。保留旧锚点。

原文片段：

> ## 源码中的预备发布

建议替换：

> ## 准备候选包并在审阅后写入目标文件

### zh-CN-23 P2

[docs/zh-cn/python-api.md](/Users/hp/Desktop/2026/anki-forge/docs/zh-cn/python-api.md:70)

本节能力已进入公开包，不能继续用“源码 API”限定。

原文片段：

> ## 源码 API 详情

建议替换：

> ## 批量媒体处理与预备发布

### zh-CN-24 P3

[docs/zh-cn/rust-api.md](/Users/hp/Desktop/2026/anki-forge/docs/zh-cn/rust-api.md:3)

consumer application 指调用库的应用，不是消费类应用。

原文片段：

> 消费者应用不需要它们。

建议替换：

> 使用本库的应用不需要这些工具。

### zh-CN-25 P3

[docs/zh-cn/rust-guide.md](/Users/hp/Desktop/2026/anki-forge/docs/zh-cn/rust-guide.md:58)

“保持不会失败”有补丁说明口吻；明确这是方法返回行为。

原文片段：

> `Media::image()` 和 `sound()` 保持不会失败。

建议替换：

> `Media::image()` 和 `sound()` 本身不会返回错误。

### zh-CN-26 P3

[website/src/content/blog/zh-cn/stable-ids-and-deck-updates.mdx](/Users/hp/Desktop/2026/anki-forge/website/src/content/blog/zh-cn/stable-ids-and-deck-updates.mdx:4)

标题直接说明读者要遵循的规则，消除英文 outlive 的直译感。

原文片段：

> title: 笔记身份应当比措辞更持久

建议替换：

> title: 修改笔记内容时，保持 ID 不变

### zh-CN-27 P3

[website/src/content/blog/zh-cn/stable-ids-and-deck-updates.mdx](/Users/hp/Desktop/2026/anki-forge/website/src/content/blog/zh-cn/stable-ids-and-deck-updates.mdx:58)

reconcile identity 不是“协调身份”，应解释为确认新旧笔记的对应关系。

原文片段：

> 如果构建无法协调某条笔记的身份，应调查源数据和基线，不能仅把存在输出文件当作更新正确的证据。

建议替换：

> 如果构建时无法确认某条笔记与旧版本的对应关系，请检查源数据和基准包；不能仅凭输出文件存在就认定更新正确。

### zh-CN-28 P3

[docs/ai-quickstart.json](/Users/hp/Desktop/2026/anki-forge/docs/ai-quickstart.json:27)

internal tools 是普通描述，不是 API 标识符，应译为中文。

原文片段：

> 依赖、internal tools、仅源码可用的 API

建议替换：

> 依赖、内部工具、仅源码可用的 API

### zh-CN-29 P3

[website/site-model.mjs](/Users/hp/Desktop/2026/anki-forge/website/site-model.mjs:90) · `page:installation.title.zh-CN`

网页实际 H1 来自注册表，源文件虽已写 Rust，网页标题仍过泛。

原文片段：

> 集成到应用

建议替换：

> 集成到 Rust 应用

### zh-CN-30 P3

[website/site-model.mjs](/Users/hp/Desktop/2026/anki-forge/website/site-model.mjs:98) · `page:concepts.description.zh-CN`

“拥有所有权的笔记与媒体”和“更新身份”难读；改为关系说明。

原文片段：

> 理解 Project、拥有所有权的笔记与媒体、稳定键、文本和更新身份。

建议替换：

> 理解 Project、笔记与媒体的所有权、稳定键，以及更新时如何识别原有笔记。
