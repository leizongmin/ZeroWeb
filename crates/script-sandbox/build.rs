fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let v8_enabled = std::env::var("CARGO_FEATURE_V8").is_ok();
    if v8_enabled && target_os == "windows" {
        println!("cargo:rustc-link-lib=dylib=advapi32");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
