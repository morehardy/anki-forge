# Template bundles

A bundle is a reusable model declaration, template text, CSS and explicit media
closure. `NoteType::from_bundle(directory)` validates the complete bundle and
takes owned snapshots before returning the same immutable `NoteType` used by
inline authoring. It does not partially modify a project.

## Run the bundled example

```sh
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
```

The workflow loads the [custom Cloze fixture](../contracts/fixtures/template-bundle/custom-cloze/anki-template.yaml)
and writes `target/docs-examples/template.apkg`. Its template targets
**Languages::Cloze**. One note generates two cards.

## Manifest and template references

The manifest filename is `anki-template.yaml`. Only the new format is accepted:

<!-- source: contracts/fixtures/template-bundle/custom-cloze/anki-template.yaml -->
```yaml
format_version: template-bundle-v2
note_type:
  key: language-cloze
  name: Language Cloze
  cloze_field: text
  fields:
    - key: text
      name: Sentence
      sort: true
      required: true
    - key: extra
      name: Extra
  templates:
    - key: cloze
      name: Cloze
      front_file: front.html
      back_file: back.html
      browser_front_file: browser-front.html
      target_deck: Languages::Cloze
css_file: style.css
```
<!-- /source -->

`note_type.key`, field keys and template keys are required stable identifiers.
`name` is optional and defaults to the corresponding key. `cloze_field` selects
a Cloze model; no separate kind declaration is needed. A field without
`required: true` may be omitted. Unknown manifest properties are rejected.

Every template uses **field keys**, including front, back and browser variants:
`{{cloze:text}}`, `{{extra}}`, and conditional sections such as `{{#extra}}`.
Display names such as Sentence and Extra are generated for Anki after binding.
The loader preserves original file paths and template byte ranges in errors.

## Load and reuse a model

<!-- source: anki_forge/examples/docs_workflow.rs#bundle -->
```rust
fn bundle(output: &Path, template_directory: &Path) -> anyhow::Result<()> {
    let model = NoteType::from_bundle(template_directory)?;
    let mut project = Project::new("docs-languages")?;
    project.add(
        "capital",
        model
            .note()
            .field("text", "{{c1::Madrid}} is in {{c2::Spain}}.")
            .field("extra", "A city and its country."),
    )?;
    let built = project.build(BuildOptions::to(output.join("template.apkg")))?;
    verify(&built, 1, 2, 0)?;
    Ok(())
}
```
<!-- /source -->

For an independent application, this full example expects the same bundle copied
to `fixtures/template-bundle`:

```rust
use ankiforge::{BuildOptions, NoteType, Project};

fn main() -> anyhow::Result<()> {
    let model = NoteType::from_bundle("fixtures/template-bundle")?;
    let mut project = Project::new("bundle-example")?;
    project.add("capital", model.note()
        .field("text", "{{c1::Madrid}} is in {{c2::Spain}}.")
        .field("extra", "A city and its country."))?;
    let output = project.build(BuildOptions::to("bundle-example.apkg"))?;
    assert_eq!(output.report().counts().cards, 2);
    Ok(())
}
```

The returned model is independent of the source directory. It can be shared with
another project or used after that directory is removed. A load failure returns
no partial model and cleans up its unshared snapshots.

## Assets and budgets

Declare every raw template/CSS/script asset explicitly. Each `assets` entry uses
a path relative to the bundle and a fixed export name:

```yaml
assets:
  - path: assets/badge.png
    export_as: badge.png
  - path: assets/labels.woff
    export_as: labels.woff
```

Then templates may use `<img src="badge.png">` and CSS may use
`url('labels.woff')`. The `css_file` property supplies the CSS source file; assets
are not inferred from arbitrary HTML/CSS/JavaScript. Declared assets remain in the
APKG even if no static reference is detected. Export names obey the same
portable-name and Unicode/case collision rules as [Media](media.md).
The complete export name is limited to 255 UTF-8 bytes, including its extension.
The JSON Schema checks syntax and character count; `NoteType::from_bundle`
also enforces the byte budget, file contents and references before returning a
validated model. A successful schema check alone does not guarantee loading.

Absolute paths, traversal escapes and symlinks outside the bundle are rejected.
The manifest is limited to 256 KiB; each template/CSS text file is limited to
2 MiB and must be UTF-8. Asset snapshots use the default media budget, or a
caller-supplied `MediaLimits` through `NoteType::from_bundle_with_limits`.
Both preflight lengths and actual bytes read are checked.

Bundle errors retain kind/code, source path, byte offset where applicable, and
the underlying schema, I/O, YAML, UTF-8 or media error. Model completion validates
schema; adding notes separately checks project-level conflicts. See
[custom note types](custom-notetypes.md) and [troubleshooting](troubleshooting.md).
