# 日德基础设施独立审计

2026-11-01，审阅者 `/root/ja_de_infra_audit`。基线 `231bcd56b71765f71b6700141e7237d8561f20bc`；Node `v24.19.0`、ICU `78.3`、本机 Chrome `156.0.8078.76`。只写本目录和 `target/remaining-locales/ja-de-infra/`，未构建共享 dist、修改发布集合或提交。未读取集成人新冻结的日德60题；搜索观察使用上一轮已冻结的小样稿诊断与词义边界反例。

**发现两个可复现的日语检索问题，集成人已经作出窄修复，本审阅者独立复核通过。CI 字体包具备日语资源；随后加入的日语 OG 字体配置与 SVG 语言标记也已作源码复核，真实产物另行验收。未发现新的六语路由、复数或索引预算阻断。此报告不等于日德内容或发布验收通过。**

## 已证实的搜索缺陷与修复复核

1. 基线 `website/src/lib/search.mjs:7` 对 NFD 字符串移除全部 `\p{M}`，同时去掉日语浊点、半浊点。用真实样稿 H1 `Rust で Anki カードを生成する`，不提供 Pagefind 或概念，查询 `カート`（cart）仍命中该标题，`matchedTerms=['カート']`；卡片与购物车被视为同词。原样稿中 `バス` 也错误匹配文件 `パス`。见 [red 日志](kana-regression-red.log)、[公开 API 回归](kana-regression.test.mjs) 与 [原样稿观察](search-observations.json)。这不是生产同义词未完成造成的缺口。
2. 基线 `search.mjs:99` 的 aliasPattern 仅保护 Latin 词界。brief 中真实术语 `ノート` 在 `ノートルダム` 中被当作 note 概念，令原样稿多页命中；同一输入纯词法模式无结果。见 [诊断日志](search-audit.log)。这个问题与浊点归一化独立。

集成人修复差分为：仅移除 Latin 基字符后的 marks，并 NFC 重组；Katakana 别名增加仅针对 Katakana 和长音符 `ー` 的起止边界。英文 stemming、Han 双字片段、排序权重、门槛及独立查询未调整。产品回归覆盖半角／NFD 等价写法、卡片与 cart、path 与 bus、Latin 重音、Notre-Dame 反例，见 [产品15项通过日志](search-product-regressions-green.log)。

我独立读取上述差分，未发现新的可行动问题。同一份原样稿回归对当前共享引擎 **3/3通过**，见 [当前引擎日志](kana-regression-current-green.log)；另验证 `ノートのフィールド`、`Ankiノートを更新`、`ノート生成` 保留 note 概念，而 `ノートルダム` 不再触发，见 [独立观察与受检源码摘要](current-search-review.json)。这证明窄修复的边界；不能代替完整语料的真实 Pagefind 与冻结30题验收。

后续词表需显式表达真正需要的完整片假名复合词。例如若作者确实使用某个长片假名词作为 note/card 的同义词，应在来源支持下写完整 alias；不能依赖以前的任意子串命中。汉字／假名互换、纯平假名 `でっきをこうしんしたい` 的语义，以及自然任务停用词仍是生产词表与独立题集要验证的内容。

德语未发现需要换引擎、增加 stemming 或全局折叠 ß 的证据。`Intl.Segmenter` 将 `Stapelaktualisierung` 作为一个词；现有样稿没有该完整词面，单独查询无结果，不能把缺少生产概念/词表认定为底层缺陷。真实 Python API 句子中的 `Große`、`große`、`GROẞE` 均精确命中，`Grosse` 不命中；`ursprünglichen` 与 `ursprunglichen` 可归一匹配。ß→ss、复合词分拆应按完整生产内容和冻结题的实际失败处理，不能把一般词形推断当成发布阻断。

检索证据边界：小样稿的 `pagefindResults` 是显式合成的正文输入，不是真实 Pagefind。沿用历史脚本把原始 Markdown 放入该字段的模式，对含 `<...>` 片段的长正文并不完全保真，故其中 `Große` 整体正文观测不能被解释为 ß 匹配失败；另用真实来源句子作为单一字段隔离验证，结果见 [contract-observations.json](contract-observations.json)。原始观察保留，不把这个夹具限制转嫁为产品缺陷。

## 字体与 OG

