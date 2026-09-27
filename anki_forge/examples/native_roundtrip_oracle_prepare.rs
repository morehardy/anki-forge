//! Prepares original distributions through only the default public authoring API.
//! The companion executable imports them into a real upstream Anki collection.
use ankiforge::note::{Mask, OcclusionMode};
use ankiforge::{BuildOptions, Field, Media, Note, NoteType, Project, Template};
use anyhow::{ensure, Context};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

fn stage(
    target: (&Path, &str, usize),
    project: Project,
    expected_fields: BTreeMap<String, String>,
    baseline: Option<&Path>,
    structural: bool,
    expected_risks: &[&str],
) -> anyhow::Result<Value> {
    let (root, scenario, ordinal) = target;
    let path = root.join(format!("{scenario}-{ordinal}.apkg"));
    let mut options = BuildOptions::to(&path);
    let mut comparison = Value::Null;
    if let Some(baseline) = baseline {
        let report = project.compare(ankiforge::update::CompareOptions::against(baseline))?;
        comparison = serde_json::to_value(report.snapshot())?;
        for required in expected_risks {
            ensure!(
                report
                    .findings()
                    .iter()
                    .any(|f| f.code().as_str() == *required),
                "{scenario}: missing risk {required}"
            );
        }
        // The oracle deliberately exercises high-risk imports to document actual
        // client behavior. This is test policy, not a recommended publisher policy.
        options = options.update_from(baseline).update_policy(
            ankiforge::update::UpdatePolicy::default()
                .fail_on(ankiforge::update::RiskLevel::Critical),
        );
    }
    let output = project.build(options)?;
    let mut zip = zip::ZipArchive::new(fs::File::open(output.artifact().path())?)?;
    let mut bytes = Vec::new();
    zip.by_name("ankiforge-identity.json")?
        .read_to_end(&mut bytes)?;
    let evidence: Value = serde_json::from_slice(&bytes)?;
    let identity = &evidence["identity"]["notes"]["subject"];
    Ok(
        json!({"apkg":path,"expected_fields":expected_fields,"structural":structural,
        "source_mtime":identity["mtime_secs"],"guid":identity["guid"],"candidate_cards":identity["cards"],
        "comparison":comparison,"build":output.snapshot()}),
    )
}
fn fields(values: &[(&str, &str)]) -> BTreeMap<String, String> {
    values
        .iter()
        .map(|(k, v)| ((*k).into(), (*v).into()))
        .collect()
}
fn basic(root: &Path) -> anyhow::Result<Value> {
    let mut stages = vec![];
    for (index, text) in ["original answer", "updated answer", "third answer"]
        .iter()
        .enumerate()
    {
        let mut p = Project::new("oracle-basic")?
            .name(if index == 0 {
                "Original title"
            } else {
                "Renamed title"
            })
            .default_deck("Oracle::Basic");
        p.add("subject", Note::basic("question", *text))?;
        let baseline = stages
            .last()
            .and_then(|s: &Value| s["apkg"].as_str())
            .map(PathBuf::from);
        stages.push(stage(
            (root, "basic", index),
            p,
            fields(&[("Front", "question"), ("Back", text)]),
            baseline.as_deref(),
            false,
            if index == 0 {
                &[]
            } else {
                &["RISK.NOTE_CHANGED"]
            },
        )?);
    }
    Ok(json!({"name":"basic","stages":stages}))
}

