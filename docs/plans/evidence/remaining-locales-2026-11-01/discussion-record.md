# 剩余目标语言方案的审计与讨论

日期：2026-11-01。对应[实施方案](../../2026-11-01-remaining-locales-implementation-plan.md)。本轮使用 grilling 技能及三个只读子代理核查事实、互相质疑；未改网站实现或执行新语言发布。

## 基线与证据

主工作区为 `1e55ce1`。发现西语集成工作树 `3b2173245f1b87aaf94681af83df29642b0af2c4` 后，全部代理改以该树审计。GitHub 连接器读取 PR 65 得到 `merged=true`、合并提交 `ad98a6f5b9293e7657429e3ed423d11e55259a74`；合并时间为 `2026-11-01T14:11:12Z`。读取远端 main 的 release-locales 得到 en／zh-CN／es，比较 `3b21732..main` 得到 ahead_by=1、files=[]。

来源：[PR 65](https://github.com/morehardy/anki-forge/pull/65)、[固定主线发布集合](https://github.com/morehardy/anki-forge/blob/ad98a6f5b9293e7657429e3ed423d11e55259a74/website/release-locales.json)、[西语候选验收](https://github.com/morehardy/anki-forge/blob/3b2173245f1b87aaf94681af83df29642b0af2c4/docs/plans/evidence/multilingual-es-integration-2026-10-31/acceptance.md)。

本轮浏览工具无法读取生产 manifest／es 首页，本机读取也因 DNS 解析失败未取得结果。因此只确认合并状态，不推断生产是否成功。GitHub combined-status 返回空 statuses，不能据此认定没有 workflow 或全部通过。PR 中的 CI 通过说明属于既有候选记录，不能替代合并后生产证明。

## 决策树

- 语言范围：沿用已经确认的 es、pt-BR、ja、de、fr、ru；es 已合并，剩余五语。
- 发布单位：沿用每语完整后发布；同批语言可以各自完成即发布。
- 审校方式：沿用独立 AI 技术与表达审校、修订和复审；不新增母语人工门槛。
- 内容规模：沿用用户 10/28 确认的 34 主题，按最新实现重新盘点其他文案。
- 工程路径：保留现有通用实现，只补已证实的测试问题及探针发现的语种问题。
- 本轮偏好问题 Q1：巴葡先完整校准、日德先准备术语与样稿，或五语全文同时准备。用户回答“是，巴葡先完成，日德先做样稿（推荐）”，采用前者；逐语发布和独立质量门槛不变。

前三项已有[明确用户记录](../multilingual-expansion-2026-10-27/discussion-record.md)，没有再次要求用户确认。grilling 的事实问题交给代理查证，不把代码可查的内容抛给用户。本次未发现必须阻止编写实施方案的新产品决定。

## 子代理与交叉讨论

| 代理 | 查证范围 | 主要结论 |
| --- | --- | --- |
| locale_readiness | 实际源码、来源映射、词典、路由、搜索与菜单 | 接入已数据化；每语29文档＋2博客＋3公共页；210＋11词典键；preview不支持残缺语言 |
| release_quality | 审核输入、任务证据、CI、历史公开面 | 新语引发全局增量复审；两项测试假设阻塞；发布基线不会自动推进 |
| challenge_scope | 旧用户决定、维护成本、并行边界 | 不重开语言范围和人工审校选择；600场景只是五个语言候选的成本下界 |

三位代理交叉核对了审核失效、全语构建后任务、测试 fixture、菜单和搜索风险。主代理独立读取相关代码，补查远端状态后将“西语收口”修正为“已合并西语的生产状态与历史基线核验”。

终审进一步确认：基线维护采用独立提交时，三语启动与四至八语维护额外需要660个构建后场景，与语言候选600个合计至少1,260个；只改基线不会使正文源审校失效。执行步骤已补上证据目录创建、明确pending索引、源审校之后的严格检查，以及prefix归档后再构建root，避免覆盖受审产物。

| 被质疑的说法 | 核查结果 | 方案处理 |
| --- | --- | --- |
| 当前 cwd 就是最新代码 | 远端已合入西语，cwd仍是共性分支 | 锁定远端 main SHA，并以零文件差异的西语树审计 |
| 后五语只需翻译正文 | 词典、元数据、brief、提示合同、搜索与证据缺一不可 | 给出逐文件数据包 |
| 只审新增34页即可 | 菜单写入全部页模型，整份AI配置进入技术依赖 | 旧语真实增量复审，新语全文审校 |
| 旧三语任务报告可直接复用 | post-build报告绑定全局输入、语言集合、配置和资源 | 当前全部语种六使用＋四机器任务，双配置 |
| 数据化已证明八语终态 | 下一语fixture依赖始终存在未发布真实语种 | 用测试私有合成locale替代 |
| pt-BR随时可以启用 | inactive指纹测试把pt-BR写死 | 巴葡之前修复测试前提 |
| preview能直接显示五篇残缺译稿 | 仅跳过审校，其他完整性检查仍生效 | 样稿先读源／临时fixture，完整后整站预览 |
| 可以先合入五语提示再逐语上线 | prompt三对象keys必须与releaseLocales集合及顺序一致 | 各语完整候选分支原子集成共享数据 |
| 当前历史检查已保护西语 | 基线仍停留早期中英提交，CI从base复制 | 生产核验后推进独立可信基线，保留旧保护 |
| 本地通过意味着CI同字节 | 西语曾被跨worktree node_modules符号链接改变HTML | 每工作树独立普通依赖目录npm ci |
| 搜索成功可以预言SEO增长 | 查询是站内验收种子，没有本站需求数据证明 | 外部观测分开，不重做市场排序 |

## 代码定位与验证边界

以下行号均基于与远端 main 同文件的 `3b21732`：

- `website/scripts/content-contract.test.mjs:234–246`：pt-BR inactive 假设。代理在临时目录将 pt-BR 加入发布集合后只改 label，观察到 sourceHash／targetHash 都变化；仓库未改动。
- `website/scripts/third-locale.test.mjs:11–12`：全部八语激活后 extraLocale 不存在，顶层断言失败；纯集合表达式已复核。
- `website/scripts/content-review-inputs.mjs:107,204–210`：发布语言菜单、源／目标模型及全局技术依赖。
- `website/scripts/post-build-reviews.mjs:94–134`：严格输入、配置、资源、当前全部语言覆盖。
- `website/scripts/ai-quickstart.mjs:21–23`、`check-content.mjs:86–87`：精确语言集合要求。
- `website/src/lib/search.mjs:9–31,100–106`：英语stem、Segmenter及概念别名边界；风险尚需真实目标语言探针。
- `website/src/components/LanguageSwitcher.astro:27`：菜单布局；八语可达性仍是待验收项。
- `website/scripts/public-surface.mjs:52`、`trusted-surface.mjs:35–44`：显式capture和immutable base选择，均不会自动推进已发布语言。
- `website/scripts/site-manifest.mjs:23–35`：基线文件属于构建输入，docs/plans证据被排除；基线维护有额外验收成本。

本轮没有重新运行全套构建／发布验收，也不把西语旧日志当成本次通过结果。交付时核对方案内文件引用、规模算式与Markdown差异；新增语言的工程检查全部是后续实施退出条件。
