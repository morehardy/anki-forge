use ankiforge::{BuildOptions, Field, Media, Note, NoteType, Project, Template};
use ankiforge::note::Mask;
use ankiforge::update::{CompareOptions, RiskCode, RiskLevel, UpdatePolicy};
use std::error::Error;

fn project(include: bool, reduced: bool) -> Result<Project, Box<dyn Error>> {
    let mut project = Project::new("revival")?;
    project.add("keep", Note::basic("keep", "answer"))?;
    if include {
        let mut builder = NoteType::builder("custom").field(Field::new("front"))
            .template(Template::new("recognize").front("{{front}}").back("{{front}}"));
        if !reduced {
            builder = builder.field(Field::new("extra"))
                .template(Template::new("reverse").front("{{extra}}").back("{{front}}"));
        }
        let model = builder.build()?;
        let mut note = model.note().field("front", "term");
        if !reduced { note = note.field("extra", "definition"); }
        project.add("term", note)?;
        let mut io = Note::image_occlusion(Media::file("assets/occlusion.png")?)
            .mask(Mask::rect("a", 0, 0, 1, 1));
        if !reduced { io = io.mask(Mask::rect("b", 2, 0, 1, 1)); }
        project.add("diagram", io.build()?)?;
    }
    Ok(project)
}
fn main() -> Result<(), Box<dyn Error>> {
    project(true, false)?.build(BuildOptions::to("v1.apkg"))?;
    project(false, false)?.build(BuildOptions::to("v2.apkg").update_from("v1.apkg")
        .update_policy(UpdatePolicy::default().allow(RiskCode::NoteRemoved).allow(RiskCode::ModelRemoved)))?;
    let restored = project(true, true)?;
    let report = restored.compare(CompareOptions::against("v2.apkg"))?;
    assert_eq!(report.highest_risk(), Some(RiskLevel::High));
    for code in [RiskCode::FieldRemoved, RiskCode::TemplateRemoved, RiskCode::MaskRemoved] {
        assert!(report.findings().iter().any(|f| f.code() == code), "restored historical entities must retain structural risk: {code}");
    }
    assert!(!report.policy().allows_publication());
    let accepted = UpdatePolicy::default().allow(RiskCode::FieldRemoved).allow(RiskCode::TemplateRemoved).allow(RiskCode::MaskRemoved);
    restored.build(BuildOptions::to("v3.apkg").update_from("v2.apkg").update_policy(accepted))?;
    let final_project = project(true, false)?;
    let report = final_project.compare(CompareOptions::against("v3.apkg"))?;
    for code in [RiskCode::FieldAdded, RiskCode::TemplateAdded, RiskCode::MaskAdded] {
        assert!(report.findings().iter().any(|f| f.code() == code), "member revival must be reported: {code}");
    }
    let output = final_project.build(BuildOptions::temporary().update_from("v3.apkg")
        .update_policy(UpdatePolicy::default().allow(RiskCode::FieldAdded).allow(RiskCode::TemplateAdded)))?;
    let evidence = |path: &std::path::Path| {
        let mut zip = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
        serde_json::from_reader::<_, serde_json::Value>(zip.by_name("ankiforge-identity.json").unwrap()).unwrap()["identity"].clone()
    };
    let first = evidence(std::path::Path::new("v1.apkg"));
    let last = evidence(output.artifact().path());
    for group in ["fields", "templates"] {
        assert_eq!(first["models"]["custom"][group], last["models"]["custom"][group]);
    }
    assert_eq!(first["notes"]["diagram"]["masks"], last["notes"]["diagram"]["masks"]);
    assert_eq!(output.report().counts().cards, 5);
    Ok(())
}