fn custom_model(root: &Path) -> anyhow::Result<Value> {
    let name = "custom-symbols";
    let mut stages = vec![];
    for index in 0..3 {
        let renamed = index > 0;
        let (front_name, back_name) = if renamed {
            ("问题", "答案")
        } else {
            ("Prompt", "Answer")
        };
        let front = Field::new("front").name(front_name).sort();
        let back = Field::new("back").name(back_name);
        let forward = Template::new("forward")
            .name(if renamed { "识别" } else { "Forward" })
            .front("{{front}}")
            .back("{{back}}");
        let reverse = Template::new("reverse")
            .name(if renamed { "回忆" } else { "Reverse" })
            .front("{{back}}")
            .back("{{front}}");
        let mut model = NoteType::builder("custom").name(if renamed {
            "重命名模型"
        } else {
            "Original model"
        });
        model = if renamed {
            model
                .field(back)
                .field(front)
                .template(reverse)
                .template(forward)
        } else {
            model
                .field(front)
                .field(back)
                .template(forward)
                .template(reverse)
        };
        let model = model.build()?;
        let (front, back) = (format!("question-{index}"), format!("answer-{index}"));
        let mut p = Project::new("oracle-custom-symbols")?;
        p.add(
            "subject",
            model
                .note()
                .field("front", front.clone())
                .field("back", back.clone()),
        )?;
        let baseline = stages
            .last()
            .and_then(|s: &Value| s["apkg"].as_str())
            .map(PathBuf::from);
        stages.push(stage(
            (root, name, index),
            p,
            fields(&[(front_name, &front), (back_name, &back)]),
            baseline.as_deref(),
            false,
            if index == 1 {
                &["RISK.MODEL_CHANGED"]
            } else {
                &[]
            },
        )?);
    }
    Ok(json!({"name":name,"stages":stages}))
}
fn structure(root: &Path) -> anyhow::Result<Value> {
    let name = "schema-add-remove-restore";
    let mut stages = vec![];
    for (index, large) in [false, true, false, true].into_iter().enumerate() {
        let mut model = NoteType::builder("structure")
            .field(Field::new("front").name("Prompt").sort())
            .field(Field::new("back").name("Answer"))
            .template(Template::new("forward").front("{{front}}").back("{{back}}"));
        if large {
            model = model
                .field(Field::new("extra").name("Extra"))
                .template(Template::new("reverse").front("{{back}}").back("{{front}}"));
        }
        let model = model.build()?;
        let (front, back, extra) = (
            format!("question-{index}"),
            format!("answer-{index}"),
            format!("extra-{index}"),
        );
        let mut note = model
            .note()
            .field("front", front.clone())
            .field("back", back.clone());
        let mut expected = fields(&[("Prompt", &front), ("Answer", &back)]);
        if large {
            note = note.field("extra", extra.clone());
            expected.insert("Extra".into(), extra);
        }
        let mut p = Project::new("oracle-structure")?;
        p.add("subject", note)?;
        let baseline = stages
            .last()
            .and_then(|s: &Value| s["apkg"].as_str())
            .map(PathBuf::from);
        let risks: &[&str] = match index {
            1 | 3 => &["RISK.FIELD_ADDED", "RISK.TEMPLATE_ADDED"],
            2 => &["RISK.FIELD_REMOVED", "RISK.TEMPLATE_REMOVED"],
            _ => &[],
        };
        stages.push(stage(
            (root, name, index),
            p,
            expected,
            baseline.as_deref(),
            index > 0,
            risks,
        )?);
    }
    Ok(json!({"name":name,"stages":stages}))
}
fn sort_field(root: &Path) -> anyhow::Result<Value> {
    let name = "sort-field-change";
    let mut stages = vec![];
    for index in 0..3 {
        let mut front = Field::new("front").name("Prompt");
        let mut back = Field::new("back").name("Answer");
        if index == 0 {
            front = front.sort();
        } else {
            back = back.sort();
        }
        let model = NoteType::builder("sort")
            .field(front)
            .field(back)
            .template(Template::new("forward").front("{{front}}").back("{{back}}"))
            .build()?;
        let (front, back) = (format!("question-{index}"), format!("answer-{index}"));
        let mut p = Project::new("oracle-sort")?;
        p.add(
            "subject",
            model
                .note()
                .field("front", front.clone())
                .field("back", back.clone()),
        )?;
        let baseline = stages
            .last()
            .and_then(|s: &Value| s["apkg"].as_str())
            .map(PathBuf::from);
        stages.push(stage(
            (root, name, index),
            p,
            fields(&[("Prompt", &front), ("Answer", &back)]),
            baseline.as_deref(),
            index == 1,
            if index == 1 {
                &["RISK.SORT_FIELD_CHANGED"]
            } else {
                &[]
            },
        )?);
    }
    Ok(json!({"name":name,"stages":stages}))
}
fn cloze(root: &Path) -> anyhow::Result<Value> {
    let name = "cloze-equal-card-replacement";
    let mut stages = vec![];
    for (index, text) in [
        "{{c1::one}} {{c2::two}}",
        "{{c2::two updated}} {{c3::three}}",
        "{{c1::one restored}} {{c2::two updated}} {{c3::three}}",
    ]
    .iter()
    .enumerate()
    {
        let mut p = Project::new("oracle-cloze")?;
        p.add("subject", Note::cloze(*text))?;
        let baseline = stages
            .last()
            .and_then(|s: &Value| s["apkg"].as_str())
            .map(PathBuf::from);
        stages.push(stage(
            (root, name, index),
            p,
            fields(&[("Text", text)]),
            baseline.as_deref(),
            false,
            if index == 1 {
                &["RISK.CARD_ADDED", "RISK.CARD_REMOVED"]
            } else {
                &[]
            },
        )?);
    }
    Ok(json!({"name":name,"stages":stages}))
}
fn image_occlusion(root: &Path, mode: OcclusionMode) -> anyhow::Result<Value> {
    let name = if mode == OcclusionMode::HideAllGuessOne {
        "io-hide-all-mask-updates"
    } else {
        "io-hide-one-mask-updates"
    };
    let image = Media::file(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/public-api/occlusion.png"),
    )?;
    let mut stages = vec![];
    for index in 0..4 {
        let mut builder = Note::image_occlusion(image.clone()).mode(mode);
        let mut occlusion = String::new();
        for key in match index {
            0 => vec!["a", "b"],
            1 => vec!["b", "a"],
            2 => vec!["b", "c"],
            _ => vec!["c", "a", "b"],
        } {
            let x = match key {
                "a" => 1.0,
                "b" => 5.0,
                _ => 9.0,
            } + if index > 0 { 0.25 } else { 0.0 };
            builder = builder.mask(Mask::rect(key, x, 1.0, 2.0, 2.0));
            // Independent expected association: a retains c1, b retains c2,
            // the later new key c gets c3; the fixture is exactly 100 x 80.
            let ordinal = match key {
                "a" => 1,
                "b" => 2,
                _ => 3,
            };
            let inactive = if mode == OcclusionMode::HideAllGuessOne {
                ":oi=1"
            } else {
                ""
            };
            occlusion.push_str(&format!("{{{{c{ordinal}::image-occlusion:rect:left={}:top=0.0125:width=0.02:height=0.025{inactive}}}}}<br>", x/100.0));
        }
        let header = format!("diagram-{index}");
        let mut p = Project::new(format!("oracle-{name}"))?;
        p.add("subject", builder.build()?.field("header", header.clone()))?;
        let baseline = stages
            .last()
            .and_then(|s: &Value| s["apkg"].as_str())
            .map(PathBuf::from);
        let risks: &[&str] = match index {
            2 => &["RISK.MASK_ADDED", "RISK.MASK_REMOVED"],
            3 => &["RISK.MASK_ADDED"],
            _ => &[],
        };
        stages.push(stage(
            (root, name, index),
            p,
            fields(&[("Header", &header), ("Occlusion", &occlusion)]),
            baseline.as_deref(),
            false,
            risks,
        )?);
    }
    Ok(json!({"name":name,"stages":stages}))
}
fn main() -> anyhow::Result<()> {
    let output = PathBuf::from(
        std::env::args()
            .nth(1)
            .context("usage: native_roundtrip_oracle_prepare OUTPUT.json")?,
    );
    let root = output.parent().context("output parent")?;
    fs::create_dir_all(root)?;
    let scenarios = vec![
        basic(root)?,
        custom_model(root)?,
        structure(root)?,
        sort_field(root)?,
        cloze(root)?,
        image_occlusion(root, OcclusionMode::HideAllGuessOne)?,
        image_occlusion(root, OcclusionMode::HideOneGuessOne)?,
    ];
    fs::write(
        &output,
        serde_json::to_vec_pretty(
            &json!({"format_version":"native-anki-oracle-v1","scenarios":scenarios}),
        )?,
    )?;
    println!("{}", output.display());
    Ok(())
}
