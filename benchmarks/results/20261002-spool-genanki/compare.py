"""Describe the frozen full matrix primarily against 2026-09-21; keep all results."""
import csv,datetime as dt,json,os,sys
from pathlib import Path
W=Path(__file__).resolve().parent;R=W.parents[2];sys.path.insert(0,str(R/'benchmarks'));import bench
new=json.loads((W/'summary.json').read_text());bases={'historical':R/'benchmarks/results/20260921-readme-genanki','preceding':R/'benchmarks/results/20261002-optimized-genanki'}
old={k:json.loads((v/'summary.json').read_text()) for k,v in bases.items()};look={k:{(c['profile'],c['notes']):c for c in v['cells']} for k,v in old.items()}
confirmation=json.loads((W/'confirmation-summary.json').read_text());confirmed={c['profile']:c for c in confirmation}
manifest_path=W/'run-manifest.json';manifest=json.loads((manifest_path if manifest_path.exists() else R/'benchmarks/.work/runs/20261002-spool-genanki/manifest.json').read_text());assert json.loads((W/'verification-summary.json').read_text())['status']=='passed'
rows=[]
for c in new['cells']:
 r={'profile':c['profile'],'label':c['label'],'notes':c['notes']}
 for a in ('rust','genanki'):
  for metric in ('time_ms','rss_mib','apkg_bytes'):
   after=c[a][metric]['median'];r[f'{a}_{metric}_after']=after
   for base in bases:
    before=look[base][(c['profile'],c['notes'])][a][metric]['median'];r[f'{a}_{metric}_{base}']=before;r[f'{a}_{metric}_change_from_{base}_pct']=100*(after/before-1)
  for q in ('q1','q3'):r[f'{a}_time_ms_after_{q}']=c[a]['time_ms'][q]
 r['rust_over_genanki_after']=c['rust']['time_ms']['median']/c['genanki']['time_ms']['median']
 for base in bases:
  b=look[base][(c['profile'],c['notes'])];r[f'rust_over_genanki_{base}']=b['rust']['time_ms']['median']/b['genanki']['time_ms']['median'];r[f'ratio_change_from_{base}_pct']=100*(r['rust_over_genanki_after']/r[f'rust_over_genanki_{base}']-1)
 rows.append(r)
bench.save(W/'delta.json',dict(baselines={k:str(v.relative_to(R)) for k,v in bases.items()},current_source_commit=new['source_commit'],exploratory_dirty_source=True,cells=rows))
with (W/'delta.csv').open('w',newline='') as f:
 writer=csv.DictWriter(f,fieldnames=list(rows[0]),lineterminator='\n');writer.writeheader();writer.writerows(rows)
local=lambda s:dt.datetime.fromisoformat(s).astimezone(dt.timezone(dt.timedelta(hours=8))).strftime('%Y-%m-%d %H:%M:%S')
faster=sum(r['rust_time_ms_change_from_historical_pct']<0 for r in rows);prev_faster=sum(r['rust_time_ms_change_from_preceding_pct']<0 for r in rows)
lines=['# 共享快照存储优化与 9 月 21 日基准对比','',
'已完成小快照共享落盘块和 ASCII portable filename 快速路径优化。完整矩阵 840 次导出及 40 次 Anki import/content/render 检查全部通过。主表使用本轮完整矩阵；它没有替换为较快的试跑或补充对照结果。','',
f'**本轮完整矩阵中，{faster}/20 格 Rust 耗时低于 9 月 21 日；{prev_faster}/20 格低于上一轮完整 Project benchmark。** 1,000 张图片为 483.938 ms，对 9 月 21 日的 238.638 ms 为 +102.8%；音频 +103.7%、独立混合媒体 +70.3%。这些原始劣化保留在报告中。','',
'另外，10 轮同机交错的“上一版 Project → 最终优化版”对照确认了代码收益：独立图片 −27.7%、独立音频 −31.6%、独立混合媒体 −21.7%；文本和共享媒体基本不变。该对照与正式矩阵分开，不能与本轮 genanki 结果拼接成一个新 benchmark。','',
f'正式测量时间：{local(new["started_utc"])}–{local(new["completed_utc"])} Asia/Taipei。HEAD `{new["source_commit"]}`，含原有及本轮未提交优化。精确运行时文件，包括新增未跟踪的 `media/snapshot/spool.rs`，均在源码归档中；工作树补丁与单独的 round2.patch 一并保留。','',
'## 本轮实现','',
'- 保持全局 4 MiB 快照驻留预算和单对象 1 MiB 内存阈值。小对象溢出时共用最多 4 MiB 的私有落盘块，每个快照保存独立 offset/length 和共享所有者。大对象仍用原来的独立流式临时文件。','- 区段 reader 使用独立文件位置，并限制读取到自身范围；seek 的 End 相对快照长度。共享块只在最后一个区段所有者释放后删除。缓存仅持有 Weak，不保留块或长期文件句柄；块内已释放区段的磁盘空间会保留到该块最后一个所有者释放，最大保留粒度为 4 MiB。','- 追加写入失败时尝试截断回原长度，保留原始 I/O 原因，并停止复用失败块；原有区段保持读取边界。fork 子进程不升级父进程 Weak，继承块的析构不删除父进程路径。','- ASCII 名称直接使用现有 UniCase ASCII 路径；其他名称保留 NFC 和完整 Unicode case folding。排序、相等和哈希兼容性测试覆盖大小写、Straße/STRASSE、Kelvin 符号及组合字符。','- 完整性、媒体内容哈希、身份历史、最终文件及目录同步仍执行。新增实现不含诊断跳过开关。','',
'## 1,000 条笔记：正式矩阵对 9 月 21 日','',
'| 场景 | 9/21 Rust ms | 本轮 Rust ms | 耗时差异 | 9/21 RSS MiB | 本轮 RSS MiB | RSS 差异 | 本轮 genanki ms |','|---|---:|---:|---:|---:|---:|---:|---:|']
for r in rows:
 if r['notes']==1000:lines.append(f'| {r["label"]} | {r["rust_time_ms_historical"]:.3f} | {r["rust_time_ms_after"]:.3f} | {r["rust_time_ms_change_from_historical_pct"]:+.1f}% | {r["rust_rss_mib_historical"]:.2f} | {r["rust_rss_mib_after"]:.2f} | {r["rust_rss_mib_change_from_historical_pct"]:+.1f}% | {r["genanki_time_ms_after"]:.3f} |')
