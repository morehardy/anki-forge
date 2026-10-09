# 2026-10-02 性能劣化定位

主因已确认：新 Project 默认构建不再传入 PreparedMedia，进入“owned 快照 → 临时输入文件 → 物理 CAS → 物理 staging → APKG”的路径。每个媒体文件在 CAS 写入和 staging 复制后各进行一次 sync_all；旧 Deck 默认构建通过有界 PreparedMedia 直接编码写包，跳过这两套物理媒体文件。

诊断基于 `4265fc4751daa18164cbf85bda425f2120e0f57d` 的代码；全部探针位于独立源码副本。产品源码未改动，正式 benchmark 二进制和 build provenance 已恢复并核验。原先 840 次完整 benchmark 及其证据保持不变。

## 最小复现与假设

固定 100 条笔记，只改变唯一图片数，三个重复的中位数为：

| 图片数 | 中位数 ms |
| ---: | ---: |
| 0 | 40.893 |
| 1 | 51.047 |
| 10 | 137.988 |
| 100 | 949.377 |

由 0 到 100 张增加约 908 ms，接近每个媒体对象 9 ms；这是媒体数量驱动的稳定复现。所有 12 个最小复现输出均通过完整内容/媒体检查。

测试前的排名：① 中间 CAS/staging 的逐文件同步；② 新路径重复复制/哈希；③ Media::file 快照注册；④ 身份/包检查。计时探针与两个独立同步开关分别验证这些预测。

## 同一可执行文件的同步对照

固定 1,000 条笔记与同一套冻结输入，独立控制 CAS 和 staging 两个中间同步点；最终 APKG 同步、原子发布、身份检查、外部检查均保持启用。每组合 1 个单独预热 + 3 个正式样本，随机顺序；共 80 个导出。全部内容/媒体验证通过，10 个选定实际 APKG 通过 Anki 导入/内容/渲染检查；同场景的逻辑字段摘要在所有模式中一致。

| 场景 | 原路径 ms | 仅绕过 CAS sync ms | 仅绕过 staging sync ms | 两者均绕过 ms | 两者均绕过的耗时减少 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 纯文本 | 95.001 | 96.524 | 101.206 | 88.169 | 7.2% |
| 独立图片 | 10095.609 | 5197.760 | 5747.571 | 1724.150 | 82.9% |
| 独立音频 | 9316.537 | 4800.207 | 5396.803 | 1556.660 | 83.3% |
| 混合独立媒体 | 6619.714 | 3512.682 | 3818.992 | 1252.071 | 81.1% |
| 混合共享媒体 | 488.356 | 378.415 | 356.244 | 179.484 | 63.2% |

两个开关分别使其对应阶段显著下降，双开关对媒体有最大影响；纯文本不触发它们，耗时保持相近。这构成同代码、同输入的因果对照，而非只凭函数名推断。这个消融实验只验证原因，不是生产补丁，也不证明中间持久化语义可无条件移除。

## 排除跨日期环境差异：重建归档源码

从 9 月 21 日 source-and-inputs.tar.gz 提取历史源码，逐文件核对 source-snapshot.json，使用相同 Rust 1.92.0、release、System allocator 构建。两版都加相同形式的低开销聚合阶段计时，交替先后顺序；每版/场景 1 个单独预热 + 3 个正式样本。16 个输出全部通过物理/语义/媒体检查，4 个选定输出通过 Anki 检查，字段摘要一致。

| 场景 | 归档 Deck ms | 当前 Project ms | 当前 / 归档 |
| --- | ---: | ---: | ---: |
| 纯文本 | 60.336 | 88.711 | 1.47× |
| 独立图片 | 237.872 | 10428.149 | 43.84× |

## 墙钟阶段与次要开销

| 1,000 张图片的阶段 | 归档 ms | 当前 ms |
| --- | ---: | ---: |
| 归一化（当前包含物理 CAS） | 57.657 | 4839.303 |
| staging | 5.919 | 4407.134 |
| APKG writer（旧版重用已准备的编码 payload） | 18.451 | 335.124 |
| 包检查 | 27.839 | 57.184 |

