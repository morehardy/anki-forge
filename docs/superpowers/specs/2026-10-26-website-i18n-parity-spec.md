# Anki Forge 官网中英文全站一致性规格

日期：2026-10-26。范围：英文与简体中文官网的全部内容页面、公共组件、文档体验、交互与发布验收。本规格描述待实现行为，不表示网站已完成改造或发布。

## Problem Statement

访客切换官网语言时，会遇到不同的网站功能和内容覆盖。英文首页提供指南、示例和博客入口，以及卡片工作台、音频、下载和更新演示；中文页面只有部分入口和模块。英文文档包含搜索、完整侧栏、页内目录、前后页与编辑入口，中文同主题文档使用另一套精简布局。使用中文的访客无法完成与英文访客相同的学习和操作流程。

2026-10-26 审计时，当前源码包含 32 个英文正常内容页和 7 个中文内容页，缺少 25 个中文对应页，其中 21 篇是文档。已有译文仍存在表格列、SDK 顺序、技术说明、下载和下一步任务的差异；中文 AI 提示也会引用英文入门文档。部分界面文本只在初始状态翻译，动态反馈或辅助文本仍依赖英文硬编码。

同次生产核验中，源码登记的全部 7 个中文页面返回 404，两篇新英文指南也返回 404；正式 sitemap 包含 30 个英文 URL、没有中文 URL。当前源码重新构建并通过既有站点检查，说明发布基线与工作区不同，而且现有检查并未要求全站双语一致。现有规则还将中文限制为原七个主题，必须随新目标一起替换。

用户需要的是同一套官网的两种语言：组件、功能、内容结构和技术事实保持一致，只改变语言及必要的本地化格式。维护者也需要在新增页面或功能时自动同步两种语言，避免再次出现中文精简版。

## Solution

以全站主题和现有完整能力为基线，统一页面模板、公共组件、导航、路由解析和交互行为，为英文与简体中文分别提供完整文案与正文。文档统一进入现有 Starlight 系统，继续使用 Pagefind 搜索；保留现有英文地址和已存在的中文主题地址。

完成当前缺少的 25 个中文内容页，并校准现有 6 组文档的章节、表格、技术说明和操作入口。按审计时的主题数量，最终覆盖 32 对、64 个正常内容 URL；该数量是基线，后续必须根据真实内容集合动态验收。404、RSS、sitemap、搜索索引和共享下载资源另行验证。

用户在任一语言下都应获得相同的 Header、Footer、栏目和模块顺序、示例、搜索、复制、下载与更新演示；切换语言保持同一主题及可对应章节。允许自然语言、日期显示、字体回退和合理换行不同，但不能减少模块、信息或动作。技术事实与产物来自同一来源，不因语言改变。

把这一要求加入既有构建和发布流程：缺译、功能不齐、非预期跨语言跳转或生产缺页均阻止完成验收。保留两端已有的完整能力，包括中文现有较完整的代码复制，不通过删减任一语言功能达到一致。

## User Stories

