# Custom templates and template bundles

Keep field declarations, HTML, CSS and assets together in a reusable directory.
The loader uses the same validation and card planning as inline custom note types.

## Try a complete bundle

From the repository root with Rust 1.92 or later:

```sh
cargo run --locked -p anki_forge --example docs_workflow -- target/docs-examples
```

Open `target/docs-examples/template.apkg` in Anki. The example imports the
[complete custom Cloze bundle](../contracts/fixtures/template-bundle/custom-cloze/anki-template.yaml),
then exports one note and two cards in **Languages::Cloze**.

## Bundle layout

```text
my-template/
├── anki-template.yaml
├── front.html
├── back.html
├── browser-front.html
└── style.css
```

Copy the working example if you want to edit your own version:

```sh
cp -R contracts/fixtures/template-bundle/custom-cloze my-template
```

This is its complete manifest:

<!-- source: contracts/fixtures/template-bundle/custom-cloze/anki-template.yaml -->
```yaml
format_version: template-bundle-v1
note_type:
  id: language-cloze
  name: Language Cloze
  kind: cloze
  cloze_field: text
  fields:
    - key: text
      name: Sentence
      identity: true
      sort: true
      required: true
    - key: extra
      name: Extra
      optional: true
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

The [front](../contracts/fixtures/template-bundle/custom-cloze/front.html) uses
`{{cloze:Sentence}}`; the [back](../contracts/fixtures/template-bundle/custom-cloze/back.html)
also displays the optional Extra field. HTML uses display names, while
`cloze_field`, identity and generation rules use stable field keys.

## Import it into a Project

The [complete program](../anki_forge/examples/docs_workflow.rs) passes the fixture
directory to this function. Pass `Path::new("./my-template")` to use your copy.
`verify` is the example's package-count check.

<!-- source: anki_forge/examples/docs_workflow.rs#bundle -->
```rust
fn bundle(output: &Path, template_directory: &Path) -> anyhow::Result<()> {
    let mut project = Project::new("Languages").stable_id("docs-languages");
    project.import_template_bundle(template_directory)?;
    project.add_note(
        Note::new("language-cloze")
            .stable_id("es:capital")
            .text("text", "{{c1::Madrid}} is in {{c2::Spain}}.")
            .text("extra", "A city and its country."),
    )?;
    let report = project.write_apkg(output.join("template.apkg"))?;
    report.ensure_success()?;
    verify(&report, 1, 2, 0)?;
    Ok(())
}
```
<!-- /source -->

## Fields and generation rules

`required: true` rejects missing or empty content. `optional: true` permits an
omitted field and lowers it as empty. A field cannot declare both. At most one
field may declare `sort: true`; otherwise Anki uses the first field.

Normal templates can declare `generation_rule` with `kind: all` or `kind: any`
and a `fields` list of stable keys. `anki_default` asks the core to infer the
requirement. If it cannot represent the template accurately as one Anki card
requirement, provide an explicit rule instead.

## Compatibility and migration

A custom Cloze selects **one cloze field and one card template**. Additional
fields, such as Extra, are allowed. Its front must use the cloze filter for the
selected field's display name. One template can generate several cards when the
note uses distinct cloze numbers.

ProductDocument v2 custom templates remain normal note types. Repository tooling
that needs custom Cloze uses v3; Rust's `NoteType::custom_cloze` and the native
bindings expose custom Cloze directly. Normal consumers do not need ProductDocument.

The [template semantics](../contracts/semantics/templates.md) describe supported
expressions and filters. Validation does not execute third-party add-on filters
or certify HTML/CSS/JavaScript behavior; unknown filters are portability warnings.

## Assets and failures

Bundles may declare CSS, browser templates, target decks and assets with explicit
export filenames. Every declared file must exist; paths and symlinks must stay
inside the bundle. Keep media references synchronized with export filenames.
See the [normal bundle with assets](../contracts/fixtures/template-bundle/custom-normal/anki-template.yaml)
and [media guide](media.md).

For missing files or invalid field references, check the reported path and
[troubleshooting](troubleshooting.md#media-and-templates). Contract fixture checks,
packaging and embedded-resource regeneration are covered separately in
[template maintenance](template-maintenance.md).
