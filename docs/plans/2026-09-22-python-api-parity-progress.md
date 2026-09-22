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
| T2 原生纵向试验 | 本机链路通过；平台、性能门禁进行中 | 4 个原生/独立 Rust 对照测试；release wheel 与 sdist 仓库外安装通过；clippy 零警告 |
| T3 作者、校验与身份 | 待实施 | |
| T4 媒体注册证据 | 待实施 | |
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
