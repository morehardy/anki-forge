use crate::{
    build::{identity::IdentityEnvelope, BuildError, BuildOptions, BuildOutput},
    note::Mask,
    schema::GenerationRule,
    update::{RiskLevel, UpdatePolicy},
    Content, Field, Media, Note, NoteType, Project, Template,
};
use serde_json::Value;
use std::io::Cursor;

thread_local! {
    static FIXED_TIMESTAMP: std::cell::Cell<Option<i64>> = const { std::cell::Cell::new(None) };
}

pub(super) fn timestamp() -> Option<i64> {
    FIXED_TIMESTAMP.with(std::cell::Cell::get)
}

fn with_fixed_time<R>(action: impl FnOnce() -> R) -> R {
    struct Restore(Option<i64>);
    impl Drop for Restore {
        fn drop(&mut self) {
            FIXED_TIMESTAMP.with(|slot| slot.set(self.0));
        }
    }
    let _restore = Restore(FIXED_TIMESTAMP.with(|slot| slot.replace(Some(1_800_000_000))));
    action()
}

fn png() -> Vec<u8> {
    let mut bytes = Cursor::new(Vec::new());
    image::DynamicImage::new_rgba8(16, 16)
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap();
    bytes.into_inner()
}

fn model(reordered: bool) -> NoteType {
    let mut fields = vec![
        Field::new("q").name("Question").sort(),
        Field::new("a").name("Answer"),
    ];
    let mut templates = vec![
        Template::new("z-all")
            .front("{{q}}{{a}}")
            .back("{{FrontSide}}")
            .generate_when(GenerationRule::all(["q", "a"]))
            .target_deck("Rules::All"),
        Template::new("a-any")
            .front("{{q}}{{a}}")
            .back("{{a}}")
            .generate_when(GenerationRule::any(["q", "a"])),
        Template::new("m-auto")
            .front("{{addon_filter:q}}")
            .back("{{a}}")
            .browser_front("{{q}}"),
        Template::new("s-static").front("Static front"),
    ];
    if reordered {
        fields.reverse();
        templates.reverse();
    }
    let mut model = NoteType::builder("dataflow-rules").name("Dataflow rules");
    for field in fields {
        model = model.field(field);
    }
    for template in templates {
        model = model.template(template);
    }
    model.build().unwrap()
}

fn project(updated: bool) -> Project {
    let model = model(updated);
    let mut project = Project::new("dataflow-structural").unwrap();
    project
        .add(
            "one",
            model
                .note()
                .field("q", Content::html("<b>中文 &amp; q</b>"))
                .field("a", "")
                .tag("中文")
                .deck("Rules::Default"),
        )
        .unwrap();
    project
        .add("two", model.note().field("q", "q").field("a", "answer"))
        .unwrap();
    project
        .add("empty", model.note().field("q", "").field("a", ""))
        .unwrap();
    project
        .add(
            "wide",
            Note::basic(
                Content::html("<b>wide 日本語 &amp;</b>".repeat(2048)),
                Content::html("<i>answer</i>"),
            ),
        )
        .unwrap();
    project
        .add(
            "cloze",
            Note::cloze(if updated {
                "{{c1::first}} {{c3::third}}"
            } else {
                "{{c1::first}} {{c4::fourth}}"
            }),
        )
        .unwrap();
    let custom_cloze = NoteType::builder("dataflow-custom-cloze")
        .field(Field::new("expr").name("Expression"))
        .cloze_field("expr")
        .template(
            Template::new("cloze")
                .front("{{cloze:expr}}")
                .back("{{cloze:expr}}"),
        )
        .build()
        .unwrap();
    project
        .add(
            "custom-cloze",
            custom_cloze
                .note()
                .field("expr", "{{c2::second}} {{c500::last}}"),
        )
        .unwrap();
    let image = Media::bytes(png(), "image/png")
        .unwrap()
        .with_export_name("occlusion.png")
        .unwrap();
    let io = Note::image_occlusion(image)
        .mask(Mask::rect(
            if updated { "new" } else { "first" },
            0,
            0,
            4,
            4,
        ))
        .mask(Mask::rect("retained", 8, 8, 4, 4))
        .build()
        .unwrap();
    project.add("io", io).unwrap();
    project
}

fn report_value(output: &BuildOutput) -> Value {
    let mut report = output.report().snapshot();
    report.duration_ms = 0;
    serde_json::to_value(report).unwrap()
}

fn error_value(error: &BuildError) -> Value {
    let mut value = serde_json::to_value(error.snapshot()).unwrap();
    value["report"]["duration_ms"] = 0.into();
    value
}

fn artifact_bytes(output: &BuildOutput) -> Vec<u8> {
    std::fs::read(output.artifact().path()).unwrap()
}

fn identity(output: &BuildOutput) -> IdentityEnvelope {
    let mut zip =
        zip::ZipArchive::new(std::fs::File::open(output.artifact().path()).unwrap()).unwrap();
    serde_json::from_reader(zip.by_name(crate::build::identity::EVIDENCE_ENTRY).unwrap()).unwrap()
}

#[test]
fn project_preserve_repeat_build_bytes_counts_warnings_and_source_paths() {
    let project = project(false);
    let baseline = with_fixed_time(|| project.build(BuildOptions::temporary()).unwrap());
    assert_eq!(baseline.report().counts().notes, 7);
    assert_eq!(baseline.report().counts().cards, 15);
    let warning = baseline
        .report()
        .diagnostics()
        .iter()
        .find(|item| item.code == "TEMPLATE.FILTER_UNKNOWN")
        .unwrap();
    assert!(warning
        .source
        .as_ref()
        .unwrap()
        .path
        .contains("templates[\"m-auto\"].front"));
    let expected_report = report_value(&baseline);
    let expected_bytes = artifact_bytes(&baseline);
    for iteration in 0..2 {
        for _ in 0..2 {
            let output = with_fixed_time(|| project.build(BuildOptions::temporary()).unwrap());
            assert_eq!(report_value(&output), expected_report, "{iteration}");
            assert_eq!(artifact_bytes(&output), expected_bytes, "{iteration}");
        }
    }
    // Build's borrowed public signature leaves the caller's wide field reusable.
    assert_eq!(
        project.notes["wide"].fields[&crate::schema::FieldKey::from("front")].render(),
        "<b>wide 日本語 &amp;</b>".repeat(2048)
    );
}

