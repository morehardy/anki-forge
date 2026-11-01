# Anki Forge 剩余五种目标语言实施方案

日期：2026-11-01，America/Los_Angeles。实施基线：远端 `main@ad98a6f5b9293e7657429e3ed423d11e55259a74`。本方案承接[原多语言规格](2026-10-27-multilingual-implementation-spec.md)，将后续工作收敛为 **巴西葡语 `pt-BR` → 日语 `ja`／德语 `de` → 法语 `fr`／俄语 `ru`**。共享多语言架构和西语内容已经进入主线；后续以完整语言包、少量已证实的测试修复、独立审校与逐语发布为主。

用户此前已确认的六语范围、独立 AI 审校、每语完整后发布继续有效。本轮交付实施方案，不启用新语言或触发部署。子代理审计、grilling 质疑与证据边界见[本轮讨论记录](evidence/remaining-locales-2026-11-01/discussion-record.md)。

实施补记：用户随后调用 `implement`，已按确认的首批范围实现巴葡完整候选及日德样稿。当前进展和真实验收见[首批验收记录](evidence/multilingual-pt-br-2026-11-01/acceptance.md)。下文基线表与“本轮未修改”描述保留制定方案时的历史状态；实现遵循后续调用要求提交到当前分支，另保留可先独立合入主线的P0维护分支。

## 一 最新代码基线

