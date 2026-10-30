# Anki Forge 官网机器读取与结构化数据规格

日期：2026-10-27。状态：待实现。范围：官网双语机器读取入口、技术文档与博客的 Markdown 表示、JSON-LD、构建与发布验收。

本规格将已有静态内容转换为可发现、可按需读取、可校验的机器资料。实现须复用当前页面模型、公开版本配置和发布流水线，不建立另一套独立维护的文档。本文的文件名和新增接口是实施约定；不表示功能已经上线。

## Problem Statement

官网已经提供静态 HTML、robots 和 sitemap，正文及示例可以在不执行 JavaScript 的情况下读取。但机器需要从页面导航和交互控件中提取正文，缺少按任务组织的紧凑入口，也没有明确描述软件、源码、技术文档和博客关系的结构化数据。

部分 MDX 文档通过 `CodeBlock` 和生成数据注入可执行示例。直接公开源 MDX 会留下组件调用、导入和表达式，遗漏读者实际看到的代码。Markdown 必须从本次构建的实际正文导出。

2026-10-27 的线上检查发现 sitemap 有 30 个英文页面，中文首页返回 404；本地代码已经具有双语页面模型。实现以当前源码与实际构建产物为准，发布验收必须核对同一 CI 产物，不能把本地成功当作线上完成。页面数量不得硬编码为这次检查的数量。

## Goals and Non Goals

必须实现：

1. 英文和中文 `llms.txt`，包含项目定位、公开版本、安装方式、使用边界和按任务组织的入口。
2. 所有已发布文档和博客详情的完整 Markdown；保留代码、表格、提示、折叠内容、附件和原始引用目标。
3. HTML 中可发现的机器入口，以及用户可点击的 Markdown 和 AI 文档索引链接。
4. 与可见内容一致的 JSON-LD，区分软件、源码、页面、技术文章和博客。
5. 两种 base path 的构建检查，以及与发布产物逐项比对的线上验收。

本期不包含：`llms-full.txt`、首页和交互示例页的 Markdown、MCP 服务、动态内容协商、按 User-Agent 分流、托管迁移、评分与评论、付费信息、自动生成正文或翻译、搜索平台账号配置。新增文件不是抓取授权规则，不改变 robots 政策。不把 AI 推荐率、搜索排名或富结果展示作为实现完成的保证。

## Existing Sources of Truth

| 来源 | 权威信息 | 使用规则 |
| --- | --- | --- |
| `website/site-model.mjs` | topic ID、页面类型、语言、路由、标题、摘要、来源文件 | 所有站内 URL 由 page ID 和 locale 推导 |
| `website/site.config.mjs` | site、base、repository | 不在生成器中硬编码生产域名和项目路径 |
| `docs/ai-quickstart.json` | publicVersion、各 SDK 安装命令和运行环境 | 公开版本不能取 website/package.json 或开发包版本 |
| Astro blog collection | 博客标题、摘要、date、category、locale、draft | 使用真实文章元数据；草稿不得进入公开产物 |
| 本次构建 HTML | 实际正文、代码、链接、语言和 canonical | Markdown 的内容来源 |
| `website/content-reviews.json` | 双语审核状态 | 保持现有门禁，不把审核日期当内容修改日期 |
| `website/scripts/site-manifest.mjs` | 实际文件清单、哈希、构建标识 | 新资源在 manifest 生成前写入 |

新增 `website/machine-content.config.mjs` 只保存精选入口分组、topic ID、顺序和本地化链接说明。版本、命令、标题、地址不能在此重复维护。项目定位和限制摘要使用已有翻译 catalog 中的专用键；每项限制须能对应到公开指南，不增加未验证的功能声明。

不对全部文档或历史博客自动声明“适用于最新公开版本”。本期只在 llms 入口标明当前公开包版本；单篇文档保留正文中原有版本说明。源码构建和历史文章在入口中明确标识用途。未来需要逐页版本适用范围时，另加经过内容审核的字段。

## Public Resource Contract

### URLs and coverage

以下路径以生产根路径为例；项目部署须统一加上 `siteConfig.base`。

