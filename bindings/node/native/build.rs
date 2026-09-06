fn main() {
    napi_build::setup();
    println!(
        "cargo:rustc-env=ANKI_FORGE_NODE_TARGET={}",
        std::env::var("TARGET").unwrap()
    );
}
