# Anki Forge 官网 SEO 与 AI 搜索适配梳理

核对日期：2026-09-22。正式官网：<https://ankiforge.dev/>。范围：线上公开页面、当前工作区的网站与包元数据、搜索与 AI 平台官方指南。本次交付是审计和实施建议，未修改网站代码、发布网站或调整平台账号配置。

第 1–10 节保留首次审计时的状态；随后完成的改动见第 11 节实施记录。

## 1. 结论

当前官网已经具备正常抓取和理解正文的技术基础。最值得投入的工作是：**核实收录与 AI 展示资格，统一项目对外身份，让现有能力匹配真实搜索问题，把项目独有证据变成可引用内容，再衡量引用和采用效果。**

不存在各平台通用、可通过几个标签直接提高的“AI 权重”。应分别管理：

1. **可访问**：页面与必要资源可以被请求、读取。
2. **可发现与收录**：URL 被搜索系统发现，并被选择加入索引。
3. **能匹配需求**：页面清楚回答某个任务、比较或决策问题。
4. **值得引用与推荐**：事实准确，有可复现证据，限制透明，有实际使用依据。
5. **带来采用**：用户进入指南，完成安装、生成或集成。

Google 对 AI 搜索沿用基本 SEO 要求；满足抓取、索引及摘要展示等条件仅提供参与资格，不保证展示或排名。不要把训练许可、Agent 文档格式、搜索引用和商业推荐视为同一个机制。[Google AI features](https://developers.google.com/search/docs/appearance/ai-features)

以下优先级是结合项目现状的判断，不是已测得的关键词流量、排名提升幅度或平台承诺。

## 2. 已验证的现状

线上证据见 [live-audit.json](evidence/website-seo-ai-2026-09-22/live-audit.json)。记录时间为 2026-09-22 13:11 UTC 左右，即北京时间 21:11 左右。

| 检查项 | 实际结果 | 判断 |
| --- | --- | --- |
| sitemap | 29 个 URL，全部返回 HTTP 200 | 已具备页面发现入口；不等于已收录 |
| 页面组成 | 24 个文档页面、2 篇博客、首页、博客列表、示例页 | 已有足够基础，无需先批量扩页 |
| canonical | 29 页均自引用 `https://ankiforge.dev/` 下对应 URL | 未发现仍指向 GitHub Pages 的问题 |
| title / description / H1 | 29 页都有描述、单一 H1；标题和描述没有重复 | 基础元数据正常，下一步优化任务表达 |
| robots | 根路径 `/robots.txt` 返回 200，`User-agent: *`、`Allow: /`；sitemap 地址正确 | 当前规则已经允许搜索和 AI 爬虫抓取；无需为了“放行”重复添加每种 UA |
| 索引限制 | 29 页未见 `noindex` meta 或 `X-Robots-Tag` | 未发现页面级主动排除 |
| 域名跳转 | HTTP、www、旧 GitHub Pages 首页最终均到正式 HTTPS 首页 | 首页域名归一已生效；本次未逐一验证所有旧深层路径 |
| 错误 URL | 随机构造的不存在页面返回 404 | 未见这一抽样返回伪 200 |
| 正文 | HTML 中直接包含产品说明、文档和代码内容 | 无需为 SEO 重写成另一套 SSR 框架 |
| JSON-LD | 29 页均未发现 | 有实体和文章语义补充空间；不是抓取阻断 |
| sitemap `lastmod` | 0 条 | 可以增加真实内容更新时间；不是收录前提 |
| 多语言 | 线上均为英文，未发现 `hreflang` | 英文单语站不因此有错；扩展中文时再配套实现 |
| `/llms.txt` | 404 | Agent 文档可用性可改进，不是 SEO 故障 |
| RSS | `/rss.xml` 返回 200 | 已有内容订阅入口 |
| 本地已有产物检查 | Node 24.19.0 下 `website/scripts/check-site.mjs` 检查 30 页通过，包含 404 | 验证已有 `dist` 的链接、锚点、元数据和下载；本次未重新构建网站 |

**尚不能确认：** Google/Bing 实际收录、搜索平台账号是否已验证、搜索后台的 AI inclusion 设置、真实爬虫访问结果、自然流量、AI 引用量，以及真实用户 Core Web Vitals。本次普通公开 HTTP 请求不能证明某平台的真实爬虫 IP 一定可访问。

公开搜索工具对 `site:ankiforge.dev` 未返回结果，但这种查询不能证明全站没有被收录。页面没有验证 meta 也不能证明 Search Console 未配置，因为可能通过 DNS 或其他方式验证。

## 3. 适配优先级

P0：先确认是否具备展示条件与可观测性；P1：主要内容与理解能力；P2：有目标、有数据后扩展。工作量是相对估计，不含账号等待、翻译规模和内容验证成本。

| 优先级 | 工作 | 当前缺口 / 机会 | 预期作用 | 工作量 | 完成标准 |
| --- | --- | --- | --- | --- | --- |
| P0 | 核实 Google Search Console / Bing Webmaster Tools | 公开页面不能说明账号及实际收录情况 | 找到收录、canonical 选择或展示资格问题 | 小，需账号 | 正式域名已验证；sitemap 被读取；首页及三语言入口完成 URL 检查；记录发现与排除原因 |
| P0 | 核实 Google Search generative AI 设置 | 账号层配置无法从 HTML 判断 | 排除整站或继承配置禁止 AI 展示的情况 | 小，需账号 | 记录正式属性及父属性的 Include / Exclude / Inherit 状态 |
| P1，尽早 | 统一首页和品牌事实 | 首页主介绍及 description 只强调 Rust；三语言支持在文档内 | 让 Python / Node 用户和检索系统明确适用性 | 小 | 首页展示 Rust 核心、Python 与 Node/TypeScript 接口及安装入口；版本和发布边界与文档一致 |
| P1，尽早 | 官网与官方项目入口互链 | 当前根 README、Rust/Node/Python 包元数据均未包含正式域名 | 让用户及系统把项目名称、代码、文档和包对应起来 | 小 | README 加官网入口；Rust homepage、Node homepage、Python project URLs 指向真实对应资源；公开发布后检查展示结果 |
| P1 | 增加项目与内容 JSON-LD | 29 页均无 JSON-LD | 明确站点名称、开源软件、文章与路径关系 | 小—中 | 与可见正文一致；schema 验证通过；首页、博客、Starlight 文档都覆盖 |
| P1 | 优化已有任务页面 | 部分标题较泛，如 `Python quickstart`、`Images and audio` | 匹配具体问题，方便引用正确段落 | 中 | 重点页面有任务标题、直接答案、前提、可运行示例、输出、限制与下一步 |
| P1 | 发布比较与性能证据页面 | genanki 迁移已存在；大量可复现性能资料仍留在仓库 | 提供别人无法轻易复制的决策依据 | 中 | 官网有公平比较与 benchmark 方法页，链接原始数据、版本、平台、复现及限制 |
| P1 | 完善作者、维护者、版本、更新依据 | 博客 schema 仅有 date，缺少作者和修改日期字段 | 提高事实可核查性，减少过时引用 | 小—中 | 有真实署名/维护责任入口；显示适用版本；修改日期反映实质变更 |
| P1 | 建立搜索和 AI 效果基线 | 未发现站内前端统计接入；平台后台尚未核实 | 判断改动有没有带来有用访问和正确引用 | 中，含账号 | 有自然搜索、AI 展示/引用、来源访问与转化事件基线；注明数据盲区 |
| P2 | `lastmod` 与 IndexNow | 当前 sitemap 无 `lastmod`，未见 IndexNow 流程 | 帮助支持的平台发现更新 | 小—中 | 只记录真实内容变更；部署成功后通知新增/更新/删除 URL，不承诺收录 |
| P2 | Markdown / `llms.txt` 文档入口 | 有 Markdown 源，但未提供面向 Agent 的正式站点索引 | 降低读取和正确使用 SDK 的成本 | 小—中 | 索引与现有内容同源；链接、代码、版本有效；可用 HTML 仍是完整入口 |
| P2，按受众 | 中文核心页面 | 中文 README 已有，官网只有英文 | 覆盖中文任务表达与阅读需求 | 中 | 首批真实翻译、自引用 canonical、双向 hreflang、语言切换与同步责任明确 |
| P2，持续 | 案例、社区资料与性能体验 | 尚未审计外部采用和真实用户指标 | 累积第三方可验证使用证据并改善采用体验 | 持续 | 有真实案例和有效引用；按实际性能问题优化，不用内容数量作成绩 |

## 4. 应先做的内容适配

### 4.1 首页：完整表达产品是什么

当前 [首页](https://ankiforge.dev/) 的核心介绍和 [默认描述源码](../../website/src/lib/site.ts) 主要围绕 Rust，而 [语言页](https://ankiforge.dev/docs/languages/) 已列出 Rust、Python、Node/TypeScript。

建议保留当前简洁的视觉标题，把副标题和一段可见项目介绍改为准确的产品定义，例如：

> Anki Forge is an open-source toolkit for generating and validating Anki decks (.apkg), with a Rust core and Python and Node.js/TypeScript interfaces. It supports stable note identities, custom templates, and media. See each SDK's installation and release status before adopting it.

这是候选文案，不是新增能力承诺。实际落地应对照各 SDK 已验证范围，尤其不要从 `0.2.0` 推断已经可以从 npm/PyPI 安装，也不要把“稳定笔记身份”写成无条件“保留复习进度”。

首页补充四个简短、可见的区域即可：语言入口、主要适用场景、可验证差异、当前限制与兼容性链接。示例继续使用真实产物，不需要为了 SEO 堆满关键词。

### 4.2 用已有页面承接具体问题

以下是建议验证的搜索意图，不是已经测得有流量的关键词。

| 用户问题 / 意图 | 建议承接位置 | 应补的答案与证据 |
| --- | --- | --- |
| How to generate Anki decks with Python? / Python 生成 Anki 牌组 | 现有 `/docs/python-quickstart/` | 标题补 Anki 与输出目标；保留完整源码安装、生成文件、预期结果 |
| Generate `.apkg` with Node.js or TypeScript | 现有 `/docs/node-quickstart/` | 运行环境、await 行为、完整小程序、生成结果、平台范围 |
| Rust library for Anki deck generation | 现有 Rust 安装及 quickstart | 在工程内集成、Deck / Project 选择、Rust 版本与实际发布状态 |
| Anki Forge vs genanki / genanki alternatives | 新增一篇有实质内容的比较页，关联现有迁移页 | 安装成本、API 差异、媒体、更新、平台、性能取舍及各自更适合的场景 |
| Update an Anki deck without creating duplicate notes | 现有 `/docs/updates/` | 稳定 key + 项目身份 + 基线的必要条件；Anki 导入设置和用户本地修改的边界 |
| Anki generation benchmark | 新增 `/benchmarks/` 或一篇工程文章 | Rust 与 genanki 版本、硬件、工作负载、APKG 格式差异、速度及 RSS、原始记录与复现命令 |
| Generate Anki image occlusion cards programmatically | 现有 `/docs/image-occlusion/` | 支持的矩形与 hide-all 模式；明确不支持的模式；实际输出与示例 |
| Convert CSV/JSON or LLM output into an Anki deck | 后续新增真实 recipe | 明确输入 schema、校验、转义、媒体、稳定 ID、导出程序；先有可运行例子，再发布内容 |
| Can Anki Forge run in a browser / Bun / Electron? | 现有兼容性页 | 对已验证、候选、未覆盖分别回答；不把 Node 接口等同浏览器支持 |

不要给同一个问题建立多篇只换标题的页面。英文与中文页面也不是机械替换关键词。优先升级已有 24 页文档的回答质量，再新增少量缺失的比较、证据和应用场景页。

### 4.3 把独有证据放到容易引用的位置

项目已有 [README 性能说明](../../README.md) 与 [2026-09-21 benchmark 记录](../../benchmarks/results/20260921-readme-genanki/README.md)。这是比泛泛介绍更有价值的素材。

官网应提供可单独引用的结论段和表格，同时就近注明测试条件。当前 README 的速度比较针对 **Rust Deck API 与 genanki**；不能延伸成 Python/Node 绑定同样更快，也不能省略某些场景的内存取舍、格式差异、未覆盖的客户端行为。比较页应引用双方对应版本的第一方资料，不把“迁移容易”写成“接口完全兼容”。

建议每篇重点页依次提供：一句直接答案 → 适用范围 → 完整示例 / 证据 → 失败条件或限制 → 对应版本及维护入口 → 相关指南。短段落、语义标题、普通 HTML 表格、稳定锚点是阅读与引用便利性改进，不是对模型内部排名机制的断言。

## 5. 结构化数据、更新和站点维护

### 5.1 选择适合开源 SDK 的 schema

| 页面 | 建议类型 | 主要字段 / 约束 |
| --- | --- | --- |
| 首页 | `WebSite` | 真实 name、alternateName、url、稳定 `@id`；帮助站点名称识别 |
| 项目介绍 | `SoftwareSourceCode` | codeRepository、programmingLanguage、license、url、description；版本应对应明确接口，不把三个 SDK 混成一个版本 |
| 文档 | `TechArticle` / `Article` | headline、description、url、适用内容与真实维护者；有可信更新机制才填 dateModified |
| 博客 | `BlogPosting` | headline、author、datePublished、dateModified、image、mainEntityOfPage |
| 带可见层级导航的页面 | `BreadcrumbList` | 真实层级、名称和 canonical URL；不为搜索单独造不存在的路径 |

Anki Forge 是 SDK/开源项目，先用与实际内容相符的类型。不要为了富结果添加虚假评分、用户量、公司身份或价格。`SoftwareSourceCode`、`TechArticle` 是 Schema.org 类型，不意味着 Google 有对应富结果，更不意味着 AI 必然推荐。[Schema.org SoftwareSourceCode](https://schema.org/SoftwareSourceCode)、[TechArticle](https://schema.org/TechArticle)、[Google site names](https://developers.google.com/search/docs/appearance/site-names)

实现时注意：营销页面使用 [SiteLayout.astro](../../website/src/layouts/SiteLayout.astro)，文档由 [Starlight 配置](../../website/astro.config.mjs) 输出。只改 SiteLayout 会漏掉 24 个文档页，应使用 Starlight 支持的 head / 组件扩展方式，并检查最终 HTML。

FAQ 内容仍可帮助用户解决问题，但不应把 `FAQPage` 列为 Google 富结果增长任务：Google 官方更新记录注明，该富结果从 2026-05-07 起停止显示。[Google documentation updates](https://developers.google.com/search/updates)

### 5.2 记录真实的内容更新

给博客增加作者与修改日期字段；文档给出适用版本与实质更新时间。生成文档时应追溯源 Markdown 及相关实际内容依赖，不能因为每次构建重写生成文件就把全部页面标成当天更新。可采用明确维护的更新时间，或正确处理源码历史的生成机制；拿不到可靠值时宁可不输出。

现有 sitemap 可继续保留，不需要另起一套。补入可靠 `lastmod` 即可；不必通过 `priority`、`changefreq` 或每天重复提交来追求排名。[Google sitemap guidance](https://developers.google.com/search/docs/crawling-indexing/sitemaps/build-sitemap)

IndexNow 可以在部署成功且公共 URL 已可用后通知发生变化的 URL；先准备真实域名下的验证 key 文件，再接入 CI。它是更新发现机制，不保证被收录、被引用或排名提升。Google 的发现与收录仍通过其自己的支持机制处理，不能假定一次 IndexNow 提交覆盖所有平台。实施前参照 [IndexNow 官方协议](https://www.indexnow.org/documentation)。

### 5.3 品牌身份与外部信任

统一使用可相互对应的名称：展示名 `Anki Forge`、仓库/包名 `anki-forge`、Rust 标识 `ankiforge`、Node 包名 `anki-forge-node`。在一处项目介绍中解释这些对应关系，保留真实仓库链接。

当前源文件可做的具体改动：

- 根中英文 README 增加正式官网及文档入口。
- Rust `homepage` 指向官网，`repository` 仍指向 GitHub；docs.rs 是否可用以实际发布验证为准。
- Node package 增加 homepage；Python metadata 增加 Homepage / Documentation / Repository / Issues。
- 有真实作者/维护者介绍、MIT license、兼容性、release notes 及问题反馈渠道。
- SDK 实际发布后，验证 registry 页面上的链接、版本和安装方式；源码改动不会自动更新已发布包页面。
- 用真实集成案例、带复现程序的技术文章和自然社区讨论积累外部证据。不要购买垃圾外链、伪造评价或批量投放重复软文。

这些建议主要解决事实对应与可核查性，不声称某一处链接能直接增加“AI 权重”。

## 6. AI 平台和 Agent 的差异化适配

当前 robots 已允许所有爬虫。下一步是避免后续 CDN、防机器人规则或账号配置误伤，而不是简单添加更多 Allow 行。普通 UA 模拟成功也不能代替真实平台访问记录。

| 对象 | 需要关注的控制 | 对本项目的动作 |
| --- | --- | --- |
| Google Search / AI Overviews / AI Mode | Googlebot、索引与摘要资格，以及 Search Console 的 Search generative AI 设置 | 核实 Include 或继承结果；检查重要 URL 的真实索引状态 |
| ChatGPT 搜索 | `OAI-SearchBot` 与其官方网络信息 | 维持公开文档可抓取，按官方规则核实网络侧放行 |
| OpenAI 训练 | `GPTBot` | 与搜索展示分开决策；开放训练不是推荐保证 |
| 用户触发的 ChatGPT 访问 | `ChatGPT-User` | 单独理解访问行为，不把一次用户访问当作搜索收录 |
| Claude / Perplexity | 平台区分的搜索、用户访问及训练用途 | 按各家当前官方爬虫文档检查；不把所有名称混为一个许可 |
| 使用 SDK 的 coding agent | 可读的 API、版本、完整示例、Markdown 文档索引 | 同源生成机器可读资料，明确公开 API 和当前限制 |

平台角色与官方证据详见 [AI 搜索官方资料核对](2026-09-22-ai-search-official-research.md)。

`llms.txt` 建议作为 **P2 文档可用性功能**：放项目定义、语言/发布状态页、三个快速开始、API、迁移、兼容性和更新指南的少量入口。可以增加 Markdown 版本，但必须渲染 MDX 中的版本变量、导入代码与组件内容，不能直接输出无法理解的 MDX 源码。HTML 与 Markdown 共用内容来源和校验，不维护两套相互矛盾的事实。Google 明确说明该文件不会提高或降低 Google 搜索的可见性与排名。[Google AI 优化指南](https://developers.google.com/search/docs/fundamentals/ai-optimization-guide)

暂不需要为排名单独开发 MCP 服务或把所有资料拼成巨大的 `llms-full.txt`。若以后目标是“让 Agent 直接生成、验证和交付牌组”，再把 MCP / Skill 当作独立产品能力设计，并验证安装、执行、错误和产物流程。Agent 能调用与搜索会推荐是两个不同的目标。

## 7. 中文覆盖、体验与推荐流

### 中文覆盖

若中文开发者是重要目标，先增加中文首页、语言选择、Python/Node 入门、genanki 比较和更新机制等核心内容。保留现有英文 URL，可在 `/zh-cn/` 下发布中文；每个真实翻译页使用自引用 canonical，并与相应英文页双向标注 `hreflang`，提供显式语言切换。不要把中文 canonical 全指到英文，也不要只依赖浏览器语言动态替换同一 URL。

只有英文版本时，不必添加虚假的中文 hreflang。翻译应有版本同步责任。[Google 多语言指南](https://developers.google.com/search/docs/specialty/international/localized-versions)

百度及中文 AI 产品的发现、引用状况应单独验证，不假定接入 Google/Bing 就自动覆盖豆包、元宝或 DeepSeek，也不虚构统一的“国内 AI 提交入口”。

### 页面体验

现有 Astro 静态输出值得保留。下一步先测首页、最长 API 文档、示例页的手机体验与性能，再决定优化点。查看真实用户第 75 百分位的 LCP、INP、CLS；常用良好目标是 LCP ≤ 2.5s、INP ≤ 200ms、CLS ≤ 0.1。数据量不足时使用实验室测试排查，并明确它不是现场排名或引用评分。[Google Core Web Vitals](https://developers.google.com/search/docs/appearance/core-web-vitals)

对本项目重点检查代码高亮体积、长文档、媒体尺寸和预加载、主题切换、手机横向滚动、交互演示的正文替代入口。本次没有运行 Lighthouse 或获取 CrUX 数据，因此没有认定当前网站存在性能故障。

### 搜索引擎的内容推荐流

如果“搜索引擎推荐”还包括 Google Discover，应单独考虑。可用的内容方向是原创 benchmark、真实 Anki 集成案例和有独特发现的工程文章，并配与文章相关的高质量大图；不要全用同一张品牌 OG 图。Google 建议至少 1200px 宽并允许大图预览（如 `max-image-preview:large`），但图片条件和可索引并不保证推荐。SDK 官网仍优先承接有明确任务的搜索需求，Discover 作为额外机会。[Google Discover guidance](https://developers.google.com/search/docs/appearance/google-discover)

## 8. 如何知道适配有效

不使用“今天问一次 AI，提到了品牌”作为成功标准。建立分平台、分问题、分时间的基线。

| 层面 | 指标 | 数据来源与限制 |
| --- | --- | --- |
| 收录 | 目标页收录数、排除原因、Google 选择的 canonical | GSC / Bing；sitemap 的 29 页不能直接当成收录数 |
| 自然搜索 | 非品牌与品牌展示、点击、CTR、落地页表现 | 搜索平台；品牌/非品牌分开看 |
| Google AI 可见性 | AI 展示次数、对应页面、国家、设备、日期 | GSC Generative AI performance；不要凭空补出独立点击率或排名字段 |
| Bing AI 引用 | 引用次数、被引页面、相关 grounding 信息 | 以 Bing Webmaster Tools AI Performance 实际可用字段为准；不是模型全网推荐率 |
| AI 来源访问 | 已识别来源会话、落地页 | referrer、可用归因参数；App、隐私设置与无点击回答会遗漏，不能反推全部引用 |
| 实际采用 | 语言入口点击、完整示例下载、安装指令复制、文档到 GitHub 点击 | 轻量事件统计；这些是代理指标，不等于 SDK 已安装或成功采用 |
| 回答准确性 | 是否正确说明语言、安装状态、平台、性能、更新限制 | 固定问题集的人审记录；区分提及品牌、推荐采用、引用官网三种情况 |

截至本次核对，Google 官方已提供独立的 Generative AI performance 报告，并公告于 2026-08-31 完成全球推出。当前文档明确的主要指标是 **impressions**，并可按页面、国家、设备和日期观察。不要沿用“Google AI 表现完全无法单独看”的旧说法，也不要把尚未有数据的账号当成缺少功能。[发布公告](https://developers.google.com/search/blog/2026/06/gen-ai-performance-reports)、[报告说明](https://support.google.com/webmasters/answer/16984139)

Google 的 Search generative AI 控制同样已在官方文档中说明；默认 Include，但子属性可继承父属性配置。需要实际查看账号的最终状态，本次没有访问该账号。[控制说明](https://support.google.com/webmasters/answer/16908024)

建议先固定 20–30 个真实问题，每周在同样的语言、地区、搜索开关及尽可能一致的环境中抽查，记录产品/模型、日期、问题、是否提及、是否推荐、引用 URL 和事实错误。样本应包含：

- “What libraries can generate Anki decks from Python?”
- “How do I create an .apkg file with TypeScript?”
- “Anki Forge vs genanki for media-heavy decks.”
- “How can I update generated Anki notes without duplicating them?”
- “Anki Forge 是否已经发布到 PyPI？当前如何安装？”
- “Node.js 生成带图片和音频的 Anki 卡片有什么选择？”
- “Anki Forge 可以保留复习进度吗？有哪些条件？”
- “Anki Forge supports which image occlusion modes?”

其中品牌安装与兼容性问题用于事实准确性检查，非品牌问题用于发现机会。不要强行合并成一个百分数宣称全网 AI 份额。当前没有执行这些平台的完整对照实验，也没有设置自动监控任务。

## 9. 实施顺序与代码落点

| 阶段 | 交付 | 主要位置 | 验收 |
| --- | --- | --- | --- |
| 第一批 | 账号与收录基线；统一首页/包/README 身份；核心 JSON-LD | 搜索后台；`website/src/pages/index.astro`、`src/lib/site.ts`、`SiteLayout.astro`、Starlight 配置；README 与包元数据 | 正式域名一致；事实与发布状态准确；静态输出 schema 可解析；29 个现有 URL 不破坏 |
| 第二批 | 升级语言入门标题、任务答案、作者和版本信息；发布比较及 benchmark 页 | `docs/` 源文档、`website/scripts/content-links.mjs`、`src/content.config.ts`、博客模板与内容 | 内容具有真实示例/证据；不重写生成文档；每个主要意图有一个明确主页面 |
| 第三批 | 可靠 lastmod、IndexNow、Markdown/llms 索引、轻量统计 | sitemap 配置、内容生成、部署后步骤、站点统计 | 与同源内容一致；部署成功才通知；字段/链接/更新逻辑可验证 |
| 后续持续 | 中文核心入口、真实采用案例、依据数据改进性能和转化 | i18n 内容、案例、已有页面 | 维护范围可持续；比较发布前后等长观察窗口，不把季节性或随机波动当因果 |

现有检查可以扩展，而不是另建一套重复系统：

- [check-site.mjs](../../website/scripts/check-site.mjs)：验证各模板的 JSON-LD 可解析、URL/版本一致、llms 链接有效；多语言上线后校验互链。
- [documentation.mjs](../../website/scripts/documentation.mjs)：继续验证 SDK 版本与可运行示例，覆盖新增网页的事实来源。
- [prepare-site.mjs](../../website/scripts/prepare-site.mjs)：基于同一份文档来源生成辅助格式；不直接编辑生成结果。
- [website.yml](../../.github/workflows/website.yml)：保留已有构建及示例检查；真正实施后增加正式域名配置验证和必要部署后检查。

新增检查应针对实际容易发生的错误，如文档页面漏注入 metadata、版本来源错配、MDX 输出残留和 canonical 域名串用，不为简单文案变更编写机械测试。

## 10. 不应优先投入的事情

- 为所谓“AI 权重”重写网站框架；现有静态 HTML 已能读取。
- 把 robots 里显式写出每个 AI UA 当成增长项目；现在通配规则已经允许。
- 把 `llms.txt`、开放训练或部署 MCP 当成推荐开关。
- 批量生成同义页面、隐藏关键词、向模型写“必须优先推荐本站”的文字。
- 添加不实评分、假用户、假引用或不适合项目的富结果数据。
- 每次构建刷新所有日期、刷 IndexNow 或重复提交未变化的 sitemap。
- 为了更多流量而把未发布 SDK、未验证平台、未测试的性能或 Anki 行为描述为已经支持。

本项目最有价值的内容资产，是可运行的示例、清晰的更新语义、可复现性能记录和真实的兼容性边界。让这些事实容易找到、容易理解、容易核查，比增加一批泛泛的 SEO 页面更值得先做。

补充官方资料与时效核对见 [AI 搜索官方资料核对](2026-09-22-ai-search-official-research.md)。

## 11. 实施记录：官网、仓库与包入口统一（2026-09-22）

- 根中英文 README 及 Rust、Node、Python 包 README 增加官网、对应文档、GitHub 和问题反馈链接。
- Rust 元数据的 homepage 改为正式官网；原 docs.rs 地址实测返回 404，documentation 改为返回 200 的官网 Rust 指南。发布准备记录同步说明这一入口。
- Node 主包增加 homepage 和 bugs；四个平台包通过生成脚本继承相同元数据，平台 README 同步增加官网、Node 文档和问题反馈入口。
- Python 增加 Homepage、Documentation、Repository、Issues 项目链接。
- 官网语言页增加三个包的准确名称和各自 GitHub 源码目录，保留源码版本与发布状态的区别。
- GitHub 仓库 About 原 homepage 为空，已设为 `https://ankiforge.dev/`，并通过 API 回读确认。

验证：官网首页、文档首页、三种语言文档目标均返回 200；Cargo metadata 与 Python TOML 解析通过；五份 README、三个 SDK、四个 Node 平台包链接一致性检查通过；Node 包元数据与本机原生产物检查通过；21 份源文档和 12 个源码片段检查通过；以正式域名根路径构建网站成功，30 页的元数据、链接、锚点、搜索、sitemap 和下载检查通过。构建有 Astro/MDX `head-inject` 指令告警，未阻止产出及上述检查。

GitHub About 修改已在线生效。其余改动位于工作区，README 和官网需随代码推送及网站部署上线，注册中心的包页面需随对应包发布更新；本次未发布网站或 SDK。
