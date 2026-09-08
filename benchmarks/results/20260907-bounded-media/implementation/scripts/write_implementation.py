"""Render paired measurements and the bounded-buffer implementation report."""
from pathlib import Path
import json
import os
import statistics

WORK = Path(__file__).resolve().parent
REPO = WORK.parents[2]
OUT = REPO / 'benchmarks/results/20260907-bounded-media'
read = lambda p: json.loads(p.read_text())
timing = {r['case']: r['variants'] for r in read(WORK / 'before-after-timing/summary.json')}
rss = {r['case']: r['variants'] for r in read(WORK / 'before-after-rss/summary.json')}
attempts = [json.loads(line) for line in (WORK / 'before-after-timing/attempts.jsonl').read_text().splitlines()]
cells = []
for case in read(WORK / 'cases.json'):
    name = case['name']
    values = {}
    samples = {}
    for variant in ('current', 'accepted'):
        samples[variant] = {r['sample']: r['measurement']['elapsed_ns'] / 1e6 for r in attempts
                            if r['case'] == name and r['variant'] == variant and r['sample'] >= 0}
        q1, _, q3 = statistics.quantiles(samples[variant].values(), n=4, method='inclusive')
        values[variant] = {'median_ms': timing[name][variant]['median_ms'], 'q1_ms': q1, 'q3_ms': q3,
                           'rss_mib': rss[name][variant]['median_rss_mib']}
    before, after = values['current'], values['accepted']
    cells.append({'case': name, 'input': case, **values,
                  'time_saved_pct': 100 * (1 - after['median_ms'] / before['median_ms']),
                  'rss_change_mib': after['rss_mib'] - before['rss_mib'],
                  'faster_pairs': sum(samples['accepted'][i] < samples['current'][i] for i in samples['current']),
                  'pairs': len(samples['current'])})

bindings = read(WORK / 'final-verification/attempt-bindings.json')
summary = {'schema': 'bounded-media-implementation-v1',
           'baseline_binary_sha256': read(WORK / 'final-comparison-plan.json')['binaries']['current'],
           'accepted_binary_sha256': read(WORK / 'final-comparison-plan.json')['binaries']['accepted'],
           'timing_samples_per_variant_case': 7, 'independent_rss_samples_per_variant_case': 5,
           'warmups_each_phase': 2, 'attempts_verified_by_identity': sum(r['verified_artifacts'] for r in bindings.values()),
           'independent_artifact_checks': len(cells) * 2, 'anki_import_render_checks': len(cells),
           'quality': read(WORK / 'quality-summary.json'), 'cells': cells}
(OUT / 'implementation-summary.json').write_text(json.dumps(summary, indent=2)+'\n')

lines = ['# 有总预算的媒体缓冲池', '',
    '本轮只采用共享媒体缓冲池：让大文件使用其他任务尚未占用的容量，并复用已消费的固定缓冲块。公共 API、同步注册指纹、来源变更检测、完整校验和最终同步发布保持原有语义。', '',
    '池内最多 72 个 64 KiB 缓冲块，合计 4.5 MiB，包括正在写入、排队和空闲的块。分配按需发生；预算耗尽时立即转入私有临时文件，不等待其他任务释放内存。消费、失败和退出都会归还缓冲块。该限制约束编码缓冲，编解码器、输入模型和分配器仍有额外成本，因此不是总 RSS 上限。', '',
    '![Large-media time and memory before and after the bounded pool](bounded-media.svg)', '',
    '## 同场前后对比', '',
    '29 个冻结输入；每个版本、每个场景 7 次交错耗时测量，以及另一次独立阶段中的 5 次 RSS 测量；每个阶段先预热 2 次。每次使用新进程，并在计时外记录原生电源来源。耗时包含启动、解析、媒体注册、默认导出及检查。保留全部样本，不挑最快轮次；文件缓存未受控。Q1–Q3 为样本分布，不是置信区间。', '',
    '| 场景 | 原版本 ms [Q1, Q3] | 当前版本 ms [Q1, Q3] | 节省耗时 | 更快配对 / 7 | RSS MiB，原 → 当前 |',
    '| --- | ---: | ---: | ---: | ---: | ---: |']
