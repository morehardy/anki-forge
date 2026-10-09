"""Render the audited formal matrix and separate diagnostic findings."""
from pathlib import Path
import csv
import json
import sys
from datetime import datetime
from zoneinfo import ZoneInfo

W = Path(__file__).resolve().parent
R = W.parents[2]
sys.path.insert(0, str(R / "benchmarks"))
import bench

new = json.loads((W / "summary.json").read_text())
old = json.loads((R / "benchmarks/results/20260921-readme-genanki/summary.json").read_text())
prior = json.loads((R / "benchmarks/results/20261002-spool-genanki/summary.json").read_text())
lookup = {(c["profile"], c["notes"]): c for c in old["cells"]}
previous = {(c["profile"], c["notes"]): c for c in prior["cells"]}
confirmation = json.loads((W / "confirmation-summary.json").read_text())
assert json.loads((W / "verification-summary.json").read_text())["status"] == "passed"
assert json.loads((W / "audit.json").read_text())["status"] == "passed"

rows = []
for c in new["cells"]:
    baseline = lookup[c["profile"], c["notes"]]
    preceding = previous[c["profile"], c["notes"]]
    rows.append({
        "profile": c["profile"], "notes": c["notes"],
        "historical_rust_ms": baseline["rust"]["time_ms"]["median"],
        "current_rust_ms": c["rust"]["time_ms"]["median"],
        "historical_delta_pct": 100 * (c["rust"]["time_ms"]["median"] / baseline["rust"]["time_ms"]["median"] - 1),
        "preceding_archived_rust_ms": preceding["rust"]["time_ms"]["median"],
        "preceding_archived_delta_pct": 100 * (c["rust"]["time_ms"]["median"] / preceding["rust"]["time_ms"]["median"] - 1),
        "historical_rss_mib": baseline["rust"]["rss_mib"]["median"],
        "current_rss_mib": c["rust"]["rss_mib"]["median"],
        "historical_genanki_ms": baseline["genanki"]["time_ms"]["median"],
        "current_genanki_ms": c["genanki"]["time_ms"]["median"],
        "historical_rust_genanki_ratio": baseline["rust"]["time_ms"]["median"] / baseline["genanki"]["time_ms"]["median"],
        "current_rust_genanki_ratio": c["rust"]["time_ms"]["median"] / c["genanki"]["time_ms"]["median"],
        "historical_apkg_bytes": baseline["rust"]["apkg_bytes"]["median"],
        "current_apkg_bytes": c["rust"]["apkg_bytes"]["median"],
    })
bench.save(W / "delta.json", {"baselines": [old["run_id"], prior["run_id"]], "cells": rows})
with (W / "delta.csv").open("w", newline="") as stream:
    writer = csv.DictWriter(stream, fieldnames=list(rows[0]), lineterminator="\n")
    writer.writeheader()
    writer.writerows(rows)

def local(value):
    return datetime.fromisoformat(value).astimezone(ZoneInfo("Asia/Taipei")).strftime("%Y-%m-%d %H:%M:%S")

