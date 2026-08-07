mod wasm_libc;
mod wasm_libcxx;

use std::sync::OnceLock;

const FLAGS: &[&str] = &[
    "/wd4189",
    "-Wno-unused-variable",
    "-Wno-type-limits",
    "-Wno-unused-but-set-variable",
    "-Wno-unused-function",
    "-Wno-misleading-indentation",
    "-Wno-stringop-overflow",
    "-Wno-array-bounds",
    "-Wno-unused-parameter",
    "-Wno-sign-compare",
    "-fno-exceptions",
    // Fix gcc optimization issue.
    // See vendored/basis_universal/transcoder/basisu.h
    // See https://github.com/godotengine/godot/pull/114839
    "-fno-strict-aliasing",
];

// Disable PVRTC1/2, ATC, FXT1 as wgpu does not support them.
const DEFINES: &[(&str, &str)] = &[
    // ("BASISU_FORCE_DEVEL_MESSAGES", "1"), // Enable debug message.
    // ("BASISD_SUPPORT_KTX2", "0"),
    // ("BASISD_SUPPORT_KTX2_ZSTD", "0"),
    #[cfg(not(feature = "transcode_uastc"))]
    ("BASISD_SUPPORT_UASTC", "0"),
    #[cfg(not(feature = "transcode_etc1s_bc1"))]
    ("BASISD_SUPPORT_DXT1", "0"), //(BC1)
    #[cfg(not(feature = "transcode_etc1s_bc4_5"))]
    ("BASISD_SUPPORT_DXT5A", "0"), //(BC3 / 4 / 5)
    #[cfg(not(feature = "transcode_etc1s_bc7"))]
    ("BASISD_SUPPORT_BC7", "0"),
    ("BASISD_SUPPORT_PVRTC1", "0"),
    #[cfg(not(feature = "transcode_etc1s_etc2"))]
    ("BASISD_SUPPORT_ETC2_EAC_A8", "0"),
    #[cfg(not(feature = "transcode_astc"))]
    ("BASISD_SUPPORT_ASTC", "0"),
    #[cfg(not(feature = "transcode_xuastc"))]
    ("BASISD_SUPPORT_XUASTC", "0"),
    ("BASISD_SUPPORT_ATC", "0"),
    #[cfg(not(feature = "transcode_etc1s_etc2"))]
    ("BASISD_SUPPORT_ETC2_EAC_RG11", "0"),
    // ("BASISD_SUPPORT_ASTC_HIGHER_OPAQUE_QUALITY", "0"),
    ("BASISD_SUPPORT_FXT1", "0"),
    ("BASISD_SUPPORT_PVRTC2", "0"),
    #[cfg(not(feature = "transcode_uastc_hdr"))]
    ("BASISD_SUPPORT_UASTC_HDR", "0"),
];

/// Bare-metal wasm targets (no OS) have no system libc/libc++; the vendored
/// musl libc + emscripten libc++ must be built and linked in for them.
///
/// Covers `wasm32-unknown-unknown`, `wasm32-unknown-none`, and `wasm32v1-none`.
/// Targets that ship their own libc (`wasm32-wasip1/2`, `wasm32-unknown-emscripten`)
/// are intentionally excluded — linking a second libc would conflict.
fn is_bare_wasm() -> bool {
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    arch == "wasm32" && matches!(os.as_str(), "unknown" | "none")
}

static ENCODER_SRCS: OnceLock<Vec<String>> = OnceLock::new();

fn get_encoder_srcs() -> &'static [String] {
    let dir = env!("CARGO_MANIFEST_DIR");
    let srcs = &[
        "vendored/basis_universal/transcoder/basisu_transcoder.cpp",
        "vendored/basis_universal/zstd/zstd.c",
    ];

    ENCODER_SRCS.get_or_init(|| {
        let mut vec = Vec::new();
        search_files(
            std::path::PathBuf::from_iter([dir, "vendored/basis_universal/encoder/"]),
            "cpp",
            &mut vec,
        );
        vec.extend(srcs.map(ToString::to_string));
        vec
    })
}

const TRANSCODER_SRCS: &[&str] = &[
    "vendored/basis_universal/encoder/basisu_wasm_transcoder_api.cpp",
    "vendored/basis_universal/transcoder/basisu_transcoder.cpp",
    "vendored/basis_universal/zstd/zstddeclib.c",
];

fn search_files(dir: impl AsRef<std::path::Path>, extension: &str, results: &mut Vec<String>) {
    let entries = std::fs::read_dir(dir).expect("Failed to read directory");

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            search_files(path.to_str().unwrap(), extension, results);
        } else if path.is_file() && path.extension().map(|s| s.to_str().unwrap()) == Some(extension)
        {
            results.push(path.into_os_string().into_string().unwrap());
        }
    }
}

fn main() {
    bindgen();

    let is_docs_rs = std::env::var("DOCS_RS").is_ok();

    if !is_docs_rs {
        if is_bare_wasm() {
            wasm_libc::main();
            wasm_libcxx::main();
        }

        compile_basisu_static();
    }

    println!("cargo::rerun-if-changed=vendored/");
    println!("cargo::rerun-if-changed=src/wasm_ffi/");
}

#[derive(Debug)]
struct MacroTypeCallbacks;

#[derive(Debug)]
struct WasmBoolRenameCallbacks;