for row in cells:
    a, b = row['current'], row['accepted']
    def ms(x): return f"{x['median_ms']:.2f} [{x['q1_ms']:.2f}, {x['q3_ms']:.2f}]"
    lines.append(f"| {row['case']} | {ms(a)} | {ms(b)} | {row['time_saved_pct']:+.2f}% | {row['faster_pairs']}/7 | {a['rss_mib']:.2f} → {b['rss_mib']:.2f} |")
lines += ['',
    '大媒体输入为固定像素、随机填充数据的有效 PNG：`image-64x1m` 含 64 个约 1 MiB 文件；`image-256x256k` 含 256 个约 256 KiB 文件；`image-skewed-128` 每四个文件中一个约 1 MiB，其余约 64 KiB。这些合成场景用于检查文件大小及落盘行为，不代表真实高分辨率图片的全部分布。两个 boundary 场景分别使用 65,536 / 65,537 字节媒体。长文本场景固定 1,000 条笔记并改变 Back 字段大小。', '',
    '## 内存与未采用的改动', '',
    '诊断分配器在三个文本场景中记录到相同的分配次数，Rust 请求的堆峰值差异不足 32 字节；当前实现未增加这些场景的 Rust 堆容量。进程 RSS 仍随构建产物和分配器表现变化，实际数据保留在上表，不将堆计数等同于进程内存。媒体 1 MiB 场景的累计 Rust 分配量由约 202.7 MiB 降至约 145.3 MiB；该计数包含每次分配/重分配请求，不能当成 RSS，也不能用带计数器的耗时宣称提速。', '',
    '另试验了借用已导入笔记的身份信息，以减少只读取 ID 时的整份身份复制。在同场 `pool` / `pool-borrowed` 对照中，10,000 条文本的中位耗时改善约 0.6%；收益较小，未纳入当前实现。没有恢复此前已否定的 SQLite 缓存、关闭同步、延迟建索引或整次构建身份缓存方案。', '',
    '## 正确性与复现', '',
    f"新增大媒体在空闲预算内不落盘的测试先在原实现失败，再在当前实现通过。并发共享、预算耗尽、落盘回退、空写、短写、错误和 panic 路径均已验证。仓库全功能测试共 1,011 项通过，25 项原有测试保持 ignored；默认 facade、Clippy、fmt、文档、打包及依赖策略检查通过，43 项 Python 基准测试通过。", '',
    f"{summary['attempts_verified_by_identity']:,} 次实验导出均按 SHA-256 绑定到对应场景的已验证 APKG。最终 29 个场景的前后共 58 个包通过独立原始 SQLite、模板、媒体与完整包检查，并逐字节相同；29 个当前版本包通过固定版本 Anki 的导入、内容与渲染检查。", '',
    '另完成 [三轮 Rust / genanki 整体矩阵](report.md)：覆盖五种标准输入和四档数量，2,520 次导出检查及 120 次 Anki 导入/渲染检查。标准矩阵的单文件约为 64 KiB PNG / 32 KiB WAV，与上面的大文件对照是不同实验；不叠加两组收益，也不跨会话比较绝对时间。', '',
    '全部计划、尝试、构建记录、失败测试、校验日志与源文件证据位于 [implementation/](implementation/)。两个诊断构建曾因复制文件保留旧时间戳而复用旧 crate；在测量前发现并排除，记录仍保留。修正构建器后重新编译，两份有效分配计数构建均有完整编译记录。详情见 [候选源代码证据](implementation/probes/README.md)。', '',
    '[source.patch](source.patch) 固定最终实现；[implementation-summary.json](implementation-summary.json) 提供全部统计。此前的 [瓶颈审计](../20260907-post-streaming-audit/README.md) 与 [上一轮实现](../20260907-streaming-followup/implementation.md) 保留原样。', '']
