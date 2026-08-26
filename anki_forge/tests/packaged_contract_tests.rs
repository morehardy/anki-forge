use anki_forge::runtime::{embedded_bundle_version, load_default_writer_stack, RuntimeMode};

#[test]
fn packaged_crate_loads_its_embedded_default_writer_stack() {
    assert_eq!(anki_forge::facade_api_version(), "0.1.0");
    assert!(!embedded_bundle_version().is_empty());

    let (runtime, _writer_policy, _build_context) =
        load_default_writer_stack().expect("load embedded writer stack");
    assert_eq!(runtime.mode, RuntimeMode::Installed);
    assert_eq!(runtime.bundle_version, embedded_bundle_version());
}
