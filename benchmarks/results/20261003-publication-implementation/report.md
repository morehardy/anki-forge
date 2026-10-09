# 四项发布性能实现：验收报告

**状态：功能实现与本机验证已完成；宽字段 RSS 回归与尚未完成的跨平台 CI 仍阻塞原 spec 验收。**

测量平台为 macOS 27 arm64、Apple M1 Pro（10 核、32 GiB），Rust 1.92.0、Node 24.19.0、CPython 3.11.0，system allocator。电源、负载、时间与可获取的温度状态见原始 plan；桌面负载、温度和页缓存没有完全隔离。所有正式批次串行运行，没有并发构建或测试。数字只描述本机测量。

## 来源与边界

实施基线为 `7c4c8ed`，它冻结了调用时已经存在的结构优化。原 HEAD `eb4b463` 加工作区差异才是完整起点，不能把结构优化再次计入四项收益。四项实现分别为 `a2bf805`、`9b152ea`、`a175697`、`3a9e5da`。各 isolated core 变体只叠加对应改动；combined 同时使用最终 core 改动和 Bundle 2.1.0。

Node COW 的两侧使用相同 P1 core，以便同时测量新增 preparePublication；一侧深复制，另一侧共享不可变快照。保存的 Node 构建副本早于等价的结果/错误转换 helper 整理；源码副本和二进制哈希均保留。最终交付 SDK 另经安装包与功能测试。Python 流程两侧使用同一最终原生 SDK，仅切换 compare+build 或 prepare+publish。

每格每种模式 2 次预热、10 次 AB/BA 交错计时、3 次独立 RSS 进程；全部有效及失败记录保留。core elapsed 是 collector spawn→exit，input/operation 为内层跨度。Node 单独报告提交、完成、修改、定时器与 RSS。固定 seed 20261003 的配对重采样范围只描述这批数据，不是跨机器保证。

原始 Node 两批的 plan 未冻结 baseline APKG/collector/inspector/oracle 哈希，不能事后补称已冻结；后续 Node operation-only 与 SDK 控制批次补齐这些输入。原两批完整报告已离线核对一致，只排除 duration_ms。collector 的回收/中断/残留子进程字段也通过原始记录离线审计；失败 smoke 不进入正式结论。

## P1–P4 计时目标

| 目标 | 第一批 baseline → candidate ms | 降低 | 第二批 baseline → candidate ms | 降低 | 门槛 |
|---|---:|---:|---:|---:|---|
| P1 10k 文本 | 909.610 → 475.358 | 47.74% | 897.478 → 472.772 | 47.32% | ≥30%：通过 |
| P1 1k 宽字段 | 857.453 → 458.822 | 46.49% | 858.869 → 462.985 | 46.09% | ≥30%：通过 |
| P3 1k 宽字段 | 376.179 → 344.393 | 8.45% | 376.541 → 354.584 | 5.83% | ≥5%：通过 |
| P4 8 MiB × 8 | 115.671 → 98.755 | 14.62% | 117.454 → 97.099 | 17.33% | ≥10%：通过 |
| P2 Node 10k build 提交 | 4.160 → 0.312 | 92.51% | 3.991 → 0.306 | 92.33% | ≥80%：通过 |

计时目标通过不抵消 RSS 回归。Node 的提交提速不等于完整操作或最坏事件循环延迟同幅改善。

## Core：隔离与组合完整结果

每格列出 spawn→exit 中位数；输入准备、操作跨度、四分位数、全部样本、逐对差值和重采样范围均保存在对应 summary.json。“复查”按总耗时回退 >5% 或 RSS 增加 >max(10%, 8 MiB) 标记，二批保留第一批并构成完整确认。

