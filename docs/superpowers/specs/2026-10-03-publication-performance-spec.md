# 四项性能优化：候选包复用、Node COW、原生 manifest 省略与重复大媒体去重

日期：2026-10-03。范围：将已验证的四个实验方向转为正式实现；本 spec 不表示功能已经合入或发布。

## Problem Statement

AnkiForge 的发布者在查看更新比较结果后调用 build，会再次生成、检查和比较同一份候选 APKG。这个重复流程使万条笔记的本机实验耗时达到约 889 ms，而复用已验证候选后约为 475 ms。发布者需要能够查看完整证据，然后发布刚才审阅的同一个候选。

Node 使用者虽然通过 Promise 执行 build/compare，但提交后台任务之前仍深复制 Project。万条短字段笔记的实验中，返回 Promise 前约阻塞 3.39 ms。项目越大，调用时的同步工作越容易影响应用响应。

原生导出还会生成后续原生管线没有使用的 staging manifest；长字段会放大完整 JSON 序列化、散列和写入成本。重复导入相同的大 Media.bytes 时，现有共享缓存能合并最终存储，但命中发生在冗余临时文件已写完之后。

这些优化必须保留现有产品保证：稳定身份、完整更新证据、有限检查预算、风险策略、调用时快照、添加失败的原子性、临时文件所有权，以及真实的发布和持久化结果。不能通过省略检查或弱化所有权换取速度。

## Solution

按以下顺序交付四项优化，各项单独验证，再验证组合结果：

1. **候选包复用**：提供显式准备接口，生成并完整检查一次 APKG，返回拥有该私有候选的 PreparedPublication。使用者读取报告后，消费该对象发布同一个候选。
2. **Node COW**：后台任务获取不可变共享 Project 快照；后续修改在需要时分离状态，减少返回 Promise 前的字段复制。
3. **原生 manifest 省略**：原生 APKG 路径跳过未被使用的 staging manifest 序列化、散列和文件写入，保留原生输出需要的全部验证。依赖 staging 的 Internal Tools Interface 继续生成完整产物。
4. **重复大媒体提前去重**：每次完成输入预算、MIME 和内容摘要校验后，先寻找仍存活的相同快照，再决定是否写临时内容。

2026-10-03 隔离实验提供以下依据。数字是同一台 macOS arm64 主机的中位数，不是正式版本或跨平台承诺，各方向收益不能相加。

| 方向及测量边界 | 原路径 | 实验候选 | 观察结果 |
| --- | ---: | ---: | --- |
| compare 后 build，对固定 baseline 的无改动更新，10,000 条短字段笔记 | 888.834 ms | 475.067 ms | 总耗时降低 46.6%；四种工作负载降低 36.3%–46.6% |
| Node build 调用至返回 Promise，10,000 条短字段笔记 | 3.391 ms | 0.247 ms | 提交耗时降低约 92.7%，不等于完整导出提速 |
| 原生 build，1,000 条宽字段笔记，总字段约 16 MiB | 369.817 ms | 342.912 ms | 总耗时降低 7.3%；普通文本、图片未获得稳定结论 |
| 同一份 8 MiB 媒体导入 8 次并导出 | 111.350 ms | 96.589 ms | 总耗时降低 13.3%；输入准备阶段降低 27.9% |

Rust 提供统一的准备/发布能力；Node 和 Python 通过其原生适配器暴露同一能力，不各自实现更新策略。COW 优化仅限 Node authoring/task 边界。媒体与 writer 优化在共享 Rust 管线中生效。

## User Stories