| HTML 或用途 | 机器资源 |
| --- | --- |
| 全站英文入口 | `/llms.txt` |
| 中文入口 | `/zh-cn/llms.txt` |
| `/docs/` | `/docs/index.md` |
| `/docs/quickstart/` | `/docs/quickstart/index.md` |
| `/zh-cn/docs/quickstart/` | `/zh-cn/docs/quickstart/index.md` |
| `/blog/your-first-anki-deck/` | `/blog/your-first-anki-deck/index.md` |
| `/zh-cn/blog/your-first-anki-deck/` | `/zh-cn/blog/your-first-anki-deck/index.md` |

对 `type=docs` 和已发布 `type=article` 的页面，将 canonical 路径末尾追加 `index.md`。不提供另一套 `.md` 别名，不改已有 HTML URL。首页、博客列表、示例页和 404 不生成 Markdown。文档首页有 Markdown，但其 JSON-LD 使用集合类型。

覆盖集合来自注册页面与 Astro 实际发布集合的交叉核验：已发布且未注册的文档/文章、注册为公开但没有构建出来的页面都报错；草稿从公开页面、机器索引和 manifest 的正常页面预期集合中一致排除。若现有检查直接把所有注册条目当公开页面，实施时需同步调整，不可只让机器生成器忽略草稿。

### llms content

固定格式为一个 H1、简短 blockquote、无额外标题的背景段落与安装列表，随后是 H2 分组的链接列表。H2 下每项必须有 Markdown 链接和简短说明。

开头必须包含项目用途、三种 SDK、公开版本、各语言安装命令与运行环境，以及以下有文档依据的边界：正常导出无需安装 Anki；公开包 API 与源码构建有区别；更新需要保留原始旧分发 APKG；Anki 导入设置和较新的本地修改可能影响结果。只描述事实，不包含要求模型优先推荐本项目的指令。

| 分组 | topic ID 或目标 |
| --- | --- |
| Start here | `docs`、`languages`、`compatibility` |
| Quickstarts | `quickstart`、`node-quickstart`、`python-quickstart` |
| API references | `api`、`rust-api`、`node-api`、`python-api` |
| Task guides | `json-to-anki`、`cards`、`media`、`templates`、`image-occlusion`、`updates` |
| Constraints | `build-guarantees`、`troubleshooting` |
| Optional | `anki-forge-vs-genanki`、`genanki-migration`、`development`、已发布博客详情、repository、另一语言 llms |

中文使用本地化分组名，保留最后一组字面名称 `Optional` 以兼容提案约定。两种语言的 topic 顺序一致；站内内容优先链接本语言 Markdown，不回退英文。源码构建标明“开发/源码环境”，博客标明历史文章和发布日期。

入口 UTF-8 编码、LF 换行、末尾一个换行，不插入构建时间或随机值。每个文件上限 24 KiB；超限构建失败，维护者应减少精选链接或缩短说明，不能截断正文或自动删除最后几项。完整目录由文档首页及正文交叉链接承接。

### Markdown format

每个文件依次包含：唯一 H1、Canonical page、Language、Description，博客额外包含 Published，之后为空行和完整正文。元信息标签固定为上述英文名称，值使用当前语言；不增加 YAML frontmatter，不重复输出页面标题。

```markdown
# Generate an Anki APKG with Rust

Canonical page: https://ankiforge.dev/docs/quickstart/
Language: en
Description: Install the public Rust package and export a first APKG.

## Install

...
```

`Canonical page` 是来源说明，不等于搜索引擎 canonical 指令。Markdown 的语言、标题和摘要必须来自与 HTML 相同的页面元数据。正文不得包含未求值的 MDX 表达式或仅存在于源文件的 import/export。

### Discovery and HTTP behavior

正常 HTML 页面输出一个 `rel=describedby` 指向本语言 llms；有 Markdown 的页面另外输出一个 `rel=alternate type=text/markdown`。404 不输出这些发现链接。链接为使用 site/base 计算的绝对 URL，不能与 RSS alternate 相互覆盖。

文档操作区和博客页尾增加“查看 Markdown / View Markdown”；全站正常页页脚增加“AI 文档索引 / AI documentation index”。使用普通 anchor，无 JavaScript 依赖，新增文案进入双语 catalog。

