# Anki Forge 官网多语言一致性优化方案

核验日期：2026-10-26，美国太平洋时间。源码基线：当前工作区，HEAD 为 b0cc49a5。范围：英文和简体中文的全部官网页面、组件、交互、内容入口及发布检查。状态：差异审计与实施方案，尚未修改网站功能或部署。

**目标是同一套官网的两种语言：页面覆盖、组件、顺序、功能、技术事实和操作结果相同，只替换自然语言文案及必要的本地化格式。当前需要同时解决发布版本差异、页面缺失、两套渲染架构和检查规则缺口。**

当前源码有 32 个英文内容页、7 个中文内容页；中文缺少 25 个对应页。即使已有对应页，Header、首页模块和文档功能也不一致。建议以现有完整能力为基线，抽取共享模板，补齐中文，统一文档系统，并把双语一致性加入发布门槛。不能通过删减英文功能达到数量相等。

## 一 线上状态与源码状态

两者必须分别判断。本轮对正式域名发起 27 次只读 HTTP 请求，并重新构建当前源码，检查全部 40 个 HTML 产物。详细响应、路由、浏览器结果和源码校验值见[审计记录](evidence/website-i18n-2026-10-26/audit.json)。

| 核验对象 | 正式域名当前状态 | 当前源码新构建 | 处理要求 |
| --- | --- | --- | --- |
| 首页 | 英文 200；中文 404；英文首页无语言切换入口 | 中英首页均存在 | 核对部署提交、制品和正式域名实际版本 |
| languages、quickstart、node-quickstart、python-quickstart | 英文 200；对应中文均为 404 | 四组中英页面存在 | 已写译文不等于用户可访问 |
| json-to-anki、anki-forge-vs-genanki | 中英文均为 404 | 两组中英页面存在 | 属于生产与工作区的版本差异，不能只归因为国际化路由故障 |
| Examples、Blog 列表 | 英文 200；中文 404 | 只有英文实现 | 同步模板并补中文路由与内容 |
| sitemap | 30 个英文内容 URL，0 个中文 URL | 32 个英文、7 个中文，共 39 个内容 URL | 最终按全站主题清单生成双语集合 |
| 构建与现有检查 | 本轮独立请求已发现上述缺页 | npm run build 和 npm run check:site 均成功；40 个 HTML 包含 1 个 404 | 现有检查通过不能证明多语言一致 |

