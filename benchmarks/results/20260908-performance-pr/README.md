# 导出性能优化的完整证据

[直接阅读实现、结果和局限](../../../docs/superpowers/specs/2026-09-08-export-performance-pr.md)。

四个压缩包逐字节保存原先封存的结果目录和六份原始报告。原始 SHA256SUMS、样本、失败尝试、源码快照、私有探针、依赖锁文件及产物验证记录均保留；归档中的脚本不进入产品或 CI。原报告中“当前工作区”“未提交”等描述对应测量当时的状态。

| 归档 | 已逐字节核对的文件数 | 压缩大小 |
| --- | ---: | ---: |
| [20260908-main-audit.tar.gz](20260908-main-audit.tar.gz) | 221 | 1.52 MiB |
| [20260908-main-optimization.tar.gz](20260908-main-optimization.tar.gz) | 434 | 0.81 MiB |
| [20260908-post-optimization-audit.tar.gz](20260908-post-optimization-audit.tar.gz) | 279 | 0.82 MiB |
| [20260908-round2-optimization.tar.gz](20260908-round2-optimization.tar.gz) | 188 | 5.79 MiB |

[archive-manifest.json](archive-manifest.json) 记录外层压缩包和原始报告的 SHA-256、文件数量，以及最后一次测量的 1852 个源码文件与提交准备状态的核对结果。只有 benchmarks/.gitignore 因归档包装发生变化；其他已测量源码保持一致。新增汇总报告不在旧快照中。

在仓库根目录验证外层文件，并解压到新临时目录：

```sh
(cd benchmarks/results/20260908-performance-pr && shasum -a 256 -c SHA256SUMS)
task_evidence_dir="$(mktemp -d)"
for archive in benchmarks/results/20260908-performance-pr/*.tar.gz; do
  tar -xzf "$archive" -C "$task_evidence_dir"
done
for evidence in "$task_evidence_dir"/benchmarks/results/*; do
  (cd "$evidence" && shasum -a 256 -c SHA256SUMS)
done
```

解压后保留仓库相对目录结构；原始报告在 docs/superpowers/specs 下，结果在 benchmarks/results 下。四组材料一起解压后可按原有相对路径阅读。部分更早的历史报告仍指向仓库已有文档，可在 checkout 中查阅。

复跑前按各归档 README 和 manifest 固定依赖、输入、构建目标及测量条件；脚本里的原工作区绝对路径需要映射。完整媒体、APKG、CAS/staging 和编译缓存均为可重建产物，没有重复打包。最终正式 genanki 矩阵为 5 场景 × 4 档，每格 3 次预热 + 10 次计时、独立 3 次预热 + 5 次 RSS；所有 840 次调用逐次通过内容验证，并覆盖 40 个 Anki 导入/渲染样本。

历史档案中的补丁描述的是当时尚未提交的源码状态；应用补丁前核对基线，不要在已经包含本次实现的 checkout 上重复应用。私有探针和诊断成绩与正式性能结果分开，具体瓶颈及统计局限见汇总报告。
