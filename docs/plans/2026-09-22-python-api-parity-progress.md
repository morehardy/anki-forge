# Python API 对齐实施记录

基线：`51a44ad`。范围保持为 [实施方案](2026-09-21-python-api-parity-implementation-plan.md)
的 T0–T9 与 PY-01–PY-20；本文件记录证据，不替代原验收标准。

测试边界沿用已确认方案：Python 公共作者/构建/报告 API、生成的 APKG、
独立 Rust producer，以及仓库外 wheel 消费者。先运行单项失败回归，修复后
运行对应文件与类型检查，完成后运行全套、code-review 并提交当前分支。

| 任务 | 状态 | 证据 |
| --- | --- | --- |
| T0 范围与迁移 ADR | 已记录，后续门禁待验证 | ADR 0021 |
| T1b 媒体引用 | 已修复，待最终审查 | 先复现序号碰撞；product_json 13 项通过 |
| T1a 模板/CSS | 已修复，待最终审查 | LF/CRLF/Tab 原文及实际导出；E2E/模型 29 项通过 |
| T1c 报告与风险参数 | 已修复，待最终审查 | 报告 39 项、E2E/校验 27 项；mypy 9 文件通过 |
| T2 原生纵向试验 | 试验通过，最终版本仍需重新跑分发门禁 | 四平台 wheel 在 Python 3.11/3.12 真实安装成功；本机 release wheel、sdist 与性能采样完成 |
| T3 作者、校验与身份 | 主要接口已实现，0.1 安全更新验收待 T5 | 自定义/多模板/Cloze/IO、类型及单笔记身份、核心校验、输入快照、默认 key、optional、Content |
| T4 媒体注册证据 | 注册与引用链路已实现，待最终矩阵与审查 | 空/超限、bytes 快照、文件错误路径、首次注册证据、源文件变化、跨项目完整产物对照 |
| T5 构建配置与安全更新 | 待实施 | |
| T6 Artifact 与输出 | 待实施 | |
| T7a 模板包导入 | 待实施 | |
| T7b 独立比较 | 待实施 | |
| T8 Deck 与转换 | 待实施 | |
| T9 对等、打包、类型与文档 | 待实施 | |

已有的 Node 计划和 website 工作不在本次修改范围。

T1 风险组合补充：有效 lockfile + fail_on high 的核心结果为 success/passed；
不可读 lockfile + report_only + fail_on high 为 blocked，policy 中保留
RISK.BASELINE_UNAVAILABLE。Python 不额外要求 compare_to。

工具环境：`target/python-sdk-venv`；pytest 9.1.1、mypy 2.3.1、maturin 1.15.0。
类型检查命令：`target/python-sdk-venv/bin/python -m mypy --config-file bindings/python/pyproject.toml bindings/python/src/anki_forge`。

## T2 证据与发现

