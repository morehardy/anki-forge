"""Seal reviewable evidence; never copy APKGs, temporary output trees or compiled binaries."""
from pathlib import Path
import difflib,hashlib,io,json,shutil,subprocess,tarfile,tempfile
W=Path(__file__).resolve().parent; R=W.parents[2]; A=R/'benchmarks/results/20260907-text-ownership'
A.mkdir(exist_ok=False)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def save(p,d): p.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n')
for p in W.iterdir():
    if p.is_file() and p.suffix in ('.py','.json','.log','.svg','.png'):
        shutil.copy2(p,A/p.name)
for p in W.glob('*/attempts.jsonl'):
    folder=A/'runs'/p.parent.name; folder.mkdir(parents=True)
    for q in p.parent.rglob('*'):
        if q.is_file() and q.suffix in ('.json','.jsonl','.log') and q.stat().st_size:
            dest=folder/q.relative_to(p.parent);dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(q,dest)
shutil.copytree(W/'final-verification',A/'verification')
shutil.copy2(R/'benchmarks/.tools/build-records.json',A/'final-build-records.json')
source_sets={
 'baseline':W/'baseline-source','early-release':W/'early-release-source',
 'move-fields-interrupted':W/'move-fields-source','accepted':W/'accepted-full-source',
 **{p.name:p for p in (W/'probe-sources').iterdir()}}
objects={}; manifests={}
for label,directory in source_sets.items():
    manifest={}
    for p in sorted(directory.rglob('*')):
        if p.is_file():
            h=sha(p);manifest[str(p.relative_to(directory))]=h;objects.setdefault(h,p)
    manifests[label]=manifest
# The reused baseline allocation probe differs only in these two adapter files.
manifests['current-memory']=dict(manifests['baseline'])
old=R/'benchmarks/.work/bounded-media-20260907'
for n,p in [('benchmarks/adapters/rust/src/main.rs',old/'memory-main.rs'),('benchmarks/adapters/rust/src/memory.rs',old/'profile-source/benchmarks/adapters/rust/src/memory.rs')]:
    h=sha(p);manifests['current-memory'][n]=h;objects.setdefault(h,p)
# Validate every measured candidate against its build-time source hashes.
for label,record in [('baseline','baseline-source-sha256.json'),('early-release','build-early-release.json'),('move-fields-interrupted','build-move-fields.json'),('current-memory','build-current-memory.json'),('current-stages','build-current-stages.json'),('accepted-memory','build-accepted-memory.json'),('accepted-stages','build-accepted-stages.json'),('accepted','accepted-source-sha256.json')]:
    expected=json.loads((W/record).read_text());expected=expected.get('source_sha256',expected)
    assert all(manifests[label].get(k)==v for k,v in expected.items()),label
with tarfile.open(A/'source-content.tar.gz','w:gz') as tar:
    for h,p in sorted(objects.items()):
        data=p.read_bytes();info=tarfile.TarInfo('sha256/'+h);info.size=len(data);info.mtime=0;info.mode=0o644;tar.addfile(info,io.BytesIO(data))
save(A/'source-content.json',{'layout':'source-content.tar.gz contains sha256/<digest>; reconstruct relative paths using these sets. Build commands are in build-*.json; private probes include their workspace metadata.','sets':manifests})
before=json.loads((W/'baseline-source-sha256.json').read_text());after=json.loads((W/'accepted-source-sha256.json').read_text())
patch='';changed=[]
for name,h in before.items():
    assert sha(R/name)==after[name]
    if h!=after[name]:
        changed.append(name)
        patch+=''.join(difflib.unified_diff((W/'baseline-source'/name).read_text().splitlines(True),(R/name).read_text().splitlines(True),fromfile='a/'+name,tofile='b/'+name))
(A/'source.patch').write_text(patch)
with tempfile.TemporaryDirectory(prefix='anki-text-replay-') as t:
    root=Path(t);shutil.copytree(W/'baseline-source',root,dirs_exist_ok=True)
    subprocess.run(['git','apply','--check',str(A/'source.patch')],cwd=root,check=True)
    subprocess.run(['git','apply',str(A/'source.patch')],cwd=root,check=True)
    assert all(sha(root/n)==h for n,h in after.items())