1. As a Chinese-speaking visitor, I want every English content topic to have a complete Chinese counterpart, so that my preferred language does not limit what I can learn or do.
2. As an English-speaking visitor, I want all existing capabilities to remain available during the multilingual migration, so that improving Chinese coverage does not reduce my experience.
3. As a visitor, I want both languages to use the same page structure and module order, so that changing language does not change how the website works.
4. As a visitor, I want the header to expose the same guide, examples and blog destinations in both languages, so that I can discover the same parts of the product.
5. As a documentation reader, I want the guide entry to open the documentation overview in either language, so that the same navigation action has the same purpose.
6. As a visitor, I want the current navigation section to remain correctly highlighted on every page, so that I know where I am.
7. As a mobile visitor, I want the same complete navigation in both mobile menus, so that smaller screens do not hide language-specific capabilities.
8. As a visitor on a narrow screen, I want the menu, language switch and theme controls to remain visible and usable, so that I can navigate without horizontal scrolling or clipped controls.
9. As a bilingual reader, I want switching languages to retain the current topic, so that I do not have to find the page again.
10. As a reader following a deep link, I want the language switch to retain the corresponding section and supported URL state, so that I can continue from the same context.
11. As a visitor, I want my selected theme to survive page and language changes, so that my reading preference remains consistent.
12. As a Rust, Node or Python developer, I want installation choices in the same order with the same package versions and runtime requirements, so that language does not change setup guidance.
13. As a developer evaluating packages, I want the language-selection table to contain the same registry links and columns, so that I can compare the same information.
14. As a developer using an AI coding assistant, I want the same SDK tabs and local task input in both languages, so that I can generate a useful brief in my preferred language.
15. As a Chinese-speaking developer, I want the generated AI brief to link to the corresponding Chinese quickstart, so that the workflow remains in my selected language.
16. As a developer copying a command or prompt, I want success and failure feedback in the current language, so that I understand whether copying succeeded and what to do next.
17. As a visitor with JavaScript disabled, I want equivalent readable templates, commands, content and downloads in both languages, so that essential information remains accessible.
18. As a visitor evaluating card types, I want Basic, Cloze, media and Image Occlusion examples in both languages, so that I can explore the same supported capabilities.
19. As a keyboard user, I want example and SDK tabs to support the same keyboard actions, so that I can operate either language without a pointer.
20. As a visitor trying a card, I want to reveal and hide its answer with localized feedback, so that I can understand the preview interaction.
21. As a visitor exploring media, I want the same audio playback and pause behavior in both languages, so that I can evaluate the same media example.
22. As a visitor downloading examples, I want both languages to provide the same generated APKG files, so that the displayed example and downloaded deck agree.
23. As a deck author, I want the same stable-note update demonstration and reset action in both languages, so that I can understand how content changes preserve identity.
24. As a deck author reviewing updates, I want the same original and updated packages and comparison report, so that I can inspect the evidence behind the demonstration.
25. As a developer comparing export performance, I want the same measured workloads, values, units and conditions in both languages, so that translation does not change the evidence.
26. As a developer assessing product claims, I want the same limitations and compatibility conditions in both languages, so that I can make an informed decision.
27. As a documentation reader, I want the same grouped sidebar topics, ordering, collapse behavior and current-page indication, so that I can follow the same learning path.
28. As a documentation reader, I want a working search entry and keyboard shortcut in my chosen language, so that I can find relevant content without changing languages.
29. As a reader searching the site, I want relevant results to open pages and sections in the current language, so that search preserves my reading context.
30. As a reader encountering an empty or failed search, I want clear localized feedback and a usable close action, so that I can recover from the unsuccessful search.
31. As a reader of a long guide, I want an equivalent table of contents and stable section links, so that I can navigate the same material in either language.
32. As a documentation reader, I want equivalent previous and next links, so that I can continue through the guide without missing topics.
33. As a contributor, I want the edit-page action to open the authoritative source for the current translation, so that I can correct the content I am reading.
34. As a developer following a tutorial, I want equivalent code rendering and copying for the same runnable examples, so that I can complete the task in either language.
35. As a Node developer, I want both quickstarts to explain the same module-system and TypeScript requirements, so that translated instructions do not omit necessary integration details.
36. As a Rust or Python developer, I want equivalent downloads, API references, diagnostics and next-task links, so that the translated quickstart supports the same follow-up work.
37. As a reader following internal links, I want guide, API, media, migration and update links to remain in the current language, so that the documentation forms a complete learning path.
38. As a visitor exploring the examples gallery, I want the same source, previews, metadata and downloads in both languages, so that I can evaluate every example.
39. As a blog reader, I want the same article catalogue and complete translated articles, so that language does not remove tutorials or engineering explanations.
40. As a blog reader, I want localized category names and date formatting with unchanged publication facts, so that the information is natural to read and remains accurate.
41. As an RSS subscriber, I want a feed for my selected language with corresponding article links, so that new posts fit my reading preference.
42. As a visitor using footer links, I want equivalent destinations and language-preserving article and feed links, so that navigation remains consistent at the end of a page.
43. As a visitor reaching a missing page, I want an understandable error state and recovery links for my language, so that I can return to useful content.
44. As a screen-reader user, I want labels, image alternatives and live status messages translated consistently, so that both languages expose the same meaning and controls.
45. As a visitor using reduced motion or increased zoom, I want equivalent readable and operable layouts, so that accessibility preferences work across languages.
46. As a visitor interacting with dynamic controls, I want loading, selected, disabled, success, error and reset states in the current language, so that the interface does not unexpectedly revert to English.
47. As a visitor arriving from search engines, I want the correct localized title, description and destination, so that I can identify the page in my preferred language.
48. As a visitor sharing a page, I want the social preview to use the corresponding language with the same product meaning, so that the shared link represents the page accurately.
49. As a developer reading technical examples, I want API names, commands, stable keys, media and output behavior to remain unchanged by translation, so that the examples preserve the product contract.
50. As a visitor entering a task into the AI builder, I want the existing local and temporary handling of my input to remain intact, so that localization does not introduce storage or transmission of my text.
51. As a maintainer, I want one source for shared page structure, navigation and behavior, so that new functionality reaches both languages together.
52. As a maintainer, I want missing translations and mismatched message parameters to fail validation, so that untranslated states cannot silently ship.
53. As a maintainer, I want missing translated content to remain distinguishable from fallback output, so that generated English text cannot count as a completed Chinese page.
54. As a maintainer, I want changes to technical content to trigger review of the paired translation, so that published languages do not drift over time.
55. As a maintainer, I want acceptance checks to inspect the actual built and deployed website, so that a self-consistent configuration cannot hide missing pages or broken functionality.
56. As a returning reader, I want existing URLs and meaningful deep links to continue working, so that bookmarks and previously shared links survive the migration.

