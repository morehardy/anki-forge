# Custom note types

Define your own fields and card layout. This example builds one Japanese
vocabulary note and one recognition card, with stable field and template keys.

## Run the complete example

From the repository root with [Rust installed](installation.md#requirements):

```sh
cargo run --locked -p anki_forge --example target_api_custom_notetype
```

The output is `jp-core.apkg` in the current directory. Import it into Anki and
open **Japanese::Core** to see **食べる → to eat**.

<!-- source: anki_forge/examples/target_api_custom_notetype.rs -->
```rust
use anki_forge::prelude::*;

fn main() -> anyhow::Result<()> {
    let vocab = NoteType::custom("jp-vocab")
        .name("Japanese Vocabulary")
        .field(Field::new("Expression").key("expr").identity().sort())
        .field(Field::new("Meaning").key("meaning").required())
        .template(
            Template::new("Recognition")
                .key("recognition")
                .front("{{Expression}}")
                .back("{{FrontSide}}<hr id=\"answer\">{{Meaning}}")
                .generate_when(GenerationRule::all(["expr"])),
        )
        .identity(IdentityRecipe::fields(["expr"]));

    let mut project = Project::new("Japanese Core")
        .stable_id("jp-core")
        .default_deck("Japanese::Core");
    project.add_notetype(vocab)?;
    project.add_note(
        Note::new("jp-vocab")
            .stable_id("jp-vocab:taberu")
            .text("expr", "食べる")
            .text("meaning", "to eat"),
    )?;

    project.validate().ensure_success()?;
    project.write_apkg("jp-core.apkg")?.ensure_success()?;
    Ok(())
}
```
<!-- /source -->

## Read the declaration

| Declaration | Effect |
| --- | --- |
| `key("expr")` | Gives the field a stable key for authoring and identity rules |
| `required()` | Rejects missing or empty content |
| `optional()` | Permits omission and lowers the field as empty; cannot be combined with required |
| `sort()` | Selects the field used for sorting; at most one field may set it |
| `{{Expression}}` | References the field's display name in a template |
| `{{FrontSide}}` | Reuses the rendered front on the answer side |
| `GenerationRule::all(["expr"])` | Generates the template's card when all listed fields have content |
| `IdentityRecipe::fields(["expr"])` | Defines the fallback identity inputs when there is no explicit note ID |

`GenerationRule::any(...)` uses any listed field. The default rule is inferred
from the template; if it cannot be represented as one Anki requirement, declare
an explicit rule. Required/optional fields and card-generation rules solve different problems.

## Keep templates reusable

Inline declarations are useful for small applications. Use a
[template bundle](template-bundles.md) when HTML, CSS and assets should be edited
as separate files. The bundle loader and inline declarations use the same core validation.

For custom Cloze, select one cloze field and one template; extra fields are allowed.
Its front must contain the cloze filter for the selected display name, such as
`{{cloze:Sentence}}`. See the complete bundle example in the next guide.

For method details, see the [Rust API guide](rust-api.md#note-types-fields-and-templates).