labels = {c["profile"]: c["label"] for c in new["cells"]}
lines = [
    "# 当前代码耗时核查与优化：保持 Project API",
    "",
    "**本轮尚未达到 9 月 21 日的全部 benchmark 基准。** 保留现有公开 API、完整身份证据、媒体校验和发布同步后，完整矩阵的 20 个 Rust 时间中位数仍高于历史 Deck 版本。同机交错复测支持独立媒体的小幅整体改善；文本没有稳定的端到端收益。",
    "",
    "在用户原有共享 spool 工作树上新增三项内部优化：每个媒体 worker 复用一个块读取句柄、直接按 canonical 顺序序列化 identity 校验码、仅对 PNG 使用快速外层 zstd 帧。原有未提交改动全部保留。公开函数、类型、选项与 JSON 字段不变；PNG 编码字节允许变化，原始媒体字节和 identity checksum 规则保持一致。",
    "",
    f"正式测量：{local(new['started_utc'])}–{local(new['completed_utc'])} Asia/Taipei。HEAD `{new['source_commit']}`，包含未提交修改；默认产品 features、System allocator、Rust 1.92.0、CPython 3.11.0 / genanki 0.13.1。",
    "",
    "## 完整 benchmark：与 9 月 21 日比较",
    "",
    "5 个场景 × 100 / 200 / 500 / 1,000 条；每个实现/格 10 次计时、5 次独立 RSS 测量，各阶段前 3 次预热。全部 840 次导出及原始 SQLite/字段/媒体/语义校验通过，40 个实际输出通过固定上游 Anki 导入、内容和渲染检查。测量前后代码、工具、依赖、输入与 2,749 个媒体文件哈希一致，每次导出前后均为 AC Power。",
    "",
    "下表只使用完整矩阵中的 1,000 条单元格。全部 20 格、分位数、极值、包大小和新测 genanki 见 [summary.json](summary.json)、[comparison.csv](comparison.csv)、[delta.csv](delta.csv)。",
    "",
    "| 场景 | 9/21 Rust ms | 本轮 Rust ms | 耗时变化 | 本轮 RSS MiB | 本轮 genanki ms |",
    "|---|---:|---:|---:|---:|---:|",
]
for c in new["cells"]:
    if c["notes"] != 1000:
        continue
    b = lookup[c["profile"], 1000]
    delta = 100 * (c["rust"]["time_ms"]["median"] / b["rust"]["time_ms"]["median"] - 1)
    lines.append(f"| {c['label']} | {b['rust']['time_ms']['median']:.3f} | {c['rust']['time_ms']['median']:.3f} | {delta:+.1f}% | {c['rust']['rss_mib']['median']:.2f} | {c['genanki']['time_ms']['median']:.3f} |")
lines += [
    "",
    "![完整矩阵：1,000 条时间与 RSS](comparison.svg)",
    "",
    "本轮 20 格均慢于历史 Rust 基准，幅度约 +12.3% 至 +56.7%。1,000 张图片 RSS 40.58 MiB，满足既有 64 MiB 验收目标；耗时 336.360 ms，距离历史 238.638 ms 仍差 97.723 ms（+40.9%）。本轮 Rust 在全部 20 格快于重新测量的 genanki，但这不意味着追平历史 Rust。",
    "",
    "不同会话的桌面后台负载与缓存未隔离。正式矩阵开始前 load 为 27.51 / 20.64 / 17.17；同机确认后约为 13.79 / 17.80 / 17.08。新测 genanki 在所有格均快于其 9/21 记录，不能简单将 Rust 的差距归结为当天主机变慢；也不能通过按 genanki 比例缩放获得所谓校正成绩。旧 Deck 与当前 Project 的功能范围不同，历史对比是描述性结果。",
    "",
    "## 同机交错复测：本轮改动的实际收益",
    "",
    "比较两个未插桩 Project 二进制：进入本轮时的共享 spool 版本与最终三项优化版本。5 个 1,000 条场景，每版本 3 次预热 + 10 次计时；每场景 5 对优化前先跑、5 对优化后先跑。130 个输出及 10 个选定 Anki 检查通过。此确认是完整矩阵之后的独立样本，不替换完整矩阵的任何格；此处 RSS 是同一计时进程的诊断高水位。",
    "",
    "| 场景 | 优化前 ms | 优化后 ms | 耗时变化 |",
    "|---|---:|---:|---:|",
]
for c in confirmation:
    before, after = c["before"]["median_ms"], c["after"]["median_ms"]
    lines.append(f"| {labels[c['profile']]} | {before:.3f} | {after:.3f} | {100*(after/before-1):+.1f}% |")
