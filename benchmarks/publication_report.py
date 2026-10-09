#!/usr/bin/env python3
"""Render a report from complete saved batches; never runs measured processes."""
import argparse
import json
import statistics
from pathlib import Path


def load(root, name):
    return json.loads((root / name / 'summary.json').read_text())


def percent(cell, before='baseline', after='candidate', metric='elapsed_ms'):
    a, b = (cell['cells'][mode][metric]['median'] for mode in [before, after])
    return a, b, 100 * (a - b) / a


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('evidence', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    root = args.evidence
    core = [load(root, f'core-batch{i}') for i in [1, 2]]
    node = [load(root, f'node-batch{i}') for i in [1, 2]]
    lines = ['# 四项发布性能实现：验收报告', '',
        '**状态：功能实现与本机验证已完成；宽字段 RSS 回归与尚未完成的跨平台 CI 仍阻塞原 spec 验收。**', '',
        '测量平台为 macOS 27 arm64、Apple M1 Pro（10 核、32 GiB），Rust 1.92.0、Node 24.19.0、CPython 3.11.0，system allocator。'
        '电源、负载、时间与可获取的温度状态见原始 plan；桌面负载、温度和页缓存没有完全隔离。'
        '所有正式批次串行运行，没有并发构建或测试。数字只描述本机测量。', '',
        '## 来源与边界', '',
        '实施基线为 `7c4c8ed`，它冻结了调用时已经存在的结构优化。原 HEAD `eb4b463` 加工作区差异才是完整起点，'
        '不能把结构优化再次计入四项收益。四项实现分别为 `a2bf805`、`9b152ea`、`a175697`、`3a9e5da`。'
        '各 isolated core 变体只叠加对应改动；combined 同时使用最终 core 改动和 Bundle 2.1.0。', '',
        'Node COW 的两侧使用相同 P1 core，以便同时测量新增 preparePublication；一侧深复制，另一侧共享不可变快照。'
        '保存的 Node 构建副本早于等价的结果/错误转换 helper 整理；源码副本和二进制哈希均保留。'
        '最终交付 SDK 另经安装包与功能测试。Python 流程两侧使用同一最终原生 SDK，仅切换 compare+build 或 prepare+publish。', '',
        '每格每种模式 2 次预热、10 次 AB/BA 交错计时、3 次独立 RSS 进程；全部有效及失败记录保留。'
        'core elapsed 是 collector spawn→exit，input/operation 为内层跨度。Node 单独报告提交、完成、修改、定时器与 RSS。'
        '固定 seed 20261003 的配对重采样范围只描述这批数据，不是跨机器保证。', '',
        '原始 Node 两批的 plan 未冻结 baseline APKG/collector/inspector/oracle 哈希，不能事后补称已冻结；'
        '后续 Node operation-only 与 SDK 控制批次补齐这些输入。原两批完整报告已离线核对一致，只排除 duration_ms。'
        'collector 的回收/中断/残留子进程字段也通过原始记录离线审计；失败 smoke 不进入正式结论。', '',
        '## P1–P4 计时目标', '',
        '| 目标 | 第一批 baseline → candidate ms | 降低 | 第二批 baseline → candidate ms | 降低 | 门槛 |',
        '|---|---:|---:|---:|---:|---|']
    targets = [('P1 10k 文本', 'reuse/text10k', 30), ('P1 1k 宽字段', 'reuse/wide1k', 30),
               ('P3 1k 宽字段', 'manifest/wide1k', 5), ('P4 8 MiB × 8', 'bytes/bytes8m', 10)]
    for label, key, threshold in targets:
        values = [percent(batch[key]) for batch in core]
        assert all(v[2] >= threshold and batch[key]['median_paired_saving_ms'] > 0 for v, batch in zip(values, core))
        a, b = values
        lines.append(f'| {label} | {a[0]:.3f} → {a[1]:.3f} | {a[2]:.2f}% | {b[0]:.3f} → {b[1]:.3f} | {b[2]:.2f}% | ≥{threshold}%：通过 |')
    a, b = [percent(batch['build/text10k'], 'before', 'cow', 'submit_ms') for batch in node]
    assert a[2] >= 80 and b[2] >= 80
    lines.append(f'| P2 Node 10k build 提交 | {a[0]:.3f} → {a[1]:.3f} | {a[2]:.2f}% | {b[0]:.3f} → {b[1]:.3f} | {b[2]:.2f}% | ≥80%：通过 |')
    lines += ['', '计时目标通过不抵消 RSS 回归。Node 的提交提速不等于完整操作或最坏事件循环延迟同幅改善。', '',
        '## Core：隔离与组合完整结果', '',
        '每格列出 spawn→exit 中位数；输入准备、操作跨度、四分位数、全部样本、逐对差值和重采样范围均保存在对应 summary.json。'
        '“复查”按总耗时回退 >5% 或 RSS 增加 >max(10%, 8 MiB) 标记，二批保留第一批并构成完整确认。', '',
        '| 场景 | 第一批 ms / RSS MiB：baseline → candidate | 第二批 ms / RSS MiB：baseline → candidate | 复查批次 |',
        '|---|---|---|---|']
    for key in core[0]:
        values = []
        flags = []
        for index, batch in enumerate(core, 1):
            a, b, gain = percent(batch[key])
            ra, rb = [batch[key]['cells'][m]['rss_mib']['median'] for m in ['baseline', 'candidate']]
            values.append(f'{a:.3f} → {b:.3f} / {ra:.2f} → {rb:.2f}')
            if batch[key]['requires_confirmation']: flags.append(str(index))
        lines.append(f'| {key} | {values[0]} | {values[1]} | {", ".join(flags) or "—"} |')
    lines += ['', 'P1 准备后持有的私有候选大小（timing 样本中位数；生命周期测试验证 owner 不额外持有作者快照或构建 workspace）：', '', '| 场景 | 第一批 MiB | 第二批 MiB |', '|---|---:|---:|']
    retained = [[json.loads(line) for line in (root / f'core-batch{i}' / 'samples.jsonl').read_text().splitlines()] for i in [1, 2]]
    for profile in ['text1k', 'image1k', 'wide1k', 'text10k']:
        sizes = [statistics.median(r['retained_candidate_bytes'] for r in rows if r['group'] == 'reuse' and r['profile'] == profile and r['mode'] == 'candidate' and r['role'] == 'timing') / 2**20 for rows in retained]
        lines.append(f'| {profile} | {sizes[0]:.3f} | {sizes[1]:.3f} |')
    lines += ['', 'P4 分段（input 包含读取/复制/MIME/散列/快照等准备成本，不是纯缓存函数耗时）：', '', '| 场景/批次 | 输入 ms：baseline → candidate | 导出操作 ms：baseline → candidate |', '|---|---:|---:|']
    for index, batch in enumerate(core, 1):
        for key in ['bytes/bytes2m', 'bytes/bytes8m', 'bytes-single/bytes2m', 'bytes-single/bytes8m', 'bytes-single/bytes-small']:
            a, b = [batch[key]['cells'][m] for m in ['baseline', 'candidate']]
            lines.append(f"| {key}/{index} | {a['input_ms']['median']:.3f} → {b['input_ms']['median']:.3f} | {a['operation_ms']['median']:.3f} → {b['operation_ms']['median']:.3f} |")
    lines += ['', '## Node COW 的成本与控制', '',
        '原 Node 批次刻意让 task 和 clone 存活，并在任务完成前执行修改，因此 complete/process 时间包含首次 COW 编辑。'
        '后续 node-operations 批次移除 clone 和这些编辑，单独检查普通操作。Rust 宽字段是正反面各约 8 KiB 确定性文本；'
        'Node 宽字段是 16 KiB 答案，这两类输入不能当作跨语言同一工作负载。', '',
        '| 操作/场景/批次 | 提交 ms：before → COW | 首次共享编辑 ms：before → COW | 每轮定时器最大延迟中的最大值 ms：before → COW |',
        '|---|---|---|---|']
    for index, batch in enumerate(node, 1):
        for key, value in batch.items():
            a, b = [value['cells'][m] for m in ['before', 'cow']]
            lines.append(f"| {key}/{index} | {a['submit_ms']['median']:.3f} → {b['submit_ms']['median']:.3f} | {a['first_shared_edit_ms']['median']:.3f} → {b['first_shared_edit_ms']['median']:.3f} | {max(a['timer_max_ms']['samples']):.3f} → {max(b['timer_max_ms']['samples']):.3f} |")
    lines += ['', 'clone、无快照修改、后续修改、失败 add 和全部 timer 样本保存在原始记录及 summary。'
        '这里的 timer 极值不是总体 p99；三个 RSS 高水位样本也不能证明稳定堆内存减少。', '',
        '## 补充控制与标准矩阵', '',
        '完整 staging 控制的 operation_ms 只覆盖 writer 调用；该子进程随后执行 manifest 解析/内容断言、APKG 复制与清理，'
        '因此 spawn→exit 和 RSS **包含这部分控制验证工作**。原始通用 plan 中“验证在计时外”的表述不适用于这一格；'
        '它仍验证真实 staging 引用、规范化内容和指纹，并保留此测量边界差异。外部 APKG verifier 与 Anki 导入仍在 collector 外。', '',
        '| 批次 | 场景 | baseline → candidate ms | 降低 | RSS MiB：baseline → candidate | 复查 |',
        '|---|---|---:|---:|---:|---|']
    for kind in ['controls-native', 'controls-node', 'controls-python', 'standard']:
        for index in [1, 2]:
            for key, value in load(root, f'{kind}-batch{index}').items():
                a, b, gain = percent(value)
                ra, rb = [value['cells'][m]['rss_mib']['median'] for m in ['baseline', 'candidate']]
                lines.append(f'| {kind}/{index} | {key} | {a:.3f} → {b:.3f} | {gain:.2f}% | {ra:.2f} → {rb:.2f} | {"是" if value["requires_confirmation"] else "—"} |')
    lines += ['', 'node-operations 的完整提交、完成、总耗时、RSS 与配对结果见其 summary.json；该批次不混入 COW 首次修改。', '',
        '## 验证与未关闭的门槛', '',
        '- Rust workspace：708 passed、0 failed、23 ignored；Node：29 passed、1 skipped；Python：67 passed、1 skipped。'
        '定向生命周期、路径/别名、策略阻止、fork、并发、缓存命中预算/MIME 和真实内容回归通过。',
        '- Rust packaged consumer、安装后的 Node ESM/CJS 与全部 README 示例、TypeScript 消费者、Python wheel 与正负类型检查通过。'
        'mypy、clippy、格式、contract governance、payload/bundle 检查通过。',
        '- 独立真实 Anki roundtrip 28 场景全部 verified；基准导出另逐个进行实际内容校验，并对计划内样本运行 Anki oracle。',
        '- 宽字段 RSS 的确认回归是开放验收项。隔离探针在字段渲染边界就观察到分歧；Rust allocator 包装探针的存活分配峰值'
        '为 baseline 40,171,247 bytes、combined 40,171,016 bytes，结束存活量相同。该探针不统计 C 库分配，且会改变布局，'
        '不能代替正式 RSS 或证明所有平台无回归。原始探针日志和失败的 time 辅助调用保留。',
        '- 未运行其他 Rust Tier 1、Node 支持 runtime 和 Python 平台的 CI；本机通过不能代替跨平台验收及 release gates。',
        '- 没有公开发布包或部署网站；依据原 spec，在 RSS 取舍得到明确解决前，本分支只作为待审候选，不能宣布完成验收。', '',
        '## 数据使用', '',
        '每份 completed marker 表示该批测量/内容验证完成，不表示性能或发布获准。原始计划、失败记录、完整比较报告、源码/二进制/fixture 哈希、'
        '构建命令、所有 timing/RSS 样本及验证结果进入归档。APKG 的时间戳、持续时间和私有路径等非语义差异不要求跨进程逐字节一致；'
        '比较报告保留完整证据，不以删减诊断或降低检查预算获取成绩。离线复算与重新导出/Anki 导入是不同验证。']
    args.output.write_text('\n'.join(lines) + '\n')


if __name__ == '__main__':
    main()
