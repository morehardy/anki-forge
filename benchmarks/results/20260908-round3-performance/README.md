# 第三轮性能优化证据

[阅读实现、完整结果与局限](../../../docs/superpowers/specs/2026-09-08-round3-build-performance.md)。本轮从 2026-09-08 开始，2026-09-09 整理完成，目录沿用开始日期。

基线为已合并的 main `e2d69b8`，其 198 个 crate 文件与第二轮封存源码完全相同。优化版采用相同适配器、Rust 1.92 release、默认 features 和 System allocator。所有端到端调用均使用原生 collector，耗时与 RSS 独立采样。

| 文件 | 内容 |
| --- | --- |
| [summary.json](summary.json) | 38 组完整 A/B 的样本、极值、IQR、中位数、配对差值及媒体复测 |
| [verification-summary.json](verification-summary.json) | 1192 次主矩阵、104 次复测的产物绑定；38 组原始内容、完整检查报告及 Anki 验收 |
| [ci-summary.json](ci-summary.json) | 本机完整 CI 门禁结果与日志位置 |
| [build-identities.json](build-identities.json) | 最终源码、二进制及 collector 身份 |
| [baseline-identity.json](baseline-identity.json) | main/封存源码核对及复用基线二进制的依据 |
| [host-and-metadata.json](host-and-metadata.json) | macOS ARM64、AC Power、编译器及适配器配置；不可用的硬件查询明确记录为不可用 |
| [measurements-and-validation.tar.gz](measurements-and-validation.tar.gz) | 3291 个文件：全部调用记录、日志、计划、候选筛选、失败尝试、验证输出、源码补丁、适配器与新增大输入 |
| [archive-manifest.json](archive-manifest.json) | 上述归档每个成员的 SHA-256 |
| [opportunities-screening.tar.gz](opportunities-screening.tar.gz) | 前一轮优化机会分析的原始证据，保留仓库相对路径 |

压缩包不重复保存编译缓存、Python 环境、媒体源、APKG 或持久化 CAS/staging。它们是可重建产物；每次测量输出均通过哈希绑定到经过验证的样本。`diff-samples.json` 的 198 次比较用于完整 DiffReport 等价检查：这次运行与产物验证并行，其计时字段不属于性能结论。

## 验证归档

在本目录执行：

```sh
shasum -a 256 -c SHA256SUMS
```

再检查主归档成员与 manifest：

```sh
python3 - <<'PY'
import hashlib, json, tarfile
expected = json.load(open('archive-manifest.json'))['main_archive_members']
with tarfile.open('measurements-and-validation.tar.gz') as archive:
    assert set(archive.getnames()) == set(expected)
    for member in archive.getmembers():
        assert hashlib.sha256(archive.extractfile(member).read()).hexdigest() == expected[member.name]
print('all archived members match')
PY
```

## 复跑材料

解压主归档到仓库下新的 `benchmarks/.work/<run-name>` 目录。脚本以该层级定位仓库。`production.patch` 记录相对 `e2d69b8` 的 crate 改动，`source-current/` 保留改动文件和隔离 workspace 配置，`driver/` 保存公开 API 适配器及锁文件，`diff-probe/` 保存原版和结构比较器的对照程序。

1. 在隔离的 `source/` 中从 `e2d69b8` 还原完整 `anki_forge/`，配上 `source-current/Cargo.toml` 与锁文件；用 `driver/Cargo.toml` 构建 release 基线。随后应用 `production.patch` 构建优化版，分别保存为 `binaries/baseline`、`binaries/optimized`。不要在已经包含改动的源码上重复应用补丁。若隔离目录仍位于外层 Git 仓库中，应用补丁时设置正确的 `GIT_CEILING_DIRECTORIES`，避免 Git 按外层仓库前缀跳过文件。
2. 按[基准说明](../../README.md)及[第二轮归档](../20260908-performance-pr/README.md)恢复冻结的基础输入与媒体源。原始 `cases.json` 路径指向第二轮工作目录；新增的 5 万条文本与超长字段输入已保存于 `fixtures/`。在新机器上先映射脚本、case 记录中的工作区绝对路径，再校验输入和媒体哈希。路径映射不应改变实际输入内容。
3. 顺序运行 `run-stress-screen-v2.py`、`run-default-matrix.py`、`run-compare-matrix.py`、`run-persistent-matrix.py`，需要重查媒体波动时再运行两个 `run-*-recheck.py`。脚本要求输出目录尚不存在，以免混入旧样本。源文件、二进制和电源发生变化应停止本次会话。
4. 全部计时结束后，以仓库锁定的 Python 依赖运行 `verify-results.py`，并准备冻结旧 inspector、当前 inspector 和固定 Anki oracle。确认实际 APKG 字节、原始内容、完整报告、导入/渲染及全部样本绑定。编译、测试、独立比较探针和验证不应与正式计时并发。

归档同时保留最初的 SQLite 三方案筛选、容量预估不足造成的 5 万条退步，以及修正后的完整矩阵。首次失败的测试夹具、旧临时文件断言、沙箱监听限制和环境准备日志亦保留；完整验收以 `verify-ci-v2.log` 加 `verify-ci-resume.log` 及附加检查日志为准。未重跑 genanki、其他平台性能或 GUI/音频播放验证。