1. As a publication author, I want to inspect an update once and publish the reviewed candidate, so that review does not require a second build.
2. As a publication author, I want the prepared report to contain the same comparison evidence as compare, so that I can keep my existing review criteria.
3. As a publication author, I want preparation to return completed findings even when policy blocks publication, so that I can understand the risks before deciding what to change.
4. As a publication author, I want publish to enforce the policy captured during preparation, so that inspecting a candidate cannot bypass update protections.
5. As a publication author, I want corrupt or missing baseline evidence to remain a hard error, so that performance changes do not publish unsafe updates.
6. As a publication author, I want baseline and candidate inspection budgets to apply independently, so that retaining a candidate does not disable resource controls.
7. As a publication author, I want a prepared candidate to remain fixed after I edit the Project, so that publication matches what I reviewed.
8. As a publication author, I want abandoned candidates to release their temporary storage, so that review workflows do not leak files.
9. As a publication author, I want each prepared owner to allow only one publication attempt, so that concurrent callers cannot publish or release it twice.
10. As a publication author, I want the destination and baseline paths to retain their preparation-time meaning across working-directory changes, so that delayed publication goes to the intended location.
11. As a publication author, I want publication to reject baseline aliases even after a symlink changes, so that reviewing an update cannot cause the original distribution to be overwritten.
12. As a publication author, I want existing output to remain intact when publication fails before replacement, so that a failed update preserves the previous deliverable.
13. As a publication author, I want errors after replacement to retain truthful publication and durability facts, so that I can decide whether retrying is appropriate.
14. As an SDK user, I want reports and JSON snapshots to remain observation-only values, so that a saved report cannot masquerade as an artifact owner.
15. As a Rust consumer, I want the new capability to work through the Supported Consumer Interface in a packaged dependency, so that it does not require repository-only tools.
16. As a Node consumer, I want preparation and publication to run on workers and expose deterministic close behavior, so that I can integrate review without synchronous filesystem work on the event loop.
17. As a Python consumer, I want equivalent preparation, reports, publication and cleanup semantics, so that update behavior stays consistent across SDKs.
18. As a Node application developer, I want large build and compare calls to return their Promises with less synchronous copying, so that the application remains responsive while exports run.
19. As a Node application developer, I want an in-flight operation to use the Project state from invocation, so that later edits cannot alter its output.
20. As a Node application developer, I want cloned Projects to remain independently mutable, so that sharing immutable state does not merge authoring branches.
21. As a Node application developer, I want note, asset and default-deck changes to honor the same snapshot boundary, so that isolation applies to the entire Project.
22. As a Node application developer, I want rejected additions to leave the Project unchanged, so that copy-on-write preserves atomic authoring behavior.
23. As a Node application developer, I want worker completion, rejection and teardown to release their snapshot owners, so that optimization does not extend temporary-file lifetimes indefinitely.
24. As a Node application developer, I want the first-edit cost while a snapshot is live to be measured and documented, so that I understand where copying was deferred.
25. As an author of long-field notes, I want native export to avoid unused staging serialization, so that large text content is not processed into an unnecessary intermediate artifact.
26. As an Internal Tools Interface consumer, I want explicit staging operations to retain their manifest, fingerprint and artifact references, so that conformance tools continue to work.
27. As a publication author, I want complete embedded identity evidence and final APKG inspection to remain mandatory, so that omitting a staging manifest does not weaken updates.
28. As a media author, I want repeated imports of identical large bytes to reuse live storage before another temporary write, so that duplicate assets require less preparation work.
29. As a media author, I want every import to enforce its own MIME and byte budget even on a cache hit, so that a prior permissive import cannot bypass my current limits.
30. As a media author, I want export names and MIME metadata to remain independent from shared byte storage, so that renaming one Media value does not change another.
31. As a media author, I want concurrent imports, last-owner cleanup and later reimports to preserve exact bytes, so that cache races cannot corrupt or strand media.
32. As a Python service developer, I want forked processes to preserve the existing media ownership rules, so that inherited cache state cannot deadlock or remove a parent's storage.
33. As an SDK user, I want a mutable input buffer to be snapshotted at invocation as before, so that later buffer edits cannot alter imported media.
34. As a maintainer, I want isolated and combined performance results with negative controls, so that I can attribute improvements without hiding regressions.
35. As a maintainer, I want full timing samples, separate RSS measurements and reproducible evidence, so that reported benefits can be independently checked.
36. As an Anki learner, I want imported fields, identities, cards and media to remain correct after these changes, so that faster generation does not damage my study material.

