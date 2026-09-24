use ankiforge::{BuildOptions, Media, Note, Project};
use ankiforge::note::Mask;
use ankiforge::update::{CompareOptions, RiskCode, RiskLevel, UpdatePolicy};
use std::{error::Error, fs};

fn diagram(keys: &[String]) -> Result<Project, Box<dyn Error>> {
    let mut builder = Note::image_occlusion(Media::file("assets/occlusion.png")?);
    for key in keys { builder = builder.mask(Mask::rect(key, 0, 0, 1, 1)); }
    let mut project = Project::new("mask-history")?;
    project.add("diagram", builder.build()?)?;
    Ok(project)
}
fn identity(path: &std::path::Path) -> serde_json::Value {
    let mut archive = zip::ZipArchive::new(fs::File::open(path).unwrap()).unwrap();
    serde_json::from_reader::<_, serde_json::Value>(archive.by_name("ankiforge-identity.json").unwrap()).unwrap()["identity"].clone()
}
fn main() -> Result<(), Box<dyn Error>> {
    let old = diagram(&["a".into(), "b".into()])?.build(BuildOptions::to("io1.apkg"))?;
    let next = diagram(&["b".into(), "c".into()])?;
    assert!(next.build(BuildOptions::temporary().update_from(old.artifact().path())).is_err());
    let accepted = next.build(BuildOptions::to("io2.apkg").update_from("io1.apkg")
        .update_policy(UpdatePolicy::default().allow(RiskCode::MaskRemoved)))?;
    let id = identity(accepted.artifact().path());
    assert_eq!(id["notes"]["diagram"]["mask_high_water"], 3);
    assert_eq!(id["notes"]["diagram"]["masks"]["a"]["active"], false);
    assert_eq!(id["notes"]["diagram"]["masks"]["b"]["ordinal"], 1);
    assert_eq!(id["notes"]["diagram"]["masks"]["c"]["ordinal"], 2);
    let revived = diagram(&["c".into(), "b".into(), "a".into()])?.build(BuildOptions::temporary().update_from("io2.apkg"))?;
    let id = identity(revived.artifact().path());
    assert_eq!(id["notes"]["diagram"]["mask_high_water"], 3);
    assert_eq!(id["notes"]["diagram"]["masks"]["a"]["ordinal"], 0);
    assert_eq!(id["notes"]["diagram"]["masks"]["a"]["active"], true);
    assert_eq!(revived.report().counts().cards, 3);
    let full = diagram(&(0..500).map(|i| format!("m{i}")).collect::<Vec<_>>())?.build(BuildOptions::to("full.apkg"))?;
    assert_eq!(full.report().counts().cards, 500);
    let error = diagram(&["m499".into(), "new-mask".into()])?.build(BuildOptions::temporary().update_from("full.apkg")).unwrap_err();
    assert_eq!(error.code(), "NOTE.IO_ORDINAL_EXHAUSTED", "retired ordinals cannot be reused for different keys");

    let mut before = Project::new("cloze-cards")?;
    before.add("fact", Note::cloze("{{c1::one}} and {{c2::two}}"))?;
    before.build(BuildOptions::to("cloze1.apkg"))?;
    let mut after = Project::new("cloze-cards")?;
    after.add("fact", Note::cloze("{{c2::two}} and {{c3::three}}"))?;
    let comparison = after.compare(CompareOptions::against("cloze1.apkg"))?;
    assert_eq!(comparison.highest_risk(), Some(RiskLevel::High));
    assert_eq!(comparison.findings().iter().filter(|f| f.code() == RiskCode::CardRemoved).count(), 1,
        "equal card counts can still replace one scheduled card with another");
    assert_eq!(comparison.findings().iter().filter(|f| f.code() == RiskCode::CardAdded).count(), 1);
    assert!(!comparison.policy().allows_publication());
    for text in ["{{c501::outside supported range}}", "{{c65537::cannot fit Anki card ordinal}}"] {
        let mut invalid = Project::new("invalid-cloze")?;
        invalid.add("fact", Note::cloze(text))?;
        assert_eq!(invalid.build(BuildOptions::temporary()).unwrap_err().code(), "NOTE.CLOZE_ORDINAL_EXCEEDED");
    }
    Ok(())
}
