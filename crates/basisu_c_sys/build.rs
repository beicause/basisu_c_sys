const FLAGS: &[&str] = &[
    "-w",
    "-fno-exceptions",
    // Fix gcc optimization issue.
    // See vendor/basis_universal/transcoder/basisu.h
    // See https://github.com/godotengine/godot/pull/114839
    "-fno-strict-aliasing",
];

// Disable PVRTC1/2, ATC, FXT1 as wgpu does not support them.
const DEFINES: &[(&str, &str)] = &[
    // ("BASISU_FORCE_DEVEL_MESSAGES", "1"), // Enable debug message.
    // ("BASISD_SUPPORT_KTX2", "1"),
    // ("BASISD_SUPPORT_KTX2_ZSTD", "1"),
    // ("BASISD_SUPPORT_UASTC", "1"),
    ("BASISD_SUPPORT_DXT1", "0"), //(BC1)
    // ("BASISD_SUPPORT_DXT5A", "1"), //(BC3 / 4 / 5)
    // ("BASISD_SUPPORT_BC7", "1"),
    // ("BASISD_SUPPORT_BC7_MODE5", "1"),
    ("BASISD_SUPPORT_PVRTC1", "0"),
    // ("BASISD_SUPPORT_ETC2_EAC_A8", "1"),
    // ("BASISD_SUPPORT_ASTC", "1"),
    // ("BASISD_SUPPORT_XUASTC", "1"),
    ("BASISD_SUPPORT_ATC", "0"),
    // ("BASISD_SUPPORT_ETC2_EAC_RG11", "1"),
    // ("BASISD_SUPPORT_ASTC_HIGHER_OPAQUE_QUALITY", "1"),
    ("BASISD_SUPPORT_FXT1", "0"),
    ("BASISD_SUPPORT_PVRTC2", "0"),
    // ("BASISD_SUPPORT_UASTC_HDR", "1"),
];

const ENCODER_SRCS: &[&str] = &[
    "vendor/basis_universal/encoder/basisu_astc_hdr_6x6_enc.cpp",
    "vendor/basis_universal/encoder/basisu_astc_hdr_common.cpp",
    "vendor/basis_universal/encoder/basisu_astc_ldr_common.cpp",
    "vendor/basis_universal/encoder/basisu_astc_ldr_encode.cpp",
    "vendor/basis_universal/encoder/basisu_backend.cpp",
    "vendor/basis_universal/encoder/basisu_basis_file.cpp",
    "vendor/basis_universal/encoder/basisu_bc7enc.cpp",
    "vendor/basis_universal/encoder/basisu_comp.cpp",
    "vendor/basis_universal/encoder/basisu_enc.cpp",
    "vendor/basis_universal/encoder/basisu_etc.cpp",
    "vendor/basis_universal/encoder/basisu_frontend.cpp",
    "vendor/basis_universal/encoder/basisu_gpu_texture.cpp",
    "vendor/basis_universal/encoder/basisu_kernels_sse.cpp",
    "vendor/basis_universal/encoder/basisu_opencl.cpp",
    "vendor/basis_universal/encoder/basisu_pvrtc1_4.cpp",
    "vendor/basis_universal/encoder/basisu_resample_filters.cpp",
    "vendor/basis_universal/encoder/basisu_resampler.cpp",
    "vendor/basis_universal/encoder/basisu_ssim.cpp",
    "vendor/basis_universal/encoder/basisu_uastc_enc.cpp",
    "vendor/basis_universal/encoder/basisu_uastc_hdr_4x4_enc.cpp",
    "vendor/basis_universal/encoder/basisu_wasm_api.cpp",
    "vendor/basis_universal/encoder/basisu_wasm_transcoder_api.cpp",
    "vendor/basis_universal/encoder/jpgd.cpp",
    "vendor/basis_universal/encoder/pvpngreader.cpp",
    "vendor/basis_universal/encoder/3rdparty/android_astc_decomp.cpp",
    "vendor/basis_universal/encoder/3rdparty/tinyexr.cpp",
    "vendor/basis_universal/transcoder/basisu_transcoder.cpp",
    "vendor/basis_universal/zstd/zstd.c",
];

const TRANSCODER_SRCS: &[&str] = &[
    "vendor/basis_universal/encoder/basisu_wasm_transcoder_api.cpp",
    "vendor/basis_universal/transcoder/basisu_transcoder.cpp",
    "vendor/basis_universal/zstd/zstddeclib.c",
];

