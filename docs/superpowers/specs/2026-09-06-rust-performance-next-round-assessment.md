Rust 后续性能优化评估 · 2026-09-06

> 历史方案记录；各阶段数字与预算保留其原始语境。最终实现、四档复测及公开证据见[优化验收报告](../../../benchmarks/results/20260907-rust-export-pr/README.md)。未公开的早期实验以下用本地记录路径标明。

第二轮优化之后仍有改进空间。建议以减少重复工作和存活数据为主线，同时隔离评估编译配置和工具链；有界并行放在数据流收敛之后。本轮仅审查代码、已有实验与官方资料，没有修改生产实现、依赖或工具链，也没有运行新的性能实验。下列预算是实验验收建议，不是已获得的收益。

当前正式结果为 Basic 200 / 500 / 1K / 10K：68.624 / 79.386 / 101.995 / 539.391 ms，10K 峰值 RSS 中位数 219.375 MiB。同轮 genanki 10K 为 261.483 ms。第二轮与冻结上一版交错对照，10K 已下降 18.5%，但内存没有改善。第二轮结果（本地历史记录 `benchmarks/results/20260906-basic-rust-genanki-optimized-02/implementation.md`）

以前的 combined 隔离探针已包含第二轮算法，可帮助选取热点：10K 构建约 44 ms、归一化 50 ms、身份准备 26+11 ms、SQLite 填充 105 ms、压实 35 ms、实际产物摘要检查 71 ms、运行时初始化 31 ms、staging 23 ms、collection zstd 20 ms。**这些不是当前生产版 539 ms 的新分段测量**，阶段也不能与父级耗时重复相加。新实现前应重新采集当前版本的阶段时间和分配存活量。原型分段证据（本地历史记录 `benchmarks/results/20260906-post-optimization-bottlenecks/variants-summary.json`）

| 方向 | 本仓库的具体机会 | 优先级与判断 |
| --- | --- | --- |
| 代码逻辑与数据流 | 摘要读取避免构造完整笔记/身份树；减少 Deck 和字段复制；每模型准备一次写入布局 | 主线，最有希望同时改善时间与 RSS |
| 冷启动 | embedded bundle 首次解包、manifest 校验和默认资源加载 | 对 200 档及新进程调用尤其值得研究 |
| 编译配置 | Thin LTO、codegen-units 单独及组合 A/B | 低实现成本，可先做独立实验 |
| Rust 工具链 | 1.92.0 与固定新稳定版本，同源码同锁比较 | 值得测，不能预设收益 |
| 分配器/热点依赖 | 默认分配器与候选；SQLite 版本；特定压缩后端 | 由分配及原生阶段占比决定 |
| 并行 | 逐笔记纯计算、有界媒体预处理 | 在减少重复工作后尝试 1/2/4 worker |
| 批量任务并发 | 同时导出独立 deck | 单独衡量吞吐、延迟和总 RSS |

当前 Rust 路径同时承担 staging、更新身份元数据、现代格式压缩、压实、写后检查和原子发布。genanki 也有字段/卡片检查和 GUID 计算，而且数据库操作调用的是原生 SQLite。两库默认导出的工作量与输出格式不同；不能把差距归因于 Rust 代码天然慢，也不能期望换编译器自动消除这些工作。

最优先的代码候选如下。

