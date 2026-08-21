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

Normal templates may also declare a generation rule:

```yaml
generation_rule:
  kind: all
  fields: [prompt]
```

The field names in `generation_rule` are stable field keys. Kinds `all`, `any`,
and `anki_default` are supported. When `anki_default` cannot be represented by
one Anki card requirement, the build asks for an explicit `all` or `any` rule
instead of storing an approximation.

`required: true` rejects missing or empty note content. `optional: true` allows
the field to be omitted and lowers it as an empty value. A field cannot declare
both modes. At most one field may declare `sort: true`; without one, Anki uses
the first field.

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
