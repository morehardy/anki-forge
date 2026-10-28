# Anki Forge 官网与文档多语言实现方案

日期：2026-10-27。适用范围：官网、文档、博客、搜索、交互提示、SEO/GEO 与发布维护。最近有限生产观察的提交：`732a40f0e6f3e534233330dac7898fdca1593a27`。本次方案对齐的源码基线：`2c07690`（已包含机器读取实现）；源码提交不等于已验证上线。

**先在现有英文和简体中文上完成共性改造并证明兼容，再完整发布西班牙语，随后推进巴葡、日语与德语、法语与俄语。所有语言采用独立 AI 审校、修改与复审；每种语言覆盖完整主题后发布。**

本方案替代[前期扩语方案](2026-10-27-multilingual-expansion-plan.md)的实施顺序、质量门槛与资源估算。市场依据保留在[语言市场研究](evidence/multilingual-expansion-2026-10-27/language-market-research.md)。讨论中的反对意见及处理见[讨论与决策记录](evidence/multilingual-expansion-2026-10-27/discussion-record.md)。

本次已对照[机器读取与结构化数据规格](../superpowers/specs/2026-10-27-website-machine-readable-spec.md)及其实际实现，保留原机器接口与验收合同，扩展其语言维度；具体兼容性结论、代码依据与调整清单见[对齐评估](evidence/multilingual-expansion-2026-10-27/machine-readable-alignment.md)。用户已确认的独立 AI 审校要求适用于机器读取验收，不继承该规格中的人工发布门槛。

## 一 已确认约束和完成定义

用户已确认三项决策：

1. **独立 AI 审校修改后即可，不需要母语验证。** 发布流程不依赖真人审校或人工用户招募。记录实际 AI 审校方式、问题、修改和复审结果。
2. **每语完整后发布。** 当前每语为 34 个主题，包括 29 个文档主题、首页、示例页、博客列表和两篇文章（2026-10-28 经用户确认纳入 CSV／AI 两个新增主题）。内部可以逐页准备，公开发布必须整语完成。
3. **锁定语言和推进顺序。** 处理既有 en／zh-CN，新增 es、pt-BR、ja、de、fr、ru；按中英共性改造 → 西语完整试点 → 巴葡 → 日语／德语 → 法语／俄语推进，印尼语保留后续候补。

本次设计的共同目标是：既有英文／中文 URL、深链接、可抓取内容、技术事实及功能没有已知工程回归；所有语言使用同一套质量与发布机制；新增语言具有自然表达、可检索的当地用语和可完成的使用路径。搜索排名、索引速度和 AI 推荐次数可能受外部系统影响，不能作为工程上“绝不波动”的保证。

实施完成需要同时具备内容与代码、检查结果、独立 AI 复审证据和实际部署证据。可生成的检查协议不能记成已经通过的检查；AI 按文档完成任务记为代理使用验证，不记为真实用户采用。

## 二 现网基线和已经存在的能力

2026-10-27 20:04:43 PDT 整理了本轮有限只读观察，读取发生于此前数分钟，对应 UTC 为 2026-10-28 03:04:43。以下五个公开 URL 均返回 200：

