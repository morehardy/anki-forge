# Rust 性能优化方案与验收指标

> 历史方案记录；各阶段数字与预算保留其原始语境。最终实现、四档复测及公开证据见[优化验收报告](../../../benchmarks/results/20260907-rust-export-pr/README.md)。未公开的早期实验以下用本地记录路径标明。

日期：2026-09-06。状态：已完成首轮生产优化和正式口径四档复测。本方案优先保持当前默认功能、数据语义和失败行为，另列有明确功能或体积代价的可选取舍。以下诊断数字保留为制定方案时的依据；生产结果见实现与验收报告（本地历史记录 `benchmarks/results/20260906-basic-rust-genanki-optimized-01/implementation.md`）。

本次已实施单事务、SQL复用、实际card范围索引、Project显式ID索引、无baseline直接检查摘要、typed staging复用和两处字段重复工作消除。10K 为604.2325ms / 212.125MiB，达到下文A/B/C的端到端数值预算；同轮genanki为245.507ms。Project显式ID 10K构建为4.718ms，10倍规模增长约10.5–10.6倍。全量Rust质量检查、独立打包消费者、23用户场景、170导出验证和8组Anki验证通过。更广泛的媒体/diff/parser重构、完整分阶段配对增益和allocation目标尚未另行验收，不应把这次合并结果表述为全部候选工作完成。

## 1. 结论与适用范围

最值得优先处理的不是删除公开能力，而是：SQLite 逐行提交、检查器和 Project 的平方级扫描、无消费者的检查指纹/JSON、重复 SQL prepare，以及重复渲染、复制和文件往返。

审查覆盖公开 Deck/Project 到规范化、身份、writer、检查、diff/risk、报告和运行时的主要路径。新增计时覆盖 Basic 200/500/1K/10K、Project 显式/省略 ID 的 add_note 构建，以及独立的分配诊断；省略ID不包含normalize时的身份推导。Cloze、自定义多模板、大媒体、baseline 更新、内部文件输入工具路径的新增性能发现仍是静态证据；必须补场景计时后才能承诺收益。编译速度和 Node/Python binding 开销不属于本轮 runtime 指标。

证据分为三类：**已测因果**（同一二进制单变量对照）、**静态确定**（能指出重复操作或复杂度，收益待测）、**取舍实验**（会改变当前输出大小、报告能力或保障，不能混入默认结果）。

详细数据和探针：本轮审查结果（本地历史记录 `benchmarks/results/20260906-rust-performance-audit/report.md`）；原事务/索引证据：前轮诊断（本地历史记录 `benchmarks/results/20260906-basic-export-diagnosis/report.md`）。所有下文毫秒数仅对应同台 M1 Pro 参考机的该组诊断，不是跨机器性能承诺。

## 2. 实际热点与处理方式