Media::file 注册在当前图片样本约 99 ms，资产输入 staging 约 157 ms。它们不是秒级主因。移除两个同步后仍需约 1.7 秒，剩余包括多套文件的写入/复制/清理、额外哈希、APKG 串行再编码和最终复制发布；本次未逐项隔离其净贡献。

纯文本同会话由 60.336 增至 88.711 ms。包检查从 5.442 增至 27.131 ms；进一步的三个阶段样本显示：

| 新阶段 | 调用次数 | 阶段耗时中位数 ms |
| --- | ---: | ---: |
| inspect.native | 1 | 28.441 |
| identity.validate | 1 | 22.421 |
| identity.note_content_read | 1000 | 13.830 |
| identity.config_rows | 1002 | 3.179 |

身份检查逐笔记重新查询 notes、template 配置和 cards，并在外层再查一次 cards ord。相同 Basic 模型模板被反复读取 1,000 次；config_rows 的另外两次是模型 field/template 检查。这是有证据的次要热点，优化应保留完整校验语义。新输出还先保留临时 artifact 再 persist_copy，而旧显式输出直接发布独占 candidate；最终复制与同步也增加了固定及大文件开销。

计时为聚合、包含子调用的墙钟时间。CAS/staging worker 的逐调用总时间会在线程之间重叠，不能累加当作进程总耗时；阶段中位数也不严格可加。最早的新探针进程有额外启动耗时，全部记录保留；正式同步/历史对照明确排除了预声明预热，身份子阶段统计保留全部三次。RSS 未用这些 timing 进程作独立内存得分。

## 代码位置与引入路径

- [build_api/normalize.rs:57](../../../anki_forge/src/build_api/normalize.rs#L57) 给 normalize_with_prepared_media 传入 None。
- [build_api/candidate.rs:92](../../../anki_forge/src/build_api/candidate.rs#L92) 调用 build_with_identity_plan；[writer_core/build.rs:98](../../../anki_forge/src/writer_core/build.rs#L98) 再给 writer 传入 None。
- [authoring_core/media_io.rs:256](../../../anki_forge/src/authoring_core/media_io.rs#L256)：每个 CAS 对象的 sync_all。
- [writer_core/media.rs:218](../../../anki_forge/src/writer_core/media.rs#L218)：每个 staging 复制文件的 sync_all。
- [build_api/identity/content.rs:197](../../../anki_forge/src/build_api/identity/content.rs#L197)：逐笔记重复模板查询；[identity/validation.rs:258](../../../anki_forge/src/build_api/identity/validation.rs#L258) 逐笔记调用。

代码历史表明 `ffa194974b8e1ce19052f4d2bd78a4bcf74df5ec`（2026-09-24，refactor(api): unify ankiforge authoring and native SDKs）新增当前 pipeline/normalize/candidate，并删除旧默认 PreparedMedia 接线路径。其父版本会创建 PreparedMedia，并传递给归一化与 writer；当前构建两端固定 None。已做冻结历史快照与当前版本同环境重测；没有对该大型提交的每个子改动逐一执行完整 Git bisect。

## 修复方向

优先在当前 owned Media 快照上恢复有界 PreparedMedia 构建通路：直接读取已归属的快照，统一计算验证/编码结果，传递给归一化和 writer，保留逻辑 staging，而不为默认私有 build 生成 durable CAS 与 staging 副本。必须保留不可变快照、资源限额、默认检查、真实媒体哈希/引用、失败清理和最终 APKG 的同步/原子发布。共享的显式持久化 CAS/staging 路径仍需要原有完整性及同步行为，不能直接删除其同步。

其次复用模板配置与 SQL prepared statements，或批量读取 notes/cards，再在内存中完成相同身份交叉检查。最后评估独占 candidate 到显式目标的发布路径，避免成功包先保留为临时 artifact 又复制一次，并维持 truthful durability/publication 证据。

验收宜从本次固定媒体数复现和失败/损坏/所有权测试开始，再跑原 20 格完整 benchmark 与 Anki oracle。本轮完成原因定位与诊断副本中的因果验证，没有提交生产修复。

机器数据：[control-summary.json](control-summary.json)、[history-summary.json](history-summary.json)、[identity-results.json](identity-results.json)。原始验证记录与复现脚本见 [README](README.md)。
