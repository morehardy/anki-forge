use ankiforge::{BuildOptions, Note, Project};
use ankiforge::build::{BuildError, BuildErrorKind, InspectLimits};
use std::{error::Error, fs, io::{Read, Write}, path::Path};
use prost::Message;

#[derive(Clone, PartialEq, Message)]
struct MediaIndex {
    #[prost(message, repeated, tag="1")] entries: Vec<MediaEntry>,
}
#[derive(Clone, PartialEq, Message)]
struct MediaEntry {
    #[prost(string, tag="1")] name: String,
    #[prost(uint32, tag="2")] size: u32,
    #[prost(bytes, tag="3")] sha1: Vec<u8>,
    #[prost(uint32, optional, tag="255")] legacy_zip_filename: Option<u32>,
}

fn copy_with(input: &Path, output: &str, mut edit: impl FnMut(&str, Vec<u8>) -> Option<Vec<u8>>) {
    let mut source = zip::ZipArchive::new(fs::File::open(input).unwrap()).unwrap();
    let mut target = zip::ZipWriter::new(fs::File::create(output).unwrap());
    for i in 0..source.len() {
        let mut entry = source.by_index(i).unwrap();
        let mut data = Vec::new(); entry.read_to_end(&mut data).unwrap();
        if let Some(data) = edit(entry.name(), data) {
            target.start_file(entry.name(), zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored)).unwrap();
            target.write_all(&data).unwrap();
        }
    }
    target.finish().unwrap();
}
fn has_source<T: Error + 'static>(error: &dyn Error) -> bool {
    let mut source = error.source();
    while let Some(cause) = source { if cause.is::<T>() { return true; } source = cause.source(); }
    false
}
fn failure(project: &Project, baseline: &str, code: &str) -> BuildError {
    fs::write("protected.apkg", "keep previous destination").unwrap();
    let error = project.build(BuildOptions::to("protected.apkg").update_from(baseline)).unwrap_err();
    assert_eq!(error.code(), code, "{error}");
    assert!(error.publications().is_empty());
    assert_eq!(fs::read_to_string("protected.apkg").unwrap(), "keep previous destination");
    error
}

