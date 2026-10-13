//! Link the extension module against the host interpreter's Python symbols.
//!
//! `maturin` already passes these flags, but adding them here keeps a plain
//! `cargo build`/`cargo check` of the workspace working too.

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:rustc-cdylib-link-arg=-undefined");
        println!("cargo:rustc-cdylib-link-arg=dynamic_lookup");
    }
}