| 场景 | 第一批 ms / RSS MiB：baseline → candidate | 第二批 ms / RSS MiB：baseline → candidate | 复查批次 |
|---|---|---|---|
| bytes/bytes2m | 46.255 → 42.183 / 16.41 → 18.36 | 46.562 → 40.689 / 16.47 → 16.84 | — |
| bytes/bytes8m | 115.671 → 98.755 / 27.27 → 27.23 | 117.454 → 97.099 / 27.09 → 27.28 | — |
| bytes-single/bytes-small | 24.447 → 25.405 / 12.83 → 12.89 | 24.185 → 24.676 / 12.77 → 12.91 | — |
| bytes-single/bytes2m | 32.749 → 31.964 / 16.56 → 16.62 | 31.225 → 31.285 / 16.41 → 16.72 | — |
| bytes-single/bytes8m | 57.410 → 57.766 / 27.12 → 27.27 | 59.370 → 59.859 / 27.11 → 27.50 | — |
| combined/audio1k | 187.887 → 187.472 / 38.27 → 37.59 | 187.024 → 182.690 / 37.98 → 37.66 | — |
| combined/image1k | 224.743 → 220.743 / 36.42 → 35.83 | 225.267 → 223.510 / 36.48 → 35.95 | — |
| combined/shared1k | 69.461 → 66.202 / 32.70 → 32.42 | 65.263 → 62.828 / 32.70 → 32.34 | — |
| combined/text10k | 355.883 → 344.396 / 121.50 → 119.33 | 352.067 → 337.591 / 121.12 → 118.58 | — |
| combined/text1k | 54.037 → 52.281 / 25.23 → 24.86 | 54.935 → 52.210 / 24.97 → 25.12 | — |
| combined/wide1k | 370.935 → 349.757 / 119.03 → 139.38 | 386.498 → 345.726 / 119.25 → 139.53 | 1, 2 |
| combined-bytes/bytes2m | 47.219 → 41.763 / 18.16 → 18.56 | 45.765 → 41.523 / 17.30 → 16.50 | — |
| combined-bytes/bytes8m | 116.419 → 96.631 / 27.20 → 27.20 | 116.528 → 98.609 / 27.06 → 27.16 | — |
| combined-reuse/image1k | 412.524 → 263.380 / 40.95 → 39.12 | 410.075 → 259.592 / 40.27 → 39.44 | — |
| combined-reuse/text10k | 891.066 → 456.096 / 135.00 → 131.83 | 901.381 → 457.119 / 134.44 → 130.23 | — |
| combined-reuse/text1k | 115.947 → 66.870 / 28.42 → 27.81 | 117.166 → 66.109 / 28.61 → 27.28 | — |
| combined-reuse/wide1k | 860.379 → 426.869 / 123.12 → 143.08 | 861.858 → 424.871 / 123.16 → 143.08 | 1, 2 |
| manifest/image1k | 226.719 → 226.512 / 36.23 → 36.48 | 225.550 → 221.652 / 36.50 → 36.23 | — |
| manifest/text10k | 362.631 → 343.421 / 118.98 → 117.09 | 356.066 → 342.017 / 118.19 → 116.41 | — |
| manifest/text1k | 54.223 → 53.149 / 25.62 → 25.03 | 53.763 → 52.782 / 24.84 → 25.30 | — |
| manifest/wide1k | 376.179 → 344.393 / 119.50 → 139.77 | 376.541 → 354.584 / 118.94 → 139.73 | 1, 2 |
| prepared-control/image1k | 227.001 → 229.510 / 36.39 → 36.03 | 226.145 → 225.549 / 36.30 → 35.94 | — |
| prepared-control/text10k | 355.924 → 358.382 / 119.78 → 117.91 | 356.108 → 355.003 / 121.67 → 117.19 | — |
| prepared-control/text1k | 54.739 → 55.118 / 25.53 → 24.92 | 54.309 → 55.007 / 25.03 → 24.53 | — |
| prepared-control/wide1k | 373.660 → 380.691 / 119.17 → 139.78 | 377.966 → 379.307 / 139.45 → 139.33 | 1 |
| reuse/image1k | 420.403 → 267.229 / 40.19 → 39.11 | 419.556 → 264.762 / 41.27 → 39.05 | — |
| reuse/text10k | 909.610 → 475.358 / 134.67 → 132.78 | 897.478 → 472.772 / 135.67 → 132.59 | — |
| reuse/text1k | 115.034 → 69.247 / 27.75 → 27.27 | 117.558 → 68.917 / 28.28 → 27.42 | — |
| reuse/wide1k | 857.453 → 458.822 / 123.17 → 142.98 | 858.869 → 462.985 / 143.36 → 143.09 | 1 |
| reuse-blocked/text1k-blocked | 113.003 → 61.571 / 28.31 → 27.94 | 112.264 → 63.805 / 28.53 → 27.44 | — |
| reuse-changed/text1k-changed | 114.203 → 69.323 / 28.36 → 27.73 | 117.180 → 67.871 / 28.50 → 27.80 | — |