fn duplicate_media_names_are_not_complete_evidence() -> Result<(), Box<dyn Error>> {
    let mut project = Project::new("media-proof")?;
    project.add("one", Note::basic("question", "answer"))?;
    project.add_asset(ankiforge::Media::bytes(b"original".to_vec(), "application/octet-stream")?
        .with_export_name("proof.bin")?)?;
    let baseline = project.build(BuildOptions::temporary())?;
    for altered_first_payload in [false, true] {
        let output = format!("duplicate-media-{altered_first_payload}.apkg");
        let mut original_payload = Vec::new();
        copy_with(baseline.artifact().path(), &output, |name, data| {
            if name == "media" {
                let decoded = zstd::decode_all(data.as_slice()).unwrap();
                let mut media = MediaIndex::decode(decoded.as_slice()).unwrap();
                assert_eq!(media.entries.len(), 1);
                media.entries.push(media.entries[0].clone());
                return Some(zstd::encode_all(media.encode_to_vec().as_slice(), 0).unwrap());
            }
            if name == "0" {
                original_payload = data.clone();
                if altered_first_payload {
                    return Some(zstd::encode_all(b"hidden!!".as_slice(), 0).unwrap());
                }
            }
            Some(data)
        });
        // The last duplicate has the original payload, so a lossy filename map
        // agrees with the untouched identity manifest despite the extra entry.
        let file = fs::OpenOptions::new().read(true).write(true).open(&output)?;
        let mut zip = zip::ZipWriter::new_append(file)?;
        zip.start_file("1", zip::write::SimpleFileOptions::default())?;
        zip.write_all(&original_payload)?;
        zip.finish()?;
        assert_eq!(failure(&project, &output, "UPDATE.EVIDENCE_INVALID").kind(), BuildErrorKind::Validation);
        let error = project.compare(ankiforge::update::CompareOptions::against(&output)).unwrap_err();
        assert_eq!(error.code(), "UPDATE.EVIDENCE_INVALID");
    }
    let accepted = project.build(BuildOptions::temporary().update_from(baseline.artifact().path()))?;
    assert_eq!(accepted.report().counts().media, 1);
    Ok(())
}
fn main() -> Result<(), Box<dyn Error>> {
    retired_card_history_is_validated()?;
    media_map_metadata_must_match_payloads()?;
    duplicate_media_names_are_not_complete_evidence()?;
    let mut project = Project::new("proof")?;
    project.add("one", Note::basic("question", "answer"))?;
    let baseline = project.build(BuildOptions::to("baseline.apkg"))?;
    copy_with(baseline.artifact().path(), "missing.apkg", |name, data| (name != "ankiforge-identity.json").then_some(data));
    assert_eq!(failure(&project, "missing.apkg", "UPDATE.EVIDENCE_MISSING").kind(), BuildErrorKind::Validation);
    copy_with(baseline.artifact().path(), "json.apkg", |name, data| Some(if name == "ankiforge-identity.json" { b"{invalid json".to_vec() } else { data }));
    assert!(has_source::<serde_json::Error>(&failure(&project, "json.apkg", "UPDATE.EVIDENCE_INVALID")));
    copy_with(baseline.artifact().path(), "mapping.apkg", |name, data| {
        if name != "ankiforge-identity.json" { return Some(data); }
        let mut value: serde_json::Value = serde_json::from_slice(&data).unwrap();
        value["identity"]["notes"]["one"]["guid"] = "another-note".into();
        Some(serde_json::to_vec(&value).unwrap())
    });
    failure(&project, "mapping.apkg", "UPDATE.EVIDENCE_INVALID");
    // Copy the exact future candidate fingerprints into an unchanged baseline.
    // Trusting these hashes would suppress the real ModelChanged/NoteChanged
    // findings, even though the candidate and SQLite baseline differ.
    for model_change in [false, true] {
        let mut candidate = Project::new("proof")?;
        if model_change {
            let model = ankiforge::NoteType::builder("basic").name("Edited Basic")
                .field(ankiforge::Field::new("front").name("Front"))
                .field(ankiforge::Field::new("back").name("Back"))
                .template(ankiforge::Template::new("card").front("{{front}}").back("{{back}}"))
                .build()?;
            candidate.add("one", model.note().field("front", "question").field("back", "answer"))?;
        } else { candidate.add("one", Note::basic("future changed question", "answer"))?; }
        let candidate_output = candidate.build(BuildOptions::temporary())?;
        let mut archive = zip::ZipArchive::new(fs::File::open(candidate_output.artifact().path())?)?;
        let future: serde_json::Value = serde_json::from_reader(archive.by_name("ankiforge-identity.json")?)?;
        let domain = if model_change { "models" } else { "notes" };
        let key = if model_change { "basic" } else { "one" };
        let output = format!("forged-future-{domain}.apkg");
        copy_with(baseline.artifact().path(), &output, |name, data| {
            if name != "ankiforge-identity.json" { return Some(data); }
            let mut value: serde_json::Value = serde_json::from_slice(&data).unwrap();
            assert_ne!(value["identity"][domain][key]["content_hash"], future["identity"][domain][key]["content_hash"]);
            value["identity"][domain][key]["content_hash"] = future["identity"][domain][key]["content_hash"].clone();
            value["identity"].sort_all_objects();
            value["identity_blake3"] = blake3::hash(&serde_json::to_vec(&value["identity"]).unwrap()).to_hex().to_string().into();
            Some(serde_json::to_vec(&value).unwrap())
        });
        failure(&candidate, &output, "UPDATE.EVIDENCE_INVALID");
        assert_eq!(candidate.compare(ankiforge::update::CompareOptions::against(&output)).unwrap_err().code(), "UPDATE.EVIDENCE_INVALID");
        let normal = candidate.compare(ankiforge::update::CompareOptions::against(baseline.artifact().path()))?;
        let expected = if model_change { ankiforge::update::RiskCode::ModelChanged } else { ankiforge::update::RiskCode::NoteChanged };
        assert!(normal.findings().iter().any(|finding| finding.code() == expected));
    }
    // A correct checksum alone does not establish semantic consistency with
    // the SQLite collection. Invalid mappings must still be rejected.
    for what in ["card_key", "model_kind", "sort_field", "model_content_hash", "note_content_hash"] {
        let path = format!("inconsistent-{what}.apkg");
        copy_with(baseline.artifact().path(), &path, |name, data| {
            if name != "ankiforge-identity.json" { return Some(data); }
            let mut value: serde_json::Value = serde_json::from_slice(&data).unwrap();
            match what {
                "model_content_hash" => { value["identity"]["models"].as_object_mut().unwrap().values_mut().next().unwrap()["content_hash"] = "0".repeat(64).into(); }
                "note_content_hash" => { value["identity"]["notes"]["one"]["content_hash"] = "0".repeat(64).into(); }
                "card_key" => { value["identity"]["notes"]["one"]["cards"] = serde_json::json!({"template:unknown-card": 0}); }
                "model_kind" => { value["identity"]["models"].as_object_mut().unwrap().values_mut().next().unwrap()["kind"] = "cloze".into(); }
                "sort_field" => { value["identity"]["models"].as_object_mut().unwrap().values_mut().next().unwrap()["sort_field"] = "back".into(); }
                _ => unreachable!(),
            }
            value["identity"].sort_all_objects();
            value["identity_blake3"] = blake3::hash(&serde_json::to_vec(&value["identity"]).unwrap()).to_hex().to_string().into();
            Some(serde_json::to_vec(&value).unwrap())
        });
        failure(&project, &path, "UPDATE.EVIDENCE_INVALID");
    }
    let target_model = ankiforge::NoteType::builder("targeted")
        .field(ankiforge::Field::new("front"))
        .template(ankiforge::Template::new("card").front("{{front}}").back("{{front}}").target_deck("Template::Destination"))
        .build()?;
    let mut target_project = Project::new("target-proof")?.default_deck("Note::Destination");
    target_project.add("one", target_model.note().field("front", "question"))?;
    let target_baseline = target_project.build(BuildOptions::temporary())?;
    let _target_control = target_project.build(BuildOptions::temporary().update_from(target_baseline.artifact().path()))?;
    // A producer can recompute unkeyed envelope digests. Semantic validation
    // must compare the evidence to the actual SQLite content nonetheless.
    for (what, sql) in [
        ("note_fields", "UPDATE notes SET flds = 'changed question' || char(31) || 'answer'"),
        ("model_name", "UPDATE notetypes SET name = 'changed display name'"),
        // Protobuf accepts a repeated scalar tag with the last value winning.
        // Preserve every existing config field while changing real CSS/format.
        ("model_css", "UPDATE notetypes SET config = CAST(config || X'1A0178' AS BLOB)"),
        ("template_format", "UPDATE templates SET config = CAST(config || X'0A0178' AS BLOB)"),
        ("field_name", "UPDATE fields SET name = 'changed field name' WHERE ord = 0"),
        ("template_name", "UPDATE templates SET name = 'changed template name'"),
        ("note_tags", "UPDATE notes SET tags = 'changed_tag'"),
        ("card_flags", "UPDATE cards SET flags = 1"),
        ("target_card_deck", "UPDATE cards SET did = 1"),
        ("card_deck", "INSERT INTO decks SELECT 987, 'Wrong Deck', mtime_secs, usn, common, kind FROM decks WHERE id = 1; UPDATE cards SET did = 987"),
    ] {
        let output = format!("resigned-{what}.apkg");
        let (test_project, source) = if what == "target_card_deck" { (&target_project, target_baseline.artifact().path()) } else { (&project, baseline.artifact().path()) };
        let mut archive = zip::ZipArchive::new(fs::File::open(source)?)?;
        let mut compressed = Vec::new();
        archive.by_name("collection.anki21b")?.read_to_end(&mut compressed)?;
        let sqlite_path = format!("resigned-{what}.sqlite");
        fs::write(&sqlite_path, zstd::decode_all(compressed.as_slice())?)?;
        let db = rusqlite::Connection::open(&sqlite_path)?;
        db.execute_batch(sql)?;
        drop(db);
        let sqlite = fs::read(&sqlite_path)?;
        let digest = blake3::hash(&sqlite).to_hex().to_string();
        let compressed = zstd::encode_all(sqlite.as_slice(), 0)?;
        copy_with(source, &output, |name, bytes| {
            if name == "collection.anki21b" { return Some(compressed.clone()); }
            if name != "ankiforge-identity.json" { return Some(bytes); }
            let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            value["collection_blake3"] = digest.clone().into();
            value["identity"].sort_all_objects();
            value["identity_blake3"] = blake3::hash(&serde_json::to_vec(&value["identity"]).unwrap()).to_hex().to_string().into();
            Some(serde_json::to_vec(&value).unwrap())
        });
        failure(test_project, &output, "UPDATE.EVIDENCE_INVALID");
        assert_eq!(test_project.compare(ankiforge::update::CompareOptions::against(&output)).unwrap_err().code(), "UPDATE.EVIDENCE_INVALID");
    }
    copy_with(baseline.artifact().path(), "collection.apkg", |name, data| {
        if name != "collection.anki21b" { return Some(data); }
        let bytes = zstd::decode_all(data.as_slice()).unwrap(); fs::write("tampered.sqlite", bytes).unwrap();
        let db = rusqlite::Connection::open("tampered.sqlite").unwrap();
        db.execute("UPDATE notes SET flds = 'edited outside publisher'", []).unwrap(); drop(db);
        Some(zstd::encode_all(fs::read("tampered.sqlite").unwrap().as_slice(), 0).unwrap())
    });
    failure(&project, "collection.apkg", "UPDATE.EVIDENCE_INVALID");
    let mut wrong = Project::new("other-project")?; wrong.add("one", Note::basic("question", "answer"))?;
    failure(&wrong, "baseline.apkg", "UPDATE.NAMESPACE_MISMATCH");
    let error = failure(&project, "does-not-exist.apkg", "BUILD.INSPECT_FAILED");
    assert_eq!(error.kind(), BuildErrorKind::Io); assert!(has_source::<std::io::Error>(&error));
    let wrapped = anyhow::Error::new(error).context("updating publisher output");
    assert_eq!(wrapped.downcast_ref::<BuildError>().unwrap().kind(), BuildErrorKind::Io);

    // Duplicate central-directory names are ambiguous even when both entries
    // contain individually valid evidence. ZipWriter itself disallows them,
    // so patch an equal-length alternate name after creating the fixture.
    fs::copy(baseline.artifact().path(), "duplicate.apkg")?;
    let data = {
        let mut zip = zip::ZipArchive::new(fs::File::open(baseline.artifact().path())?)?;
        let mut bytes = Vec::new(); zip.by_name("ankiforge-identity.json")?.read_to_end(&mut bytes)?; bytes
    };
    let file = fs::OpenOptions::new().read(true).write(true).open("duplicate.apkg")?;
    let mut zip = zip::ZipWriter::new_append(file)?;
    zip.start_file("ankiforge-identitz.json", zip::write::SimpleFileOptions::default())?;
    zip.write_all(&data)?; zip.finish()?;
    let mut bytes = fs::read("duplicate.apkg")?;
    let old = b"ankiforge-identitz.json"; let new = b"ankiforge-identity.json";
    for index in (0..=bytes.len()-old.len()).rev() { if &bytes[index..index+old.len()] == old { bytes[index..index+old.len()].copy_from_slice(new); } }
    fs::write("duplicate.apkg", bytes)?;
    failure(&project, "duplicate.apkg", "BUILD.INSPECT_FAILED");

    let mut limits = InspectLimits::default(); limits.max_identity_bytes = 1;
    let error = project.build(BuildOptions::temporary().update_from("baseline.apkg").inspect_limits(limits.clone())).unwrap_err();
    assert_eq!(error.kind(), BuildErrorKind::ResourceLimit);
    assert_eq!(error.limit_exceeded().unwrap().resource, "identity_bytes");
    assert_eq!(error.limit_exceeded().unwrap().limit, 1);
    // Without a baseline, the independently counted candidate has the same finite budget.
    let candidate_error = project.build(BuildOptions::temporary().inspect_limits(limits)).unwrap_err();
    assert_eq!(candidate_error.limit_exceeded().unwrap().resource, "identity_bytes");
    assert_eq!(candidate_error.report().counts().notes, 1);
    let _output = project.build(BuildOptions::temporary().update_from("baseline.apkg").inspect_limits(InspectLimits::default()))?;
    Ok(())
}


