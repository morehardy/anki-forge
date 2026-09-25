//! Consumer of an unpacked crate, running without repository files at runtime.
mod inspection;
use ankiforge::build::{BuildErrorKind, InspectLimits};
use ankiforge::note::{Mask, OcclusionMode};
use ankiforge::update::{CompareOptions, RiskCode, UpdatePolicy};
use ankiforge::{BuildOptions, Content, Field, Media, Note, NoteType, Project, Template};
use anyhow::{ensure, Context};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn model() -> anyhow::Result<NoteType> {
    Ok(NoteType::builder("vocab")
        .name("词汇")
        .field(Field::new("front").name("正面").required())
        .field(Field::new("back").name("背面"))
        .template(
            Template::new("recognition")
                .name("识别")
                .front("{{front}}")
                .back("{{FrontSide}}<hr>{{back}}"),
        )
        .build()?)
}
fn basic_custom_cloze() -> anyhow::Result<()> {
    let custom = model()?;
    let mut p = Project::new("packaged-content")?.name("打包消费者");
    p.add(
        "basic",
        Note::basic("<literal>", Content::html("<b>answer</b>")),
    )?;
    p.add("cloze", Note::cloze("{{c1::<tag>}} {{c2::second}}"))?;
    p.add(
        "custom",
        custom.note().field("front", "cell").field("back", "细胞"),
    )?;
    let error = p
        .add("basic", Note::basic("duplicate", "answer"))
        .unwrap_err();
    ensure!(
        error.code() == "NOTE.KEY_DUPLICATE" && p.len() == 3,
        "explicit key atomicity"
    );
    let output = p.build(BuildOptions::to("content.apkg"))?;
    ensure!(
        output.report().counts().notes == 3 && output.report().counts().cards == 4,
        "Basic/Cloze/custom counts"
    );
    let seen = inspection::inspect(output.artifact().path())?;
    ensure!(
        seen.fields
            .contains(&"&lt;literal&gt;\u{1f}<b>answer</b>".into()),
        "Text/HTML semantics"
    );
    ensure!(
        seen.fields
            .iter()
            .any(|s| s.contains("{{c1::&lt;tag&gt;}} {{c2::second}}")),
        "Cloze text semantics"
    );
    ensure!(
        seen.models.values().any(|m| m["name"] == "词汇"
            && m["fields"] == serde_json::json!(["正面", "背面"])
            && m["templates"][0]["front"] == "{{正面}}"),
        "template key binding"
    );
    Ok(())
}
fn owned_media() -> anyhow::Result<()> {
    let png = fs::read("fixtures/pixel.png")?;
    fs::write("source.png", &png)?;
    let image = Media::file("source.png")?.with_export_name("badge.png")?;
    fs::write("source.png", b"changed after successful snapshot")?;
    fs::remove_file("source.png")?;
    let sound = Media::bytes(fs::read("fixtures/silence.wav")?, "audio/wav")?
        .with_export_name("tone.wav")?;
    let font = Media::file("fixtures/labels.woff")?.with_export_name("labels.woff")?;
    let custom = NoteType::builder("media")
        .field(Field::new("front"))
        .field(Field::new("back"))
        .template(
            Template::new("card")
                .front("{{front}}<img src=\"badge.png\">")
                .back("{{back}}"),
        )
        .css("@font-face {font-family:labels;src:url('labels.woff')}")
        .asset(font)
        .asset(image.clone())
        .build()?;
    let note = custom
        .note()
        .field(
            "front",
            Content::sequence([Content::text("<lead>"), image.image()]),
        )
        .field("back", sound.sound());
    let mut p = Project::new("packaged-media")?;
    p.add("media", note.clone())?;
    let large = b"/* comment */\n".repeat(6000);
    p.add_asset(Media::bytes(large.clone(), "text/css")?.with_export_name("large.css")?)?;
    let conflict = Media::bytes(png.clone(), "image/png")?.with_export_name("BADGE.png")?;
    ensure!(
        p.add("conflict", Note::basic(conflict.image(), "answer"))
            .is_err()
            && p.len() == 1,
        "media add atomicity"
    );
    let out = p.build(BuildOptions::to("owned-media.apkg"))?;
    let seen = inspection::inspect(out.artifact().path())?;
    ensure!(
        seen.assets.len() == 4
            && seen.assets["badge.png"] == png
            && seen.assets["large.css"] == large,
        "owned media closure"
    );
    ensure!(
        seen.assets["labels.woff"] == fs::read("fixtures/labels.woff")?,
        "explicit font asset bytes"
    );
    ensure!(
        seen.fields[0] == "&lt;lead&gt;<img src=\"badge.png\">\u{1f}[sound:tone.wav]",
        "typed dependency rendering"
    );
    let mut other = Project::new("packaged-reuse")?;
    other.add("same-value", note)?;
    ensure!(
        other
            .build(BuildOptions::temporary())?
            .report()
            .counts()
            .media
            == 3,
        "cross-project owned resources"
    );
    ensure!(
        Media::bytes(png, "audio/wav").is_err(),
        "MIME mismatch must fail"
    );
    Ok(())
}
fn copy_directory(source: &Path, destination: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_directory(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}
fn owned_bundle() -> anyhow::Result<()> {
    let directory = Path::new("owned-bundle");
    copy_directory(Path::new("fixtures/template-bundle"), directory)?;
    fs::copy("fixtures/labels.woff", directory.join("labels.woff"))?;
    fs::copy("fixtures/pixel.png", directory.join("badge.png"))?;
    let manifest = directory.join("anki-template.yaml");
    let mut text = fs::read_to_string(&manifest)?;
    text.push_str("\nassets:\n  - path: labels.woff\n    export_as: bundle-labels.woff\n  - path: badge.png\n    export_as: bundle-badge.png\n");
    fs::write(manifest, text)?;
    let style = directory.join("style.css");
    fs::write(
        &style,
        format!(
            "{}\n@font-face {{font-family:labels;src:url('bundle-labels.woff')}}",
            fs::read_to_string(&style)?
        ),
    )?;
    let front = directory.join("front.html");
    fs::write(
        &front,
        format!(
            "{}<img src=\"bundle-badge.png\">",
            fs::read_to_string(&front)?
        ),
    )?;
    let model = NoteType::from_bundle(directory)?;
    fs::remove_dir_all(directory)?;
    let mut project = Project::new("packaged-bundle")?;
    project.add(
        "capital",
        model
            .note()
            .field("text", "{{c1::Madrid}} is in {{c2::Spain}}.")
            .field("extra", "A city and its country."),
    )?;
    let out = project.build(BuildOptions::to("bundle.apkg"))?;
    let seen = inspection::inspect(out.artifact().path())?;
    ensure!(seen.ordinals == [0, 1], "bundle Cloze card count");
    ensure!(
        seen.assets.len() == 2
            && seen.assets["bundle-badge.png"] == fs::read("fixtures/pixel.png")?,
        "bundle complete snapshot"
    );
    ensure!(
        seen.models.values().any(|m| m["templates"][0]["front"]
            .as_str()
            .is_some_and(|v| v.contains("{{cloze:Sentence}}") && v.contains("bundle-badge.png"))),
        "bundle key and asset compilation"
    );
    Ok(())
}
fn occlusion() -> anyhow::Result<()> {
    let image = Media::file("fixtures/occlusion.png")?;
    for (name, mode) in [
        ("all", OcclusionMode::HideAllGuessOne),
        ("one", OcclusionMode::HideOneGuessOne),
    ] {
        let note = Note::image_occlusion(image.clone())
            .mode(mode)
            .mask(Mask::rect("left", 10, 10, 20, 20))
            .mask(Mask::rect("right", 50, 50, 20, 20))
            .build()?;
        let mut p = Project::new(format!("packaged-io-{name}"))?;
        ensure!(
            p.add("invalid", note.clone().field("image", "override"))
                .is_err()
                && p.is_empty(),
            "IO reserved field atomicity"
        );
        p.add("diagram", note)?;
        let output = p.build(BuildOptions::to(format!("io-{name}.apkg")))?;
        let seen = inspection::inspect(output.artifact().path())?;
        ensure!(
            seen.ordinals == [0, 1] && seen.assets.len() == 1,
            "IO cards and media"
        );
        ensure!(
            seen.fields[0]
                .contains("{{c1::image-occlusion:rect:left=0.1:top=0.125:width=0.2:height=0.25")
                && seen.fields[0].contains("{{c2::image-occlusion"),
            "independent normalized masks"
        );
        ensure!(
            seen.fields[0].contains(":oi=1") == (mode == OcclusionMode::HideAllGuessOne),
            "IO mode encoding"
        );
    }
    Ok(())
}
fn updates_and_reports() -> anyhow::Result<()> {
    let mut original = Project::new("packaged-update")?;
    original.add("keep", Note::basic("question", "original"))?;
    original.add("remove", Note::basic("omitted", "answer"))?;
    original.build(BuildOptions::to("v1.apkg"))?;
    let mut next = Project::new("packaged-update")?.name("Renamed project");
    next.add("keep", Note::basic("question", "changed"))?;
    let comparison = next.compare(CompareOptions::against("v1.apkg"))?;
    ensure!(
        !comparison.policy().allows_publication()
            && comparison
                .findings()
                .iter()
                .any(|f| f.code() == RiskCode::NoteRemoved),
        "comparison policy is separate from completion"
    );
    let error = next
        .build(BuildOptions::to("blocked.apkg").update_from("v1.apkg"))
        .unwrap_err();
    ensure!(
        error.kind() == BuildErrorKind::PolicyBlocked && !Path::new("blocked.apkg").exists(),
        "strict update blocked before publication"
    );
    let error_snapshot: ankiforge::build::json::BuildSnapshot = error.snapshot();
    ensure!(
        serde_json::to_value(error_snapshot)?["result"]["status"] == "failure",
        "actual failure outcome"
    );
    let policy = UpdatePolicy::default()
        .allow(RiskCode::NoteRemoved)
        .allow(RiskCode::MaskRemoved);
    let accepted =
        next.compare(CompareOptions::against("v1.apkg").update_policy(policy.clone()))?;
    ensure!(
        accepted.policy().allows_publication(),
        "explicit category acceptance"
    );
    let output = next.build(
        BuildOptions::to("v2.apkg")
            .update_policy(policy)
            .update_from("v1.apkg"),
    )?;
    let report: ankiforge::build::json::ReportSnapshot = output.report().snapshot();
    let snapshot: ankiforge::build::json::BuildSnapshot = output.snapshot();
    let report = serde_json::to_value(report)?;
    ensure!(
        report.get("result").is_none() && !output.report().diagnostics().is_empty(),
        "observations do not infer outcome"
    );
    ensure!(
        serde_json::to_value(snapshot)?["result"]["status"] == "success",
        "warning does not change success"
    );
    let before = inspection::inspect(Path::new("v1.apkg"))?;
    let after = inspection::inspect(Path::new("v2.apkg"))?;
    let guid = before.identity["identity"]["notes"]["keep"]["guid"]
        .as_str()
        .context("previous GUID")?;
    ensure!(
        after.guids == [guid] && after.fields == ["question\u{1f}changed"],
        "updated body with stable database GUID"
    );
    ensure!(
        after.identity["identity"]["notes"]["keep"]["mtime_secs"].as_i64()
            > before.identity["identity"]["notes"]["keep"]["mtime_secs"].as_i64(),
        "updated revision"
    );
    ensure!(
        next.build(BuildOptions::temporary().update_policy(UpdatePolicy::default()))
            .unwrap_err()
            .kind()
            == BuildErrorKind::Configuration,
        "Create rejects update policy"
    );
    ensure!(
        "RISK.UNKNOWN".parse::<RiskCode>().is_err(),
        "unknown policy code"
    );
    let mut limits = InspectLimits::default();
    limits.max_archive_bytes = 1;
    ensure!(
        next.build(BuildOptions::temporary().inspect_limits(limits))
            .unwrap_err()
            .kind()
            == BuildErrorKind::ResourceLimit,
        "candidate inspection budget"
    );
    let (path, snapshot) = {
        let out = next.build(BuildOptions::temporary())?;
        (out.artifact().path().to_owned(), out.snapshot())
    };
    ensure!(
        !path.exists() && serde_json::to_value(snapshot)?["result"]["status"] == "success",
        "snapshot does not retain temporary file"
    );
    Ok(())
}
fn native_path_snapshots() -> anyhow::Result<()> {
    use ankiforge::build::json::{BuildResultSnapshot, PathSnapshot};
    #[cfg(unix)]
    let path = {
        use std::os::unix::ffi::OsStringExt;
        PathBuf::from(std::ffi::OsString::from_vec(b"native-\xff.apkg".to_vec()))
    };
    #[cfg(windows)]
    let path = {
        use std::os::windows::ffi::OsStringExt;
        PathBuf::from(std::ffi::OsString::from_wide(&[0x61, 0xd800, 0x62]))
    };
    let value = serde_json::to_value(BuildResultSnapshot::Success {
        artifact: path.clone(),
        temporary: false,
    })?;
    ensure!(value["artifact"].is_object(), "native path encoding");
    let restored: PathSnapshot = serde_json::from_value(value["artifact"].clone())?;
    ensure!(
        restored.into_path_buf() == path,
        "lossless native path round trip"
    );
    Ok(())
}

fn main() -> anyhow::Result<()> {
    ensure!(
        !ankiforge::facade_api_version().is_empty()
            && !ankiforge::embedded_contract_version().is_empty(),
        "packaged metadata"
    );
    ensure!(
        PathBuf::from("fixtures/labels.woff").is_file(),
        "local fixtures required"
    );
    basic_custom_cloze()?;
    owned_media()?;
    owned_bundle()?;
    occlusion()?;
    updates_and_reports()?;
    native_path_snapshots()?;
    println!("Packaged consumer verified Basic/Cloze/custom, owned media and bundle closure, IO modes, strict comparison/update, reports, budgets and artifact lifetime using actual APKG contents.");
    Ok(())
}