## Implementation Decisions

### Scope and compatibility

- 支持英文与简体中文，内部语言标识、URL 前缀、显示名称和格式化规则集中定义。新增第三种语言不属于此次交付。
- 以当前全部英文主题与两端已有完整能力的并集作为基线。首页、文档总览与指南、API 参考、示例、博客列表和文章均须配对；错误页和辅助资源也必须有明确的语言行为。
- 首轮补齐 21 篇中文文档及示例页、博客列表、两篇博客，并校准原 6 组文档。主题包括安装、核心概念、卡片、媒体、自定义笔记类型、模板、图片遮挡、更新、Rust 编写、API、兼容性、保证、排错、迁移和源码构建。
- 保留已公开的英文与中文主题地址，英文继续使用根语言路径，中文沿用 zh-cn 前缀。页面对数随真实主题集合增长，不使用固定数量限制后续扩展。
- 本次只改变官网呈现与内容组织，不修改 Rust/Node/Python 的公开 API、Project/Note/Media 语义、APKG 身份证据、更新策略或分发行为。术语与当前概念说明、ADR 0023 的所有权和更新语义保持一致。

### Shared website model

- 建立全站主题登记，记录稳定 pageId、页面类型、所属栏目、导航顺序、各语言内容来源与路径、标题/描述及必需能力。博客使用稳定 translationKey，标题不是配对依据。
- 导航模型统一管理 Header、移动菜单、文档主题树、前后页、Footer 和后续任务；同主题能力模型定义模块顺序、动作、技术数据与共享资源引用。
- 主题清单是公共解析入口，但验证必须独立发现实际内容源和产物，发现漏登记或多余页面，避免仅检查清单自身的一致性。这与已有契约治理中检查实际资产而非第二份白名单的原则一致。
- 每一种页面类型使用一个模板。语言路由只选择 locale 与内容；不能通过语言条件分支增删模块、改变操作结果或重排栏目。
- 品牌和图标继续共享。字体回退、自然换行、日期显示及相同事实的数字格式可以本地化；不将字形或文本长度差异视为功能差异。

### Header, footer and routing

- 两语 Header 都提供指南、示例、博客及相同公共工具；Guide 均进入文档总览。当前栏目按稳定 sectionId 判断，不能以某篇入门页路径替代整个文档分区。
- 桌面和移动导航使用同一项目模型。紧凑布局必须容纳两种语言的菜单、语言切换与主题控件，修复 320px 裁切和菜单逐字换行。
- Header、Footer 和页面内的站内内容链接通过 pageId、locale 和可选 sectionId 解析。共享制品、registry、许可证和外部仓库按资源类型保留共同目标，不统一强加语言前缀。
- 语言切换落到同主题，保留可对应章节及页面支持的查询状态。章节使用稳定 ID 或明确映射，已公开旧锚点保留兼容入口；不能把一个不存在的 hash 拼到另一语言页面。
- 主题偏好跨页面和语言共享。AI 任务输入继续按现有本地临时原则处理，不增加持久化或上报。

### Homepage and interactive examples

