# anki-forge

`anki-forge` 是一个以 contract 为核心的 Anki 产物构建项目，支持：

- Rust API（`anki_forge`）
- CLI 管线（`contract_tools`）
- Node/Python 绑定（`bindings/node`、`bindings/python`）

最常见流程是：
`Authoring IR -> normalize -> build -> inspect -> diff`

## 1. 环境要求

- Rust `1.92.0`（见 `rust-toolchain.toml`）
- `cargo`
- `jq`（用于从 normalize 结果中提取 `normalized_ir`）
- 可选：Node.js（运行 Node 绑定示例/测试）
- 可选：Python `3.11+`（运行 Python 绑定示例/测试）
- 可选：`protoc` + 本地 `docs/source/anki`（仅 roundtrip oracle 需要）

建议先在仓库根目录执行：

```bash
rustup toolchain install 1.92.0
rustup override set 1.92.0
```

## 2. 快速开始（最短可运行路径）

在仓库根目录执行：

```bash
cargo run -q -p contract_tools -- verify --manifest "$(pwd)/contracts/manifest.yaml"
cargo run -q -p anki_forge --example minimal_flow
```

这两步分别用于：

- 检查 contract bundle 与 gate 是否通过
- 跑通一条最小端到端流程（会在 `tmp/phase4-examples/minimal-flow` 生成输出）

## 3. 基本操作速查

### 3.1 Contract 校验与打包

```bash
cargo run -q -p contract_tools -- verify --manifest "$(pwd)/contracts/manifest.yaml"
cargo run -q -p contract_tools -- summary --manifest "$(pwd)/contracts/manifest.yaml"
cargo run -q -p contract_tools -- package --manifest "$(pwd)/contracts/manifest.yaml" --out-dir "$(pwd)/dist"
```

- `verify`：校验 contracts 与可执行 gate
- `summary`：输出 bundle 版本与组件摘要
- `package`：将版本化产物输出到 `dist/`

### 3.2 标准管线：normalize -> build -> inspect -> diff

```bash
mkdir -p tmp/readme-basic

cargo run -q -p contract_tools -- normalize \
  --manifest "$(pwd)/contracts/manifest.yaml" \
  --input "$(pwd)/contracts/fixtures/phase3/inputs/basic-authoring-ir.json" \
  --output contract-json > "$(pwd)/tmp/readme-basic/normalize.result.json"

jq -e '.normalized_ir' "$(pwd)/tmp/readme-basic/normalize.result.json" > "$(pwd)/tmp/readme-basic/normalized-ir.json"

cargo run -q -p contract_tools -- build \
  --manifest "$(pwd)/contracts/manifest.yaml" \
  --input "$(pwd)/tmp/readme-basic/normalized-ir.json" \
  --writer-policy default \
  --build-context default \
  --artifacts-dir "$(pwd)/tmp/readme-basic/artifacts" \
  --output contract-json > "$(pwd)/tmp/readme-basic/build.result.json"

cargo run -q -p contract_tools -- inspect \
  --staging "$(pwd)/tmp/readme-basic/artifacts/staging/manifest.json" \
  --output contract-json > "$(pwd)/tmp/readme-basic/staging.inspect.json"

cargo run -q -p contract_tools -- inspect \
  --apkg "$(pwd)/tmp/readme-basic/artifacts/package.apkg" \
  --output contract-json > "$(pwd)/tmp/readme-basic/apkg.inspect.json"

cargo run -q -p contract_tools -- diff \
  --left "$(pwd)/tmp/readme-basic/staging.inspect.json" \
  --right "$(pwd)/tmp/readme-basic/apkg.inspect.json" \
  --output contract-json > "$(pwd)/tmp/readme-basic/diff.result.json"
```

主要输出文件：

- `tmp/readme-basic/artifacts/package.apkg`
- `tmp/readme-basic/staging.inspect.json`
- `tmp/readme-basic/apkg.inspect.json`
- `tmp/readme-basic/diff.result.json`

### 3.3 Rust 示例

```bash
cargo run -q -p anki_forge --example deck_basic_flow
cargo run -q -p anki_forge --example product_basic_flow
cargo run -q -p anki_forge --example minimal_flow
```