## Implementation Decisions

### 1. 共同边界与交付顺序

- 以当前已存在的结构优化为前置状态，冻结实施基线的 commit、必要工作区 diff 和构建配置。原实验建立在未提交改动之上，不能只用当时的 HEAD 代表完整基线，也不能把之前的优化再次计入收益。
- 实施顺序固定为：候选包复用及 SDK 适配 → Node COW → 原生 manifest 省略 → 重复大媒体去重 → 组合验证。每项具有独立的实现、验证记录和可撤回边界。
- 共享 Rust authoring、身份计划、writer、inspection、comparison 和 publication 管线。新增能力不得复制第二套身份生成、风险分析或发布逻辑。
- 实验补丁包含未被选中的 ZIP 直写及测量开关；只迁移本 spec 所需设计，不能整份套用。正式行为不依赖实验环境变量，也不增加面向消费者的性能开关。

### 2. PreparedPublication 的公共契约

- Rust 新增 Project::prepare_publication，接收现有 BuildOptions，成功返回 build 模块中的 PreparedPublication，失败返回 BuildError。Node 对应 project.preparePublication，返回 Promise；Python 对应 project.prepare_publication，保持现有同步调用风格。新类型属于 Supported Consumer Interface。
- 复用 BuildOptions 的 create/update、temporary/persistent、inspection limits 和 update policy 语义。允许准备 create，但本轮收益目标是 update 的先比较再发布流程。create 上显式配置 update policy 仍是配置错误。
- 准备完成前必须检查真实 baseline（若有）、生成真实候选、检查真实候选、验证完整身份证据并完成比较。只有硬错误全部排除后才返回 owner；不能用内存中的计划或缓存报告替代磁盘 APKG inspection。
- 完整比较被策略阻止时，prepare 仍成功返回 owner 和报告，语义与 compare 的“完成分析不等于允许发布”一致。publish 必须使用准备时确定的策略结果阻止该候选，不得自动 allow 风险类别。
- PreparedPublication 提供只读 BuildReport；更新的 ComparisonReport 从该报告取得。报告及其 JSON snapshot 均不保有文件，也不表示已经发布。保持已有报告、比较和身份证据的 schema 语义。
- owner 持有已经检查的私有、不可变候选。不给调用者候选路径、可写文件、可克隆的未发布 artifact 或从 JSON/path 重建 owner 的构造器。Debug/错误信息不得意外提供可绕过发布的公共候选接口。
- Project 后续变化不改变 owner 中的候选、完整报告或身份分配。准备完成后只保留发布必需的文件与观察值，释放 authoring 快照、baseline reader 和构建 workspace 等不再需要的资源。
- 输出目的地、baseline、检查预算和策略在 prepare 时绑定；publish 不接受新的 BuildOptions。更换 Project、baseline、目的地、预算或策略需要重新 prepare。使用者若在阅读阻止报告后增加风险 allowance，也必须显式重新准备，不隐式重评或修改已经审阅的 candidate。
- baseline 证据是 prepare 时完成的观察，继续遵守原始分发 APKG 应保持不可变的现有要求。publish 不重新读取一个可能已变化的 baseline 来改写比较结论，也不自动发现更新版本。

### 3. 生命周期、发布与错误

