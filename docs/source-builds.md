# Build SDKs from source

Use this guide for repository development and unreleased changes.
For the published `0.3.0` packages, use the [Rust](installation.md),
[Node](node/quick-start.md) or [Python](python/quick-start.md) installation guide.

Requirements: Git, Rust 1.92.0 with Cargo, and Node.js 22.13+ with npm or
CPython 3.11/3.12 for the corresponding SDK. The checkout pins the Rust toolchain;
install it with `rustup toolchain install 1.92.0` if needed.

## Source builds

To build the current source checkout:

```sh
git clone https://github.com/morehardy/anki-forge.git
cd anki-forge
```

Rust consumers can use `ankiforge = { path = "/absolute/path/to/anki-forge/anki_forge" }`
in their application manifest. Run examples with `cargo run --locked -p ankiforge --example target_api_basic`.

Build the Node addon from this checkout:

```sh
cd bindings/node
npm run setup
npm run build
npm run pack:local
```

Install both actual facade and host-native tarballs printed by `pack:local` into a separate application using
`npm install --offline --ignore-scripts --omit=optional /absolute/path/facade.tgz /absolute/path/native.tgz`.
These are source builds, not registry installations. TypeScript configuration is in the [Node API](node/api.md#typescript).

For Python, activate a CPython 3.11/3.12 virtual environment, install Maturin,
then run from the repository root:

```sh
python -m pip install maturin==1.15.0
maturin develop --manifest-path bindings/python/native/Cargo.toml --locked
```

Ordinary installed wheels need no Cargo at runtime. Do not mix source wrappers with older registry native binaries.