(OUT / 'implementation.md').write_text('\n'.join(lines))

os.environ.setdefault('MPLCONFIGDIR', str(WORK / 'matplotlib'))
os.environ.setdefault('XDG_CACHE_HOME', str(WORK / 'cache'))
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
plt.rcParams.update({'font.family': 'DejaVu Sans', 'font.size': 10,
                     'svg.fonttype': 'none', 'svg.hashsalt': 'bounded-media-20260907'})
selected = [('image-64x1m', '64 × 1 MiB images'),
            ('image-skewed-128', '128 mixed-size images'),
            ('image-256x256k', '256 × 256 KiB images'),
            ('image-1000', '1,000 × 64 KiB images')]
lookup = {r['case']: r for r in cells}
fig, (left, right) = plt.subplots(1, 2, figsize=(12, 4.9), gridspec_kw={'width_ratios': [1.5, 1]})
colors = {'current': '#94a3ad', 'accepted': '#16745e'}
for i, (name, label) in enumerate(selected):
    row = lookup[name]
    for variant, offset in [('current', -.17), ('accepted', .17)]:
        r = row[variant]
        left.barh(i+offset, r['median_ms'], height=.28, color=colors[variant],
                  label='Before' if variant == 'current' and i == 0 else 'After' if i == 0 else None)
        left.errorbar(r['median_ms'], i+offset,
                      xerr=[[r['median_ms']-r['q1_ms']], [r['q3_ms']-r['median_ms']]],
                      color='#263c46', capsize=3, linewidth=1)
    left.text(max(row[v]['q3_ms'] for v in colors)+4, i+.03,
              f"{row['time_saved_pct']:+.1f}% time saved", va='center', fontsize=9)
    a, b = row['current']['rss_mib'], row['accepted']['rss_mib']
    right.plot([a,b], [i,i], color='#d3dce0', linewidth=3)
    right.scatter([a,b], [i,i], c=[colors['current'],colors['accepted']], s=50, zorder=3)
    right.text(max(a,b)+1.1, i, f'{a:.1f} → {b:.1f}', va='center', fontsize=9)
left.set_yticks(range(len(selected)), [label for _, label in selected])
right.set_yticks(range(len(selected)), [''] * len(selected))
left.set_xlim(0, max(lookup[n]['current']['q3_ms'] for n,_ in selected)*1.47)
right.set_xlim(0, max(lookup[n][v]['rss_mib'] for n,_ in selected for v in colors)*1.37)
for axis in (left, right):
    axis.set_ylim(-.6, len(selected)-.4)
    axis.invert_yaxis()
    axis.spines[['top','right','left']].set_visible(False)
    axis.tick_params(axis='y', length=0)
    axis.grid(axis='x', color='#e7edef', linewidth=.7)
    axis.set_axisbelow(True)
left.set_xlabel('End-to-end export time (ms) · median, Q1–Q3')
right.set_xlabel('Peak RSS (MiB) · independent samples')
left.legend(loc='upper left', bbox_to_anchor=(0,1.16), frameon=False, ncol=2)
fig.suptitle('Shared buffers reduce large-media export time', x=.03, y=.98, ha='left', fontsize=17, fontweight='bold')
fig.text(.03,.02,'Before → after: same inputs, 7 interleaved timings and 5 independent RSS samples each. Apple M1 Pro · battery power.',fontsize=9,color='#51616b')
fig.subplots_adjust(left=.20,right=.98,top=.80,bottom=.17,wspace=.12)
fig.savefig(OUT/'bounded-media.svg', metadata={'Date': None})
fig.savefig(WORK/'bounded-media.png', dpi=170)
plt.close(fig)
print(OUT/'implementation.md')