P1 准备后持有的私有候选大小（timing 样本中位数；生命周期测试验证 owner 不额外持有作者快照或构建 workspace）：

| 场景 | 第一批 MiB | 第二批 MiB |
|---|---:|---:|
| text1k | 0.416 | 0.416 |
| image1k | 119.072 | 119.072 |
| wide1k | 8.930 | 8.930 |
| text10k | 3.497 | 3.497 |

P4 分段（input 包含读取/复制/MIME/散列/快照等准备成本，不是纯缓存函数耗时）：

| 场景/批次 | 输入 ms：baseline → candidate | 导出操作 ms：baseline → candidate |
|---|---:|---:|
| bytes/bytes2m/1 | 16.320 → 11.493 | 26.195 → 26.657 |
| bytes/bytes8m/1 | 63.197 → 47.838 | 48.439 → 47.050 |
| bytes-single/bytes2m/1 | 2.048 → 2.081 | 26.891 → 26.280 |
| bytes-single/bytes8m/1 | 7.720 → 7.797 | 46.080 → 45.839 |
| bytes-single/bytes-small/1 | 0.228 → 0.229 | 20.549 → 21.836 |
| bytes/bytes2m/2 | 16.310 → 11.546 | 26.369 → 25.580 |
| bytes/bytes8m/2 | 64.466 → 47.063 | 49.029 → 46.966 |
| bytes-single/bytes2m/2 | 2.236 → 2.104 | 25.314 → 25.648 |
| bytes-single/bytes8m/2 | 7.914 → 8.008 | 47.138 → 47.753 |
| bytes-single/bytes-small/2 | 0.226 → 0.232 | 20.408 → 21.093 |

## Node COW 的成本与控制

原 Node 批次刻意让 task 和 clone 存活，并在任务完成前执行修改，因此 complete/process 时间包含首次 COW 编辑。后续 node-operations 批次移除 clone 和这些编辑，单独检查普通操作。Rust 宽字段是正反面各约 8 KiB 确定性文本；Node 宽字段是 16 KiB 答案，这两类输入不能当作跨语言同一工作负载。

| 操作/场景/批次 | 提交 ms：before → COW | 首次共享编辑 ms：before → COW | 每轮定时器最大延迟中的最大值 ms：before → COW |
|---|---|---|---|
| build/text10k/1 | 4.160 → 0.312 | 0.119 → 4.343 | 4.744 → 13.385 |
| build/text1k/1 | 0.559 → 0.265 | 0.031 → 0.447 | 1.731 → 1.596 |
| build/wide1k/1 | 1.328 → 0.290 | 0.048 → 2.077 | 3.023 → 2.357 |
| compare/text10k/1 | 4.511 → 0.302 | 0.125 → 4.166 | 4.568 → 10.009 |
| compare/text1k/1 | 0.652 → 0.274 | 0.036 → 0.400 | 2.313 → 6.706 |
| compare/wide1k/1 | 1.206 → 0.260 | 0.045 → 1.907 | 0.883 → 6.209 |
| prepare/text10k/1 | 4.092 → 0.372 | 0.123 → 4.345 | 14.702 → 4.707 |
| prepare/text1k/1 | 0.750 → 0.336 | 0.034 → 0.431 | 7.775 → 1.050 |
| prepare/wide1k/1 | 1.525 → 0.318 | 0.050 → 2.062 | 10.394 → 5.838 |
| build/text10k/2 | 3.991 → 0.306 | 0.118 → 4.424 | 4.060 → 6.438 |
| build/text1k/2 | 0.595 → 0.244 | 0.034 → 0.448 | 0.735 → 0.757 |
| build/wide1k/2 | 1.555 → 0.271 | 0.045 → 2.801 | 2.656 → 2.951 |
| compare/text10k/2 | 4.321 → 0.296 | 0.114 → 4.307 | 4.780 → 4.633 |
| compare/text1k/2 | 0.581 → 0.263 | 0.031 → 0.411 | 1.201 → 3.647 |
| compare/wide1k/2 | 1.314 → 0.225 | 0.044 → 1.986 | 3.010 → 1.842 |
| prepare/text10k/2 | 4.063 → 0.357 | 0.121 → 4.513 | 8.483 → 4.734 |
| prepare/text1k/2 | 0.652 → 0.305 | 0.031 → 0.455 | 0.488 → 0.918 |
| prepare/wide1k/2 | 1.336 → 0.320 | 0.044 → 2.112 | 1.607 → 3.100 |

