#[path = "../../build-support/product_version.rs"]
mod product_version;

fn main() {
    product_version::emit_cargo_env();
    println!("cargo:rerun-if-changed=../../build-support/product_version.rs");
}
