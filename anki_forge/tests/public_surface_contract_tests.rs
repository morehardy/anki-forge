//! Consumer contracts deliberately compiled outside the repository workspace.

use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Consumer {
    root: tempfile::TempDir,
}

impl Consumer {
    fn new() -> Self {
        static NEXT_CONSUMER_ID: AtomicUsize = AtomicUsize::new(0);
        // Cargo releases its target lock before `run` launches the binary.
        // Unique names keep parallel probes from replacing each other's program
        // while still sharing the compiled dependency cache.
        let consumer_name = format!(
            "ankiforge_public_consumer_{}_{}",
            std::process::id(),
            NEXT_CONSUMER_ID.fetch_add(1, Ordering::Relaxed)
        );
        let root = tempfile::Builder::new()
            .prefix("ankiforge-public-consumer-")
            .tempdir()
            .unwrap();
        fs::create_dir(root.path().join("src")).unwrap();
        fs::create_dir(root.path().join("assets")).unwrap();
        for name in [
            "pixel.png",
            "silence.wav",
            "labels.woff",
            "occlusion.png",
            "rotated.jpg",
        ] {
            fs::copy(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/fixtures/public-api")
                    .join(name),
                root.path().join("assets").join(name),
            )
            .unwrap();
        }
        let crate_path = serde_json::to_string(env!("CARGO_MANIFEST_DIR")).unwrap();
        fs::write(
            root.path().join("Cargo.toml"),
            format!(
                r#"[package]
name = "{consumer_name}"
version = "0.0.0"
edition = "2021"

[workspace]

[dependencies]
ankiforge = {{ path = {crate_path}, default-features = false }}
anyhow = "1"
serde_json = "1"
zip = {{ version = "2.2", default-features = false, features = ["deflate"] }}
zstd = "0.13"
rusqlite = {{ version = "0.32", features = ["bundled"] }}
prost = "0.13"
blake3 = "1"
"#
            ),
        )
        .unwrap();
        Self { root }
    }

    fn invoke(&self, source: &str, action: &str) -> std::process::Output {
        fs::write(self.root.path().join("src/main.rs"), source).unwrap();
        Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
            .args([action, "--quiet", "--offline"])
            .current_dir(self.root.path())
            .env(
                "CARGO_TARGET_DIR",
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/public-consumer"),
            )
            .output()
            .unwrap()
    }

    fn run(&self, source: &str) {
        let output = self.invoke(source, "run");
        assert!(
            output.status.success(),
            "consumer failed:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn rejects(&self, source: &str, expected: &[&str]) {
        let output = self.invoke(source, "check");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success(),
            "invalid consumer compiled: {source}"
        );
        for diagnostic in expected {
            assert!(
                stderr.contains(diagnostic),
                "missing {diagnostic}:\n{stderr}"
            );
        }
    }
}

#[test]
fn release_workflow_consumer_uses_the_current_public_api() {
    let workflow = include_str!("../../.github/workflows/rust-crate-release.yml");
    let source = workflow
        .split_once("<<'RS'\n")
        .expect("release verification Rust heredoc")
        .1
        .lines()
        .take_while(|line| line.trim() != "RS")
        .collect::<Vec<_>>()
        .join("\n");
    assert!(source.contains("fn main()"));
    Consumer::new().run(&source);
}

