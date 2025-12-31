use std::{
    ffi::{OsStr, OsString},
    str::FromStr,
};

fn main() {
    let flags = [
        "-fno-exceptions",
        "-Wno-unused-function",
        "-Wno-unused-const-variable",
        "-Wno-unused-but-set-variable",
        "-Wno-unused-variable",
        "-Wno-unused-value",
        "-Wno-deprecated",
    ];
    // Disable BC1/3/4/5 as we always use BC7 when supported.
    // Disable PVRTC1/2, ATC, FXT1 as wgpu does not support them.
    let defines = [
        // ("BASISU_FORCE_DEVEL_MESSAGES", "1"),
        // ("BASISD_SUPPORT_KTX2", "1"),
        // ("BASISD_SUPPORT_KTX2_ZSTD", "1"),
        // ("BASISD_SUPPORT_UASTC", "1"),
        ("BASISD_SUPPORT_DXT1", "0"),  //(BC1)
        ("BASISD_SUPPORT_DXT5A", "0"), //(BC3 / 4 / 5)
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
    let files = [
        "../../vendor/basis_universal/transcoder/basisu_transcoder.cpp",
        "../../vendor/transcoding_wrapper.cpp",
        "../../vendor/basis_universal/zstd/zstddeclib.c",
    ];
    let target_emcc_env = std::env::var("BASISU_VENDOR_TARGET_EMSCRIPTEN").ok();
    let emcc_args = std::env::var("BASISU_VENDOR_EMCC_ARGS").ok();
    if target_emcc_env
        .map(|v| v.to_lowercase())
        .map(|v| v == "0" || v == "false" || v == "no" || v == "off")
        .unwrap_or(true)
    {
        let mut build = cc::Build::new();
        let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
        // Use c++_static for Android.
        if target_os == "android" {
            build.cpp_link_stdlib("c++_static");
        }
        build.cpp(true).std("c++17").flags(&flags);
        for (define, value) in defines {
            build.define(define, value);
        }
        build.files(&files).compile("basisu_vendor");
    } else {
        let wasm_args = [
            "-msimd128",
            "-sSTRICT",
            "-sEXPORT_ES6",
            "-sALLOW_MEMORY_GROWTH",
            "-sEXPORTED_RUNTIME_METHODS=HEAPU8",
            "-sEXPORTED_FUNCTIONS=_malloc,_free,_c_basisu_transcoder_init,_c_ktx2_transcoder_new,_c_ktx2_transcoder_delete,_c_ktx2_transcoder_transcode_image,_c_ktx2_transcoder_get_r_dst_buf,_c_ktx2_transcoder_get_r_dst_buf_len,_c_ktx2_transcoder_get_r_width,_c_ktx2_transcoder_get_r_height,_c_ktx2_transcoder_get_r_levels,_c_ktx2_transcoder_get_r_layers,_c_ktx2_transcoder_get_r_faces,_c_ktx2_transcoder_get_r_target_format,_c_ktx2_transcoder_get_r_is_srgb",
        ];
        let mut cmd = std::process::Command::new("em++");
        cmd.args(["-xc++", "-std=c++17"])
            .args(flags)
            .args(
                defines
                    .iter()
                    .map(|(define, value)| format!("-D{define}={value}")),
            )
            .args(wasm_args)
            .args(files);
        if let Some(arg) = emcc_args {
            cmd.args(arg.split(" "));
        }
        cmd.args(["-o", "wasm/basisu_vendor.js"]);
        println!(
            "cargo:warning=Build `basisu_vendor.js` using em++ {:?}\x1b[0m",
            cmd.get_args()
                .collect::<Vec<&OsStr>>()
                .join(&OsString::from_str(" ").unwrap())
        );
        let exit_status = cmd.spawn().unwrap().wait().unwrap();
        if !exit_status.success() {
            panic!("emcc didn't exit with success status: {}", exit_status);
        }
        println!(
            "cargo::rerun-if-env-changed=BASISU_VENDOR_TARGET_EMSCRIPTEN,BASISU_VENDOR_EMCC_ARGS"
        );
    }
    println!("cargo::rerun-if-changed=../../vendor/");
}
