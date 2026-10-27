# Anki Forge 项目与官网 SEO 复核

核对日期：2026-10-20，America/Los_Angeles。官网：[ankiforge.dev](https://ankiforge.dev/)。范围：当前源码、正式站点 sitemap 中的全部 30 页、公开搜索抽样、公开 npm/PyPI 元数据，以及搜索平台官方规则。

**判断：官网已具备良好的技术抓取基础。下一阶段的主要投入应是搜索任务覆盖、内容准确性、项目身份辨识，以及搜索效果测量。** 当前证据没有显示需要通过更换框架解决 SEO；也不足以判断真实排名或哪些改动会带来多少流量。

本次新增审计文档与证据，未修改网站、发布配置或线上账号。9 月审计是历史记录；以下结论以本次复核为准。

## 1. 实际检查结果

线上抓取记录见 [live-audit.json](evidence/website-seo-2026-10-20/live-audit.json)，平台依据见 [official-guidance.md](evidence/website-seo-2026-10-20/official-guidance.md)。

| 检查项 | 当前结果 | 含义 |
| --- | --- | --- |
| sitemap 页面 | 30 页：25 个文档页、2 篇文章、首页、博客列表、示例页；均 HTTP 200 | 有正常的公开发现入口；不等于全部已被搜索引擎收录 |
| 正文 | 直接存在于静态 HTML | 产品说明、代码、比较数据不依赖执行 JavaScript 才能读取 |
| title、description、H1 | 30 页均存在，标题/描述未发现重复，每页一个 H1 | 基础标签完整；仍需改善表达和与正文的一致性 |
| canonical | 30 页均指向正式 HTTPS 域名下自己的 URL | 未发现线上 canonical 指向旧 GitHub Pages 的问题 |
| robots 与索引限制 | `User-agent: *`、`Allow: /`；30 页未发现 noindex 或 X-Robots-Tag | 未发现公开响应中的主动抓取/索引阻断 |
| 域名与路径归一 | HTTP 首页、www 首页、旧域名 Python 文档、无尾斜杠 Python 文档均返回 301 到对应规范 URL | 抽样的域名与路径迁移正常；未穷举每个旧 URL |
| 错误页 | 随机不存在 URL 返回 404 | 该抽样未见 soft 404 |
| 内链锚点 | 30 页之间的链接锚点未发现失效 | 现有导航的技术连通性正常 |
| JSON-LD | 30 页均无 | 可补充软件实体、文章和导航关系；不是收录阻断 |
| sitemap lastmod | 0 条 | 可补充可信的内容更新时间；该字段是可选项 |
| 内容语言 | 全部英文，未发现 hreflang | 英文站配置本身正常，中文搜索覆盖仍有拓展空间 |
| RSS / llms.txt | RSS 200；llms.txt 404 | 已有订阅入口；可按 agent 使用需要补充文档索引 |
| 注册中心 | npm/PyPI 均公开 0.3.1，官网链接正确 | 9 月提出的包入口关联已经落实；不能继续列为“缺少官网链接” |
| 公开搜索 | 能找到官网及多个文档页，也出现同名项目 | 已有搜索可见性；无法据此推算 Google/Bing 完整索引覆盖或排名 |

本地已有 `website/dist` 的 31 页检查通过，包括 404 页、元数据、链接、锚点、搜索索引、feed、sitemap 和下载一致性。该产物按默认 GitHub Pages `/anki-forge` 配置生成，不是正式域名构建；本次没有重新构建网站，线上结论来自单独的公开 HTTP 检查。用正式域名参数检查该旧产物时会因缺少 CNAME 失败，不能把这一配置不匹配当作线上故障。

未访问本站 Search Console、Bing Webmaster Tools、托管后台或真实爬虫日志，未取得 CrUX/Core Web Vitals 数据。因此，实际收录、搜索点击、AI 引用、外链规模和页面体验表现均属于待验证项。普通 HTTP 请求不能证明特定平台真实爬虫 IP 一定畅通。

## 2. 优先处理的问题

### P0：建立实际收录和效果基线

这是待核实事项，不是已经证明“网站没有收录”。先在站长平台检查正式域名属性、sitemap 读取、首页与三语言入门页的索引状态、搜索系统选择的 canonical、被排除原因，以及生成式搜索展示设置。公开搜索抽样不具有完整性。

建立以下基线：非品牌查询的展示/点击、核心落地页、普通搜索和 AI 来源访问、安装命令复制/示例牌组下载/仓库跳转。复制命令只能代表意向，不能等同于成功安装。源码未发现常见前端统计接入，但不能据此断言不存在托管侧或其他统计。

Google 当前提供独立生成式 AI 展示报告，Bing 也提供 AI 引用数据；有些功能需要足够数据才出现。应分平台记录，不能混成一个“AI 权重”分数。[Google 报告说明](https://support.google.com/webmasters/answer/16984139)、[Bing AI Performance](https://blogs.bing.com/webmaster/2026/2/Introducing-AI-Performance-in-Bing-Webmaster-Tools-Public-Preview/)

### P1：页面标题需要对应具体搜索任务

当前标题对已经了解项目的开发者有用，但部分标题缺少 Anki、APKG、生成牌组等任务语义。Google 建议独特、具体且与页面内容相符的标题；实际收益仍需查询数据验证。[Google 标题指南](https://developers.google.com/search/docs/appearance/title-link)

| 当前页面/标题 | 建议的页面主标题方向 | 执行注意 |
| --- | --- | --- |
| `/docs/python-quickstart/` — Python quickstart | Generate Anki decks with Python | 开头直接回答输出 APKG、安装方式、运行环境 |
| `/docs/node-quickstart/` — Node quickstart | Generate Anki decks with Node.js and TypeScript | 保留 Node 运行时要求，并链接 TS 配置 |
| `/docs/quickstart/` — Your first Anki deck | Create your first Anki deck with Rust | 明确语言和持久化输出 |
| `/docs/media/` — Images and audio | Add images and audio to generated Anki decks | 补充各语言实际入口、媒体路径与播放边界 |
| `/docs/updates/` — Update and distribute | Update generated Anki decks with stable note IDs | 解释重复卡片、原始 APKG 基线和客户端导入限制 |
| 稳定身份博客的抽象标题 | How stable note IDs help update generated Anki decks | 保留原有技术解释，用任务式标题承接问题 |

这些是内容方向示例，不是有搜索量证据的关键词清单。保持已有 URL；没有必要为了换标题同时迁移路径。文档标题来自 [content-links.mjs](../../website/scripts/content-links.mjs)，不是简单修改 `docs/*.md` 第一行就能改变网页标题。

首页已有清晰的三语言介绍，可以进一步用一句定义表明“开源 Anki APKG 生成库、Rust 核心、Python/Node 接口”，并评估首页 title 是否需要包含主要语言。避免把每种功能都塞进同一个标题。

### P1：修复内容与标签错位，减少相同入门内容的重复

**已证实的错位：** `/docs/api/` 标题为 Public API overview，描述提到兼容边界和 artifact ownership，实际正文来自 crate README，主要是安装和 hola/hello 入门示例。[线上 API overview](https://ankiforge.dev/docs/api/)、[生成映射](../../website/scripts/content-links.mjs)

`/docs/quickstart/`、`/docs/installation/`、`/docs/api/` 与第一篇 Rust 博客都围绕同一个入门程序。页面各有自引用 canonical，但主体任务重叠。没有 Search Console 查询数据，不能断言已经发生关键词竞争或排名惩罚。

建议保留一个明确的 Rust 快速入门主页面；安装页聚焦环境、依赖与集成；API overview 改成真正的接口导航，或在确认完全重复后合并并做永久跳转；博客讲一个独立的完整使用任务。不同内容仍保留各自 canonical，不要把不同页面全部 canonical 到首页。

**已证实的文案漂移：** 第一篇 Rust 博客显示的代码使用 `es:hola`，解释文字仍说 `hola`；“Run it from source” 一节声称 quickstart 提供 source-checkout 命令，而当前 quickstart 已使用公开包安装。这是内容一致性问题，会给读者和检索摘要提供不同答案。[文章](https://ankiforge.dev/blog/your-first-anki-deck/)、[源码](../../website/src/content/blog/your-first-anki-deck.mdx)

Python quickstart 的网页 description 还称提供真实文本、图片、音频示例，正文只有 Basic 文本牌组，媒体部分是后续链接；建议同步修改描述或补齐对应内容。描述没有必要机械凑到固定字符数。

### P1：补齐真实输入到牌组的任务教程

目前官网有 25 个文档页、2 篇文章，内容集中在安装、接口和概念；尚无独立的 CSV/JSON 到 APKG 完整教程。现有跨语言能力有价值，但很多任务指南主要展示 Rust，Python/Node 用户需要自行拼接 API 页面。

优先做少量可运行、可验证的内容：

| 读者问题 | 建议落地内容 | 必须提供的证据 |
| --- | --- | --- |
| 如何用 Python 把 CSV 生成 Anki？ | CSV → Basic/Cloze → APKG 教程 | 输入 CSV、解析依赖、完整程序、输出下载、稳定 key 规则 |
| 如何用 Node/TypeScript 从 JSON 生成牌组？ | JSON 数据到 APKG 教程 | Node/TS 项目文件、类型定义、运行命令、原生包平台限制 |
| 如何给生成的卡片带上图片和音频？ | 基于现有媒体页完善三语言路径 | 实际文件、相对路径处理、打包验证、客户端播放限制 |
| 如何更新已发布牌组，避免重复？ | 基于现有 updates 页增加端到端案例 | 前后两版数据和 APKG、身份规则、报告解读、导入条件 |
| 如何批量制作图片遮挡卡片？ | 基于现有 Image Occlusion 页增加完整数据流程 | 图片、遮挡坐标、输出、支持的模式与限制 |

CSV/JSON 教程应明确由调用方解析输入，不能暗示 SDK 已提供尚不存在的通用格式转换器。浏览器、PDF 提取、OCR 等能力也只能在真实实现并验证后宣传。

每个教程围绕一个不同问题展开，结构包含直接答案、适用版本、输入、可运行代码、输出、边界和下一步。在语言入口与相关 API 页增加正文内链。页面数量和字数都不应成为目标本身。

### P1：把比较与性能证据做成独立可引用页面

首页已有五类 workload 的性能图表、内存代价、测试条件和固定 commit 的原始证据链接，这是现有优势。官网仍缺少独立的选型比较页与 benchmark 方法页；现在的 genanki 页面主要解释概念迁移。[首页](https://ankiforge.dev/)、[迁移指南](https://ankiforge.dev/docs/genanki-migration/)、[性能组件](../../website/src/components/PerformanceEvidence.astro)

建议新增明确的比较入口，回答适合谁、功能差异、安装成本、平台支持、媒体处理、身份与更新、迁移限制、何时继续使用 genanki。独立 benchmark 页复用同一份数据源，保留原始数据与复现程序链接，并从首页和比较页互链。

**关键边界：** 现有性能数据比较 Rust Project API 与 genanki，不能写成 Python 或 Node SDK 也快相同比例；旧 genanki 包身份不能自动转换为本项目稳定身份；稳定 key 也不保证每种 Anki 客户端设置下都保留全部复习状态。准确表达这些边界，有利于系统引用可核实的选型结论。

### P1：减少同名项目混淆，改进对外描述

公开搜索能看到同名但用途不同的项目，例如 [alphatra/ankiforge](https://github.com/alphatra/ankiforge) 和 [Xpom1/AnkiForge](https://github.com/Xpom1/AnkiForge)。这是可观察的名称碰撞；本次没有证明某个搜索系统已经把它们合并为同一实体。

当前官网、README、包注册中心已经互链，继续保持。建议在首页、包描述、仓库简介和 About/维护者说明中使用一致的定义和正式域名，清楚说明这是由 morehardy 仓库维护的 Anki APKG 生成库，提供 Rust、Node/TypeScript、Python 接口。

PyPI 当前摘要仍是 `Python Product API for Anki Forge`，没有充分解释用户能完成的任务。可以改为类似 `Generate Anki APKG decks in Python with media, custom note types and stable note IDs.`，随下次正常发版同步。npm 摘要中的 stable Anki publications 也可以更直接表达生成 Anki decks。[PyPI](https://pypi.org/project/ankiforge/)、[Python 元数据](../../bindings/python/pyproject.toml)、[Node 元数据](../../bindings/node/package.json)

后续补充真实使用案例、可运行集成仓库，以及维护者参与的社区教程，作为外部可验证的采用证据。本次未取得 backlink 或外部引用数据，不能断言外链不足，也不建议购买或批量制造链接。

### P2：补充作者、更新时间与结构化数据

当前 30 页未发现 JSON-LD；博客 schema 只有 title、description、date、category、draft，没有作者或修改日期字段，模板也不显示维护责任。建议：

- 首页描述 WebSite 与真实软件实体，关联官网、代码仓库和包注册页；维护者信息以真实公开身份为准。
- 文章增加真实 author、实质修改时间与适用版本，并用 Article/BlogPosting 表达相同可见事实。
- 文档导航有合适的可见路径时，补充 BreadcrumbList；需要同时覆盖独立 SiteLayout 和 Starlight 文档模板。

这是理解与可信度补充，不是保证提高排名的开关。Google 的 SoftwareApplication 富结果有额外要求；没有真实评分时不伪造评分。当前 Google FAQ 富结果已停止，不应以增加 FAQPage 获取该展示为目标。[结构化数据政策](https://developers.google.com/search/docs/appearance/structured-data/sd-policies)、[软件富结果要求](https://developers.google.com/search/docs/appearance/structured-data/software-app)、[Google 更新记录](https://developers.google.com/search/updates)

代码落点：[SiteLayout](../../website/src/layouts/SiteLayout.astro)、[Starlight 配置](../../website/astro.config.mjs)、[内容字段](../../website/src/content.config.ts)、[博客模板](../../website/src/pages/blog/[...id].astro)。

### P2：改进内容更新发现与发布校验

本次 sitemap 没有 lastmod，未发现 IndexNow 流程；公开搜索抽样有些摘要仍展示旧 Node 包名或旧安装方式，而直接抓取的当前页面已经更新。这只能说明抽样结果存在缓存/更新滞后，不能断定原因或保证加 lastmod 就能解决。

可从真实内容依赖生成 lastmod。网站页面由 Markdown、代码示例和生成数据共同构成，仅使用构建时间或单个文件的修改时间都可能不准确。发布成功后可按实际变化向支持的平台发送 IndexNow 通知，并在后台复核索引。[Google sitemap 指南](https://developers.google.com/search/docs/crawling-indexing/sitemaps/build-sitemap)、[IndexNow FAQ](https://www.indexnow.org/faq)

另一个维护风险是默认 site config 与 CI fallback 仍指向旧 GitHub Pages 域名。当前线上正确；建议正式部署缺少域名变量时直接失败，并增加部署后的 canonical/robots/sitemap 抽样检查，减少以后配置丢失造成回退的风险。[配置](../../website/site.config.mjs)、[工作流](../../.github/workflows/website.yml)

### P2：按目标受众扩展中文与 agent 文档入口

现有中文 README 没有对应的中文官网。若中文开发者是目标用户，先翻译首页、Python/Node 快速入门、genanki 比较和常见问题，再逐步扩展。中文页使用自己的 canonical，与真实对应的英文页建立双向 hreflang，并明确维护责任。单纯修改 lang 标签不能替代翻译。[Google 多语言说明](https://developers.google.com/search/docs/specialty/international/localized-versions)

项目已经有 AI prompt builder 和 Skill。可以把 `docs/ai-assistants.md` 纳入官网导航，并由同一份文档生成简洁的 llms.txt/Markdown 入口，帮助 agent 找到正确版本与公共 API；Google 明确不把 llms.txt 用于提高搜索排名，因此其优先级应低于教程、内容准确性和实际收录诊断。[Google AI 优化指南](https://developers.google.com/search/docs/fundamentals/ai-optimization-guide)

## 3. 建议执行顺序与验收

| 阶段 | 内容 | 验收结果 |
| --- | --- | --- |
| 第一批 | 检查搜索后台；修复 API overview 与博客漂移；重写核心任务标题/描述；统一项目定义 | 有索引与查询基线；标题和正文一致；已有路径与链接保持有效 |
| 第二批 | Python CSV 教程、Node JSON 教程、选型比较与独立 benchmark 页 | 每页都有完整输入、运行程序、输出及适用边界；加入语言页和相关文档正文内链 |
| 第三批 | 真实作者/日期与 JSON-LD；可靠 lastmod；生产域名检查；可选 IndexNow | 静态输出可验证；日期反映实质变化；部署后正式域名信号一致 |
| 持续 | 中文核心页、真实案例、agent 文档入口，按数据改进页面体验 | 比较相近时间窗口的非品牌查询、落地页点击、AI 引用和使用意向，记录每批上线日期 |

页面体验应结合移动端 Lighthouse 和实际 Core Web Vitals 检查 LCP、INP、CLS。本次只检查了 HTML、外部入口脚本和资源响应，没有完成渲染性能实验，不把动画或图片当成已证实的速度问题。[Google 页面体验说明](https://developers.google.com/search/docs/appearance/page-experience)

如果只选三件先做：**修正现有页面的任务表达与事实一致性；发布 Python/Node 的完整数据到牌组教程；把 genanki 比较与性能证据建立为独立官网入口。** 搜索后台诊断可与这三项并行开展，发现真实索引阻断时优先处理。