lines+=['','## 完整 20 格耗时与本轮独立比较实现','','| 场景 | 笔记 | 9/21 Rust ms | 本轮 Rust ms | 差异 | 本轮 Q1–Q3 ms | 本轮 genanki ms | Rust/genanki | 对上一轮 Project 耗时差异 |','|---|---:|---:|---:|---:|---:|---:|---:|---:|']
for r in rows:lines.append(f'| {r["label"]} | {r["notes"]} | {r["rust_time_ms_historical"]:.3f} | {r["rust_time_ms_after"]:.3f} | {r["rust_time_ms_change_from_historical_pct"]:+.1f}% | {r["rust_time_ms_after_q1"]:.3f}–{r["rust_time_ms_after_q3"]:.3f} | {r["genanki_time_ms_after"]:.3f} | {r["rust_over_genanki_after"]:.2f}× | {r["rust_time_ms_change_from_preceding_pct"]:+.1f}% |')
lines+=['','## 完整 20 格 RSS 与默认包体积','','| 场景 | 笔记 | 9/21 RSS MiB | 本轮 RSS MiB | RSS 差异 | 9/21 APKG MiB | 本轮 APKG MiB |','|---|---:|---:|---:|---:|---:|---:|']
for r in rows:lines.append(f'| {r["label"]} | {r["notes"]} | {r["rust_rss_mib_historical"]:.2f} | {r["rust_rss_mib_after"]:.2f} | {r["rust_rss_mib_change_from_historical_pct"]:+.1f}% | {r["rust_apkg_bytes_historical"]/2**20:.3f} | {r["rust_apkg_bytes_after"]/2**20:.3f} |')
lines+=['','## 同机代码收益确认：独立补充对照','',
'正式矩阵完成后才启动这组预先保存计划的对照：五个 1,000-note 场景，每版 3 次预热 + 10 次计时，同一 profile 相邻运行，计时各 5 次旧版先/新版先，profile 顺序按记录种子打乱。只比较上一版未插桩 Project 二进制与最终未插桩共享块版；使用本轮正式 fixture。全部 130 次导出验证、10 次选中 Anki 检查通过；源码、工具和依赖在整个确认阶段保持不变。RSS 为这些计时进程的附带高水位，不能代替上面的独立 RSS 数据。','',
'| 场景 | 旧 Project ms | 新共享块 ms | 差异 | 全部新版样本 ms |','|---|---:|---:|---:|---|']
for c in new['cells']:
 if c['notes']==1000:
  p=confirmed[c['profile']];a=p['before']['median_ms'];b=p['after']['median_ms'];lines.append(f'| {c["label"]} | {a:.3f} | {b:.3f} | {100*(b/a-1):+.1f}% | '+', '.join(f'{n:.3f}' for n in p['after']['samples_ms'])+' |')