clone、无快照修改、后续修改、失败 add 和全部 timer 样本保存在原始记录及 summary。这里的 timer 极值不是总体 p99；三个 RSS 高水位样本也不能证明稳定堆内存减少。

## 补充控制与标准矩阵

完整 staging 控制的 operation_ms 只覆盖 writer 调用；该子进程随后执行 manifest 解析/内容断言、APKG 复制与清理，因此 spawn→exit 和 RSS **包含这部分控制验证工作**。原始通用 plan 中“验证在计时外”的表述不适用于这一格；它仍验证真实 staging 引用、规范化内容和指纹，并保留此测量边界差异。外部 APKG verifier 与 Anki 导入仍在 collector 外。

| 批次 | 场景 | baseline → candidate ms | 降低 | RSS MiB：baseline → candidate | 复查 |
|---|---|---:|---:|---:|---|
| controls-native/1 | concurrent-miss/bytes2m | 36.483 → 37.982 | -4.11% | 30.52 → 30.61 | — |
| controls-native/1 | concurrent-miss/bytes8m | 86.147 → 82.514 | 4.22% | 83.33 → 83.50 | — |
| controls-native/1 | reuse-compare-control/image1k | 297.812 → 307.746 | -3.34% | 39.22 → 39.33 | — |
| controls-native/1 | reuse-compare-control/text10k | 472.346 → 469.942 | 0.51% | 132.70 → 131.91 | — |
| controls-native/1 | reuse-compare-control/text1k | 60.911 → 63.921 | -4.94% | 27.78 → 27.50 | — |
| controls-native/1 | reuse-compare-control/wide1k | 476.025 → 470.005 | 1.26% | 122.14 → 142.91 | 是 |
| controls-native/1 | staging-control/text1k | 21.786 → 21.693 | 0.43% | 20.09 → 20.17 | — |
| controls-native/1 | staging-control/wide1k | 265.500 → 254.159 | 4.27% | 149.70 → 149.92 | — |
| controls-native/2 | concurrent-miss/bytes2m | 35.742 → 35.014 | 2.04% | 30.48 → 30.58 | — |
| controls-native/2 | concurrent-miss/bytes8m | 70.755 → 69.128 | 2.30% | 83.34 → 83.52 | — |
| controls-native/2 | reuse-compare-control/image1k | 246.794 → 246.800 | -0.00% | 39.33 → 38.70 | — |
| controls-native/2 | reuse-compare-control/text10k | 451.477 → 455.423 | -0.87% | 131.73 → 131.53 | — |
| controls-native/2 | reuse-compare-control/text1k | 59.275 → 59.026 | 0.42% | 27.70 → 27.14 | — |
| controls-native/2 | reuse-compare-control/wide1k | 428.425 → 444.283 | -3.70% | 122.67 → 142.86 | 是 |
| controls-native/2 | staging-control/text1k | 21.582 → 21.736 | -0.72% | 20.11 → 20.06 | — |
| controls-native/2 | staging-control/wide1k | 253.109 → 249.472 | 1.44% | 149.77 → 149.92 | — |
| controls-node/1 | sdk-node/text10k | 776.390 → 455.202 | 41.37% | 216.75 → 215.20 | — |
| controls-node/1 | sdk-node/text1k | 139.776 → 100.316 | 28.23% | 78.41 → 78.19 | — |
| controls-node/1 | sdk-node/wide1k | 617.107 → 350.859 | 43.14% | 257.78 → 258.03 | — |
| controls-node/2 | sdk-node/text10k | 766.003 → 452.371 | 40.94% | 220.05 → 214.41 | — |
| controls-node/2 | sdk-node/text1k | 139.423 → 97.819 | 29.84% | 78.19 → 77.44 | — |
| controls-node/2 | sdk-node/wide1k | 581.760 → 343.505 | 40.95% | 289.09 → 258.36 | — |
| controls-python/1 | sdk-python/text10k | 5259.123 → 2720.461 | 48.27% | 140.34 → 123.98 | — |
| controls-python/1 | sdk-python/text1k | 611.601 → 345.939 | 43.44% | 52.73 → 48.08 | — |
| controls-python/1 | sdk-python/wide1k | 6515.847 → 3318.145 | 49.08% | 169.89 → 147.39 | — |
| controls-python/2 | sdk-python/text10k | 5283.393 → 2703.887 | 48.82% | 149.09 → 122.31 | — |
| controls-python/2 | sdk-python/text1k | 610.022 → 343.144 | 43.75% | 52.31 → 47.95 | — |
| controls-python/2 | sdk-python/wide1k | 6510.281 → 3315.127 | 49.08% | 169.53 → 148.20 | — |
| standard/1 | standard/basic-audio-unique-v2-100 | 41.708 → 41.547 | 0.39% | 21.20 → 21.17 | — |
| standard/1 | standard/basic-audio-unique-v2-1000 | 182.010 → 174.633 | 4.05% | 37.98 → 37.78 | — |
| standard/1 | standard/basic-audio-unique-v2-200 | 52.671 → 54.785 | -4.02% | 23.98 → 23.95 | — |
| standard/1 | standard/basic-audio-unique-v2-500 | 100.532 → 97.655 | 2.86% | 29.22 → 29.11 | — |
| standard/1 | standard/basic-image-unique-v2-100 | 42.242 → 41.260 | 2.32% | 20.53 → 20.81 | — |
| standard/1 | standard/basic-image-unique-v2-1000 | 222.032 → 216.498 | 2.49% | 36.61 → 36.09 | — |
| standard/1 | standard/basic-image-unique-v2-200 | 62.100 → 60.377 | 2.77% | 22.52 → 22.67 | — |
| standard/1 | standard/basic-image-unique-v2-500 | 122.195 → 119.459 | 2.24% | 27.84 → 27.64 | — |
| standard/1 | standard/basic-mixed-shared-v2-100 | 35.191 → 34.001 | 3.38% | 20.89 → 21.02 | — |
| standard/1 | standard/basic-mixed-shared-v2-1000 | 62.313 → 59.934 | 3.82% | 32.44 → 32.27 | — |
| standard/1 | standard/basic-mixed-shared-v2-200 | 37.052 → 34.975 | 5.60% | 22.19 → 22.34 | — |
| standard/1 | standard/basic-mixed-shared-v2-500 | 47.251 → 45.002 | 4.76% | 25.94 → 25.72 | — |
| standard/1 | standard/basic-mixed-text-v1-100 | 26.625 → 25.192 | 5.38% | 13.83 → 13.84 | — |
| standard/1 | standard/basic-mixed-text-v1-1000 | 51.897 → 49.663 | 4.31% | 25.39 → 24.44 | — |
| standard/1 | standard/basic-mixed-text-v1-200 | 28.264 → 26.490 | 6.28% | 15.06 → 15.20 | — |
| standard/1 | standard/basic-mixed-text-v1-500 | 36.628 → 35.641 | 2.69% | 18.58 → 18.39 | — |
| standard/1 | standard/basic-mixed-unique-v2-100 | 35.767 → 35.941 | -0.49% | 22.17 → 22.23 | — |
| standard/1 | standard/basic-mixed-unique-v2-1000 | 161.100 → 151.692 | 5.84% | 37.06 → 36.88 | — |
| standard/1 | standard/basic-mixed-unique-v2-200 | 49.541 → 47.993 | 3.12% | 24.34 → 24.23 | — |
| standard/1 | standard/basic-mixed-unique-v2-500 | 89.826 → 88.002 | 2.03% | 29.44 → 29.11 | — |
| standard/2 | standard/basic-audio-unique-v2-100 | 40.749 → 42.046 | -3.18% | 21.03 → 21.09 | — |
| standard/2 | standard/basic-audio-unique-v2-1000 | 179.261 → 179.375 | -0.06% | 38.25 → 37.61 | — |
| standard/2 | standard/basic-audio-unique-v2-200 | 52.482 → 53.084 | -1.15% | 23.64 → 23.70 | — |
| standard/2 | standard/basic-audio-unique-v2-500 | 101.261 → 100.078 | 1.17% | 29.27 → 29.09 | — |
| standard/2 | standard/basic-image-unique-v2-100 | 42.290 → 42.451 | -0.38% | 20.70 → 20.69 | — |
| standard/2 | standard/basic-image-unique-v2-1000 | 222.898 → 219.711 | 1.43% | 36.14 → 36.03 | — |
| standard/2 | standard/basic-image-unique-v2-200 | 61.826 → 60.662 | 1.88% | 22.69 → 22.81 | — |
| standard/2 | standard/basic-image-unique-v2-500 | 126.744 → 119.874 | 5.42% | 27.92 → 27.58 | — |
| standard/2 | standard/basic-mixed-shared-v2-100 | 34.780 → 34.681 | 0.28% | 20.95 → 20.98 | — |
| standard/2 | standard/basic-mixed-shared-v2-1000 | 64.481 → 61.344 | 4.86% | 32.77 → 32.16 | — |
| standard/2 | standard/basic-mixed-shared-v2-200 | 36.379 → 36.234 | 0.40% | 22.19 → 22.25 | — |
| standard/2 | standard/basic-mixed-shared-v2-500 | 47.036 → 44.704 | 4.96% | 25.98 → 26.11 | — |
| standard/2 | standard/basic-mixed-text-v1-100 | 28.175 → 26.629 | 5.49% | 14.02 → 13.91 | — |
| standard/2 | standard/basic-mixed-text-v1-1000 | 52.138 → 51.124 | 1.94% | 24.95 → 24.69 | — |
| standard/2 | standard/basic-mixed-text-v1-200 | 28.262 → 27.557 | 2.50% | 15.03 → 15.17 | — |
| standard/2 | standard/basic-mixed-text-v1-500 | 36.969 → 35.491 | 4.00% | 18.81 → 18.42 | — |
| standard/2 | standard/basic-mixed-unique-v2-100 | 36.490 → 35.513 | 2.68% | 22.12 → 22.20 | — |
| standard/2 | standard/basic-mixed-unique-v2-1000 | 159.514 → 157.121 | 1.50% | 37.20 → 36.92 | — |
| standard/2 | standard/basic-mixed-unique-v2-200 | 50.171 → 49.802 | 0.74% | 24.12 → 24.17 | — |
| standard/2 | standard/basic-mixed-unique-v2-500 | 91.940 → 87.765 | 4.54% | 29.42 → 29.34 | — |

