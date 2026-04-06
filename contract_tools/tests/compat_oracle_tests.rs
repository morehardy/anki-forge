use contract_tools::{compat_oracle::run_compat_oracle_gates, contract_manifest_path};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

#[test]
fn compat_oracle_gates_accept_bundled_writer_phase3_fixtures() {
    run_compat_oracle_gates(copied_bundled_manifest_path("compat-oracle"))
        .expect("bundled compat oracle gate should pass");
}

fn temp_contract_root(label: &str) -> PathBuf {
    static NEXT_TEMP_ROOT_ID: AtomicU64 = AtomicU64::new(0);
    let unique = NEXT_TEMP_ROOT_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "anki-forge-contract-tools-{}-{}-{}",
        label,
        std::process::id(),
        unique
    ))
}

fn copy_tree(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).expect("create destination tree");
    for entry in fs::read_dir(src).expect("read source tree") {
        let entry = entry.expect("read source entry");
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_tree(&src_path, &dst_path);
        } else {
            fs::copy(&src_path, &dst_path).expect("copy source file");
        }
    }
}

fn copied_bundled_manifest_path(label: &str) -> PathBuf {
    let root = temp_contract_root(label);
    copy_tree(
        contract_manifest_path()
            .parent()
            .expect("contracts root for bundled manifest"),
        &root,
    );
    root.join("manifest.yaml")
}