- 两语首页采用同一主序列：Hero、安装、AI 生成器、性能、示例工作台、更新演示、功能与后续操作。中文已有 JSON 与选型入口进入两语共同的后续操作集合。
- 保留共享 Hero 卡片动画、三语言安装数据和五组性能数据。Hero 展示不能替代可手动操作的示例工作台。
- 两语均提供 Basic、Cloze、媒体与图片遮挡工作台，具有相同源码、Tab、键盘操作、翻卡及音频控制；切换示例时保持既有媒体暂停行为。
- 两语均提供稳定笔记 ID 的内容更新与重置演示、真实比较报告及两个版本的 APKG。四个示例包加两个更新包构成当前首页的六个下载入口，两个语言指向同一批制品。
- AI 生成器继续支持 Rust、Node / TypeScript、Python。任务输入、提示、安装命令复制和无脚本模板行为一致，生成的文档引用按 SDK 主题与当前语言解析。

### Unified documentation

- 保留 Astro 与 Starlight，启用英语 root locale 和简体中文 locale，让两语文档使用同一内容模型、渲染器和布局；搜索继续使用 Pagefind。
- 完整中文内容准备好后，在同一个迁移变更中撤除独立中文文档集合、布局和路由生成，避免 URL 冲突、重复产物和两套行为长期共存。
- 文档统一提供相同主题树、分组与折叠行为、当前位置、搜索入口与快捷键、页内目录、前后页和编辑入口。公共品牌与工具通过共享适配组件接入，保留框架已有搜索及移动侧栏行为。
- 编辑入口指向当前语言的权威内容源；生成内容不能成为新的人工维护副本。
- 采用共同代码展示和复制规则，使命令与完整程序获得等价能力；中文现有的完整程序复制必须保留，英文相应补齐。
- 中文已有搜索索引，缺失的是完整入口与体验。验收同时检查索引、UI、结果语言、目标章节和失败状态。
- 缺译 fallback 不能计作译文。开启多语言之前检查真实来源，生成之后核对正文语言、来源与页面能力，避免英文正文出现在标为中文的页面。

### Content and message contracts

- 短 UI 文案集中为受类型或 schema 约束的消息集合。两语 key、插值参数和必需字段必须匹配，覆盖初始态及客户端选中、成功、失败、禁用、重置等状态。
- 可见文字、alt、title、aria-label、aria-live、无脚本说明和复制回退信息共同纳入翻译范围；接口标识符、品牌名及示例教材语言保留原义。
- 长文分别维护地道的 Markdown/MDX 内容，章节任务、步骤、表格列、限制条件、代码和后续动作必须对应。不要求逐段字数相等，也不以自动文本相似度代替审校。
- 原六组文档必须同步缺失的 registry 列、SDK 顺序、APKG 下载、ESM/CommonJS 与 TypeScript 说明，以及对应 SDK 的 API、媒体、诊断、迁移和后续任务入口。
- 安装版本、运行环境、可执行代码、source marker、示例媒体、APKG 和性能指标沿用现有单一技术来源；翻译不改变稳定键、代码逻辑或产物。
- 记录内容修订与配对译文的复审状态。技术事实、步骤或行为变化必须同步复核另一语言；带待复审状态的内容不能作为最终交付发布。

### Blog, feeds, metadata and error states

- 博客列表、详情和文章卡片共用模板，按主题配对完整正文、分类和相关链接；日期格式接收 locale，发布日期事实不变。
- RSS 由共享生成器输出对应语言的条目、链接与 language 元数据；页面发现链接及 Footer 均指向当前语言 feed。
- 正常内容页具有自引用 canonical、双向语言 alternate、正确 html lang 和 OG locale，以及对应语言的标题、描述和图片说明。迁入原生多语言后消除重复的手工 alternate；每页保留一个现有 analytics beacon。
- 社交图片共用构图与技术素材，自然语言文字分别生成，不以只翻译 alt 代替图片文字本地化。
- 错误页共用内容与恢复动作模型，必须验证真实托管平台对未知语言路径的响应。新建一个中文错误页地址不能证明所有中文无效地址都会使用它。
- 在当前静态托管边界内优先完成语言正确的错误恢复。无脚本条件下若主机不能按请求路径选择语言，必须如实记录该限制；不能将客户端语言切换宣称为服务器已经返回本地化错误页。任何必要的托管或边缘路由变更单独明确，不自动扩大为全站平台迁移。

### Rollout and ongoing synchronization