node-operations 的完整提交、完成、总耗时、RSS 与配对结果见其 summary.json；该批次不混入 COW 首次修改。

## 验证与未关闭的门槛

- Rust workspace：708 passed、0 failed、23 ignored；Node：29 passed、1 skipped；Python：67 passed、1 skipped。定向生命周期、路径/别名、策略阻止、fork、并发、缓存命中预算/MIME 和真实内容回归通过。
- Rust packaged consumer、安装后的 Node ESM/CJS 与全部 README 示例、TypeScript 消费者、Python wheel 与正负类型检查通过。mypy、clippy、格式、contract governance、payload/bundle 检查通过。
- 独立真实 Anki roundtrip 28 场景全部 verified；基准导出另逐个进行实际内容校验，并对计划内样本运行 Anki oracle。
- 宽字段 RSS 的确认回归是开放验收项。隔离探针在字段渲染边界就观察到分歧；Rust allocator 包装探针的存活分配峰值为 baseline 40,171,247 bytes、combined 40,171,016 bytes，结束存活量相同。该探针不统计 C 库分配，且会改变布局，不能代替正式 RSS 或证明所有平台无回归。原始探针日志和失败的 time 辅助调用保留。
- 未运行其他 Rust Tier 1、Node 支持 runtime 和 Python 平台的 CI；本机通过不能代替跨平台验收及 release gates。
- 没有公开发布包或部署网站；依据原 spec，在 RSS 取舍得到明确解决前，本分支只作为待审候选，不能宣布完成验收。

## 数据使用

每份 completed marker 表示该批测量/内容验证完成，不表示性能或发布获准。原始计划、失败记录、完整比较报告、源码/二进制/fixture 哈希、构建命令、所有 timing/RSS 样本及验证结果进入归档。APKG 的时间戳、持续时间和私有路径等非语义差异不要求跨进程逐字节一致；比较报告保留完整证据，不以删减诊断或降低检查预算获取成绩。离线复算与重新导出/Anki 导入是不同验证。