- Rust 的 publish 消费 PreparedPublication；类型不实现 Clone。未使用而 drop 会删除候选。一次 publish 尝试，无论成功、策略拒绝还是 I/O 失败，都消费该 owner；重试通过重新 prepare 完成。
- Node/Python 的 wrapper 持有原生 owner。publish 在调用时原子取走 owner，然后执行任务；同一 wrapper 的第二次 publish 或关闭后的 publish 立即产生 PreparedPublicationStateError，代码 BUILD.PREPARED_UNAVAILABLE，带 closed 或 consumed 原因，且不得再安排工作。Rust 用所有权在编译期排除此误用。
- 两个 SDK 都提供幂等 close；Node 的资源清理遵循现有 worker 机制。GC/析构作为兜底。publish 取得 owner 后的 close 不取消进行中的发布，owner 由任务持有直到完成或失败。报告可以在 close 后继续作为普通观察值使用。
- 准备阶段仅生成私有候选，不替换或创建公开目的 APKG。publish 成功才返回已有 BuildOutput 和真实 ApkgArtifact；temporary 输出随后遵守最后一个 artifact owner 删除文件的规则，persistent 输出保留在用户路径。
- 相对 baseline 和目的地在 prepare 调用时锚定；Node 必须在排队前捕获路径语义。保持操作系统对 symlink、父路径组件、Windows drive-relative 路径的现有解析规则，不以字符串折叠替代文件系统解析。
- 记录 baseline 的锚定路径及本次解析位置；prepare 与 publish 都执行目的地别名排除。发布时覆盖 symlink 指向变化、同一文件的硬链接和目的目录变化，不能覆盖已检查的原始 baseline 或当前 baseline 路径所指对象。无法可靠排除别名时返回原有路径错误。
- publish 使用已有原子替换、同步和跨设备回退实现。保留结构化错误、源错误链、完成的观察结果，以及 not_published/published、temporary、confirmed/unconfirmed 等真实 publication facts。发布前失败保留旧目的文件；发布后失败不能谎报从未写入。
- 准备报告的 duration 表示准备工作的耗时；最终成功或错误报告的 duration 表示准备加发布工作的耗时，不包括使用者阅读报告的等待时间。现有单次 build 的 duration 语义保持不变。
- 现有 build 与 compare 签名、结果和异常规则保持兼容。build 复用同一准备/发布核心；compare 继续只返回报告并释放候选，不暗中给 Project 增加跨调用缓存。

### 4. Node 写时复制

- 在 Node 原生适配器的 Project 存储与后台任务交接处采用 Arc<Project> 一类不可变共享所有权。build、compare 和新增 preparePublication 在调用时取得同一版本的 owner；排队前不再深复制笔记字段。
- 所有修改入口，包括 add、addAsset 和 defaultDeck，统一通过写时复制取得独占可变状态。clone 的分支可以共享不可变数据，但任何一方修改都不能影响另一方或在途任务。
- 不改变共享 Rust Project 的值语义，不引入公开 mutable shared state、读写锁或新的 ProjectBusy 使用限制。Node 的同步 authoring API 继续同步；任务仍使用现有 worker 调度与错误转译。
- 没有其他快照时允许原地修改；旧快照或 clone 存活时的第一次修改可能复制 Project。这个成本是明确取舍，必须报告，不能声称所有 authoring 操作都加速。
- 添加失败后 Project 的可见状态仍原子不变。任务完成、拒绝、worker 销毁和 wrapper 清理都应释放 owner；媒体、模型和 artifact 的生命周期不能因共享出现循环或泄漏。
- 保留字节输入在原生任务排队前取得 owned snapshot 的规则。COW 不能被用作持有可被 JavaScript 继续修改的 Uint8Array 内存的理由。报告序列化不纳入本轮优化。

### 5. 原生 staging manifest 省略

- 在 writer 内部明确区分需要完整 staging 产物的调用与仅需原生 APKG 的调用。原生路径跳过完整 manifest JSON、其 SHA-1 和对应文件写入；不需要按字段大小决定是否启用，宽字段是已测得收益的工作负载。
- 验证与物化分离：保留 IR、模型/笔记 ID、deck registry、media binding/CAS integrity 等现有验证，保留必要目录、媒体准备、诊断与最终 APKG inspection。
- staging manifest 与 APKG 内嵌 identity evidence 是不同产物。完整 identity evidence、media map、collection 和更新比较不能被省略。正式导出的身份和兼容性不改变。
- 依赖 staging 的 Internal Tools Interface 和 contract fixtures 继续得到真实 manifest、原有规范化序列化、指纹及可读取的 artifact references。不存在的 staging 文件不得伪造路径或指纹；内部结果形状应能准确表达“未请求 staging”。
- 显式记录行为变化：省略后不再出现仅由冗余 manifest 创建/序列化/写入造成的失败。其余配置、校验和必要 I/O 失败继续保留类型、源错误链与已完成观察值，不把错误统一降为泛化错误。

