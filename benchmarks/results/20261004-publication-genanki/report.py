"""Report a single audited session against Sep 21; no measurement or filtering."""
from pathlib import Path
from datetime import datetime
from zoneinfo import ZoneInfo
import csv,json
W=Path(__file__).resolve().parent
read=lambda name:json.loads((W/name).read_text())
current=read('summary.json'); historical=read('historical-summary.json')
old={(c['profile'],c['notes']):c for c in historical['cells']}
rows=[]
for c in current['cells']:
 a=old[c['profile'],c['notes']]['rust'];b=c['rust']
 rows.append(dict(profile=c['profile'],label=c['label'],notes=c['notes'],sep21_ms=a['time_ms']['median'],current_ms=b['time_ms']['median'],time_change_pct=100*(b['time_ms']['median']/a['time_ms']['median']-1),sep21_rss_mib=a['rss_mib']['median'],current_rss_mib=b['rss_mib']['median'],rss_change_mib=b['rss_mib']['median']-a['rss_mib']['median'],sep21_apkg_bytes=a['apkg_bytes']['median'],current_apkg_bytes=b['apkg_bytes']['median'],genanki_ms=c['genanki']['time_ms']['median']))
(W/'delta.json').write_text(json.dumps(rows,ensure_ascii=False,indent=2)+'\n')
with (W/'delta.csv').open('w',newline='') as f:
 writer=csv.DictWriter(f,fieldnames=list(rows[0]),lineterminator='\n');writer.writeheader();writer.writerows(rows)
faster=sum(r['time_change_pct']<0 for r in rows);slower=sum(r['time_change_pct']>0 for r in rows)
genanki=sum(c['rust']['time_ms']['median']<c['genanki']['time_ms']['median'] for c in current['cells'])
beg=datetime.fromisoformat(current['started_utc']).astimezone(ZoneInfo('Asia/Taipei'));end=datetime.fromisoformat(current['completed_utc']).astimezone(ZoneInfo('Asia/Taipei'))
def table(selected):
 return '\n'.join(f"| {r['label']} | {r['notes']} | {r['sep21_ms']:.3f} | {r['current_ms']:.3f} | {r['time_change_pct']:+.1f}% | {r['sep21_rss_mib']:.2f} | {r['current_rss_mib']:.2f} | {r['genanki_ms']:.3f} |" for r in selected)
header='| 场景 | 笔记数 | 9/21 ms | 本轮 ms | 耗时变化 | 9/21 RSS MiB | 本轮 RSS MiB | 本轮 genanki ms |\n|---|---:|---:|---:|---:|---:|---:|---:|'
lines=['# 当前实现与 9 月 21 日基准对比','',f'**本轮相对 9 月 21 日：{faster}/20 格耗时中位数降低，{slower}/20 格升高，{20-faster-slower}/20 格相同。** 当前 Rust 在 {genanki}/20 格快于本轮新测的 genanki。','',f'测量源码为 `{current["source_commit"]}`（当前发布性能实现）；工作区已有无关未提交文件，完整状态与差异已冻结。这是一轮完整会话，没有选择性重跑、替换或删去慢样本。','',f'测量时间：{beg:%Y-%m-%d %H:%M:%S}–{end:%H:%M:%S} Asia/Taipei。同一台 M1 Pro / 32 GiB / macOS 27，System allocator、默认产品 features、Rust 1.92.0、CPython 3.11.0、genanki 0.13.1。每实现每格 10 次计时、5 次独立 RSS，两阶段各 3 次预热，Rust/genanki 相邻交错。','', '## 1,000 条结果','',header,table(r for r in rows if r['notes']==1000),'','负百分比表示更快；RSS 为独立进程峰值。耗时包含启动、输入读取、转换、导出、检查、写入和退出。','', '![1,000 条耗时与独立 RSS](comparison.png)','', '## 全部 20 格','',header,table(rows),'','## 验证与解释范围','','- 20 份 JSON 输入及 2,749 份媒体与 9/21 的记录一致；输入、源码、依赖及可执行工具在测量前后不变。','- 840 次导出的原始 SQLite、字段、媒体与语义检查全部通过；40 个选定包实际通过 Anki 导入、内容和渲染验证。全部导出前后均为 AC Power。','- 采集前 46 项 benchmark 测试通过，并完成 200 条 smoke。所有构建、测试和 smoke 在正式采集前结束。','- 旧版 Deck API 与当前 Project API 的默认工作范围不同；不同日期的后台负载、页缓存、温度与 CPU 状态未隔离。这些差值是跨会话观察，不能全归因于新增优化，也不使用 genanki 比例校正历史数字。','- 本轮只测普通 Rust build，不覆盖 Node 提交/COW、compare 后复用候选或重复大字节媒体导入的专门收益；也不解除上一轮宽字段 RSS 与跨平台 CI 的验收缺口。','- 图中耗时误差线是 Q1–Q3，非置信区间；全部分布、包大小及新测 genanki RSS 见 [summary.json](summary.json) 和 [comparison.csv](comparison.csv)。历史差值见 [delta.csv](delta.csv)。','','## 可复查证据','','[README](README.md) 列出固定协议、源码与输入身份、逐次采集记录、校验和及离线重放方式。硬件 JSON 沿用同机记录；本次负载、电源及可获取的温度信息独立保存，不把旧硬件记录当作新探测。实际 APKG 在验证并记录哈希后删除；离线重放只复核保留记录，不重新导出或导入 Anki。','']
(W/'report.md').write_text('\n'.join(lines))
print(json.dumps({'faster_than_sep21':faster,'slower_than_sep21':slower,'faster_than_fresh_genanki':genanki,'large_cells':[r for r in rows if r['notes']==1000]},ensure_ascii=False,indent=2))