所有新增资源直接返回静态 UTF-8 文件，不重定向到 HTML、不设置登录、不按 UA 改变字节。线上 `llms.txt` 须为 `text/plain`；Markdown 可为 `text/markdown` 或 `text/plain`。忽略 MIME 参数顺序，验证实际字节可按 UTF-8 解码。若 GitHub Pages 返回 `text/html`、`application/octet-stream` 或强制附件下载，则线上验收失败；保留构建产物并报告托管限制，不把本地 Response 的 Content-Type 当成已生效的线上配置，也不擅自迁移托管或改 URL 契约。

sitemap 仍只包含正常 HTML 页面，不把 Markdown 当新主题加入；不增加 robots Disallow、非标准 AI 抓取规则或 Markdown noindex。Pagefind 继续索引 HTML，后生成的 Markdown 不进入搜索索引。

## Markdown Generation

### Extraction boundaries

导出发生在 Astro 完成静态 HTML 构建后。文档适配器选择唯一的 `.sl-markdown-content`；该选择器已用于当前文档样式，但必须在首个真实构建中验证恰好命中一次。它不包含 Starlight 独立渲染的页标题；H1 从页面元数据补充。

博客在 `ArticlePage.astro` 的正文 `.prose` 上增加 `data-machine-content`，提取唯一 `[data-machine-content]`；文章 header 中的摘要与日期单独进入文件头，页尾导航不进入正文。不能使用整页 `main.text()` 作为兜底。

选择器零命中、多命中、正文为空或标题缺失时，报出 topic、locale、HTML 路径并使构建失败。主题升级导致 DOM 改变时，更新适配器与真实 DOM fixture，不能悄悄导出整页。

### Conversion rules

使用成熟 HTML 转 Markdown 库 `turndown` 和 GFM 插件 `turndown-plugin-gfm`，作为构建开发依赖并锁定版本。业务规则在 `machine-content.mjs` 中集中注册。Cheerio 用于提取、归一化、验证。不得以正则移除 HTML 作为转换算法。

| 内容 | 必须保留或转换的行为 |
| --- | --- |
| h2–h6、段落、强调、引用、列表 | 保留语义、顺序、嵌套及列表起始编号 |
| 表格 | 简单表转 GFM，转义竖线；rowspan/colspan 等复杂表保留清理后的 HTML table，绝不丢列或展平 |
| pre/code | 读取实际代码文本，保持缩进、空行、尾随空格；仅允许 CRLF 转 LF，不 trim 代码 |
| 代码语言 | 优先明确的 data-language 或 language-*；自有 CodeBlock 增加语言属性，无法判定时用 text，不能猜测 |
| 代码围栏 | 使用长于代码中连续反引号最大长度的围栏，至少三字符 |
| 高亮代码包装 | 保留源码及实际文件名标签；排除 copy 按钮、行号、装饰、live status |
| 提示框 | 保留可见类型/标题和全部文字，转换为引用段落 |
| Tabs | 按 DOM 顺序展开所有 panel，输出对应标签；隐藏状态不构成删除理由 |
| details/summary | 输出 summary 和全部折叠内容，无论 open 状态 |
| 图片 | 保留 alt、title（若有）和绝对 src；链接图片保留外层链接 |
| audio/video | 保留说明、可用 source 和下载链接，不生成虚构转录 |
| 页面导航与控件 | 仅移除明确登记的复制、锚点装饰、导航和状态控件 |
| 普通 div/span | 递归保留子内容；不能因未知 class 丢弃整个节点 |
| 语义不支持的 SVG、canvas、iframe、交互组件 | 没有明确文本/资源替代规则时失败，要求新增适配器与 fixture |

不能全局删除 `[hidden]` 或 `[aria-hidden]`，以免丢失 Tabs 和折叠内容。允许丢弃的装饰节点须在适配器中具名登记；所有其余文本和语义资源必须被转换。空布局包装可忽略。

