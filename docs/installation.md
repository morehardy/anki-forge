# Install and run

These guides describe the current source checkout. See
[compatibility and release status](compatibility.md) before selecting a published
package. A source version is not evidence that the same version is available in
a public registry.

## Requirements

- Rust 1.92 or later for the Rust library and source builds.
- Node.js 22.13 or later for the native Node SDK.
- Ordinary CPython 3.11/3.12 for the declared Python verification matrix.

A compiler is needed to build native packages from source. Applications using a
matching prebuilt native package do not invoke Cargo at runtime.

## Rust application from a checkout

Create a small binary application. In its `Cargo.toml`, point `ankiforge` at the
checkout's `anki_forge` directory; replace the path with your actual checkout.
`anyhow` is an application choice for concise error propagation, and `serde_json`
is used by the reporting examples.

```toml
[dependencies]
ankiforge = { path = "/absolute/path/to/anki-forge/anki_forge" }
anyhow = "1"
serde_json = "1"
```

Save this as `src/main.rs`, then run `cargo run`:

```rust
use ankiforge::{BuildOptions, Note, Project};

fn main() -> anyhow::Result<()> {
    let mut project = Project::new("spanish")?.default_deck("Spanish");
    project.add("hola", Note::basic("hola", "hello"))?;
    let output = project.build(BuildOptions::to("spanish.apkg"))?;
    println!("{}", output.artifact().path().display());
    Ok(())
}
```

Open `spanish.apkg` with Anki. The namespace `spanish` and key `hola` identify the
publication and note; the deck name is a display destination. Continue with
[the Rust guide](rust-guide.md) or [Basic and Cloze cards](cards.md).

## Run repository examples

From the repository root:

```sh
cargo run --locked -p ankiforge --example target_api_basic
cargo run --locked -p ankiforge --example target_api_custom_notetype
cargo run --locked -p ankiforge --example target_api_media
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
```

The workflow example uses repository fixtures for bundle, image and audio
coverage and creates its output directory. No network media download is needed.

## Node and Python

Use [the Node quick start](node/quick-start.md) or
[the Python quick start](python/quick-start.md) for native builds and local package
installation. Keep each facade and native binary at matching versions. Supported
host packages are platform-specific; an executable for another operating system
or CPU cannot be substituted.
