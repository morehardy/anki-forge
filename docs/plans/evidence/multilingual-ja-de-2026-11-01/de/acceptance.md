# 德语完整候选验收

2026-11-01，America/Los_Angeles。承接日语五语候选，德语形成六语完整候选。最终源码冻结于 `7b5f539ec2b8c34a0a9ac052531cda9571e97725`。**本地实现与双配置独立验收全部完成**；两套 `release-evidence.mjs finish` 和 `check:release` 均通过。未推送或部署。

## 实现与独立源审

德语交付29篇文档、2篇博客、210个业务词条和11个框架词条，以及全部元数据、导航、提示、语言规范和检索数据。六语共204个正式页面，另有404页；机器资源为186份Markdown和6份llms。

内容作者与技术、表达审阅者分别独立。最终204条双轴源审报告保存在 `technical-source-review-final.json`、`linguistic-source-review-final.json`。旧五语按真实差分和原始谱系复核，保留38项既有表达 minor；德语无未关闭 major／critical。MIME 修正另有两页增量及 Rust 单页增量报告，原报告保持原字节；当前源索引绑定三层审校链。

德语首次搜索验收为核心12/12、正例21/25、负例4/5。基于实际概念页、更新页和兼容性边界补充自然德语词形与检索数据后，最终达到12/12、25/25、5/5；没有改题、目标页、阈值或为德语修改排序引擎。原失败产物与日志保存在 `preview/initial-prefix-failure`，修订解释见 `search-data-correction.md`。

## 最终构建与工程验证

完整单测149/149；严格 Astro 类型检查123文件，0错误／警告／提示。文档、AI、Skill quickstart合同通过。实际执行14个完整文档程序并检查23个APKG；之后的 MIME 修正只增加行内代码标记，未改变可执行程序，严格内容门禁再次校验代码保真。

两配置 `inputSha256` 均为 `c1c3b90c31ad677b850f62e53bb1634d8a1696034faa7543fc877246ee6c0695`。

| 配置 | site / base / customDomain | 原始manifest SHA-256 | 交付文件 |
| --- | --- | --- | ---: |
| prefix | `https://morehardy.github.io` / `/anki-forge` / 空 | `62f9efd1c1417f9392407093209128887498c0a33295a4806ebfd4423f2250c6` | 705 |
| root | `https://ankiforge.dev` / `/` / `ankiforge.dev` | `a7276d9989d0648de6f0bfcca3e78a6d7a0f4a3db5d92dbc670b1ab20f60d279` | 706，含CNAME |

每配置实际复建确认397个HTML／Markdown／llms文件逐字节一致，八项工程门禁全部通过，含51项浏览器检查。真实Pagefind结果为 en 25/25、zh-CN 24/25、es 25/25、pt-BR 23/25、ja 23/25、de 25/25，全部语言核心12/12、负例5/5。既有非核心缺口如实保留。

## 产物任务与 MIME 修正

最终两配置均重新执行六语全部任务：72个浏览器使用任务、48个机器读取任务，共120个场景；72次真实程序复制、48次输入下载、168个必需机器HTTP响应。五页MIME检查另增加20份真实HTML／Markdown响应，不替代四类必需机器任务。

旧五语和已审德语正文仅在实际逐字或精确部署URL比较成功后复用语义判断；浏览器操作、复制、下载和HTTP请求均为最终产物的新运行。SDK消费执行证据是明确复用，独立重新核验23项输入、10个APKG、6份比较报告和14份日志；不宣称重新执行全部公开SDK程序或操作Anki UI。

每份使用报告绑定244项资源，其中192项为必需机器资源；每份机器报告绑定192项资源。全部绑定不等于逐页人工阅读。独立回答、引用和观察记录位于 `usage-tasks-final-{prefix,root}.json`、`machine-read-final-{prefix,root}.json` 及相应 conclusions／audit 文件。

旧 `8f2c77e` 产物的机器审读发现德语API的裸 MIME 通配符被 Markdown 强调语法破坏。先修 Python／Node，再由同类扫描补齐 Rust；media 和 rust-guide 原已正确。`6f5f860` 中间候选没有完成构建后验收，保存在 `intermediate-mime-prefix` 相关路径。最终五页15处字面量均通过实际 HTML 文本、代码节点、机器 Markdown 与 HTTP 响应检查；独立读者另外完整读取 Rust API 机器正文。红证据、增量审校和旧失败产物均保留，详见 `mime-correction.md` 与 `artifact-mime-closure-final-{prefix,root}.json`。

德语两配置各五题搜索负例，实际读取每环境12个命中，判断完整正文能力边界。root 与 prefix 的 canonical URL 及首页 noscript 链接按部署变化，审校明确记录，未宣称原始正文全字节相同。

## 布局与发布边界

prefix 的16组视口／CSS文字缩放测量均无横向溢出或页面错误；人工读取五张最终截图，包含两张与旧产物字节不同的768px截图和德语OG。原有搜索标题抽取缺少换行空格为非阻塞 minor，320px／200%下长词换行为 preference；未修改无关共享布局。实际字体检查来自macOS Chrome，不冒充Linux CI、真实浏览器菜单缩放或root完整窄屏扫描。

最终归档为 `target/remaining-locales/de/final-{prefix,root}-artifact`，原manifest及工程收据位于本目录 `final-*-origin-manifest.json` 和 `engineering/final-*`。证据提交不改变构建输入；新HEAD的CI仍须真实重建工程收据。

生产基线保持已核对的三语主线状态。P0维护、巴葡先发布及逐语verified基线推进仍是上线前步骤；本记录仅证明本地完整候选，不代表远端CI、合并或部署完成。
