const FLAGS: [&str; 7] = [
    "-fno-exceptions",
    "-Wno-unused-function",
    "-Wno-unused-const-variable",
    "-Wno-unused-but-set-variable",
    "-Wno-unused-variable",
    "-Wno-unused-value",
    "-Wno-deprecated",
];
// Disable PVRTC1/2, ATC, FXT1 as wgpu does not support them.
const DEFINES: [(&str, &str); 5] = [
    // ("BASISU_FORCE_DEVEL_MESSAGES", "1"),
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
    ("BASISD_SUPPORT_ATC", "0"),
    // ("BASISD_SUPPORT_ASTC_HIGHER_OPAQUE_QUALITY", "1"),
    // ("BASISD_SUPPORT_ETC2_EAC_RG11", "1"),
    ("BASISD_SUPPORT_FXT1", "0"),
    ("BASISD_SUPPORT_PVRTC2", "0"),
    // ("BASISD_SUPPORT_UASTC_HDR", "1"),
];
const SRCS: [&str; 3] = [
    "vendor/basis_universal/transcoder/basisu_transcoder.cpp",
    "vendor/transcoding_wrapper.cpp",
    "vendor/basis_universal/zstd/zstddeclib.c",
];

fn main() {
    bindgen();
    let target = std::env::var("TARGET").unwrap();
    if target != "wasm32-unknown-unknown" {
        compile_basisu_static();
    }
    gen_wasm_build_cmd();
    println!("cargo::rerun-if-changed=vendor/");
}

fn bindgen() {
    let binding_file =
        std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("transcoding.rs");
    bindgen::Builder::default()
        .clang_args(&["-x", "c++", "-std=c++17", "-fvisibility=default"])
        .header("vendor/transcoding_wrapper.hpp")
        .use_core()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_type("Transcoder")
        .allowlist_type("TextureTranscodedFormat")
        .allowlist_type("TextureCompressionMethod")
        .allowlist_function("c_basisu_transcoder_init")
        .allowlist_function("c_ktx2_transcoder_new")
        .allowlist_function("c_ktx2_transcoder_delete")
        .allowlist_function("c_ktx2_transcoder_transcode_image")
        .allowlist_function("c_ktx2_transcoder_get_r_dst_buf")
        .allowlist_function("c_ktx2_transcoder_get_r_dst_buf_len")
        .allowlist_function("c_ktx2_transcoder_get_r_width")
        .allowlist_function("c_ktx2_transcoder_get_r_height")
        .allowlist_function("c_ktx2_transcoder_get_r_levels")
        .allowlist_function("c_ktx2_transcoder_get_r_layers")
        .allowlist_function("c_ktx2_transcoder_get_r_faces")
        .allowlist_function("c_ktx2_transcoder_get_r_target_format")
        .allowlist_function("c_ktx2_transcoder_get_r_is_srgb")
        .opaque_type("Transcoder")
        .bitfield_enum("TextureCompressionMethod")
        .newtype_enum("TextureTranscodedFormat")
        .newtype_enum("ChannelType")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(binding_file)
        .expect("Couldn't write bindings!");
}

fn compile_basisu_static() {
    let mut build = cc::Build::new();
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    // Use c++_static for Android.
    if target_os == "android" {
        build.cpp_link_stdlib("c++_static");
    }
    build.cpp(true).std("c++17");
    for f in FLAGS {
        build.flag_if_supported(f);
    }
    for (define, value) in DEFINES {
        build.define(define, value);
    }
    build.files(&SRCS).compile("basisu_vendor");
}

fn gen_wasm_build_cmd() {
    let wasm_args = [
        "-msimd128",
        "-sSTRICT",
        "-sEXPORT_ES6",
        "-sINCOMING_MODULE_JS_API=wasmBinary",
        "-sALLOW_MEMORY_GROWTH",
        "-sEXPORTED_RUNTIME_METHODS=HEAPU8",
        "-sEXPORTED_FUNCTIONS=_malloc,_free,_c_basisu_transcoder_init,_c_ktx2_transcoder_new,_c_ktx2_transcoder_delete,_c_ktx2_transcoder_transcode_image,_c_ktx2_transcoder_get_r_dst_buf,_c_ktx2_transcoder_get_r_dst_buf_len,_c_ktx2_transcoder_get_r_width,_c_ktx2_transcoder_get_r_height,_c_ktx2_transcoder_get_r_levels,_c_ktx2_transcoder_get_r_layers,_c_ktx2_transcoder_get_r_faces,_c_ktx2_transcoder_get_r_target_format,_c_ktx2_transcoder_get_r_is_srgb",
    ];
    let mut cmd = std::process::Command::new("em++");
    cmd.args(["-xc++", "-std=c++17"])
        .args(FLAGS)
        .args(
            DEFINES
                .iter()
                .map(|(define, value)| format!("-D{define}={value}")),
        )
        .args(wasm_args)
        .args(SRCS);
    cmd.args(["-o", "wasm/basisu_vendor.js"]);
    let default_emcc_args = cmd
        .get_args()
        .map(|s| s.to_string_lossy().to_string())
        .collect::<Vec<String>>();

    std::fs::write(
        std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("build_wasm_emcc_args.rs"),
        format!(
            "const DEFAULT_EMCC_ARGS: [&str; {}] = {:?};",
            default_emcc_args.len(),
            default_emcc_args
        ),
    )
    .unwrap();
}