lines+=['',
'原始短试跑也保留：首次共享块版 40 次导出/10 次 Anki；当前块写句柄复用候选 24 次导出/6 次 Anki。后者媒体中位数额外差约 1%–2%，同时文本负控制约 +5%，不足以确认稳定收益，因此最终采用按需打开写句柄的共享块版。候选的生产源码副本留在诊断归档中，未加入最终库。','',
'## 条件、验证与剩余限制','',
f'- 同一 M1 Pro / 32 GiB / macOS 27.0 / arm64；Rust 1.92.0 release/default/System allocator，CPython 3.11.0 / genanki 0.13.1。正式负载测前 {manifest["host_before"]["load"]}，测后 {manifest["host_after"]["load"]}。所有 exporter 前后是 AC Power；正式测前电池 95% 放电，测后 93% 放电，供电状态详情原样记录。','- 桌面背景负载、缓存、充电和文件系统写回未隔离。本轮 1,000-note genanki 耗时比上一轮完整矩阵高约 27%–37%。这是环境/会话变化的线索，不能用于精确扣除环境成本，也不能用较快的确认样本替换完整矩阵。','- 9/21 测的是旧 Deck API；当前 Project 包含不可变快照所有权、完整 identity 证据与检查等新行为。跨日期原始差异仍包含 API 工作范围差异；尚未证明可在保留全部新语义时恢复旧版耗时。','- 五场景 × 100/200/500/1,000；20 份 JSON 哈希与 9/21 相同，2,749 个媒体文件在正式测量前后不变。每实现每格 3 次计时预热、10 次计时、3 次 RSS 预热、5 次独立 RSS；共 400 timing、200 RSS、240 warmup。每格计时顺序各 5 次 Rust-first/genanki-first。','- 840 个真实输出全部通过原始 SQLite、内容/card/deck、媒体及语义验证；40 个选定实际产物通过固定 upstream Anki 的导入、逐项内容与代表性渲染检查。所有源码、输入、二进制、依赖前后哈希一致；没有删除异常值或选择性重试。Q1/Q3 使用 type 7 插值。','- 回归先观察到 256 个小快照产生 192 个独立文件，修改后同样 12 MiB 落盘内容只用 3 个最多 4 MiB 的块。64 个 FD 限制下导出 256 个资产成功；另在该限制下保留 74 个块/296 MiB 落盘内容，最后所有者释放全部存储。','- 10 个默认消费者生命周期测试覆盖源删除、重复所有者、相对 TMPDIR、真实部分写入/追加失败、受限 FD 与清理；核心测试覆盖 reader 边界/seek/块轮换/继承清理，以及 ASCII/Unicode 键兼容性。最终 scripts/verify-ci.sh --fast 通过（格式、合同治理、clippy、workspace tests 和脚本检查）；46 个 benchmark 行为测试和最终 200-note smoke 通过。','- 4 MiB 是快照驻留预算，不是总 RSS 上限；导入缓冲、编码队列、数据库和检查器另占内存。本轮 1,000 独立图片 RSS 为 43.59 MiB，仍高于 9/21 的 40.25 MiB。','',
'完整证据：[README](README.md)、[逐格 CSV](delta.csv)、[完整统计](summary.json)、[同机确认](confirmation-summary.json)、[840 项验证](verification-summary.json)。图示仅使用完整矩阵。','']
(W/'report.md').write_text('\n'.join(lines))
os.environ.setdefault('MPLCONFIGDIR',str(R/'benchmarks/.work/matplotlib'));import matplotlib;matplotlib.use('Agg');import matplotlib.pyplot as plt;import numpy as np
cells=[c for c in new['cells'] if c['notes']==1000];x=np.arange(5);width=.25;fig,axes=plt.subplots(2,1,figsize=(11,7.2),sharex=True)
for offset,key,label,color in [(-1,'historical','Sep 21: Deck','#8898a5'),(0,'preceding','Oct 02: preceding Project','#ba8661'),(1,None,'Oct 02: shared-spool Project','#22796f')]:
 data=cells if key is None else [look[key][(c['profile'],1000)] for c in cells]
 for ax,metric in zip(axes,('time_ms','rss_mib')):
  values=[c['rust'][metric]['median'] for c in data];bars=ax.bar(x+offset*width,values,width,label=label,color=color);ax.bar_label(bars,labels=[f'{v:.1f}' for v in values],fontsize=8,padding=3)
axes[0].scatter(x+width,[c['genanki']['time_ms']['median'] for c in cells],color='#a43b40',marker='D',s=28,label='Current genanki',zorder=4);axes[0].set_ylabel('Elapsed time (ms)');axes[0].set_ylim(0,620);axes[0].legend(ncol=4,fontsize=8,loc='lower center',bbox_to_anchor=(.5,1.03));axes[1].set_ylabel('Peak RSS (MiB)');axes[1].set_ylim(0,55);axes[1].set_xticks(x,[c['english_label'].replace(', ','\n') for c in cells])
for ax in axes:ax.grid(axis='y',alpha=.2);ax.set_axisbelow(True);ax.spines[['top','right']].set_visible(False)
fig.suptitle('Complete default-export benchmark: 1,000 notes per scene',fontsize=14)
fig.text(.5,.015,'M1 Pro / 32 GiB / System allocator · 10 timings and 5 independent RSS samples per cell\nSeparate desktop sessions and API scope; background load and writeback uncontrolled. Confirmation samples are excluded.',ha='center',fontsize=8)
fig.tight_layout(rect=(0,.07,1,.95));fig.savefig(W/'comparison.png',dpi=160);fig.savefig(W/'comparison.svg',metadata={'Date':None})
print(json.dumps({'historical_faster_cells':faster,'preceding_faster_cells':prev_faster,'complete_cells':20},indent=2))
