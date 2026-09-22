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
| T3 作者、校验与身份 | 主要接口及 0.1 更新链路已实现，待最终矩阵/审查 | 自定义/多模板/Cloze/IO、类型及单笔记身份、核心校验、输入快照、默认 key、optional、Content |
| T4 媒体注册证据 | 注册与引用链路已实现，待最终矩阵与审查 | 空/超限、bytes 快照、文件错误路径、首次注册证据、源文件变化、跨项目完整产物对照 |
| T5 构建配置与安全更新 | 配置与迁移链路已实现；旧 CLI 文件清理待集成 | 11 预算、报告、媒体策略、更新模式、真实 0.1 更新、路径保护 |
| T6 Artifact 与输出 | 主要链路已实现，待最终类型/分发/审查 | 原生 clone/persist、报告/异常持有、context/close、bytes、有界 short-write 复制 |
| T7a 模板包导入 | 已实现，待最终审查 | 独立 Rust 包对照、Unicode 字节偏移、资源中途失败回滚 |
| T7b 独立比较 | 已实现，待最终审查 | 完整 Rust 报告对照、预算、无发布/锁文件副作用 |
| T8 Deck 与转换 | 已实现，待最终审查 | 真实 Deck、自动身份 IO、边界检查、快照转换与追加 custom 独立对照 |
| T9 对等、打包、类型与文档 | 集成完成；最终 CI/源码包/审查运行中 | 单一公开实现、版本检查、py.typed、consumer 类型检查、并发/fork、文档/许可证 |

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
- 迁移说明开始记录于 [Python 0.2 迁移](../../bindings/python/MIGRATION.md)。T3 检查点时尚缺真实 APKG/lockfile 安全更新验证，后续 T5 的结果记录于下节。
- 本轮聚焦验证：Python 原生作者/媒体/项目/独立对照共 19 项通过，mypy 16 文件通过，native crate 全 target clippy 零警告；默认 Rust facade 的 custom_notetype_api_tests 4 项通过。
- 开启 internal-tools 的 product_v3_tests 另有 10 项通过；此前未开启该 feature 的运行收集 0 项，不算该项证据。

作者/媒体检查点提交：`70c3aef`。未更新远端 main。

## T5 构建配置与真实升级

- 冻结的 `BuildOptions` 接入 output/artifacts_dir/report_json/inspect、全部 11 个 InspectLimits、基线/风险/锁文件/更新模式、自包含和媒体策略。默认预算直接读取 Rust，Python int 保留 u64 精度，并拒绝 bool/浮点/负数/溢出。
- 每个预算单独设为 0，断言相应核心资源诊断，验证 `inspect=False` 不跳过最终检查且旧输出不被覆盖。额外验证基线预算对 strict/report_only/report-only/disabled 的核心区别。
- `write_apkg` 恢复旧关键字入口，也接受 BuildOptions；重复且冲突的参数在发布前拒绝。路径基于构造时 base_dir，并保留 symlink 语义供核心检查。
- 原生 BuildReport 暴露 failure_cause/failure_code；`ensure_success()` 抛 BuildError（继承 DiagnosticsError），保留原 report，不把有效失败报告变成无上下文异常。
- 实际 0.1 APKG/lockfile 的 unchanged、答案修改、标签修改、重复构建、回退更新通过 GUID/model/template ID 与 revision 对照。旧锁文件缺 revision 时 strict 阻断且不改旧输出；提供原 APKG 后可恢复证据并更新锁文件。
- 对 unchanged 升级，GUID 来源由 current_derivation 改为 previous_apkg；Rust Project 会为两个自定义模板补写 inferred all generation requirement，旧 ProductDocument 产物未记录它。测试逐项断言这些差异，其余完整观察保持比较，不删除差异字段来通过测试。
- 既有 Python E2E 的风险阻断、两种 lockfile-only 风险场景和 hardlink 保护 4 项已在原生接口通过。新用例验证 symlink、报告同路径、staging 内基线的拒绝行为。
- 仍需 T6–T9：产物扩展、输出、bundle/diff、Deck、公开模块清理、全矩阵/打包文档/最终 code-review。此检查点仍不可发布。

## T6 产物与文件对象

- `copy.copy/deepcopy(ApkgArtifact)` 克隆真实 Rust Arc；`persist_to` 先取得独立 core owner，再释放解释器执行原子复制。关闭原 handle 不会提前删除复制操作正在读取的源文件。
- 普通 Python 赋值仍共享同一 handle，显式 close 会影响其别名。复制报告则复制 Artifact handle；`report.close()` 只释放报告自己的引用，独立持有的 Artifact 继续可用。
- 临时自路径/hardlink alias、目录目标等持久化失败保留原件；成功持久化和显式输出不随 close 删除。JSON 路径快照不延长临时文件寿命。
- 晚期锁文件写失败的 BuildError 保留完整 report 和可恢复 Artifact；`report_json` 没有持久化输出时遵守核心拒绝规则。
- `to_apkg_bytes` 返回完整 bytes；`write_to` 先完成 APKG 再以 64 KiB 有界复制。处理短写、零进展、None、非法 count 和调用者异常，不关闭调用者流。
- 512 KiB 非压缩友好媒体的 short-write sink 输出，与独立 Rust 文件生产者完整观察一致。sink 回调可继续修改同一 Project，证明复制阶段未持有项目操作锁。
- 在独立临时目录断言 bytes/流复制成功与失败后无临时残留；异常保留 traceback 时也显式清理复制用 Artifact。
- T3–T6 聚焦原生验证共 57 项通过；mypy 17 文件通过；native 全 target clippy 零警告。全仓库测试和最终平台包验证尚未运行。