fn media_map_metadata_must_match_payloads() -> Result<(), Box<dyn Error>> {
    // One entry takes the serial path; 17 exercise parallel completion and reuse
    // of hash-worker slots without changing archive read/decoded-byte budgets.
    for count in [1, 17, 0] {
        let mut project = Project::new(format!("media-metadata-{count}"))?;
        project.add("one", Note::basic("question", "answer"))?;
        for index in 0..count {
            project.add_asset(ankiforge::Media::bytes(format!("payload-{index}").into_bytes(), "application/octet-stream")?
                .with_export_name(format!("proof-{index}.bin"))?)?;
        }
        let baseline = project.build(BuildOptions::temporary())?;
        let mut archive = zip::ZipArchive::new(fs::File::open(baseline.artifact().path())?)?;
        let mut collection = Vec::new();
        archive.by_name("collection.anki21b")?.read_to_end(&mut collection)?;
        let collection_digest = blake3::hash(&zstd::decode_all(collection.as_slice())?).to_hex().to_string();
        let variants: &[&str] = if count == 0 { &["protobuf", "zstd"] }
            else { &["size", "sha1", "short_sha1", "legacy_zero", "legacy_missing", "protobuf", "zstd"] };
        for &what in variants {
            let output = format!("media-metadata-{count}-{what}.apkg");
            copy_with(baseline.artifact().path(), &output, |name, data| {
                if name == "ankiforge-identity.json" {
                    let mut value: serde_json::Value = serde_json::from_slice(&data).unwrap();
                    value["collection_blake3"] = collection_digest.clone().into();
                    value["identity"].sort_all_objects();
                    value["identity_blake3"] = blake3::hash(&serde_json::to_vec(&value["identity"]).unwrap()).to_hex().to_string().into();
                    return Some(serde_json::to_vec(&value).unwrap());
                }
                if name != "media" { return Some(data); }
                if what == "zstd" { let mut truncated = data; truncated.pop(); return Some(truncated); }
                if what == "protobuf" { return Some(zstd::encode_all([0xff].as_slice(), 0).unwrap()); }
                let mut media = MediaIndex::decode(zstd::decode_all(data.as_slice()).unwrap().as_slice()).unwrap();
                // Exercise both a hash received during slot reuse and the final
                // pending entry drained after the media loop.
                let index = if what == "sha1" { 0 } else { media.entries.len() - 1 };
                let entry = &mut media.entries[index];
                match what {
                    "size" => entry.size += 1,
                    "sha1" => entry.sha1[0] ^= 0xff,
                    "short_sha1" => { entry.sha1.pop(); },
                    "legacy_zero" => entry.legacy_zip_filename = Some(0),
                    "legacy_missing" => entry.legacy_zip_filename = Some(999),
                    _ => unreachable!(),
                }
                Some(zstd::encode_all(media.encode_to_vec().as_slice(), 0).unwrap())
            });
            let error = failure(&project, &output, "UPDATE.EVIDENCE_INVALID");
            assert_eq!(error.kind(), if what == "zstd" { BuildErrorKind::Io } else { BuildErrorKind::Validation });
            if what == "protobuf" { assert!(has_source::<prost::DecodeError>(&error), "retain the original protobuf cause: {error:?}"); }
            if what == "zstd" { assert!(has_source::<std::io::Error>(&error), "retain the original decoder cause: {error:?}"); }
            assert_eq!(project.compare(ankiforge::update::CompareOptions::against(&output)).unwrap_err().code(), "UPDATE.EVIDENCE_INVALID");
        }
        let valid = project.build(BuildOptions::temporary().update_from(baseline.artifact().path()))?;
        assert_eq!(valid.report().counts().media, count);
        assert!(valid.report().comparison().unwrap().findings().is_empty());
        if count == 0 { continue; }
        let mut limits = InspectLimits::default(); limits.max_media_bytes = 8;
        let error = project.build(BuildOptions::temporary().update_from(baseline.artifact().path()).inspect_limits(limits)).unwrap_err();
        assert_eq!(error.kind(), BuildErrorKind::ResourceLimit);
        assert_eq!(error.limit_exceeded().unwrap().resource, "media_bytes");
        assert_eq!(error.limit_exceeded().unwrap().limit, 8);
    }
    Ok(())
}