#[test]
fn schema_consumer_contract() {
    let consumer = Consumer::new();
    // A successful control distinguishes compiler failures from missing dependencies.
    consumer.run("fn main() { assert!(!ankiforge::facade_api_version().is_empty()); }");
    consumer.run(
        r#"
#![deny(unused_must_use)]
use std::borrow::Cow;
use ankiforge::{Field, NoteType, Template};
use ankiforge::schema::{FieldKey, GenerationRule, NoteTypeBuilder, TemplateKey};

fn model(field: impl Into<FieldKey>, template: impl Into<TemplateKey>) -> NoteTypeBuilder {
    NoteType::builder("vocab")
        .name("词汇")
        .field(Field::new(field).name("正面"))
        .field(Field::new("back").name("背面"))
        .template(Template::new(template).name("识别")
            .front("{{#front}}{{text:front}}{{/front}}")
            .back("{{FrontSide}}<hr>{{back}}")
            .generate_when(GenerationRule::all(["front"])))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let front = FieldKey::from("front");
    let card = TemplateKey::from("recognition");
    let field_string = String::from("front");
    let template_string = String::from("recognition");
    let models = [
        model("front", "recognition").build()?,
        model(field_string.clone(), template_string.clone()).build()?,
        model(&field_string, &template_string).build()?,
        model(Cow::Borrowed("front"), Cow::Borrowed("recognition")).build()?,
        model(Cow::Owned::<str>(field_string), Cow::Owned::<str>(template_string)).build()?,
        model(front.clone(), card.clone()).build()?,
        model(&front, &card).build()?,
    ];
    for built in models {
        assert_eq!(built.key(), "vocab");
        assert_eq!(built.display_name(), "词汇");
        assert_eq!(built.fields()[0].key(), &front);
        assert_eq!(built.fields()[0].display_name(), "正面");
        assert_eq!(built.templates()[0].key(), &card);
    }
    assert_eq!(front.as_ref(), "front");
    assert_eq!(front.to_string(), "front");
    assert_eq!(card.as_str(), "recognition");
    let _all = GenerationRule::all([&front]);
    let _any = GenerationRule::any([front.clone()]);
    let _cloze = NoteType::builder("cloze-custom")
        .field(Field::new(&front).name("正文"))
        .cloze_field(&front)
        .template(Template::new(&card).front("{{cloze:front}}").back("{{cloze:front}}"))
        .build()?;
    Ok(())
}
"#,
    );
    for statement in [
        "Field::new(TemplateKey::from(\"card\"))",
        "Template::new(FieldKey::from(\"front\"))",
        "GenerationRule::all([TemplateKey::from(\"card\")])",
        "GenerationRule::any([TemplateKey::from(\"card\")])",
        "NoteType::builder(\"vocab\").cloze_field(TemplateKey::from(\"card\"))",
    ] {
        consumer.rejects(
            &format!("use ankiforge::{{Field, Template, NoteType, schema::{{FieldKey, TemplateKey, GenerationRule}}}}; fn main() {{ let _ = {statement}; }}"),
            &["E0277", "FieldKey", "TemplateKey"],
        );
    }
    consumer.rejects(
        "#![deny(unused_must_use)]\nfn main() { ankiforge::NoteType::builder(\"vocab\"); }",
        &["unused", "NoteTypeBuilder", "unused_must_use"],
    );
}

#[test]
fn schema_errors_keep_machine_codes_and_original_source_locations() {
    Consumer::new().run(
        r#"
use ankiforge::{Field, NoteType, Template};
use ankiforge::schema::{NoteTypeBuilder, SchemaError, SchemaErrorKind, TemplateSide};
use anyhow::Context;

fn model(key: &str) -> NoteTypeBuilder {
    NoteType::builder("vocab")
        .field(Field::new(key).name("正面"))
        .template(Template::new("recognition").front("{{front}}"))
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    fn standard_error<T: std::error::Error + Send + Sync + 'static>() {}
    standard_error::<SchemaError>();
    for key in ["", "   ", " front", "a:b", "FrontSide"] {
        let error = model(key).build().unwrap_err();
        assert_eq!(error.kind(), SchemaErrorKind::InvalidKey);
        assert_eq!(error.code(), "SCHEMA.KEY_INVALID");
        let wrapped = Err::<(), _>(error).context("loading vocabulary").unwrap_err();
        let original = wrapped.downcast_ref::<SchemaError>().unwrap();
        assert_eq!(original.kind(), SchemaErrorKind::InvalidKey);
        assert_eq!(original.code(), "SCHEMA.KEY_INVALID");
    }
    let source = "前缀 {{ text: missing }} {{! missing}}";
    let error = NoteType::builder("vocab")
        .field(Field::new("front").name("正面"))
        .template(Template::new("recognition").front("{{front}}")
            .browser_back(source))
        .build().unwrap_err();
    assert_eq!(error.kind(), SchemaErrorKind::InvalidTemplate);
    assert_eq!(error.code(), "TEMPLATE.RENDER_FIELD_UNKNOWN");
    let location = error.location().unwrap();
    assert_eq!(location.template.as_str(), "recognition");
    assert_eq!(location.side, TemplateSide::BrowserBack);
    assert_eq!(location.byte_range, 16..23);
    assert_eq!(&source[location.byte_range.clone()], "missing");
    let error = NoteType::builder("vocab")
        .field(Field::new("front").name("正面"))
        .template(Template::new("recognition").front("{{正面}}"))
        .build().unwrap_err();
    assert_eq!(error.code(), "TEMPLATE.RENDER_FIELD_UNKNOWN");
    let error = model("front").field(Field::new("front").name("另一个名字"))
        .build().unwrap_err();
    assert_eq!(error.kind(), SchemaErrorKind::Duplicate);
    // A failed declaration does not poison the independent valid declaration.
    let valid = model("front").build()?;
    assert_eq!(valid.fields().len(), 1);
    Ok(())
}
"#,
    );
}