正式域名核验覆盖了源码登记的全部 7 个中文页面，结果全部为 404。响应时间约为当日 02:20–02:22；读取到的英文页面 Last-Modified 为 2026-10-25 20:29:05 PDT。该时间是响应头信息，不能据此推断失败发生在哪个部署环节。[正式英文首页](https://ankiforge.dev/)、[正式 sitemap](https://ankiforge.dev/sitemap-0.xml)与上述审计记录共同构成生产基线。

本地构建使用项目默认的 /anki-forge 前缀，验证 base path 兼容性；它不是一次正式域名发布。下文的组件差异均基于本轮新构建及当前源码。

## 二 一致性的具体标准

| 必须相同 | 允许随语言变化 |
| --- | --- |
| 页面主题集合、页面类型、栏目层级、模块顺序、导航项目及顺序 | 标题、正文、导航标签、按钮文字 |
| 搜索、菜单、主题切换、示例切换、翻卡、音频、复制、下载、更新演示等能力 | 提示、错误、空状态、复制反馈、无脚本文案 |
| 表格列含义、步骤、后续任务、相关链接和限制条件 | 日期和数字的本地化显示，事实值不变 |
| 包版本、安装命令、API 名、代码逻辑、示例输入、导出文件及性能数据 | 图片 alt、aria-label、title、读屏状态文本 |
| 响应式规则、键盘行为、焦点管理、主题和减少动画行为 | 字体回退、合理换行与行高；不要求不同字形逐像素一致 |
| 同主题链接和操作结果；站内导航默认保持当前语言 | URL 的语言前缀、对应语言的元数据和社交图片文字 |

Anki Forge、Rust、API 标识符、命令及示例教材里的 hola / hello 保持原值。它们属于品牌或示例数据；外围的“问题”“显示答案”“下载”等界面文本需要翻译。技术例外应有明确范围，不能用“允许英文术语”掩盖整句界面文案漏译。

同一种页面在两种语言中必须使用同一模板。首页与文档可以有各自的页面布局，但不能因为切换语言就从完整文档界面变成精简文章页。

## 三 页面覆盖差异

| 页面类型 | 英文 | 中文 | 缺少中文对应页 |
| --- | ---: | ---: | ---: |
| 首页 | 1 | 1 | 0 |
| 文档 | 27 | 6 | 21 |
| 示例页 | 1 | 0 | 1 |
| 博客列表 | 1 | 0 | 1 |
| 博客文章 | 2 | 0 | 2 |
| 正常内容页合计 | 32 | 7 | 25 |

中文内容主题覆盖率为 21.9%，文档主题覆盖率为 22.2%。统计不包含 404、RSS、robots、sitemap、搜索索引和下载资源。按当前主题数，完成后应有 **32 对、64 个正常内容 URL**；后续新增页面时按清单动态计算，不将 64 写死。

数量来源为[英文导入文档映射](../../website/scripts/content-links.mjs)、3 篇[手写文档](../../website/src/content/docs/docs/)、[当前七主题清单](../../website/scripts/core-pages.mjs)和[博客内容集合](../../website/src/content/blog/)。

现有 6 对文档是 languages、quickstart、node-quickstart、python-quickstart、json-to-anki、anki-forge-vs-genanki。其余需要补齐：

| 分组 | 缺少的中文路由 | 对应英文内容 |
| --- | --- | --- |
| 入门 | /zh-cn/docs/ | 文档总览 |
| 入门 | /zh-cn/docs/installation/ | 项目集成与安装 |
| 入门 | /zh-cn/docs/concepts/ | 核心概念 |
| 任务指南 | /zh-cn/docs/cards/ | Basic 与 Cloze |
| 任务指南 | /zh-cn/docs/media/ | 图片与音频 |
| 任务指南 | /zh-cn/docs/custom-notetypes/ | 自定义笔记类型 |
| 任务指南 | /zh-cn/docs/templates/ | 模板包 |
| 任务指南 | /zh-cn/docs/image-occlusion/ | 图片遮挡 |
| 任务指南 | /zh-cn/docs/updates/ | 更新与分发 |
| 语言指南 | /zh-cn/docs/rust-guide/ | Rust 编写指南 |
| 语言指南 | /zh-cn/docs/python-migration/ | Python API 迁移 |
| 语言指南 | /zh-cn/docs/genanki-migration/ | 从 genanki 迁移 |
| API 与保证 | /zh-cn/docs/api/ | 三语言 API 总览 |
| API 与保证 | /zh-cn/docs/rust-api/ | Rust API |
| API 与保证 | /zh-cn/docs/node-api/ | Node / TypeScript API |
| API 与保证 | /zh-cn/docs/python-api/ | Python API |
| API 与保证 | /zh-cn/docs/build-guarantees/ | 构建与输出保证 |
| API 与保证 | /zh-cn/docs/compatibility/ | 兼容性与版本 |
| 帮助 | /zh-cn/docs/troubleshooting/ | 排错 |
| 帮助 | /zh-cn/docs/python-diagnostics/ | Python 诊断 |
| 帮助 | /zh-cn/docs/development/ | 从源码构建 SDK |
| 示例 | /zh-cn/examples/ | 全部卡片与更新示例 |
| 博客 | /zh-cn/blog/ | 博客列表 |
| 博客 | /zh-cn/blog/your-first-anki-deck/ | 第一份牌组教程 |
| 博客 | /zh-cn/blog/stable-ids-and-deck-updates/ | 稳定 ID 与牌组更新 |

全部补齐意味着正文信息和操作入口也对应，不能只创建中文 URL、翻译标题或渲染英文 fallback。

## 四 组件与功能差异

### 全局导航和语言切换

| 项目 | 当前差异 | 同步方案 |
| --- | --- | --- |
| 桌面 Header | 英文 Guide、Examples、Blog；中文只有“指南” | 共享导航清单：指南、示例、博客；两语只替换标签与同主题地址 |
| 指南入口 | 英文进入 /docs/ 总览，中文进入 /zh-cn/docs/languages/ | 新增中文总览后统一入口角色 |
| 导航高亮 | 中文用 languages 路径作 startsWith 判断，其他中文文档不匹配 | 按页面所属 sectionId 判断当前栏目 |
| 移动菜单 | 英文三项、中文一项；同样缺示例和博客 | 桌面与移动读取同一导航模型 |
| 文档 Header | 英文使用 Starlight Header，中文使用官网 Header；品牌字标、工具布局不同 | 同主题共享文档壳，品牌与公共工具从共享组件接入 |
| 语言切换 | 仅七主题出现；其余英文页面无切换入口 | 全站 pageId 配对，所有内容页可切到同一主题 |
| URL 状态 | 从首页 #ai-quickstart 切换语言后锚点消失；href 未携带 query | 使用稳定章节 ID，保留可对应的锚点及页面支持的查询状态 |
| Footer | 中文“文章（英文）”仍去英文 Blog；RSS 也是英文 | 列表、详情、feed 均按当前 locale 解析；MIT/GitHub 保持共享外部目标 |

直接证据：[Header](../../website/src/components/Header.astro) 第 9–30 行、[LanguageSwitcher](../../website/src/components/LanguageSwitcher.astro) 第 4–7 行、[Footer](../../website/src/components/Footer.astro) 第 10 行。浏览器实测确认锚点丢失，但深色主题在中英首页切换后保留，主题持久化可继续复用。

### 首页内容与交互

| 能力或模块 | 英文新构建 | 中文新构建 | 同步要求 |
| --- | --- | --- | --- |
| Hero 与卡片合成动画 | 存在 | 存在，已传 locale | 保留共享组件、动画逻辑和素材 |
| 三语言安装卡片 | Rust、Node、Python | Rust、Node、Python | 继续共享版本、运行环境与命令 |
| AI 提示生成器 | 三种 SDK、任务输入、提示与安装复制 | 相同功能及中文反馈 | 修正中文提示里的英文 quickstart URL |
| 性能图表 | 5 个工作负载 | 同样 5 个工作负载 | 保持同一数据源、指标、单位与限制说明 |
| ExampleWorkbench | 4 个示例 Tab、源码、卡片预览 | 整块缺失 | 补齐同一工作台，支持同样的键盘切换 |
| 卡片翻转与音频 | 4 个 CardPreview，1 个音频控件 | 缺失 | 接入相同预览、翻转、媒体播放逻辑 |
| 更新演示 | 更新/重置、稳定 ID、比较报告、v1/v2 | 缺失 | 接入 UpdateDemo，翻译动作与动态状态 |
| 导出前验证与模板介绍 | 两个功能介绍及深入链接 | 缺失 | 共用模块与对应语言的目标页面 |
| 带 download 属性的入口 | 6 个：4 个示例包加 v1/v2 | 0 个 | 使用同一制品、文件名与校验来源 |
| 中文独有工作流区 | 无独立对应区 | “从第一份牌组到下一次更新” | 将有用的 JSON/选型入口整合进两语共享的后续操作区 |

最后两列比较的是首页能力，不能将两端都有的 Hero 动画当作已经拥有交互工作台。源码位置：[英文首页](../../website/src/pages/index.astro) 第 40–57 行、[中文首页](../../website/src/pages/zh-cn/index.astro) 第 34–40 行。

建议统一首页主序列为：Hero → 安装 → AI 生成器 → 性能 → 示例工作台 → 更新演示 → 功能与后续操作。中文现有 JSON/选型入口进入共同的后续操作数据，避免继续保留另一套独立页面结构。

### 文档阅读与内容导航

| 项目 | 英文 | 中文 | 同步要求 |
| --- | --- | --- | --- |
| 文档布局 | Starlight，侧栏、正文、页内目录 | CoreGuide，简化双栏 | 全部进入同一 Starlight 文档系统 |
| 文档导航 | 27 篇、5 组，支持折叠与当前位置 | 6 篇平铺，加英文 API 链接 | 同一个主题树、排序与折叠规则 |
| 搜索入口 | 有，可实际搜索 JSON 等内容 | 无界面入口 | 两语提供相同搜索入口、快捷键和结果操作 |
| 搜索索引 | 英文索引存在 | 中文索引也存在 | 复用索引能力，不能误判为中文完全无法被索引 |
| 页内目录 | 有 | 无 | 相同目录生成规则与稳定章节 ID |
| 上一页与下一页 | 有 | 无 | 依据共享主题顺序生成 |
| 编辑入口 | 有 | 无 | 指向对应语言的权威源文件 |
| 移动文档导航 | 文档菜单和页内目录 | 顶部多行平铺链接占用首屏 | 使用同一移动文档导航方式 |
| 代码呈现与复制 | Rust quickstart 的自定义程序块无复制，两个命令块有复制 | 同页 3 个代码块都有复制，但提示仍为英文 | 共用代码渲染规则，保留两端较完整的复制能力并本地化反馈 |

中文已有 data-pagefind-body。本轮直接调用中文 Pagefind 索引，查询“牌组”得到 7 个结果；英文搜索 UI 查询 JSON 返回英文文档及章节链接。因此准确的差异是**中文缺少搜索入口和同等搜索体验**。[SiteLayout](../../website/src/layouts/SiteLayout.astro)、[CoreGuide（审计时版本）](https://github.com/morehardy/anki-forge/blob/b0cc49a5d9e621d197658cdcc756404cccac5b8b/website/src/layouts/CoreGuide.astro)、[文档配置](../../website/astro.config.mjs)及审计记录可复核这些结果。

生成器已经为中文写出 editUrl，但独立 coreGuides collection 与 CoreGuide 没有消费它。统一布局时应接通已有元数据。[内容生成器](../../website/scripts/prepare-site.mjs)第 60–66 行、[内容集合](../../website/src/content.config.ts)第 10–13 行。

### 文案 辅助页面和视觉细节

| 项目 | 当前差异或问题 | 同步方案 |
| --- | --- | --- |
| AI 提示引用 | 中文提示的“快速开始文档”仍为 /docs/ 地址 | 在 renderPrompt 中按 SDK 的 pageId 和 locale 获取公开文档 URL |
| 动态交互文案 | 翻卡、更新、重置、读屏状态直接在 site.ts 写英文 | 组件注入完整消息对象；服务端初始态与客户端变更态使用同一词典 |
| 辅助文本 | 中文 GitHub aria-label 仍为 GitHub repository；中文代码复制提示仍为 Copy to clipboard / Copied! | 校验可见文字、title、alt、aria-label、aria-live 和错误状态 |
| 博客 | 无中文列表、详情和卡片；分类展示与日期固定英文 | 共用列表/详情/PostCard；按 translationKey 配对，日期格式接收 locale |
| RSS | 只有英文 feed、英文条目链接和 language=en | 共用生成器输出两语 feed；Footer 和 head discovery 随页面语言切换 |
| 404 | 英文错误页和英文恢复链接 | 共用错误页内容模型，验证实际托管平台的错误路由处理 |
| SEO | 只有七主题被纳入手工语言配对；两套布局分别维护 head | 全站统一 title/description/canonical/hreflang/OG 的来源与输出规则 |
| 社交图 | 共用含英文宣传文字的 og.png，中文只翻译 alt | 同一构图生成双语文字版本，品牌、示例和技术事实不变 |
| 极窄屏 Header | 320px 时英文页面宽度达到 338px，Menu 被裁切；中文“菜单”分成两行 | 用共同的紧凑布局处理两种语言，保证菜单和语言控件都可见 |
| 中文 Hero 排版 | 1440px 截图中“变成 Anki 牌组。”末尾“组。”单独成行 | 调整共享文字区域和断行规则，避免短词孤行；不另造中文首页布局 |

源码入口：[AI 渲染器](../../website/src/lib/ai-quickstart.mjs)第 21 行、[交互脚本](../../website/src/scripts/site.ts)第 54–85 行、[日期格式](../../website/src/lib/site.ts)第 9–10 行、[RSS](../../website/src/pages/rss.xml.ts)、[404](../../website/src/pages/404.astro)、[社交图生成器](../../website/scripts/social-image.mjs)。

### 全部现有组件的处理范围

以下覆盖 src/components 的全部 15 个 Astro 组件，避免只修 Header 而漏掉内部能力。

| 组件 | 实施处理 |
| --- | --- |
| Header、Footer、LanguageSwitcher | 共享导航、路由解析和语言状态，覆盖桌面与移动 |
| HomeShowcase、PerformanceEvidence | 保留共享实现，将散落文案归入统一词典，技术数据仍共用 |
| AiQuickstart、InstallCommand、CopyBlock | 保留已对齐交互，修正文档目标，统一消息与缺失翻译检查 |
| ExampleWorkbench、CardPreview、UpdateDemo | 接入中文页面，补全部文案和动态状态，不复制行为脚本 |
| PostCard | 添加 locale 与稳定文章配对，翻译阅读入口、分类和无障碍文本 |
| CodeBlock | 统一代码渲染、复制规则与辅助标签，减少与 Markdown 渲染器的能力差异 |
| BrandMark、Icon | 继续复用同一图形资源；组件外承担含义的文本由词典提供 |

该表按组件名逐一覆盖；实际数量应由目录扫描复核，而非作为未来新增组件的固定上限。

## 五 已有译文也需要同步内容

只补缺页不能达到“只是语言变化”。现有六组文档还要对齐内容结构、技术说明和下一步动作。

| 页面 | 已确认差异 | 需要同步 |
| --- | --- | --- |
| 选择开发语言 | 英文表四列，中文三列，缺 public package registry 链接列；英文 Rust→Node→Python，中文 Rust→Python→Node | 使用共享列定义、SDK 顺序、包链接及任务入口 |
| Rust quickstart | 中文缺英文的 Basic APKG 下载入口，后续任务集合也更少 | 同步下载、媒体、模板、遮挡、API 等后续入口；共享同一完整程序 |
| Node quickstart | 中文缺 ESM/CommonJS、TypeScript 声明说明；少 Node API、媒体、模板、排错等后续入口 | 补齐技术信息及同语目标链接 |
| Python quickstart | 中文少 Python API、诊断、媒体、模板、遮挡、genanki 迁移等入口 | 使用相同的后续任务集合 |
| JSON 生成与更新 | 程序、素材和 SDK 锚点已有共享校验；中文 update/media/API 链接仍进入英文 | 保留正确的技术来源，补同语内容并重写链接 |
| genanki 比较 | 主要主题已经对应；中文兼容性、迁移链接仍进入英文 | 同步引用路径，保持相同适用范围与限制，不扩大能力承诺 |

来源：[英文语言选择](../../website/src/content/docs/docs/languages.mdx)、[英文 Rust 入门](../../website/src/content/docs/docs/quickstart.mdx)、[Node 入门](../node/quick-start.md)、[Python 入门](../python/quick-start.md)与[六篇中文源文件](../zh-cn/)。

长文允许地道改写，但章节所讲的任务、步骤、表格列、警告条件、代码与行动入口需要对应。不能用逐段字数或整页文本相似度替代技术审校。

## 六 结构性原因

1. **翻译清单只表达七个主题。** corePages 同时决定中文路由、导航、语言切换和 hreflang；它不是全站页面模型。
2. **首页独立编排。** 两个 index.astro 分别选择模块；共用几个组件不能保证整页相同。
3. **文档生成后走两条路径。** 英文进入 Starlight docs collection，中文进入 src/generated/zh-cn，再由 CoreGuide 单独渲染。
4. **路由解析缺少当前语言。** content-links 按原文件路径找目标；中文引用英文源路径自然得到英文 URL。
5. **文案与行为交错。** 文案分散在 Astro、TypeScript、配置和生成数据中，静态文字翻译后，客户端状态仍可能回到英文。
6. **旧发布策略被测试固定。** check-site.mjs 第 81 行要求中文页面恰好为原七页；check-live-site.mjs 第 49–50 行要求中文 Rust API 返回 404；content-links.test.mjs 还固定了中文转英文 API 的行为。

依据：[core-pages](../../website/scripts/core-pages.mjs)、[prepare-site](../../website/scripts/prepare-site.mjs)、[content-links](../../website/scripts/content-links.mjs)、[构建产物检查](../../website/scripts/check-site.mjs)、[线上检查](../../website/scripts/check-live-site.mjs)、[链接测试](../../website/scripts/content-links.test.mjs)。

旧 [GEO 方案](2026-10-25-geo-optimization-plan.md)采用的是“核心七主题双语、其余继续英文”的阶段范围。本次需求扩大为全站一致，实施时应同步更新 [website README](../../website/README.md)和[文档维护说明](../documentation.md)，保留旧方案作为历史记录。

## 七 推荐架构

### 全站主题清单加共享模板

建立一个覆盖所有页面的清单，统一维护 pageId、页面类型、导航分组和顺序、各语言 source/path/title/description，以及该主题必须具备的组件能力。博客通过稳定 translationKey 接入，不能因中英标题不同失去配对。

| 模型 | 管理内容 | 消费方 |
| --- | --- | --- |
| locales | 内部语言 en / zh-CN、URL 前缀、显示名称、格式化规则 | 页面、词典、元数据和路径工具 |
| pages | 稳定主题 ID、两语内容来源与路径、所属栏目 | 路由生成、切换器、sitemap、检查 |
| navigation | 全局导航、文档树、页脚、后续任务顺序 | Header、移动菜单、侧栏、前后页、Footer |
| messages | 可见及辅助 UI 文案、动态状态和变量插值 | Astro 与客户端组件 |
| capabilities | 同主题模块序列、动作、示例和下载引用 | 共享模板及行为验收 |

清单可以由手写页面配置与博客集合汇总而成，但只能有一个解析入口。构建时还要反向扫描实际源文件与产物，发现漏登记页面；只检查一份清单自己内部一致仍会漏项。

首页、Examples、Blog 列表和文章分别使用一个共享模板，语言路由只负责传入 locale 和内容。保留现有英文 URL；中文统一使用 /zh-cn 前缀。组件不能再通过 locale 分支增删模块、改变功能或重排导航。

### 统一文档系统

推荐保留 Astro 与 Starlight，使用英语 root locale 和 zh-cn locale。英文文档继续位于 src/content/docs/docs/；中文对应文档生成到 src/content/docs/zh-cn/docs/，仍以 docs/zh-cn 的人工维护文件为内容来源。[Starlight 国际化说明](https://starlight.astro.build/guides/i18n/)支持 root locale 和多语言文档路由。

完整中文内容就绪后，在同一个切换变更中移除独立 coreGuides collection、CoreGuide 布局和中文 docs catch-all，避免同一个 URL 被两套系统生成。侧栏、目录、前后页、编辑链接及代码表现由同一个文档壳提供。

公共品牌、导航、语言和主题工具从共享组件接入；文档特有的搜索与移动侧栏通过 Starlight 适配组件保留。官方支持覆盖并组合现有组件，无需复制整个文档框架。[Starlight 组件覆盖说明](https://starlight.astro.build/guides/overriding-components/)

Starlight 对缺译内容有 fallback 机制。因此启用 locales 之前必须检查真实双语源文件齐全，生成后再检查语言、正文来源和路由集合；不能把 fallback 生成的英文正文作为中文完成量。搜索继续使用现有 Pagefind，同时验证两语结果和入口。[fallback 说明](https://starlight.astro.build/guides/i18n/#fallback-content)、[搜索说明](https://starlight.astro.build/guides/site-search/)

### 统一文案与技术事实

短 UI 文案进入类型或 schema 约束的字典：两种语言的 key 集合、插值参数和必要字段必须一致。选择器值、组件 ID 和状态 ID 保持稳定，只翻译显示值。

长文继续分语言维护 Markdown/MDX，不把每个段落拆成 JSON。两语共用章节 ID、步骤/表格/相关链接契约，记录译文对应的内容修订；技术事实变化时触发复审。自动检查负责发现缺项，人工审校负责语义与中文质量。

继续复用 docs/ai-quickstart.json、可执行示例、source marker、生成媒体与性能展示数据。安装版本、程序和 APKG 不分别维护两份。代码注释需要翻译时，只允许不改变程序语义的展示层处理，并保持可执行源校验。

### 统一链接与元数据

所有站内内容链接通过 pageId + locale + 可选 sectionId 解析；共享下载、品牌资源、包 registry 和外部仓库链接维持共同目标。中文页面内部不再依靠直接拼接 /zh-cn 或引用某个英文源文件来决定导航。

语言切换落到同一主题，保留对应章节。现有中英文自动生成的标题锚点不同，需要稳定 ID 或明确映射；已经公开的旧锚点保留兼容入口，不能盲目携带失效 hash。主题偏好继续共享；任务输入保持既有的本地临时处理原则，不为切换语言擅自加入持久化或上报。

统一 head 生成来源：每页自引用 canonical、互相对应的 hreflang、正确 lang/OG locale、翻译后的 title/description/图片 alt。接入 Starlight 原生 locales 后要去掉重复的手工 alternate 输出，避免双份 hreflang；保留每页一个 analytics beacon。

## 八 文件改造范围

| 工作包 | 现有入口 | 具体动作 |
| --- | --- | --- |
| 页面与路由模型 | scripts/core-pages.mjs、scripts/content-links.mjs、src/lib/site.ts | 扩为全站主题清单；添加 locale 路由解析、导航模型和格式化工具 |
| 首页及公共组件 | 两个首页、Header、Footer、LanguageSwitcher、HomeShowcase、PerformanceEvidence | 抽单一首页模板，统一导航与模块顺序，整理文案 |
| 交互组件 | ExampleWorkbench、CardPreview、UpdateDemo、AiQuickstart、site.ts、lib/ai-quickstart.mjs | 补中文接入，集中状态文案，修复提示中的文档 URL |
| 文档内容与壳 | docs/zh-cn、astro.config.mjs、content.config.ts、prepare-site.mjs、CoreGuide、中文 docs 路由、docs.css | 补 21 篇、校准 6 篇；统一 Starlight 路由、侧栏、目录、代码及编辑链接 |
| 示例与博客 | examples/index、blog/index、blog/[...id]、PostCard、blog collection | 共用模板，补中文内容、日期/分类翻译和同主题配对 |
| 辅助资源 | rss.xml.ts、404.astro、SiteLayout、route-middleware、social-image.mjs | 双语 feed、正确错误恢复、统一元数据、双语社交图 |
| 检查与维护 | check-site、check-live-site、content-links.test、网站 workflow、README、documentation.md | 替换七页限制与中文 API 404 断言，新增完整覆盖和能力对比 |

上述路径均相对于 website，docs/zh-cn 与文档维护文件除外。新目录可选 src/i18n、src/lib/routes 和共享页面模板目录，具体命名以实现时减少重复为准。不要手工修改 src/generated、public/generated 或导入后的文档产物。

## 九 实施顺序与完成条件

| 阶段 | 工作与依赖 | 阶段出口 |
| --- | --- | --- |
| 0 确认发布基线 | 核对当前代码、Actions 制品和正式域名版本；确认现有七个中文页及两个新英文页未出现的原因 | 能说明线上使用的版本与待发布内容，不以本地成功代替线上可用 |
| 1 建共享模型 | 建全站主题/语言/导航清单及翻译 schema；把 32 对目标和能力差异变成可运行检查 | 能准确报告缺页、缺模块、缺翻译；现有英文 URL 和能力保留 |
| 2 统一公共页面 | 共用 Header/Footer/首页/示例/博客模板，接通交互词典；修复 320px Header | 预览中两语同类页面组件和动作一致，首页工作台/更新/下载齐全 |
| 3 补齐内容 | 依赖阶段 1 的主题与章节模型；可与阶段 2 并行；完成 21 篇文档、示例/博客和原 6 篇校准 | 全部主题有真实译文，技术事实和后续任务经过复核 |
| 4 切换文档系统 | 依赖内容齐全；迁入共同 Starlight 壳，同步链接、RSS、SEO、404 与维护规则 | 所有主题均可访问，文档导航与搜索一致，无重复路由或 fallback 冒充译文 |
| 5 全量验收与发布验证 | 对完整制品执行结构、功能、内容与视觉检查；按现有流程发布后扫描正式站点 | 全部登记内容页在正式域名返回正确内容，线上结果与制品一致 |

优先顺序是发布基线 → 页面和模板结构 → 缺失功能与内容 → 隐藏状态及视觉细节。表中的所有工作均属于最终完成范围；后排项目不是可以留作“已完成后再补”的可选项。

迁移在分支预览中逐步完成。任何阶段都不能靠缩减英文内容、将中文导向英文、只改测试预期，或生成缺译 fallback 来通过验收。分阶段合并时可以先增加共享基础设施，但向用户发布的完整双语入口必须满足对应内容和功能要求。

### 迁移中需要特别处理的边界

| 边界 | 处理方式 |
| --- | --- |
| Starlight 与独立中文路由冲突 | 切换时同时移除旧生成目录/集合/路由，检查重复 URL 与残留产物 |
| 自动锚点随翻译变化 | 稳定 sectionId、保留旧锚点别名；检查跨语言深链接往返 |
| 静态主机的 404 | [GitHub Pages 的自定义错误入口](https://docs.github.com/en/pages/getting-started-with-github-pages/creating-a-custom-404-page-for-your-github-pages-site)为站点 404.html；新增 /zh-cn/404.html 不代表未知中文路径会使用它。需验证按请求路径本地化的实际方案；若要求无脚本也只输出对应语言，必须具备路由层处理能力，不能把客户端切换当作已满足此条件 |
| 翻译后的布局长度 | 共享自适应布局覆盖长英文和中文，不通过隐藏按钮或减少导航项适配 |
| 生成与手写源混淆 | 源文件、可执行代码和生成器分别拥有明确职责；编辑链接回到真实源文件 |
| 发布与源码脱节 | 发布后独立读取完整路由清单及代表组件，不能只检查首页 200 |

## 十 持续同步与验收门槛

新增页面或功能时，在同一个变更中维护共享结构、两语内容和验证用例。文案 key 新增但任一语言缺值、内容主题缺对应来源、必需能力只出现在一种语言时，构建失败。原有“仅七页中文”和“中文 Rust API 必须 404”的断言替换为全站配对检查；随机不存在的路径仍应验证真实 404。

| 验收层 | 必须通过的检查 |
| --- | --- |
| 页面覆盖 | 扫描所有实际内容源与 dist；每个 pageId 均有 en / zh-CN 真实内容，集合完全相等，正常内容不混入 404 |
| 结构 | 归一化语言前缀与文案后，组件 ID、模块顺序、导航 ID、动作、表列和下载引用一致；不使用整页 HTML 字符串相等作为标准 |
| Header 与 Footer | 两语同类页面具有相同项目与顺序、正确当前栏目；移动菜单同样完整；语言切换可往返同一主题 |
| 首页 | 四类示例 Tab、翻卡、音频、更新/重置、比较报告、六个下载入口均可用；版本、命令、五组性能值一致 |
| 交互状态 | 初始、选择、成功、拒绝/失败、重置、禁用、无脚本状态均覆盖；aria-live 反馈随语言变化 |
| 代码与内容 | 同一技术源、代码块、安装版本、限制条件与下一步动作对应；复制出的代码不因翻译损坏 |
| 内部链接 | 默认保持 locale；href/hash 指向实际同语页面和章节；外站及共享下载按资源类型区分 |
| 搜索 | 英文和中文代表词分别得到正确语言结果；可打开、可用键盘关闭、空结果/加载失败文案正确；404 不进入结果 |
| 文档体验 | 两语都有相同侧栏、页内目录、上一页/下一页、编辑入口和代码复制规则；移动端交互一致 |
| 视觉与无障碍 | 至少覆盖 320、390、768、1440px；浅色/深色/系统、键盘、200% 缩放、减少动画和无脚本；不裁切语言/菜单控件，不出现整页横向滚动 |
| 元数据和 feed | 自引用 canonical、双向 hreflang、正确 html lang/OG locale；sitemap 全覆盖；RSS 条目与发现链接保持语言，社交图对应 |
| 部署 | 根路径生产配置和 /anki-forge 前缀构建均通过；正式域名逐一验证全部内容 URL，校验代表组件、语言与元数据 |

自动化分两层：所有内容页执行轻量的路由、DOM 能力、链接及文案检查；首页、示例、博客列表/详情、文档和错误页按同一套参数化浏览器场景执行两种语言。视觉基线分别维护，评审关注结构和操作完整，不比较中英字形像素是否相等。

人工内容审校重点检查 API 行为、限制、表列、代码与下一步任务，以及中文的自然程度。源内容更新后应标记对应译文待复审；该标记在发布前必须清除，不能静默沿用旧译文。

现有 CI 已有 build → check:site → 上传制品 → 部署 → check-live 的流程，直接增加上述门槛即可，不需要另建一套发布系统。[网站工作流](../../.github/workflows/website.yml)

## 十一 本轮验证与证据

已完成当前源码构建、现有站点检查、40 个 HTML 的结构清点，以及中英首页/入门文档的桌面和移动浏览器比较。补验包括 320/768px 首页、英文搜索 UI、中文索引检索、AI 输入与 SDK 切换、翻卡、更新、主题保留、语言切换和禁用 JavaScript 的首页。

已对齐的部分可以保留：两语 AI 均有 3 个 SDK 选项；无脚本时均显示 3 份提示模板，9 个复制按钮均禁用；成功/回退的中文提示已提供；安装版本及性能数据来自共享源。复制成功与拒绝在本轮用模拟剪贴板结果检查 UI 反馈，未作为操作系统剪贴板权限的验证结论。

这份方案没有把规划中的全浏览器、缩放或全部错误状态矩阵描述为已经通过；其完成条件见上一节。

| 对比证据 | 英文 | 中文 |
| --- | --- | --- |
| 1440px 首页 Header 与 Hero | [截图](evidence/website-i18n-2026-10-26/home1440.png) | [截图](evidence/website-i18n-2026-10-26/zh-cn-1440.png) |
| 1440px 同主题文档 | [截图](evidence/website-i18n-2026-10-26/docs-quickstart-1440.png) | [截图](evidence/website-i18n-2026-10-26/zh-cn-docs-quickstart-1440.png) |
| 390px 展开的首页菜单 | [截图](evidence/website-i18n-2026-10-26/home390-menu.png) | [截图](evidence/website-i18n-2026-10-26/zh-cn-390-menu.png) |
| 390px 文档首屏 | [截图](evidence/website-i18n-2026-10-26/docs-quickstart-390.png) | [截图](evidence/website-i18n-2026-10-26/zh-cn-docs-quickstart-390.png) |
| 320px Header 边界 | [菜单被裁切](evidence/website-i18n-2026-10-26/home320.png) | [菜单换成两行](evidence/website-i18n-2026-10-26/zh-home320.png) |

机器可读证据统一保存在 [audit.json](evidence/website-i18n-2026-10-26/audit.json)，包含生产响应状态与哈希、本地路由/组件数量、搜索和交互结果、截图索引及源码校验值，便于实施后按同一基线复核。