- 先确认当前源码、发布提交、Actions 制品与正式域名的对应关系。审计时的 404 是生产快照，不能仅凭本地页面存在宣布问题已解决，也不能在未核对前断言部署失败原因。
- 顺序为：发布基线核对 → 全站模型与失败检查 → 共享公共页面与交互 → 补齐及审校译文 → 统一文档系统与辅助资源 → 全量制品及生产验收。内容工作可在共享主题模型稳定后并行推进。
- 替换原“中文仅七页”“中文 Rust API 必须 404”及“中文引用英文 API”的旧预期。仍然检查真正不存在的路径返回 404。
- 扩展既有构建、制品检查和发布后检查流程。新增主题或功能必须同变更提供双语实现、对应内容及必要验收，不能单独放宽检查后发布。
- 可以先交付不影响访问的共享基础设施；向用户提供完整双语入口前，所有对应内容和能力必须就绪。维护说明同步更新，原局部翻译策略保留为历史而非新实现约束。

## Testing Decisions

### Primary seam

**主要测试边界为生产构建后的完整官网，经 HTTP 访问其公开页面与资源。** 本地预览与正式域名是同一外部契约的两个环境。全量静态产物检查与浏览器交互检查围绕这一边界组织，不为每个 Astro 组件建立独立 mock 或私有状态测试接口。

测试应通过用户可观察的导航、文字含义、内容覆盖、控件行为、下载字节和 HTTP 结果判断成败。可以使用稳定主题、章节和能力标识辅助定位，但不能依赖偶然的 class 名、私有函数调用顺序或整页 HTML 字符串相等。两种语言分别保留视觉基线，不要求不同文字逐像素相同。

测试覆盖全站路由与元数据、公共导航、首页交互、示例、文档检索与阅读、博客及 feed、错误恢复和生产发布结果。消息与内容完整性通过构建或检查命令对不完整输入的可观察拒绝验证；现有链接重写、提示渲染等纯函数测试继续保留，作为边界情况补充，不另造第二套页面模型。

### Prior art and verification matrix

复用现有 check:site 对真实 HTML、锚点、元数据、下载及哈希的检查，check:live 对正式 HTTP 页面、域名与 sitemap 的检查，以及现有 base-path、内容链接、AI 提示、文档 source marker、可执行示例和公开包验证。此次审计的双语浏览器场景作为参数化整站回归的起点。

| 验收项 | 公开行为与断言 |
| --- | --- |
| T1 全量主题覆盖 | 独立清点真实源与产物；每个正常主题具有真实的英文和中文内容，集合相同；漏登记、缺译或 fallback 冒充译文均失败。当前基线为 32 对，未来按主题集合计算 |
| T2 导航与深链接 | 两语 Header/Footer/移动菜单有相同项目、顺序和目标角色；指南进入总览；切换语言可往返同主题与有效章节；站内内容默认保持语言；旧地址和公开锚点继续有效 |
| T3 首页模块与事实 | 两语模块顺序一致，三 SDK 命令、版本、运行环境和五组性能值来自同一输入；JSON/选型等后续任务均存在；不能以删掉另一语言模块通过比较 |
| T4 示例与更新 | 四个示例的 Tab、方向键/Home/End、翻卡及读屏反馈正确；媒体可播放并按切换规则暂停；更新/重置表现一致；六个下载及比较报告真实可读，内容和哈希符合共享生成记录 |
| T5 AI 与复制状态 | 三 SDK、任务输入、对应语言文档引用、复制成功/失败回退/恢复可操作及中文动态反馈一致；异常场景允许在浏览器权限边界控制结果，仍检查最终可见反馈；不以模拟成功代替正常剪贴板集成验证 |
| T6 文档体验 | 相同侧栏主题、分组、目录、前后页、编辑入口和代码复制能力；语言选择表列/SDK 顺序、入门技术说明、下载及后续动作完整；编辑进入正确的源内容 |
| T7 搜索 | 两语代表关键词均通过真实 UI 得到正确语言的结果及章节；结果可访问；空结果、加载失败、快捷键、关闭及焦点恢复可用；错误页不进入搜索结果 |
| T8 博客与辅助资源 | 两语列表与文章一一对应，日期事实相同；RSS 条目、发现链接和站内导航保留语言；社交图片语言与正文相符 |
| T9 主题与无障碍 | 浅色、深色、系统主题及跨语言保持行为一致；键盘、读屏标签、状态消息、图片替代文本、减少动画、200% 缩放均覆盖 |
| T10 响应式与无脚本 | 覆盖 320、390、768、1440px，无整页横向溢出或被裁切导航；无脚本时两语必需内容、三份 AI 模板、安装命令及下载仍可访问，复制等增强控件的降级明确且一致 |
| T11 元数据与真实 404 | 正常页自引用 canonical、互惠 alternate、正确语言/OG 元数据及唯一 beacon；未知中英路径保持真实 404 与正确恢复动作；无脚本错误页按宿主实际能力单独验证，不被正常页面数掩盖 |
| T12 发布与配置 | 根域部署和项目路径前缀两种配置通过构建与整站检查；部署后逐一检查全部登记内容 URL、语言、关键能力和元数据；生产集合与已验证制品一致 |
| T13 门槛有效性 | 用缺少一篇译文、漏掉一个必需动作、失效锚点、错误语言链接或不完整消息的受控输入证明检查会失败；不能仅证明正常配置自洽 |

