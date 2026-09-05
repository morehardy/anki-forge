use contract_tools::{
    contract_manifest_path, manifest::load_manifest, semantics::run_semantics_gates,
};
use std::{fs, path::PathBuf, process::Command};
use tempfile::{tempdir, TempDir};

fn bundle() -> (TempDir, PathBuf) {
    let temp = tempdir().unwrap();
    let artifact =
        contract_tools::package::build_artifact(contract_manifest_path(), temp.path()).unwrap();
    tar::Archive::new(flate2::read::GzDecoder::new(
        fs::File::open(artifact).unwrap(),
    ))
    .unpack(temp.path())
    .unwrap();
    let manifest = temp.path().join("contracts/manifest.yaml");
    (temp, manifest)
}

#[test]
fn semantics_gate_checks_every_manifest_semantics_asset() {
    for key in [
        "target_selector_grammar",
        "identity_semantics",
        "merge_risk_semantics",
        "canonical_serialization_semantics",
        "identity_update_safety_semantics",
    ] {
        let (_temp, manifest_path) = bundle();
        let manifest = load_manifest(&manifest_path).unwrap();
        let path = manifest.contracts_root.join(&manifest.data.assets[key]);
        fs::write(
            path,
            "---\nasset_refs:\n  - schema/not-registered.json\n---\n# Invalid\n",
        )
        .unwrap();
        let error = run_semantics_gates(&manifest_path).expect_err(key);
        assert!(
            format!("{error:#}").contains("not-registered.json"),
            "{error:#}"
        );
    }
}

#[test]
fn semantics_gate_discovers_new_manifest_assets_without_code_changes() {
    let (_temp, path) = bundle();
    let root = path.parent().unwrap();
    let mut schema: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("schema/manifest.schema.json")).unwrap())
            .unwrap();
    schema["properties"]["assets"]["properties"]["future_rules"] =
        serde_json::json!({"type": "string"});
    fs::write(
        root.join("schema/manifest.schema.json"),
        serde_json::to_vec_pretty(&schema).unwrap(),
    )
    .unwrap();
    let mut manifest: serde_yaml::Value =
        serde_yaml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    manifest["assets"]["future_rules"] = "semantics/future.md".into();
    fs::write(&path, serde_yaml::to_string(&manifest).unwrap()).unwrap();
    fs::write(root.join("semantics/future.md"), "# Missing frontmatter\n").unwrap();
    let error = run_semantics_gates(&path).expect_err("new semantics must be checked");
    assert!(format!("{error:#}").contains("future.md"), "{error:#}");
}

#[test]
fn repository_diagnostic_literals_are_registered() {
    let manifest = contract_manifest_path();
    let root = manifest.parent().unwrap().parent().unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_contract_tools"))
        .arg("verify")
        .arg("--manifest")
        .arg(&manifest)
        .arg("--source-root")
        .arg(root)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn source_gate_checks_constants_helpers_raw_strings_macros_and_feature_branches() {
    let temp = tempdir().unwrap();
    for dir in ["anki_forge/src", "contract_tools/src"] {
        fs::create_dir_all(temp.path().join(dir)).unwrap();
    }
    fs::write(
        temp.path().join("anki_forge/src/lib.rs"),
        r###"
        // COMMENTS.IGNORED
        /// DOCS.IGNORED
        const KNOWN: &str = "PROJECT.EMPTY";
        const UNKNOWN: &str = r#"FUTURE.CONSTANT"#;
        #[cfg(feature = "optional")]
        fn optional() { helper("FUTURE.HELPER"); }
        fn macro_call() { emit!("FUTURE.MACRO"); }
        #[cfg(any(test, feature = "optional"))]
        fn possible_production() { helper("FUTURE.CONDITIONAL"); }
        #[cfg(test)]
        const TEST_ONLY: &str = "TEST.CONSTANT";
        #[cfg(all(test, feature = "optional"))]
        mod tests { const CODE: &str = "TEST.MODULE"; }
        #[test]
        fn example_test() { emit!("TEST.FUNCTION"); }
    "###,
    )
    .unwrap();
    fs::write(
        temp.path().join("contract_tools/src/lib.rs"),
        "fn helper() { DiagnosticCode::new(\"FUTURE.TOOL\"); }",
    )
    .unwrap();
    let error =
        contract_tools::registry::run_source_registry_gates(&contract_manifest_path(), temp.path())
            .unwrap_err()
            .to_string();
    for code in [
        "FUTURE.CONSTANT",
        "FUTURE.HELPER",
        "FUTURE.MACRO",
        "FUTURE.CONDITIONAL",
        "FUTURE.TOOL",
    ] {
        assert!(error.contains(code), "{error}");
    }
    for excluded in [
        "COMMENTS.IGNORED",
        "DOCS.IGNORED",
        "PROJECT.EMPTY",
        "TEST.CONSTANT",
        "TEST.MODULE",
        "TEST.FUNCTION",
    ] {
        assert!(!error.contains(excluded), "{error}");
    }
    assert!(error.contains("anki_forge/src/lib.rs:"), "{error}");
    assert!(error.contains("contract_tools/src/lib.rs:1:"), "{error}");
}

#[test]
fn source_gate_rejects_removed_codes_and_missing_source_trees() {
    let (_temp, manifest) = bundle();
    let sources = tempdir().unwrap();
    assert!(
        contract_tools::registry::run_source_registry_gates(&manifest, sources.path())
            .unwrap_err()
            .to_string()
            .contains("source directory is missing")
    );
    for dir in ["anki_forge/src", "contract_tools/src"] {
        fs::create_dir_all(sources.path().join(dir)).unwrap();
    }
    fs::write(
        sources.path().join("anki_forge/src/lib.rs"),
        "fn emit() { warning(\"PROJECT.EMPTY\"); }",
    )
    .unwrap();
    let registry_path = manifest
        .parent()
        .unwrap()
        .join("errors/error-registry.yaml");
    let mut registry: serde_yaml::Value =
        serde_yaml::from_str(&fs::read_to_string(&registry_path).unwrap()).unwrap();
    let entry = registry["codes"]
        .as_sequence_mut()
        .unwrap()
        .iter_mut()
        .find(|v| v["id"].as_str() == Some("PROJECT.EMPTY"))
        .unwrap();
    entry["status"] = "removed".into();
    fs::write(registry_path, serde_yaml::to_string(&registry).unwrap()).unwrap();
    let error = contract_tools::registry::run_source_registry_gates(&manifest, sources.path())
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("PROJECT.EMPTY is registered as removed"),
        "{error}"
    );
}

