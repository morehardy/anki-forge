# Add Anki Forge to a Rust application

Canonical page: https://ankiforge.dev/docs/installation/
Language: en
Description: Integrate the Rust crate, configure Cargo dependencies and choose the next guide.

Use Rust 1.92 or later. In your existing Cargo application, install the public crate:

```sh
cargo add ankiforge@0.3.1
```

Normal APKG export needs no Anki installation. For a complete first program and `cargo run` instructions, follow the [Rust quickstart](https://ankiforge.dev/docs/quickstart/).

## Cargo dependencies

The public `ankiforge` crate includes Basic, Cloze, custom note types, owned media, Image Occlusion, comparison and prepared publication. Default features are empty; normal application code does not need repository-only features.

The crate is a native Rust library. Install your platform’s Rust linker and build tools as required by your Rust toolchain. See [verified environments](https://ankiforge.dev/docs/compatibility/) when choosing a deployment host. Rust 1.92 is the minimum requirement; a platform being listed does not guarantee every target or third-party toolchain configuration.

For JSON input, add caller-side parsing explicitly:

```sh
cargo add serde --features derive
cargo add serde_json
```

The [JSON to APKG recipe](https://ankiforge.dev/docs/json-to-anki/#rust) includes the complete parser, media and update workflow. The SDK does not provide a generic JSON or CSV converter.

## Choose the next guide

-   [Rust authoring guide](https://ankiforge.dev/docs/rust-guide/) for project and model design
-   [Rust API](https://ankiforge.dev/docs/rust-api/) for methods, reports and errors
-   [Media](https://ankiforge.dev/docs/media/) for owned images and audio
-   [Update and distribute](https://ankiforge.dev/docs/updates/) for baseline and publication conditions
-   [Source builds](https://ankiforge.dev/docs/development/) for repository development