### 6. 重复大 Media.bytes 提前去重

- 优化 bytes 的大输入路径，保留既有内存/磁盘阈值和共享内存预算。先完成该次调用的长度限制、MIME 语法与内容匹配规则，再计算完整 BLAKE3 和长度，以现有内容键查询仍存活的快照。
- 命中时在创建或写入冗余临时内容前返回强 owner。新的 Media 仍保留本次 MIME 与命名信息；共享仅针对不可变字节存储，不能直接返回其他调用者的整份 Media 值。
- 未命中沿用现有存储、失败清理和登记流程。登记时再次处理竞争：并发 miss 允许暂时各自创建内容，但最终采用仍存活的共享结果并清理多余临时存储。首版不引入按键等待的 in-flight cache。
- 沿用进程内弱引用缓存、PID/fork 保护和内存预算核算；不增加永久强引用、跨进程共享缓存或全局无界历史。缓存不可用、继承锁不可等待或 Weak 已失效时，正确回退至独立快照创建。
- 缓存锁只保护必要的查找/登记，不为提前查询增加全局锁下的大内容散列或文件写入。强 owner 必须在离开锁前取得，防止查找与最后一个 owner 释放之间的竞态。
- 相同 bytes 即使命中也不能跳过更严格预算、MIME mismatch 或 export-name conflict。源文件删除、跨 Project 使用、最后一个 owner 清理和重新导入维持已有行为。
- 命中路径不再经历冗余临时文件操作独有的失败。不得把理论减少的应用层写入字节数称为实测物理磁盘流量；单次导入不承诺明显加速或降低 RSS。

### 7. 文档与兼容性治理

- 补充一次准备、报告审阅、一次发布、显式清理及失败 publication facts 的公开示例，更新 Rust、Node、Python 的类型声明和能力说明。Packaged Consumer Test 必须能够使用新 Rust API。
- 保留 ADR 0023 的 owned authoring 与完整 APKG 更新证据、ADR 0024 的媒体验证与结构化错误，以及既有 bounded inspection 和 publication 保证。ADR 0022 是历史说明，不用于恢复已经废除的 Deck/Busy API。
- 新增准备/发布能力及 manifest 行为差异应有 ADR/RFC 评审记录；句柄状态错误和原生适配协议按现有治理登记、验证和版本化。不得静默接受不包含新方法的旧 native binary。
- APKG 格式和已有 identity/report/comparison schema 不主动改变。Crate Version 与 Bundle Version 分开评估；只有实际契约资产变化时按政策更新 Bundle Version、资产清单和变更记录，不为内部性能重构无故升级格式。

## Testing Decisions

### 测试边界与先例

主要测试边界采用公开 Project/Media/build/compare/prepare-publication API → 真实 APKG → inspect 和内容断言。测试验证消费者能观察到的行为，不断言 Arc 引用计数、私有辅助函数的调用顺序或某个容器的实现。

沿用已有 Packaged Consumer Test、artifact lifecycle、update policy/evidence、media snapshot lifecycle、Node public API、Python fork ownership，以及实际 Anki round-trip oracle。对无法在公开接口稳定制造的并发 miss、继承锁、发布失败和 writer staging 差异，复用已有最靠外的内部测试入口，避免建立新的全局 mock 系统。仅在证明“未重复生成候选”无法通过现有数据流入口表达时，允许仓库测试专用的生成事件计数；不进入消费者 API。

### 功能验收矩阵

