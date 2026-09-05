use contract_tools::versioning::{change_record_template, run_change_gates};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::{tempdir, TempDir};

struct Bundles {
    _temp: TempDir,
    baseline: PathBuf,
    current: PathBuf,
}

impl Bundles {
    fn new() -> Self {
        let temp = tempdir().unwrap();
        let baseline = temp.path().join("old");
        let current = temp.path().join("new");
        for root in [&baseline, &current] {
            fs::create_dir_all(root.join("schema")).unwrap();
            fs::create_dir_all(root.join("versioning")).unwrap();
            fs::write(
                root.join("schema/manifest.schema.json"),
                "{\"type\":\"object\"}",
            )
            .unwrap();
            fs::write(root.join("schema/example.json"), "{\"type\":\"object\"}").unwrap();
            fs::write(root.join("manifest.yaml"), "bundle_version: 0.5.0\ncomponent_versions: {}\ncompatibility:\n  public_axis: bundle_version\nassets:\n  manifest_schema: schema/manifest.schema.json\n  example_schema: schema/example.json\n").unwrap();
        }
        Self {
            _temp: temp,
            baseline: baseline.join("manifest.yaml"),
            current: current.join("manifest.yaml"),
        }
    }

    fn change(&self) {
        fs::write(
            self.current.parent().unwrap().join("schema/example.json"),
            "{\"type\":\"object\",\"description\":\"extended schema\"}",
        )
        .unwrap();
    }

    fn version(&self, version: &str) {
        edit_manifest(&self.current, |v| v["bundle_version"] = version.into());
    }

    fn record(&self, class: &str) {
        let root = self.current.parent().unwrap();
        let mut record = change_record_template(&self.current, &self.baseline).unwrap();
        record.compatibility_class = class.into();
        record.summary = "Regression fixture for a reviewed contract change".into();
        record.migration_notes = Some("Migrate consumers before adopting the new contract".into());
        fs::write(
            root.join("versioning/change.yaml"),
            serde_yaml::to_string(&record).unwrap(),
        )
        .unwrap();
        edit_manifest(&self.current, |v| {
            v["assets"]["bundle_change"] = "versioning/change.yaml".into()
        });
    }

    fn check(&self) -> anyhow::Result<()> {
        run_change_gates(&self.current, &self.baseline)
    }
}

fn edit_manifest(path: &Path, edit: impl FnOnce(&mut serde_yaml::Value)) {
    let mut v = serde_yaml::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    edit(&mut v);
    fs::write(path, serde_yaml::to_string(&v).unwrap()).unwrap();
}

#[test]
fn unchanged_bundle_needs_no_change_record() {
    Bundles::new().check().unwrap();
}

#[test]
fn actual_asset_change_requires_record_and_minor_bump() {
    let b = Bundles::new();
    b.change();
    assert!(b.check().unwrap_err().to_string().contains("bundle_change"));
    b.record("additive_compatible");
    assert!(b.check().unwrap_err().to_string().contains("minor"));
    b.version("0.5.1");
    b.record("additive_compatible");
    assert!(b.check().unwrap_err().to_string().contains("minor"));
    b.version("0.6.0");
    b.record("additive_compatible");
    b.check().unwrap();
}

#[test]
fn declared_breaking_change_requires_major_bump_and_migration_notes() {
    let b = Bundles::new();
    b.change();
    b.version("0.6.0");
    b.record("behavior_changing_incompatible");
    assert!(b.check().unwrap_err().to_string().contains("major"));
    b.version("1.0.0");
    b.record("behavior_changing_incompatible");
    b.check().unwrap();
    let path = b.current.parent().unwrap().join("versioning/change.yaml");
    edit_manifest(&path, |v| v["migration_notes"] = serde_yaml::Value::Null);
    assert!(b
        .check()
        .unwrap_err()
        .to_string()
        .contains("migration_notes"));
}

#[test]
fn change_record_is_bound_to_exact_content_and_baseline() {
    let b = Bundles::new();
    b.change();
    b.version("0.6.0");
    b.record("additive_compatible");
    fs::write(
        b.current.parent().unwrap().join("schema/example.json"),
        "{\"type\":\"string\"}",
    )
    .unwrap();
    assert!(b
        .check()
        .unwrap_err()
        .to_string()
        .contains("actual asset changes"));
    b.record("additive_compatible");
    edit_manifest(&b.baseline, |v| v["bundle_version"] = "0.4.0".into());
    assert!(b
        .check()
        .unwrap_err()
        .to_string()
        .contains("baseline_version"));
}

