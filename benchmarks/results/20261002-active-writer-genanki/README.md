# 2026-10-02 活跃写句柄与同步去重基准

结论及优化取舍见 [report.md](report.md)。公开 API 保持不变，**仍未追平 9 月 21 日基准**。本目录保留全部正式样本，没有剔除较慢结果。

正式采集时间为 **2026-10-02 21:55–21:59，Asia/Taipei**。完整矩阵包含 840 次导出及 40 次 Anki 检查；五组诊断包含 570 次导出及 65 次 Anki 检查。真实 fork 探针另有父、子进程各一次导出，核验 19 个原始媒体 payload；不计入前两组样本数。

## 结果与来源索引

- [summary.json](summary.json)、[comparison.csv](comparison.csv)：正式矩阵的时间、独立 RSS 和包大小；[delta.csv](delta.csv)：与历史记录的比较。
- `import`、`fixed`、`budget`、`hash`、`confirmation` 各自的 `*-config.json`、`*-plan.json`、`*-results.json`、`*-summary.json`：控制条件、预声明顺序、全部诊断样本和汇总。同机确认与正式矩阵分别保留。
- [audit.py](audit.py)、[audit.json](audit.json)：570 次诊断的离线核查；[analyze.py](analyze.py)、[verification-summary.json](verification-summary.json)：840 次正式记录的离线核查。
- [run.py](run.py)、[measure.py](measure.py)：正式采集与诊断采集脚本。
- [round4.patch](round4.patch)：本轮相对上一轮 reader-cache 冻结源码的改动；[source.patch](source.patch)：正式冻结时相对 Git HEAD 的完整修改。
- [source-snapshot.json](source-snapshot.json)、[final-identity.json](final-identity.json)：正式测量源码和最后复核身份；[prepared-builds.json](prepared-builds.json)、[oracle-reuse-check.json](oracle-reuse-check.json)：构建与固定上游 Anki oracle 来源。
- [regression.log](regression.log)、[verify-fast.log](verify-fast.log)：回归测试及 CI 快检。真实 fork 见 [fork-probe-result.json](fork-probe-result.json)、[fork-probe.stdout.log](fork-probe.stdout.log)、[fork-probe.stderr.log](fork-probe.stderr.log) 和 `fork-binding-build*.log`。

## 归档内容

| 文件 | 内容 |
|---|---|
| `measurements-and-validation.tar.gz` | `run/` 下全部正式原始记录；`diagnostic/` 下 570 个诊断 case、各组 JSON 和 5 个 1,000 条输入 JSON。 |
| `source-and-probes.tar.gz` | `diagnostic/{import-probe,sync,digest,budget,hash}-source/` 的完整冻结源码；`source/baseline-source/`、`source/bindings/` 必要源码，以及实验脚本和日志。 |
| `source-and-inputs.tar.gz` | 正式采集冻结的生成器、源码及 20 个场景/规模输入。 |

[archive-manifest.json](archive-manifest.json) 记录每个归档成员的 SHA-256；[verify-archives.py](verify-archives.py) 检查成员集合、路径安全和哈希，只接受安全的普通文件成员。二进制不随档分发。原工作区的离线审计已核验 7 个二进制；解包重放会明确报告这 7 个二进制未随档，不能将记录之间的一致性核查称为重新构建验证。

## 离线重放

在仓库根目录使用现有 benchmark Python 环境：

```sh
benchmarks/.venv/bin/python benchmarks/results/20261002-active-writer-genanki/verify-archives.py
benchmarks/.venv/bin/python benchmarks/results/20261002-active-writer-genanki/replay.py
```

[replay.py](replay.py) 在仓库 `benchmarks/.work` 下建立临时目录，核查并解包归档，离线复算 570 次诊断和 840 次正式记录，并逐字节核对 `summary.json`、`verification-summary.json`、`comparison.csv` 三个输出。该过程不重新执行导出或 Anki 导入；原 APKG 和 Anki 临时集合已在成功校验后删除。也可将完整解包的诊断证据目录传给 `audit.py --work-dir`。

`run.py` 含本机准备路径；归档支持审计与复算，不承诺直接复制脚本即可重新测量。源码冻结时工作树已有未提交修改，本轮没有创建提交。Windows 未执行验证。

真实 fork 探针首次链接失败、准备阶段沿用旧二进制哈希假设导致失败的日志均保留；修正后的构建身份与成功结果单独记录。这些准备失败没有替换或删除任何正式采集样本。