| 验收项 | 必须覆盖的行为 | 主要测试边界与现有先例 |
| --- | --- | --- |
| F1 候选内容与报告 | 同一个 Project、baseline、policy、limits 下，对比旧 compare+build 与 prepare+publish 的完整 comparison JSON、字段、GUID、ID/ordinal、revision、媒体字节和数量；覆盖无改动及真实改动更新 | Public API、更新身份 consumer、APKG raw content 校验 |
| F2 策略与硬错误 | 高风险报告可读取但 publish 拒绝；显式类别 allowance；无匹配 allowance 警告；缺失/损坏证据、namespace 不匹配、candidate 无效、baseline/candidate 独立预算失败 | 现有 update policy/evidence、inspection limits 测试 |
| F3 准备生命周期 | create/update、temporary/persistent；准备后修改或销毁 Project；drop/close 未发布 owner；报告不延长资源生命周期；单次发布与重复/并发 publish、close 与 publish 交错 | Artifact lifecycle、三个 SDK 的公开 owner 行为 |
| F4 发布失败与路径 | 已存在输出、预发布 I/O 失败、发布后持久化确认失败；cwd 改变、相对路径、symlink 与父组件、硬链接、准备后目录 symlink 改变、非 Unicode 路径；不得覆盖 baseline | 现有 artifact/path/publication 测试与定向失败入口 |
| F5 Node 快照隔离 | build、compare、preparePublication 分别在排队后 add/addAsset/defaultDeck；多个并发任务；clone 双向修改；失败 add 原子性；核对实际字段/GUID/媒体，不只核对 note count | Node public API 的快照与 clone 测试扩展 |
| F6 Node 生命周期 | 成功/异常任务结束、worker teardown、原 Project/clone 被回收、媒体最后 owner 清理；不可出现悬挂任务、崩溃或未释放的临时资源 | 现有 worker teardown、artifact/media lifecycle 测试 |
| F7 manifest 双路径 | 原生路径与完整 staging 路径语义等价；低层 manifest 字节/指纹/引用仍真实；所有必要验证保留；manifest 专属失败点按新行为更新 | Writer conformance、canonical serialization、public build 错误测试 |
| F8 大媒体共享 | 2/8 MiB 各重复 8 次并保留 owner；相同内容 file/bytes 共享；不同 MIME/名称；更严格预算、零预算、MIME mismatch；唯一输入和小输入负控制 | Public Media、snapshot lifecycle、原生内容检查 |
| F9 缓存竞争与清理 | 并发 miss、Weak 失效、最后 owner 释放再导入、写入失败、fork/PID、继承锁、预算释放；保留真实字节且不删除其他 owner 的文件 | 已有 media lifecycle、Python fork 测试，必要的定向内部入口 |
| F10 分发与兼容 | Rust packaged consumer、Node installed-package、Python wheel consumer；旧 build/compare 兼容；native 协议不匹配明确拒绝；三个 SDK 的准备与发布结果一致 | 现有公开接口、打包和加载器检查 |
| F11 真实 Anki | Basic、Cloze、Image Occlusion、自定义模型、长文本、图片/音频；首次导入及内容变更更新；检查字段、GUID、卡片集合、媒体和代表性渲染 | 现有实际 Anki oracle；客户端调度/本地编辑继续按既有边界验证 |

生成包之间的真实时间戳、持续时间、临时路径等非语义差异单独解释，不要求独立进程的整份 APKG 永远逐字节相同。也不能仅删除这些字段后比较一个缩减报告，必须保留完整比较证据与内容检查。

### 性能验收与测量口径

复用现有 exporter、原生 collector、Node 计时 runner、fixture generator、内容校验和 Anki oracle。正式测量前完成构建、功能测试和 smoke；测量串行进行，不与编译或测试并发。

