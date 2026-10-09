"""Generate the round-four report from audited, unmodified result records."""
from pathlib import Path
import csv,json
from datetime import datetime
from zoneinfo import ZoneInfo

W=Path(__file__).resolve().parent;R=W.parents[2]
def read(path):return json.loads(path.read_text())
current=read(W/'summary.json')
old=read(R/'benchmarks/results/20260921-readme-genanki/summary.json')
previous=read(R/'benchmarks/results/20261002-reader-cache-genanki/summary.json')
old={(c['profile'],c['notes']):c for c in old['cells']}
previous={(c['profile'],c['notes']):c for c in previous['cells']}
confirmation=read(W/'confirmation-summary.json')
labels={c['profile']:c['label'] for c in current['cells']}
large=[c for c in current['cells'] if c['notes']==1000]
delta=[]
for c in current['cells']:
 key=c['profile'],c['notes'];baseline=old[key]['rust']['time_ms']['median'];before=previous[key]['rust']['time_ms']['median'];now=c['rust']['time_ms']['median']
 delta.append(dict(profile=c['profile'],notes=c['notes'],historical_ms=baseline,round3_ms=before,round4_ms=now,historical_change_pct=100*(now/baseline-1),round3_change_pct=100*(now/before-1),round4_rss_mib=c['rust']['rss_mib']['median']))
(W/'delta.json').write_text(json.dumps(delta,ensure_ascii=False,indent=2)+'\n')
with (W/'delta.csv').open('w',newline='') as stream:
 writer=csv.DictWriter(stream,fieldnames=list(delta[0]));writer.writeheader();writer.writerows(delta)
rows=[]
for c in large:
 d=next(d for d in delta if d['profile']==c['profile'] and d['notes']==1000)
 rows.append(f"| {c['label']} | {d['historical_ms']:.3f} | {d['round3_ms']:.3f} | {d['round4_ms']:.3f} | {d['historical_change_pct']:+.1f}% | {d['round4_rss_mib']:.2f} |")
pairs=[]
for c in confirmation:
 before=c['before']['median_ms'];after=c['after']['median_ms']
 pairs.append(f"| {labels[c['profile']]} | {before:.3f} | {after:.3f} | {100*(after/before-1):+.1f}% |")
