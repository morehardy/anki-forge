# Custom templates and template bundles

Anki Forge supports inline custom templates and directory-based external
template bundles. Both use the same Rust validation and card-planning path.

## Bundle layout

```text
my-template/
├── anki-template.yaml
├── front.html
├── back.html
├── browser-front.html
├── style.css
└── assets/
```

The manifest declares `format_version: template-bundle-v1`, one note type,
stable field/template keys, template file paths, optional CSS and browser
templates, an optional target deck, and optional assets with explicit
`export_as` names. See
`contracts/fixtures/template-bundle/custom-cloze/anki-template.yaml` for a
complete example.

Rust usage:

```rust
let mut project = Project::new("Languages").stable_id("languages");
project.import_template_bundle("./my-template")?;
project.add_note(
    Note::new("language-cloze")
        .stable_id("es:capital")
        .text("text", "{{c1::Madrid}} is in {{c2::Spain}}"),
)?;
let report = project.write_apkg("languages.apkg")?;
```

## Compatibility and migration

Existing ProductDocument v2 custom templates remain normal note types. Use
ProductDocument v3—or `NoteType::custom_cloze` in Rust/Python—when custom Cloze
semantics are required. A custom Cloze declares one field and one template; its
front must use `{{cloze:Display Field Name}}`.

Supported template expressions and limitations are documented in
`contracts/semantics/templates.md`. In particular, validation does not prove
HTML/CSS/JavaScript correctness and does not execute third-party add-on filters.
Unknown filters are retained as portability warnings.