fn history_project(stage: u8) -> Result<Project, Box<dyn Error>> {
    use ankiforge::{Field, Media, NoteType, Template};
    use ankiforge::note::Mask;
    let mut project = Project::new("retired-card-history")?;
    project.add("keep", Note::basic("keep", "answer"))?;
    let mut builder = NoteType::builder("normal-history").field(Field::new("front")).field(Field::new("gate"));
    for key in if stage == 0 { &["a", "b", "c"][..] } else { &["c"][..] } {
        let template = Template::new(*key).front("{{front}}").back("{{front}}");
        builder = builder.template(if *key == "b" { template.front("{{gate}}{{front}}")
            .generate_when(ankiforge::schema::GenerationRule::all(["gate"])) } else { template });
    }
    let normal = builder.build()?;
    project.add("normal-peer", normal.note().field("front", "peer").field("gate", "on"))?;
    if stage != 1 {
        project.add("retired-normal", normal.note().field("front", "normal").field("gate", ""))?;
        project.add("retired-empty", normal.note().field("front", "").field("gate", ""))?;
        project.add("retired-cloze", Note::cloze("{{c1::one}} {{c3::three}}"))?;
        project.add("retired-io", Note::image_occlusion(Media::file("assets/occlusion.png")?)
            .mask(Mask::rect("a", 0, 0, 1, 1)).mask(Mask::rect("b", 2, 0, 1, 1)).build()?)?;
    }
    if stage == 1 {
        // The shared model can change from stock IO to custom cloze while a
        // retired note still records its last published IO mask/card history.
        let mut builder = NoteType::builder("image_occlusion");
        for key in ["occlusion", "image", "header", "back_extra", "comments"] {
            builder = builder.field(Field::new(key));
        }
        let cloze = builder.cloze_field("occlusion")
            .template(Template::new("card").front("{{cloze:occlusion}}").back("{{cloze:occlusion}}"))
            .build()?;
        project.add("io-peer", cloze.note().field("occlusion", "{{c1::one}}").field("image", "")
            .field("header", "").field("back_extra", "").field("comments", ""))?;
    } else {
        project.add("io-peer", Note::image_occlusion(Media::file("assets/occlusion.png")?)
            .mask(Mask::rect("peer", 0, 0, 1, 1)).build()?)?;
    }
    Ok(project)
}