#[test]
fn asset_removal_or_retargeting_cannot_claim_additive_compatibility() {
    for remove in [true, false] {
        let b = Bundles::new();
        b.version("1.0.0");
        if remove {
            edit_manifest(&b.current, |v| {
                v["assets"]
                    .as_mapping_mut()
                    .unwrap()
                    .remove(serde_yaml::Value::String("example_schema".into()));
            });
        } else {
            fs::copy(
                b.current.parent().unwrap().join("schema/example.json"),
                b.current.parent().unwrap().join("schema/renamed.json"),
            )
            .unwrap();
            edit_manifest(&b.current, |v| {
                v["assets"]["example_schema"] = "schema/renamed.json".into()
            });
        }
        b.record("additive_compatible");
        assert!(b.check().unwrap_err().to_string().contains("incompatible"));
        b.record("behavior_changing_incompatible");
        b.check().unwrap();
    }
}

#[test]
fn schema_changes_cannot_claim_documentation_or_fixture_only() {
    let b = Bundles::new();
    b.change();
    b.version("0.6.0");
    for class in [
        "documentation_only_normative_clarification",
        "fixture_only_non_semantic",
    ] {
        b.record(class);
        assert!(b
            .check()
            .unwrap_err()
            .to_string()
            .contains("compatibility_class"));
    }
}

#[test]
fn malformed_decreasing_and_metadata_only_versions_are_rejected() {
    let b = Bundles::new();
    for version in ["banana", "../1.0.0", "01.0.0", "0.4.0"] {
        b.version(version);
        assert!(b.check().is_err(), "{version}");
    }
    b.change();
    b.version("0.5.0+build2");
    b.record("additive_compatible");
    assert!(b.check().unwrap_err().to_string().contains("minor"));
}

#[test]
fn transitive_fixture_changes_are_inventoried_and_allow_patch_only_class() {
    let b = Bundles::new();
    for manifest in [&b.baseline, &b.current] {
        let root = manifest.parent().unwrap();
        fs::create_dir_all(root.join("fixtures")).unwrap();
        fs::write(
            root.join("fixtures/index.yaml"),
            "cases:\n  - id: test\n    category: valid\n    input: fixtures/input.json\n",
        )
        .unwrap();
        fs::write(root.join("fixtures/input.json"), "{}").unwrap();
        edit_manifest(manifest, |v| {
            v["assets"]["fixture_catalog"] = "fixtures/index.yaml".into()
        });
    }
    fs::write(
        b.current.parent().unwrap().join("fixtures/input.json"),
        "{\"sample\":true}",
    )
    .unwrap();
    let inventory = change_record_template(&b.current, &b.baseline).unwrap();
    assert_eq!(inventory.changes.len(), 1);
    assert_eq!(inventory.changes[0].path, "fixtures/input.json");
    b.version("0.5.1");
    b.record("fixture_only_non_semantic");
    b.check().unwrap();
}

#[test]
fn removal_or_retirement_of_registered_codes_forces_incompatible_class() {
    for status in [Some("removed"), None] {
        let b = Bundles::new();
        for manifest in [&b.baseline, &b.current] {
            let root = manifest.parent().unwrap();
            fs::create_dir_all(root.join("errors")).unwrap();
            fs::write(
                root.join("errors/registry.yaml"),
                "codes:\n  - id: TEST.CODE\n    status: active\n    summary: test code\n",
            )
            .unwrap();
            edit_manifest(manifest, |v| {
                v["assets"]["error_registry"] = "errors/registry.yaml".into()
            });
        }
        let registry = b.current.parent().unwrap().join("errors/registry.yaml");
        if let Some(status) = status {
            fs::write(
                registry,
                format!(
                    "codes:\n  - id: TEST.CODE\n    status: {status}\n    summary: test code\n"
                ),
            )
            .unwrap();
        } else {
            fs::write(registry, "codes: []\n").unwrap();
        }
        b.version("1.0.0");
        b.record("additive_compatible");
        assert!(b.check().unwrap_err().to_string().contains("incompatible"));
        b.record("behavior_changing_incompatible");
        b.check().unwrap();
    }
}

#[test]
fn duplicate_or_incomplete_change_records_fail() {
    let b = Bundles::new();
    b.change();
    b.version("0.6.0");
    b.record("additive_compatible");
    let record_path = b.current.parent().unwrap().join("versioning/change.yaml");
    edit_manifest(&record_path, |v| {
        let first = v["changes"][0].clone();
        v["changes"].as_sequence_mut().unwrap().push(first);
    });
    assert!(b.check().unwrap_err().to_string().contains("duplicate"));
    b.record("additive_compatible");
    edit_manifest(&record_path, |v| {
        v["changes"][0]["after"] = "blake3:invalid".into()
    });
    assert!(b.check().unwrap_err().to_string().contains("digest"));
}