站内链接保持指向 HTML canonical，不统一改写成 Markdown。以当前 HTML canonical 为基准通过 URL 解析器解析相对 href/src；页内 `#section` 变为原 HTML 页的绝对锚点链接，query/hash 原样保留。不重复添加 base。mailto 等合法非 HTTP 链接保留；javascript 链接不能成为输出内容。

### Files and determinism

导出仅写入本次 `website/dist/`，不回写 authored sources 或 `public/`。输出路径由注册路由计算，并验证最终路径在 dist 内；不得根据正文链接任意创建文件。

生成器先完成全部提取、转换和验证，再写文件。每次正常 Astro build 产生干净 dist；单独运行生成命令时，按固定 `docs/**/index.md`、`blog/**/index.md` 及中文对应命名空间清理旧导出，并拒绝与人工资源冲突。这些命名空间保留给生成器，不允许手工 Markdown 文件。

同样输入的两次生成结果必须逐字节一致。不要加入生成时间、绝对工作区路径或本地用户名。外部网站不可用不应影响构建；URL 存在性检查限本站构建资源，外链检查另行人工完成。

## Structured Data

### Entity identity and page types

每个正常 HTML 页面输出一个由本站管理的 `script[type=application/ld+json][data-anki-forge-schema]`，内容为 `@context: https://schema.org` 和 `@graph`。所有 JSON-LD 块都需要检查是否重复声明同一实体的冲突值，包括框架自行添加的块。

定义 `siteRoot = new URL(withBase('/', base), site).href`。所有站点级 ID 使用 siteRoot，页面级 ID 使用其完整 canonical，不含 query/hash。

| 对象 | 类型与 ID |
| --- | --- |
| 站点 | WebSite，`siteRoot + '#website'` |
| 软件 | SoftwareApplication，`siteRoot + '#software'` |
| 源码 | SoftwareSourceCode，`siteRoot + '#source'` |
| 页面 | WebPage 或 CollectionPage，`canonical + '#webpage'` |
| 文章 | TechArticle 或 BlogPosting，`canonical + '#article'` |

| 页面类型 | graph 内容 |
| --- | --- |
| home | WebSite、WebPage、完整 SoftwareApplication、SoftwareSourceCode |
| docs 且 id=docs | WebSite、CollectionPage、最小软件引用节点 |
| 其余 docs | WebSite、WebPage、TechArticle、最小软件引用节点 |
| article | WebSite、WebPage、BlogPosting、最小软件引用节点 |
| blog-list、examples | WebSite、CollectionPage、最小软件引用节点 |
| 404 或 noindex | 不输出本站 JSON-LD |

第一期集合页不输出 ItemList，避免额外维护列表状态；不输出 BreadcrumbList，因本期未增加可见面包屑。软件的最小引用节点只包含 @id、@type、name，不能把完整安装及版本信息复制到不展示它们的每一页。

### Fields and relationships

WebSite 使用稳定 name、url 与 `inLanguage: ['en', 'zh-CN']`。WebPage/CollectionPage 使用 name、description、url、inLanguage、isPartOf；软件相关页用 about 指向软件。首页 mainEntity 指向软件，文章页 mainEntity 指向文章。

SoftwareApplication 的 name 为 Anki Forge，url 使用 siteRoot，applicationCategory 为 DeveloperApplication，softwareVersion 来自 publicVersion，license 链接到仓库 LICENSE，softwareHelp 指向当前语言文档首页。description 使用当前页可见介绍；中英文允许自然语言差异，版本、许可证和能力事实必须相同。

SoftwareSourceCode 包含 name、codeRepository、license、targetProduct，其中 targetProduct 引用软件。第一期不输出 programmingLanguage、runtimePlatform 或源码 version，避免将 SDK 支持语言误当源码实现语言、将 main 误当已发布版本源码。也不填 operatingSystem、offers、aggregateRating、review、Organization 或未经确认的 author/publisher。

TechArticle 与 BlogPosting 必须有 headline、description、url、inLanguage、mainEntityOfPage、about。headline 等于可见 H1；mainEntityOfPage 引用当前页面；about 引用软件。可通过 encoding 嵌入 MediaObject：`contentUrl` 为 Markdown 地址，`encodingFormat` 为 `text/markdown`，`inLanguage` 与文章一致。该字段描述文件表示格式，不替代线上 HTTP MIME 验证。