- `deck_basic_flow`：基础 Rust Deck API（会写出 `spanish.apkg`）
- `product_basic_flow`：Phase 5A product authoring 示例
- `minimal_flow`：文件驱动 runtime 示例

### 3.4 Stable Note Identity

新建 Basic、Cloze 和 Image Occlusion note 默认会使用 AFID（`afid:v1:*`）作为稳定 note id，而不是旧的 `generated:*` id。AFID 来自规范化后的 identity payload：Basic 默认使用 front 字段，Cloze 使用 cloze 结构与文本骨架，Image Occlusion 使用图片内容锚点、尺寸、模式和排序后的遮罩几何。

显式 `stable_id` 仍然优先，并会作为 explicit identity snapshot 保存。`generated:*` 前缀如果由调用方显式传入，会被当作普通显式 stable id 保留；`afid:v1:*` 是保留命名空间，不能作为显式 stable id 传入。

Basic note 可以用类型化 API 改变 identity 字段：

```rust
use anki_forge::{BasicIdentityField, BasicIdentityOverride, BasicIdentitySelection, BasicNote, Deck};

let mut deck = Deck::builder("Spanish")
    .basic_identity(BasicIdentitySelection::new([BasicIdentityField::Back])?)
    .build();
deck.add(BasicNote::new("hola", "hello"))?;

let override_cfg = BasicIdentityOverride::new(
    [BasicIdentityField::Front, BasicIdentityField::Back],
    "sense-disambiguation",
)?;
deck.basic()
    .note("banco", "bank / bench")
    .identity_override(override_cfg)
    .add()?;
```

`validate_report()` 会保留旧的 stable id 诊断（blank、missing/generated legacy、unknown media、empty IO masks、duplicate ids），并在 note 使用 note-level identity override 时返回 `NoteLevelIdentityOverrideUsed` warning。AFID duplicate payload、hash collision 和 stable id duplicate 在 add-time 或 load-time rebuild 阶段是 blocking error。

序列化会保留 resolved identity snapshot（`stable_id`、`recipe_id`、`provenance`、`canonical_payload`、`used_override`）。反序列化会重建运行时索引并校验 snapshot 与 note id、payload hash、payload duplicate/collision 是否一致，因此 round-trip 后继续保持相同的 note id 与重复检测行为。

### 3.5 Node 绑定

```bash
npm --prefix bindings/node install
npm --prefix bindings/node run example:minimal
npm --prefix bindings/node test
```

### 3.6 Python 绑定

```bash
PYTHONPATH=bindings/python/src python3.11 bindings/python/examples/minimal_flow.py
PYTHONPATH=bindings/python/src python3.11 -m unittest discover -s bindings/python/tests -p "test_*.py"
```

## 4. 手动 Anki Desktop 场景

生成全部手动验证场景 APKG：

```bash
./scripts/run_manual_desktop_scenarios.sh
```

只生成单个场景（示例）：

```bash
./scripts/run_manual_desktop_scenarios.sh S05_basic_audio
```

输出目录：

- `tmp/manual-desktop-v1/<scenario>/package.apkg`
- `tmp/manual-desktop-v1/<scenario>/apkg.inspect.json`

## 5. Roundtrip Oracle（可选，高级）

仅在你要验证与本地 Anki 上游实现的 roundtrip 行为时使用。

前置条件：

- `docs/source/anki/rslib/Cargo.toml` 存在
- `protoc` 在 `PATH` 中可用

运行：

```bash
./scripts/run_roundtrip_oracle.sh
```

## 6. 常见问题

- 报错 `failed to discover contracts/manifest.yaml from workspace path`
  - 请确认当前目录在本仓库内，或在调用绑定时显式设置 `cwd`
- 报错 `missing vendored upstream Anki crate ... docs/source/anki/rslib`
  - 这是 roundtrip oracle 缺少本地 Anki 源码，不影响常规流程
- 报错 `protoc is required on PATH`
  - 安装 `protoc` 后重试（仅 roundtrip oracle 需要）

## 7. 相关文档

- `bindings/node/README.md`
- `bindings/python/README.md`
- `contracts/fixtures/phase3/manual-desktop-v1/README.md`