- 每个工作负载每种模式至少 2 次预热、10 次 AB/BA 交错计时、3 次独立 RSS 测量；先后顺序各占一半。保留所有有效及失败记录，不选择性删除慢样本或补跑单个成绩。
- 冻结工具链、依赖锁、源码身份、运行配置和 fixture 哈希，记录 OS/CPU、电源与运行环境。每轮使用全新的进程；进程内媒体重复组明确保留 owner，区分内容缓存命中与操作系统页缓存。
- Core 报告 spawn→exit 总耗时，并分别记录输入准备与操作耗时。Node 分别记录调用→Promise 返回、Promise 完成、每次运行的最大定时器延迟以及单独 RSS。首次 COW 修改另行测量，不能藏在吞吐量均值中。
- 报告两种模式的中位数、四分位数、逐对差值及固定 seed 的配对重采样范围。Node 定时器样本不称为总体 p99；少量 RSS 高水位样本不称为稳定堆内存减少。
- 每项先与固定基线单独比较，最终组合再与同一基线比较。组合收益不得用四项百分比相加代替测量。

以下是本 spec 制定的工程验收目标，依据已有实验留出余量；它们不是已有产品 SLO，也不是本次写 spec 已经完成的正式验证。性能验收使用相同环境下两个完整批次，不在普通单元测试或共享 CI runner 上硬编码毫秒阈值。

| 项目 | 主要验收工作负载 | 实施验收目标 | 必须同时报告的控制与代价 |
| --- | --- | --- | --- |
| P1 候选复用 | 固定 baseline、无改动更新；1k 普通文本、1k 独立图片、1k 宽字段、10k 普通文本 | 两个批次的 10k 文本及 1k 宽字段“prepare+publish”总耗时中位数各比“compare+build”降低至少 30%，配对节省中位数为正 | 所有四种工作负载；新增真实改动更新与策略阻止的计时；现有单次 build/compare；准备后保留候选的临时存储占用；SDK 适配后的真实流程 |
| P2 Node COW | 1k 短字段、1k 16 KiB 答案、10k 短字段 | 两个批次的 10k build 调用→Promise 返回中位数降低至少 80%；compare 与 preparePublication 使用相同快照路径并独立给出提交计时 | 完整操作耗时、clone、无快照修改、快照存活时第一次及后续修改、失败 add、定时器全部极值、RSS；不把首次修改提速作为目标 |
| P3 manifest | 1k 笔记，正反面各约 8 KiB 确定性文本 | 两个批次的原生总耗时中位数各降低至少 5%，配对节省中位数为正 | 1k/10k 普通文本、1k 图片、低层完整 staging、RSS；不承诺普通文本稳定加速 |
| P4 大媒体去重 | 同一 8 MiB WAV 导入 8 次，保留 owner，最终一个媒体及 3 条代表笔记 | 两个批次的总耗时中位数各降低至少 10%，配对节省中位数为正 | 2 MiB×8、2/8 MiB 各单次导入、小输入、并发 miss、输入准备与导出分段、RSS；输入准备含读取/复制等成本，不标成纯缓存函数耗时 |

基线和组合结果都运行现有标准性能矩阵。任一负控制/既有场景出现总耗时中位数回退超过 5%，或独立 RSS 中位数增加超过 max(基线的 10%, 8 MiB)，必须执行一次完整确认批次并保留首批结果；若重复出现，不默认启用对应候选，先修正或在后续 spec 修订中明确取舍。这是本 spec 的回归检查门槛，不把所有波动解释为优化失败。显式排除 COW 预期的首次修改成本作为“不得回退”控制，但必须公开其分布及影响。

目标未达到时记录未达标，不用缩小场景、放宽检查预算、减少诊断或跳过最终 inspection 来补足成绩。公开性能结论只引用测得收益且配对结果支持的场景。

### 完成定义

- F1–F11 中适用平台的功能验收通过，四项 P1–P4 达标，组合基准与负控制完成且不存在未处理的确认回归。
- Rust 四个 Tier 1 Platform、Node 已支持 native runtimes 和 Python 现有支持矩阵通过对应功能及打包检查。性能声明限定在实际测量平台，不把单机结果推广至其他 OS。
- 更新契约时通过现有 contract verification、summary、package 和 governance 检查；新增文档示例、类型和错误信息验证通过。
- 原始样本、源码身份、环境、fixture 哈希、构建/校验记录、报告和离线复算入口进入可追溯的 PR 证据。离线统计复算与重新导出/真实 Anki 导入明确区分。
- 删除实验开关及仅为探索增加的生产分支；不修改已有基准记录以“更新”历史成绩。完成不等于批准发布到包仓库，沿用现有 Release Gate 与 Publication Approval。

