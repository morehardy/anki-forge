use anki_forge::product::{HelperDeclaration, ProductDocument};
use anki_forge::{normalize, NormalizationRequest};

fn main() -> anyhow::Result<()> {
    let lowering = ProductDocument::new("example-doc")
        .with_default_deck("Default")
        .with_basic("basic-main")
        .with_helper(
            "basic-main",
            HelperDeclaration::AnswerDivider {
                title: "Answer".into(),
            },
        )
        .add_basic_note("basic-main", "note-1", "Default", "front", "back")
        .lower()
        .map_err(|err| anyhow::anyhow!("lower product example: {:?}", err))?;

    let normalized = normalize(NormalizationRequest::new(lowering.authoring_document));
    println!("{}", anki_forge::to_authoring_canonical_json(&normalized)?);

    Ok(())
}
