"""Offline comparison to the immutable 2026-09-21 archived baseline."""
import csv
import datetime as dt
import json
from pathlib import Path
import os
import sys

WORK = Path(__file__).resolve().parent
REPO = WORK.parents[2]
sys.path.insert(0, str(REPO / 'benchmarks'))
import bench
BASE = REPO / 'benchmarks/results/20260921-readme-genanki'
old = json.loads((BASE / 'summary.json').read_text())
new = json.loads((WORK / 'summary.json').read_text())
manifest_path = WORK / 'run-manifest.json'
if not manifest_path.is_file():
    manifest_path = REPO / 'benchmarks/.work/runs/20261002-latest-genanki/manifest.json'
manifest = json.loads(manifest_path.read_text())
verification = json.loads((WORK / 'verification-summary.json').read_text())
assert verification['status'] == 'passed'
lookup = {(c['profile'], c['notes']): c for c in old['cells']}
rows = []
for c in new['cells']:
    b = lookup[(c['profile'], c['notes'])]
    row = {'profile': c['profile'], 'label': c['label'], 'notes': c['notes']}
    for adapter in ('rust', 'genanki'):
        for metric in ('time_ms', 'rss_mib', 'apkg_bytes'):
            before = b[adapter][metric]['median']
            after = c[adapter][metric]['median']
            row.update({f'{adapter}_{metric}_before': before, f'{adapter}_{metric}_after': after,
                        f'{adapter}_{metric}_change_pct': 100 * (after / before - 1),
                        f'{adapter}_{metric}_delta': after - before})
        for period, cell in (('before', b), ('after', c)):
            for q in ('q1', 'q3'):
                row[f'{adapter}_time_ms_{period}_{q}'] = cell[adapter]['time_ms'][q]
    row['time_saved_pct_before'] = b['time_saved_pct']
    row['time_saved_pct_after'] = c['time_saved_pct']
    row['time_saved_change_pp'] = c['time_saved_pct'] - b['time_saved_pct']
    row['rust_relative_to_genanki_ratio_change_pct'] = 100 * ((c['rust']['time_ms']['median'] / c['genanki']['time_ms']['median']) / (b['rust']['time_ms']['median'] / b['genanki']['time_ms']['median']) - 1)
    rows.append(row)
bench.save(WORK / 'delta.json', {'baseline': str(BASE.relative_to(REPO)), 'current_source_commit': new['source_commit'], 'cells': rows})
with (WORK / 'delta.csv').open('w', newline='') as stream:
    writer = csv.DictWriter(stream, fieldnames=list(rows[0]), lineterminator='\n')
    writer.writeheader(); writer.writerows(rows)
local = lambda s: dt.datetime.fromisoformat(s).astimezone(dt.timezone(dt.timedelta(hours=8))).strftime('%Y-%m-%d %H:%M:%S')
lines = ['# 2026-10-02 当前代码 benchmark 与历史基线对比', '',
         f'全部 {len(rows)} 格 Rust 耗时中位数均高于旧基线。1,000 条独立图片/音频的耗时达到旧值的约 41.3/53.5 倍；峰值 RSS 也分别从 40.25/39.56 升至 98.28/66.80 MiB。本轮纯文本仍快于 genanki，全部媒体场景则慢于 genanki。跨日期比较包含环境与 API 路径差异，未定位单一根因。', '',
         f'当前提交：`{new["source_commit"]}`。测量：{local(new["started_utc"])}–{local(new["completed_utc"])}（Asia/Taipei）。', '',
         '基线：[2026-09-21 完整报告](../20260921-readme-genanki/report.md)。当前测量使用 Rust `Project` API；基线使用旧 `Deck` API。两者测量学习内容相同的默认完整导出路径，不能把差异归因到单一函数或提交。', '',
         '## 1,000 条笔记', '',
         '| 场景 | Rust 旧 → 新 ms | 耗时变化 | RSS 旧 → 新 MiB | genanki 旧 → 新 ms | 当前 Rust / genanki 耗时 |',
         '| --- | ---: | ---: | ---: | ---: | ---: |']
for r in rows:
    if r['notes'] == 1000:
        lines.append(f"| {r['label']} | {r['rust_time_ms_before']:.3f} → {r['rust_time_ms_after']:.3f} | {r['rust_time_ms_change_pct']:+.1f}% | {r['rust_rss_mib_before']:.2f} → {r['rust_rss_mib_after']:.2f} | {r['genanki_time_ms_before']:.3f} → {r['genanki_time_ms_after']:.3f} | {r['rust_time_ms_after']/r['genanki_time_ms_after']:.2f}× |")
lines += ['', '耗时变化 =（当前 / 历史 − 1）× 100%，负值表示更快。', '',
          '![20 个场景规模的耗时变化](time-delta.png)', '', '## 全部 20 格耗时', '',
          '| 场景 | 笔记 | Rust 旧 → 新 ms | 变化 | 当前 Rust Q1–Q3 ms | genanki 变化 | 相对 genanki 节省 旧 → 新 |',
          '| --- | ---: | ---: | ---: | ---: | ---: | ---: |']
for r in rows:
    lines.append(f"| {r['label']} | {r['notes']} | {r['rust_time_ms_before']:.3f} → {r['rust_time_ms_after']:.3f} | {r['rust_time_ms_change_pct']:+.1f}% | {r['rust_time_ms_after_q1']:.3f}–{r['rust_time_ms_after_q3']:.3f} | {r['genanki_time_ms_change_pct']:+.1f}% | {r['time_saved_pct_before']:.1f}% → {r['time_saved_pct_after']:.1f}% |")
