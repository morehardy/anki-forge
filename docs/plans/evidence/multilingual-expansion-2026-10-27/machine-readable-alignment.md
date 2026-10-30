# 多语言方案与机器读取方案的对齐评估

日期：2026-10-27（America/Los_Angeles）。对齐对象：[机器读取规格](../../../superpowers/specs/2026-10-27-website-machine-readable-spec.md)、[多语言实现方案](../../2026-10-27-multilingual-implementation-spec.md)及当前源码 `2c07690`。主代理与 SEO/GEO、语言质量两个子代理独立阅读后交叉讨论；本次只修改规划文档，没有修改网站代码、运行构建或重新核验生产。

## 结论

总体兼容。机器方案解决静态内容如何被读取、关联和校验，多语言方案扩展同一套内容与接口的语言范围。机器实现已提交到当前 HEAD，可以继续原有验收；多语言工作应继承其实现并泛化，不再新建 Markdown、llms 或 schema 管线。机器规格的“待实现”状态与早期生产观察不是当前上线证据；最近有限生产观察仍只确认 `732a40f` 的中英 HTML。

需要消除的直接流程冲突是“人工读取验收”与用户已确认的独立 AI 审校要求。另有两项会在实施时形成循环的设计需要提前修正：构建后的审核 hash 回写构建输入，以及候选验收后改变语言状态再重建。其余主要是双语范围的实现限制和需要补充的兼容合同，并非当前双语功能已被证明存在故障。

## 对照结果与归属

| 优先级／性质 | 当前事实与依据 | 已吸收到多语言方案的处理 |
| --- | --- | --- |
| P0 集合接口 | [published-pages.mjs](../../../../website/scripts/published-pages.mjs) 第 14–25 行读取全局 localeIds，但仅比较前两语 draft；[生成器](../../../../website/scripts/machine-content.mjs) 第 209–214 行又严格要求实际 HTML 集合一致 | 第四节显式区分 configured、sourceReady、releaseLocales、verified published；冻结 buildLocales／buildPages，所有生成器和检查器使用同一选择，planned 不参与源读取，第三语 draft 不能漏检 |
| P0 发布循环 | [site-manifest.mjs](../../../../website/scripts/site-manifest.mjs) 第 21–26 行把 content-reviews 纳入输入，排除 docs/plans 与 docs/superpowers | 第七节分离构建前源审核与构建后 releaseEvidence；报告绑定最终 manifest，但不回写本次构建输入。最终构建前提交 releaseLocales，验收后发布相同字节，verified 记录事后推进 |
| P0 旧接口保护 | 旧多语言基线仅列 HTML；manifest 资源有 route/hash/bytes，不能独自证明旧接口未被删除 | C1、S0／S1 增加已验证公开的机器 URL、发现关系、实体 ID、UTF-8／MIME／直达响应合同；保留内容更新自由，不冻结旧 hash |
| P1 双语暂限 | [machine-urls.mjs](../../../../website/src/lib/machine-urls.mjs) 第 17–20 行拒绝第三语；[生成器](../../../../website/scripts/machine-content.mjs) 第 173、184、237–239 行存在另一语、双语清理和非英文一律写中文路径 | C11／S2 同批改 URL、otherLocales、输出、所有权清理、实际覆盖扫描与线上检查；不能只增加 localeIds，防止新语覆盖中文 llms 或留下撤回语言资源 |
| P1 元数据集成 | [官网布局](../../../../website/src/layouts/SiteLayout.astro) 第 17–39 行、[文档中间件](../../../../website/src/route-middleware.ts) 第 14–31 行已有 metadata／schema；[structured-data.mjs](../../../../website/src/lib/structured-data.mjs) 第 8 行固定两语 | C3 收敛现有描述器；hreflang／x-default、Markdown alternate、RSS alternate 按语义分别去重；保留稳定全站实体 ID、各语页面 ID 与单一受管理 graph |
| P1 本地化边界 | 机器规格固定 Optional 及 Markdown 头标签；当前它们部分来自普通词典。全部英语禁止规则会误伤，单纯与词典相等也会接受共同误译 | 第五节精确协议白名单与独立固定值断言；自然语言值、摘要、限制、历史说明及按钮仍完整审校 |
| P1 验收要求冲突 | 机器规格 Acceptance／Done 仍写人工读取验收，用户已明确不需要真人复核 | C11 按原四项读取任务改为独立 AI 检查，保存输入、实际 URL、回答、引用、问题及复审；不设置人工 gate，也不把工具读取当自然 GEO 推荐 |
| P1 结构比较边界 | 机器导出要求忠于同语 HTML；多语言允许章节自然拆合及表格表达调整 | C4／C11 区分跨语语义核对与同语导出保真。旧 alias 留在对应 HTML 章节；Markdown fragment 指向该 HTML，无须保留所有 DOM ID；代码中的 marker 文本不得被解析为章节 |
| P1 索引与体积 | [renderLlms](../../../../website/scripts/machine-content.mjs) 第 155–177 行有共同分组、公开版本和 24 KiB 字节门槛；八语会新增其他语言链接 | 保留共同精选顺序、24 KiB 与全部核心事实；复核七个其他语言入口、日语／俄语 UTF-8 字节量和长摘要。超限先压缩说明，必要时显式调整共同可选集合，不自动删项或截断 |
| P1 线上验收范围 | [check-live-site.mjs](../../../../website/scripts/check-live-site.mjs) 第 39、59、82 行仍有两语资源／发现／不存在路径集合 | 随生成器同步泛化实际请求与未知路径测试；机器资源原 MIME、哈希、无 attachment、404 合同持续执行；根路径及项目 base 均验收 |

