# Rust: public Anki Forge 0.3.1

Requires Rust 1.92+. Inspect `Cargo.toml` and `Cargo.lock` in an existing application.
For a new application:

```sh
cargo new anki-deck
cd anki-deck
cargo add ankiforge@0.3.1
```

Save as `src/main.rs`. This is the same complete program as the public quickstart:

<!-- source: anki_forge/examples/target_api_basic.rs -->
```rust
use ankiforge::{BuildOptions, Note, Project};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut project = Project::new("spanish")?.default_deck("Spanish");
    project.add("es:hola", Note::basic("hola", "hello"))?;
    let output = project.build(BuildOptions::to("spanish.apkg"))?;
    println!("{}", output.artifact().path().display());
    Ok(())
}
```
<!-- /source -->

```sh
cargo fmt --check
cargo check
cargo run
```

Expect `spanish.apkg` in the application's working directory, with one note,
one Basic card (**hola → hello**) and no media. Check that the file exists;
import it into Anki when the user wants client verification. Export itself does
not need Anki. For a programmatic check, the output report exposes
`output.report().counts()` with `notes`, `cards` and `media`.

Keep namespace `spanish` and note key `es:hola` stable for later edits.
`BuildOptions::to(path)` produces persistent output that survives dropped handles.
`BuildOptions::temporary()` instead produces owned temporary output; retain its
artifact until readers finish or persist it explicitly. Do not report a temporary
path as the user's saved deck.

For additional tasks read the matching section of [workflows.md](workflows.md).

- [Rust quickstart](https://ankiforge.dev/docs/quickstart/)
- [Rust authoring guide](https://ankiforge.dev/docs/rust-guide/)
- [Rust API](https://ankiforge.dev/docs/rust-api/)
- [Output ownership](https://ankiforge.dev/docs/build-guarantees/)
- [Source installation guide](https://github.com/morehardy/anki-forge/blob/main/docs/installation.md)
