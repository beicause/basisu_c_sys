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
    // ("BASISD_SUPPORT_XUASTC", "1"),
    ("BASISD_SUPPORT_ATC", "0"),
    // ("BASISD_SUPPORT_ETC2_EAC_RG11", "1"),
    // ("BASISD_SUPPORT_ASTC_HIGHER_OPAQUE_QUALITY", "1"),
    ("BASISD_SUPPORT_FXT1", "0"),
    ("BASISD_SUPPORT_PVRTC2", "0"),
    // ("BASISD_SUPPORT_UASTC_HDR", "1"),
];
const SRCS: &[&str] = &[
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
    "vendor/basis_universal/transcoder/basisu_transcoder.cpp",
    "vendor/basis_universal/zstd/zstd.c",
    "vendor/basis_universal/zstd/zstddeclib.c",
];

fn main() {
    bindgen();
    let target = std::env::var("TARGET").unwrap();
    if target != "wasm32-unknown-unknown" {
        compile_basisu_static();
    } else {
        panic!("bevy_basisu_saver_sys doesn't support wasm32-unknown-unknown yet");
    }
    println!("cargo::rerun-if-changed=vendor/");
}

fn bindgen() {
    let binding_file =
        std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("basisu_c_api.rs");
    bindgen::Builder::default()
        .clang_args(&["-fvisibility=default"])
        .header("vendor/basis_universal/encoder/basisu_wasm_api.h")
        .use_core()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(binding_file)
        .expect("Couldn't write bindings!");

    let binding_file = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap())
        .join("basisu_c_transcoder_api.rs");
    bindgen::Builder::default()
        .clang_args(&["-fvisibility=default"])
        .header("vendor/basis_universal/encoder/basisu_wasm_transcoder_api.h")
        .use_core()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
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
    build.cpp(true).std("c++17").flag_if_supported("-xc++");
    for f in FLAGS {
        build.flag_if_supported(f);
    }
    for (define, value) in DEFINES {
        build.define(define, *value);
    }
    // Enable debug message.
    // build.define("BASISU_FORCE_DEVEL_MESSAGES", "1");
    build.files(SRCS).compile("basisu_c_api_vendor");
}
