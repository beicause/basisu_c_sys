use std::sync::OnceLock;

mod wasm_bindgen;

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
    // See vendor/basis_universal/transcoder/basisu.h
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

static ENCODER_SRCS: OnceLock<Vec<String>> = OnceLock::new();

fn get_encoder_srcs() -> &'static [String] {
    let dir = env!("CARGO_MANIFEST_DIR");
    let srcs = &[
        "vendor/basis_universal/transcoder/basisu_transcoder.cpp",
        "vendor/basis_universal/zstd/zstd.c",
    ];

    ENCODER_SRCS.get_or_init(|| {
        let mut vec = Vec::new();
        search_files(
            std::path::PathBuf::from_iter([dir, "vendor/basis_universal/encoder/"]),
            "cpp",
            &mut vec,
        );
        vec.extend(srcs.map(ToString::to_string));
        vec
    })
}

const TRANSCODER_SRCS: &[&str] = &[
    "vendor/basis_universal/encoder/basisu_wasm_transcoder_api.cpp",
    "vendor/basis_universal/transcoder/basisu_transcoder.cpp",
    "vendor/basis_universal/zstd/zstddeclib.c",
];

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/write_cmake_args.rs"));

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
    let target = std::env::var("TARGET").unwrap();

    if target == "wasm32-unknown-unknown" {
        wasm_bindgen::generate();
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let target_feature = std::env::var("CARGO_CFG_TARGET_FEATURE").unwrap();
        let args_dir = std::path::PathBuf::from_iter([&out_dir, "build_args"]);
        match std::fs::create_dir(&args_dir) {
            Ok(_) => {}
            Err(err) => assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists),
        }

        let dst = if !is_docs_rs {
            let (mut default_encoder_emcc_args, mut default_transcoder_emcc_args) =
                (Vec::<String>::new(), Vec::<String>::new());
            get_wasm_build_args(
                &mut default_encoder_emcc_args,
                &mut default_transcoder_emcc_args,
            );
            write_cmake_args(
                &default_encoder_emcc_args,
                &default_transcoder_emcc_args,
                get_encoder_srcs(),
                TRANSCODER_SRCS,
                target_feature.contains("simd128").then_some("-msimd128"),
                &args_dir,
            );
            let mut cmake = cmake::Config::new(".");
            cmake
                .profile("")
                .target("wasm32-unknown-emscripten")
                .define("BUILD_ARGS_DIR", &args_dir)
                .build_target("transcoder");
            if std::env::var("PROFILE").unwrap() != "debug" {
                cmake.cflag("-flto=full").cxxflag("-flto=full");
            }
            let opt_flag = "-O".to_string() + &std::env::var("OPT_LEVEL").unwrap();
            cmake.cflag(&opt_flag).cxxflag(&opt_flag);
            #[cfg(feature = "encoder")]
            cmake.build_target("all");
            cmake.build()
        } else {
            // Write empty js and wasm files to work around cargo docs-rs.
            match std::fs::create_dir(std::path::PathBuf::from_iter([&out_dir, "build"])) {
                Ok(_) => {}
                Err(err) => assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists),
            }
            for name in ["encoder", "transcoder"] {
                let path =
                    std::path::PathBuf::from_iter([&out_dir, &format!("build/basisu_{name}.js")]);
                std::fs::write(path, []).unwrap();
                let path =
                    std::path::PathBuf::from_iter([&out_dir, &format!("build/basisu_{name}.wasm")]);
                std::fs::write(path, []).unwrap();
            }
            (&out_dir).into()
        };

        for name in ["encoder", "transcoder"] {
            #[cfg(not(feature = "encoder"))]
            if name == "encoder" {
                continue;
            }
            let path = dst.join(format!("build/basisu_{name}.js"));
            std::fs::write(
                std::path::PathBuf::from_iter([&out_dir, &format!("wasm_{name}_inline_js.rs")]),
                format!(
                    r##"#[wasm_bindgen(inline_js = r#"{}"#)]
                        extern "C" {{
                            #[wasm_bindgen(js_name = "default")]
                            pub async fn new_instance(args: &Object) -> Basisu;
                            }}"##,
                    std::fs::read_to_string(path).unwrap()
                ),
            )
            .unwrap();
            println!("cargo::rerun-if-changed=CMakeLists.txt");
        }
    }

    if !is_docs_rs && target != "wasm32-unknown-unknown" {
        compile_basisu_static();
    }

    println!("cargo::rerun-if-changed=vendor/");
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
    let binding_file =
        std::path::PathBuf::from_iter([&std::env::var("OUT_DIR").unwrap(), "basisu_api_common.rs"]);
    bindgen::Builder::default()
        .clang_args(&["-fvisibility=default"])
        .header("vendor/basis_universal/encoder/basisu_wasm_api_common.h")
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
        .header("vendor/basis_universal/encoder/basisu_wasm_api.h")
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
        .header("vendor/basis_universal/encoder/basisu_wasm_transcoder_api.h")
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

fn get_wasm_build_args(
    default_encoder_emcc_args: &mut Vec<String>,
    default_transcoder_emcc_args: &mut Vec<String>,
) {
    let encoder_api_file =
        std::path::PathBuf::from_iter([&std::env::var("OUT_DIR").unwrap(), "basisu_c_api.rs"]);
    let transcoder_api_file = std::path::PathBuf::from_iter([
        &std::env::var("OUT_DIR").unwrap(),
        "basisu_c_transcoder_api.rs",
    ]);
    let mut encoder_apis = Vec::new();
    let mut transcoder_apis = Vec::new();

    fn extract_func_name(line: &str) -> Option<&str> {
        let line = line.trim_start();
        if let Some(start) = line.find("pub fn ")
            && let Some(end) = line.rfind("(")
        {
            Some(&line[(start + "pub fn ".len())..end])
        } else {
            None
        }
    }

    for (vec, file) in [
        (&mut encoder_apis, encoder_api_file),
        (&mut transcoder_apis, transcoder_api_file),
    ] {
        for line in std::fs::read_to_string(file).unwrap().lines() {
            if let Some(func) = extract_func_name(line) {
                vec.push("_".to_string() + func);
            }
        }
    }
    encoder_apis.extend(transcoder_apis.iter().cloned());
    for (default_emcc_args, apis) in [
        (default_encoder_emcc_args, encoder_apis),
        (default_transcoder_emcc_args, transcoder_apis),
    ] {
        let emcc_args = [
            "-sSTRICT".to_string(),
            "-sEXPORT_ES6".to_string(),
            "-sINCOMING_MODULE_JS_API=wasmBinary".to_string(),
            "-sALLOW_MEMORY_GROWTH".to_string(),
            "-sEXPORTED_RUNTIME_METHODS=HEAPU8".to_string(),
            "-sEXPORTED_FUNCTIONS=".to_string() + &apis.join(","),
        ];

        default_emcc_args.extend(
            FLAGS
                .iter()
                .filter(|f| !f.starts_with("/wd"))
                .map(ToString::to_string),
        );
        default_emcc_args.extend(
            DEFINES
                .iter()
                .map(|(define, value)| format!("-D{define}={value}")),
        );
        default_emcc_args.extend(emcc_args);
    }
}