beg=datetime.fromisoformat(current['started_utc']).astimezone(ZoneInfo('Asia/Taipei'))
end=datetime.fromisoformat(current['completed_utc']).astimezone(ZoneInfo('Asia/Taipei'))
faster=sum(d['round3_change_pct']<0 for d in delta)
behind=sum(d['historical_change_pct']>0 for d in delta)
genanki=sum(c['rust']['time_ms']['median']<c['genanki']['time_ms']['median'] for c in current['cells'])
text=f'''# 继续优化：活跃 spool 写句柄与同步去重

本轮新增三项内部优化，公开函数、类型、选项和 JSON 字段不变：复用活跃 spool 块写句柄；用私有、不可克隆的候选所有者保留“已同步”事实；解压 collection 时同步计算校验摘要。保留此前所有未提交优化、4 MiB 快照驻留预算、完整媒体和身份校验。

**仍未追平 9 月 21 日。** 正式矩阵 {behind}/20 格的 Rust 中位数高于历史记录。相对上一轮归档有 {faster}/20 格更快，但这是跨会话比较；本轮同机交错确认才用于判断新增改动的整体收益。

## 正式矩阵

数据采集于 {beg:%Y-%m-%d %H:%M:%S}–{end:%H:%M:%S} Asia/Taipei。沿用同一份 9/21 输入：5 个场景 × 100/200/500/1,000 条。每实现/格 10 次计时、5 次独立 RSS，各阶段前 3 次预热；默认 features、System allocator、Rust 1.92.0、CPython 3.11.0 / genanki 0.13.1。840 次导出全部通过原始 SQLite、字段、媒体和语义检查，40 个实际输出通过固定上游 Anki 的导入、内容和渲染检查。测量前后源码、工具、依赖、输入和 2,749 个媒体哈希不变，每次运行前后均为 AC Power。

下表为完整矩阵的 1,000 条单元格，时间单位 ms：

| 场景 | 9/21 Rust | 上一轮 Rust | 本轮 Rust | 相对 9/21 | 本轮 RSS MiB |
|---|---:|---:|---:|---:|---:|
{chr(10).join(rows)}

![历史、上一轮与本轮的完整矩阵结果](comparison.svg)

全部 20 格及分位数见 [summary.json](summary.json)、[comparison.csv](comparison.csv)、[delta.csv](delta.csv)。500 条纯文本略慢于上一轮（45.985→46.255 ms，约 +0.6%），没有移除或替换这格。本轮 Rust 在 {genanki}/20 格快于本轮重新测量的 genanki；这不等于追平历史 Rust。1,000 张图片 RSS 40.12 MiB，保持低于既有 64 MiB 目标。

旧 Deck 和当前 Project 的行为范围不同，桌面后台负载、CPU 状态和缓存没有隔离。不同会话的绝对差值是描述性数据，不能全部归因于某个改动，也不能用 genanki 比例线性校正。

## 未插桩版本的同场确认

在正式矩阵前单独运行两个未插桩二进制：上一轮最终版本与本轮三项改动。每场景 3 次预热、10 对交错计时，其中 5 对优化前先跑、5 对优化后先跑。130 次导出、10 次 Anki 检查全部通过；其样本没有替换正式矩阵中的任何结果。

| 场景（1,000 条） | 优化前 ms | 优化后 ms | 耗时变化 |
|---|---:|---:|---:|
{chr(10).join(pairs)}

见 [confirmation-summary.json](confirmation-summary.json)。这是 10 个描述性样本的中位数，不作统计显著性或稳定百分比承诺。诊断组 RSS 来自计时进程的高水位，正式矩阵才使用独立进程 RSS 样本。

## 瓶颈与单变量实验

四组探索控制均预先固定顺序，2 次预热、6 次正式计时，覆盖 5 个 1,000 条场景；每个输出完整校验，每配置/场景选择一个实际输出做 Anki 检查。所有正反样本保留。

- **活跃 writer：采用。** 同一插桩二进制的图片 spool 写入 57.73→32.05 ms，整个 Snapshot::file 139.39→113.60 ms；音频 spool 45.63→14.27 ms，Snapshot::file 106.74→73.76 ms。整体图片 375.41→337.13 ms，音频 254.96→216.93 ms。共享媒体不触发 spool，其整体反向样本 74.70→76.20 ms 也保留。缓存只有 Weak，不延长块所有权；仅活跃块保留一个 writer，轮换、部分写失败或最后所有者释放时关闭；worker reader cache 保持独立文件位置。
- **私有候选同步去重：采用。** 初次 retain 仍同步，所以临时输出、比较和首次同步失败的分类/顺序保持原样。未暴露的 PrivateCandidate 经过只读检查后，同设备最终 rename 可复用已同步 inode，仍同步目标目录。EXDEV 新副本以及公开 persist_to 都必须再次同步。独立未插桩对照中文本 64.02→56.82 ms、共享媒体 77.83→71.75 ms；图片反向 318.97→328.93 ms。单个短组的端到端差值不能直接视为 sync 的净成本。
- **collection 解压时计算 BLAKE3：采用。** 在现有有界 sink 成功写入的字节上计算相同摘要，消除随后 file_hash 的再次打开/整文件读取。CRC、zstd、多帧与解码预算、身份格式优先级和全部 SQL 验证保留。独立对照文本 64.02→61.58 ms，图片反向 318.97→342.48 ms；收益以最终组合确认解释，不宣称每格单独显著变快。
- **保留 NONBLOCK 以省去两次 fcntl：不采用。** 图片所有 F_GETFL/F_SETFL 合计只有约 0.65 ms，根本不足以解释该组超过 20 ms 的整体差值。正常媒体读取的同步语义保持原实现。
- **扩大内存预算：不采用。** 在 active writer 基础上，4/12/16 MiB 的图片耗时约 289.41/294.22/297.27 ms，诊断 RSS 40.12/48.10/52.38 MiB。音频和混合媒体有部分收益，但图片没有改善，内存明确增加；保留 4 MiB。
- **小文件一次性 BLAKE3：不采用。** 图片 hash 阶段 38.93→38.04 ms，音频 20.42→20.36 ms，混合 21.48→21.41 ms；整体差值远大于这些阶段差值，且音频/混合反向。额外的延迟哈希和大文件提升分支没有足够收益。

阶段是包含子阶段的 wall time，不能直接相加。新增细分探针指出：图片源文件 open 约 24 ms、预检 metadata 约 8 ms、BLAKE3 约 38 ms；源 close 仅约 0.6 ms、内存复制约 1.3 ms。因此后续重点仍是导入和必要指纹成本，关闭句柄或减少一次 memcpy 不足以填补剩余差距。不能删除源文件类型/限额/FIFO 防护，也不能通过跳过指纹或身份验证获得成绩。

## 行为与失败路径验证

本轮相对上一轮源码的完整差异见 [round4.patch](round4.patch)，共 9 个生产/测试文件。生产不包含实验环境开关或计时日志。

- 活跃块轮换关闭旧 writer，reader seek 不改变 append 位置；部分 append 失败后封存旧块并能在新块重试；创建下一块失败后恢复；最后一个 owner 删除存储。64 FD 限制下导出 74 个块/300 个媒体，并追加 96 次活跃块的创建与释放。
- 构造继承状态，验证遗留 mutex 锁不会阻塞析构、FD 已关闭、父路径保留；真实 Python fork 探针覆盖继承对象拒绝使用、子进程丢弃继承 Python 包装（绑定层刻意遗忘其 Rust 值，继承 FD 由进程退出回收）并独立导入/导出 9 个媒体、父进程随后继续向原活跃块追加并导出 10 个媒体、19 个解码 payload 的独立 SHA-256，以及两边的最终清理。它测试 Python 支持的 fork 边界，未宣称验证直接操作继承 Rust 对象。直接使用 Rust 的场景中，fork 复制的 Arc 计数也可能使 owner 无法析构，继承 FD 保留到子进程退出。
- 首次 sync 失败仍是 Io / BUILD.WORKSPACE_FAILED，并先于包检查；比较和策略阻断清理私有候选；EXDEV 新副本先同步再替换，副本 sync 失败保留旧目标和未发布事实；公开 artifact 被调用者改写后 persist_to 仍复制并同步；晚期目录 sync 失败保留 Published / Unconfirmed 事实。
- collection sink 的短写、部分失败、flush 错误、多帧/skippable 帧、精确/少一字节预算，以及 archive/identity 错误优先级均有回归。最终目标回归为 159 个库单测、11 个 artifact 生命周期、11 个媒体生命周期、23 个公开表面消费者测试，均通过；仓库 verify-fast 通过。Windows 专属发布故障路径本机未执行。

正式 840 次导出/40 次 Anki，加诊断 570 次导出/65 次 Anki，全部成功。另有真实 fork 的两次导出（非计时、未执行 Anki oracle）。离线审计核对完整样本顺序、原始测量、学习内容、trace、summary 和冻结探针源码哈希，不把离线复算当成重新导入 Anki。

oracle 本轮 locked/offline 重建，二进制哈希改变；checker 源码、上游 revision/patch 与冻结参考一致，新构建来源已记录。准备脚本最初假设二进制哈希不变而停止，修正记录后才开始最终确认和正式矩阵；没有因此丢弃导出样本。真实 fork 的首次直接 cargo 链接缺少 macOS 扩展动态查找参数，第二次按扩展所需参数链接成功；两个构建日志均保留。这些准备失败不计作 benchmark 样本。

## 剩余差距

1,000 张图片相对 9/21 仍约多 32.46 ms。下一阶段可独立评估复用身份验证已读取的 collection/model/card 事实，避免 native Summary 再扫描；必须保留 notes.data、cards.nid 等额外解码检查，不能只信任一个“已验证”标志。启动期内置默认值解码也有数毫秒成本。媒体导入的必要哈希仍是较大固定成本；本轮未找到保持既有身份与校验行为且显著降低它的方法。

没有宣称已经达标。复核入口与完整证据索引见 [README](README.md)。
'''
(W/'report.md').write_text(text)