#[test]
fn every_schema_key_entry_accepts_owned_and_borrowed_values() {
    Consumer::new().run(
        r#"
#![deny(unused_must_use)]
use std::borrow::Cow;
use ankiforge::{Field, NoteType, Template};
use ankiforge::schema::{FieldKey, GenerationRule, TemplateKey};

fn fields<K: Into<FieldKey> + Clone>(key: K) -> Result<(), Box<dyn std::error::Error>> {
    for rule in [GenerationRule::all([key.clone()]), GenerationRule::any([key.clone()])] {
        let model = NoteType::builder("normal")
            .field(Field::new(key.clone()).name("正文"))
            .template(Template::new("card").front("{{text}}").generate_when(rule))
            .build()?;
        assert_eq!(model.fields()[0].key().as_str(), "text");
    }
    let model = NoteType::builder("cloze").cloze_field(key.clone())
        .field(Field::new(key).name("正文"))
        .template(Template::new("card").front("{{cloze:text}}")).build()?;
    assert_eq!(model.cloze_field().unwrap().as_str(), "text");
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let string = String::from("text");
    let key = FieldKey::from("text");
    fields("text")?;
    fields(string.clone())?;
    fields(&string)?;
    fields(Cow::Borrowed("text"))?;
    fields(Cow::Owned::<str>(string))?;
    fields(key.clone())?;
    fields(&key)?;
    fields(&key)?;
    // Conversion itself neither validates nor normalizes symbols.
    assert_eq!(FieldKey::from(" ").as_str(), " ");
    assert_eq!(TemplateKey::from("识别").as_ref(), "识别");
    let _ = NoteType::builder("abandoned");
    drop(NoteType::builder("abandoned"));
    Ok(())
}
"#,
    );
}

#[test]
fn owned_media_snapshot_contract() {
    Consumer::new().run(include_str!("consumers/media_snapshot.rs"));
}

#[test]
fn schema_assets_use_portable_names_and_complete_ownership() {
    Consumer::new().run(include_str!("consumers/schema_assets.rs"));
}

#[test]
fn notes_own_their_models_and_typed_content() {
    let consumer = Consumer::new();
    consumer.run(include_str!("consumers/owned_note.rs"));
    consumer.rejects(
        "use ankiforge::{Note, schema::TemplateKey}; fn main() { let _ = Note::basic(\"q\", \"a\").field(TemplateKey::from(\"card\"), \"value\"); }",
        &["E0277", "FieldKey", "TemplateKey"],
    );
    consumer.rejects(
        "#![deny(unused_must_use)]\nfn main() { ankiforge::Note::basic(\"q\", \"a\"); }",
        &["unused", "Note", "unused_must_use"],
    );
}

#[test]
fn project_collects_note_dependencies_atomically() {
    Consumer::new().run(include_str!("consumers/project_add.rs"));
}

#[test]
fn project_builds_real_anki_content_from_owned_values() {
    Consumer::new().run(include_str!("consumers/build_create.rs"));
}

#[test]
fn build_errors_preserve_codes_sources_and_publication_facts() {
    Consumer::new().run(include_str!("consumers/build_failures.rs"));
}

