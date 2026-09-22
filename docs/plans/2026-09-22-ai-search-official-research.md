# Anki Forge 官网：AI 搜索与 SEO 官方依据核对

核对日期：2026-09-22。官网：[ankiforge.dev](https://ankiforge.dev/)。

本文是官网整体审计的资料依据，覆盖 Google、Bing、OpenAI、Anthropic、Perplexity 及 llms.txt 提案。所有引用均已打开第一方页面；文档中的平台能力以访问当日为准。本文没有访问本站 Search Console、Bing Webmaster Tools 或 CDN 账号，因此不能据此判断本站实际收录、排名、引用次数或账号配置。源码与线上页面的问题由整体审计另行记录。

证据标记：**A** = 平台明确公开的规则或功能；**B** = 官方建议，但没有针对本站的效果数据；**C** = 根据项目场景提出的实施建议，需要上线验证。

## 1. 应优化什么

应分别观察“能否抓取、能否索引、是否被引用、是否获得推荐、是否带来有效访问”。本次查阅的官方资料没有定义跨平台统一的“AI 权重”。Google 的生成式搜索依赖既有搜索索引、排名及质量系统；满足技术条件并不保证展示。因此，不应把增添某个文件或标签描述为确定加权。**证据 A**。[Google 生成式搜索优化指南](https://developers.google.com/search/docs/fundamentals/ai-optimization-guide)

Anki Forge 的执行顺序建议为：核实抓取、索引及展示资格，如有障碍再修复 → 统一站点身份和 URL → 完善可核验的产品与文档内容 → 增加 AI 工具读取文档的便利性 → 持续衡量引用及转化。此排序是项目建议，非平台公开的排名公式。**证据 C**。配套线上审计已确认 29 个 sitemap 页面可公开访问且 canonical 一致，因此不能据本通用清单推断本站存在抓取阻断。

## 2. 基础技术与多语言：优先级最高

| 项目 | 官方依据与实施含义 | 证据 |
| --- | --- | --- |
| 可抓取、可索引、允许摘要 | 先确保公开页面和正文可访问；检查 robots、noindex、摘要限制、CDN/WAF 阻挡。Google AI 展示以可索引且可显示摘要的页面为基础。 | A；[Google AI 指南](https://developers.google.com/search/docs/fundamentals/ai-optimization-guide) |
| canonical 与域名统一 | canonical 是规范网址选择的强信号，不是强制指令。正式域名、跳转、内部链接、sitemap 应一致；规范页面也应自引用。双语内容选择同语言 canonical。 | A；[Google canonical 指南](https://developers.google.com/search/docs/crawling-indexing/consolidate-duplicate-urls) |
| sitemap | 使用正式域名的绝对 URL；lastmod 应反映实质内容更新，不应每次构建全部刷新。Google 忽略 priority、changefreq；提交也不保证抓取。 | A；[Google sitemap 指南](https://developers.google.com/search/docs/crawling-indexing/sitemaps/build-sitemap) |
| hreflang | 翻译页互相指向并包含自身；使用有效语言代码，按需设置 x-default。不要把未翻译或不存在的页面声明为对应语言。Google 根据正文识别语言，不能只改 lang 标签。 | A；[Google 多语言版本指南](https://developers.google.com/search/docs/specialty/international/localized-versions) |
| 页面体验 | 关注移动端阅读、内容稳定和 Core Web Vitals。后者被排名系统使用，但满分不等于排名保证；不应以追逐工具分数替代内容改进。 | A/B；[Google 页面体验指南](https://developers.google.com/search/docs/appearance/page-experience) |

对本站的验收建议：选取首页、中英文入门页、API 页及示例页，核对实际 HTTP 返回的正文、标题、canonical、语言链接、索引指令和 sitemap URL；再用站长平台的 URL 检查核实爬虫实际获取的内容。日志中的 403、429、验证码或空正文应单独排查。**证据 C**。

## 3. AI 爬虫：搜索、训练和用户访问应分别决策

| 平台及标识 | 官方用途 | 对官网的建议 |
| --- | --- | --- |
| OpenAI `OAI-SearchBot` | ChatGPT 搜索发现和展示；拒绝后不会出现在搜索答案中，但仍可能有导航链接 | 如希望被搜索引用，允许公开文档抓取，同时检查官方 IP 请求能通过 CDN/WAF |
| OpenAI `GPTBot` | 可能用于基础模型训练的数据抓取 | 与搜索策略独立；允许训练不等于获得推荐 |
| OpenAI `ChatGPT-User` | 用户触发的网页访问；不是自动搜索爬虫，robots 规则可能不适用 | 不用它替代 OAI-SearchBot 管理搜索展示 |

上述区别来自 **OpenAI Docs**；官方明确允许单独开放搜索而拒绝训练。其文档未给出搜索排名承诺。**证据 A**。[OpenAI Crawlers](https://developers.openai.com/api/docs/bots)

| 平台及标识 | 官方用途 | 对官网的建议 |
| --- | --- | --- |
| Anthropic `Claude-SearchBot` | 搜索结果相关性与索引 | 希望被 Claude 搜索发现时避免误拦 |
| Anthropic `Claude-User` | 用户发起的内容获取 | 可用性应单独检查；Anthropic 表示其 bots 遵循 robots 规则 |
| Anthropic `ClaudeBot` | 模型训练相关抓取 | 作为独立训练选择，不能用其访问量代表搜索曝光 |

Anthropic 文档标注日期为 2026-04-07。**证据 A**。[Anthropic 爬虫说明](https://support.claude.com/en/articles/8896518-does-anthropic-crawl-data-from-the-web-and-how-can-site-owners-block-the-crawler)

| 平台及标识 | 官方用途 | 对官网的建议 |
| --- | --- | --- |
| Perplexity `PerplexityBot` | 搜索展示与链接，不用于基础模型训练 | 允许公开内容抓取；WAF 放行应结合 UA 与官方 IP，而非仅凭可伪造 UA |
| Perplexity `Perplexity-User` | 用户触发的获取，通常忽略 robots 规则 | 区分用户访问和搜索爬虫，不将其当作索引统计 |

Perplexity 建议使用持续更新的官方 IP 列表并检查请求日志。**证据 A**。[Perplexity Crawlers](https://docs.perplexity.ai/docs/resources/perplexity-crawlers)

Google 的 `Googlebot` 控制 Google 搜索抓取；`Google-Extended` 是用途控制标识，没有独立 HTTP UA，涉及 Gemini 训练及部分 Gemini/Vertex grounding。官方明确后者不影响 Google 搜索收录，也不是搜索排名信号。**证据 A**。[Google 常见爬虫](https://developers.google.com/crawling/docs/crawlers-fetchers/google-common-crawlers#google-extended)

本项目建议：先检查通配 robots 规则及实际 WAF 策略，避免为“增加 AI 权重”无差别开放训练爬虫；也不要假设增加几个 Allow 即可解决基础设施拦截。**证据 C**。

## 4. 内容与可信度：提供可以准确复述、验证的事实

Bing 建议用清晰标题、表格和 FAQ 组织信息，提供例子、数据和出处，并保持内容准确。这些是内容质量建议，不是格式排名保证。**证据 B**。[Bing AI Performance 公告](https://blogs.bing.com/webmaster/February-2026/Introducing-AI-Performance-in-Bing-Webmaster-Tools-Public-Preview)

对 Anki Forge 的内容建议如下，需以实际已发布能力为依据。**证据 C**：

- 明确“是什么、适合谁、提供什么输入输出、有哪些约束”，并保持官网、README、包注册页的名称与主页链接一致。
- 为真实需求提供完整教程，例如从数据生成 Anki 卡片、媒体处理、Node/Python 使用方式；给出可运行的版本化示例及预期输出。
- 提供有事实依据的选型、迁移与兼容性说明，写清何时适用和何时不适用；比较页记录核对日期。
- 性能、格式支持和兼容性等主张附测试条件、版本与可复现方法，避免无依据的“最好”“最快”。
- 公开维护者、许可证、更新记录、仓库和问题反馈入口，帮助读者识别项目来源及维护状态。

不要批量生成只替换关键词的页面、购买操纵排名的链接、伪造评论或向爬虫展示不同的夸张事实。Google 的垃圾内容政策明确覆盖操纵生成式 AI 答案的行为；违反规则可能降低排名或失去展示。**证据 A**。[Google 垃圾内容政策](https://developers.google.com/search/docs/essentials/spam-policies)

## 5. 结构化数据：真实表达实体，不制造推荐信号

JSON-LD 应表达页面可见事实；价格、评分、作者或组织关系不能编造。通过 Rich Results Test 也不保证富结果。**证据 A**。[Google 结构化数据通用规则](https://developers.google.com/search/docs/appearance/structured-data/sd-policies)

应按页面类型评估结构化数据，而不是全站粘贴相同 schema。特别是 `SoftwareApplication` 的 Google 富结果要求包含名称、价格，以及真实评分或评论；若本站不具备后者，不要虚构评分补齐测试器。合法描述软件实体与取得富结果资格应分开判断。**证据 A/C**。[Google SoftwareApplication 指南](https://developers.google.com/search/docs/appearance/structured-data/software-app)

**时效纠正：FAQ rich result 已停止。** Google 更新日志的 2026-05-08 条目写明，该展示自 2026-05-07 起不再出现在搜索结果中；2026-06-15 移除对应文档。FAQ 正文仍可帮助用户理解问题，但不能把补 `FAQPage` 当作获取 Google FAQ 富结果的收益。**证据 A**。[Google Search 文档更新记录](https://developers.google.com/search/updates)

## 6. llms.txt / Markdown：文档可用性收益与搜索排名分开

原提案发布日期为 2024-09-03，访问当日已是 2026-08-10 修改的 v2。它提出提供简短背景、关键 Markdown 链接，供 agent 按需获取详细内容；支持根目录或子路径，并提出 `rel="alternate"` 和 `rel="describedby"` 发现方式。这是社区提案，不是所有搜索平台的共同准入要求。**证据 A（提案自身定位）**。[llms.txt v2 原文](https://llmstxt.org/)

Google Search 明确表示忽略 llms.txt 对搜索排名和可见性的作用；无需特殊 AI 文件或特殊 schema。**证据 A**。[Google AI 优化指南的误区说明](https://developers.google.com/search/docs/fundamentals/ai-optimization-guide#mythbusting-generative-ai-search-what-you-dont-need-to-do)

另一方面，Chrome Lighthouse 已提供 agentic browsing 的 llms.txt 检查，页面更新于 2026-05-05。它仍是可选项，文件不存在返回 404 时记为 N/A。这说明“方便 agent 读取”与“Google 搜索加权”是两个不同目标。**证据 A**。[Lighthouse llms.txt 检查](https://developer.chrome.com/docs/lighthouse/agentic-browsing/llms-txt)

本站可以从现有文档自动生成精简索引及 Markdown 页面，列明安装、快速入门、API、限制、示例、版本和正式网址；用同一份内容源避免维护两套冲突文档。验收采用“仅给文档入口，agent 能否找到正确 API 并完成示例”，不以文件是否存在或其体积衡量效果。**证据 C**。

浏览器 agent 还会读取 DOM、截图和可访问性树。采用真实链接和按钮、明确表单标签、稳定布局，能减少操作歧义；这些属于可用性改进。**证据 B**。[Google web.dev：构建适合 agent 的网站](https://web.dev/articles/ai-agent-site-ux)

## 7. 发现新内容：Bing / IndexNow

IndexNow 适用于向参与的搜索引擎通知新增、实质更新、删除或迁移的 URL；它补充 sitemap，不保证立即索引或排名。官网文档建议按真实变更提交，并保留提交响应日志。不要将它描述为 Google 提交接口，或每次构建都重复提交所有未修改页面。**证据 A**。[IndexNow 官方 FAQ](https://www.indexnow.org/faq)

对本站建议：在发布工作流完成后，根据文档变更集提交正式域名 URL；首次配置、站点迁移等情形按官方规则处理。将“提交被接收”与“URL 已被索引”作为不同状态记录。**证据 C**。

## 8. 衡量效果：2026 年站长工具已有新能力

### Google Search Console

2026-06-03 的官方公告推出独立的生成式 AI 效果报表；公告后加注说明，2026-08-31 已推广到全球网站。数据同时包含在整体效果报表中，因此“只能在 Web 总流量里看 AI”已不是当前完整描述。**证据 A**。[Google 报表发布公告](https://developers.google.com/search/blog/2026/06/gen-ai-performance-reports)

当前帮助页主要明确 **AI 展示次数**，支持页面、国家、日期和设备等维度。不要将它扩写成已提供独立 AI 点击、排名或转化。帮助页仍保留灰度说明，并指出展示不足的站点可能看不到报表；本站实际可见性需登录确认。**证据 A**。[Generative AI performance report (Search)](https://support.google.com/webmasters/answer/16984139)

另外，Search Console 的 Settings → Search generative AI 可控制是否纳入 AI Overviews、AI Mode 及 Discover 相关 AI 展示。默认包括，但子属性可能继承父属性；核对本站没有被排除。它不是其他搜索结果的排名信号，也不控制模型训练。该帮助页同样标注 2026-08-31 全球推出。**证据 A**。[Search generative AI control](https://support.google.com/webmasters/answer/16908024)

### Bing Webmaster Tools

2026-02-10 官方公开预览公告介绍 AI Performance：引用次数、被引用页面、采样的 grounding queries 等，范围包括 Microsoft Copilot、Bing AI 摘要及部分合作入口。公告明确这些数据不表示排名、权威度或单次答案中的位置；不能推算为全网 AI 份额。**证据 A**。[Bing AI Performance 公告](https://blogs.bing.com/webmaster/February-2026/Introducing-AI-Performance-in-Bing-Webmaster-Tools-Public-Preview)

### 本站建议的数据面板

以下为实施建议，非平台保证提供的统一指标。**证据 C**：

| 层次 | 建议指标 | 注意事项 |
| --- | --- | --- |
| 抓取 | 核验后的 bot 请求、成功率、被拦截 URL | UA 可伪造；请求次数不代表推荐量 |
| 索引 | 重要页面索引覆盖、Google 所选 canonical、sitemap 错误 | 不能用一次公开搜索无结果断言全站未收录 |
| AI 展示与引用 | Google AI 展示、Bing 引用页及趋势 | 分平台统计，勿混成“AI 权重” |
| 到站及使用 | AI referral、文档入口、安装命令复制、下载或仓库跳转 | 无点击的引用与无 referrer 的访问可能无法完整归因 |
| 内容准确性 | 固定问题下是否提及项目、是否引用正式域名、是否准确复述功能 | 重复测试、记录日期/平台/语言/是否联网；小样本不是全网排名 |

## 9. 推进建议与边界

| 优先级 | 工作 | 可验收结果 |
| --- | --- | --- |
| P0 | 核对站长平台的实际收录与 AI inclusion；若发现正文、robots/noindex、WAF、域名等障碍再修复 | 重要页面实际可读、URL 信号一致、未被错误排除 |
| P1 | 完整 sitemap、多语言映射、索引诊断及变更提交 | 核心 URL 均有发现路径；提交与索引状态可区分 |
| P1 | 补充高价值教程、版本化示例、可信依据和选型说明 | 用户及 agent 能准确理解适用场景和限制 |
| P2 | 真实结构化数据、精简 llms.txt 与 Markdown 入口、可访问性 | 内容一致、标签不虚构、示例任务可完成 |
| 持续 | 记录基线和发布批次，观察多周趋势 | 能区分技术改善、内容变化、平台波动和业务结果 |

以上排序需要结合实际源码与线上审计调整。没有证据支持“开放训练爬虫即可提高推荐”“schema 越多权重越高”“llms.txt 必然提高排名”“大批量 AI 文章可以稳定抢占答案”“购买服务可以保证被 AI 推荐”等承诺。应把预算投向可访问、可核验、有独特价值且持续维护的内容及测量。