fn main() {
    bindgen();
    wasm_bindgen();
    gen_wasm_build_cmd();
    let target = std::env::var("TARGET").unwrap();
    if target != "wasm32-unknown-unknown" {
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
        std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("basisu_api_common.rs");
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
        std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("basisu_c_api.rs");
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

    let binding_file =
        std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("basisu_c_transcoder_api.rs");
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

fn wasm_bindgen() {
    let encoder_api_file =
        std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("basisu_c_api.rs");
    let transcoder_api_file =
        std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("basisu_c_transcoder_api.rs");
    let encoder_api_file = std::fs::read_to_string(encoder_api_file).unwrap();
    let transcoder_api_file = std::fs::read_to_string(transcoder_api_file).unwrap();
    let mut encoder = vec![
        "#[wasm_bindgen]".to_string(),
        "extern \"C\" {".to_string(),
        "    #[derive(Debug)]".to_string(),
        "    pub type Basisu;".to_string(),
    ];
    encoder.extend(process_file_to_wasm_binding(encoder_api_file));
    let transcoder = process_file_to_wasm_binding(transcoder_api_file);

    encoder.extend(transcoder.iter().cloned());
    encoder.push("}".to_string());
    std::fs::write(
        std::path::Path::new(&std::env::var("OUT_DIR").unwrap())
            .join(format!("wasm_encoder_binding.rs")),
        encoder.join("\n"),
    )
    .unwrap();

    let mut transcoder_file = vec![
        "#[wasm_bindgen]".to_string(),
        "extern \"C\" {".to_string(),
        "    #[derive(Debug)]".to_string(),
        "    pub type Basisu;".to_string(),
    ];
    transcoder_file.extend(transcoder);
    transcoder_file.push("}".to_string());
    std::fs::write(
        std::path::Path::new(&std::env::var("OUT_DIR").unwrap())
            .join(format!("wasm_transcoder_binding.rs")),
        encoder.join("\n"),
    )
    .unwrap();
}

fn process_file_to_wasm_binding(mut api_file: String) -> Vec<String> {
    let s = "pub struct Bool32(pub u32);";
    let pos = api_file.find(s).unwrap() + s.len();
    api_file.replace_range(0..pos, "");
    let lines0: Vec<&str> = api_file.lines().collect();
    let mut lines: Vec<String> = api_file.lines().map(str::to_string).collect();
    for (idx, line) in lines.iter_mut().enumerate() {
        *line = line.replace("Bool32", "u32");
        if line.starts_with(r#"unsafe extern "C" {"#) {
            *line = "    #[wasm_bindgen(method,js_name=_".to_string()
                + extract_func_name(lines0[idx + 1])
                    .or_else(|| extract_func_name(lines0[idx + 2]))
                    .unwrap()
                + ")]";
        } else if line.starts_with("}") {
            *line = "".to_string();
        } else if line.trim_start().starts_with("pub fn ")
            && let Some(end) = line.rfind("(")
        {
            line.insert_str(end + 1, "this: &Basisu,");
        }
    }
    lines
}

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

fn compile_basisu_static() {
    let mut build = cc::Build::new();

    // Use c++_static for Android.
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os == "android" {
        build.cpp_link_stdlib("c++_static").flag("-U_GNU_SOURCE");
    }

    build.cpp(true).std("c++17");
    for f in FLAGS {
        build.flag_if_supported(f);
    }
    for (define, value) in DEFINES {
        build.define(define, *value);
    }
    if cfg!(feature = "encoder") {
        build.files(ENCODER_SRCS);
    } else {
        build.files(TRANSCODER_SRCS);
    }
    build.compile("basisu_c_api_vendor");
}

fn gen_wasm_build_cmd() {
    let encoder_api_file =
        std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("basisu_c_api.rs");
    let transcoder_api_file =
        std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("basisu_c_transcoder_api.rs");
    let mut encoder_apis = Vec::new();
    let mut transcoder_apis = Vec::new();
    for (vec, file) in [
        (&mut encoder_apis, encoder_api_file),
        (&mut transcoder_apis, transcoder_api_file),
    ] {
        for line in std::fs::read_to_string(file).unwrap().lines() {
            if let Some(func) = extract_func_name(line) {
                vec.push(func.to_string());
            }
        }
    }
    encoder_apis.extend(transcoder_apis.iter().cloned());
    for (name, apis, srcs) in [
        ("encoder", encoder_apis, ENCODER_SRCS),
        ("transcoder", transcoder_apis, TRANSCODER_SRCS),
    ] {
        let emcc_args = [
            "-sSTRICT".to_string(),
            "-sEXPORT_ES6".to_string(),
            "-sINCOMING_MODULE_JS_API=wasmBinary".to_string(),
            "-sALLOW_MEMORY_GROWTH".to_string(),
            "-sEXPORTED_RUNTIME_METHODS=HEAPU8".to_string(),
            "-sEXPORTED_FUNCTIONS=".to_string() + &apis.join(","),
        ];
        let mut cmd = std::process::Command::new("em++");
        cmd.args(["-xc++", "-std=c++17"])
            .args(FLAGS)
            .args(
                DEFINES
                    .iter()
                    .map(|(define, value)| format!("-D{define}={value}")),
            )
            .args(emcc_args)
            .args(srcs);
        cmd.args(["-o".to_string(), format!("wasm/basisu_{name}.js")]);
        let default_emcc_args = cmd
            .get_args()
            .map(|s| s.to_string_lossy().to_string())
            .collect::<Vec<String>>();

        std::fs::write(
            std::path::Path::new(&std::env::var("OUT_DIR").unwrap())
                .join(format!("build_{name}_emcc_args.rs")),
            format!(
                "const DEFAULT_{}_EMCC_ARGS: &[&str] = &{:?};",
                name.to_uppercase(),
                default_emcc_args
            ),
        )
        .unwrap();
    }
}
