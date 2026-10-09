"""Offline comparison of the optimized run with both immutable baselines."""
import csv
import datetime as dt
import json
import os
from pathlib import Path
import sys
WORK = Path(__file__).resolve().parent
REPO = WORK.parents[2]
sys.path.insert(0, str(REPO / 'benchmarks'))
import bench
new = json.loads((WORK / 'summary.json').read_text())
bases = {
    'preoptimization': REPO / 'benchmarks/results/20261002-latest-genanki',
    'historical': REPO / 'benchmarks/results/20260921-readme-genanki',
}
old = {name: json.loads((path / 'summary.json').read_text()) for name, path in bases.items()}
lookups = {name: {(c['profile'],c['notes']):c for c in summary['cells']} for name,summary in old.items()}
manifest_path = WORK / 'run-manifest.json'
if not manifest_path.exists():
    manifest_path = REPO / 'benchmarks/.work/runs/20261002-optimized-genanki/manifest.json'
manifest = json.loads(manifest_path.read_text())
verification = json.loads((WORK / 'verification-summary.json').read_text())
assert verification['status'] == 'passed'
rows=[]
for c in new['cells']:
    r={'profile':c['profile'],'label':c['label'],'notes':c['notes']}
    for adapter in ('rust','genanki'):
        for metric in ('time_ms','rss_mib','apkg_bytes'):
            after=c[adapter][metric]['median']
            r[f'{adapter}_{metric}_after']=after
            for name,lookup in lookups.items():
                before=lookup[(c['profile'],c['notes'])][adapter][metric]['median']
                r[f'{adapter}_{metric}_{name}']=before
                r[f'{adapter}_{metric}_change_from_{name}_pct']=100*(after/before-1)
        for q in ('q1','q3'):
            r[f'{adapter}_time_ms_after_{q}']=c[adapter]['time_ms'][q]
    r['rust_over_genanki_after']=c['rust']['time_ms']['median']/c['genanki']['time_ms']['median']
    pre=lookups['preoptimization'][(c['profile'],c['notes'])]
    r['rust_over_genanki_relative_change_pct']=100*(r['rust_over_genanki_after']/(pre['rust']['time_ms']['median']/pre['genanki']['time_ms']['median'])-1)
    rows.append(r)
bench.save(WORK/'delta.json',{'baselines':{name:str(path.relative_to(REPO)) for name,path in bases.items()},'current_source_commit':new['source_commit'],'exploratory_dirty_source':True,'cells':rows})
with (WORK/'delta.csv').open('w',newline='') as f:
    writer=csv.DictWriter(f,fieldnames=list(rows[0]),lineterminator='\n');writer.writeheader();writer.writerows(rows)
local=lambda s:dt.datetime.fromisoformat(s).astimezone(dt.timezone(dt.timedelta(hours=8))).strftime('%Y-%m-%d %H:%M:%S')
faster=sum(r['rust_time_ms_change_from_preoptimization_pct']<0 for r in rows)
lines=['# 2026-10-02 性能优化与完整 benchmark 复测','',
 f'20 格矩阵中有 {faster} 格 Rust 耗时中位数低于本日优化前结果。全部 840 次导出验证和 40 次 Anki 导入、内容、代表性渲染检查通过。以下同时给出 9 月 21 日历史基线与本轮 genanki 数据，供评估剩余差距。','',
 f'测量时间：{local(new["started_utc"])}–{local(new["completed_utc"])}（Asia/Taipei）。Git HEAD：`{new["source_commit"]}`；本轮包含未提交的生产优化，精确源码和补丁已冻结，结果为 exploratory 本机描述性数据。','',
 '## 实现修改','',
 '- Project 的不可变媒体快照直接进入 PreparedMedia 的有界压缩队列和顺序 ZIP 写入。移除了原始快照到输入文件、CAS、staging 的重复字节拷贝及中间文件 sync；最终产物同步、校验与发布保留。持久化 CAS 的底层接口保留原来的完整性和同步逻辑。',
 '- 媒体快照共享 4 MiB 驻留内存预算（按 Vec capacity 计费）；大对象或预算之外的对象进入私有临时文件。去重快照共享一次计费，最后一个所有者释放预算和存储。临时快照只保留 TempPath，读时打开文件，避免长期占用数百个句柄。该预算不包含导入中的缓冲、调用者原有内存、编码池、数据库或检查器，所以不是整个进程的 RSS 上限。',
 '- 身份校验从已有 notes 扫描读取字段和标签，复用 cards/decks SQL 语句，每个模型只解码一次模板目标 deck 配置，并复用同一批 card ordinal 做映射校验。内容指纹、缺失 deck/template、目标 deck、flags、重复/未映射 card 检查仍然执行。',
 '- 完成检查和策略判定后，独占临时产物在同一文件系统中同步并原子 rename 到目的地，避免再次复制整包；跨设备使用原来的 copy-before-replace 路径。公共 persist_to 和共享句柄仍然复制原文件。目录同步失败的已发布/持久性未确认事实仍然上报。','',
 '## 1,000 条笔记','',
 '| 场景 | 优化前 → 后 ms | 耗时变化 | 9/21 ms | 相对 9/21 | RSS 前 → 后 MiB | 9/21 RSS MiB | 本轮 genanki ms |',
 '| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |']