BlogPosting 的 datePublished 使用现有 date，articleSection 使用当前语言显示的 category。第一期不新增 author、dateModified、image 字段；没有真实可见署名、修改日期、文章图片时省略，不用 logo、通用 OG 图片或构建日期补齐。普通技术文档无已维护日期时省略日期。

英文与中文有不同页面/文章 ID，共享站点、软件、源码 ID。保留已有 self canonical 和 reciprocal hreflang；本期不添加 translationOfWork，因为现有 translationKey 只表明对应关系，不证明翻译方向。

### Rendering and serialization

`structured-data.mjs` 为纯数据构造模块，不读取文件、不修改全局状态、不执行网络请求。普通页面从 SiteLayout 调用，文档在 Starlight route middleware 中调用。文章通过 ArticlePage 将真实 post 元数据传给 layout；不从格式化后的日期字符串反推时间。

所有模板通过统一 `serializeJsonLd` 输出。先 JSON.stringify，再把 `<` 编码为 `\u003c`，并统一转义 U+2028/U+2029；不得用 HTML entity 替代 JSON 字符串转义。必须验证包含 `</script>`、引号和中文的标题往返解析一致，且不会产生额外 DOM 元素。

只允许包含本页面有依据的信息；软件描述/许可证等如未在相应首页可见，应先补齐对应可见说明再输出字段。发现链接与 JSON-LD 在服务器生成的 HTML 中存在，不等待浏览器执行脚本。

## Implementation Boundaries

建议按以下文件职责实施；允许拆分纯工具函数，但不能创建第二套内容注册表。

| 文件 | 职责 |
| --- | --- |
| `website/machine-content.config.mjs` | llms 精选 topic 分组及本地化说明键 |
| `website/src/lib/machine-urls.mjs` | Markdown、llms、canonical 和实体 ID 的统一计算 |
| `website/scripts/machine-content.mjs` | 导出函数与命令行入口，实际 HTML 转 Markdown、生成 llms |
| `website/src/lib/structured-data.mjs` | graph 构造与 JSON 安全序列化 |
| `website/scripts/check-machine-content.mjs` | 独立读取实际 dist，验证覆盖、代码和发现链接 |
| `website/scripts/check-structured-data.mjs` | 独立检查实际 HTML graph 与可见元数据 |
| `website/src/layouts/SiteLayout.astro` | 公共页面发现链接与 JSON-LD |
| `website/src/route-middleware.ts` | Starlight 页面发现链接与 JSON-LD |
| `website/src/components/ArticlePage.astro` | 正文标记、文章元数据、Markdown 可见入口 |
| `website/src/components/DocsFooter.astro`、`Footer.astro` | 文档 Markdown 入口、双语索引链接 |
| `website/src/components/CodeBlock.astro` | 显式语言属性，保留现有复制行为 |
| `website/src/i18n/en.json`、`zh-CN.json` | 新增可见文案及 llms 摘要 |
| `website/package.json`、lockfile、workflow | 构建步骤、依赖、测试和门禁 |
| `website/scripts/check-live-site.mjs` | 新资源 MIME、正文哈希、发现与 graph 验证 |
| `website/README.md` | 维护入口配置、预览方式和验收命令 |

核心接口的输入与输出约束：

```typescript
type Locale = 'en' | 'zh-CN';
type SiteConfig = { site: string; base: string; customDomain: string };
type PageMetadata = {
  id: string;
  type: 'home' | 'docs' | 'article' | 'blog-list' | 'examples';
  locale: Locale;
  canonical: string;
  title: string;
  description: string;
  publishedAt?: string;
  category?: string;
};

markdownUrl(page: PageMetadata): string | null;
llmsUrl(locale: Locale, config: SiteConfig): string;
extractDocument(html: string, page: PageMetadata): ExtractedDocument;
renderMarkdown(document: ExtractedDocument): string;
renderLlms(locale: Locale, publishedDocuments: PublishedDocument[], context: BuildContext): string;
buildStructuredData(page: PageMetadata, context: SiteContext): JsonLdGraph;
serializeJsonLd(graph: JsonLdGraph): string;
```