impl bindgen::callbacks::ParseCallbacks for MacroTypeCallbacks {
    fn int_macro(&self, name: &str, _value: i64) -> Option<bindgen::callbacks::IntKind> {
        if name.starts_with("BU_COMP_FLAGS_") {
            Some(bindgen::callbacks::IntKind::U64)
        } else if name.starts_with("BU_EFFORT_") || name.starts_with("BU_QUALITY_") {
            Some(bindgen::callbacks::IntKind::I32)
        } else if name.starts_with("BTF_")
            || name.starts_with("TF_")
            || name.starts_with("DECODE_FLAGS_")
        {
            Some(bindgen::callbacks::IntKind::U32)
        } else {
            None
        }
    }
}

impl bindgen::callbacks::ParseCallbacks for WasmBoolRenameCallbacks {
    fn item_name(&self, item_info: bindgen::callbacks::ItemInfo) -> Option<String> {
        if item_info.name == "wasm_bool_t" {
            Some("Bool32".to_string())
        } else {
            None
        }
    }
}

fn bindgen() {
    // rustc-only bare-wasm triples (wasm32v1-none, wasm32-unknown-none) are
    // not valid clang target triples; clang only knows the underlying LLVM
    // triple wasm32-unknown-unknown. Pass it explicitly so bindgen doesn't
    // forward the raw cargo TARGET (it would make libclang error out).
    // For wasm32-unknown-unknown itself this is the same value bindgen would
    // infer, so the generated bindings are unchanged.
    let clang_target_args: &[&str] = if is_bare_wasm() {
        &["--target=wasm32-unknown-unknown"]
    } else {
        &[]
    };

    let binding_file =
        std::path::PathBuf::from_iter([&std::env::var("OUT_DIR").unwrap(), "basisu_api_common.rs"]);
    bindgen::Builder::default()
        .clang_args(&["-fvisibility=default"])
        .clang_args(clang_target_args)
        .header("vendored/basis_universal/encoder/basisu_wasm_api_common.h")
        .use_core()
        .allowlist_var("^(BU_QUALITY_.*)$")
        .allowlist_var("^(BU_EFFORT_.*)$")
        .allowlist_var("^(BU_COMP_FLAGS_.*)$")
        .allowlist_var("^(BTF_.*)$")
        .allowlist_var("^(TF_.*)$")
        .allowlist_var("^(DECODE_FLAGS_.*)$")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .parse_callbacks(Box::new(MacroTypeCallbacks))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(binding_file)
        .expect("Couldn't write bindings!");

    let binding_file =
        std::path::PathBuf::from_iter([&std::env::var("OUT_DIR").unwrap(), "basisu_c_api.rs"]);
    bindgen::Builder::default()
        .clang_args(&["-fvisibility=default"])
        .clang_args(clang_target_args)
        .header("vendored/basis_universal/encoder/basisu_wasm_api.h")
        .use_core()
        .must_use_type("wasm_bool_t")
        .new_type_alias("Bool32")
        .allowlist_function("^(bu_.*)$")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .parse_callbacks(Box::new(WasmBoolRenameCallbacks))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(binding_file)
        .expect("Couldn't write bindings!");

    let binding_file = std::path::PathBuf::from_iter([
        &std::env::var("OUT_DIR").unwrap(),
        "basisu_c_transcoder_api.rs",
    ]);
    bindgen::Builder::default()
        .clang_args(&["-fvisibility=default"])
        .clang_args(clang_target_args)
        .header("vendored/basis_universal/encoder/basisu_wasm_transcoder_api.h")
        .use_core()
        .must_use_type("wasm_bool_t")
        .new_type_alias("Bool32")
        .allowlist_function("^(bt_.*)$")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .parse_callbacks(Box::new(WasmBoolRenameCallbacks))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(binding_file)
        .expect("Couldn't write bindings!");
}

fn compile_basisu_static() {
    for is_cpp in [true, false] {
        let mut build = cc::Build::new();

        if is_bare_wasm() {
            if is_cpp {
                // libc++ headers must come before the C library headers,
                // otherwise libc++'s <cstddef>/<cctype>/... wrappers can't find
                // their own <stddef.h>/<ctype.h> and abort.
                build
                    .includes(wasm_libcxx::includes())
                    .includes(wasm_libc::includes())
                    .cpp_link_stdlib(None);
            } else {
                build.includes(wasm_libc::includes());
            }
        }

        // Use c++_static for Android.
        let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
        if target_os == "android" {
            build.flag("-U_GNU_SOURCE");
            build.cpp_link_stdlib("c++_static");
        }

        build.cpp(is_cpp).std(if is_cpp { "c++17" } else { "c17" });
        for f in FLAGS {
            build.flag_if_supported(f);
        }
        for (define, value) in DEFINES {
            build.define(define, *value);
        }
        if cfg!(feature = "encoder") {
            build.files(
                get_encoder_srcs()
                    .iter()
                    .filter(|src| src.ends_with(if is_cpp { ".cpp" } else { ".c" })),
            );
        } else {
            build.files(
                TRANSCODER_SRCS
                    .iter()
                    .filter(|src| src.ends_with(if is_cpp { ".cpp" } else { ".c" })),
            );
        }
        build.compile(&format!(
            "basisu_c_sys_{}",
            if is_cpp { "cpp" } else { "c" }
        ));
    }
}