fn retired_card_history_is_validated() -> Result<(), Box<dyn Error>> {
    use ankiforge::update::{CompareOptions, RiskCode, UpdatePolicy};
    let original = history_project(0)?.build(BuildOptions::temporary())?;
    let retired = history_project(1)?;
    let baseline = retired.build(BuildOptions::temporary().update_from(original.artifact().path())
        .update_policy(UpdatePolicy::default().allow(RiskCode::NoteRemoved).allow(RiskCode::ModelRemoved)
            .allow(RiskCode::TemplateRemoved).allow(RiskCode::MaskRemoved)))?;
    let evidence = |path: &Path| -> serde_json::Value {
        let mut archive = zip::ZipArchive::new(fs::File::open(path).unwrap()).unwrap();
        serde_json::from_reader::<_, serde_json::Value>(archive.by_name("ankiforge-identity.json").unwrap()).unwrap()["identity"].clone()
    };
    let history = evidence(baseline.artifact().path());
    assert_eq!(history["models"]["normal-history"]["templates"]["a"]["ordinal"], serde_json::Value::Null);
    assert_eq!(history["models"]["normal-history"]["templates"]["c"]["ordinal"], 0);
    assert_eq!(history["notes"]["retired-normal"]["cards"], serde_json::json!({"template:a":0,"template:c":2}));
    assert_eq!(history["notes"]["retired-empty"]["cards"], serde_json::json!({}));
    assert_eq!(history["notes"]["retired-io"]["active"], false);
    assert_eq!(history["notes"]["retired-io"]["masks"]["a"]["active"], true);
    for (what, key, cards) in [
        ("cloze_out_of_range", "retired-cloze", serde_json::json!({"cloze:501":500})),
        ("cloze_key", "retired-cloze", serde_json::json!({"cloze:01":0})),
        ("cloze_prefix", "retired-cloze", serde_json::json!({"template:card":0})),
        ("bare_key", "retired-cloze", serde_json::json!({"bogus":0})),
        ("unknown_template", "retired-normal", serde_json::json!({"template:unknown":0})),
        ("duplicate_ordinal", "retired-normal", serde_json::json!({"template:a":0,"template:b":0})),
        ("template_out_of_range", "retired-normal", serde_json::json!({"template:c":3})),
        ("template_order", "retired-normal", serde_json::json!({"template:b":1,"template:c":0})),
        ("template_gap", "retired-normal", serde_json::json!({"template:b":0,"template:c":2})),
        ("unknown_mask", "retired-io", serde_json::json!({"mask:unknown":0,"mask:b":1})),
        ("mask_ordinal", "retired-io", serde_json::json!({"mask:a":1,"mask:b":0})),
        ("mask_inactive", "retired-io", serde_json::json!({"mask:a":0,"mask:b":1})),
        ("mask_all_retired", "retired-io", serde_json::json!({"mask:a":0,"mask:b":1})),
        ("missing_mask_card", "retired-io", serde_json::json!({"mask:a":0})),
        ("mask_prefix", "retired-io", serde_json::json!({"cloze:1":0,"cloze:2":1})),
    ] {
        let output = format!("retired-card-history-{what}.apkg");
        copy_with(baseline.artifact().path(), &output, |name, data| {
            if name != "ankiforge-identity.json" { return Some(data); }
            let mut value: serde_json::Value = serde_json::from_slice(&data).unwrap();
            value["identity"]["notes"][key]["cards"] = cards.clone();
            if what == "mask_inactive" || what == "mask_all_retired" { value["identity"]["notes"][key]["masks"]["a"]["active"] = false.into(); }
            if what == "mask_all_retired" { value["identity"]["notes"][key]["masks"]["b"]["active"] = false.into(); }
            value["identity"].sort_all_objects();
            value["identity_blake3"] = blake3::hash(&serde_json::to_vec(&value["identity"]).unwrap()).to_hex().to_string().into();
            Some(serde_json::to_vec(&value).unwrap())
        });
        failure(&retired, &output, "UPDATE.EVIDENCE_INVALID");
        assert_eq!(retired.compare(CompareOptions::against(&output)).unwrap_err().code(), "UPDATE.EVIDENCE_INVALID");
    }
    let unchanged = retired.build(BuildOptions::temporary().update_from(baseline.artifact().path()))?;
    assert_eq!(history, evidence(unchanged.artifact().path()), "a retired note keeps its historical card ordinals while the model evolves");
    assert!(unchanged.report().comparison().unwrap().findings().is_empty());
    let revived = history_project(2)?.build(BuildOptions::temporary().update_from(baseline.artifact().path())
        .update_policy(UpdatePolicy::default().allow(RiskCode::CardRemoved)))?;
    let restored = evidence(revived.artifact().path());
    for key in ["retired-normal", "retired-empty", "retired-cloze", "retired-io"] {
        assert_eq!(history["notes"][key]["guid"], restored["notes"][key]["guid"]);
        assert_eq!(restored["notes"][key]["active"], true);
    }
    assert_eq!(restored["notes"]["retired-normal"]["cards"], serde_json::json!({"template:c":0}));
    assert_eq!(history["notes"]["retired-io"]["masks"], restored["notes"]["retired-io"]["masks"]);
    let repeat = history_project(2)?.build(BuildOptions::temporary().update_from(revived.artifact().path()))?;
    assert!(repeat.report().comparison().unwrap().findings().is_empty());
    Ok(())
}
