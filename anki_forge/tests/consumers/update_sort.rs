use ankiforge::{BuildOptions, Field, NoteType, Project, Template};
use ankiforge::update::{CompareOptions, RiskCode, RiskLevel};
use std::error::Error;

fn project(reorder: bool, sort_back: bool) -> Result<Project, Box<dyn Error>> {
    let front = Field::new("front");
    let back = if sort_back { Field::new("back").sort() } else { Field::new("back") };
    let fields = if reorder { [back, front] } else { [front, back] };
    let mut builder = NoteType::builder("model");
    for field in fields { builder = builder.field(field); }
    let model = builder.template(Template::new("card").front("{{front}}").back("{{back}}")).build()?;
    let mut project = Project::new("sort-order")?;
    project.add("one", model.note().field("front", if sort_back { "new content" } else { "old content" }).field("back", "back"))?;
    Ok(project)
}
fn main() -> Result<(), Box<dyn Error>> {
    project(false, false)?.build(BuildOptions::to("v1.apkg"))?;
    let reorder = project(true, false)?.compare(CompareOptions::against("v1.apkg"))?;
    assert!(reorder.findings().is_empty(), "a declaration reorder must retain implicit baseline sort semantics");
    let changed = project(false, true)?.compare(CompareOptions::against("v1.apkg"))?;
    let sort = changed.findings().iter().find(|f| f.code() == RiskCode::SortFieldChanged).unwrap();
    assert_eq!(sort.level(), RiskLevel::High);
    assert!(!changed.policy().allows_publication());
    assert!(project(false, true)?.build(BuildOptions::temporary().update_from("v1.apkg")).is_err());
    Ok(())
}
