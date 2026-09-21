//! Build script: compiles the vendored musl libc and emscripten libc++ into
//! static libraries for the bare-metal wasm32 targets and publishes their
//! include paths through `cargo::metadata`.

mod wasm_libc;
mod wasm_libcxx;

use std::path::{Path, PathBuf};

/// Bare-metal wasm targets (no OS) have no system libc/libc++; the vendored
/// musl libc + emscripten libc++ must be built and linked in for them.
///
/// Covers `wasm32-unknown-unknown`, `wasm32-unknown-none`, and `wasm32v1-none`.
/// Targets that ship their own libc (`wasm32-wasip1/2`,
/// `wasm32-unknown-emscripten`) are intentionally excluded — linking a second
/// libc would conflict.
fn is_bare_wasm() -> bool {
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    arch == "wasm32" && matches!(os.as_str(), "unknown" | "none")
}

fn main() {
    let is_docs_rs = std::env::var("DOCS_RS").is_ok();

    if !is_docs_rs && is_bare_wasm() {
        wasm_libc::main();
        wasm_libcxx::main();
        emit_include_metadata();
    }

    println!("cargo::rerun-if-changed=vendored/");
    println!("cargo::rerun-if-changed=src/wasm_ffi/");
}

/// Publish the include paths for downstream C/C++ compilation.
///
/// The paths are made absolute against this crate's manifest directory: a
/// dependent crate's build script runs with a different current directory,
/// so relative `-I` entries would resolve to the wrong place.
fn emit_include_metadata() {
    println!(
        "cargo::metadata=include_libc={}",
        join_paths(wasm_libc::includes())
    );
    println!(
        "cargo::metadata=include_libcxx={}",
        join_paths(wasm_libcxx::includes())
    );
}

fn join_paths<T: AsRef<Path>>(paths: impl IntoIterator<Item = T>) -> String {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let absolute: Vec<PathBuf> = paths
        .into_iter()
        .map(|path| manifest_dir.join(path.as_ref()))
        .collect();
    std::env::join_paths(&absolute)
        .expect("include paths must not contain the platform path separator")
        .display()
        .to_string()
}