## Out of Scope

- collection 压缩直接写 ZIP、ZIP64/CRC 回填及更广的并行媒体直写。本轮 ZIP 实验没有稳定速度收益，不随四项候选夹带合入。
- 增量导出、跨 build 的全局候选缓存、跨进程候选恢复、从 JSON/path 重建 PreparedPublication、任意候选路径的直接发布。
- 在已准备候选上更换目的地、策略、预算或 baseline；多次发布同一 prepared owner；发布失败后恢复该 owner；远程多人审批、自动风险豁免及发布取消。
- 改变 Project 身份模型、Anki APKG 格式、identity evidence 格式、更新 revision 规则，或削弱原生检查与媒体预算。
- Python authoring 状态的 COW 改造、JavaScript 输入缓冲区零复制、报告序列化优化、Node worker pool 重新设计。
- 媒体解码、播放质量、新的文件格式支持、全局 MIME 推断变化、持久化媒体缓存或按键 in-flight 去重等待机制。
- 把短字段单次 build、唯一媒体导入、最坏事件循环延迟或总体内存下降作为已证明收益；将应用层 I/O 推算冒充物理磁盘测量。
- 修改学习者的本地调度或删除同步语义；自动发行 Rust、npm 或 Python 包；部署网站。

## Further Notes

实验事实与正式实施验收分开记录。2026-10-03 的隔离实现通过了 433 项 Rust 库测试、576 个正式导出包内容检查、44 次选定包的真实 Anki 检查，以及归档记录的离线复算；这些不替代新接口、SDK 适配和组合实现的验证。

现有收益证据有两个重要代价：2 MiB 重复媒体组的 RSS 中位数约从 18.52 增至 20.62 MiB；Node COW 把部分复制成本转移到快照仍存活时的首次修改。Node 候选的各场景全部样本最大定时器延迟并未改善，宽字段 RSS 也有更高离群样本，因此不设置未经证明的“最坏延迟更低”或“稳定省内存”宣传。

候选复用的已测场景是固定 baseline 的无改动更新。内容变化、阻止策略、延迟发布和 SDK 生命周期已有部分功能依据，但都必须在正式实现中补足端到端验证与对应计时。

证据批次为 `20261003-next-directions`，测量窗口为 2026-10-03 17:25:22–17:31:19 Asia/Taipei。环境为 macOS 27 arm64、Rust 1.92.0、Node 24.19.0、CPython 3.11.0、system allocator。桌面负载、页缓存和温度未完全隔离，配对重采样范围仅用于描述本轮数据。

来源工作区 HEAD 为 `eb4b463418e423020be567c2a5c145b97f0c7cdf`，另含未提交的结构优化；完整源码来源以实验冻结记录为准。证据当前保存在工作区，尚不能当作远端可下载附件；实施 PR 应提交其报告、原始记录及复算入口，或提供可访问的等效证据。

- 实验报告 SHA-256：`f87cc1d65410e6f7aacec68472a75961ea820745c6bfd07d22a4f1811ac43d6c`。
- 含五项探索的实验补丁 SHA-256：`331e86e18d88f959485bc09f7cc7f72b812698b2d6f8874c7e8c4b9371ec17fd`。其中只有本 spec 选定的四项进入实施范围。

仓库术语遵循现有领域说明：Project 是一次 publication 的 authoring 容器；APKG 是分发内容，不等于 Rust Distribution；Supported Consumer Interface 与 Internal Tools Interface 的兼容承诺不同；Crate Version 与 Bundle Version 独立管理。当前语义以 ADR 0023/0024 及 build/update guarantees 为准。
