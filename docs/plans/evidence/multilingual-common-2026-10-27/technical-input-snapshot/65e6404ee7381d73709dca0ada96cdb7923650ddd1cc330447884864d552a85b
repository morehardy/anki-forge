use ankiforge::{BuildOptions, Note, Project};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut project = Project::new("spanish")?.default_deck("Spanish");
    project.add("es:hola", Note::basic("hola", "hello"))?;
    project.build(BuildOptions::to("spanish-original.apkg"))?;

    let mut next = Project::new("spanish")?.default_deck("Spanish");
    next.add("es:hola", Note::basic("hola", "hello; hi"))?;
    let prepared = next.prepare_publication(
        BuildOptions::to("spanish-reviewed.apkg").update_from("spanish-original.apkg"),
    )?;
    println!("{:?}", prepared.report().comparison());
    if prepared
        .report()
        .comparison()
        .is_some_and(|report| report.policy().allows_publication())
    {
        let output = prepared.publish()?;
        println!("{}", output.artifact().path().display());
    }
    Ok(())
}