1. **摘要检查按需保留对象。** 默认路径已经不生成完整 observations/fingerprint，但仍经 read_apkg_facts 构造 NormalizedNote、字段、标签、全部身份 JSON 与 card-deck map。可让共用的有界读取/解码过程向摘要或完整报告消费者输出，摘要尽量逐行处理并释放。继续读取实际 APKG，保持限额、退化、错误、模型资格和卡片去重语义；不能用输入 counts 或简单 COUNT(*) 替代。实验先要求摘要阶段耗时下降至少 30%，再判断整体 RSS 和四档端到端收益。[入口](../../../anki_forge/src/writer_core/inspect.rs#L244) · [完整对象构造](../../../anki_forge/src/writer_core/inspect.rs#L1376)

2. **减少中间表示的复制。** write_apkg 先 clone Deck，Project lowering 又 clone Deck，Authoring 到 Normalized 再 clone 字段/标签。先移动已拥有的 Authoring 数据，再研究内部借用 lowering，保持公开 &self API、重复导出和错误返回；不为去掉 clone 强迫调用方放弃 Deck。用实际分配次数、累计字节和峰值存活量证明收益，不能以源码中 clone 数量作为验收。[Deck 导出](../../../anki_forge/src/deck/export.rs#L97) · [Project lowering](../../../anki_forge/src/product/project.rs#L2654) · [规范化](../../../anki_forge/src/authoring_core/normalize.rs#L251)

3. **身份数据用借用视图序列化，减少树和索引副本。** revision 先构造 JSON Value 再 canonical/hash；GUID assignment 查询 map 保存整份克隆；新 notes.data 对恒定空对象仍走通用解析/合并。可保留通用 merge，给新建数据用严格等价的序列化路径。必须证明 canonical bytes、revision hash、metadata、GUID、lockfile 和错误顺序一致，包括 Unicode、转义、标签和 preserve_order 特性统一。这里只优化表达方式，保留实际身份信息。[revision](../../../anki_forge/src/writer_core/note_revision.rs#L27) · [assignment 索引](../../../anki_forge/src/writer_core/apkg.rs#L84) · [notes.data 写入](../../../anki_forge/src/writer_core/apkg.rs#L548)

4. **每模型准备一次字段布局和校验信息。** 当前每条 note 查找模型、重排字段、构建字段集合，并 clone 字段内容后 join。先准备排序、首字段、sort 索引和预期字段集，字段内容借用或写入复用 buffer。Basic 只有一个模型，因此模型查找本身不是主要解释；收益需分别用 Basic、多模型、宽字段和多模板样本确认。[SQL 准备](../../../anki_forge/src/writer_core/apkg.rs#L527) · [字段排序与复制](../../../anki_forge/src/writer_core/apkg.rs#L732)

5. **staging 流式序列化与哈希。** 当前已复用 typed IR，没有原来的整份 staging IR clone，但仍创建完整 canonical Value/String。明确类型的借用序列化视图可输出到 buffered file 并累计 hash，要求 manifest 和 fingerprint 字节一致。直接取消 staging 属于另一项接口/契约决策：当前 build 明确要求 materialize_staging，不能算作已证实无行为影响的修改。[staging 输出](../../../anki_forge/src/writer_core/staging.rs#L252) · [现有要求](../../../anki_forge/src/writer_core/build.rs#L57)

冷启动值得独立安排一个实验。当前 OnceLock 已经缓存 embedded bundle，因此重复增加同类缓存不能消除新进程首次约 31 ms 的历史成本。进一步拆分解压、解包写盘、资源解析、manifest schema 编译与校验，评估在构建/打包期生成与 bundle 绑定的默认数据，以及运行时按需从内存读取。对可修改的 workspace/external bundle 仍执行现有校验；嵌入路径的错误与版本契约也需要明确验收。默认 policy/context 的读取与解析是否还需要缓存，应另测常驻调用，不能和首次成本混算。[embedded](../../../anki_forge/src/runtime/embedded.rs#L17) · [manifest 与默认资源](../../../anki_forge/src/runtime/assets.rs#L36)

编译配置可以先做四组同锁实验：原 release、只开 Thin LTO、只设 codegen-units=1、两项组合。默认 release 已是 opt-level=3；lto=false 仍有 crate 内部的 local Thin LTO，当前欠缺的是跨 crate 优化。组合候选为：

```toml
[profile.release]
lto = "thin"
codegen-units = 1
```

这是一组待测配置，可能增加构建/链接时间，也不保证每种输入更快。Cargo 只接受最终 workspace 根的 profile；基准 adapter 自己声明了 workspace，所以仅修改仓库根不会作用于它，公共库也不能替第三方消费者决定 profile。[adapter 配置](../../../benchmarks/adapters/rust/Cargo.toml#L8) · [Cargo 官方说明](https://doc.rust-lang.org/cargo/reference/profiles.html)

现有固定编译器为 Rust 1.92.0 / LLVM 21.1.3。官方截至本次检查已发布 1.98.1（2026-09-03），适合选作固定升级对照；本机名为 stable 的旧安装实际上是 1.81.0，不能直接当新版本使用。先保持源码、Cargo.lock、C 编译器、profile 和目标 CPU 一致，再比较 rustc 版本。使用新编译器构建不等于必须提高 crate 的最低支持版本，继续保留 1.92 兼容测试。新版本的整数格式化等 API 若要使用则是另一项源码/MSRV 选择，不能计作不改代码自动获得的收益。[Rust 1.98.1 发布](https://blog.rust-lang.org/2026/09/03/Rust-1.98.1/) · [1.98 新 API](https://blog.rust-lang.org/2026/08/20/Rust-1.98.0/)

SQLite 与 zstd 主要代码由 C 编译器构建，普通 Rust 工具链升级或 Rust Thin LTO 不会自动改进它们内部的机器码。target-cpu=native 可作为固定机器专项，但产物针对宿主 CPU；不应把机器专用配置静默用于可分发二进制，且 Rust 选项不会自动等价地设置 CFLAGS。PGO 应放在代码和数据流稳定后，使用代表性训练集和独立验证集，覆盖 Cloze、媒体、更新与失败路径；不能只训练 README Basic 四档再宣称通用改善。[目标 CPU](https://doc.rust-lang.org/rustc/codegen-options/index.html#target-cpu) · [PGO](https://doc.rust-lang.org/rustc/profile-guided-optimization.html)

依赖方面，应区分运行时间、编译时间、二进制体积和原生算法版本：

| 当前依赖/配置 | 可能的实验 | 不应推导的结论 |
| --- | --- | --- |
| rusqlite 0.32.1，bundled SQLite 3.46.0 | 单独升级 SQLite 组合，比较原生阶段并复验 SQL/Anki/指纹 | wrapper 更新必然加速，或 Rust 升级自动更新 SQLite |
| zstd 0.13.3；两个 lock 的 zstd-sys 版本不同，但原生均为 1.5.7 | 热点占比足够时测压缩参数/worker，记录包大小与 RSS | wrapper 版本号不同就意味着压缩算法版本不同 |
| 主 APKG ZIP Stored + 内层 zstd；flate2 用于 embedded gzip 等 | 针对冷启动测 gzip 后端；裁剪未执行特性 | 换 zlib 后端会直接加速主 collection zstd |
| 默认 Rust 分配器，未自定义 global_allocator | 在实验 binary 中 A/B 候选分配器，分别测 wall time/RSS/分配量 | 公共库可以替宿主统一分配器，或 C malloc 自动随之变化 |
| zip deflate features 带入未执行的 zopfli 等 | 核实读取能力后裁剪，测构建/二进制体积 | 移除未执行代码必然显著降低 Basic 导出时间 |

替换分配器不会减少逻辑要求的存活数据，应优先消除冗余副本。global_allocator 影响最终 Rust 程序，由应用/绑定宿主决定；公共库不应强加。不要用 panic=abort、关闭校验、线程安全或同步来混入无功能变化的提速实验。zstdmt feature 本身也不会自动启用 worker，需要显式使用相应编码接口并重新验证输出。[Rust 分配器边界](https://doc.rust-lang.org/std/alloc/index.html)

并行化建议采用“独立预计算、有界缓冲、按原序提交”。逐 note 的字段清洗、revision 与身份 payload 可测试 1/2/4 worker，先按输入 index 收集结果，再保持错误优先级、GUID 去重和 SQL row/card ID 分配顺序。用 256/512/1024 notes 分块作为候选，并同时限制累计字节；不要先复制完整输入和 SQL rows 再开线程。线程阈值应依据总字段字节和实测工作量，不仅依据 note 数量。[身份计算](../../../anki_forge/src/deck/identity.rs#L107) · [SQL 顺序](../../../anki_forge/src/writer_core/apkg.rs#L519)

SQLite 保持单 writer，ZIP 条目及最终发布保持有序。共享连接在 serialized 模式下也会串行处理，对同一连接增加线程不代表并行插入；换成 async 只改善调用方的阻塞管理，不减少 CPU 工作。举例：若只有总耗时 30% 可以并行，4 worker 的理想加速上限是 1/(0.7+0.3/4)≈1.29 倍，还未计入调度和额外内存。该 30% 是解释上限的假设，并非本仓库实测占比。[SQLite 官方线程说明](https://www.sqlite.org/threadsafe.html)

媒体读/hash/预处理可单独做 2/4 并发，再按规范顺序合并去重、限额与诊断，最后写 ZIP。保留现有流式 64 KiB 读取，不改成全文件常驻内存。该实验不改善当前无媒体 Basic 负载。原型约 20 ms 的 collection zstd 也不是半秒延迟的主要来源，先给它增加线程的整体收益有限。[媒体流式处理](../../../anki_forge/src/authoring_core/media_io.rs#L184)

Python binding 和 Node legacy 入口通过 subprocess/spawn 调用 CLI；Node 产品 SDK 已加载进程内原生模块，不能将其归为 CLI 包装。后续应分别以 1/2/4 个同时导出任务衡量 decks/s、每任务 p50/p95 和对应进程范围内的总 RSS，避免外层任务数与内层 worker 数相乘。每任务隔离输出、staging、报告、临时目录和身份锁文件。Node 产品 SDK 的后台任务、线程池复用和取消需要单独设计；Python 若迁移到进程内绑定，再评估 GIL 脱离。fresh-process 与常驻首次/后续导出分别展示。本次 Basic 基准仍不包含任何 binding。[Python 调用](../../../bindings/python/src/anki_forge/runtime.py#L151) · [Node legacy 调用](../../../bindings/node/legacy/src/raw.js#L114) · [Node 原生加载](../../../bindings/node/src/internal/native.ts#L68)

建议执行顺序和验收：

| 顺序 | 工作 | 进入下一步的证据 |
| --- | --- | --- |
| 1 | 对当前第二轮实现重新分段和采集分配；完善构建配置身份记录 | 能区分 Rust CPU、原生库、I/O、峰值存活和退出释放；确认探针对端到端的扰动 |
| 2 | 并行开发隔离的逻辑候选与编译器/profile 候选，测量时串行执行 | 每次控制一个变量，保存独立 target/binary/hash；编译、验证不与测量重叠 |
| 3 | 优先落地摘要精简、所有权转移，再组合其余小候选 | 目标阶段显著下降，功能/错误/发布语义通过；收益不能把各原型比例直接相加 |
| 4 | 内存仍高时评估 allocator；CPU 仍集中于独立 note 工作时测有界并行 | 默认小档和 RSS 无不可接受回退，确定性与错误顺序不变 |
| 5 | 稳定实现重新确认四档和必要扩展负载 | 保存所有样本、失败和未达标项；不以更少功能或热进程替换原默认比较 |

建议把单个候选的采用依据设为可重复的端到端 ≥5% 耗时改善，或 ≥10% 峰值 RSS 改善；低复杂度结构修复可单独说明理由。四档超过 5% 回退作为调查线，不自动认定噪声。对下一组合版本，可把 **10K ≤450 ms、RSS ≤180 MiB** 作为探索预算（相对本轮约需 16.6% / 18.0% 改善），不能承诺达成或追平 genanki。摘要阶段 ≥30%、冷启动阶段 ≥50% 的减少可作为专项实验目标，它们不是整条路径的收益比例。

正式测量沿用每格 3 warmup +10 独立进程计时，另行 3 warmup +5 RSS；旧新交错、冻结四档数据，全部产物验收后再清理。编译器实验额外记录 rustc/LLVM、Cargo、锁文件、所有 Cargo profile 覆盖、RUSTFLAGS、CC/CFLAGS、原生库版本、CPU 特性、allocator 和二进制 hash。当前 manifest 尚未完整记录 CARGO_PROFILE_RELEASE_*、CC/CFLAGS 和 allocator，实验前应补齐。[当前记录位置](../../../benchmarks/bench.py#L150)

代码回归覆盖完整字段/卡片/身份、Cloze/多模板、原始与更新导入、损坏输入、资源限额、媒体源变更、canonical/preserve_order、重复导出与原子发布；新增并发还要验证首个错误和顺序。在没有新的配对实验之前，此评估不作任何新增性能优势声明。
