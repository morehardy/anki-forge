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
    cloze_conversion_retains_mask_history()?;
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


fn custom_cloze(text: &str) -> Result<Project, Box<dyn Error>> {
    let mut builder = ankiforge::NoteType::builder("image_occlusion").name("Custom cloze");
    for key in ["occlusion", "image", "header", "back_extra", "comments"] {
        builder = builder.field(ankiforge::Field::new(key));
    }
    let model = builder.cloze_field("occlusion")
        .template(ankiforge::Template::new("card").front("{{cloze:occlusion}}").back("{{cloze:occlusion}}"))
        .build()?;
    let mut project = Project::new("mask-history")?;
    project.add("diagram", model.note().field("occlusion", text).field("image", "")
        .field("header", "").field("back_extra", "").field("comments", ""))?;
    Ok(project)
}

fn cloze_conversion_retains_mask_history() -> Result<(), Box<dyn Error>> {
    let original = diagram(&["a".into(), "b".into()])?.build(BuildOptions::temporary())?;
    let original_id = identity(original.artifact().path());
    let custom = custom_cloze("{{c2::two}} {{c3::three}}")?;
    let comparison = custom.compare(CompareOptions::against(original.artifact().path()))?;
    assert!(comparison.findings().iter().any(|finding| finding.code() == RiskCode::ModelChanged));
    assert_eq!(comparison.findings().iter().filter(|finding| finding.code() == RiskCode::MaskRemoved).count(), 2);
    assert_eq!(comparison.findings().iter().filter(|finding| finding.code() == RiskCode::CardAdded).count(), 2);
    assert!(!comparison.findings().iter().any(|finding| finding.code() == RiskCode::CardRemoved), "mask removal already has its own risk category");
    fs::write("conversion-protected.apkg", "keep previous destination")?;
    let blocked = custom.build(BuildOptions::to("conversion-protected.apkg").update_from(original.artifact().path())).unwrap_err();
    assert_eq!(blocked.code(), "UPDATE.POLICY_BLOCKED");
    assert_eq!(fs::read_to_string("conversion-protected.apkg")?, "keep previous destination");
    let converted = custom.build(BuildOptions::temporary().update_from(original.artifact().path())
        .update_policy(UpdatePolicy::default().allow(RiskCode::MaskRemoved)))?;
    let converted_id = identity(converted.artifact().path());
    assert_eq!(converted_id["notes"]["diagram"]["mask_high_water"], 2);
    for (key, ordinal) in [("a", 0), ("b", 1)] {
        assert_eq!(converted_id["notes"]["diagram"]["masks"][key]["active"], false);
        assert_eq!(converted_id["notes"]["diagram"]["masks"][key]["ordinal"], ordinal);
    }
    assert_eq!(converted_id["notes"]["diagram"]["cards"], serde_json::json!({"cloze:2": 1, "cloze:3": 2}));
    let repeat = custom.build(BuildOptions::temporary().update_from(converted.artifact().path()))?;
    assert_eq!(converted_id, identity(repeat.artifact().path()), "unchanged custom cloze keeps retired IO history and revision");
    assert!(repeat.report().comparison().unwrap().findings().is_empty());

    // Historical masks must not suppress ordinary cloze card replacement risks.
    let next = custom_cloze("{{c3::three}} {{c4::four}}")?;
    let comparison = next.compare(CompareOptions::against(converted.artifact().path()))?;
    for code in [RiskCode::CardAdded, RiskCode::CardRemoved] {
        assert_eq!(comparison.findings().iter().filter(|finding| finding.code() == code).count(), 1);
    }
    assert!(!comparison.findings().iter().any(|finding| matches!(finding.code(), RiskCode::MaskAdded | RiskCode::MaskRemoved)));
    let next_output = next.build(BuildOptions::temporary().update_from(converted.artifact().path())
        .update_policy(UpdatePolicy::default().allow(RiskCode::CardRemoved)))?;

    // Returning to IO restores existing keys' ordinals and allocates new ones
    // after the historical high water, independently of authored order.
    let restored = diagram(&["b".into(), "c".into(), "a".into()])?;
    let comparison = restored.compare(CompareOptions::against(next_output.artifact().path()))?;
    assert!(comparison.findings().iter().any(|finding| finding.code() == RiskCode::ModelChanged));
    assert_eq!(comparison.findings().iter().filter(|finding| finding.code() == RiskCode::MaskAdded).count(), 3);
    assert_eq!(comparison.findings().iter().filter(|finding| finding.code() == RiskCode::CardRemoved).count(), 2);
    assert!(!comparison.findings().iter().any(|finding| finding.code() == RiskCode::CardAdded), "mask addition already has its own risk category");
    assert!(!comparison.policy().allows_publication(), "changing back to IO must not hide removed cloze cards");
    let restored_output = restored.build(BuildOptions::temporary().update_from(next_output.artifact().path())
        .update_policy(UpdatePolicy::default().allow(RiskCode::CardRemoved)))?;
    let restored_id = identity(restored_output.artifact().path());
    assert_eq!(restored_id["models"]["image_occlusion"]["id"], original_id["models"]["image_occlusion"]["id"]);
    assert_eq!(restored_id["notes"]["diagram"]["guid"], original_id["notes"]["diagram"]["guid"]);
    assert_eq!(restored_id["notes"]["diagram"]["mask_high_water"], 3);
    for (key, ordinal) in [("a", 0), ("b", 1), ("c", 2)] {
        assert_eq!(restored_id["notes"]["diagram"]["masks"][key]["active"], true);
        assert_eq!(restored_id["notes"]["diagram"]["masks"][key]["ordinal"], ordinal);
    }
    assert_eq!(restored_id["notes"]["diagram"]["cards"], serde_json::json!({"mask:a": 0, "mask:b": 1, "mask:c": 2}));
    let repeat = restored.build(BuildOptions::temporary().update_from(restored_output.artifact().path()))?;
    assert!(repeat.report().comparison().unwrap().findings().is_empty());
    assert_eq!(identity(repeat.artifact().path()), restored_id);
    Ok(())
}