#[test]
fn source_gate_filters_test_only_impl_and_trait_members_without_skipping_production() {
    let temp = tempdir().unwrap();
    for dir in ["anki_forge/src", "contract_tools/src"] {
        fs::create_dir_all(temp.path().join(dir)).unwrap();
    }
    for container in ["impl Example", "trait Example"] {
        for (cfg, test_only) in [
            ("test", true),
            ("all(test, feature = \"optional\")", true),
            ("not(test)", false),
            ("any(test, feature = \"optional\")", false),
            ("feature = \"optional\"", false),
        ] {
            let source = format!(
                r#"{container} {{
                    #[cfg({cfg})]
                    fn helper() {{ emit!("FUTURE.METHOD"); }}
                    #[cfg({cfg})]
                    const CODE: &'static str = "FUTURE.CONST";
                    #[cfg({cfg})]
                    type Value = member_type!("FUTURE.TYPE");
                    #[cfg({cfg})]
                    members!("FUTURE.MACRO");
                }}"#,
            );
            fs::write(temp.path().join("anki_forge/src/lib.rs"), source).unwrap();
            let result = contract_tools::registry::run_source_registry_gates(
                &contract_manifest_path(),
                temp.path(),
            );
            if test_only {
                result.unwrap_or_else(|error| panic!("{container} cfg({cfg}): {error:#}"));
            } else {
                let error = result.unwrap_err().to_string();
                for code in [
                    "FUTURE.METHOD",
                    "FUTURE.CONST",
                    "FUTURE.TYPE",
                    "FUTURE.MACRO",
                ] {
                    assert!(error.contains(code), "{container} cfg({cfg}): {error}");
                }
            }
        }
    }
}

#[test]
fn release_verification_rejects_an_identical_bundle_baseline() {
    let (_temp, manifest) = bundle();
    let output = Command::new(env!("CARGO_BIN_EXE_contract_tools"))
        .args(["verify", "--release", "--manifest"])
        .arg(&manifest)
        .arg("--baseline-manifest")
        .arg(&manifest)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("release baseline bundle_version must be older"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_verification_requires_a_baseline_manifest() {
    let output = Command::new(env!("CARGO_BIN_EXE_contract_tools"))
        .args(["verify", "--release", "--manifest"])
        .arg(contract_manifest_path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--baseline-manifest"));
}

#[test]
fn manifest_versions_obey_semver_in_the_published_schema() {
    for version in ["01.0.0", "0.6", "../escape", "0.6.0-01", "0.6.0+"] {
        let (_temp, path) = bundle();
        let mut manifest: serde_yaml::Value =
            serde_yaml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        manifest["bundle_version"] = version.into();
        fs::write(&path, serde_yaml::to_string(&manifest).unwrap()).unwrap();
        assert!(load_manifest(&path).is_err(), "{version}");
    }
}

#[test]
fn verify_cli_enforces_baseline_and_accepts_a_fresh_record() {
    let (_old, baseline) = bundle();
    let (_new, current) = bundle();
    let asset = current.parent().unwrap().join("semantics/compatibility.md");
    let old_text = fs::read_to_string(&asset).unwrap();
    fs::write(
        &asset,
        format!("{old_text}\nAdditional normative clarification.\n"),
    )
    .unwrap();
    let verify = |release| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_contract_tools"));
        command
            .arg("verify")
            .arg("--manifest")
            .arg(&current)
            .arg("--baseline-manifest")
            .arg(&baseline);
        if release {
            command.arg("--release");
        }
        command.output().unwrap()
    };
    assert!(
        !verify(false).status.success(),
        "changed bytes without a matching record must fail"
    );

    let old = load_manifest(&baseline).unwrap();
    let old_version = semver::Version::parse(&old.data.bundle_version).unwrap();
    let mut manifest: serde_yaml::Value =
        serde_yaml::from_str(&fs::read_to_string(&current).unwrap()).unwrap();
    manifest["bundle_version"] =
        format!("{}.{}.0", old_version.major, old_version.minor + 1).into();
    fs::write(&current, serde_yaml::to_string(&manifest).unwrap()).unwrap();
    let mut record =
        contract_tools::versioning::change_record_template(&current, &baseline).unwrap();
    record.summary = "A reviewed clarification of the existing semantics".into();
    let record_path = manifest["assets"]["bundle_change"]
        .as_str()
        .unwrap_or("versioning/test-change.yaml")
        .to_owned();
    fs::write(
        current.parent().unwrap().join(&record_path),
        serde_yaml::to_string(&record).unwrap(),
    )
    .unwrap();
    manifest["assets"]["bundle_change"] = record_path.into();
    fs::write(&current, serde_yaml::to_string(&manifest).unwrap()).unwrap();
    for release in [false, true] {
        let result = verify(release);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
}