for r in rows:
    if r['notes']==1000:
        lines.append(f"| {r['label']} | {r['rust_time_ms_preoptimization']:.3f} → {r['rust_time_ms_after']:.3f} | {r['rust_time_ms_change_from_preoptimization_pct']:+.1f}% | {r['rust_time_ms_historical']:.3f} | {r['rust_time_ms_change_from_historical_pct']:+.1f}% | {r['rust_rss_mib_preoptimization']:.2f} → {r['rust_rss_mib_after']:.2f} | {r['rust_rss_mib_historical']:.2f} | {r['genanki_time_ms_after']:.3f} |")
lines+=['','负值表示降低。耗时为新进程启动到退出的中位数，包含媒体导入、构建、默认检查、发布和清理。RSS 为另 5 次独立进程的 OS 高水位中位数。','',
 '![三个版本在 1,000 条笔记时的耗时和 RSS](comparison.png)','',
 '本轮 1,000 条笔记的 genanki 耗时也比优化前降低约 20%–32%，提示会话环境存在变化。Rust 的独立媒体耗时相对本日优化前降低约 96%，但仍比 9/21 高约 67%–99%，且比本轮 genanki 慢约 16%–28%。共享媒体 RSS 从 32.91 升到 36.94 MiB，文本 RSS 也小幅增加；有界并行压缩有额外工作内存。这轮没有给这些剩余差距做新的单变量归因。','',
 '## 完整 20 格耗时','',
 '| 场景 | 笔记 | 优化前 → 后 ms | 变化 | 优化后 Q1–Q3 ms | 相对 9/21 | 本轮 genanki ms | Rust / genanki |',
 '| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |']
for r in rows:
    lines.append(f"| {r['label']} | {r['notes']} | {r['rust_time_ms_preoptimization']:.3f} → {r['rust_time_ms_after']:.3f} | {r['rust_time_ms_change_from_preoptimization_pct']:+.1f}% | {r['rust_time_ms_after_q1']:.3f}–{r['rust_time_ms_after_q3']:.3f} | {r['rust_time_ms_change_from_historical_pct']:+.1f}% | {r['genanki_time_ms_after']:.3f} | {r['rust_over_genanki_after']:.2f}× |")
lines+=['','## 完整 20 格 RSS 和包体积','','| 场景 | 笔记 | RSS 前 → 后 MiB | RSS 变化 | 9/21 RSS MiB | APKG 前 → 后 MiB |','| --- | ---: | ---: | ---: | ---: | ---: |']
for r in rows:
    lines.append(f"| {r['label']} | {r['notes']} | {r['rust_rss_mib_preoptimization']:.2f} → {r['rust_rss_mib_after']:.2f} | {r['rust_rss_mib_change_from_preoptimization_pct']:+.1f}% | {r['rust_rss_mib_historical']:.2f} | {r['rust_apkg_bytes_preoptimization']/2**20:.3f} → {r['rust_apkg_bytes_after']/2**20:.3f} |")