上述为接口语义，实际可用 JSDoc 类型。ExtractedDocument 保存标题、元数据和带语言的有序代码块等语义信息；PublishedDocument 包含 topic、locale、HTML URL、Markdown URL 和实际存在的输出。BuildContext/SiteContext 显式传入 siteConfig、公开安装配置和文案，不隐式重读另一份版本。

验证器不能调用生成器后将其输出当作唯一期望值。必须独立从实际 HTML、原始代码 fixture 和站点注册表获取预期，才能发现生成与模板共同出错。

## Build and Deployment

新增 npm scripts：`generate:machine`、`check:machine`、`check:schema`。build 顺序改为：

```text
npm prebuild → prepare:site
astro build
node scripts/machine-content.mjs
node scripts/check-machine-content.mjs
node scripts/check-structured-data.mjs
node scripts/site-manifest.mjs
```

上述任一步失败即停止，不能产生可用于发布的成功 manifest。保留原有 check:site、check:gates、test:browser、SDK 示例与公开安装检查。将机器和 schema 故障注入检查接入现有 check:gates，而非替换旧门禁。

CI 必须在项目路径和生产根路径构建中各执行一次新增检查。生成 Markdown 时 Astro 与 Pagefind 已完成，无需重新构建或把机器文件放进 src/content。manifest 的现有 resources 扫描自动包含 `.md` 和 `.txt`，不另建重复版本清单。

开发服务器不承诺提供构建后生成文件。HTML 中仍可展示目标链接，README 明确机器资源须通过 build + preview 验证；不能为了 dev 体验添加第二个运行时转换器。

部署后继续比对独立上传的 verified manifest、HTML 及所有 resources 的哈希。扩展请求辅助函数以保留 Content-Type/Content-Disposition，验证新增资源类型、无附件强制下载和无重定向。新机器资源的不存在路径须返回真实 404，不能回退首页。旧版 HTML、旧版 llms、新版 Markdown 混用时必须失败。

“实现完成”要求本地和 CI 验收通过；“发布完成”额外要求同一 manifest 的线上校验通过。发布权限或托管问题不影响保留已完成代码，但不得把发布状态标为通过。

## Acceptance Tests

| ID | 情景 | 必须满足的结果 |
| --- | --- | --- |
| M01 | 遍历真实构建页面 | 文档/文章的发布集合与 Markdown 一一对应；无漏页、草稿或孤立旧文件 |
| M02 | 同一输入重复生成 | 所有新增文件逐字节相同 |
| M03 | 根路径与项目路径 | HTML、Markdown、llms、实体 ID 都正确带 base；无生产域名泄漏到项目配置 |
| M04 | Rust quickstart 和两篇博客的动态 CodeBlock | Markdown 代码与实际 HTML code 一致，另与可执行源 fixture 核对，不能只验证“非空” |
| M05 | 代码包含中文、HTML、反引号、空行及尾空格 | 围栏解析后代码文本完整往返，仅 CRLF 可转 LF |
| M06 | GFM/复杂表格、嵌套列表、提示、Tabs、details | 所有文本、列、标签和隐藏面板内容均在输出中，控件文案排除 |
| M07 | 图片、附件、相对链接、query 和 fragment | 保留语义；绝对 URL 对应真实本地资源和 HTML 锚点 |
| M08 | 内容选择器缺失/重复或未知有意义组件 | 构建失败，并定位 topic、locale 和节点 |
| M09 | llms 与中英文发现链接 | 精选集合和顺序对应，无语言回退，公开版本/安装命令一致，入口大小合规 |
| M10 | 删除一篇 Markdown 或破坏一个入口 | check:machine 独立失败，不因生成配置自洽而通过 |
| S01 | 各页面类型 | graph 类型、数量、ID、关系匹配契约，404 无本站 graph |
| S02 | 篡改版本、headline、canonical 或 inLanguage | check:schema 对照独立权威来源失败 |
| S03 | 日期与历史内容 | 博客发布日期真实；文档无伪造日期；没有统一追加的“最新版本适用”标签 |
| S04 | 标题含 script 结束标签、引号、Unicode | HTML 不新增可执行节点；JSON.parse 恢复原始文本 |
| S05 | 软件/源码/文章与双语关系 | 相同软件 ID，语言页面 ID 不同；无软件版本与开发版本混用 |
| D01 | 禁用 JavaScript 的实际页面 | 正文、机器发现链接、JSON-LD 和可点击 Markdown 入口均存在 |
| D02 | 真实静态预览 | Markdown/llms 为 UTF-8 文本，无 SPA 回退；新增下载和原有页面交互不受影响 |
| D03 | 线上同一 verified manifest | 所有新增资源 200、MIME 合规、哈希匹配、未知机器路径 404 |
| D04 | 线上返回旧内容或错误 MIME | 发布验证明确失败，不被成功首页请求掩盖 |