| 入口 | 观察结果 |
| --- | --- |
| [发布清单](https://ankiforge.dev/generated/site-manifest.json) | 当时提交为 `732a40f0e6f3e534233330dac7898fdca1593a27`；65 个 HTML，含 64 个正常内容页；语言为 en 与 zh-CN；正式域名、根路径与 CNAME 配置正确 |
| [站点地图索引](https://ankiforge.dev/sitemap-index.xml) | 指向正式域名的 sitemap-0.xml |
| [站点地图](https://ankiforge.dev/sitemap-0.xml) | 64 个 loc |
| [robots.txt](https://ankiforge.dev/robots.txt) | 允许公开抓取，并列出正式 sitemap |
| [中文首页](https://ankiforge.dev/zh-cn/) | html lang 为 zh-CN，自引用 canonical 正确 |

这五个响应没有 `X-Robots-Tag`。本轮未逐页重新获取 64 页及全部资源，实施阶段仍需完成正式发布基线提取。

现有静态正文、三套 SDK 程序、共享版本与性能来源、双语 canonical/hreflang、真实 404、Pagefind、产物资源哈希、生产域名独立校验及根路径／项目前缀构建均应继承。框架继续使用当前 lockfile 的 Astro 与 Starlight，不将框架升级、托管迁移或首页重设计混入扩语。

发现的实际缺口包括：页面与语言仍有二元硬编码；兼容性检查缺少独立的历史公开面；正常页缺少统一的意外 noindex/nosnippet 与 `X-Robots-Tag` 检查；章节按标题下标配对；译文检查强制标题深度和表格行形状；审校只记录双语 hash；数量文案有数字与名词拼接；浏览器搜索词仅有很少的中英样本。机器实现已读取响应头验证 MIME 和下载行为；扩语时复用该能力，补齐正常 HTML 的索引／摘要检查。

机器读取实现现已进入当前 HEAD `2c07690`（`Add machine-readable website resources`），包含 llms、实际 HTML 导出的 Markdown、JSON-LD、机器／schema 检查及线上资源校验。本次只读核对规格和代码，未重新验证生产、运行构建或修改网站实现。机器规格中“待实现”及更早的生产观察是历史记录，不能代替当前代码状态或本节有限生产观察。

实施时分别保存“当前代码集成基线”和“最近真实验证的公开基线”。若机器能力先完成正式发布验证，S0 随即把它的公开接口纳入保护；若尚未上线，则作为本次中英共性发布必须继承、完成验收的能力，不能伪称已有线上证据，也不因等待部署而阻塞只读盘点、语言准备或本地接口泛化。

## 三 每种语言都需要的共性修改

下表的实现对所有语言生效，语言具体词句、地区风格和查询词另在内容流程中维护。

| 工作包 | 当前问题与修改 | 主要位置 | 必须通过的验收 |
| --- | --- | --- | --- |
| C1 历史公开面保护 | 保存上一已验证发布的 HTML／机器 URL、锚点、页面角色、发现关系和实体身份；后续检查不得仅与当前 registry 自洽 | 新增 `website/public-surface-baseline.json`、提取／对比脚本，接入 `check-site.mjs` | 旧 URL 与有含义的旧锚点保留；故意同时删 registry 和页面／机器入口仍被拦截 |
| C2 通用语言与页面模型 | 替换 page.en/page.zh、zhSource 和中英判断；一个发布集合驱动所有入口 | `site-model.mjs`、`core-pages.mjs`、`src/content.config.ts`、组件 locale 类型 | 仅启用中英时仍有原 64 个内容页；新增语言不需复制业务组件 |
| C3 元数据和抓取资格 | 收敛已有 metadata 描述器与 schema 调用，官网和 Starlight 各自适配；检查正文与响应头 | `SiteLayout.astro`、`route-middleware.ts`、`astro.config.mjs`、robots、两套检查脚本 | 唯一 self canonical；双向 hreflang；Markdown／RSS alternate 共存；正常页无意外索引／摘要限制 |
| C4 语义章节与链接 | 用稳定章节 ID 保护任务含义，解除标题下标依赖；来源语言显式传递 | `content-sections.mjs`、`legacy-sections.json`、`content-links.mjs`、生成器 | 旧锚点落到原任务；切换到同主题对应章节；正文可自然拆合与调整非依赖顺序 |
| C5 通用事实和审校 | 技术事实、代码与版本共享；审校按主题、语言和检查类型记录 | `content-reviews.json`、`check-content.mjs`、新增内容契约与语言规则文件 | 零未关闭 critical/major；变更按依赖失效；禁止自动更新 hash 伪装审校 |
| C6 完整 UI 表达 | 完整句子翻译、数量与复数、日期数字、错误／空态／aria 文案 | 两组词典、`messages.mjs`、HomeShowcase、Examples、Docs 系列组件 | 关键状态全覆盖；占位符正确；0/1/2/5/21 等数量成立；代码数值不本地化 |
| C7 同语搜索和切换 | 按 locale 配置 Pagefind、查询夹具和语言选择；构建期验证目标锚点 | LanguageSwitcher、DocsSearch、Pagefind、`browser.test.mjs` | 真实构建中查询命中、语言正确；日语分词及 pt-BR 映射可用；无脚本语言链接有效 |
| C8 共享任务与 AI 入口 | 三 SDK 安装、代码、输出、限制保持统一；AI 模板按自然语言展开 | `docs/ai-quickstart.json`、AI 渲染／检查器、公开包消费者 | 三 SDK 的页面、复制内容和静态模板一致；生成／更新程序实际结果满足契约 |
| C9 资源与部署一致 | 生成器只处理 releaseLocales 中来源完整的语言；清除受管理残留；浏览器不携带八套词典 | prepare-site、客户端导入链、social-image、feed、404、manifest、workflow | 无草稿与 fallback 泄漏；新增语言不导致客户端资源线性膨胀；正式产物与 verified manifest 一致 |
| C10 SEO/GEO 观测 | 所有语言使用稳定任务 ID，按真实查询／回答与来源评估 | 既有 GEO 协议、题集、采用记录及新的语言查询夹具 | 本地测试、外部搜索表现、AI 回答与采用证据分别记录，不相互替代 |
| C11 机器读取能力扩语 | 继承 `2c07690` 的 llms／Markdown／JSON-LD，同时泛化生成、清理、检查与线上资源集合 | machine-content 配置／生成器、published-pages、machine-urls、structured-data、machine／schema／HTTP 检查 | 每语 HTML 与机器资源同批完整；旧 URL、正文保真、实体身份和 MIME 合同持续通过 |

### 旧内容兼容性如何实现

从上一份 verified artifact 的实际 HTML 提取 `route / pageId / locale / public anchors / canonical / protected capabilities`，保留对应发布提交与来源摘要。关键安装、版本、API 能力与更新条件由现有技术来源建立清单。这个历史基线不随每次 build 自动更新；只有新发布验证完成后才显式推进基线，并保留前一版本。

对已验证公开的机器能力，同时提取 llms／Markdown 的 route、所属 pageId／locale、HTML 发现关系与可见入口、稳定的站点／软件／源码／页面／文章实体 ID；记录直达 200、UTF-8、允许的 MIME 和非 attachment 合同。manifest 的资源 hash 用于证明本次部署等于本次产物，历史兼容检查保护 URL 和语义；正常更新允许资源 hash 改变。机器 URL 与 HTML 一同受旧公开面保护，不能只检查新 registry 中仍存在的资源。

CI 使用最近已验证发布的不可变 artifact／记录，或 PR 基线版本中对应的受信清单；核对发布提交与摘要，不直接信任候选分支修改后的基线文件。推进基线和兼容性豁免也不能由候选代码自证。本轮不删除既有公开路由或语义锚点；未来确需迁移时，以单独审阅的迁移说明、映射与验证证据处理，禁止通过刷新允许列表绕过历史保护。

基础设施泛化阶段保持英文／中文正文、页面角色和既有标题描述稳定；确需修正的内容单独列出差异并审核。检查使用“旧公开面必须被新产物兼容”，并允许已审核的新增语言、alternate 与章节别名。不能要求所有 HTML 字节相等，也不能允许重新生成一份遗漏相同内容的基线来过关。

旧锚点除了存在，还必须指向同一任务的正文位置。把所有别名插在页顶不能通过语义复核。代码、示例下载和必要限制保持可达；源码编辑链接仍指向真实可编辑来源。

### 索引和摘要资格如何保护

正式正常页验证 200、最终 URL 无非预期跳转、正文存在于静态 HTML、canonical 指向自己、robots 可抓取；读取 HTML robots／googlebot 指令与实际 HTTP `X-Robots-Tag`，检查意外的 `noindex`、`none`、`nosnippet`、`max-snippet:0` 等限制，以及关键正文的 `data-nosnippet`。现有不存在的限制不应在泛化时悄然引入。404 单独保持真实错误状态与 noindex。

Google 的 AI 搜索入口要求页面可索引并具备摘要资格，因此这些检查同时保护 SEO 和 Google AI 搜索入口。[Google AI features](https://developers.google.com/search/docs/appearance/ai-features)、[robots 与响应头规则](https://developers.google.com/search/docs/crawling-indexing/robots-meta-tag)

### 值得同步完成的 SEO 和 GEO 增强

加强以下共同内容结构：首段直接说明能完成的任务；前提贴近步骤；限制贴近会影响判断的结论；代码可运行且有预期输出；选型比较有真实条件及来源；更新任务清楚说明原始 APKG 基线。保留稳定项目身份、正式包名、版本来源和三 SDK 同等可见的入口。内容改写须有具体问题依据，避免批量改写当前英文标题以追逐关键词。

本轮新增 x-default，并统一现有元数据生成；Google 不要求为 AI 搜索额外创建专门 schema 或 AI 文本文件。扩语复用当前 llms 与 schema 系统，按 C11 保留和泛化，不能以它们不是排名门槛为由移除。批量 FAQ schema、IndexNow、新分析服务及全站复杂日期依赖图不作为扩语首发新增工作。[Google AI 优化指南](https://developers.google.com/search/docs/fundamentals/ai-optimization-guide)

未来新增作者、导航等 schema 字段时仍需真实可见依据。`lastmod` 仅在能准确记录重要修改时输出；无法可靠维护时省略，不能使用每次构建时间制造新鲜度。[Sitemap 指导](https://developers.google.com/search/docs/crawling-indexing/sitemaps/build-sitemap)

### 机器读取能力的扩语合同

当前双语实现中，`llmsUrl()` 的白名单、llms 输出路径、`otherLanguage`、WebSite `inLanguage`、导出命名空间、覆盖扫描和线上检查均有中英限定。`publishedPages()` 遍历全局 localeIds，却只比较前两个 draft 值。它们在当前范围内成立，增加第三语时必须成组泛化，不能仅放宽 URL 工具或增加词典。

| 合同 | 多语言处理与保留边界 |
| --- | --- |
| 同一构建集合 | HTML、sitemap、hreflang、Pagefind、JSON-LD、llms、Markdown、检查器使用第四节冻结的 `buildLocales / buildPages`；配置了语言不等于对外启用 |
| URL 与覆盖 | 保留 `/llms.txt`、`/zh-cn/llms.txt` 和既有 canonical 下的 `index.md`；新语使用其注册 prefix 下的 llms。只导出 docs／article，首页、示例、博客列表和 404 不增加 Markdown；当前每语 31 份 Markdown 加 1 个 llms |
| 唯一正文来源 | 继续从本次 HTML 的 `.sl-markdown-content`／`[data-machine-content]` 提取；保留代码空白、表格数据、Tabs 全部面板、details、提示、附件与链接。不直接公开未求值 MDX，不维护第二套翻译正文 |
| 精选索引 | 继续复用同一 topic 分组及顺序；本语内容只链接本语 Markdown，`otherLanguage` 泛化为其余构建语言列表。公开版本／安装／runtime 仍来自共享配置，摘要与限制说明单独审校 |
| 索引大小 | 保留每文件 UTF-8 **24 KiB** 上限、LF、末尾换行和确定性；八语时计入最多七个其他语言入口。按字节测试日语／俄语与长摘要；先精简自然说明，必要时显式调整各语共同精选集合并复审，不能自动截断、删尾或提高上限 |
| 发现与搜索 | 正常 HTML 保留本语 `rel=describedby`；docs／article 保留 `rel=alternate type=text/markdown` 和无脚本可见入口；RSS 与 hreflang 同时存在。404 不输出机器发现。sitemap 与 Pagefind 继续只收正常 HTML，不把机器文件当新增主题 |
| 实体身份与真实性 | 保留站点根下 `#website / #software / #source`，页面／文章 ID 来自其 canonical；WebSite `inLanguage` 使用本次构建集合。保留现有页面类型、最小软件引用和安全序列化；不批量新增作者、评分或虚构日期。公开版本仅在有可见依据的位置声明，历史文章不自动标为适用最新版 |
| HTTP 与清单 | llms 必须 text/plain；Markdown 接受 text/markdown 或 text/plain；实际字节为 UTF-8、直达 200、无强制下载，未知机器路径真实 404。沿用 generate→check:machine→check:schema→manifest 顺序，线上继续核对同一 artifact 全资源 hash 与响应头 |
| 所有权与清理 | 原本仅识别 en／zh-cn 的保留命名空间、生成、实际覆盖扫描、故障注入与线上检查一起泛化。受管理集合还包含先前生成或撤回候选的语言前缀，以移除残留；只清理受管理导出物，不能扩大为删除任意 Markdown。全部提取和验证成功后才写出 |

实体共享指身份及技术事实一致；各语可见软件描述和 `softwareHelp` 可指本语文档。Markdown 的 `Canonical page` 只是来源说明，不替代 HTML canonical；不新增针对 Markdown 的 noindex、robots 特殊政策、UA 分流、内容协商或 `llms-full.txt`。

自然表达与导出保真是两个检查方向：跨语言按任务、事实、步骤依赖和程序身份核验，允许自然重排；同语 HTML→Markdown 按实际正文校验内容与顺序。不能把八语 Markdown 的标题层级、表格形状或代码块位置强制对齐。历史锚点留在对应 HTML 章节，Markdown 内链接继续指向 HTML canonical＋fragment，无须保留空 alias span 或制造另一套 Markdown 锚点。

构建后的独立 AI 从 llms 入口定位 Python 安装、Node API、JSON 更新与兼容性限制，保存实际读取 URL、回答、引用、缺失与复审结果；这取代机器规格的“人工读取验收”，沿用其任务和证据要求。全部机器文件先通过确定性检查，AI 再复核每语索引摘要及这些任务涉及的输出；转换器或组件变更时扩查所有受影响输出。不重复人工翻译或逐页重写生成结果，不把被告知 URL 的读取测试计为自然 GEO 发现。

## 四 通用模型与生成边界

### 语言状态

语言注册表包含 `id / pathPrefix / starlightKey / htmlLang / ogLocale / label / formatLocale / dir`。自然语言称为 `locale`，Rust／Node／Python 称为 `sdk`，避免语言菜单与 SDK 选择混淆。注册表不再保存另一份可变发布 `state`；planned／candidate／published 由以下事实派生，避免验收后改状态导致重建：

| 事实 | 来源与用途 |
| --- | --- |
| configuredLocales | 所有已配置语言；只表示配置存在，不驱动全部页面生成 |
| sourceReady | 完整来源、UI、元数据及构建前事实／表达审校通过；尚不代表 Pagefind、机器输出或浏览器通过 |
| releaseLocales | 最终构建前提交的显式发布目标快照；普通发布保持已公开集合，扩语批次加入本次完整候选语言 |
| publishedLocales | 最近真实验证成功的发布记录所含集合；用作历史事实和防止删语的下限，不由候选产物自我宣告 |

一次正式候选构建固定 `buildLocales = releaseLocales`；所有成员必须 sourceReady，并包含最近 verified 发布的全部语言。planned 或不完整译稿不参与源读取、Starlight 注册、机器导出及正常发布检查，只在隔离编辑预览中查看。为中英泛化编写第三语小型 fixture 不要求把不完整西语注册进正常站点。

`publishedPages()` 改为显式消费本次 `buildLocales`：对所有构建语言检查来源完整、draft 一致，再过滤整组均为 draft 的未公开主题，得到 `buildPages`。不遍历所有 configuredLocales，也不能仅比较前两语；某一语设 draft 不能成为跳过缺译的方式。已公开主题即使全语改 draft，仍会被历史公开面检查拦截。

将 `buildLocales / buildPages / siteConfig / publicInstall` 形成单次构建上下文，官网、Starlight、元数据、搜索、机器导出和所有检查器共同消费其发布选择；验证器仍独立检查实际 HTML、源文件和历史基线，不能只比较生成器自报的结果。上下文显式传递，不让各模块再次用不同的全局 localeIds 猜测语言集合。

完整候选通过构建后验收后，直接部署这一份 artifact；不先改 catalog 为 published 再重建。线上验证成功才推进 verified 发布记录，后续普通构建必须继续包含其全部语言，不能静默漏掉新语。候选失败不产生已发布状态；删除已公开语言仅允许第十节明确的应急恢复例外。

Starlight 会为缺失译文生成 fallback。继续保留 `route.isFallback` 失败门槛，且在路由生成前先检查源覆盖。[Starlight fallback 说明](https://starlight.astro.build/guides/i18n/#fallback-content)

### 页面来源

`pageId` 是全站稳定主题身份，统一解析到 `sources[locale]`、`metadata[locale]`、路由和必需能力。保留英文现有来源、`docs/zh-cn/` 和英文根路径；新增 `docs/es/`、`docs/pt-br/`、`docs/ja/`、`docs/de/`、`docs/fr/`、`docs/ru/`。博客保留 translationKey，按 locale 路径维护。

`pagePath(pageId, locale, sectionId)` 负责地址；`resolvePage(path, base)` 负责解析。显式处理 base、尾斜线和已知语言前缀，未知前缀不能伪装成英文页。站内主题链接保持当前 locale；共享媒体、APKG、registry 和外部源码使用原目标。

生成目录只保存产物，不作为编辑源。清理范围仅限生成器拥有的目录，不能删掉英文手写 MDX 或其他源文件。反向扫描源和 dist，发现未登记文件、缺页或残留。

### 元数据

收敛官网和 Starlight 现有 metadata 对象，保留 `id / type / locale / canonical / title / description / publishedAt / category`，由同一描述器生成 self canonical、构建集合的语言 alternate、同主题英文 x-default、OG 和 RSS 信息；继续调用现有 `markdownUrl / llmsUrl / entityIds / buildStructuredData`，不建立第二套 URL 或 JSON-LD 生成器。标准语言代码与路径目录明确映射，例如 `pt-BR → /pt-br/ → pt_BR`。

标签按语义去重：语言 alternate 的键含 hreflang，机器表示的键含 `type=text/markdown`，RSS 的键含 `type=application/rss+xml`；不能过滤全部 `rel=alternate` 后只加语言链接。每正常页只有一个受管理 JSON-LD 图；文档中间件不能把所有非中文 route.lang 转成英文。共享实体 ID 和页面类型沿用 C11。

hreflang 只列出真实可访问的同主题版本，各版本相互回链。sitemap 列出实际发布内容，排除错误页和草稿。英文保留 `/docs/.../`，中文保留 `/zh-cn/docs/.../`；其余 slug 与主题保持一致。[Google 本地化版本说明](https://developers.google.com/search/docs/specialty/international/localized-versions)

### 章节与内容契约

每个主题维护 `requiredTasks / requiredSections / protectedFacts / codeSources / dependencyOrder / dataTables`。章节采用稳定 ID；推荐在真实任务章节前使用显式 source marker，例如 `<!-- section: installation -->`，由 Markdown／MDX 解析阶段绑定真实章节并生成稳定锚点及历史别名。代码围栏或示例字符串中的同样文本不能被识别为 marker；重复或无法定位的真实 marker 报错。现有自然语言标题仍可用于显示和本地 slug，语义映射不依赖标题在数组中的下标。

允许本地化标题、拆合句段、增加解释层级、重排没有依赖的章节及调整表格表达形式。必须保留事实、必要步骤和数据，依赖步骤保持正确先后。受保护的程序与共享片段仍从原执行源校验；表格校验针对单元数据与信息完整性，不以管道字符数量替代语义。

自动检查验证 marker、值、引用和链接；独立技术 AI 必须逐项指向实际正文证据，判断条件是否完整、锚点是否仍有原含义。填写契约 JSON 本身不构成正文通过。

## 五 如何验证表达地道和技术准确

### 每语输入包

每种语言维护一个轻量 brief：目标地区与受众、称谓和语体、术语表、允许保留的英语、搜索表达、禁用直译及证据来源。术语条目至少包含 `conceptId / preferredTerm / allowedVariants / explanation / source / observedAt`。同一个术语在不同上下文可有不同自然表达，但 note、card、deck 等概念必须可区分。

依据依次为：本项目实际技术事实和当前公开 API；对应 Anki UI 的概念；本语言开发者在真实资料中使用的表达；AI 提出的待验证种子。关于功能的事实和关于语言用法的证据分开。旧 Anki 译本可提供线索，但部分译本可能过期，必须记录版本与适用范围。[Anki 翻译说明](https://docs.ankiweb.net/manual/intro#translations)

所有标题、描述、按钮、aria 文案、复制反馈、搜索空态、AI 模板及社交图文案均属于审校范围。示例数据 `hola → hello`、命令、路径、API 名称、note key、原始程序输出保持共同技术源。

机器内容中的自然语言也进入 brief 与审校：`machine.intro / version / constraints / sourceBuild / history / repository`、分组显示名、其他语言入口说明及可见按钮。协议例外使用精确白名单：`Optional` 分组字面值、`Canonical page / Language / Description / Published` 标签、Schema.org 类型／属性、locale ID、代码语言标记、section ID、URL、占位符与安装命令不翻译；对应自然语言值仍需审校。独立检查固定协议值，不能仅验证“输出等于同样误译的词典”，也不能把任何英语片段都误判为 fallback。

数量采用完整计数短语；确需复数变化时使用 `Intl.PluralRules` 对应类别。词典仍校验稳定消息 key 和变量，但允许不同语言需要不同复数分支，不假定英语 one/other 两档覆盖全部语言。先修复 HomeShowcase 的数字与名词拼接以及示例计数短语。[Intl.PluralRules](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/PluralRules)

### 独立 AI 审校流水线

| 步骤 | 角色和可见输入 | 输出与约束 |
| --- | --- | --- |
| 1 编写 | 作者读取事实清单、源文、术语与语言 brief | 完整译稿、元数据和 UI；不得自行扩充能力承诺 |
| 2 单语盲读 | 表达审校代理先只看目标语言、读者任务与本地语料 | 找出翻译腔、不自然术语、歧义和难用动作；不先接受作者解释 |
| 3 事实对照 | 独立技术代理读源文、译文、程序与事实清单 | 逐条核对事实、条件、代码与输出；标记缺漏、扩张承诺和错译 |
| 4 搜索与任务 | 独立使用代理读页面并使用浏览器、搜索与公开执行环境 | 查询结果、任务路径、复制内容和失败位置；不借隐藏源码弥补文档 |
| 5 修改 | 作者按具体反馈修订；协调者用技术源和语料裁决分歧 | 每个问题关联修改位置和理由 |
| 6 复审 | 非作者的对应审校代理复核最终修订 | 关闭已修问题，并检查修改是否引入新问题；作者不能独自宣告通过 |

不同代理可使用同一模型，但需独立任务上下文；不临时要求外部模型或人工团队。最多四个并发角色时，将编写、审校、使用测试按依赖轮换。模型间意见一致不能替代证据，具体原句、技术来源、可运行结果和本地用法必须可追溯。

每项问题至少记录 `pageId / sectionId / span / category / severity / explanation / proposedFix / evidence / resolution / verifierRun`。只有“很地道”或总分不构成审查结果。每页最多连续三轮定向修订；仍有严重问题时，先重写问题源段或拆解歧义，再新开独立复审，不能自动降格发布。其他页面和语言可以继续准备。

### 四层验收门槛

以下数值是本项目可执行门槛，西语试点可依据实际失败样本调整；修改 rubric 必须注明理由，不能为放行已知错误临时降低标准。

| 层 | 范围 | 通过标准 |
| --- | --- | --- |
| 技术准确 | 全 34 主题、全部 UI／元数据／模板 | 已知 critical 和 major 为零；共享事实、限制、命令、API 和输出无差异 |
| 自然表达 | 每篇全文独立盲读；所有标题描述、关键动作与动态状态 | 措辞术语、句法连贯、技术文体／地区一致、动作与页面适配四项分别至少 4/5；没有系统性翻译腔或影响理解的表达；平均分不能抵消任一分项缺陷 |
| 搜索适配 | 每语 30 条冻结查询及其结果 | 12 条核心任务全部在前 3 位命中；25 条正例至少 23 条在前 3 位命中；同语结果及负例行为正确 |
| 使用完成 | 每语六个代理任务 | 三 SDK 入门、JSON 媒体、原始基线更新和排错均可按页面完成；零文档导致的失败 |

自然度各分项评分解释：5 表示自然且无需语言改写；4 表示仅少量风格修改、不影响理解；3 表示明显翻译腔或需要重组；2 表示妨碍理解；1 表示误导或不可用。每页每分项均给原句位置及判断依据，关键 UI／元数据按所属页面或明确消息组记录。不使用字数比例、反向翻译相似度或汉字出现率代替。

错误等级按影响划分，参考 MQM 的准确性、术语、流畅性和语境类别，但门槛属于本项目定义。[MQM 分类](https://new.themqm.org/mqm-pillars/typology/)

- **Critical：**错误安装命令、错误身份或更新基线、可能破坏数据的操作指引、虚构能力或保证。
- **Major：**混淆 note/card/deck、漏关键前提、动作与链接语义错误、标题承诺与正文不符、步骤无法照做。
- **Minor：**不影响含义的局部语法、用词或标点。反复出现的同类问题需扩查全部受影响位置。
- **Preference：**两个说法都成立的风格偏好，不计为错误，也不能无限循环修改。

## 六 如何符合当地搜索和使用习惯

### 搜索词的来源与边界

每语查询记录包含 `query / locale / intentId / sdk / expectedPageIds / provenance / sourceUrl / observedAt`。`provenance` 至少区分本站真实查询、外部实际表达、AI 构造测试词。Google 结果页上出现某个说法，只证明表达和内容存在；不能据此填写搜索量。

有 Search Console 时优先读取本站任务查询、页面、国家和设备；没有后台时，仍可完成本地语料整理与站内检索验收。页面 locale、推断查询语言与国家分别记录；纯 APKG/API 名词保留语言未知。Search Console 没有直接的用户语言维度。[Search Console 维度](https://support.google.com/webmasters/answer/7576553)

关键词作者先读取任务场景和本地语料，再写查询，不先照抄待测页面标题，以减少过度拟合。自然查询优先覆盖生成牌组、JSON、媒体、genanki 选择／迁移、Image Occlusion、稳定身份与更新。对应页面在自然句子中使用合适术语与必要同义表达，不堆砌关键词。

### 每语 30 条站内检索夹具

| 类型 | 数量 | 内容 |
| --- | ---: | --- |
| 核心任务 | 12 | 生成、媒体与 JSON、选型／迁移、更新和排错；三 SDK 均有覆盖 |
| 术语与 API | 8 | deck/note/card 当地术语、APKG、genanki、Image Occlusion 和真实 API 名称 |
| 有效变体 | 5 | 自然同义词、重音／屈折或分词差异、英语技术词夹杂的真实表达 |
| 负例 | 5 | 无意义输入与明确不支持任务；预先定义空结果、明确边界页或安全恢复的预期 |

25 条正例的 expectedPageIds 在执行前冻结；12 条核心任务全部要求前 3 位命中，其余按总计至少 23/25 验收。负例不一律要求空结果，例如不支持的操作可以返回明确说明边界的页面，但不能给出承诺支持该操作的错误结果。top 3 指 Pagefind 的站内结果，不是 Google 排名承诺。

Pagefind 依据 html lang 分组；实际构建需验证日语无空格查询、重音／变体、pt-BR 标签映射及错误状态。结果默认保持本语言；可以提供明确的英文搜索入口，不混入未标记的英文结果。[Pagefind 多语言说明](https://pagefind.app/docs/multilingual/)

### 六个使用任务

1. 从首页选择 Rust，找到并完成最小 APKG 程序。
2. 从本语言搜索选择 Python，找到并完成最小 APKG 程序。
3. 从文档导航选择 Node／TypeScript，按页面编译／运行并得到产物。
4. 找到 JSON 和媒体教程，取得输入文件，理解并完成预期牌组。
5. 保留原始 v1 APKG，找到并完成答案更新和比较流程，确认不会把 Anki 重导出包作为有效基线。
6. 面对一个受控安装／媒体／身份错误，从对应语言页面找到正确修复与验证步骤。

保存实际 URL、查询词、页面截图、复制内容、命令、产物和失败原因。相同源码与依赖的执行结果可以复用；每语仍须核对页面入口、复制出的代码／命令和本地说明。任何翻译改变了可执行输入，都重新执行相关程序。缺少运行环境时标记未验证，不记通过。

## 七 审校记录和持续更新

保留 `website/content-reviews.json` 作为紧凑的发布索引，升级为带版本的每主题每语言记录；较长的审校问题、修复和执行材料存入 `docs/plans/evidence/` 对应批次目录并以相对路径引用，不进入产品侧栏。

```text
schemaVersion: 2
entries[pageId][locale]
  sourceHash / targetHash / technicalDependencyHash
  rubricVersion / glossaryVersion
  technical: status, reviewerType=ai, reviewerRun, evidence, openIssues
  linguistic: status, reviewerType=ai, reviewerRun, score, evidence, openIssues
  search: querySetVersion, requiredCoverage, resultContract
  task: scenarioVersion, requiredCoverage, resultContract

releaseEvidence[artifactDigest]  # CI 附件或构建输入之外的批次证据
  commit / inputSha256 / configuration / releaseLocales / manifestSha256
  searchResults / taskResults / browserResults
  machineChecks / schemaChecks / aiMachineReadReview
  evidenceDigests / openIssues / releaseGateResult
```

采用两段式门槛：**source-ready → 固定源码与 releaseLocales 构建 → 构建后验收 → 发布同一 artifact**。`content-reviews.json` 保存构建前事实／表达审核、不可变证据摘要和搜索／任务覆盖合同；Pagefind、六任务、浏览器、机器内容和 JSON-LD 的实际通过结果保存在构建后的 releaseEvidence。不能要求这些实际结果先通过才允许第一次构建。

`content-reviews.json` 本身属于 manifest 的输入哈希范围。构建后证明存入已排除的 `docs/plans/evidence/` 或 CI 附件，绑定 commit、inputSha256、configuration、releaseLocales、manifest 及相关资源摘要，不能将本次 artifact hash／通过状态写回构建输入后又重建。发布门槛连接源审核和产物证明；任一缺失或摘要不符均失败。发布时复用已验收字节，不因“转为 published”改变输入或产物。

每轴证据保存或引用不可变输入快照，绑定实际 source、target、技术依赖及所用 rubric、glossary、query、scenario 的摘要；不能只在顶层填当前 hash，却引用针对旧输入的报告。targetHash 不包含审校记录自身、运行时间或最终产物。修订后只允许复用输入确实未变化的检查结果；全站 inputSha256 标识构建，不单独作为所有语言全文审核失效的依据。

首页、示例和博客列表的 targetHash 覆盖实际参与渲染的本语言词典、元数据、内容数据及对应模板；不能只 hash 共享 Astro 文件。动态状态文案、AI 模板和机器索引文案纳入各自来源闭包，确保更改文案会使旧审核失效。

search/task 合同可在 releaseEvidence 中关联该语言级别的一次报告，说明当前页面属于哪些任务；不要求每篇 API 页重复执行六个场景。无对应独立场景的页面标记检查覆盖来源，不能伪造逐页执行记录。机器入口任务可复用相同程序、依赖及输入的执行证据，但须保存实际机器读取路径与结论。全量技术与表达审核仍逐页完成。英文来源页也执行表达和事实检查，不虚构另一份英文译文；中文与新增语言均使用同一质量机制。

| 变更类型 | 需要失效和重验的范围 |
| --- | --- |
| 某语言某页的措辞 | 该页表达审核、受影响语义及标题／搜索用例 |
| 英文技术事实、共享代码、版本或命令 | 依赖该输入的全部已发布语言页面和相关执行场景 |
| 英文纯表达调整 | 独立 AI 确认语义未变后更新关联记录，不能自动刷新所有 hash |
| 术语表改变 | 使用该术语的语言页面与 UI；发现错误应扩查同类位置 |
| 共享 UI 或插值改变 | 对应 key 的所有语言与涉及的交互状态 |
| 共享页面模板改变 | 相关页面类型的功能、布局、静态正文和 metadata，不机械重审无变化长文 |
| 机器导出器、正文组件或精选索引改变 | 受影响 HTML→Markdown 保真、索引摘要、发现关系和产物证明；源文未变不机械重做全部表达盲读 |
| 新增主题 | 所有已发布语言的正文、元数据、导航、链接、机器表示与审核 |

源摘要和依赖摘要负责发现变化，独立 AI 证据负责说明复核结果。重大技术错误不能因为某语尚未完成而继续发布新的错误内容；先在分支修复相关来源和已发布语言，再整体交付。

## 八 语言范围和分批方式

| 阶段 | 处理语言 | 正常 HTML | Markdown | llms | 重点和退出条件 |
| --- | --- | ---: | ---: | ---: | --- |
| M0 | 现有 en、zh-CN | 64 | 待核验 | 待核验 | 分别冻结源码与已验证公开面；确认机器能力实际发布状态 |
| M1 | 现有 en、zh-CN | 68 | 62 | 2 | 完成 C1–C11；中英正文与全部旧接口回归，机器能力同一产物验收 |
| M2 | es | 102 | 93 | 3 | 首个完整新增语言；独立审校、搜索、使用和机器读取任务、正式部署通过 |
| M3 | pt-BR | 136 | 124 | 4 | 验证区域语言 ID、目录、OG、搜索及机器 URL 映射 |
| M4 | ja、de | 204 | 186 | 6 | 各自完成即发布；验证日语分词、索引字节预算、德语长文案和布局 |
| M5 | fr、ru | 272 | 248 | 8 | 各自完成即发布；验证法语表达、俄语屈折和数量短语，全六语维护机制就绪 |

数值是各阶段完成后的规模说明，不是已通过结果；M4／M5 中每单语独立增加 34 个 HTML、31 份 Markdown 和 1 个 llms。当前每语 34 个主题、31 个 docs／article；八语合计 272 个正常 HTML 与 256 个机器文本资源。CI 按实际注册主题与构建集合计算，不硬编码永久数量，404 不计入正常 HTML。

既有英文／中文进入第一阶段的共同检查，修正有证据的表达问题。例如当前中文“关于制作笔记的笔记”需要独立通读，owned media 也应按上下文解释，避免机械翻译；技术词仍保持原语义。

新增语言范围为中性西语 `es`、巴西葡语 `pt-BR`、日语 `ja`、德语 `de`、通用法语 `fr` 和俄语 `ru`，分别使用 `/es/`、`/pt-br/`、`/ja/`、`/de/`、`/fr/`、`/ru/`。不同时拆出西语各国、葡萄牙葡语或法语地区副本。

此排序沿用公开市场研究，不能证明本站实际流量排名。印尼语 `id` 是下一轮最强候补，韩语、意大利语、印地语、阿拉伯语等保留候选；实施过程中不静默替换六语范围。阿拉伯语另需 RTL 布局与双向代码展示设计。

每种语言先在隔离预览校准五类样本：首页／UI、一个 quickstart、JSON recipe、更新页、一个较密集 API 页。样本用于发现术语、模板和审核问题；随后完成整套主题，不能把五篇样本当成部分公开发布。

同一队列可并行准备，但发布独立进行；巴葡无需与西语同日发布。前一语言的质量／工程试点完成后即可推进下一队列，不必等待 28 天搜索增长。搜索观测独立进行。

## 九 实现切片及依赖

| 切片 | 具体交付 | 依赖与验收 |
| --- | --- | --- |
| S0 冻结基线 | 记录机器实现提交，提取最近 verified 公开面；HTML／已公开机器 URL、锚点、发现关系、实体 ID 和关键能力 | 区分源码、本地、CI、生产证据；机器功能已上线则立即纳入旧接口保护 |
| S1 增加回归门槛 | public surface 对比、索引／摘要资格、HTTP 合同与负向 fixtures | S0；删旧页／锚点／机器入口、错误 MIME、意外 noindex 均失败 |
| S2 通用 locale 和 metadata | 冻结 releaseLocales／buildPages；统一 metadata、路径、菜单／feed／404／OG；同批泛化机器 URL、输出、清理、schema 与检查器 | S1；仅中英真实构建保持兼容，第三语 fixture 证明无二元分支和资源覆盖；不留下半泛化提交 |
| S3 自然表达所需契约 | 稳定 section IDs、历史别名、任务／事实契约、显式 locale 链接重写 | S2；允许标题自然改写但不能丢任务、条件或锚点语义 |
| S4 质量流水线 | 构建前审核与构建后证明分离；语言 brief、协议例外、查询夹具、代理评审与定向失效 | S3；真实中英问题与五类西语样本校准；无审校哈希自指循环 |
| S5 生成和资源收尾 | 完整候选构建、撤回残留清理、当前语词典、数量短语和静态 AI 模板；验证 S2 的机器流水线与 S4 证明 | S2–S4；无草稿泄漏，交互／无脚本有效；机器内容同源，证明不改变已验收 artifact |
| S6 中英共性发布 | 全量静态检查、浏览器、公开包消费者、机器读取及生产核验 | S0–S5；68 HTML、62 Markdown、2 llms 和既有接口通过；与新增语言分开记录 |
| S7 六语分批交付 | 按 M2–M5，每语内容与 releaseLocales 快照独立提交，完成源审核后构建并证明 | 每语四轴审核与机器任务完整，直接发布同一 artifact，线上验证后更新发布记录 |
| S8 维护移交 | 变更影响规则、来源记录、观测窗口和故障恢复说明 | 已发布语言纳入所有常规检查，无未关闭严重问题 |

S1–S5 可按依赖并行准备，但共享文件按同一集成负责人收敛，先继承机器实现及其检查，再做语言泛化，避免两条工作线同时重写 layout、middleware、生成器或发布脚本。机器实现尚未完成的原有验收由原工作继续闭环；它若先发布成功，S0 更新可信公开基线。中英共性发布与新增语言发布保持独立，内容原稿与共享平台修改分开提交；每次正式产物包含同一版本的正文、菜单、alternate、Pagefind、Markdown、llms、schema 与 manifest。

新增检查脚本可使用 `check:surface`、`check:localization`、`check:search` 等名称；这些是拟新增入口。继续复用现有 `npm test`、`check:ai`、`check:docs`、`check:content`、`check:examples`、`check`、`build`、`check:site`、`generate:machine`、`check:machine`、`check:schema`、`check:gates`、`test:browser` 和 `check:live`。公开包消费者继续验证三 SDK，内容语言相同代码不重复执行八遍。

## 十 发布验收与恢复

每次候选构建对整个 buildLocales（既有语言及本次完整候选）执行路由、source、元数据、链接、锚点、审核状态和静态能力检查。新语言与受影响页面类型执行完整浏览器场景；每个现有语言保留最小切换／搜索／复制回归。共享搜索或交互代码变更时扩展到所有语言的相关完整场景。

固定覆盖：320、390、768、1440px；深浅主题、键盘、200% 文字缩放、减少动画、无 JavaScript；复制成功／失败／恢复；搜索正常／空结果／加载失败；示例翻卡、音频、更新与下载。长文案不裁剪关键操作，代码区可独立滚动。

有意义的负向检查至少包括：旧页连同 registry 一起删除；旧锚点丢失或指错任务；正常页意外 noindex；译文复制英文；pt-BR 路径与 lang 映射错误；日语预期词无法查到；德语导航跳回英文；共享安装版本改变但译文审核未失效；hreflang 缺回链；未完整语言进入 sitemap。

继承机器规格 M01–M10、S01–S05、D01–D04 的适用检查，再补以下多语言夹具；既有提取和安全序列化测试无需另写一套：

| 夹具／故障 | 必须验证的结果 |
| --- | --- |
| 第三语 draft 不一致；planned 缺源；候选不完整 | 第三语不一致和目标候选缺源失败；未选入 releaseLocales 的 planned 不被读取或输出 |
| es／pt-BR 输出路径；撤回未公开候选；残留旧导出 | 不覆盖中文 llms；已撤回语言无孤立机器文件；已公开语言缺失被历史基线拦截 |
| schema 仍只有两语；删除 Markdown alternate；误删 RSS | 构建集合／实体关系检查失败；三类 alternate 与 x-default 互不覆盖 |
| 误译 Optional 或固定头标签；本地文字包含反引号、方括号及 HTML 字面文本 | 固定协议被改时报错；正确本地化内容保持可解析，不引入额外链接或可执行节点 |
| 章节重排、表格改写、旧 alias；代码中出现 marker | 跨语任务语义仍完整；同语导出忠于 HTML；旧链接命中原任务；代码不被误解析为 marker |
| 日语／俄语文本接近及超过 24 KiB | 按 UTF-8 字节计数；合规内容确定性输出；超限明确失败，不截断 |
| 新增构建后证明；随后更改正文或转换器 | 仅增加外部证明不改变原产物；改真实输入后旧证明失效；发布前不靠更新 hash 自行过关 |
| 已公开机器路由丢失、错误 MIME、旧 llms 混入新 Markdown | 历史接口或线上同一 manifest 检查失败；不能只凭首页返回 200 判成功 |

正式部署继续先验证 artifact／manifest，再读取生产的正文、资源及 headers。增加的检查不能被同源错误配置绕过。预览和根路径两套构建均须通过；GitHub Pages 的单一 404 文件保留脚本本地化与无脚本恢复链接。

恢复按阶段处理：

- 发布前发现问题：撤回候选，继续修复，不改变公共 URL。
- 刚发布出现明确工程故障：可应急恢复上一完整 verified artifact／manifest，并记录受影响新 URL 与恢复安排。
- 新语言已稳定公开：优先前滚修复，或回退有问题的共享代码、保留当前语言与内容重新构建。旧整站 artifact 可能删除新增 URL，仅作为严重故障的应急选择。
- 仅搜索点击下降：先排查统计延迟、英文访问转移、同期变化及索引状态，不自动撤下语言。

保留当前与上一完整产物、源码、releaseLocales／verified 发布记录及恢复说明，包含 HTML 和全部机器资源。回滚不能将新 HTML 与旧 llms／Markdown／schema／搜索索引拼接；已公开的新语机器 URL 与 HTML URL 同受保护，也不将整种语言的 URL 批量重定向到首页。

## 十一 搜索和 GEO 效果观测

有后台时记录最近 90 天搜索数据与最近 28 天基线；每批实际发布后在第 14、28、56、84 天按相同窗口复盘。14 天先看抓取与工程故障，后续看查询、点击和任务表现。以同主题全部语言的合计点击判断净新增，不能将英文访问转移到译文的数量当成全站增长。

GEO 继续复用现有协议：每语 6 个非品牌任务问题覆盖三 SDK 的生成与更新，另有 2 个品牌事实问题；共用 intent ID，使用本地自然问法并独立 AI 审校。真实采样固定可访问平台、模型／搜索模式、语言、时间和原始回答，分别记录提及、有效推荐、官方来源引用和事实错误。每次在独立真实会话中提问，不提示项目 URL；提供文档后的代理答题只算事实测试，不能充当自然发现样本。[既有测量协议](geo-measurement-protocol.md)

平台或后台不可用时对应字段为未取得，不能填零、生成模拟回答或宣称观测通过；这不阻止内容与工程发布。Google AI 功能的表现与其他平台分别记录，不把一个平台的规则外推为全部 GEO。

Cloudflare 继续用于已有页面与来源指标；当前没有自定义事件或 UTM 采集能力，复制／下载意向不能从现有 beacon 推断。若以后确需新增事件服务，再独立设计与验收。[Cloudflare FAQ](https://developers.cloudflare.com/web-analytics/faq/)

## 十二 工作量与完成清单

源码 `2c07690` 的只读盘点仍为每语 32 主题；业务词典已由前期 189 个增为 204 个 key，另有框架／搜索覆盖、三套 SDK 提示模板、元数据、RSS、OG、404 及机器输出。六语新增 192 个页面、174 篇长文译文及 180 个机器文本资源；机器正文自动派生，不另计为 174 篇手工翻译任务。前期基线 29 篇英文长文约 13,440 源词、六语约 80,640 源词仅为规模估算；S0 应以最终实施版本重新扫描全部来源和文案。

取消原方案基于母语人工团队的 48–78 人日和 6–8 周估算。西语试点记录实际生成、独立盲读、技术检查、修订轮数、构建和浏览器验收耗时，以及可取得的代理消耗，再据此估算后续语言。优先用语种完成门槛安排执行，不预先保证未经验证的日期或流量。

最终完成检查：

- 中英旧公开面兼容，所有正常页可抓取、可索引且具备预期摘要资格。
- 八语内容、UI、元数据、搜索及三 SDK 使用路径完整；没有 fallback、错语言导航或未关闭严重问题。
- 每语 llms、全部 docs／article Markdown、发现关系和 JSON-LD 与实际 HTML 一致，旧机器接口和 MIME 合同保持。
- 所有页面有真实独立 AI 审校和修订复审记录，检查证据与当前源／依赖一致。
- 源审核与构建后证明分离；新语言按里程碑独立发布已验收的同一 artifact，线上结果与 verified manifest 对应。
- 后续变更有明确影响范围、复核方式和恢复流程；外部搜索表现与 AI 使用验证分开记录。

本轮产出为可实施方案与讨论记录，尚未开始网站代码改造或新语言发布。


## 2026-10-28 实施补充

用户已确认将并行新增的 CSV／AI 教程和中英标题调整纳入本批，按每语 34 主题重新独立审校与验收。上文实施规模采用这一集合；`2c07690` 与实际生产 `732a40f` 的 32 主题盘点保留为历史证据，不修改旧产物来适配候选版本。六语新增规模相应为 204 个正常 HTML、186 篇长文和 192 个机器文本资源。

中英共性改造仍先于西语启用。完整源审校包含页面实际渲染的导航、侧栏、语言菜单、首页任务入口与对应本语言文案。构建后的发布证明另强制关联每语六个使用任务、四个独立 AI 机器读取任务；缺少报告或资源摘要不符时不能发布。

为了让本地独立 AI 报告能在 CI 重建时复核，保留最初实际读取的 manifest 原始字节、输入摘要、配置、发布语言与全部机器资源摘要。只有输入与受审资源字节完全相同、仅 commit／manifest 标识变化时允许明确记录证明复用；当前 artifact 的工程检查仍独立执行。复用不等于声称重新读取过新 URL，也不能复用已改变的代码、依赖或机器正文。
