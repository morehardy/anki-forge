"""Render the complete Chinese report from the validated statistics."""
import json
from pathlib import Path

WORK = Path(__file__).resolve().parent
REPO = WORK.parents[2]
DEST = REPO / "benchmarks/results/20260909-round3-genanki"
s = json.loads((WORK / "summary.json").read_text())
cells = s["cells"]
labels = list(dict.fromkeys(c["label"] for c in cells))
saved = [c["time_saved_pct"] for c in cells]
lines = [
    "# 当前优化版与 genanki：五种资源场景、四个规模档位", "",
    f"2026-09-09 10:58–11:02（北京时间）实测。当前 Rust 优化版在全部 20 组中耗时更低，中位耗时缩短 {min(saved):.1f}%–{max(saved):.1f}%。"
    f"预先规定的每组至少缩短 30% 目标通过 {s['time_target_passed_cells']}/20；1000 张独立图片的 5 次 RSS 采样最高值为 {s['image_1000_max_rss_mib']:.2f} MiB，低于 64 MiB 目标。", "",
    "本报告是同一台机器、同一轮运行中，不同卡片数量及媒体资源负载的比较。硬件固定为 Apple M1 Pro（10 核）、32 GiB 内存、macOS 27.0（26A5421a）、ARM64、交流电；没有额外限制 CPU 或内存，也没有模拟其他硬件配置。", "",
    f"代码基于 `{s['source_commit']}`，加上已完成的第三轮优化，仍是未提交工作区。198 个产品源码文件逐一匹配上一轮封存哈希，标准 Rust adapter 从这些源码重新编译；本次结果属于固定源码快照的探索性实测，不代表一个已发布版本。", "",
    f"执行口径遵循[现行 benchmark 标准]({REPO}/benchmarks/README.md:128)，五场景输入沿用冻结的 v2 媒体数据和 v1 文本数据。", "",
    "- anki-forge：native Rust 公共 Deck API、Rust 1.92.0、release、产品默认 features、System allocator、逐个注册媒体。Crate 0.1.0 / bundle 0.6.3。",
    "- genanki：0.13.1，CPython 3.11.0 ARM64，依赖按 requirements.lock 的哈希安装，复用内置 Basic model。",
    "- 每个场景/数量/实现：3 次计时预热 + 10 次正式计时，再做 3 次内存预热 + 5 次独立 RSS 采样。共 840 次导出：400 次正式计时、200 次 RSS、240 次预热。",
    "- 计时从原生收集器启动进程到进程退出，包括 Python imports、输入读取/解析、HTML 转义、构造数据、默认校验、写包和退出。编译、外部校验和 Anki 导入不计入。",
    "- 场景顺序打乱，两实现相邻交替；每组正式计时恰好 5 次 Rust 先运行、5 次 genanki 先运行。两个实现都在同一个 Codex exec 沙箱环境下运行，测试期间没有并行启动本任务的重型编译或测试。",
    "- 每一对导出结束后按现行媒体 runner 校验、清理，控制磁盘占用；文件系统缓存经过预热但未控制，不作冷缓存结论。没有剔除离群值或选择性重试。", "",
    "资源构成如下。每条 Basic note 生成一张卡片；混合场景均为 30% 纯文本、40% 图片、30% 音频。独立图片每个 PNG 为 64,152 字节（约 62.65 KiB），独立音频每个 WAV 为 32,044 字节（约 31.29 KiB）。共享场景始终复用 49 个文件。", "",
    "| 场景 | 100 条：文件数 / 媒体 MiB | 200 条 | 500 条 | 1000 条 |",
    "| --- | ---: | ---: | ---: | ---: |",
]
for label in labels:
    group = [c for c in cells if c["label"] == label]
    values = [f"{c['resources']['media_files']} / {c['resources']['media_bytes']/2**20:.2f}" for c in group]
    lines.append(f"| {label} | " + " | ".join(values) + " |")
lines += ["", "完整耗时结果如下，单位为毫秒。方括号是 [Q1, Q3]，不是置信区间；每格 n=10。倍数为 genanki 中位耗时 ÷ Rust 中位耗时，缩短比例为 1 − Rust/genanki。", "",
          "| 场景 | 条数 | Rust 中位数 [Q1, Q3] ms | genanki 中位数 [Q1, Q3] ms | genanki/Rust | 缩短 ms | 缩短比例 |",
          "| --- | ---: | ---: | ---: | ---: | ---: | ---: |"]