单元测试使用小型真实 DOM fixture；集成检查使用实际完整 dist。新增测试加入 npm test，破坏产物的负向检查加入 check:gates，检查后的产物须恢复或重新构建，不能发布受损 fixture。

Schema.org Validator 用于人工确认词汇与属性适用性；Google Rich Results Test 仅检查其支持的展示类型。缺少富结果资格不是 TechArticle 语义无效。外部工具不作为需要凭据或网络的单元测试依赖。

额外执行一次人工读取验收：只给工具 llms URL，要求定位 Python 安装、Node API、JSON 更新和兼容性限制，检查引用目标与版本是否正确。保存问题、读取 URL、答案和人工结论，不把模型输出完全一致或推荐概率作为 CI 门槛。

## Implementation Sequence

1. **定义路由与提取契约。** 新增 URL 工具、精选入口配置，核对实际 Starlight 和博客 DOM，建立代码、Tabs、表格和提示 fixture。交付可测纯函数，确认既有命名空间无人工文件冲突。
2. **实现 Markdown 导出。** 加入转换依赖、语言标记、正文标记、清理规则和独立验证；完成 M01–M08、M10。此步不发布指向缺失资源的入口。
3. **实现双语 llms 与发现。** 接入公开安装配置和 catalog，添加 head 及可见链接，完成 M09、D01、D02。英文与中文同时进入同一构建。
4. **实现结构化数据。** 接入两个布局通道、真实博客元数据、安全序列化和独立验证，完成 S01–S05；检查可见信息依据。
5. **接入发布门禁。** 调整 build 顺序和 npm test，完成两种 base 的现有回归及新增故障注入，扩展 live checker 和 README；部署后完成 D03–D04 与人工读取验收。

每步可独立提交，公开发现链接只能随对应资源一起发布。不得修改或清理本任务之外的工作区变更。

## Definition of Done

- [ ] 所有机器资源与结构化数据来自本次已校验的内容与配置。
- [ ] 中英文覆盖、代码完整性、路径计算、负向门禁及 JSON 安全测试通过。
- [ ] 既有 Astro 检查、站点检查、浏览器测试和公开 SDK 发布门禁保持通过。
- [ ] README 说明维护入口、生成命令、预览限制与 MIME 验收方法。
- [ ] 发布记录区分本地、CI、线上结果，附 verified manifest 标识。
- [ ] 线上新增资源与已验证构建一致，人工读取验收有可复核记录。

## References

- [llms.txt 提案](https://llmstxt.org/)：入口格式、按页 Markdown 和发现链接；不是 robots 授权规则。
- [Schema.org SoftwareApplication](https://schema.org/SoftwareApplication)、[SoftwareSourceCode](https://schema.org/SoftwareSourceCode)、[TechArticle](https://schema.org/TechArticle)、[BlogPosting](https://schema.org/BlogPosting)：实体类型及字段语义。
- [Google 结构化数据通用规范](https://developers.google.com/search/docs/appearance/structured-data/sd-policies)：可见内容一致性、实体关联与真实性要求。
- [Google AI 搜索与网站](https://developers.google.com/search/docs/appearance/ai-features)：无需专用 AI 文件或特殊 Schema 才能参与 AI 搜索展示。
- [官网中英文全站一致性规格](2026-10-26-website-i18n-parity-spec.md)：既有双语、路由和发布验收约束。