| 对象 | 本轮核对结果 | 对实施的影响 |
| --- | --- | --- |
| 当前主工作区 | `codex/multilingual-common@1e55ce1`，只有 en／zh-CN；本地 main 和 origin/main 也未反映最新远端 | 不在这个旧分支上直接开始新语言实现 |
| 最新远端主线 | [PR 65](https://github.com/morehardy/anki-forge/pull/65) 已合并，合并提交 `ad98a6f`；远端 releaseLocales 为 en／zh-CN／es | 西语不再属于待翻译范围 |
| 可复核实现 | 本地西语集成 `3b21732` 与上述远端 main 比较，文件差异为零，仅多一个合并提交 | 本轮代码审计使用该集成树，未切换或修改其他工作树 |
| 西语验收 | [集成验收记录](https://github.com/morehardy/anki-forge/blob/3b2173245f1b87aaf94681af83df29642b0af2c4/docs/plans/evidence/multilingual-es-integration-2026-10-31/acceptance.md)记录双配置本地完整通过；PR 记录候选提交 CI 通过 | 可复用其实现与流程；不能把旧证据直接用于新增语言的产物 |
| 生产状态 | 本轮未取得线上 manifest 与独立 verified artifact 的一致性证明 | P0 核对合并后部署；“已合并”不写成“已验证上线” |

旧文档中的以下工作已完成：locale 注册与发布集合分离、动态词典、语言映射元数据／导航／侧栏、通用路由、稳定章节、语言菜单、各语搜索目录、HTML 派生 Markdown、llms、JSON-LD、双配置发布检查及独立 AI 证据校验。西语集成还已泛化原中英浏览器断言。保留这些实现，不再次重写路由、翻译框架、机器导出或审核系统。

当前真实规模为每语 **34 个正常 HTML：29 篇文档、2 篇博客、首页／示例／博客列表**；机器资源为每语 31 份 Markdown 与 1 份 llms。业务词典为 210 键，框架／搜索提示为 11 键。旧文档的 32 主题、189／204 个业务键仅保留为历史盘点。

## 二 范围与交付批次

| 阶段完成后 | 新增语言 | 发布语种数 | 正常 HTML | Markdown | llms | 特有验收重点 |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| 当前代码 | 已有 en／zh-CN／es | 3 | 102 | 93 | 3 | 确认生产基线及西语历史保护 |
| 第一语 | pt-BR | 4 | 136 | 124 | 4 | 地区代码、目录大小写、巴葡表达、长语言名称 |
| 第二批第一语 | ja 或 de | 5 | 170 | 155 | 5 | 按对应语种独立验收 |
| 第二批完成 | ja、de | 6 | 204 | 186 | 6 | 日语搜索／字体，德语复合词／长文案 |
| 第三批第一语 | fr 或 ru | 7 | 238 | 217 | 7 | 按对应语种独立验收 |
| 最终 | fr、ru | 8 | 272 | 248 | 8 | 俄语数量表达，八语菜单与完整回归 |

同批两语各自完整即可发布，不绑定同一天。总增量为 170 个正常页面、155 篇长文、160 个机器文本资源、1,050 个业务词典条目和 55 个框架条目；这些数字不包含元数据、提示模板和审核证据，也不表示工时。机器正文由 HTML 生成，不增加手工翻译副本。404 单独计数，后续新增主题时按 registry 重算。

发布目标保持 `/pt-br/`、`/ja/`、`/de/`、`/fr/`、`/ru/`，英文根路径与现有中文、西语路径稳定。巴葡使用 `pt-BR`／`pt_BR`／`pt-br` 各自正确的 locale、OG 和目录表示，不扩展葡萄牙葡语或其他国家副本。印尼语、韩语及 RTL 语言不进入本批。

本轮用户已确认：先由巴葡完成一次最新流程，日德并行准备术语和五类样稿；巴葡收口后再扩大下一批全文生产。共享配置集成与正式发布逐语进行。

## 三 必须先处理的工程问题

### P0 核对生产与推进历史保护

从最新 main 创建实施分支前，记录 commit、releaseLocales、上一成功 Website workflow、独立上传的 verified manifest 和对应 Pages artifact。若西语合并后的部署尚未通过，先完成其原有发布闭环；其他语种的编辑可以继续。

现有 `public-surface-baseline.json` 仍捕获自 `732a40f`，`machine-surface-baseline.json` 来自 `2c07690`，都不是三语公开面。`trusted-surface.mjs` 只读取不可变 base revision 的文件，**不会随新语言上线自动推进基线**。仅把 es 放进 releaseLocales，不能证明后续误删 es 页面会受到历史合同保护。

生产核验成功后，使用 `scripts/public-surface.mjs` 的 `captureSurface(dist, verifiedManifest, provenance)` 从独立下载的同一根路径产物捕获三语公开面。保留 workflow／artifact 来源与摘要；先证明新产物仍满足旧公开面及合法 heading migrations，再推进 `public-surface-baseline.json`。保留旧机器基线与历史证据，新的完整 public baseline 同时包含新增机器资源，不能通过覆盖旧记录批准删页。

基线推进应作为独立、可复核的维护提交先进入主线，使下一语 PR 的 immutable base 确实包含上一语。该提交改动构建输入，需遵守现有构建与证据校验，不能只提交 JSON 就宣称发布完成。每语上线后重复此闭环；它的额外验收成本单列，不藏在翻译工作量里。仅维护基线而没有新增公开面的提交，不再次触发基线捕获，避免为了更新来源提交号形成循环。

### P1 修复两个依赖未发布语言的测试

| 已核实问题 | 最小修改 | 回归验收 |
| --- | --- | --- |
| `content-contract.test.mjs` 将 pt-BR 固定当作 inactive，改其 label 后断言页面指纹不变；巴葡启用即不成立 | 在临时测试 registry 注入专用合成 planned locale，仅修改该条目；保留 active 菜单变化必须失效的断言 | 三语、巴葡已启用及八语集合均验证 active 会失效、planned 不影响当前审校 |
| `third-locale.test.mjs` 顶层要求 configured 中永远有尚未发布语言；八语全开时找不到 | 临时副本自行注入合成 locale，并使用它构造额外语言数据和路径 | 不依赖生产 registry 留空位，仍证明下一语言只改数据即可接入 |

合成语言只存在于临时 fixture，不为测试新增第九种真实产品语言。当前审计已用隔离探针验证两个前提错误；本轮未修改测试代码。修复在巴葡候选冻结前完成。

### P2 用样稿验证语种风险

| 语言或场景 | 先做的探针 | 出现失败时的处理 |
| --- | --- | --- |
| pt-BR | localeId 与路径、OG、搜索资源、提示合同、长名称；巴葡术语及重音输入 | 修正对应数据映射，不增加葡语默认回退 |
| ja | 无空格任务句、汉字与假名变体、技术词混排；浏览器及社交图字体；UTF-8 索引大小 | 对实际失败增加回归，定向调整分词／概念或字体，不预先换搜索引擎 |
| de | 复合词和自然任务句；长按钮、表格、导航和标题 | 先校正术语与检索语义；布局按实测放宽换行 |
| fr | 撇号、连字／重音、同义表达；自然技术指令 | 保留生产词汇与独立题集边界，验证归一化不会造成误命中 |
| ru | 西里尔词形变化、拉丁 API 混排、概念词子串误命中、数量短语 | 以正反例证明词界和数量表达正确；不假定英文 stemming 适用 |
| 八语菜单 | 320／390px、短视口、200% 文字缩放、键盘和无脚本访问全部候选项 | 若有不可达项，补最小高度／滚动约束，不重设计 Header |

样稿阶段在隔离 fixture 中检查分词、概念归一化与布局；完整语言数据齐备后再执行真实 Pagefind 和30题发布验收。当前 `src/lib/search.mjs` 已有 `Intl.Segmenter`，自定义 stemming 仅用于英文，概念别名边界主要针对拉丁文字；这是需要探针的依据，不是已经证实其他语言失败。`LanguageSwitcher.astro` 当前没有菜单高度和滚动上限，三语通过不等于八语通过。每语 llms 保持 24 KiB UTF-8 字节上限，超限精简摘要并复审，不截断或无依据提高额度。

## 四 每语必须交付的数据包

| 文件或来源 | 每语工作 | 保护边界 |
| --- | --- | --- |
| `docs/<prefix>/` | 29 篇文档，保留明确的 section marker 与任务依赖 | 翻译源码文档，不编辑生成的 Starlight 内容 |
| `website/src/content/blog/<prefix>/` | 2 篇 MDX，填写本语标题、描述、正文和 locale | 保留 translationKey、日期、类别；历史文章不自动升级公开版本 |
| `website/src/i18n/<localeId>.json` | 210 个业务键，包括首页／示例／列表／404／机器摘要／交互状态 | key、插值参数与复数形式通过现有检查 |
| `website/src/content/i18n/<localeId>.json` | 11 个框架与搜索提示键 | 缺译不能用英文 fallback 满足合同 |
| `website/site-model.mjs` | 29 组文档 title／description、5 个侧栏组标签、3 个导航标签 | 新增 locale map 数据，不新增语言条件分支 |
| `website/language-briefs.json` | 受众、语体、术语、禁用混淆、保护标识 | 区分 note／card／deck、原始 APKG 更新基线、公开 API 与源码能力 |
| `docs/ai-quickstart.json` | prompts、requestPlaceholders、promptContracts 三处增加本语 | 一个参数模板服务三 SDK；包名、版本、代码标识与协议标签保持真实 |
| `website/search-terms.json` | 31 个 docs／article 的自然主题词 | 有正文与任务依据，不按测试问题硬编码路由 |
| `website/search-concepts.json` | 本语概念、同义词与 stopwords | 结合真实检索缺口补充，不机械直译英文词表 |
| `website/search-queries.json` | 独立编写并冻结 30 条查询 | 12 核心、8 术语、5 变体、5 负例；不是搜索量统计 |
| `website/content-reviews.json` | 当前全部发布语言的有效源审校索引 | 新语全文双轴审校，旧语按真实输入变化增量复审 |
| `website/post-build-reviews.config.json` | 预先确定本批 root／prefix 的报告路径 | 在最终构建前冻结路径；构建后只写外部报告 |
| `website/release-locales.json` | 在完整候选分支中加入本语 | 一次只增加本批完整语言，不把未完成语言带入正式产物 |

现有 registry 已配置五语，无需重复注册。新语初次发布没有历史锚点，不复制中文的 `legacy-sections.json` 别名；沿用稳定 section IDs，在正文正确任务处放置标记即可。Markdown、llms、RSS、OG、sitemap、hreflang 与搜索目录均从相同发布集合自动生成。

## 五 编辑与集成顺序

1. **准备语言 brief 和五类样稿。** 首页／UI、一个 quickstart、JSON recipe、更新页、密集 API 页。技术审校核对原始 APKG、媒体边界、图像遮挡坐标及三 SDK 的公开能力；表达审校先独立盲读，再与源文对照。
2. **完成本语全部内容与数据。** 样稿通过后扩展到 34 个主题，并补齐博客、元数据、搜索、提示模板及错误状态。源版本发生变化时先归并差分再审校，不继续翻旧内容。
3. **单一集成人组装完整候选。** 作者可分语种并行修改独立文件或提交数据 patch；`site-model.mjs`、共享 JSON、审核索引、发布集合和构建输出由集成人串行处理。
4. **冻结 releaseLocales 与报告位置，再生成源审校快照。** `ai-quickstart.mjs` 要求三个提示对象的 keys 与 releaseLocales 的集合及顺序一致，`check-content.mjs` 要求各主题审核语言集合一致。因此不先向三语 main 塞入五语提示数据，再逐个启用。
5. **取得当前输入上的技术与语言审核，修订并独立复审。** 完成后进入严格构建和构建后验收。任何后续源修改按指纹重新判断影响范围。

`CONTENT_REVIEW_MODE=preview` 只跳过源审校门槛，仍要求完整来源、词典、章节和提示集合，以及覆盖全部主题／语言的审核索引，且不能产出 release manifest。若先做编辑预览，应在候选分支补齐明确为 pending 的审核条目，不能伪造 reviewed 或证据。五类样稿的初审可以直接读原稿或使用临时展示 fixture；不要为了局部样稿预览向生产内容填英文占位，也不要承诺现有 preview 能直接构建残缺语言。完整内容就绪后才执行该语言的整站编辑预览与真实 Pagefind 验证。

工作树应基于最新 main，使用 `codex/locale-<prefix>` 一类分支。每个构建工作树独立执行 `npm ci`，使用普通目录，不能把其他工作树的整个 `node_modules` 符号链接进来；西语集成已实际遇到这种路径差异导致的 Astro HTML／CSS 哈希不一致。每个工作树的 dist、target 和验收报告分别隔离。

## 六 独立审校和验收成本

技术轴与表达轴由内容作者以外的代理完成，记录具体问题、输入快照和修订结果。表达的 terminology、syntax、register、usability 四项分别达到至少 4/5，无未关闭 major／critical。minor 观察可按既有规则保留，不编造“零问题”。沿用独立 AI 审校，不增加母语真人审批。

新增语言会使旧语言审核也失效：语言菜单进入每页 sourceHash／targetHash；整份 `docs/ai-quickstart.json` 进入技术依赖摘要。新语审核完整正文，旧语审阅真实差分、共享菜单／提示依赖及受影响功能，不需要机械重译。不得只刷新摘要或搬用针对旧输入的报告。

构建后报告绑定 inputSha256、configuration、releaseLocales 和资源摘要，因此三语报告不能直接批准四语产物。每次对**当次全部语言**执行六个使用任务与四个机器读取任务，分别覆盖 root 和 prefix：

- 使用任务：Rust、Python、Node 基础制卡，JSON／媒体批量任务，原始 APKG 基线更新，排错与恢复。
- 机器任务：从本语 llms 实际定位 Python 安装、Node API、JSON 更新、兼容性边界，记录读取 URL、回答、引用和技术复核。

若五语各自发布，语言数依次为 4、5、6、7、8。仅这五次语言候选累计涉及 `34 × (4+5+6+7+8) = 1,020` 个页面语言审核条目，其中 170 个是新增内容，850 个是既有页面增量复核；双配置累计 360 次使用任务和 240 次机器读取，共 600 个场景。两轴审核、修订轮次、P0 基线推进、额外修复和重新构建的成本另计。这是当前合同的工作量下界，不是预计工时。

按本方案独立推进基线，三语启动维护加四至八语的发布后维护，还需 `20 × (3+4+5+6+7+8) = 660` 个构建后场景；连同语言候选合计至少 1,260 个。仅更新 baseline JSON 不导致正文源审校失效，但全局 inputSha 改变，旧构建后报告不能直接复用。此成本来自现有证据合同；任何以后缩小复核范围的优化应单独证明安全性，不能在翻译批次中静默放宽。

可复用的是输入与依赖确实不变的 SDK 执行证据，以及校验器明确允许且独立验证的字节相同资源。新增语言改变全局输入时，不把这种有限复用扩展成旧产物任务报告整体复用。首个巴葡批次记录真实耗时与返工，再估算后续；不沿用早期人工团队的 6–8 周估算。本轮不把审核指纹粒度重构混入扩语，以免同时改变质量门槛。

## 七 每语可执行检查顺序

以下命令在新实施工作树的 `website/` 运行，目录占位符 `<batch>`／`<verified-dir>` 需替换。Node 24、Rust 1.92 及浏览器依赖按工作流准备；先核对 `node --version`，不能只假定 shell 使用仓库要求的 Node。

```sh
npm ci
npm run check:ai
npm run check:docs
node ../skills/anki-forge/scripts/verify_quickstart.mjs
mkdir -p ../docs/plans/evidence/<batch>
node scripts/content-review-candidates.mjs ../docs/plans/evidence/<batch>/source-candidate.json
```

候选报告只创建 pending 条目，不生成 passing 审核。编辑阶段完成上述完整来源与 pending 索引后，可用 `CONTENT_REVIEW_MODE=preview npm run check` 做结构检查；最终必须执行严格检查。独立评审完成当前输入快照后，分别导入两轴报告：

```sh
node scripts/record-content-review.mjs ../docs/plans/evidence/<batch>/technical-source-review.json
node scripts/record-content-review.mjs ../docs/plans/evidence/<batch>/linguistic-source-review.json
npm run check:content
npm test
npm run check:examples
npm run check
```

首次构建前用 `cargo fetch --locked --manifest-path ../Cargo.toml` 准备示例所需的锁定依赖，Chromium 与字体按现有 `setup-browser-ci.sh`／本机对应环境准备。公开包执行使用 `npm run check:public-recipe`；复用执行证据必须满足第六节的输入规则，CI 的公开包任务仍保留。

使用 CI 相同的不可变 base revision 选择可信公开面；本地调用 `trusted-surface.mjs` 后应显式设置其输出目录对应的 `PUBLIC_SURFACE_BASELINE` 与 `MACHINE_SURFACE_BASELINE`。脚本在 CI 通过 GITHUB_ENV 设置它们，不会自动修改本地父 shell 环境。

两套环境分别完成下列流程，不能一个配置的报告用于另一配置。先 prefix，完成后在构建输入之外归档受审 dist 与原始 manifest；再构建 root 并归档。两配置共用的 dist 会被覆盖，独立审校必须读取对应产物，最终上传验收通过的 root 产物。

| 环境 | SITE_URL | SITE_BASE | SITE_CUSTOM_DOMAIN | SITE_DEPLOYMENT |
| --- | --- | --- | --- | --- |
| prefix | `https://morehardy.github.io` | `/anki-forge` | 空字符串 | `preview` |
| root | `https://ankiforge.dev` | `/` | `ankiforge.dev` | 与最终候选工作流一致；PR 为 preview，main 为 production |

```sh
npm run build
npm run check:rebuild
node scripts/release-evidence.mjs run --engineering-only
```

在实际产物上完成全部当前语言的独立使用与机器读取报告，写入构建前已经确定的外部证据路径，再运行：

```sh
node scripts/release-evidence.mjs finish
npm run check:release
```

工程阶段实际运行八项检查：content、site、surface、machine、schema、故障注入、browser、search。最终浏览器矩阵包括 320／390／768／1440px、深浅主题、键盘、200% 字号、减少动画、无 JavaScript、复制失败与恢复、搜索空结果与失败恢复、音频和下载；不能仅满足已有测试名称就省略新语种风险探针。

每语冻结查询的核心命中必须 12/12，25 条正例至少 23 条进入前三，5 条负例全部满足边界合同。有结果的负例还需审核其边界说明，不能只看数量。独立题集在调检索前冻结，不删除失败题或堆叠无正文依据的关键词。

CI 继续执行 `npm run verify:release` 和 `npm run check:release`，并保留 Python／Node 示例与公开包 recipe jobs。既有严格规则决定 PR 到 main、commit-only 或环境变化时哪些证据可复用：若 identity 或受审字节不符合规则，就重新验收，不手工改报告里的 configuration。最终上传的必须是验收通过的同一份产物。

部署后用独立上传的 manifest 执行：

```sh
npm run check:live -- <verified-dir>/site-manifest.json
```

同时取得工作流 live browser 结果，核对各语言正常 URL、机器文本、未知路径的真实 404、响应 MIME、资源摘要和索引条件。线上通过后推进 verified 发布记录与 P0 历史基线；本地通过、CI 通过、已部署、线上核验分别记录。

## 八 实施切片与退出条件

| 切片 | 具体交付 | 退出条件 |
| --- | --- | --- |
| A 生产基线与测试修复 | 三语生产记录／可信公开面推进；修复两项 fixture 假设；更新 README 的固定语言数叙述 | 下一语以可信三语主线为基线，测试不再占用真实未发布语种 |
| B 巴葡校准 | brief、五类样稿、地区映射／搜索／长文案探针 | 独立双轴样稿问题关闭，明确每语数据包与集成步骤 |
| C 巴葡完整交付 | 34 主题、全部数据、136 条源审核索引、双配置全语言任务证据 | 内容完整、严格检查／CI／线上核验通过，随后推进四语基线 |
| D 日语与德语 | 每语完整数据包；搜索、字体或布局仅按失败证据定向修复 | 各自独立发布；共享修复扩查全部已发布语言 |
| E 法语与俄语 | 每语完整数据包；词形、边界、复数和八语菜单验收 | 八语规模与正常功能符合当前 registry，八语 fixture 通过 |
| F 维护移交 | 最新 verified 记录、基线来源、输入影响规则、恢复说明、观测记录 | 后续版本、共享模板和新主题变更有明确复审范围 |

每语可拆为“完整来源与数据候选”“独立源审校记录”“构建后证据”几个可审查提交；含缺失来源或不匹配 releaseLocales 的中间提交不作为可合并候选。作者不能同时审批自己的译稿；负责全局集成的代理管理共享文件和证据冻结，语言代理只负责分配的内容及数据 patch，独立技术与表达审校可并行。

## 九 失败处理与完成定义

发布前失败只修候选；不要降低搜索、语言评分、资源保真或证据门槛。已上线语言优先前滚修复；严重工程故障可应急恢复上一完整 verified artifact，并记录会暂时失去的新语 URL，随后恢复。不得拼接不同批次的 HTML、Pagefind、llms、Markdown、schema。流量下降本身不触发撤下整个语言。

后续来源或共享代码变更，按 `content-review-inputs.mjs` 实际失效范围复审；新增主题必须补齐全部已发布语言。SEO／GEO 继续沿用[既有测量协议](geo-measurement-protocol.md)，上线后 14／28／56／84 天分别观察抓取、同主题跨语言合计点击与真实问答采样。没有后台数据时记录未取得，不用站内夹具或被告知官方 URL 的代理阅读替代自然发现结果，也不阻止工程发布。

最终完成需满足：八语完整内容及数据；零缺译／错误 fallback；各语链接、锚点、搜索、交互与机器入口一致；当前有效的独立双轴源审核和双配置任务证明；实际部署与 HTTP 核验；新增公开面进入下一次构建使用的可信历史基线。保留原文档的质量合同，本方案替代其中尚未更新的实施基线、待办顺序和数量估算。
