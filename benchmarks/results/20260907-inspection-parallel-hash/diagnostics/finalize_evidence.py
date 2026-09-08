"""Link all accepted implementation outputs to the verified baseline bytes."""
from pathlib import Path
import hashlib
import json
import shutil

WORK = Path(__file__).resolve().parent
ROOT = WORK.parents[2]
OUT = ROOT / 'benchmarks/results/20260907-inspection-parallel-hash'
OLD = ROOT / 'benchmarks/results/20260907-readme-comparison'

def rows(path):
    return [json.loads(line) for line in path.read_text().splitlines()]

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

known = {}
for index in (1, 2, 3):
    for row in rows(OLD / f'round-{index}/verified.jsonl'):
        if row['adapter'] == 'rust':
            known.setdefault((row['profile'], row['size']), set()).add(row['artifact_sha256'])
assert len(known) == 20 and all(len(value) == 1 for value in known.values())
checks = []
for index in (1, 2, 3):
    checked = [r for r in rows(OUT / f'round-{index}/verified.jsonl') if r['adapter'] == 'rust']
    assert all(r['artifact_sha256'] in known[r['profile'], r['size']] for r in checked)
    checks.append({'run': f'formal-round-{index}', 'byte_identical_rust_exports': len(checked)})
paired = rows(WORK / 'paired-final/attempts.jsonl')
assert len(paired) == 520
assert all(r['artifact_sha256'] in known[r['profile'], r['size']] for r in paired)
checks.append({'run': 'paired-final', 'byte_identical_exports': len(paired)})
diag = OUT / 'diagnostics'
diag.mkdir(exist_ok=False)
for name in ['before-source-sha256.json', 'before-build-records.json', 'after-build-record.json',
             'compare.py', 'instrument.py', 'hash-probe.log']:
    shutil.copy2(WORK / name, diag / name)
for name in ['profile-before', 'profile-detail', 'buffered-index', 'normalization', 'small-frame', 'parallel-hash', 'paired-final']:
    (diag / name).mkdir()
    for filename in ['plan.json', 'attempts.jsonl', 'summary.json']:
        shutil.copy2(WORK / name / filename, diag / name / filename)
for name in ['tests-targeted.log', 'tests-workspace.log', 'tests-benchmark.log', 'tests-doc.log',
             'clippy.log', 'doc.log', 'contracts.log', 'payload.log', 'release-metadata.log', 'dependency-policy.log']:
    shutil.copy2(WORK / name, diag / name)
(diag / 'artifact-equivalence.json').write_text(json.dumps({
    'baseline': str(OLD.relative_to(ROOT)),
    'baseline_records_sha256': {f'round-{i}/verified.jsonl': digest(OLD / f'round-{i}/verified.jsonl') for i in (1, 2, 3)},
    'method': 'Exact SHA256 equality with all corresponding previously verified and Anki-imported Rust artifacts, including every warmup.',
    'checks': checks,
    'cells': [{'profile': p, 'size': n, 'artifact_sha256': next(iter(h))} for (p, n), h in sorted(known.items())],
}, indent=2) + '\n')
shutil.copy2(__file__, diag / 'finalize_evidence.py')
summary = json.loads((WORK / 'paired-final/summary.json').read_text())
labels = {cell['profile']: cell['label'] for cell in json.loads((OUT / 'summary.json').read_text())['cells']}
table = ['| 场景 | 笔记数 | 旧版 ms | 新版 ms | 耗时减少 |', '| --- | ---: | ---: | ---: | ---: |']
for row in summary:
    old = row['variants']['before']['median_ms']
    new = row['variants']['after']['median_ms']
    table.append(f"| {labels[row['profile']]} | {row['size']} | {old:.2f} | {new:.2f} | {100*(1-new/old):.2f}% |")
text = (WORK / 'implementation-draft.md').read_text()
text = text.replace('正式统计及最终版本的前后交错对照结果在确认完成后补充。原三轮 README 基准证据保留为历史对照。', '''三轮正式确认全部通过：2520 次导出、120 次 Anki 导入/内容/渲染检查。三轮源码和二进制身份相同，无任何单元触发预先约定的 5 个百分点波动标记。

## 最终版本的前后交错对照

另用冻结旧版与最终版二进制在同场交错运行全部 20 格，每格每版 3 次预热、10 次计时，共 520 次导出。下表是该独立对照的中位数，不与 README 的三轮 genanki 对比混合。运行时段不同，不能把早先三轮的绝对耗时直接相减当作代码收益。这里不对 RSS 作前后结论；正式三轮单独测量了 RSS。

''' + '\n'.join(table) + '''

1000 条独立图片、音频、混合独立媒体的耗时分别减少 8.4%、6.4%、7.9%。纯文本为 60.12 → 60.40 ms（−0.46%），共享媒体为 72.73 → 71.76 ms（1.34%），未显示同等程度收益。结果为本机描述性测量，不代表其他机器、大型单文件或其他笔记类型。

## 证据与复现

- [当前三轮正式对比及图表](report.md)：1000 条时相对 genanki 的耗时减少为纯文本 44.2%、图片 13.3%、音频 17.6%、混合独立媒体 19.6%、混合共享媒体 38.1%。这些百分比不是相对旧版的提升。
- [前后交错原始记录](diagnostics/paired-final/attempts.jsonl)、[计划与二进制哈希](diagnostics/paired-final/plan.json)、[逐格摘要](diagnostics/paired-final/summary.json)。
- [字节一致性证明](diagnostics/artifact-equivalence.json)：新的三轮中 1260 次 Rust 导出及前后交错的全部 520 次导出，均与相应旧版已验证产物的 SHA256 完全相同。包大小未改变。
- [旧版三轮证据](../20260907-readme-comparison/report.md)独立保留；[源码补丁](source.patch)与[冻结源码哈希](source-snapshot.json)明确标识未提交版本。
- `diagnostics/` 保存所有探索性计时、未采用方案、计时器和测试日志。诊断脚本按原 workspace 的 `.work/scale-optimization-20260907` 布局运行；冻结二进制和媒体文件不包含在此紧凑归档中。正式基准的可复现命令见[基准说明](../../README.md)。
''')
(OUT / 'implementation.md').write_text(text)
# Pin new raw evidence without including generated reports/charts or this digest file.
evidence = json.loads((OUT / 'evidence-sha256.json').read_text())
for path in diag.rglob('*'):
    if path.is_file():
        evidence[str(path.relative_to(OUT))] = digest(path)
(OUT / 'evidence-sha256.json').write_text(json.dumps(evidence, indent=2) + '\n')
files = sorted(p for p in OUT.rglob('*') if p.is_file() and p.name != 'SHA256SUMS')
(OUT / 'SHA256SUMS').write_text(''.join(f'{digest(p)}  {p.relative_to(OUT)}\n' for p in files))
print(json.dumps(checks))
