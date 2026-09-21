"""Seal compact evidence; retain all attempts and checks, never APKG payloads."""
import hashlib
import json
from pathlib import Path
import shutil
import tarfile

WORK = Path(__file__).resolve().parent
REPO = WORK.parents[2]
DEST = REPO / "benchmarks/results/20260909-round3-genanki"
RUN = REPO / "benchmarks/.work/runs/20260909-round3-genanki"
SMOKE = REPO / "benchmarks/.work/runs/20260909-round3-genanki-smoke"
REPORT = WORK / "report.md"
assert json.loads((WORK / "completed.json").read_text())["status"] == "completed"
assert json.loads((WORK / "verification-summary.json").read_text())["status"] == "passed"
DEST.mkdir(exist_ok=False)

def sha(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()

top_files = ["summary.json", "comparison.csv", "verification-summary.json", "comparison.svg", "comparison.png",
             "source-and-inputs.tar.gz", "source-snapshot.json", "source.patch", "media-inputs.json", "plan.json",
             "identity-after.json", "completed.json", "host-hardware.json", "round3-source-match.json",
             "prepared-builds.json", "prepare.py", "run.py", "analyze.py", "plot.py", "render-report.py", "archive.py",
             "prepare.log", "smoke.log", "benchmark-tests.log", "run.log", "analysis.log"]
for name in top_files:
    shutil.copy2(WORK / name, DEST / name)
shutil.copy2(REPORT, DEST / "report.md")
archive_files = {}
archive_path = DEST / "measurements-and-validation.tar.gz"
with tarfile.open(archive_path, "w:gz") as archive:
    for source, prefix in ((RUN, "run"), (SMOKE, "smoke")):
        for path in sorted(source.rglob("*")):
            if path.is_file() and (path.suffix in (".json", ".jsonl", ".log") or path.name == "report.md"):
                name = prefix + "/" + str(path.relative_to(source))
                archive_files[name] = {"sha256": sha(path), "bytes": path.stat().st_size}
                archive.add(path, arcname=name, recursive=False)
manifest = {"schema": "round3-genanki-archive-v1", "archive": archive_path.name,
            "archive_sha256": sha(archive_path), "members": archive_files,
            "report_source": "report.md", "report_sha256": sha(REPORT),
            "apkg_retention": "Removed only after each original artifact passed required raw/semantic/media and selected Anki checks."}
(DEST / "archive-manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
readme = """# 2026-09-09：第三轮优化版与 genanki 的标准矩阵

完整中文结果见 [report.md](report.md)。
20 组 = 5 个资源场景 × 100/200/500/1000 条 Basic notes；本机 M1 Pro、32 GiB、macOS 27.0、AC Power。
这是一轮固定未提交源码快照的描述性比较，基于 e2d69b8025fd6dcf817ccbe56208bd7022c5671c 加第三轮优化。
Rust 使用 System allocator 和默认产品 features；genanki 0.13.1 使用 CPython 3.11.0 ARM64。
本目录不修改此前封存报告，也不把不同时间或电源环境的旧 genanki 数据合入本轮。

- summary.json / comparison.csv：全部 20 组的计时、RSS、包体积；每种统计含 n、median、Q1、Q3、min、max。
- verification-summary.json：840 次导出、40 次 Anki 导入/字段/渲染验证的哈希关联及完整性结果。
- measurements-and-validation.tar.gz：全部实际 attempt、collector/stdout/stderr 日志、原始校验、Anki 结果、电源记录、manifest；包括预热和独立冒烟样本，未筛选最快样本。
- archive-manifest.json：上述归档的逐成员 SHA-256；SHA256SUMS 覆盖本目录除其本身之外的所有文件。
- source-snapshot.json / source.patch / source-and-inputs.tar.gz：测量前的源码哈希、完整 tracked patch、产品/adapter/verifier/锁文件源码及 20 份输入 JSON。输入媒体不重复存储，可由归档中的固定生成器重建后逐一校验 media-inputs.json。
- prepared-builds.json：collector、Rust adapter、inspector、Anki oracle 的构建命令、构建环境白名单和实际二进制哈希。身份在运行前后复核，identity-after.json 保留复核结果。
- plan.json / run.py：取样前固定的完整矩阵、840 次运行、40 次 Anki 检查、逐次电源保护，以及不选择性重试的规则。
- prepare.log / smoke.log / benchmark-tests.log：重新编译、200 条双实现冒烟、43 项 harness 测试的记录。测试日志里的预期故障来自 verifier 的反例测试；最终结果为 43 tests / OK。
- analyze.py / plot.py / render-report.py：离线校验、统计和展示代码。四分位数采用仓库 report.stats 的 Hyndman–Fan type 7；原始 runner summary 的 iqr_ms 算法不同，不用于正文。

历史事实复核无需重新运行 exporter。先执行 `shasum -a 256 -c SHA256SUMS`，再按 archive-manifest.json 核对归档成员；完整原始产物已在逐一通过对应检查后清理，重新生成的 APKG 不能充当这批历史字节的证据。

复现实验应使用新的干净目录，避免覆盖已有 run 名称。检出以上 base commit，应用 source.patch，解包 source-and-inputs.tar.gz 的 source/ 覆盖对应源码；固定 Anki 上游 2d44d4d6bc486803f9236033ad840df203c87036，应用 source-snapshot.json 内保存的 upstream_patch。用同一 Rust 1.92.0 和原生 CPython 3.11.0 按 requirements.lock / Cargo.lock 准备依赖及所有工具，构建步骤见 prepare.log 和 prepared-builds.json；这些步骤必须在计时前完成。

用归档中的 media_workload.py 在 benchmarks/.work/readme-comparison/fixtures 生成 v2 输入，并核对保存的 20 份 JSON、2749 个媒体路径/大小/SHA-256。将本目录的 run.py 与 round3-source-match.json 复制至 benchmarks/.work/round3-genanki-20260909/，用 benchmarks/.venv/bin/python 执行 run.py；它调用仓库原有 media_bench.run，使用 3+10 次计时、3+5 次独立 RSS，要求 Anki 验证和逐次 AC Power 检查。run.py 在新测量前重新生成该次源码和运行身份，不能用它覆盖历史记录。

离线重新生成本轮报告时，将归档的 run/ 放回 benchmarks/.work/runs/20260909-round3-genanki/，把本目录中的脚本、plan、source snapshot、source archive、media inventory、identity-after、completed、host-hardware、round3-source-match 文件复制到 benchmarks/.work/round3-genanki-20260909/，依次运行 analyze.py、plot.py、render-report.py。analyze.py 会拒绝遗漏/重复样本、顺序不平衡、错误退出、失败验证、缺失 Anki 证据和变动的快照文件。

这些复现步骤固定软件、数据及执行协议；其他机器、系统后台负载、电源条件或 Python patch 版本构成新的实验，不能混合本轮样本。此桌面会话的前后系统 load 为 8.72/10.69，温控查询不可用；pmset 报告 AC Power / lowpowermode 0，但电池状态为 discharging、90%→87%。完整原始 host state 随 run manifest 保存，不声称受控空闲或冷缓存。
"""
(DEST / "README.md").write_text(readme)
(DEST / "SHA256SUMS").write_text("".join(f"{sha(path)}  {path.name}\n" for path in sorted(DEST.iterdir()) if path.is_file() and path.name != "SHA256SUMS"))
print(json.dumps({"destination": str(DEST), "archive_members": len(archive_files),
                  "archive_bytes": archive_path.stat().st_size,
                  "total_bytes": sum(path.stat().st_size for path in DEST.iterdir() if path.is_file())}, indent=2))