for c in cells:
    r, g = c["rust"]["time_ms"], c["genanki"]["time_ms"]
    lines.append(f"| {c['label']} | {c['notes']} | {r['median']:.3f} [{r['q1']:.3f}, {r['q3']:.3f}] | "
                 f"{g['median']:.3f} [{g['q1']:.3f}, {g['q3']:.3f}] | {c['genanki_over_rust']:.2f}× | "
                 f"{c['time_saved_ms']:.3f} | {c['time_saved_pct']:.1f}% |")
lines += ["", f"![五种资源场景下的耗时中位数与四分位区间]({DEST}/comparison.png)", "",
          "内存和包体积如下。RSS 是 5 次独立进程的操作系统峰值，再取中位数，括号内为这些峰值的最小值–最大值；不是平均内存、堆分配量或整个应用进程树的内存。APKG 为 10 次正式计时产物大小的中位数。1 MiB = 1,048,576 字节。", "",
          "| 场景 | 条数 | Rust RSS MiB（范围） | genanki RSS MiB（范围） | Rust APKG MiB | genanki APKG MiB |",
          "| --- | ---: | ---: | ---: | ---: | ---: |"]
for c in cells:
    r, g = c["rust"]["rss_mib"], c["genanki"]["rss_mib"]
    lines.append(f"| {c['label']} | {c['notes']} | {r['median']:.2f}（{r['min']:.2f}–{r['max']:.2f}） | "
                 f"{g['median']:.2f}（{g['min']:.2f}–{g['max']:.2f}） | "
                 f"{c['rust']['apkg_bytes']['median']/2**20:.3f} | {c['genanki']['apkg_bytes']['median']/2**20:.3f} |")
media_1000 = [c for c in cells if c["notes"] == 1000 and c["resources"]["media_files"]]
increases = [-c["rss_mib_saved_pct"] for c in media_1000]
lines += ["", f"本次 1000 条媒体场景中，Rust 的 RSS 中位数比 genanki 高 {min(increases):.1f}%–{max(increases):.1f}%，"
          "因此不能概括为所有规模都更省内存。小档位的相对耗时优势较大，其中包含每次全新 Python 进程的启动和 imports 成本，不能直接解释为纯写入器或长驻进程的吞吐差距。", "",
          "包体积比较保留双方默认输出：Rust 使用现代 APKG v3、zstd 压缩的 collection.anki21b 及兼容占位数据库；genanki 使用 legacy APKG v1、未压缩的 collection.anki2。内容等价，但格式、CSS、身份元数据和压缩策略不同；没有重新压缩任一方来统一包格式。", "",
          "全部 840 个原始产物均通过 SQLite 实际行数、字段/模板、媒体字节/名称/数量/引用验证；每个场景/数量/实现的第一轮正式计时产物被固定版本 Anki 导入并检查字段与代表性渲染，共 40/40 通过。两轮预热也包含在 840 个校验样本内。没有 GUI 或实际音频播放验证。", "",
          "每次导出前后均记录电源，840 条记录的来源均为 AC Power，低功耗模式为 0。系统同时报告电池 discharging、电量从 90% 变为 87%，未据此推断实际功耗。桌面并非受控的空闲专机：前后 1 分钟系统 load 为 8.72 / 10.69，温控查询不可用。两实现采用相邻交替采样来减轻环境漂移影响，结果保留这些环境限制。", "",
          "源码、可执行文件、Python 包、20 份输入 JSON 和 2749 个媒体文件的哈希在前后检查中保持一致。Anki oracle 固定于上游 2d44d4d6bc486803f9236033ad840df203c87036，包含已有的 tokio/io-util 构建补丁；修订号、补丁和 oracle 可执行文件哈希一起封存。运行前冒烟导出 2/2 通过，基准 harness 的 43 项测试通过；核心优化的完整 CI 结果见上一轮封存报告。", "",
          "统计直接来自原始样本，使用仓库 report.stats 的 Hyndman–Fan type 7 线性插值。原生收集器时钟分辨率为 1000 ns。原始 media_bench summary 自带的 iqr_ms 使用另一种四分位算法，只作原始记录封存，本报告统一使用 type 7 的 Q1/Q3。没有跨规模平均分、p95、显著性或置信区间结论；这是一次完整的描述性运行。", "",
          f"完整机器可读数据：[JSON]({DEST}/summary.json)、[CSV]({DEST}/comparison.csv)。"
          f"复现步骤和证据索引：[README]({DEST}/README.md)。"
          f"上一轮优化及回归验证：[优化报告]({REPO}/docs/superpowers/specs/2026-09-08-round3-build-performance.md)。", ""]
(WORK / "report.md").write_text("\n".join(lines))
print(WORK / "report.md")
