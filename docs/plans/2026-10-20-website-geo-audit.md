# Anki Forge 项目与官网 GEO 审计

核对日期：2026-10-20（America/Los_Angeles）。正式官网：[ankiforge.dev](https://ankiforge.dev/)。本地提交：`0738f61c192a5dc4a9ec297dd695a749e87fbf65`。

本轮检查了当前源码、全部 sitemap 页面、公开包元数据、GitHub 项目资料和搜索结果样本，并复核平台官方指南。只新增审计资料，没有修改网站、包、GitHub 设置或线上账号。以下优先级是基于现状的判断，不是已测得的排名因素或增长预测。

**主要判断：基础抓取条件已经较完整；下一阶段最有价值的是统一产品身份与版本事实、补齐选型和实际任务内容、把工程证据整理成可引用页面，并建立推荐与采用的测量基线。** 面向开发者的开源 SDK 应争取“用代码生成和维护 Anki 牌组”这类推荐，而不是泛泛竞争所有“AI 学习工具”问题。

## 1. 当前已经做对的部分

本轮普通 HTTP 抓取结果见 [live-audit.json](evidence/website-geo-2026-10-20/live-audit.json)。时间为 2026-10-21 03:38 UTC，即用户时区 10 月 20 日晚。

| 项目 | 本次实测 | 含义 |
| --- | --- | --- |
| sitemap | 30 个 URL，全部 200；25 个文档页、2 篇文章、首页、博客列表、示例页 | 已有完整内容入口；无需先批量扩充页面 |
| HTML | 正文和代码直接存在于静态 HTML | 没有发现必须运行客户端 JS 才能读取主体的问题 |
| 元数据 | 每页单一 H1；标题、description 无重复；canonical 均为正式域名的对应 URL | 基础规范正常 |
| 索引控制 | robots 为 `User-agent: *`、`Allow: /`；30 页未见 noindex meta 或 noindex 响应头 | 未发现公开页面主动阻止索引；不证明实际已收录 |
| 域名归一 | HTTP、www、旧 GitHub Pages 首页及一个旧文档深层路径均跳转至正式 HTTPS URL | 抽样跳转正常，没有必要无依据更换域名 |
| 错误页 | 人工构造不存在路径返回 404 | 此样本没有软 404 问题 |
| 发布入口 | 首页和语言页均有三语言 0.3.1 安装命令；npm latest、PyPI 当前版本均核实为 0.3.1 | 9 月报告中的主要采用障碍已明显改善 |
| 工程证据 | 可执行示例、下载 APKG、更新报告、按提交固定的 benchmark 原始证据 | 是项目值得继续放大的优势 |

README、官网与包的官方链接已经统一到 `ankiforge.dev`，不能继续把 9 月的“官网链接缺失”“首页只介绍 Rust”“Python 尚未公开发布”当作当前问题。公开检索本轮也已经能找到官网和多个文档页；这与上次抽样不同，但不能换算成 Google 全站收录率。

## 2. P1：品牌身份与包简介仍不够一致

**观察到的事实：**

- GitHub About 仍以 Rust toolkit 描述产品，没有提及 Python 或 Node/TypeScript；topics 也没有对应语言入口。[GitHub](https://github.com/morehardy/anki-forge)，[本次快照](evidence/website-geo-2026-10-20/github-metadata.json)
- PyPI summary 是 `Python Product API for Anki Forge`，几乎没有表达“生成 Anki .apkg、媒体、更新”等用途。npm 的简介使用较抽象的 stable Anki publications；两个注册表元数据的 keywords 都为空。[公开包快照](evidence/website-geo-2026-10-20/package-metadata.json)
- 品牌检索出现另一个提供账户、套餐和 Deck Studio 的 [AnkiForge 网站](https://ankiforge.com/account)，以及同名的 [软件开发服务](https://forge.anki.so/)。这些页面足以说明名称存在歧义；本轮没有判断它们的所有权关系。

**影响判断：**系统在汇总官网、GitHub、包目录时，更容易遗漏语言支持，或把项目与同名产品混在一起。这是实体识别风险，尚未通过跨平台回答实验量化。

**建议：**统一一组简洁事实：名称 Anki Forge、包名 `ankiforge`、官网、仓库、MIT、Rust 核心及三语言 SDK、生成 `.apkg`、需要稳定身份和持续更新的适用场景。首页首段应直接说明它是开发者库。可以使用类似介绍：

> Anki Forge is an open-source library for generating Anki .apkg decks from Python, Node.js/TypeScript, and Rust, with media packaging, stable note identities, and update diagnostics.

GitHub About、首页、语言页和包简介使用同一事实来源；各语言包突出自己的用途。补充少量准确 topics/keywords，目的是准确分类，不把堆关键词作为增长策略。包元数据修改需要随正常版本发布才能更新注册表，不应覆盖已发布版本。

## 3. P1：文档能教使用，但选型和任务入口不足

目前标题包括 `Python quickstart`、`Node quickstart`、`Images and audio`、`Update and distribute`。它们适合已进入文档的用户，对“用 Python 生成 Anki 卡组”“TypeScript 导出 apkg”“批量更新而不重复建卡”等问题，表达还可以更直接。标题来源主要在 [content-links.mjs](../../website/scripts/content-links.mjs)，首页和 Rust 入门在手写 MDX 中。

建议优先升级已有页，保留稳定 URL；只为不同的真实意图新增内容：

| 用户问题 | 建议页面处理 | 应提供的答案和证据 |
| --- | --- | --- |
| How do I generate Anki decks with Python? | 改进现有 Python quickstart 的 title/H1/首段 | 发布包安装、完整程序、预期 APKG、运行环境 |
| How do I export .apkg with TypeScript? | 改进现有 Node quickstart | 完整 TypeScript 运行方式；说明这是 Node 环境，并链接平台限制 |
| 如何把 CSV/JSON 词表生成 Anki 卡组？ | 新增一篇实际数据 recipe，优先 Python | 输入样例、持久 ID、转换代码、图片/音频、预期 note/card 数 |
| Anki Forge 和 genanki 怎么选？ | 新增独立决策比较页，与迁移页互链 | 适用/不适用场景、安装成本、媒体、更新、迁移成本、实测边界 |
| 修改生成的牌组后如何避免重复？ | 在现有 updates 页先给简短答案，再进入流程 | 稳定 namespace/key、原始发布 APKG、Anki 导入条件、不保证所有本地状态 |
| 如何批量创建 Image Occlusion？ | 改进既有页面并补最常用语言的完整示例 | 输入图片和区域、两个模式、导出结果及已验证条件 |

现有 [genanki 迁移页](https://ankiforge.dev/docs/genanki-migration/) 是 API 概念映射，解决“已经决定迁移后怎么做”；还不足以回答“为什么值得迁移、什么情况不应迁移”。应保留这个区别。比较 genanki 时不要暗示其完全不能指定稳定 GUID，也不能承诺直接迁移任意旧牌组并保留身份；比较对象的能力应链接它自己的文档并标注核对日期。

每个重要页面适合采用“直接答案 → 适用条件 → 完整示例 → 限制 → 证据/下一步”。这改善可读性和引用时的完整性，不是平台承诺的排名模板。暂时没有关键词流量数据，以上是从产品能力推导的候选意图。

## 4. P1：工程证据很好，但缺少完整、稳定的推荐依据页

首页已有真实性能数据，并明确 Node/Python bindings 未被该 benchmark 测量、部分媒体场景内存更高。这些限制应该继续保留。[性能组件源码](../../website/src/components/PerformanceEvidence.astro)

目前方法与数据主要在首页折叠区及 GitHub 报告里。建议建立官网可单独引用的性能页，包含：测试问题、双方版本/提交、机器、输入规模、时间与 RSS 表、输出验证、局限、原始数据和复现命令。首页只保留概要并链接过去。现有 Rust 与 genanki 的比较不能泛化成“Python SDK 快两倍”或所有工作负载的结论；如果希望争取 Python 选型推荐，应另外测已发布 Python 包的端到端场景。

另外，当前 30 个官网 URL 中没有独立真实采用案例页。GitHub 本次快照为 3 stars、0 forks，这只能说明该渠道公开活动规模较小，不能证明没有用户，也不是已知的 AI 排名阈值。建议完成两到三个可公开验证的集成示例，记录输入、实现、导出规模、限制和代码：例如词表转换、图片遮挡、课程牌组版本更新。维护者自己的示例应明确标为示例，不能包装成客户案例。

对外发布实用教程、参与相关开发者社区、争取真实下游项目引用，价值在于产生独立可核验的采用证据。本轮没有发布帖子、联系他人或创建外链。先准备有用成果，再做推广，避免批量目录提交和虚构评价。

## 5. P1：版本、正文和搜索快照存在不同步

本轮发现一个可以直接修复的正文问题：[Rust 入门文章](https://ankiforge.dev/blog/your-first-anki-deck/) 的当前代码使用 `project.add("es:hola", ...)`，正文却仍解释 `project.add("hola", note)`，并说 current quickstart 提供 source-checkout command；而当前 quickstart 已是 `cargo add ankiforge@0.3.1`。位置见 [文章源文件](../../website/src/content/blog/your-first-anki-deck.mdx) 第 22、26–28、36 行。

另有名称残留：媒体和更新指南仍使用 `from source` 小节标题，正文已说明这些方法包含在公开 0.3.1 包中；Python 迁移导航仍写 `Python 0.1 to 0.2`。这些不一定使代码无法使用，但容易让读者误判最新发布能力。应逐项审阅含义，更新标题时维护被链接的锚点。

公开检索工具读取 `/docs/quickstart/` 时还返回旧的 `git clone` 版本，而同一 URL 直接 HTTP 获取已经是新安装方式。这是本轮检索缓存/索引内容落后于线上内容的观察，不能据此判定所有平台都落后，也不能把它当成线上未部署。

建议增加实际适用版本、经过复核的更新时间、维护者信息。当前博客 collection 只有 date/category，缺少 author/updatedAt；30 条 sitemap 均无 lastmod。使用实质内容更新时间，不以每次构建时间刷新全站。为变更页保留稳定 URL、维护内链，按平台支持方式通知更新并在站长后台复查。当前 GitHub 0.3.1 Release 正文仅有 Full Changelog 链接，也可以补简短用户向说明，帮助理解新增能力、兼容性及文档入口。[Release](https://github.com/morehardy/anki-forge/releases/tag/anki-forge-v0.3.1)

## 6. P0 待核实：真正的收录、AI 引用和采用数据

公开页面无法判断 Search Console / Bing Webmaster Tools 是否已经配置，不能把“本轮没访问账号”写成“项目没有账号”。源码未发现常见站点统计埋点，也不能据此断言完全没有外部统计。

建议记录以下基线，与内容改进并行进行：

| 层级 | 要看的数据 | 不能混为一谈的指标 |
| --- | --- | --- |
| 抓取与索引 | sitemap 处理状态、首页与语言入口的 URL 检查、所选 canonical、排除原因、真实爬虫日志（如可取得） | HTTP 200 不等于已收录 |
| AI 展示/引用 | Google AI 展示、Bing AI Performance 引用页与查询样本 | 引用不等于在选型中被推荐 |
| 推荐准确性 | 固定问题集中的提及、推荐、官网引用及能力错误 | 品牌问答与非品牌发现分别统计 |
| 采用 | AI 来源访问、语言入门阅读、GitHub/安装复制等行为 | 来源标记可能缺失；下载/安装量不能直接全部归因于 GEO |

Google 当前提供 AI inclusion 设置和 AI 展示报表；应检查实际属性及继承状态。Bing AI Performance 提供引用和页面数据；新推出的 Topics Portfolio beta 可辅助发现机会，但小站数据不足可能看不到。详细官方依据及可见性限制见 [官方复核](2026-10-20-geo-official-research.md)。

人工评估可以先固定 12 个中英文问题：分别覆盖 Python/TypeScript 生成 APKG、CSV 转换、带媒体的牌组、持续更新、genanki 选型，以及品牌安装/兼容性。记录平台、模型/模式、是否联网、日期、完整回答和引用 URL；使用新会话、固定条件、重复采样。品牌问题衡量事实准确性，非品牌问题衡量发现和选型。用数周数据观察方向，不能把一次回答当作全平台推荐率。本轮未执行这套跨平台基准，也未创建定时监控。

## 7. P2：结构化数据、AI 文档和中文入口

**结构化数据。** 当前 30 页均没有 JSON-LD。可补首页的软件实体、官网与代码仓库关系，以及文章作者/时间；开发库可考虑 `SoftwareSourceCode`，文章使用适当 Article 类型。标注必须来自可见事实。它的用途是明确语义和减少歧义，不能保证更多推荐，也不要为通过富结果测试虚构评分或机构。

实现时需要同时覆盖 [SiteLayout.astro](../../website/src/layouts/SiteLayout.astro) 和 [Starlight 文档布局配置](../../website/astro.config.mjs)：25 个文档页不走自定义 SiteLayout，只改该文件不会覆盖它们。作者与更新时间需要扩充 [content.config.ts](../../website/src/content.config.ts)，生成文档要从源文件和内容导入规则维护，不手改生成产物。

**AI 文档入口。** `/llms.txt` 和 `/llms-full.txt` 当前都是 404，但这不是索引故障。项目已有 AI brief 和 Skill，它们有助于被选择之后正确使用，尚不能证明能提高被主动推荐的概率。建议把现有 [AI assistant 指南](../ai-assistants.md) 接入官网文档目录，提供安装/API/限制/版本的简洁机器可读索引，按需生成 Markdown，并保持与 HTML 同源。

首页 AI quickstart 区域占本轮抽取主文本约 39%（4,194/10,736 字符，包含无脚本模板；不等同于用户可视区域占比）。可保留简洁入口，将完整三语言模板放到独立指南，给首页更明确的定位、适用场景和证据留空间。这是信息组织建议，没有测得当前布局造成搜索惩罚。

Google 官方明确无需特殊 AI schema，且 llms.txt 不影响 Google 排名或可见性；OpenAI 搜索使用 `OAI-SearchBot`，与训练用 `GPTBot` 分开控制。当前通配 robots 已允许抓取，实际平台访问还需要结合其官方 IP 和主机日志核验。[Google 指南](https://developers.google.com/search/docs/fundamentals/ai-optimization-guide)、[OpenAI 爬虫说明](https://developers.openai.com/api/docs/bots)

**中文。** 当前官网均为英文，中文 README 不等于中文官网。如果中文开发者是目标用户，优先提供中文首页、Python/Node 入门、比较与更新指南，并实现对应 canonical/hreflang。先维护少量完整页，避免只翻译导航或铺设大量近似页面。英文单语本身不是技术错误。

## 8. 建议执行顺序与验收

| 批次 | 工作 | 验收标准 |
| --- | --- | --- |
| 第一批：事实与基线 | 修正文代码差异；统一 GitHub/首页/包简介；核对发布能力；记录站长基线 | 三语言和包名描述一致，教程与当前包对应，已知冲突清除，账号状态有实际证据 |
| 第二批：选型与场景 | 优化现有语言页；新增 genanki 决策比较和 CSV/JSON recipe；官网性能证据页 | 每页直接回答一个不同需求，有可运行程序、输入输出和限制，引用链接可复核 |
| 第三批：传播与可维护性 | 实际集成示例和用户案例；补作者/真实更新时间/lastmod；核心 JSON-LD；AI 指南及按需中文 | 内容事实来自维护源，所有关键页面均覆盖，案例归属准确，更新不会让文档分叉 |
| 持续观察 | 固定问题集、引用页、采用路径；按数据调整内容 | 分别报告提及、推荐、引用与采用，不合并成没有依据的“AI 权重” |

实施后沿用已有文档/示例/站点校验即可；新增检查应针对实际风险，例如 Starlight 漏加元数据、schema URL/版本错配、Markdown 链接失效和生成代码旁的正文过期。无需因为 GEO 重写 Astro、迁移主机、重复显式放行所有 bot 或搭建一个没有实际调用需求的 MCP 服务。

## 9. 证据与边界

- [线上抓取记录](evidence/website-geo-2026-10-20/live-audit.json)：状态、最终 URL、headers、页面元数据和正文；临时 raw_body_path 仅作本机复查，正文已保存在 JSON 中。
- [包注册表记录](evidence/website-geo-2026-10-20/package-metadata.json)：npm/PyPI 直接公开请求；crates.io API 本轮返回 403，因此未独立确认其当前注册表元数据，未将此视为官网故障。
- [GitHub 元数据](evidence/website-geo-2026-10-20/github-metadata.json)、[Release 记录](evidence/website-geo-2026-10-20/releases.json)：通过 GitHub 只读工具获取。
- [搜索抽样说明](evidence/website-geo-2026-10-20/search-observations.md)：记录缓存差异和品牌同名观察；不代表全网排序或平台推荐份额。
- [本次官方依据](2026-10-20-geo-official-research.md)；[9 月旧审计及实施记录](2026-09-22-website-seo-ai-audit.md)。

没有访问搜索后台或真实 bot 日志，没有测真实用户 Core Web Vitals，没有执行多平台 AI 推荐实验，没有重新运行 SDK/网站构建测试或复跑性能实验。本轮是公开事实和源码审计，不据此宣称某项优化必然带来排名、引用或转化增长。
