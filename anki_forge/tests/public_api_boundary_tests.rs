use ankiforge::{build, diagnostics, media, note, schema, update};
use ankiforge::{
    BuildOptions, BuildOutput, Content, Field, Media, Note, NoteType, Project, Template,
};

#[test]
fn public_errors_are_nameable_standard_errors() {
    fn standard<T: std::error::Error + Send + Sync + 'static>() {}
    standard::<schema::SchemaError>();
    standard::<schema::TemplateBundleError>();
    standard::<media::MediaError>();
    standard::<note::ImageOcclusionError>();
    standard::<note::AddError>();
    standard::<update::PolicyError>();
    standard::<update::CompareError>();
    standard::<build::BuildError>();
    standard::<build::PersistError>();
}

#[test]
fn owned_authoring_values_and_reports_can_cross_threads() {
    fn share<T: Send + Sync + 'static>() {}
    share::<Project>();
    share::<Note>();
    share::<NoteType>();
    share::<Content>();
    share::<Media>();
    share::<Field>();
    share::<Template>();
    share::<BuildOptions>();
    share::<BuildOutput>();
    share::<build::BuildReport>();
    share::<build::ApkgArtifact>();
    share::<diagnostics::Diagnostic>();
    share::<update::ComparisonReport>();
    share::<build::json::BuildSnapshot>();
}

#[test]
fn public_domain_types_support_the_complete_success_path() {
    let model: NoteType = NoteType::builder("vocab")
        .field(Field::new(schema::FieldKey::from("front")))
        .template(Template::new(schema::TemplateKey::from("card")).front("{{front}}"))
        .build()
        .unwrap();
    let mut project: Project = Project::new("public-domains").unwrap();
    project
        .add("word", model.note().field("front", Content::text("hello")))
        .unwrap();
    let output: BuildOutput = project.build(BuildOptions::temporary()).unwrap();
    let report: &build::BuildReport = output.report();
    let _: build::json::ReportSnapshot = report.snapshot();
    let _: build::json::BuildSnapshot = output.snapshot();
    let comparison: update::ComparisonReport = project
        .compare(update::CompareOptions::against(output.artifact().path()))
        .unwrap();
    let _: update::json::ComparisonSnapshot = comparison.snapshot();
    assert!(comparison.policy().allows_publication());
}