所有内容页执行轻量全量覆盖、结构能力、元数据和链接检查。浏览器按首页、示例、博客列表/详情、文档与错误页类型选择代表场景，在两语运行同一套步骤；新增不共享的页面行为必须进入矩阵。内容审校复核章节任务、表格、API 事实与中文自然程度，作为自动检查的补充。

### Completion criteria

- 所有正常内容主题真正双语配对，当前 25 项缺页与原六组内容差异全部处理。
- 同主题组件、功能、动作、技术事实与资源一致；翻译覆盖静态和动态状态及无障碍文本。
- T1–T13 的适用检查通过，未完成的宿主错误页边界明确列出且不能计为已通过的能力。
- 正式域名与本轮已验收制品一致；本地构建成功、旧检查通过或索引存在均不能代替该结论。
- 维护说明、持续同步规则和回归门槛随实现交付，历史部分翻译策略不再阻碍新增中文内容。

## Out of Scope

- 新增英文与简体中文以外的语言、地区版本或自动机器翻译服务。
- 改变 Anki Forge 的公开 SDK、Project/Note/Media 所有权、模板语义、APKG 格式、身份或更新策略；本规格中的“语言”指官网自然语言，不是 SDK 能力扩展。
- 重做产品品牌、另建 CMS、替换 Astro/Starlight/Pagefind，或为多语言另建应用后端。
- 新增账户、支付、云端牌组制作、AI 模型调用或保存/上传访客任务输入。
- 增加新的示例场景、性能实验、性能承诺或与本次一致性无关的营销内容。
- 翻译 GitHub、包 registry、第三方文档和开源许可证原文；站内提供对应的说明与链接即可。
- 在本次规格生成任务中修改网站、部署站点或发布 SDK。后续实现按既有工程与发布流程执行。
- 自动迁移托管平台、扩展统计采集或改写历史审计结果；若错误页严格语言行为确实需要宿主能力调整，单独明确最小变更及其影响。

## Further Notes

依据为本轮《Anki Forge 官网多语言一致性优化方案》、当前工作区源码和 2026-10-26 审计。该审计区分了本地构建与生产快照：本地新构建有 39 个正常内容页加一个 404，正式 sitemap 有 30 个英文 URL。缺页生产状态需要实施时重新核对，不能把这份历史观察当作持续在线监测。

已确认可复用的部分包括共享安装/性能数据、Hero 动画、三 SDK 的 AI 生成器、中文复制反馈和主题保留。已确认的关键差异包括：中文首页缺工作台、音频、更新演示及六个下载入口；中文文档缺搜索 UI、目录、前后页和编辑入口；中文索引本身可检索；320px 英文菜单裁切、中文菜单分行；切换语言丢失锚点；中文代码复制辅助提示仍是英文。

本轮完成了源码构建、现有检查、40 个 HTML 清点、选定中英页面的浏览器与无脚本比较、英文搜索 UI 和中文索引查询。模拟剪贴板成功/拒绝用于反馈审计，不是操作系统权限验收；本规格要求的完整功能矩阵尚待实现后执行。

内容技术语义使用项目当前概念及 ADR，不将笔记、卡片、牌组、Project 和 APKG 混用。领域契约来源及代码示例的既有校验继续有效；网站国际化不建立另一套 SDK 语义或技术事实来源。

实施过程中如果发现新主题或新增真实能力，应纳入两语清单和同一验收边界，不以审计时 32 对主题或 15 个组件作为扩展上限。
