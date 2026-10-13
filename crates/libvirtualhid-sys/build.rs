//! Build the vendored libvirtualhid C++ library and the flat C ABI shim.

use std::path::{Path, PathBuf};

/// Depth-first search for a build artifact produced by CMake.
fn find_artifact(root: &Path, file_name: &str) -> Option<PathBuf> {
    let mut stack = vec![root.to_path_buf()];
    while let Some(directory) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.file_name().and_then(|name| name.to_str()) == Some(file_name) {
                return Some(path);
            }
        }
    }
    None
}

fn main() {
    // libvirtualhid uses std::format and floating-point std::to_chars, which
    // libc++ only exposes for macOS 13+. Pin the deployment target so the
    // build does not depend on whatever the caller's environment supplies.
    std::env::set_var("MACOSX_DEPLOYMENT_TARGET", "14.2");

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let shim_dir = manifest_dir.join("shim");
    let upstream = manifest_dir
        .join("../../third_party/libvirtualhid")
        .canonicalize()
        .expect("vendored third_party/libvirtualhid is missing");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", shim_dir.join("lvh_shim.cpp").display());
    println!("cargo:rerun-if-changed={}", shim_dir.join("lvh_shim.h").display());

    // Build only the portable static library; skip tools, tests, docs, and the
    // licensed macOS broker (which needs an Apple-restricted entitlement).
    let destination = cmake::Config::new(&upstream)
        .build_target("libvirtualhid")
        .define("CMAKE_BUILD_TYPE", "Release")
        .define("CMAKE_OSX_DEPLOYMENT_TARGET", "14.2")
        .define("BUILD_TESTS", "OFF")
        .define("BUILD_DOCS", "OFF")
        .define("BUILD_EXAMPLES", "OFF")
        .define("LIBVIRTUALHID_BUILD_TOOLS", "OFF")
        .define("LIBVIRTUALHID_BUILD_MACOS_BROKER", "OFF")
        .define("LIBVIRTUALHID_INSTALL", "OFF")
        .define("LIBVIRTUALHID_ENABLE_PACKAGING", "OFF")
        .define("LIBVIRTUALHID_WARNINGS_AS_ERRORS", "OFF")
        .build();

    let archive = find_artifact(&destination, "libvirtualhid.a")
        .unwrap_or_else(|| panic!("libvirtualhid.a not found under {}", destination.display()));
    let library_dir = archive.parent().expect("archive has no parent").to_path_buf();

    println!("cargo:rustc-link-search=native={}", library_dir.display());
    println!("cargo:rustc-link-lib=static=virtualhid");
    println!("cargo:rustc-link-lib=c++");
    for framework in [
        "ApplicationServices",
        "Carbon",
        "CoreFoundation",
        "CoreGraphics",
        "IOKit",
    ] {
        println!("cargo:rustc-link-lib=framework={framework}");
    }
    // The library is compiled with -fexperimental-library, so consumers must match.
    println!("cargo:rustc-link-arg=-fexperimental-library");

    cc::Build::new()
        .cpp(true)
        .file(shim_dir.join("lvh_shim.cpp"))
        .include(&shim_dir)
        .include(upstream.join("src/include"))
        .flag("-std=c++23")
        .flag("-fexperimental-library")
        .warnings(false)
        .compile("lvh_shim");
}