|优先级|路径与发现|证据与收益|保持功能的处理方式|
|---|---|---|---|
|P0|`writer_core/apkg.rs:339,468,494`：notes/cards 逐条自动提交|已测；10K 完整进程 10.34s → 1.29s|一次数据填充事务，提交前释放 statements；保留同步、压缩、检查和失败不发布|
|P0|`writer_core/inspect.rs:544–578`：每 note 扫全部 cards|已测；事务后另一组 1.35s → 0.99s|使用实际 card 映射的 note 范围查询/索引，保留 ord 顺序及实际卡语义|
|P0|`product/project.rs:739–747`：显式 ID 每次扫描之前的 notes|已测；构建 1K 2.84ms → 10K 246.50ms|私有 ID→首次索引表；成功加入后更新，clone/失败后状态与诊断保持|
|P1|同一 SQL INSERT 被不断重新准备|已测；P0 后 959→875ms，填充 179→109ms|在事务作用域准备 notes/cards statements 并复用；元数据语句可保留简单实现|
|P1|`product/comparison.rs:90,113–131`：无 baseline 时完整检查指纹计算后丢弃|已测；P0 后 959→772ms，辅助 RSS454→296MiB|内部摘要路径先只省没有消费者的指纹，继续完整解析、观察与失败判定|
|P2|观察 JSON、fingerprint payload、canonical JSON 多份共存|已测；Rust请求峰值在此由186升到336MiB|摘要路径再省完整 observation JSON；完整 inspect/diff 用借用/流式 canonical 表示保持原指纹|
|P2|`writer_core/staging.rs:177,224` / `:94,104`：clone IR、写出后重读并完整反序列化两次|静态确定，收益待测|保留 staging 文件/ref/fingerprint，writer 直接借用已验证 typed 数据；磁盘输入只 decode 一次|
|P2|typed Project 多轮 rendered_fields/identity/source-map 工作|静态确定，Basic/custom 可多轮渲染同字段|一次 build 内复用 rendered fields/identity；仅需字段名的路径直接读 keys，模型布局每 notetype 计算一次|
|P2|Disabled reconciliation 构造不会返回的逐 note Info diagnostics，并 clone GUID plan|静态确定，未测|不构造无消费者的诊断；保留 GUID 冲突检查/metadata；仅保留一份 assignment，缩短生命周期|
|P2|diff 深复制两侧 observation，再逐项 clone/序列化比较|静态确定，仅 baseline/diff 触发|借用索引与精确递归比较/借用 serializer；只复制最终差异结果，保留顺序及 evidence|
|P2|媒体注册/复核/复制/CAS/staging/压缩多轮 I/O|静态确定，Basic 无媒体未测|合并流式 hash+copy、CAS verify+compress；保留源变化/损坏检测、限额、原子发布|
|P3|每 note 重建字段表、排序和查找模型；CSS/畸形 HTML 极端扫描|静态复杂度确定，幅度待测|每模型预计算索引；HTML/CSS 用线性游标/行号索引，严格保留解析与身份语义|
|P3|collection 压缩前后及最终包 hash 的整块 buffer；冷运行时加载|前者静态确定；后者已有部分缓存|collection流式压缩、完成 ZIP 后流式 hash；仅缓存不可变默认配置，按冷/热场景决定投入|

每项独立测量，不能把不同轮或父子阶段耗时相加。Basic 上约21ms的 Zstd 压缩不应成为首要优化；RSS 峰值也不能全部归因于 SQLite 或某一处 clone。

### 两个需要明确设计的内部接口

**检查摘要与完整报告分开。** 默认无 baseline 的构建只消费 `InspectSummary`，不消费 `InspectReport.artifact_fingerprint`。先增加 writer_core 私有摘要结果，复用同一套 `ApkgReader`、SQL/protobuf、媒体 hash、limits 和观察逻辑，只省 `build_report/fingerprint_report`。下一步才引入共享观察枚举，由摘要收集器累计计数，完整收集器创建 JSON。

独立 inspect 与 comparison 继续使用完整报告。只要请求了 baseline，当前 APKG 仍走完整检查；不可读的 baseline 保持现有 unavailable/错误结果。摘要必须来自实际 APKG，不能以写入前 IR 的预期 counts 替代。共享解析和观察资格判断，避免复制第二套规则。不能用空指纹/假指纹包装成完整 `InspectReport`。

**typed writer 与磁盘入口共用一个写入实现。** build 已持有的 normalized IR、已验证 model IDs 和 GUID plan 通过内部借用传给 writer。保留原 staging materialize 行为、fingerprints 和工具的磁盘输入能力；磁盘入口解码一次后进入同一个 writer。两个入口均保留 GUID/model-ID plan 校验；旧 staging 缺少 model IDs 时仍恢复原 positional IDs，显式空 plan 仍报错。不要创建一个仅供 Basic benchmark 的专用写包器。

## 3. 分阶段目标