- PyO3 固定 0.29.2（依赖 MSRV 1.83），Maturin 固定 1.15.0；实际用 Rust 1.92 构建。
- `cargo clippy -p anki_forge_python_native --all-targets --locked --offline -- -D warnings` 通过。
- 原生 Basic 与独立 Rust producer 的完整 APKG 观察、GUID、模型 ID、身份来源和 revision 相同；只归一化检查器已验证的输入文件路径。
- 注册后修改源文件得到 MEDIA.SOURCE_CHANGED；恢复源内容后同一项目可继续构建。
- 报告释放后独立 Artifact 引用仍保留文件；最后 handle 关闭后删除；显式输出不会随报告释放删除。
- release wheel 已在独立 venv、仓库外中文/空格路径运行，PATH 不含 CLI/编译器。
- sdist 经实际 PEP 517 后端在仓库外 `--locked --offline` 构建，再安装生成的 wheel 通过相同 smoke。
- 本机 macOS 27 暴露 Rust debuginfo strip 后的 Mach-O string-pool 对齐问题；仅关闭 Python native crate 的 release strip 后可正常加载。
  [Rust 上游问题](https://github.com/rust-lang/rust/issues/157750)。未修改核心或 Node 的编译 profile。
- Maturin 会裁剪 sdist 的 workspace members，却留下额外 lock entries。PEP 517 后端现在离线裁剪锁图，检查版本/source/checksum 不变，重新验证 `--locked` 后才输出 sdist。
- 真实 0.1 源码 `51a44ad` 生成的 APKG/lockfile 已保存在 `bindings/python/tests/fixtures/python01`，待 T3/T5 迁移验证。

当前是实施中的原生试验检查点，完整 API 和旧测试迁移尚未结束，不能发布。
旧 CLI 文件暂存于源码中用于过渡；T5/T9 完成时移除默认包中的双实现。
四平台证据由 `.github/workflows/python-native-trial.yml` 产生，尚不能用本机结果替代。

检查点提交：`8a44a944c656674a6343d28204333cfe040c7040`；只推送专用分支
`codex/python-sdk-parity`，没有更新远端 main 或发布包。
[四平台运行](https://github.com/morehardy/anki-forge/actions/runs/35687448346) 已全部成功，验证 Python 3.11/3.12 安装。
此结果对应 T2 提交，后续 API 修改仍要在最终提交重新验证。

## T2 本机性能采样

可复现命令：`target/python-sdk-venv/bin/python bindings/python/scripts/benchmark_native_trial.py --output bindings/python/benchmarks/native-trial-2026-09-22.json`。
运行前用 Maturin develop 的 `--release --locked` 安装原生扩展。
[原始结果](../../bindings/python/benchmarks/native-trial-2026-09-22.json) 包含平台、输入规模及各阶段毫秒数。

| Basic 数量 | CLI 作者输入 | 原生作者输入 | CLI 首次构建 | 原生首次构建 | CLI 重复构建 | 原生重复构建 |
| --- | --- | --- | --- | --- | --- | --- |
| 1,000 | 4 ms | 16 ms | 81 ms | 46 ms | 42–43 ms | 30–31 ms |
| 10,000 | 36 ms | 86 ms | 254 ms | 246 ms | 246–251 ms | 235–237 ms |

原生注册时执行核心校验/媒体取证；10 次 512 KiB 文件注册约 6 ms，CLI 记录路径约 0.5 ms。
原生首个进程 import 观察到 1.66 s，后一个进程约 14 ms；CLI 约 17–30 ms。
这些是有顺序的一次本机采样，不能推断普遍的冷启动或吞吐收益。
原生 10 次默认临时构建释放后，已知的 Artifact 路径残留数为 0；未对 CLI 的所有进程临时文件作同等断言。
采用原生层的理由仍是语义与所有权对齐，并非“所有操作都会更快”。

## T3/T4 作者与媒体证据

- 新增 `IdentityRecipe`、`NoteType.identity`、`Note.identity`；保留 `Field(identity=True)` 回退，显式类型 recipe 优先，最终身份完全由 Rust 计算。
- `Project.add_notetype/add_note` 使用核心同步校验；重复 ID、未知字段、身份不足、模板错误保持 code/path/span，失败后对象可复用。
- `Project.validate()` 调用核心聚合校验，不读取注册文件，也不写 APKG；自动 key 的警告没有在适配时丢失。
- `notes/notetypes/notetype_order` 按需读取核心快照。Rust 只增加 internal-tools 下的借用型只读视图和 typed field 观察接口，没有增加公开 mutation seam。
- Field/Template 默认 key 分别调用核心构造器；保留原字段和模板名称。`Field(optional=True)` 支持回读，required/optional 同时为真在 Python 入口拒绝。
- `Content.text/html`、`note.field`、`MediaRef.image/sound` 使用核心渲染。图片遮挡 builder 的校验与渲染已替换为真正的 Rust builder。
- 独立 Rust producer 对照覆盖 Basic、自定义双模板、三层身份优先级、stock/custom Cloze、图片遮挡和跨项目媒体；比较完整 inspector 观察与身份/revision，仍仅归一化已核实的 APKG 来源路径。
- `add_bytes` 使用真实 Rust 注册；空内容、65,537 字节被拒绝，65,536 字节可用，bytearray 在解释器边界快照。同名同内容去重保留首次注册的文件/inline 来源证据。
- 文件注册错误保留核心 code 与源路径；同名不同内容报 `MEDIA.DUPLICATE_FILENAME_CONFLICT`。跨项目引用先因目标缺 filename 失败，注册目标同名内容后恢复且与 Rust 产物相同。
- 迁移说明开始记录于 [Python 0.2 迁移](../../bindings/python/MIGRATION.md)。0.1 真实 APKG/lockfile 的安全更新测试仍依赖 T5，尚不能据此关闭完整迁移验收。
- 本轮聚焦验证：Python 原生作者/媒体/项目/独立对照共 19 项通过，mypy 16 文件通过，native crate 全 target clippy 零警告；默认 Rust facade 的 custom_notetype_api_tests 4 项通过。