lines+=['','## 验证与条件','',
 '- 修复前的媒体复制路径回归测试和共享内存预算测试均观察到失败；64 个文件描述符的子进程测试另复现了溢出后长期持有文件句柄的问题。修复后 142 项默认核心测试和 8 项媒体生命周期测试通过，包括部分写入失败、资源清理、去重、相对 TMPDIR 和 fork 锁/所有权行为。',
 '- 仓库 verify-ci.sh --fast（含 Python 脚本测试、格式、合同治理、clippy 和 workspace tests）通过；最后一次实现调整后又运行 workspace clippy、核心和生命周期测试。完整日志归档。',
 '- 46 项 benchmark 行为测试及最终独立 200-note smoke 导出通过。正式矩阵不包含构建、测试、smoke 或绘图。Anki oracle 在当前锁定依赖上重建，保留相同 pinned upstream 和 checker 源文件。',
 '- 相同 M1 Pro / 32 GiB / macOS 27.0 (26A428) / ARM64、Rust 1.92.0 release/default/System allocator、CPython 3.11.0 / genanki 0.13.1。只测 Rust API；Node/Python 绑定未单独计时。',
 '- 5 场景 × 100/200/500/1,000；20 份输入 JSON 与历史基线 SHA-256 一致，2,749 个媒体文件测前测后相同。每实现每格 3 次计时预热、10 次交错计时、3 次 RSS 预热、5 次独立 RSS。共 840 次导出，400 计时、200 RSS、240 预热；每格计时次序各 5 次 Rust-first/genanki-first。',
 '- 840 个原始 APKG 的 SQLite、内容、媒体及语义验证通过，40 个选中产物的 Anki 导入检查通过。源码、二进制、依赖、输入测前测后哈希相同，每次测量前后为 AC Power。Q1/Q3 为 Hyndman–Fan type 7，不剔除异常值。',
 f'- 负载（1/5/15 分钟）：测前 {manifest["host_before"]["load"]}，测后 {manifest["host_after"]["load"]}。供电详情：测前 {manifest["host_before"]["power"]!r}，测后 {manifest["host_after"]["power"]!r}。三次完整矩阵来自不同桌面会话，负载、充电和缓存未受控；本日优化前电池为 17% 放电 → 12% 充电，9/21 为 100%。不能将跨会话差值直接等同于单变量因果效果。诊断的同二进制消融和同机历史源码对照见 [诊断证据](../20261002-performance-diagnosis/report.md)。',
 '- 新旧 API 对比仍包含 Project 快照所有权、身份完整性检查、报告等新行为；与 genanki 的文件大小比较包含默认 APKG 格式和压缩差异。保留所有慢于历史基线或 genanki 的格子，不作显著性或跨机器推断。',
 '- 测前 Git 状态包含本轮生产优化和其他正在进行的 README/网站/文档修改；本轮未修改这些无关内容，完整状态与快照记录在 plan/source-snapshot 中。','',
 '机器数据：[summary.json](summary.json)、[comparison.csv](comparison.csv)、[两份基线差异 delta.csv](delta.csv)。完整证据与重放方法见 [README](README.md)。']
(WORK/'report.md').write_text('\n'.join(lines)+'\n')
os.environ.setdefault('MPLCONFIGDIR',str(REPO/'benchmarks/.work/matplotlib'))
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import numpy as np
cells=[c for c in new['cells'] if c['notes']==1000]
x=np.arange(5);width=.24
fig,axes=plt.subplots(2,1,figsize=(10,7.5),sharex=True)
for offset,name,label,color in [(-1,'historical','2026-09-21 Deck','#8696a5'),(0,'preoptimization','2026-10-02 before','#c96747'),(1,None,'2026-10-02 optimized','#247a73')]:
    data=cells if name is None else [lookups[name][(c['profile'],1000)] for c in cells]
    for ax,metric in zip(axes,('time_ms','rss_mib')):
        values=[c['rust'][metric]['median'] for c in data]
        bars=ax.bar(x+offset*width,values,width,label=label,color=color)
        ax.bar_label(bars,labels=[f'{v:.1f}' for v in values],fontsize=8,padding=3)
axes[0].set_yscale('log');axes[0].set_ylim(20,22000);axes[0].set_ylabel('Elapsed time (ms, log scale)');axes[0].legend(ncol=3,loc='lower center',bbox_to_anchor=(.5,1.04),fontsize=9)
axes[1].set_ylim(0,125);axes[1].set_ylabel('Peak RSS (MiB)');axes[1].set_xticks(x,labels=[c['english_label'].replace(', ','\n') for c in cells])
for ax in axes:
    ax.grid(axis='y',alpha=.2);ax.set_axisbelow(True);ax.spines[['top','right']].set_visible(False)
fig.suptitle('Rust default export: 1,000 notes per scene',fontsize=15)
fig.text(.5,.018,'M1 Pro / 32 GiB / System allocator · 10 timings and 5 independent RSS samples per cell\nSeparate desktop sessions; load, power state and cache uncontrolled',ha='center',fontsize=8)
fig.tight_layout(rect=(0,.06,1,.96));fig.savefig(WORK/'comparison.png',dpi=160);fig.savefig(WORK/'comparison.svg',metadata={'Date':None})
print(json.dumps({'faster_cells_vs_preoptimization':faster,'at_1000':[r for r in rows if r['notes']==1000]},ensure_ascii=False,indent=2))