所有绝对指标均为 M1 Pro / 32GiB / arm64、release/default features、冻结输入的参考验收预算。耗时预算取10次独立进程的中位数；RSS预算取另行5次进程各自 `ru_maxrss` 的中位数，同时报告范围和最大值。诊断分配指标按同一探针的独立重复中位数比较。跨机器以同机修改前后比值及复杂度指标判断；普通单元 CI 不设紧毫秒门槛。

### 阶段 A：消除确定的复杂度与提交问题

实现单事务、实际 card 索引、Project 显式 ID 索引；分成可独立 review/回滚的小改动。SQL prepared statements 可随后单独进入阶段 B，防止把收益与事务混在一起。

|Basic规模|本轮仅事务+card索引原型|阶段A参考预算|相对原正式基线最低改善目标|
|---|---:|---:|---:|
|200|81ms|≤120ms|耗时下降≥50%|
|500|106ms|≤150ms|耗时下降≥65%|
|1K|146ms|≤200ms|耗时下降≥75%|
|10K|944ms|≤1.2s|耗时下降≥85%|

结构指标：全部 note/card 数据填充处于一个事务；inspector 不再逐 note 全表扫描；Project 10K 显式 ID 的**构建阶段**参考目标≤25ms，10倍输入的构建/观察阶段耗时≤15倍，且重复 ID 错误仍指向首次定义。扩大到100K仅作专用复杂度压力诊断，不混入当前 README 四档。

本阶段不宣称已解决内存峰值。独立 RSS 相比生产基线不得出现无法解释的>5%回退。

### 阶段 B：省去未消费的指纹与重复 SQL prepare

无 baseline 的默认导出采用摘要结果，保留完整读取/检查，只省 fingerprint；复用 prepared statements。两个改动单独验收后组合测量。

- **10K 默认导出≤800ms，独立峰值 RSS≤330MiB**，并相对同机已落地阶段A耗时至少下降15%。原型组合699ms、辅助RSS296MiB提供可行性证据；生产接入和独立RSS仍需正式确认。
- 本轮带计数器 control 的请求峰值336MiB，省指纹后186MiB。实现应以阶段A同一计数方式确认峰值 requested bytes 至少下降35%、累计请求字节至少下降25%；这些分配指标不能代替RSS。
- 普通 Basic 的其余三档无显著回退。默认成功/失败的 BuildReport、limits、risk、policy 与发布行为一致；完整 inspect/diff 的 fingerprint 不变。

`copyless strip` 仅减少约42万次分配及62MiB累计请求，没有降低本轮峰值。将其放在完整报告路径的后续优化中，避免把小改动当作内存问题的主要解法。

### 阶段 C：减少完整数据副本，扩展到其他 workload

按数据证明的占比处理：摘要不构造 observation JSON、typed staging 借用、写入后不再需要的 identity/IR/plan 尽早释放、复用字段渲染和模型索引。释放前先保存返回结果/lockfile仍需的数据，不能依靠猜测直接drop。

- **10K≤650ms：研发目标，尚未证明保留VACUUM的完整方案能稳定达到。** 不计重叠收益；从699ms再减少约50ms才可达标。无VACUUM原型611ms只证明另一种体积取舍下的可行性。
- **RSS≤256MiB：探索目标，不能当已证明的承诺。** observation阶段新增约63MiB存活请求，省去它可能让先前writer峰值成为新上限；分配器回收和native内存使RSS变化并非线性。以实测决定是否继续收紧。
- typed build 不重读自己刚写出的 staging manifest；每 note 输入侧的字段渲染和 resolved identity 计算至多一次/build；固定字段布局只按模型数构建。完整 inspector 仍从实际 APKG 重新计算 revision 内容摘要，不复用输入 hash。跨build必须正确失效，不隐瞒 SOURCE_CHANGED。
- 完整 inspect/fingerprint 路径减少中间副本，要求 canonical bytes/hash完全一致；只为默认路径省指纹的收益不能再次计入这一项。