这里的 P0 表示应在实施新增语言前定清合同，P1 表示首个第三语候选前必须完成的工程／验证工作；不是当前生产事故等级。

## 直接继承，不重复实现

- Markdown 继续从本次真实 HTML 生成。唯一正文容器、动态代码、空白、复杂表格、隐藏 Tabs／details、媒体和实际链接目标的处理均沿用现有转换器与 fixture。
- canonical 与机器 URL 继续从同一页面身份计算；站点、软件、源码 ID 共享，页面与文章 ID 随各语 canonical 区分。软件身份与事实一致，本语描述和帮助页地址可以不同。
- 公开版本、命令、runtime 继续取 `docs/ai-quickstart.json`；历史博客日期来自真实元数据。审校时间不是 dateModified，开发版本不是公开版本，不给所有旧文章添加“适用最新版”。
- 保留 generate:machine → check:machine → check:schema → manifest 的构建顺序及原有 HTML、浏览器、公开 SDK 检查。验证器必须从实际输出、源 fixture、受信基线独立取证。
- sitemap 与 Pagefind 继续索引 HTML。机器文件是同一内容的表示，不增加 HTML 主题，不扩展到首页或示例 Markdown，不增加 llms-full、动态协商、UA 分流或特殊 robots 规则。

## 优化后的先后与发布范围

1. **机器工作继续完成原验收。** 多语言 S0 记录 `2c07690`，区分已提交、本地、CI、生产；若机器接口先验证上线，立即纳入可信历史基线。无需等待它的生产结果才准备术语、查询与译稿。
2. **先完成中英共性泛化。** S2 将新旧机器相关双语分支一并处理，复用既有模块；S3／S4 处理自然表达与审核证据边界；S5 证明完整构建与候选撤回；S6 独立发布中英共性改造。
3. **每语完整发布包含机器表示。** 当前每语交付 32 个正常 HTML、29 份 Markdown、1 个 llms，以及对应 schema、发现关系、搜索和全部 UI。机器正文自动派生，避免重复翻译成本。全八语预计 256 个正常 HTML、232 份 Markdown、8 个 llms；实际 CI 按注册内容计算。
4. **保持既定队列。** es 完整试点 → pt-BR → ja／de → fr／ru；每语独立达标即发布。机器能力并不改变六语流量优先级，也不要求等待前一语 SEO 观测窗口。

候选生成不等于发布。先提交最终 releaseLocales 快照与源审核，使用生产配置构建，运行搜索、任务、机器读取、schema 与浏览器验收；将外部证明绑定这份 artifact，发布后验证再更新已公开事实。以后普通构建不得少于最新 verified 的语言集合，防止候选状态没有同步而静默撤掉新语。

## 验证边界

本次实际完成：读取两份规格、当前提交、相关实现和检查器；用只读脚本盘点 32 主题（27 docs、2 article、3 其他正常页）与每语 204 个业务词典 key；与两个子代理交叉讨论；更新多语言方案和本评估。未运行网站构建、测试、部署或新的生产 HTTP 全站复验，不声称上述新增门槛已经实现或通过。

更新后的文本已由两个子代理再次独立核对，均未发现待解决的实质冲突；主代理检查三个本次修改文档的 20 个本地链接及表格列数，均通过。此结论只针对方案一致性，不替代后续代码与发布验收。

后续验收需特别覆盖：第三语 draft、候选缺源、llms 路径冲突、撤回残留、三种 alternate 共存、固定标签被误译、章节自然改写与锚点保真、UTF-8 24 KiB 边界、构建后证明不改变原 artifact，以及旧机器 URL／MIME／不同版本资源混用。具体夹具和完成条件已合入完整方案第十节。