#[test]
fn project_update_preserve_stable_ordinals_cloze_mask_history_and_revisions() {
    let old = project(false);
    let baseline = with_fixed_time(|| old.build(BuildOptions::temporary()).unwrap());
    let baseline_identity = identity(&baseline);
    let updated = project(true);
    let request = BuildOptions::temporary()
        .update_from(baseline.artifact().path())
        .update_policy(UpdatePolicy::default().fail_on(RiskLevel::Critical));
    let expected = with_fixed_time(|| updated.build(request.clone()).unwrap());
    let updated_identity = identity(&expected);
    for (key, before) in &baseline_identity.identity.models["dataflow-rules"].templates {
        let after = &updated_identity.identity.models["dataflow-rules"].templates[key];
        assert_eq!(
            before.ordinal, after.ordinal,
            "declaration order must not renumber {key}"
        );
        assert_eq!(before.id, after.id);
    }
    let io = &updated_identity.identity.notes["io"];
    assert!(!io.masks["first"].active);
    assert_eq!(io.masks["retained"].ordinal, 1);
    assert_eq!(io.masks["new"].ordinal, 2);
    assert!(io.mtime_secs > baseline_identity.identity.notes["io"].mtime_secs);
    assert_eq!(
        updated_identity.identity.notes["cloze"]
            .cards
            .values()
            .copied()
            .collect::<Vec<_>>(),
        vec![0, 2]
    );
    let expected_bytes = artifact_bytes(&expected);
    let expected_report = report_value(&expected);
    for iteration in 0..2 {
        let output = with_fixed_time(|| updated.build(request.clone()).unwrap());
        assert_eq!(artifact_bytes(&output), expected_bytes, "{iteration}");
        assert_eq!(report_value(&output), expected_report, "{iteration}");
    }
}

#[test]
fn project_missing_media_failure_keeps_context_and_borrowed_project_can_recover() {
    let model = model(false);
    let mut original = Project::new("dataflow-recover").unwrap();
    original
        .add(
            "missing",
            model
                .note()
                .field("q", Content::html("<img src=\"missing.png\">"))
                .field("a", "answer"),
        )
        .unwrap();
    let expected = with_fixed_time(|| original.build(BuildOptions::temporary()).unwrap_err());
    assert_eq!(expected.code(), "MEDIA.MISSING_REFERENCE");
    assert!(expected
        .report()
        .diagnostics()
        .iter()
        .any(|item| item.code == "TEMPLATE.FILTER_UNKNOWN"));
    let missing = expected
        .report()
        .diagnostics()
        .iter()
        .find(|item| item.code == "MEDIA.MISSING_REFERENCE")
        .unwrap();
    assert!(missing
        .source
        .as_ref()
        .unwrap()
        .path
        .contains("project.notes[\"missing\"]"));
    let expected_error = error_value(&expected);
    let mut expected_recovered = None;
    for iteration in 0..2 {
        let mut project = original.clone();
        for _ in 0..2 {
            let error = with_fixed_time(|| project.build(BuildOptions::temporary()).unwrap_err());
            assert_eq!(error_value(&error), expected_error, "{iteration}");
            assert!(error.publications().is_empty());
        }
        project
            .add_asset(
                Media::bytes(png(), "image/png")
                    .unwrap()
                    .with_export_name("missing.png")
                    .unwrap(),
            )
            .unwrap();
        let recovered = with_fixed_time(|| project.build(BuildOptions::temporary()).unwrap());
        let observations = (artifact_bytes(&recovered), report_value(&recovered));
        if let Some(expected) = &expected_recovered {
            assert_eq!(&observations, expected, "{iteration}");
        } else {
            expected_recovered = Some(observations);
        }
    }
}

#[test]
fn project_cloze_failure_preserve_error_order_and_baseline_observations() {
    for (body, code) in [
        ("{{c1::}}", "PRODUCT.CLOZE_MARKER_MALFORMED"),
        ("no cloze", "PRODUCT.CLOZE_MARKER_MISSING"),
        ("{{c501::too high}}", "NOTE.CLOZE_ORDINAL_EXCEEDED"),
    ] {
        let mut original = Project::new("dataflow-cloze-errors").unwrap();
        original.add("one", Note::cloze("{{c1::valid}}")).unwrap();
        let baseline = with_fixed_time(|| original.build(BuildOptions::temporary()).unwrap());
        let mut invalid = Project::new("dataflow-cloze-errors").unwrap();
        invalid.add("one", Note::cloze(body)).unwrap();
        let request = BuildOptions::temporary().update_from(baseline.artifact().path());
        let expected = with_fixed_time(|| invalid.build(request.clone()).unwrap_err());
        assert_eq!(expected.code(), code);
        assert_eq!(expected.report().baseline_counts().unwrap().notes, 1);
        let expected_error = error_value(&expected);
        for iteration in 0..2 {
            let error = with_fixed_time(|| invalid.build(request.clone()).unwrap_err());
            assert_eq!(error_value(&error), expected_error, "{iteration}: {body}");
            assert!(error.publications().is_empty());
        }
    }
}