阶段 C 之后的媒体、diff、模板优化必须先跑下面场景，再承诺对应阶段改善≥25%；若目标阶段占端到端比例太小，降低优先级。

## 4. 可省工作与功能取舍

|候选|本轮收益/上界|代价与建议|
|---|---|---|
|不生成无人使用的检查指纹|P0后约187ms端到端；RSS也明显减少|推荐内部按需计算，保留所有实际检查；不属于取消公开功能|
|默认Disabled路径不生成被丢弃的Info诊断、重复渲染/索引|静态确定未消费，尚未量化|推荐删除内部死工作；不能连同公开diagnostics、GUID校验和metadata一起删|
|取消latest SQLite VACUUM|组合699→611ms；APKG约+7.65%|正向内容/Anki通过，但以约115KB体积换约88ms；单列取舍，默认先保留。可设≤5%体积增长为默认优化预算，本例超出；5%是产品预算，不是兼容规则|
|完全取消导出后检查|P0后约306ms为该阶段上界；省指纹后仍约127ms|ADR0013要求当前检查失败不可发布，直接取消会损失实际输出验证；不建议。阶段B已能无损省掉其中大块成本|
|不保留staging文件|尚未量化；会省写文件及部分编码|可能损害显式artifacts/tooling契约；先消除重读，不直接删staging能力。若要新轻量模式，另定输出/诊断契约并单独benchmark|
|删除身份metadata/revision/更新安全能力|未测，不得承诺大收益|破坏稳定GUID/model ID、重复导入与复习状态保留，默认不建议。当前默认已经不做baseline更新比对|
|关闭媒体校验、资源限额、同步；删除legacy dummy/Zstd|无Basic大收益证据；压缩仅约21ms|涉及数据完整性、失败/持久性或APKG格式能力；不纳入无功能影响方案。临时CAS同步策略只能在明确恢复语义后独立评估|

如果未来添加显式轻量模式，默认模式的能力不能静默减少，结果也不能替代默认流程对 genanki 的比较。当前无需以删除公开能力作为性能优化的前提。

## 5. 验收矩阵与方法

### 正式性能

继续在本仓库 `benchmarks/` 中维护公共输入、runner协议、独立验证和结果schema，Rust/未来Node/Python binding 只增加 adapter。库内部阶段探针与分配诊断另存，不能污染端到端默认 API 指标。

- 四档 Basic 使用冻结fixture、相同字段/卡数/HTML语义、release和锁定依赖，沿用每配置3次warmup、10次独立进程计时，以及另行3次warmup、5次RSS；构建/下载/验证不计时。
- 配置交替、随机化块顺序，记录median/IQR、全部样本、失败率、输出体积、源码/二进制/fixture/硬件身份。发生热状态/负载异常时如实标注，不删除慢样本。
- 先诊断，再冻结实现与配置做确认轮。小档不以10K改善掩盖回退；预先约定5%为调查线。超过调查线不得直接发布“无回退”结论，必须看分布及同机确认结果。
- 统计成功与正确性分别记录，失败输出不能进入成功耗时。公开README只能使用完整功能、通过独立验证的正式数据；本轮4次诊断和辅助RSS不能直接替换。

### 扩展场景与专项指标

