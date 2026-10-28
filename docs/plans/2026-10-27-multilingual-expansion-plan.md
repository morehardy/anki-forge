# Anki Forge 官网与文档六种新增语言方案

实施以[后续完整实现方案](2026-10-27-multilingual-implementation-spec.md)为准。用户已选择独立 AI 审校、每语完整后发布；本文原有人工审校资源和 6–8 周估算不再适用，语言发布队列也以新方案为准。

日期：2026-10-27。设计基线：本地提交 `732a40f`。本方案承接现有中英双语实现，目标是扩大与 Anki 制卡开发相关的自然搜索访问，并让新访客能够完成安装、生成牌组和阅读 API 文档。

**建议新增西班牙语 `es`、巴西葡萄牙语 `pt-BR`、日语 `ja`、德语 `de`、法语 `fr`、俄语 `ru`，形成八语网站。发布顺序为西语与巴葡 → 日语与德语 → 法语与俄语。每种语言完整覆盖现有官网及文档后发布。**

这是依据公开市场数据、开发者市场规模和现有维护成本作出的投入排序。本站尚无可用于本次排序的 Search Console 或分析后台导出，因此不能将其表述为已经证实的“本站新增流量前六”，也不能承诺某个流量增长百分比。上线后的衡量重点是全站同主题的新增搜索点击，避免把原英文访问转移到译文页误算为增长。

## 一 语言选择与流量依据

Anki Forge 是面向 Rust、Node、Python 开发者的 APKG 生成库。应优先覆盖“通过代码生成或更新 Anki 牌组”的搜索任务。学习某门外语的 Anki 用户数量，不能直接代表使用该语言阅读开发文档的用户数量。

| 批次 | 新增语言 | 公开市场信号 | 本次选择理由 | 需要验证的假设 |
| --- | --- | --- | --- | --- |
| 1 | 西班牙语 `es` | W3Techs 网站内容语言占比 6.1%；可覆盖西班牙及多个拉美市场 | 用一套中性西语覆盖多个市场，优先建立跨市场的任务入口 | 西语任务查询是否带来原英文站未获得的点击 |
| 1 | 巴西葡萄牙语 `pt-BR` | 葡语网站占比 4.1%；GitHub 巴西账号约 832 万 | 巴西有较大的开发者市场，先确定一种葡语编辑规范以控制成本 | 葡语内容是否提升 Python、Node 入门页面的获客效果 |
| 2 | 日语 `ja` | 日语网站占比 4.9%；GitHub 日本账号约 543 万 | 市场规模足以支持独立的日语技术文档投入 | 日语本地搜索需求、术语和分词搜索体验是否匹配 |
| 2 | 德语 `de` | 德语网站占比 5.9%；GitHub 德国账号约 526 万 | 内容市场与开发者市场均有规模，适合完整技术文档 | 德语页面相对英文页面的额外价值有多大 |
| 3 | 法语 `fr` | 法语网站占比 4.6%；GitHub 法国账号约 403 万 | 可服务多个法语市场，保持通用法语内容 | 不同地区的开发任务查询能否由同一版本覆盖 |
| 3 | 俄语 `ru` | 俄语网站占比 3.4%；GitHub 俄罗斯账号约 463 万 | 具备独立内容市场和开发者规模，作为第六种优先验证 | 相对印尼语的净新增访问是否更高，搜索来源是否需补充 Yandex |