#[test]
fn bundles_own_complete_asset_closures_and_build_like_native_models() {
    Consumer::new().run(include_str!("consumers/template_bundle.rs"));
}

#[test]
fn bundle_failures_keep_sources_and_enforce_input_boundaries() {
    Consumer::new().run(include_str!("consumers/bundle_errors.rs"));
}

#[test]
fn typed_images_encode_portable_names_as_url_paths() {
    Consumer::new().run(include_str!("consumers/media_references.rs"));
}

#[test]
fn packages_carry_complete_identity_evidence_and_isolate_namespaces() {
    Consumer::new().run(include_str!("consumers/identity_evidence.rs"));
}

#[test]
fn updates_keep_identity_through_renaming_reordering_and_content_changes() {
    Consumer::new().run(include_str!("consumers/update_identity.rs"));
}

#[test]
fn updates_require_verified_evidence_and_enforce_both_inspection_budgets() {
    Consumer::new().run(include_str!("consumers/update_evidence_errors.rs"));
}

#[test]
fn comparisons_report_blocked_risks_and_builds_apply_the_same_typed_policy() {
    Consumer::new().run(include_str!("consumers/update_policy.rs"));
}

#[test]
fn updates_keep_mask_history_and_detect_card_replacement_at_equal_counts() {
    Consumer::new().run(include_str!("consumers/update_cards.rs"));
}

#[test]
fn reviving_retired_entities_preserves_both_identity_and_structural_risk() {
    Consumer::new().run(include_str!("consumers/update_revival.rs"));
}

#[test]
fn updates_preserve_implicit_sort_order_and_flag_explicit_sort_changes() {
    Consumer::new().run(include_str!("consumers/update_sort.rs"));
}

#[test]
fn image_occlusion_keeps_masks_structured_and_renders_distinct_cards() {
    let consumer = Consumer::new();
    consumer.run(include_str!("consumers/image_occlusion.rs"));
    consumer.rejects(
        "#![deny(unused_must_use)]\nfn main() { let image = ankiforge::Media::file(\"assets/pixel.png\").unwrap(); ankiforge::Note::image_occlusion(image); }",
        &["unused", "ImageOcclusionBuilder", "unused_must_use"],
    );
}

#[test]
fn every_uncommitted_value_warns_when_discarded_but_explicit_drop_is_allowed() {
    let consumer = Consumer::new();
    consumer.run("fn main() { let _ = ankiforge::BuildOptions::temporary(); drop(ankiforge::Note::basic(\"front\", \"back\")); }");
    for (expression, type_name) in [
        ("ankiforge::Field::new(\"front\")", "Field"),
        ("ankiforge::Template::new(\"card\")", "Template"),
        ("ankiforge::Content::text(\"hello\")", "Content"),
        ("ankiforge::Media::bytes(vec![1], \"application/octet-stream\").unwrap()", "Media"),
        ("ankiforge::NoteType::builder(\"model\").field(ankiforge::Field::new(\"front\")).template(ankiforge::Template::new(\"card\").front(\"{{front}}\")).build().unwrap()", "NoteType"),
        ("ankiforge::BuildOptions::temporary()", "BuildOptions"),
        ("ankiforge::update::CompareOptions::against(\"baseline.apkg\")", "CompareOptions"),
        ("ankiforge::update::UpdatePolicy::default()", "UpdatePolicy"),
    ] {
        consumer.rejects(&format!("#![deny(unused_must_use)]\nfn main() {{ {expression}; }}"),
            &["unused_must_use", type_name]);
        consumer.run(&format!("#![deny(unused_must_use)]\nfn main() {{ let _ = {expression}; }}"));
    }
    consumer.rejects("#![deny(unused_must_use)]\nfn main() { ankiforge::Project::new(\"course\").unwrap().name(\"New name\"); }", &["unused_must_use", "name"]);
    consumer.rejects("#![deny(unused_must_use)]\nfn main() { ankiforge::Project::new(\"course\").unwrap().default_deck(\"New deck\"); }", &["unused_must_use", "default_deck"]);
}