|场景|输入轴|关注指标/候选|
|---|---|---|
|typed Project|显式/省略ID；1K/10K，后续100K压力|构建阶段斜率、字段渲染/identity重复次数；不能用删除稳定ID替代修复|
|Cloze、自定义模型与多模板|1/多card；稀疏ord；1/10/100模型；2/20/100字段|实际notes/cards分别计数，模板/字段布局复用；不能把v3持久化requirement误称为每note重解析|
|长字段、HTML/CSS|同notes但字段bytes×10；多URL和畸形HTML|每字节成本、行号正确性、扫描斜率，10倍规模时间≤15倍|
|媒体|单64MiB；1000小文件；已压缩/可压缩；别名共用；冷/热CAS|按唯一bytes和binding数分别计；writer CAS读取3遍→2遍，减少至少一次完整源hash/copy；路径输入流式内存不随单文件线性增长|
|baseline/update-safe|未变化、1%变化、全部变化；APKG+lockfile；同输入重复导入|默认与更新分开计时；diff借用索引、身份历史合并；baseline只捕获一次的现有机制保留|
|full inspect/diff/report JSON|独立full inspect、comparison、显式report_json|完整fingerprint/证据不变；去掉深复制及全String；阶段目标≥25%须由对应baseline证明|
|同进程重复build|冷首次与热后续；更换输入/options/source|默认资源OnceLock已有收益；缓存不得跨build保留旧字段/媒体/diagnostics|

### 功能是硬门槛

1. **产物语义**：raw SQLite行/完整字段、GUID、model IDs、tags、template ord、deck路由、mtime/revision、media bytes/hash；每配置重复确定性；固定上游Anki导入与渲染、更新后复习状态。布局/压缩改变允许字节变化时，必须声明并重算真实fingerprint，不能同时要求旧新字节完全相同。
2. **检查/报告等价**：完整report/fingerprint与固定golden；摘要与旧完整路径差分比较公开结果，排除字段仅限真实不稳定的路径/耗时。覆盖actual card不等于plan、0/多card、前缀ID、缺模板、重复GUID/ord、缺collection/media、损坏ZIP/zstd/SQL/protobuf、限额边界。
3. **失败与发布**：新增事务中途失败注入，验证旧APKG/lockfile不变、候选不发布；媒体注册后变化、CAS损坏、读到一半失败仍保持同错误；保留strict/report-only、限额terminal及atomic替换规则。
4. **缓存和索引**：Project clone后继续add；失败add后再add；同Project跨build修改输入/options/媒体；同错误优先级与首次索引，不用缓存掩盖变化。
5. **运行现有回归**：复用deck/project、Cloze/custom/product-v3、media/CAS、identity/update-safety、inspect/diff、publication/limits、packaged consumer及真实Anki roundtrip测试。具体文件映射见审查附录（本地历史记录 `benchmarks/results/20260906-rust-performance-audit/reviews/authoring-media.md`）。普通CI做语义和结构回归，专用runner做性能门槛。

媒体流融合后，APKG仍以CAS为来源，不能把可修改的staging副本当作源真相；同一读取流结束后必须验证size/BLAKE3/SHA1再完成候选，APKG媒体`uint32`大小边界与现有资源限额保持。

制定方案时的隔离诊断只执行了69个Basic APKG正向验证、6个上游Anki样本与公开report等价对照，支持优化方向与预算。本次生产实现的全量回归、失败路径验证与真实Anki覆盖边界见上方实现与验收报告；不能将Basic性能结果推广到所有workload。

## 6. 建议实施顺序与停止条件

按顺序交付：事务 → card索引 → Project ID索引 → prepared statements → 摘要结果先省指纹 → 正式四档确认 → 摘要收集器/typed数据复用 → 用扩展workload决定媒体/diff/模板投入。每个实现以对应语义测试和单变量测量闭环，避免同时改格式、算法和验证策略后无法归因。

VACUUM作为独立取舍提案，不是阶段B的依赖。runtime默认bundle已经OnceLock缓存；baseline已有一次捕获共享；公开typed Deck/Project默认不走大型authoring/normalized JSONSchema验证，均不应重写为当前不存在的问题。`ensure_success`的大报告clone也只发生在错误路径；不要把小 `InspectSummary` clone当主内存来源。

达到预算且没有剩余显著热点时停止扩展重构。未通过语义或发布硬门槛的改动回退；收益低于噪声、增加明显复杂度的小项不合入。最终是否比genanki快必须重新运行双方完整矩阵，不能由Rust语言或本轮699ms原型预先推断。