lines += ['', '## 内存和文件体积', '',
          '| 场景 | 笔记 | Rust RSS 旧 → 新 MiB | RSS 变化 | Rust APKG 旧 → 新 MiB | APKG 变化 |',
          '| --- | ---: | ---: | ---: | ---: | ---: |']
for r in rows:
    lines.append(f"| {r['label']} | {r['notes']} | {r['rust_rss_mib_before']:.2f} → {r['rust_rss_mib_after']:.2f} | {r['rust_rss_mib_change_pct']:+.1f}% | {r['rust_apkg_bytes_before']/2**20:.3f} → {r['rust_apkg_bytes_after']/2**20:.3f} | {r['rust_apkg_bytes_change_pct']:+.1f}% |")
lines += ['', '## 方法与验证', '',
          '- 测前当前 HEAD 的产品源代码无已跟踪修改；当时工作区有 3 份未跟踪文档，完整 Git 状态已记录。测量按仓库规则保留为 exploratory 单次本机结果。',
          '- 相同 Apple M1 Pro、32 GiB、macOS 27.0 (26A428)、ARM64；Rust 1.92.0 release/default features/System allocator；genanki 0.13.1、CPython 3.11.0。未测 Node/Python 绑定。',
          '- 5 种场景 × 100/200/500/1,000 条笔记，20 份输入 JSON 与历史基线 SHA-256 全部相同，2,749 个媒体文件测前测后核验。',
          '- 每实现每格：3 次计时预热、10 次计时、3 次 RSS 预热、5 次独立 RSS 采样；共 840 次导出，含 400 次计时、200 次 RSS 和 240 次预热。每格计时有 5 次 Rust 先运行、5 次 genanki 先运行。',
          '- 840 次输出内容/SQLite/媒体校验和 40 次 Anki 导入/内容/代表性渲染校验全部通过；代码、二进制、依赖与输入哈希测前测后保持相同；每次导出前后均检查 AC Power。',
          '- 46 项 benchmark 测试和单独 200 条 smoke 导出通过。Anki oracle 按当前锁定依赖重建，仍使用与基线一致的 benchmark checker 源文件和 pinned upstream Anki/patch。',
          '- 计时覆盖新进程启动、解析、构建和默认导出检查、写文件到进程退出；RSS 为独立进程 OS 高水位。Q1/Q3 使用 Hyndman–Fan type 7；不删除异常值。',
          f'- 当前系统负载（1/5/15 分钟）：测前 {manifest["host_before"]["load"]}；测后 {manifest["host_after"]["load"]}。历史 1 分钟负载为 19.51 → 16.09。桌面负载与缓存未控制，跨日期变化包含运行环境影响；重新测量 genanki 作为参照，不能视为受控的同会话旧/新 Rust A/B。',
          '- 供电细节（原生记录）：测前 ' + repr(manifest['host_before']['power']) + '；测后 ' + repr(manifest['host_after']['power']) + '。历史基线电池为 100%；本轮电池电量和充电状态不同，虽 AC Power 来源保持一致，也不把它视为完全匹配的供电条件。',
          '- 单轮结果为描述性统计，不提供显著性或跨机器结论；两库默认 APKG 格式和压缩不同，体积比较包含这些差异。', '',
          '机器数据：[本轮 summary.json](summary.json)、[完整差异 delta.csv](delta.csv)、[验证记录](verification-summary.json)。原始记录和复现脚本见 [README](README.md)。']
(WORK / 'report.md').write_text('\n'.join(lines) + '\n')
os.environ.setdefault('MPLCONFIGDIR', str(REPO / 'benchmarks/.work/matplotlib'))
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.colors import Normalize
import numpy as np
profiles = [(c['profile'], c['english_label']) for c in new['cells'] if c['notes'] == 1000]
indexed = {(r['profile'],r['notes']):r for r in rows}
values = np.array([[indexed[(p,n)]['rust_time_ms_change_pct'] for n in (100,200,500,1000)] for p,_ in profiles])
lim = max(5, float(np.abs(values).max()))
fig, ax = plt.subplots(figsize=(10, 5.7))
im = ax.imshow(values, cmap='YlOrRd', norm=Normalize(vmin=0,vmax=lim), aspect='auto', alpha=.65)
ax.set_xticks(range(4),labels=['100 notes','200 notes','500 notes','1,000 notes'])
ax.set_yticks(range(5),labels=[label for _,label in profiles])
for i,(p,_) in enumerate(profiles):
    for j,n in enumerate((100,200,500,1000)):
        r=indexed[(p,n)]
        ax.text(j,i,f"{r['rust_time_ms_change_pct']:+.1f}%\n{r['rust_time_ms_before']:.1f} → {r['rust_time_ms_after']:.1f} ms",ha='center',va='center',fontsize=10)
ax.set_title('Rust export elapsed time: 2026-09-21 → 2026-10-02\nDeck API → Project API · same fixtures · 10 fresh-process samples/cell',pad=14)
fig.colorbar(im,ax=ax,label='Elapsed-time increase (%)')
fig.text(.5,.02,'M1 Pro / 32 GiB / macOS 27.0 / System allocator · Separate desktop sessions; load and cache uncontrolled',ha='center',fontsize=8)
fig.tight_layout(rect=(0,.04,1,1))
fig.savefig(WORK/'time-delta.png',dpi=160)
fig.savefig(WORK/'time-delta.svg',metadata={'Date':None})
print(json.dumps({'rust_faster_cells':sum(r['rust_time_ms_change_pct']<0 for r in rows), 'total_cells':len(rows), 'at_1000':[r for r in rows if r['notes']==1000]},ensure_ascii=False,indent=2))