import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import numpy as np
plt.rcParams.update({'font.family':'DejaVu Sans','font.size':10,'svg.fonttype':'none'})
fig,axes=plt.subplots(1,2,figsize=(13,5.6),gridspec_kw={'width_ratios':[1.45,1]})
y=np.arange(len(large));width=.23
for index,(source,name,color) in enumerate([(old,'Sep 21','#9299a2'),(previous,'Reader cache','#99bab4'),({(c['profile'],c['notes']):c for c in current['cells']},'Active writer','#227969')]):
 for ax,metric in zip(axes,['time_ms','rss_mib']):
  values=[source[c['profile'],1000]['rust'][metric]['median'] for c in large]
  positions=y+(index-1)*width
  ax.barh(positions,values,height=width*.92,label=name,color=color)
  for pos,value in zip(positions,values):ax.text(value+(.9 if metric=='time_ms' else .3),pos,f'{value:.1f}',va='center',fontsize=8)
for ax,title in zip(axes,['Elapsed time (ms)','Independent peak RSS (MiB)']):
 ax.set_yticks(y,[c['english_label'] for c in large] if ax is axes[0] else [])
 ax.invert_yaxis();ax.set_title(title,loc='left',fontweight='bold');ax.spines[['top','right','left']].set_visible(False);ax.grid(axis='x',alpha=.15);ax.set_axisbelow(True)
axes[0].set_xlim(0,395);axes[1].set_xlim(0,70);axes[1].axvline(64,ls='--',lw=1,color='#777777');axes[0].legend(loc='lower right',frameon=False,fontsize=9)
fig.suptitle('1,000 notes · complete benchmark matrix',x=.16,y=.98,ha='left',fontweight='bold',fontsize=16)
fig.text(.16,.035,'Separate sessions and API generations; background load/cache uncontrolled.\nMedian: 10 timing runs and 5 independent RSS runs per cell. No samples replaced.',fontsize=9,color='#555555')
fig.subplots_adjust(left=.16,right=.98,top=.87,bottom=.16,wspace=.1)
fig.savefig(W/'comparison.svg');fig.savefig(W/'comparison.png',dpi=160);plt.close(fig)
print(json.dumps({'historical_slower_cells':behind,'faster_than_previous_archive':faster,'faster_than_fresh_genanki':genanki,'report':'report.md'}))
