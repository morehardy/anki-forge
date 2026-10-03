# 2026-10-02：读取句柄、canonical checksum 与 PNG 外层压缩

[完整核查报告](report.md)包含瓶颈、全部实验的正反结果、代码变更和后续优先级。**本轮尚未追平 9 月 21 日：20 个 Rust 时间中位数均高于历史记录。** 同场交错确认中，1,000 条图片 / 音频 / 独立混合媒体分别减少约 4.7% / 2.4% / 6.4%；文本基本持平。这些描述性结果不构成稳定收益承诺。

保留现有公开 API 和原有未提交优化，仅新增 worker-local spool 读取句柄、逐字节兼容的 canonical checksum 专用序列化及 PNG 快速外层 zstd。内部 4 MiB 快照驻留预算、完整身份和媒体检查、发布同步均保留。本轮差异见 [round3.patch](round3.patch)，冻结的完整工作树差异见 [source.patch](source.patch)。

正式矩阵时间为 2026-10-02 21:03:34–21:07:34 Asia/Taipei。5 场景 × 4 个规模 × 两个实现，共 840 次导出：400 个计时样本、200 个独立 RSS 样本、240 次预热；40 个选定输出经过 Anki 导入、内容和渲染检查。另有 640 次诊断导出和 90 次 Anki 检查，包括最终两个未插桩二进制的 130 次交错确认。两套结果分开保留，未用确认数据替换正式矩阵。诊断 RSS 来自计时进程，不能与正式独立 RSS 混为一谈。

## 文件索引

- [summary.json](summary.json)、[comparison.csv](comparison.csv)：正式矩阵全部 20 格和新测 genanki；[delta.csv](delta.csv)比较历史 Rust。
- [confirmation-summary.json](confirmation-summary.json)：优化前后同机交错确认；其余 `*-summary.json` 为各诊断组。
- [verification-summary.json](verification-summary.json)、[audit.json](audit.json)：正式矩阵与诊断记录的离线审计结果。
- [plan.json](plan.json)、[run-manifest.json](run-manifest.json)、[source-snapshot.json](source-snapshot.json)：测量配置、工具链、二进制、源码和输入身份；[final-identity.json](final-identity.json)与测量前记录一致。
- [baseline-input-check.json](baseline-input-check.json)、[media-inputs.json](media-inputs.json)：与 9/21 相同的 20 个 JSON 输入与 2,749 个媒体文件身份。
- [host-hardware.json](host-hardware.json)、[pre-matrix-host.json](pre-matrix-host.json)：硬件、电源和后台负载。跨日期桌面负载与缓存未隔离，旧 Deck 和当前 Project 功能范围也不同。
- [oracle-reuse-check.json](oracle-reuse-check.json)、[prepared-builds-final.json](prepared-builds-final.json)：当前环境重新构建的 Anki oracle；源码、上游 revision 和 patch 一致，二进制哈希不同，不能声称复用了旧二进制。
- `measurements-and-validation.tar.gz`：完整正式矩阵的原始记录、640 次诊断原始记录和文本 smoke 记录。包含所有快慢样本，输出 APKG 在成功校验后清理。
- `source-and-inputs.tar.gz`：正式测量的源码、输入和依赖信息；不包含发布二进制。
- `source-and-probes.tar.gz`：各隔离探针源码、本轮修改前源码、脚本及所有准备/验证日志，包括失败记录。
- [archive-manifest.json](archive-manifest.json)、`SHA256SUMS`：逐成员和逐文件 SHA-256。

首次 baseline 准备因为输入路径错误在 exporter 启动前失败，零导出；随后重新命名并完整执行 baseline2。最初 stage 探针的 build JSON 被迭代构建覆盖：[stage-reconstruction.json](stage-reconstruction.json)说明源码重建依据，计划保留原二进制哈希；重建二进制没有当作原测量证据。三个后续控制组和正式矩阵的构建身份完整保留。正式矩阵及 confirmation 计划中的说明文字未列出后来加入的 PNG 策略，实际冻结源码和最终二进制哈希包含三项改动，结果按组合收益解释。

## 离线复核

在仓库根目录运行；这些命令只读取归档，不重新计时，也不重新导入 Anki。需要现有 benchmark Python 环境及仓库 benchmark 工具。

```sh
benchmarks/.venv/bin/python benchmarks/results/20261002-reader-cache-genanki/verify-archives.py
cd benchmarks/results/20261002-reader-cache-genanki
shasum -a 256 -c SHA256SUMS
```

完整离线复算可在仓库根目录运行以下脚本。输出保留到新的 `.work` 目录，原归档不变；它校验成员安全路径与哈希，解包两套原始记录，核查诊断源码，然后重新计算正式矩阵的 JSON / CSV 并逐字节比较。

```sh
benchmarks/.venv/bin/python benchmarks/results/20261002-reader-cache-genanki/replay.py
```

重新进行性能测量需要按冻结源码和依赖重建两个 exporter 与 oracle，并恢复输入；本目录 `run.py` 使用测量时的本机工作路径，直接执行不是可移植重放。旧版本优化前二进制的来源另见已有 `20261002-spool-genanki` 归档。报告中的时间不能由离线复算重新证明，离线复算验证的是记录完整性和统计推导。