old_seals=json.loads((W/'previous-archive-seals.json').read_text()); old_counts={}
for name,h in old_seals.items():
    p=R/name;assert sha(p)==h,name;count=0
    for line in p.read_text().splitlines():
        digest,relative=line.split('  ',1); assert sha(p.parent/relative)==digest,(name,relative);count+=1
    old_counts[name]=count
save(A/'source-audit.json',{'base_revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=R,text=True).strip(),'changed_runtime_or_test_files':changed,'source_patch_replay_passed':True,'accepted_core_source_files_unchanged':len(after),'source_objects':len(objects),'all_measured_candidate_sources_match_build_records':True,'previous_archives_verified_unchanged':old_counts,'main_adapter_matches_accepted':sha(R/'benchmarks/adapters/rust/target/release/anki-forge-benchmark')==json.loads((W/'build-accepted.json').read_text())['binary_sha256']})
s=json.loads((W/'summary.json').read_text()); cases={c['name']:c for c in s['cases']}
lines=['# 文本所有权优化：减少临时 Project 的字段副本','','本轮主要降低长字段内存，整体耗时收益较小。Deck/Package 导出创建的临时 Project 现在会被消费：先解析身份，再移动 HTML 字符串，并在归一化后续阶段开始前释放临时笔记和身份缓存。公开 Deck、Project 接口仍可重复编辑和导出。','','![同输入下的峰值内存与耗时](text-ownership.svg)','','## 同一会话的完整对照','','Apple M1 Pro / 32 GiB / macOS ARM64，Rust 1.92.0 release、默认 features、system allocator，2026-09-07，**AC Power**。对照为上一版共享媒体缓冲池与本次文本所有权改动。固定 29 个输入，每版本每场景 2 次预热 + 7 次计时；另跑 2 次预热 + 5 次 RSS。顺序交替、进程独立，文件缓存不受控；没有同时编译、测试或运行 Anki 导入。所有样本均保留，四分位数使用线性插值。','','| 场景 | 时间 ms，前 → 后 | 时间变化 | 峰值 RSS MiB，前 → 后 | RSS 变化 |','| --- | ---: | ---: | ---: | ---: |']
for c in s['cases']:
    b,a=c['variants']['current'],c['variants']['accepted'];lines.append(f"| {c['name']} | {b['time_ms']['median']:.3f} → {a['time_ms']['median']:.3f} | {c['time_change_pct']:+.2f}% | {b['rss_mib']['median']:.2f} → {a['rss_mib']['median']:.2f} | {c['rss_change_pct']:+.2f}% |")
lines+=['','时间变化为负表示更快，为正表示更慢。普通长字段本轮为 +1.30%，共享媒体部分小场景为 +1.68%～+2.57%；不把微小变化当作普遍提速。29 个场景的 RSS 中位数均降低。完整分布见 [summary.json](summary.json)，原始过程见 [runs](runs)。RSS 使用独立测量阶段；该阶段的耗时没有混入计时结果。','','## 实际存活分配与阶段耗时','','私有分配探针把每次分配转发给 System，只记录计数；它的进程 RSS 和整体耗时不用于上表。每版本每场景 1 次预热 + 3 次采样。峰值统计包含原始 Deck；累计分配量只取导出前后之差。','','| 场景 | Rust 活跃堆峰值 MiB，前 → 后 | 导出累计分配 MiB，前 → 后 | 导出分配次数，前 → 后 |','| --- | ---: | ---: | ---: |']
for c in s['allocations']:
    b,a=c['variants']['current-memory'],c['variants']['accepted-memory'];lines.append(f"| {c['case']} | {b['peak_live_mib']['median']:.2f} → {a['peak_live_mib']['median']:.2f} | {b['export_allocated_mib']['median']:.2f} → {a['export_allocated_mib']['median']:.2f} | {b['export_allocation_calls']['median']:,.0f} → {a['export_allocation_calls']['median']:,.0f} |")
lines+=['','四倍长字段累计分配减少约 26.82 MiB，符合消除一份完整字段副本的预期。Rust 活跃堆和进程 RSS 是不同口径；分配器释放对象后未必马上归还页面。','','阶段探针每版本 2 次预热 + 5 次采样。由于身份提取从 reconcile 移到了准备阶段，公平比较范围是 **归一化 + 身份提取**：']
for c in s['stages']:
    b=c['variants']['current-stages']['normalization_and_identity_ms']['median'];a=c['variants']['accepted-stages']['normalization_and_identity_ms']['median'];lines.append(f"- {c['case']}：{b:.3f} → {a:.3f} ms（{100*(a/b-1):+.2f}%）。")
lines+=['','先前约 101 ms 的归一化数值来自另一次电源/测量会话，不能直接减去这里的新数值。本次万条旧版单独归一化为 84.008 ms，另有约 7.187 ms 的身份提取；上面的合计先逐样本相加，再取中位数。各阶段和探针源码均保留。','','## 正确性与边界','','- 正式前后对照共 **928 次导出**，每个场景两个版本的 APKG 全部逐字节一致。','- 58 份选定产物分别通过独立 ZIP、SQLite 行、身份和媒体核验；29 个场景均通过固定 upstream Anki 的导入、字段、媒体与渲染核验。','- 所有 1,175 次已产出文件的尝试，包括诊断探针和中断批次，均绑定到对应场景已经独立核验的产物哈希，见 [核验绑定](verification/attempt-bindings.json)。','- 完整工作区 **1,015 项 Rust 测试通过、25 项原有忽略**；43 项基准工具测试通过。Clippy `-D warnings`、格式、文档、打包、契约、版本及依赖检查通过。','- 新增测试覆盖长 HTML 的存储转移、嵌套内容渲染、精确 APKG/锁文件一致、重复导出和媒体修复后重试；扩展了无效部分计划、身份冲突及来源映射的差分检查。','','## 保留的试验与来源','','`baseline-reproduction` 复现初始分配；`early-release-screen` 只提前释放临时 Project，没有稳定的耗时收益。`move-fields-screen` 在第 57 次导出遇到原生电源来源切换而停止，**整批排除于性能比较**。其原始日志、来源和逐字节产物核验保留。之后修复 Clippy 报告的内部枚举体积问题，并从预热重新开始最终完整对照，没有续接或筛选旧样本。初次检查失败日志也保留。','','- 基线二进制：`4abdcea28de9157d255ef80969be573631a4b38a1ce4483d2075a45877b5a75e`。','- 采用二进制：`4a3b8857ee9c1477f63990cb511fb221c71f18c32753bdd41f367d03c38ba4af`。','- [源码差异](source.patch) 可应用于 [上一版快照](../20260907-bounded-media/implementation.md)；已重放并验证全部 199 个 crate/adapter 文件的哈希。','- [源码内容包](source-content.tar.gz) 通过 [路径映射](source-content.json) 保存基线、所有测量候选和探针的真实文件内容；全部与构建时哈希相符。私有探针每次强制刷新源码 mtime，构建日志确认编译，随后恢复正式 adapter。','- [输入列表](cases.json)、[正式构建身份](accepted-build.json)、[源码复核](source-audit.json)、[测试结果](quality-summary.json) 和原始命令均保留。','- Anki checker 保留已记录的 `tokio/io-util` 构建 feature 补丁。旧档案全部按各自 SHA256SUMS 重新核验，内容未改动。','','README 中旧的 Rust/genanki 三会话矩阵仍属于其原始冻结版本；这里是两个 Rust 版本的对照，电源条件也不同，不叠加百分比或把旧 genanki 耗时与新 Rust 耗时交叉比较。','','## 复现','','在新的 `benchmarks/.work/<name>/` 目录准备 `cases.json` 指向相同哈希的冻结输入，并恢复 `binaries/current`、`binaries/accepted` 后，执行归档的 `final_compare.py`。它调用 `run.py` 写入计划、每次导出前后检查原生电源、交替顺序并验证哈希。使用同一固定 inspector/oracle 执行 `verify_final.py`；不要把编译、导入或绘图与计时并行。`setup_probe.py`、`build_probe.py` 与对应源码集合记录了私有探针构建方式。','','离线重新绘图（不重测）：','','```sh','benchmarks/.venv/bin/python benchmarks/results/20260907-text-ownership/render_chart.py benchmarks/results/20260907-text-ownership','```','']
(A/'README.md').write_text('\n'.join(lines))
print('Archived',len(objects),'source objects and',len(s['cases']),'cases at',A)