## T7/T8 模板、比较和 Deck

- 模板导入调用 Rust loader；CSS/字体/图片及模板原文与独立手工 Rust Project 产物一致。错误保留 UTF-8 byte_offset；未知字段修复后可重试；第二个 asset 冲突不会留下第一个 asset 或 note type。
- diff 返回完整 ProjectDiffReport / ProjectDiffError，保留 failure_cause、未知扩展与大整数。独立 Rust 报告只归一化 duration_ms。比较前后用户目录、原 APKG、lockfile 和候选临时目录保持不变。
- Deck 持有真实 Rust Deck，支持 Basic identity selection/override、Cloze、自动身份 IO、Rust 图片尺寸/矩形检查。Project.from_deck 克隆核心快照后转换，原 Deck 可继续使用，Project 可追加自定义类型，媒体指纹和身份/revision 与 Rust 一致。
- Project/Deck 共用构建输出适配层和独占状态租约，保持不同作者语义。共享核心的 grouped IO 限制仍明确失败。

## T9 集成与候选验证（进行中）

- 公开 project/media 模块已统一为原生实现，删除默认包的旧 CLI/runtime/ProductDocument 序列化层与 staging 脚本。dev-only anki_forge_python 保留；迁移指南明确 old MediaItem 观察视图与手工序号引用的版本归属。
- 完整报告解析回归保留在 test_report，旧作者/更新安全测试迁移为真正的 Rust 调用；不再通过 mock subprocess 构造成功报告。单独保留 raw/structured 旧 CLI 消费侧回归。
- 泛型 BuildReport 区分拥有型 ApkgArtifact 与 JSON 路径 Mapping；外部已安装 wheel 的正向 mypy 用例通过，负向用例准确报告 6 个类型错误（包括 JSON 路径无 persist_to）。
- 公开 versions()、导入时 extension/version 错配错误、py.typed/native stub、MIT 与依赖 notices 已加入。依赖 notices 从锁图和实际 license 文件生成，缺失的两个文本取自对应发布源码的固定 commit。
- 两个 5,000-note 并发用例证明 Rust 释放 GIL、同对象忙时拒绝修改、独立 Project 可继续工作；领域错误后对象恢复可用。
- fork 回归先暴露子进程 close 会误删父进程 Artifact；增加 PID 检查与子进程析构保护后，close/GC 均不会删除父进程临时文件。Project/Deck fork 前置拒绝同样通过。
- 完整 Python 首轮：162 通过，1 个新字段改名测试断言失败。核查发现核心把身份字段显示名称纳入推导；Python 与独立 Rust 全观察本来一致。测试改为分别验证重排/改名/显式 ID，三个聚焦用例通过，并把迁移影响写入指南，没有修改 Rust 身份算法。
- 本机 release wheel 已在独立 venv/仓库外中文路径，通过版本、Basic、媒体证据、Artifact、完整 native_workflow 示例及正负类型消费者。完整平台矩阵、sdist 重建、最终审查仍在运行；不能据此标记整体完成。

## 最终审查修复与再次验证

- [双轴审查记录](2026-09-22-python-api-parity-review.md)：Standards 和 Spec 的问题已修复，复查均为 0 项未解决。
- 保留字段/模板的精确 Unicode、空白和核心派生空 key；stock alias 在 setter 时统一，公开字典的歧义输入在添加前拒绝。新增失败回归后修复；快照和 IO 共用 Note 解码。
- 新增独立 Rust `names` 和 `bundle_cloze` 完整产物对照；补齐模板包 UTF-8、manifest、三种大小上限、越界路径/symlink、重复类型。
- 历史样本重新从完整 `51a44ad` 源码、Cargo.lock 与 contracts 构建；APKG 与 lockfile 字节均未变化，provenance 增加固定 contracts commit 和两个 SHA256。
- 本地最终 Python 全套 **192 项通过、5 个子测试通过**；mypy **16 文件通过**；native crate 全 targets Clippy 零警告。
- Rust 工作区 **1,036 项通过、25 项按原配置忽略**（79 个测试套件）。此后 Rust 修改仅为独立测试 producer，已重新编译、Clippy 和实际调用验证。
- 候选 `37f3593` 的四平台运行暴露 Windows 测试边界问题：UTF-8 JSON 被按系统默认编码读取、等价 canonical 路径直接比字符串。已指定 UTF-8、通过 samefile 验证来源路径，保留 Unicode 用例。
- 修复候选还需重新完成四平台 Python 3.11/3.12 安装；新增 Linux 独立 sdist 离线重建 CI 门禁。最终 run 和状态将在结果完成后记录。
