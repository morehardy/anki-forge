#![cfg(unix)]

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Command, Output},
};
use tempfile::{tempdir, TempDir};

struct Repository {
    root: TempDir,
    previous: String,
}

impl Repository {
    fn new() -> Self {
        let root = tempdir().unwrap();
        for dir in ["scripts", "contracts", "bin"] {
            fs::create_dir(root.path().join(dir)).unwrap();
        }
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        fs::copy(
            repo_root.join("scripts/check_contract_governance.sh"),
            root.path().join("scripts/check_contract_governance.sh"),
        )
        .unwrap();
        // Isolate the shell's Git/ref validation; CLI/bundle behavior is tested
        // with the real binary in governance_tests and versioning_change_tests.
        let cargo = root.path().join("bin/cargo");
        fs::write(
            &cargo,
            "#!/usr/bin/env bash\nprintf 'cargo-arg:%s\\n' \"$@\"\n",
        )
        .unwrap();
        fs::set_permissions(cargo, fs::Permissions::from_mode(0o755)).unwrap();
        let mut repo = Self {
            root,
            previous: String::new(),
        };
        repo.git(&["init", "-q", "-b", "main"]);
        fs::write(
            repo.root.path().join("contracts/manifest.yaml"),
            "bundle_version: 0.5.0\n",
        )
        .unwrap();
        repo.git(&["add", "contracts/manifest.yaml"]);
        repo.git(&["commit", "-qm", "previous bundle"]);
        repo.previous = repo.git(&["rev-parse", "HEAD"]);
        fs::write(
            repo.root.path().join("contracts/manifest.yaml"),
            "bundle_version: 0.6.0\n",
        )
        .unwrap();
        repo.git(&["commit", "-qam", "current bundle"]);
        repo.git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
        repo
    }

    fn git(&self, args: &[&str]) -> String {
        let output = Command::new("git")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .args([
                "-c",
                "user.name=Governance Test",
                "-c",
                "user.email=governance@example.invalid",
            ])
            .args(args)
            .current_dir(self.root.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    }

    fn verify(&self, baseline: Option<&str>, release: bool) -> Output {
        let mut paths = vec![self.root.path().join("bin")];
        paths.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap()));
        let mut command = Command::new("bash");
        command
            .arg(
                self.root
                    .path()
                    .join("scripts/check_contract_governance.sh"),
            )
            .env("PATH", std::env::join_paths(paths).unwrap())
            .env("CONTRACT_BASE_REF", baseline.unwrap_or_default())
            .current_dir(self.root.path());
        if release {
            command.arg("--release");
        }
        command.output().unwrap()
    }
}

#[test]
fn release_rejects_current_commit_through_head_branch_sha_and_tag() {
    let repo = Repository::new();
    let head = repo.git(&["rev-parse", "HEAD"]);
    repo.git(&["tag", "-am", "current release", "current-release"]);
    for baseline in ["HEAD", "main", head.as_str(), "current-release"] {
        let output = repo.verify(Some(baseline), true);
        assert!(
            !output.status.success(),
            "self-baseline accepted: {baseline}"
        );
        assert!(String::from_utf8_lossy(&output.stderr).contains("strict ancestor"));
        assert!(!String::from_utf8_lossy(&output.stdout).contains("cargo-arg:"));
    }
}

#[test]
fn release_rejects_non_ancestor_before_invoking_verification() {
    let repo = Repository::new();
    let unrelated = repo.git(&["commit-tree", "HEAD^{tree}", "-m", "unrelated bundle"]);
    let output = repo.verify(Some(&unrelated), true);
    assert!(!output.status.success(), "non-ancestor baseline accepted");
    assert!(String::from_utf8_lossy(&output.stderr).contains("strict ancestor"));
    assert!(!String::from_utf8_lossy(&output.stdout).contains("cargo-arg:"));
}

#[test]
fn release_requires_an_explicit_previous_ref() {
    let repo = Repository::new();
    let output = repo.verify(None, true);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("CONTRACT_BASE_REF"));
    assert!(!String::from_utf8_lossy(&output.stdout).contains("cargo-arg:"));
}

#[test]
fn release_accepts_a_strict_ancestor_and_forwards_release_mode() {
    let repo = Repository::new();
    let output = repo.verify(Some(&repo.previous), true);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("cargo-arg:--release\n"));
}

#[test]
fn ordinary_local_verification_still_allows_an_unchanged_checkout() {
    let repo = Repository::new();
    let output = repo.verify(None, false);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cargo-arg:verify\n"));
    assert!(!stdout.contains("cargo-arg:--release\n"));
}