lines += [
    "",
    "图片 −4.7%、音频 −2.4%、独立混合媒体 −6.4%；共享媒体 −1.4%；文本 −0.08%，应视为基本持平。只有 10 个描述性样本，没有统计显著性或稳定收益承诺。探针阶段降低的累计时间不能直接当作整次进程收益。见 [confirmation-summary.json](confirmation-summary.json)。",
    "",
    "## 耗时瓶颈：当前入口到包发布",
    "",
    "先用历史 Deck 探针与本轮进入时的未插桩 Project 交错复现差距（每版 3 次预热 + 6 次计时）。1,000 条文本 51.748 → 64.471 ms，图片 239.874 → 352.407 ms；5 场景共 90 个输出均校验成功。历史探针来源沿用前轮已核验的 9/21 源码，含少量粗粒度计时，不能据此认定某个新特性的精确净成本。",
    "",
    "下表为本轮进入时共享 spool 源码的阶段探针，5 次正式样本的中位数；不是最终代码的完整分解。",
    "",
    "| 阶段 / ms | 文本 | 独立图片 | 独立音频 | 独立混合 |",
    "|---|---:|---:|---:|---:|",
]
stage = {c["profile"]: c["default"]["stages"] for c in json.loads((W / "stage-summary.json").read_text())}
profiles = ["basic-mixed-text-v1", "basic-image-unique-v2", "basic-audio-unique-v2", "basic-mixed-unique-v2"]
for label, key in [("媒体导入", "adapter.assets"), ("其中共享块写入", "spool.store"),
                   ("媒体准备 wall time", "media.prepare_all"), ("完整 native inspect", "inspect.native"),
                   ("其中身份验证", "identity.validate"), ("其中媒体历史检查", "identity.media_history"),
                   ("初次候选同步", "artifact.retain_sync"), ("最终文件同步", "artifact.final_file_sync"),
                   ("最终目录同步", "artifact.final_directory_sync")]:
    lines.append("| " + label + " | " + " | ".join(f"{stage[p].get(key, {'ms':0})['ms']:.3f}" for p in profiles) + " |")