网站占比取自 [W3Techs 历史语言统计](https://w3techs.com/technologies/history_overview/content_language)的 2026-10-27 列，表示检测到内容语言的网站份额，**不表示流量、搜索量或用户份额**；葡语统计也没有单独拆出巴葡。GitHub 数值取 2026 年第一季度的 [Innovation Graph 固定版本](https://github.com/github/innovationgraph/blob/054c7dbc527518fa2ecfd316efe2aa01f3986c39/data/developers.csv)，是按经济体归属的账号数，包含不再活跃的账号，不等同于母语用户或活跃开发者。固定数据版本、定义和比较记录见[语言市场研究](evidence/multilingual-expansion-2026-10-27/language-market-research.md)。表内选择理由是产品判断，不能由单个统计量直接推出。

候补优先考虑印尼语 `id`，其次比较韩语 `ko`。印尼的 GitHub 账号规模足以挑战第六席；如果本站查询与使用反馈显示更强机会，应在俄语内容启动前调整顺序。印度的开发者规模不能直接转化为印地语 `hi` 的需求；阿拉伯语 `ar` 还需要 RTL 布局验收。意大利语、土耳其语、繁体中文等进入后续候选池，由实际需求决定，不按人口机械扩张。

语言版本采用以下范围：西语先做一套中性 `es`，不复制 `es-ES`、`es-MX`；葡语先明确为 `pt-BR`，不把巴葡当成已完成 `pt-PT`；德语、法语、日语和俄语使用语言级代码，不按国家重复建站。俄语版本的受众也不限定为俄罗斯。

## 二 当前基础与工作规模

现有 [site-model.mjs](../../website/site-model.mjs) 已提供稳定主题 ID、路由、文档分组和来源；网站使用 Astro 7.3.3、Starlight 0.42.2、Pagefind、静态部署和 GitHub Pages。保留这些基础设施。框架版本以 [package.json](../../website/package.json) 与 lockfile 为准，扩语不与框架升级捆绑。

| 内容 | 当前每语 | 六种新增语言 | 最终八语 |
| --- | ---: | ---: | ---: |
| 首页、示例页、博客列表 | 3 | 18 个页面 | 24 个页面 |
| 文档主题 | 27 | 162 篇译文 | 216 个页面 |
| 博客文章 | 2 | 12 篇译文 | 16 个页面 |
| 正常内容页面合计 | 32 | 192 个页面 | 256 个页面 |

另有每语 189 个业务 UI 文案键、11 个 Starlight／搜索覆盖键、3 种 SDK 的 AI 提示模板，以及页面标题、描述、RSS、社交图和 404 反馈。六语需完成至少 1,200 个现有词典条目，新增语言菜单等功能会继续增加条目。这里不把 RSS、图片和错误页计入 256 个正常内容页。

29 篇英文长文按剔除 fenced 代码块、HTML 标签与注释后的粗略英文词匹配统计约 13,440 词，六语对应约 80,640 源词翻译量。统计仍包含部分链接、标识符，且未完整计入短 UI、页面元数据和 AI 模板，适合排期估算，正式报价需重新提取可翻译文本。

当前仍存在以下中英专用实现：

- `site-model.mjs` 的 `page.en/page.zh`、`zhSource`、`zhTitle` 和路径判断。
- `LanguageSwitcher.astro` 的中英互切链接；多数组件的 `'en' | 'zh-CN'` 类型。
- `prepare-site.mjs`、`content-links.mjs`、章节映射和中译文目录扫描。
- Starlight 配置、文档 Header/Footer/Search、route middleware 的中文判断。
- `content-reviews.json` 的双语 hash 对、以汉字判断译文的内容检查。
- 首页与文档的 alternate／社交图、博客 schema、RSS、404、AI 提示检查及浏览器用例。

这些部分需同时泛化，否则仅添加六个语言目录会产生错链、错误正文语言或校验盲区。当前代码清点不是生产发布证明；实施第一步应核对正式站点的发布 manifest 与提交。

## 三 路由和用户体验

所有语言继续使用同一域名及子目录，英文保留根路径，既有中文 URL 保持稳定。页面 slug 保持跨语言一致，内容标题自然本地化。

| 语言显示名 | locale 与 html lang | URL 前缀 | Python 入门页示例 |
| --- | --- | --- | --- |
| English | `en` | 无 | `/docs/python-quickstart/` |
| 简体中文 | `zh-CN` | `/zh-cn` | `/zh-cn/docs/python-quickstart/` |
| Español | `es` | `/es` | `/es/docs/python-quickstart/` |
| Português (Brasil) | `pt-BR` | `/pt-br` | `/pt-br/docs/python-quickstart/` |
| 日本語 | `ja` | `/ja` | `/ja/docs/python-quickstart/` |
| Deutsch | `de` | `/de` | `/de/docs/python-quickstart/` |
| Français | `fr` | `/fr` | `/fr/docs/python-quickstart/` |
| Русский | `ru` | `/ru` | `/ru/docs/python-quickstart/` |

语言入口采用地球图标与当前语言名称，打开后列出全部已发布语言，用语言自称和选中标记，不使用国旗。桌面放在公共 Header 与文档 Header 的同一工具区；移动端保留可见入口并允许面板滚动。使用可展开的原生链接列表，保证键盘、焦点、关闭操作和无 JavaScript 时仍能访问语言链接，避免只能靠 `select` 的脚本跳转。

切换语言落到同一 `pageId`；通过稳定章节 ID 携带有效锚点，并保留合法查询参数。站内导航、文档侧栏、上一页／下一页、搜索结果和“下一步”都保持当前语言。品牌资源、SDK 包、源码仓库及 APKG 下载使用共同目标。主题偏好继续共享，AI 输入内容保持现有临时本地处理，不因扩语而持久化或上报。

用户访问的 URL 决定内容语言；显式选择只用于记忆偏好。浏览器偏好可用于显示一次可关闭的语言建议，不进行 IP 或语言自动重定向，也不覆盖来自搜索结果和分享链接的语言选择。独立 URL 与可选择的语言链接符合 [Google 多语言站点指导](https://developers.google.com/search/docs/specialty/international/managing-multi-regional-sites)。

为德语、法语、俄语的较长文案留出弹性宽度；首页标题、按钮和表头允许换行。为日语配置合适的系统字体与断行规则。图表数字使用 `Intl.NumberFormat`，日期使用 `Intl.DateTimeFormat`；终端命令、版本号、代码数值和程序输出保持原样。代码区可以独立横向滚动，整页不能因翻译溢出。

## 四 内容覆盖与本地化方式

每个已发布语言包含完整 32 个主题及相同功能。新增语言进入开发预览时可以逐步翻译；生产 locale 只有在整套内容通过审核后才启用。草稿语言不进入正式语言菜单、sitemap、RSS 或 hreflang，也不让 Starlight 的英文 fallback 充当译文。

制作顺序先覆盖完整获客路径，再完成其余参考内容：

| 制作顺序 | 具体内容 | 验收重点 |
| --- | --- | --- |
| A | 首页、示例页及 12 个文档主题：`docs`、`languages`、`quickstart`、`installation`、`python-quickstart`、`node-quickstart`、`json-to-anki`、`anki-forge-vs-genanki`、`genanki-migration`、`compatibility`、`troubleshooting`、`updates` | 搜索进入后能理解价值、选 SDK、复制命令并完成第一份牌组 |
| B | 其余 15 个文档主题，包括核心概念、媒体、模板、Image Occlusion、三套 API、构建保证与诊断 | 能完成复杂任务；限制、参数与代码和英文一致 |
| C | 博客列表、两篇文章、RSS、社交图、所有 UI 状态、404 与辅助文案 | 全站体验完整，所有内容均有审核记录 |

A/B/C 是同一语言内部的制作顺序，均属于该语言公开发布的前置条件。

官网首页的本地化重点是价值表达、按钮和真实示例；文档重点是技术准确、术语一致、步骤可复现。各语保持相同模块和行为，使用当地自然表达，避免把英文词序直接套入译文。术语表至少覆盖 `deck`、`note`、`card`、`note type`、`Cloze`、`Image Occlusion`、稳定身份、基线 APKG 和更新比较，必要时首次出现附原术语。

翻译边界明确如下：

- 翻译正文、标题、描述、导航、错误解释、无障碍标签、搜索提示、图片说明和社交图文案。
- 保留 API 标识符、包名、命令、文件名、note key、JSON 字段、诊断代码、版本、哈希和 URL 的技术语义。
- 代码继续来自同一可执行来源。教学用的 `hola → hello` 与示例牌组保持一致，不按界面语言改写牌组实际内容。
- 每个语言的 AI 模板均覆盖 Rust、Node、Python，并链接同语 quickstart；命令、版本和 SDK 限制仍读共同数据。现有两个 README 继续维护，不将新增六份 README 作为本轮交付条件。
- 长文保留相同任务章节、表格信息、限制与下一步。可以调整句式，不通过删段或省略技术条件降低翻译成本。

可先用受术语表约束的机器翻译生成初稿，再由熟悉对应语言及 SDK 的审校者复核。`reviewed` 只能在实际审校后设置，hash 相等或机器检查通过都不能替代语义审核。

## 五 从双语扩展为任意语言的技术设计

### 语言与页面模型

把 `site-model.mjs` 中的二元结构改为 `locale → metadata` 与 `pageId → locale source`。语言配置明确区分：内部 BCP 47 标识、URL 前缀、Starlight locale key、`html lang`、OG locale、显示名称、日期数字格式与发布状态。例如 `pt-BR` 对应目录 `pt-br`、`lang="pt-BR"`、OG `pt_BR`；英文 Starlight key 继续为 `root`。

增加显式的 `planned / preview / published` 状态，所有生产生成器仅消费已发布语言集合。不能通过“哪个文件存在就发布哪个页面”推断发布范围。已发布语言强制覆盖全部注册主题，新增语言草稿可以保留在源目录中而不生成正式路由。

保留 `pagePath(pageId, locale, sectionId)`，将 `pagePair` 逐步替换为返回 `{ pageId, locale }` 的通用路径解析。按完整路径段匹配 locale，兼容 `/anki-forge` base，未知前缀不能自动伪装成英文内容。导航文案改用 message key，文档分组 label 与页面元数据按 locale 读取，避免继续增加 `jaTitle` 等字段。

### 内容来源和生成

英文来源保留现有位置，中文仍在 `docs/zh-cn/`；新增 `docs/es/`、`docs/pt-br/`、`docs/ja/`、`docs/de/`、`docs/fr/`、`docs/ru/`。博客使用 `website/src/content/blog/<locale-path>/`，保留相同 `translationKey`。业务短文案使用 `src/i18n/<locale-id>.json`，Starlight 覆盖词典按其 locale loader 的规则生成或维护，并显式验证目录 key 与 `lang` 的映射。

`prepare-site.mjs` 按发布集合生成 `src/content/docs/<locale-path>/docs/`；英文保持 root 目录。生成前清理受管理的旧产物，防止草稿或下线配置留下可抓取文件。共享首页、示例、博客组件继续保留一份实现；非英文 Astro 静态路由通过 locale 参数枚举，保留英文 root wrappers，并检查不会与 Starlight 的 `/docs/` 路由重复。

Starlight 支持 root locale 与多语言目录，也会为缺失内容生成 fallback；本站保留 `route.isFallback` 失败门槛，且在框架生成前检查源文件完整性。[Starlight 国际化说明](https://starlight.astro.build/guides/i18n/)

### 链接与章节

链接解析接收明确的 source locale，不再判断 `docs/zh-cn/`。对注册内容先解析 `pageId` 再生成本语言 URL；源文件和下载按资源类型单独处理，不能给所有 URL 无差别添加语言前缀。

章节映射从一对中英标题扩展为 `pageId → stable sectionId → locale heading/aliases`。继承现有英文别名和已公开的中文锚点，对重复标题做唯一性检查。切换目标锚点在构建期验证并写入小型清单，避免把原来的“预取一个目标页”扩展为每次打开页面就预取七种语言。浏览器只携带确认存在的目标 hash。

### 文案与审校记录

词典继续校验 key 集合和插值参数。组件使用统一 `Locale` 类型；客户端只获得当前语言必要词典和轻量语言列表，避免一次把八套正文相关文案打进 JS。构建时所需完整词典与浏览器数据分开，尤其检查 `site-model.mjs → messages.mjs` 的导入链。

把审校记录改成“每个主题、每个目标语言各一条”：

```text
reviews[pageId][targetLocale]
  sourceLocale: en
  sourceHash: 当前英文源摘要
  targetHash: 当前译文摘要
  status: pending | reviewed | stale
  reviewedAt: 实际复核日期
  reviewer: 实际复核者或可追溯记录
  scope: 技术事实、步骤、表格、代码、限制和后续任务
```

源摘要应覆盖正文及实际影响页面的技术输入，或另存依赖摘要，确保代码片段、版本、契约变化也能使相关复核失效。英文变化将对应七种译文标为待复核；某一译文变化只使该条记录失效。所有已发布语言仍必须在发布前完成复核，避免扩语后逐步积累过期文档。

现有汉字正则不能推广为任意语言识别。自动检查应覆盖目标来源、禁止 fallback、正文不得完整复制英文、受保护代码、章节与表格契约；对短 API 文档的语言识别只作辅助提示，由审校确认，避免把合法英文标识符当作漏译。

## 六 搜索与搜索引擎配置

站内搜索继续使用 Pagefind，默认只查当前页面语言。其索引根据 `html lang` 分组，浏览器加载对应索引。日语需验证 extended 构建实际提供的分词能力；不能只检查“生成了索引文件”。葡语 locale 与搜索 UI 词典也要验证 `pt-BR` 的实际映射。[Pagefind 多语言搜索](https://pagefind.app/docs/multilingual/)

每语准备一组测试词，覆盖本地任务词、`APKG`、`genanki`、`Image Occlusion` 与 API 标识符；例如日语 `画像遮蔽` 与无空格的任务短语。测试正常结果、无结果、网络失败、键盘关闭和焦点恢复。无结果时可以提供明确的“切换到英文搜索”链接，但不混入未标注的英文结果。

SEO 配置从同一主题／语言模型派生：

1. 每个真实译文页使用自引用 canonical，指向自己的 URL。
2. 同主题页面输出已发布语言完整、互相回链且包含自身的 hreflang；最终为八种语言，并增加指向同主题英文页的 `x-default`。
3. 选择 HTML head 作为 alternate 的统一输出方式。Starlight 页复用或规范化其原生输出，不额外叠加第二套；无需再维护一份独立的 sitemap alternate 逻辑。
4. sitemap 包含所有正式内容 URL，排除草稿、404 和无真实正文的页面。`lastmod` 使用真实内容修改时间。
5. 本地化 title、description、OG 文案及图片；OG 地区代码只用于对应元数据，不据此声称地域定向。
6. RSS 按语言生成，文章列表、日期、分类显示和链接一致；结构化数据若已有则同步 `inLanguage` 和对应 URL。

Google 要求 alternate 使用完整 URL，并保持同主题各版本相互对应；这些标记用于识别版本关系，不能代替实际本地化内容。[Google 本地化版本说明](https://developers.google.com/search/docs/specialty/international/localized-versions)

GitHub Pages 继续只有一个静态 `404.html`：脚本根据已知前缀显示对应恢复文案；无脚本时提供清晰的多语言恢复链接。错误 URL 返回真实 404，且不进入索引；不将这一方案描述成服务端已经实现按语言渲染。

## 七 面向新增访问的内容策略

先围绕现有高意图主题建立查询映射，避免为笼统的“Anki”“免费背单词”制造无关页面。同一个任务由一个主要落地页承接；入门、API、比较和迁移页分别解决不同问题。

| 搜索意图 | 主要落地页 | 本地化要点 |
| --- | --- | --- |
| 用 Python、Node 或 Rust 生成 Anki 牌组 | 三套 quickstart | 可运行的最小程序、准确安装命令、产物与验证步骤 |
| JSON 转 Anki，含图片或音频 | `json-to-anki` | 输入结构、媒体路径、完整生成流程 |
| genanki 替代与迁移 | `anki-forge-vs-genanki`、`genanki-migration` | 公平比较、迁移边界、兼容条件和成本 |
| 编程生成图片遮挡、Cloze | `image-occlusion`、`cards` | 对应代码与真实示例，避免将预览当成 Anki 渲染验证 |
| 更新牌组与稳定身份 | `updates` | 原始 APKG 基线、稳定 note key、客户端限制 |

下面仅是关键词调研种子，需要母语审校及真实查询数据核对，不表示已经查得搜索量：

| 语言 | 种子任务短语 |
| --- | --- |
| es | `crear mazos Anki con Python`、`JSON a Anki` |
| pt-BR | `gerar baralhos Anki com Python`、`JSON para Anki` |
| ja | `PythonでAnkiデッキを作成`、`JSONからAnki` |
| de | `Anki Decks mit Python erstellen`、`JSON in Anki umwandeln` |
| fr | `créer des paquets Anki avec Python`、`convertir JSON en Anki` |
| ru | `создать колоду Anki на Python`、`JSON в Anki` |

页面先直接回答任务，再提供前提、程序、输出、限制和相关页；品牌名、包名、版本及证据链接跨语言一致。新文章只在实际查询暴露内容缺口时增加，沿用全站语言覆盖契约。语言上线后可准备对应社区的介绍材料；对外发布与推广另按实际渠道安排。

## 八 数据基线与效果判断

启动时导出最近 90 天搜索数据并保留最近 28 天基线，按查询、落地页、国家、设备和日期分析。Search Console 没有直接的用户语言维度：页面语言由 URL 推导，查询语言需要单独分类，纯 `APKG` 等词保留为未知，不能把国家当成语言。[Search Console 指标与维度](https://support.google.com/webmasters/answer/7576553)

| 问题 | 指标与来源 | 判读方式 |
| --- | --- | --- |
| 新页面能否被发现 | sitemap、URL 检查、索引状态、canonical 选择 | 技术通过与搜索引擎实际收录分开记录 |
| 是否获得更多目标搜索访问 | 非品牌任务查询的展示、点击、CTR；Search Console | 按 locale 与 pageId 看增长，并汇总同主题所有语言检查净增 |
| 哪些市场值得继续投入 | 国家、查询语言推断、目标页及维护工时 | 国家只作辅助；比较持续效果与维护成本 |
| 访客读了哪些内容 | Cloudflare Path、Referer、Visits、PV | 记录口径、日期窗口和采样信息；Visits 不当成独立用户 |
| 是否有使用意向 | 可选新增事件：命令复制、示例下载、仓库／包链接点击 | 属于后续事件采集，不宣称现有 beacon 已有此能力 |
| 是否真正完成任务 | 自愿反馈、独立用户运行示例、可核验的采用记录 | 点击、复制、registry 下载均不能单独证明成功生成 APKG |

现有 Cloudflare Web Analytics 不支持自定义事件，也不记录 query strings／UTM，因此当前可以直接复用的是页面与来源统计。若确需转化事件，再增加独立采集方案；字段只需 `locale`、`pageId`、`sdk`、`action`、`result`，不包含用户输入或代码。[Cloudflare FAQ](https://developers.cloudflare.com/web-analytics/faq/)、[指标定义](https://developers.cloudflare.com/web-analytics/data-metrics/high-level-metrics/)

每批次记录实际发布时间，并在第 14、28、56、84 天复盘。14 天侧重抓取和技术问题；28 天看初始查询；56／84 天比较等长窗口中的点击和内容效果。优先观察原英文及新语言合计的同主题点击变化，以未受此次翻译影响的主题趋势辅助解释，不把前后对比当作严格因果证明。小样本报告原始点击数并延长观察期，不只显示增长百分比。

上线前不给绝对访问量承诺。发布门槛使用可控的技术与内容标准；增长阶段根据实际展示和点击形成目标。若已有展示但 CTR 低，检查标题与意图；有点击但任务反馈弱，检查文档完成路径；没有展示，先检查索引、需求与内容竞争，再判断语言是否值得继续新增内容。对已有语言保留有效 URL 与必要维护，不因短期流量低删除整种语言。

## 九 文件工作包与实施顺序

| 工作包 | 主要文件 | 交付及依赖 |
| --- | --- | --- |
| 基线与需求清单 | `docs/plans/evidence/`、发布 manifest、已有测量记录 | 记录生产提交、现有路由、搜索数据可用性和六语负责人 |
| 通用 locale 与主题模型 | `website/site-model.mjs`、`scripts/core-pages.mjs`、`src/content.config.ts` | 明确语言身份、发布集合、通用路径解析与 locale 类型 |
| 内容生成与链接 | `scripts/prepare-site.mjs`、`content-links.mjs`、`content-sections.mjs` | 任意语言来源、章节别名和可追踪生成产物 |
| 内容与审校 | `docs/<locale>/`、`src/content/blog/`、两组词典、`content-reviews.json` | 174 篇新增长文译文与全部短文案，独立审校状态 |
| 页面与语言选择 | `src/pages/`、`LanguageSwitcher.astro`、共享组件及 `src/scripts/site.ts` | 八语菜单、相同功能、同主题切换、只加载当前语言词典 |
| 文档壳与搜索 | `astro.config.mjs`、`src/route-middleware.ts`、Docs 系列组件 | 从发布集合配置 Starlight、侧栏、Pagefind 与编辑链接 |
| AI 与辅助资源 | `docs/ai-quickstart.json`、AI 渲染／检查脚本、`src/lib/feed.ts`、社交图脚本、`404.astro` | 三种 SDK 的六语模板、新 feeds／图片／错误状态 |
| SEO 与验证 | `SiteLayout.astro`、检查脚本、browser tests、`.github/workflows/website.yml` | 256 页产物覆盖、八语互链与真实部署验证 |
| 维护说明 | `docs/documentation.md`、`website/README.md` | 更新来源规则、语言发布步骤、审校流程和回滚流程 |

先完成通用模型、生成器和验证泛化，在原有中英内容上证明兼容；新增语言翻译可同时推进。每批通过完整审校后修改发布集合并运行现有发布流程。不要复制六套页面组件，也不要手工编辑 `src/generated`、`public/generated` 或生成后的文档。

## 十 排期与成本假设

按 1 名熟悉仓库的工程师、每种语言均有合格审校者且至少两语能并行审核的条件，预留 **6–8 周**。这是资源排期假设，机器翻译生成速度不能替代审校时间。

| 阶段 | 建议时间 | 出口 |
| --- | --- | --- |
| 基线和通用基础设施 | 第 1 周，复杂边界可延至第 2 周 | 中英原功能保留，任意语言模型和缺译检查通过 |
| 西语与巴葡 | 第 2–3 周 | 两语各 32 个主题、全部交互和独立审核完成并发布 |
| 日语与德语 | 第 4–5 周 | 完成内容、日语检索、长文本布局与发布验证 |
| 法语与俄语 | 第 6–7 周 | 完成内容、搜索与发布验证；六语覆盖完成 |
| 缓冲和维护移交 | 第 8 周或前述阶段内消化 | 修复评审问题，形成维护清单与测量记录 |

初步工作量预算：工程 8–12 人日，六语内容复核合计 36–60 人日，综合 QA 与发布验证 4–6 人日，共 **48–78 人日**。内容估算按每语约 6–10 人日预留，包含术语、长文、UI、元数据和代码语义核对；属于规划值，首语完成后按实际耗时校准。人工费用按各角色实际日费率计算，外包翻译按提取后的源词及报价另算，避免把已经包含的翻译审校重复计费。

发布后预留持续容量：新增一个主题需要维护八份正文／元数据；现有技术源变化需要复核七种译文。每语明确负责人和替补，设置变更通知与待审列表。既有生产内容的技术准确性优先于新增推广文章。

## 十一 验收和回滚

| 验收层 | 必须满足的条件 |
| --- | --- |
| 覆盖 | 最终 32 × 8 个正常内容路由；独立发现源文件与产物，零缺页、零未登记生产页面、零 fallback |
| 内容 | 所有发布译文有匹配源摘要与译文摘要的审校记录；版本、代码、表格信息、限制和任务步骤齐全 |
| 功能 | 首页、示例、更新演示、下载、三种 SDK 提示、复制成功／失败／恢复、无脚本状态在每语一致 |
| 链接 | 站内链接保持 locale；八语主题切换、章节锚点、旧英文／中文锚点和查询参数有效 |
| 搜索 | 每语本地任务词和 API 标识符可检索；正常／空结果／加载失败文案正确，结果语言一致 |
| SEO | 一份自引用 canonical；最终八语 alternate 加 x-default；sitemap／RSS／OG 与发布集合一致 |
| 视觉与无障碍 | 320、390、768、1440px；深浅主题、键盘、200% 文字缩放、减少动画；无整页横向溢出或截断按钮 |
| 资源 | 不发送八套客户端词典，不预取七个完整目标页；搜索按语言加载；社交图具有所需字体并人工检查 |
| 部署 | 根路径和 `/anki-forge` 前缀都通过；实际正式内容、资源与 verified manifest 对应，未知 URL 返回 404 |

复用现有 `npm test`、`check:ai`、`check:docs`、`check:content`、`check:examples`、`check`、`build`、`check:site`、`check:gates`、`test:browser` 和发布后 `check:live`。保留共享 Rust／Node／Python 示例与公开包消费者验证，代码相同的例子无需仅因语言数量扩大而执行八遍。

所有页面执行轻量结构与链接检查；每语代表页面执行完整浏览器场景。额外保留能证明门槛有效的负向用例：删除西语正文、复制英文冒充法语、弄错葡语 locale key、破坏日语索引、让德语导航跳回英文、保留过期俄语 hash、生成不对称 hreflang。目录含有译文或构建成功都不能单独视为完成。

部署前保留上一个已验证 artifact 与 manifest。出现批量错链、缺译或搜索故障时回滚整个网站产物，保持正文、语言菜单、alternate 和索引属于同一版本；恢复后重新核验正式 URL。发布已经稳定后，优先修复对应语言的问题并保留 URL，避免将整种语言重定向到首页造成搜索入口损失。

本轮交付为语言选择与实施设计。正式执行时先完成通用模型和验证门槛，再按三批内容计划扩展；扩语完成的定义是六种新增语言均通过上述验收并取得生产发布证据。