- `website/scripts/setup-browser-ci.sh:10` 已安装 `fonts-noto-cjk`。Ubuntu 官方文件清单列出 Noto Sans CJK Regular/Bold TTC，上游说明该 OTC 包含五种 CJK 地域字形；现有依赖集合已含日语，无需为“可能缺字”先增加包。[Ubuntu 包清单](https://packages.ubuntu.com/noble/all/fonts-noto-cjk/filelist)、[Noto 上游格式说明](https://raw.githubusercontent.com/notofonts/noto-cjk/main/Sans/README.md)。本次没有运行 Linux CI，`ubuntu-latest` 的实际 runner 仍以该次构建日志为准。
- 页面采用 `--af-font` 系统字体栈且 HTML 带 `lang=ja`。本机 Chrome 在隔离样稿上实际选择 `Hiragino Kaku Gothic ProN` 与 `.SF NS`，正文及混排示例没有方框缺字。见 [平台字体观察](font-observations.json)、[浏览器截图](browser-ja-fonts.png)。这只证明当前 macOS/Chrome 环境，不能代替 CI 字体或跨平台结论。
- 基线 `website/scripts/social-image.mjs:26–28` 的独立 SVG 无 `lang`/`xml:lang`，统一显式优先 `Noto Sans CJK SC`、`PingFang SC`。因此“安装了 JP”不等于“OG 选择了日文字形”。集成人已收到建议：对 ja 使用 JP 优先 family，并给 SVG 标记 locale；保留其他 locale 的既有字体选择。浏览器系统字体的成功不能证明 Sharp/Pango 的独立 SVG 也正确。
- [当前族栈样本](current-og-family.png) 与 [显式 JP 对照](explicit-jp-control.png) 已直接查看，可读、未见 tofu。它们只是 glyph fixture：为把 fontconfig 缓存写入本审计目录，使用仅列出本机字体目录的隔离配置；没有 Noto CJK SC，不是生产 OG，也不作为 Linux/SC 地域字形证据。

最终日语 OG 必须读取真实 `og-ja.png`，核对浊点、假名、汉字地域字形、两条70px headline 是否侵入 x=764 起的卡片、25px description/footer 是否越界。德语同样核对长文案。若文字过长先调整相应 social 文案；不据字体夹具重做通用版式。

源码复核补记：随后实际差分为 `locale-model.mjs` 的 ja 条目增加 `socialFontFamily: 'Arial, Noto Sans CJK JP, Hiragino Kaku Gothic ProN, sans-serif'`；`social-image.mjs` 按 locale 读取配置、对字体字符串作 XML escape，并给 SVG 设置 `lang` 与 `xml:lang`。其他语言继续使用原有默认字体栈。这个窄修改关闭了本审计提出的字体选择缺口；本审阅者没有把本机 fixture 或集成人的图像观察当作自己完成的最终图像验收。

## 参数、复数、菜单及六语假设

现有相关静态测试 **57/57通过**，覆盖 locale-model、third-locale、content-contract、messages、machine-content、ai-quickstart、search-ranking，见 [日志](static-contract-tests.log)。这次运行发生于共享检索修复前；产品新增两项检索回归另有上节红绿证据。未跑新的全站构建。

[六语隔离探针](contracts-probe.mjs) 另验证 en/zh-CN/es/pt-BR/ja/de 的 root/prefix 路由与六个不同 OG 文件名，日德复数及参数拒绝、llms 24 KiB **UTF-8字节**恰好边界通过、再增加1字节拒绝。全部通过，见 [结果](contract-observations.json)。这是合成合同测试，不能当作尚未集成的真实词典通过。

| 项目 | 结论与集成约束 |
| --- | --- |
| 日语/德语复数 | 日语只需 `other`；德语需 `one`、`other`。0/1/2/21/1000/1.5实际格式正常；缺类别或计数参数会失败。不要照搬英文字符串来占位。 |
| 插值 | `{count}`、`{language}` 等参数名必须与源一致；框架的 `{{title}}`、`[SEARCH_TERM]`、`[COUNT]`、`[DIFFERENT_TERM]` 同样受检查。 |
| AI提示 | `ai-quickstart.mjs:20–21` 用 deepEqual 检查 prompts/requestPlaceholders/promptContracts 的 keys **集合及顺序**等于 releaseLocales。启用第五语时三处只加入第五语；第六语另一次同时更新。ja宜用 maxCharacters，de可用 maxWords，二者不可同时配置。 |
| llms | `machine-content.mjs:184` 与 check-machine-content 都按 UTF-8 bytes 检查24 KiB，日语不会误按字符通过。真实摘要尚未测量；超限精简摘要并复审，不能截断或提高门槛。`Optional`协议标题保持英文。 |
| 六语测试前提 | 旧 P1 问题已修复：测试注入 qaa，且既有测试包括所有8个配置语言启用的合成夹具，不再占用 pt-BR 或要求永久有 planned locale。未发现新的固定4语/6语总数假设。 |
| 菜单 | 当前组件已有 max-width/max-height、内部滚动、短视口定位；真实浏览器回归会注入8个配置语言，在320/390×360、100%/200%、无JS下逐项验证可见焦点。无需重复旧修复；最终仍运行正式浏览器矩阵与日德当前项长文案。 |
| 浏览器查询前提 | `browser.test.mjs:148–149` 要求每语存在一条 core 且 expectedPageIds 含 media 的已冻结题。这是既有测试消费合同，应由集成人核对；本审阅者未打开新冻结题集，也不建议为测试改题。 |

## 第五语、第六语候选的最小操作顺序

复用现有脚本，不重写验收链。每次只启用一个完整语言。第五语为170个正常HTML、155份Markdown、5份llms；第六语为204、186、6；404另外计数。

1. 集成该语完整29篇文档、2篇博客、210/11词典、metadata/导航/brief、三组AI提示、31主题搜索数据。按当前候选语言顺序更新 releaseLocales；冻结新的 root/prefix 构建后报告位置。另一未启用语的共享 JSON 数据不能提前破坏精确集合合同。
2. Node24 下运行 `npm run check:ai`、`npm run check:docs`、`node ../skills/anki-forge/scripts/verify_quickstart.mjs`；用 `content-review-candidates.mjs` 生成当前输入快照。若只做编辑预览，补齐真实 pending 索引后才使用 `CONTENT_REVIEW_MODE=preview`，不可把 preview 结果当发布证明。
3. 当前全部170/204条页面语言索引取得有效的独立技术/表达证据；新增语看全文，旧语看真实菜单、AI依赖及其他变化。用 `record-content-review.mjs` 分轴导入，运行 `check:content`、`npm test`、`check:examples`、`npm run check`；公开包/SDK证据按现有合同执行或明确核验可复用输入。
4. 调用 `trusted-surface.mjs` 选择真实不可变 base，并显式设置它输出的 `PUBLIC_SURFACE_BASELINE`、`MACHINE_SURFACE_BASELINE`。prefix 使用 `https://morehardy.github.io` + `/anki-forge` + 空域名 + preview；root 使用 `https://ankiforge.dev` + `/` + `ankiforge.dev`，deployment 与本次候选身份一致。
5. 每配置执行 `npm run build` → `npm run check:rebuild` → `node scripts/release-evidence.mjs run --engineering-only`。它已有 content/site/surface/machine/schema/故障注入/browser/search八门；无需替换。独立保留各次dist与原始manifest，先prefix再root，避免受审产物被下一次构建覆盖。
6. 每配置对当次全部语言实际完成6使用任务+4机器读取任务。第五语共60使用+40机器任务；第六语共72+48。报告绑定对应inputSha、configuration、releaseLocales与实际读取资源，不能继承四语 passing 报告。阅读真实OG；真实Pagefind须核心12/12、正例至少23/25、负例5/5，并复核有命中负例的能力边界。
7. 报告写到预先冻结路径后，运行 `release-evidence.mjs finish` 与 `npm run check:release`。第二语言集成改变输入后，重新完成其六语候选链，不能把五语receipt作为六语证据。
8. 若授权进入发布，再由CI对实际HEAD执行同一链、上传受审root artifact；独立verified manifest执行 `check:live` 和live browser。上一语已真正上线后再推进下一语 immutable base 的历史公开面。当前四语本地候选尚未被本审计证明已发布，不能在五语记录中把远端三语或本地四语状态混写。

## 后续独立审阅建议

技术审阅优先核对日语、德语中的原始APKG更新基线、note/card/deck、模型/配置ID、所有权/临时文件生命周期、图片遮挡坐标及三个SDK公开边界；表达审阅按brief盲读后对照源文。检索审阅保留冻结题与生产词表的作者分离，用真实命中内容解释每个负例与未命中，尤其区分缺同义词与词义碰撞。实际五语/六语工程构建、源审校、OG最终图、完整浏览器、机器任务、部署与线上结果都仍需后续分别取得；本审计不签发这些结果。