lines += [
    "",
    "阶段是 inclusive wall time；导入包含共享块写入，完整检查包含身份验证，worker 累计 reader/prepare_one 时间存在并行重叠。不要相加还原端到端时间。媒体准备包含压缩、读取、哈希和有序写入，不应全算作 codec。",
    "",
    "1. **独立媒体的首要损耗仍在导入。** `Snapshot::file` 同步读取原文件、构造不可变快照；4 MiB 驻留预算外的小对象追加到共享块。1,000 张图片中 935 个溢出对象的写入约 54 ms，整个导入约 133 ms。当前 `Pool::store` 仍每次打开、定位、写入并关闭活跃块；本轮只减少构建读取时的打开次数，没有消除导入端的写入成本。",
    "2. **媒体准备是第二个大头。** worker 重复打开共享块，以及对已压缩 PNG 进行外层 zstd 搜索，都有可测开销。本轮复用一个块句柄，区段 `Take` 仍限制到自身长度；PNG 单独选快模式，每个下一个 job 显式恢复默认级别。完整 SHA-1/BLAKE3 校验、CRC、编码缓冲上限和包检查保留。",
    "3. **文本/共享媒体主要是固定成本。** native inspect 约 12–13 ms，其中 identity 验证约 7–8 ms；还有几毫秒身份绑定、约 2.6 ms 初次载入内置默认值，以及文件/目录同步。新 API 要提供完整可验证更新证据，不能用旧版较浅检查替代它。",
    "4. **媒体名称已不是优先瓶颈。** 现有 ASCII 路径下，1,000 个媒体历史名称检查约 1 ms；逐 note/card 检查仅几毫秒。把主要精力放到 NFC/JSON 键排序或取消线程，不符合这轮量级。",
    "",
    "## 单变量对照与实际保留的实现",
    "",
    "各控制组固定 1,000 条输入，每配置/场景 1 次预热 + 5 次计时，顺序预先随机化。每个预热和正式输出都经过原始媒体、SQLite 与语义检查；每配置/场景选一个实际输出做 Anki 检查。完整原始样本及失败准备日志保留。",
    "",
    "- **保留：构建内读取句柄缓存。** 图片共享块打开次数约 935 → 60，音频约 870 → 28，混合约 618 → 64；累计 reader 打开/定位范围约 23–25 → 2–3.4 ms，但跨 worker 累计值不等于 wall time。图片 prepare_all 79.24 → 71.82 ms，音频 29.23 → 21.35 ms；混合反向 26.87 → 32.18 ms 也保留。最多四个 worker 各持有一个 FD，构建结束关闭，缓存只持 Weak 身份。",
    "- **保留：canonical checksum 专用序列化。** 两次计算合计，文本约 2.29 → 0.69 ms、图片 3.09 → 0.92 ms。普通 sidecar 序列化不变；专用 wrapper 消除完整 Value 树、键排序和 String 的中间层，仍使用相同 serde 字符串/整数规则及 BTreeMap 键顺序。兼容测试比较逐字节 canonical JSON，覆盖 Unicode、转义、退休状态、掩码、None、极值整数。整体文本收益未在最终交错确认中稳定体现。",
    "- **保留：仅 PNG 的快速 zstd 外层。** 单变量全媒体 fast(-1) 探针中，图片 prepare_all 58.33 → 49.47 ms，输出包中位数均约 64,874,060 字节；音频会增加约 0.5% 包大小，所以最终仅调整 PNG，其余媒体恢复原来 level 3。媒体原字节一致，但不承诺任意 PNG 的压缩字节/包大小完全相同。",
    "- **不采用：按偏移读写。** 图片 324.67 → 336.44 ms、音频 238.20 → 258.95 ms，没有稳定整体收益。",
    "- **不采用：预载所有卡片后批量匹配。** 校验通常只减少约 0.5–1 ms，增加整批卡片持有，端到端结果有正有负；不是主瓶颈。",
    "- **不采用：扩大到 16 MiB 快照预算。** 图片 324.67 → 298.03 ms，但诊断 RSS 43.30 → 56.00 MiB；音频、混合也改善。它是明确的速度/内存取舍，不能称为零成本优化。本轮保持既有 4 MiB 预算。",
    "- **不采用：只取消初次候选 sync。** 有些场景整体改善，但同步时间转移到最终文件/目录；前轮有反向样本。本轮该控制保留最终同步，却仍未覆盖临时成功、跨设备、失败分类、目录持久化等完整 API 语义，所以没有直接合入。",
    "",
    "对照中的文本负控制存在数毫秒波动，例如 positional 控制在无媒体文本上也变快；canonical 控制的图片整次收益远大于其 checksum 的约 2 ms 减少。不能把这些端到端差值全部归功于单个开关。实际效果以最终两个未插桩版本的交错确认表为准。",
    "",
    "## 继续追平基准的优先顺序",
    "",
    "1. 先进一步测导入路径：给活跃 spool 块复用一个写句柄，并在最后一个活跃块所有者释放、块旋转、错误和 fork 时明确关闭；单独计时原文件的 metadata/open/fcntl/read 与 spool append。上一轮短测未证明活跃 writer 的稳定收益，不能未经新对照就把约 54 ms 全计为可节省时间。需保留低 FD、部分写失败、独立快照和最后所有者清理语义。",
    "2. 如接受改变内部资源策略，可把 16 MiB（或进一步测 20/24 MiB）驻留作为独立候选。16 MiB 对照支持导入收益且当次 RSS 约 56 MiB，但没有证明最终组合、所有 tier 或更大工作负载的 64 MiB 目标；20/24 MiB 尚未测量。本轮没有改变该预算，也不能宣称它足以追平历史。",
    "3. 重构私有候选生命周期，让临时输出与持久输出各在实际交付边界只进行一次必要文件 sync；仍保留目标目录同步和全部发布失败事实。必须用临时成功、比较/策略阻断、EXDEV、原有目标保护和 sync 失败测试证明语义，再跑完整矩阵；不能简单注释 sync_all。",
    "4. 最后再考虑共享只读 collection/model 事实、流式卡片游标与启动期内置默认值，减少重复 SQL/解码/分配。量级是数毫秒，适合缩短文本差距，无法独自解决近 100 ms 图片差距。",
    "",
    "上述是按实测量级排序的下一步实验，不是未经验证的收益承诺。若保持全部新 API 行为，追平旧 Deck 成绩需要继续减少真实的 I/O 与固定成本；本轮结果不足以宣布达标。",
    "",
    "## 代码与验证",
    "",
    "代码位置：`anki_forge/src/media/snapshot/spool.rs:147` 的 worker reader cache；`media/snapshot.rs` 的借用入口；`prepared_media.rs:156/182` 的 worker-local 生命周期和 `:312` 的 PNG 编码策略；`build_api/identity/canonical.rs` / `identity.rs:385` 的 checksum 字节构造。公开 API 未增加方法或参数。见本轮相对已有 spool 工作树的 [round3.patch](round3.patch)。",
    "",
    "新增/扩展回归覆盖：区段边界、不同 worker 文件位置、弱身份轮换与最后所有者清理；低 64 FD 下导入并导出 74 个快照块/300 个媒体；canonical JSON 逐字节兼容；PNG 后的非 PNG 必须与独立默认编码器字节一致。最终 149 个 core 测试、10 个媒体生命周期测试、23 个公开表面消费者测试通过；仓库 verify-fast 和 46 个 benchmark 行为测试通过，独立 200-note 文本 smoke 通过。",
    "",
    "共 640 次完成的诊断导出 / 90 个 Anki 检查和正式矩阵 840 次 / 40 个 Anki 检查。离线 audit 核验全部原始计时记录、样本计数、学习内容 digest、AC 电源、trace 与 summary 一致性，及三个控制探针的完整源码哈希。首次 baseline 准备因错误输入路径在 exporter 启动前失败，零导出；完整新命名 baseline2 保留该日志并重新执行整组，没有替换实际慢样本。最初阶段探针的 build JSON 被迭代构建覆盖；保留初始 setup/patch、计划中的二进制哈希和重建源码说明，不把后来重建的二进制作为原始测量证据。正式矩阵和三个控制组均保留独立构建身份。",
    "",
    "此次 oracle 按当前环境 locked/offline 重建，二进制哈希与前轮不同；实际 checker 源码、上游 revision 和 patch 与冻结参考一致，新的构建来源单独记录，未声称复用了相同历史二进制。所有 debug 控制都在 `.work` 私有源码，生产源码没有诊断开关。旧结果目录未覆盖。",
    "",
    "证据及重放方法见 [README](README.md)。",
]
(W / "report.md").write_text("\n".join(lines) + "\n")

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np
cells = [c for c in new["cells"] if c["notes"] == 1000]
x = np.arange(len(cells))
fig, axes = plt.subplots(2, 1, figsize=(10.5, 7), sharex=True)
for offset, dataset, label, color in [(-.18, lookup, "Sep 21 archived Deck", "#929ba6"),
                                     (.18, None, "Oct 02 final Project", "#247b77")]:
    for axis, metric, ylabel in [(axes[0], "time_ms", "Process elapsed time (ms)"),
                                (axes[1], "rss_mib", "Independent peak RSS (MiB)")]:
        stats = [c["rust"][metric] if dataset is None else dataset[c["profile"], 1000]["rust"][metric] for c in cells]
        med = np.array([v["median"] for v in stats])
        errors = np.array([[v["median"] - v["q1"] for v in stats],
                           [v["q3"] - v["median"] for v in stats]])
        axis.bar(x + offset, med, .32, yerr=errors, capsize=3, label=label, color=color)
        axis.set_ylabel(ylabel)
        axis.spines[["top", "right"]].set_visible(False)
        axis.grid(axis="y", alpha=.18)
        axis.set_axisbelow(True)
axes[0].legend(frameon=False)
axes[0].set_title("1,000 notes: complete benchmark matrix; median with Q1/Q3")
axes[1].axhline(64, color="#ba734c", linestyle="--", linewidth=1)
axes[1].set_xticks(x, ["Text", "Unique images", "Unique audio", "Mixed unique", "Mixed shared"])
fig.text(.08, .012, "Separate desktop sessions; API scope/background load/cache differ. Paired confirmation is reported separately.", fontsize=8)
fig.tight_layout(rect=[0, .03, 1, 1])
fig.savefig(W / "comparison.svg")
fig.savefig(W / "comparison.png", dpi=160)
