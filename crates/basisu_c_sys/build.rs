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
    "vendor/basis_universal/encoder/3rdparty/android_astc_decomp.cpp",
    "vendor/basis_universal/encoder/3rdparty/tinyexr.cpp",
    "vendor/basis_universal/transcoder/basisu_transcoder.cpp",
    "vendor/basis_universal/zstd/zstd.c",
];

fn main() {
    bindgen();
    let target = std::env::var("TARGET").unwrap();
    if target != "wasm32-unknown-unknown" {
        compile_basisu_static();
    } else {
        panic!("basisu_c_sys doesn't support wasm32-unknown-unknown yet");
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
        std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("basisu_api_common.rs");
    bindgen::Builder::default()
        .clang_args(&["-fvisibility=default"])
        .header("vendor/basis_universal/encoder/basisu_wasm_api_common.h")
        .use_core()
        .allowlist_type("wasm_bool_t")
        .new_type_alias("wasm_bool_t")
        .allowlist_var("BU_QUALITY_MIN")
        .allowlist_var("BU_QUALITY_MAX")
        .allowlist_var("BU_EFFORT_MIN")
        .allowlist_var("BU_EFFORT_MAX")
        .allowlist_var("BU_EFFORT_SUPER_FAST")
        .allowlist_var("BU_EFFORT_FAST")
        .allowlist_var("BU_EFFORT_NORMAL")
        .allowlist_var("BU_EFFORT_DEFAULT")
        .allowlist_var("BU_EFFORT_SLOW")
        .allowlist_var("BU_EFFORT_VERY_SLOW")
        .allowlist_var("BU_COMP_FLAGS_NONE")
        .allowlist_var("BU_COMP_FLAGS_USE_OPENCL")
        .allowlist_var("BU_COMP_FLAGS_THREADED")
        .allowlist_var("BU_COMP_FLAGS_DEBUG_OUTPUT")
        .allowlist_var("BU_COMP_FLAGS_KTX2_OUTPUT")
        .allowlist_var("BU_COMP_FLAGS_KTX2_UASTC_ZSTD")
        .allowlist_var("BU_COMP_FLAGS_SRGB")
        .allowlist_var("BU_COMP_FLAGS_GEN_MIPS_CLAMP")
        .allowlist_var("BU_COMP_FLAGS_GEN_MIPS_WRAP")
        .allowlist_var("BU_COMP_FLAGS_Y_FLIP")
        .allowlist_var("BU_COMP_FLAGS_PRINT_STATS")
        .allowlist_var("BU_COMP_FLAGS_PRINT_STATUS")
        .allowlist_var("BU_COMP_FLAGS_DEBUG_IMAGES")
        .allowlist_var("BU_COMP_FLAGS_REC2020")
        .allowlist_var("BU_COMP_FLAGS_VALIDATE_OUTPUT")
        .allowlist_var("BU_COMP_FLAGS_XUASTC_LDR_FULL_ARITH")
        .allowlist_var("BU_COMP_FLAGS_XUASTC_LDR_HYBRID")
        .allowlist_var("BU_COMP_FLAGS_XUASTC_LDR_FULL_ZSTD")
        .allowlist_var("BU_COMP_FLAGS_XUASTC_LDR_SYNTAX_SHIFT")
        .allowlist_var("BU_COMP_FLAGS_XUASTC_LDR_SYNTAX_MASK")
        .allowlist_var("BU_COMP_FLAGS_TEXTURE_TYPE_2D")
        .allowlist_var("BU_COMP_FLAGS_TEXTURE_TYPE_2D_ARRAY")
        .allowlist_var("BU_COMP_FLAGS_TEXTURE_TYPE_CUBEMAP_ARRAY")
        .allowlist_var("BU_COMP_FLAGS_TEXTURE_TYPE_VIDEO_FRAMES")
        .allowlist_var("BU_COMP_FLAGS_TEXTURE_TYPE_SHIFT")
        .allowlist_var("BU_COMP_FLAGS_TEXTURE_TYPE_MASK")
        .allowlist_var("BU_COMP_FLAGS_VERBOSE")
        .allowlist_var("BTF_ETC1S")
        .allowlist_var("BTF_UASTC_LDR_4X4")
        .allowlist_var("BTF_UASTC_HDR_4X4")
        .allowlist_var("BTF_ASTC_HDR_6X6")
        .allowlist_var("BTF_UASTC_HDR_6X6")
        .allowlist_var("BTF_XUASTC_LDR_4X4")
        .allowlist_var("BTF_XUASTC_LDR_5X4")
        .allowlist_var("BTF_XUASTC_LDR_5X5")
        .allowlist_var("BTF_XUASTC_LDR_6X5")
        .allowlist_var("BTF_XUASTC_LDR_6X6")
        .allowlist_var("BTF_XUASTC_LDR_8X5")
        .allowlist_var("BTF_XUASTC_LDR_8X6")
        .allowlist_var("BTF_XUASTC_LDR_10X5")
        .allowlist_var("BTF_XUASTC_LDR_10X6")
        .allowlist_var("BTF_XUASTC_LDR_8X8")
        .allowlist_var("BTF_XUASTC_LDR_10X8")
        .allowlist_var("BTF_XUASTC_LDR_10X10")
        .allowlist_var("BTF_XUASTC_LDR_12X10")
        .allowlist_var("BTF_XUASTC_LDR_12X12")
        .allowlist_var("BTF_ASTC_LDR_4X4")
        .allowlist_var("BTF_ASTC_LDR_5X4")
        .allowlist_var("BTF_ASTC_LDR_5X5")
        .allowlist_var("BTF_ASTC_LDR_6X5")
        .allowlist_var("BTF_ASTC_LDR_6X6")
        .allowlist_var("BTF_ASTC_LDR_8X5")
        .allowlist_var("BTF_ASTC_LDR_8X6")
        .allowlist_var("BTF_ASTC_LDR_10X5")
        .allowlist_var("BTF_ASTC_LDR_10X6")
        .allowlist_var("BTF_ASTC_LDR_8X8")
        .allowlist_var("BTF_ASTC_LDR_10X8")
        .allowlist_var("BTF_ASTC_LDR_10X10")
        .allowlist_var("BTF_ASTC_LDR_12X10")
        .allowlist_var("BTF_ASTC_LDR_12X12")
        .allowlist_var("BTF_TOTAL_FORMATS")
        .allowlist_var("TF_ETC1_RGB")
        .allowlist_var("TF_ETC2_RGBA")
        .allowlist_var("TF_BC1_RGB")
        .allowlist_var("TF_BC3_RGBA")
        .allowlist_var("TF_BC4_R")
        .allowlist_var("TF_BC5_RG")
        .allowlist_var("TF_BC7_RGBA")
        .allowlist_var("TF_PVRTC1_4_RGB")
        .allowlist_var("TF_PVRTC1_4_RGBA")
        .allowlist_var("TF_ASTC_LDR_4X4_RGBA")
        .allowlist_var("TF_ATC_RGB")
        .allowlist_var("TF_ATC_RGBA")
        .allowlist_var("TF_FXT1_RGB")
        .allowlist_var("TF_PVRTC2_4_RGB")
        .allowlist_var("TF_PVRTC2_4_RGBA")
        .allowlist_var("TF_ETC2_EAC_R11")
        .allowlist_var("TF_ETC2_EAC_RG11")
        .allowlist_var("TF_BC6H")
        .allowlist_var("TF_ASTC_HDR_4X4_RGBA")
        .allowlist_var("TF_RGBA32")
        .allowlist_var("TF_RGB565")
        .allowlist_var("TF_BGR565")
        .allowlist_var("TF_RGBA4444")
        .allowlist_var("TF_RGB_HALF")
        .allowlist_var("TF_RGBA_HALF")
        .allowlist_var("TF_RGB_9E5")
        .allowlist_var("TF_ASTC_HDR_6X6_RGBA")
        .allowlist_var("TF_ASTC_LDR_5X4_RGBA")
        .allowlist_var("TF_ASTC_LDR_5X5_RGBA")
        .allowlist_var("TF_ASTC_LDR_6X5_RGBA")
        .allowlist_var("TF_ASTC_LDR_6X6_RGBA")
        .allowlist_var("TF_ASTC_LDR_8X5_RGBA")
        .allowlist_var("TF_ASTC_LDR_8X6_RGBA")
        .allowlist_var("TF_ASTC_LDR_10X5_RGBA")
        .allowlist_var("TF_ASTC_LDR_10X6_RGBA")
        .allowlist_var("TF_ASTC_LDR_8X8_RGBA")
        .allowlist_var("TF_ASTC_LDR_10X8_RGBA")
        .allowlist_var("TF_ASTC_LDR_10X10_RGBA")
        .allowlist_var("TF_ASTC_LDR_12X10_RGBA")
        .allowlist_var("TF_ASTC_LDR_12X12_RGBA")
        .allowlist_var("TF_TOTAL_TEXTURE_FORMATS")
        .allowlist_var("DECODE_FLAGS_PVRTC_DECODE_TO_NEXT_POW2")
        .allowlist_var("DECODE_FLAGS_TRANSCODE_ALPHA_DATA_TO_OPAQUE_FORMATS")
        .allowlist_var("DECODE_FLAGS_BC1_FORBID_THREE_COLOR_BLOCKS")
        .allowlist_var("DECODE_FLAGS_OUTPUT_HAS_ALPHA_INDICES")
        .allowlist_var("DECODE_FLAGS_HIGH_QUALITY")
        .allowlist_var("DECODE_FLAGS_NO_ETC1S_CHROMA_FILTERING")
        .allowlist_var("DECODE_FLAGS_NO_DEBLOCK_FILTERING")
        .allowlist_var("DECODE_FLAGS_STRONGER_DEBLOCK_FILTERING")
        .allowlist_var("DECODE_FLAGS_FORCE_DEBLOCK_FILTERING")
        .allowlist_var("DECODE_FLAGS_XUASTC_LDR_DISABLE_FAST_BC7_TRANSCODING")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .parse_callbacks(Box::new(MacroTypeCallbacks))
        .parse_callbacks(Box::new(WasmBoolRenameCallbacks))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(binding_file)
        .expect("Couldn't write bindings!");

    let binding_file =
        std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("basisu_c_api.rs");
    bindgen::Builder::default()
        .clang_args(&["-fvisibility=default"])
        .header("vendor/basis_universal/encoder/basisu_wasm_api.h")
        .use_core()
        .allowlist_function("bu_get_version")
        .allowlist_function("bu_enable_debug_printf")
        .allowlist_function("bu_init")
        .allowlist_function("bu_alloc")
        .allowlist_function("bu_free")
        .allowlist_function("bu_new_comp_params")
        .allowlist_function("bu_delete_comp_params")
        .allowlist_function("bu_comp_params_get_comp_data_size")
        .allowlist_function("bu_comp_params_get_comp_data_ofs")
        .allowlist_function("bu_comp_params_clear")
        .allowlist_function("bu_comp_params_set_image_rgba32")
        .allowlist_function("bu_comp_params_set_image_float_rgba")
        .allowlist_function("bu_compress_texture")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .parse_callbacks(Box::new(WasmBoolRenameCallbacks))
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
        .allowlist_function("bt_get_version")
        .allowlist_function("bt_enable_debug_printf")
        .allowlist_function("bt_init")
        .allowlist_function("bt_alloc")
        .allowlist_function("bt_free")
        .allowlist_function("bt_basis_tex_format_is_xuastc_ldr")
        .allowlist_function("bt_basis_tex_format_is_astc_ldr")
        .allowlist_function("bt_basis_tex_format_get_block_width")
        .allowlist_function("bt_basis_tex_format_get_block_height")
        .allowlist_function("bt_basis_tex_format_is_hdr")
        .allowlist_function("bt_basis_tex_format_is_ldr")
        .allowlist_function("bt_basis_get_bytes_per_block_or_pixel")
        .allowlist_function("bt_basis_transcoder_format_has_alpha")
        .allowlist_function("bt_basis_transcoder_format_is_hdr")
        .allowlist_function("bt_basis_transcoder_format_is_ldr")
        .allowlist_function("bt_basis_transcoder_texture_format_is_astc")
        .allowlist_function("bt_basis_transcoder_format_is_uncompressed")
        .allowlist_function("bt_basis_get_uncompressed_bytes_per_pixel")
        .allowlist_function("bt_basis_get_block_width")
        .allowlist_function("bt_basis_get_block_height")
        .allowlist_function("bt_basis_get_transcoder_texture_format_from_basis_tex_format")
        .allowlist_function("bt_basis_is_format_supported")
        .allowlist_function("bt_basis_compute_transcoded_image_size_in_bytes")
        .allowlist_function("bt_ktx2_open")
        .allowlist_function("bt_ktx2_close")
        .allowlist_function("bt_ktx2_get_width")
        .allowlist_function("bt_ktx2_get_height")
        .allowlist_function("bt_ktx2_get_levels")
        .allowlist_function("bt_ktx2_get_faces")
        .allowlist_function("bt_ktx2_get_layers")
        .allowlist_function("bt_ktx2_get_basis_tex_format")
        .allowlist_function("bt_ktx2_is_etc1s")
        .allowlist_function("bt_ktx2_is_uastc_ldr_4x4")
        .allowlist_function("bt_ktx2_is_hdr")
        .allowlist_function("bt_ktx2_is_hdr_4x4")
        .allowlist_function("bt_ktx2_is_hdr_6x6")
        .allowlist_function("bt_ktx2_is_ldr")
        .allowlist_function("bt_ktx2_is_astc_ldr")
        .allowlist_function("bt_ktx2_is_xuastc_ldr")
        .allowlist_function("bt_ktx2_get_block_width")
        .allowlist_function("bt_ktx2_get_block_height")
        .allowlist_function("bt_ktx2_has_alpha")
        .allowlist_function("bt_ktx2_get_dfd_color_model")
        .allowlist_function("bt_ktx2_get_dfd_color_primaries")
        .allowlist_function("bt_ktx2_get_dfd_transfer_func")
        .allowlist_function("bt_ktx2_is_srgb")
        .allowlist_function("bt_ktx2_get_dfd_flags")
        .allowlist_function("bt_ktx2_get_dfd_total_samples")
        .allowlist_function("bt_ktx2_get_dfd_channel_id0")
        .allowlist_function("bt_ktx2_get_dfd_channel_id1")
        .allowlist_function("bt_ktx2_is_video")
        .allowlist_function("bt_ktx2_get_ldr_hdr_upconversion_nit_multiplier")
        .allowlist_function("bt_ktx2_get_level_orig_width")
        .allowlist_function("bt_ktx2_get_level_orig_height")
        .allowlist_function("bt_ktx2_get_level_actual_width")
        .allowlist_function("bt_ktx2_get_level_actual_height")
        .allowlist_function("bt_ktx2_get_level_num_blocks_x")
        .allowlist_function("bt_ktx2_get_level_num_blocks_y")
        .allowlist_function("bt_ktx2_get_level_total_blocks")
        .allowlist_function("bt_ktx2_get_level_alpha_flag")
        .allowlist_function("bt_ktx2_get_level_iframe_flag")
        .allowlist_function("bt_ktx2_start_transcoding")
        .allowlist_function("bt_ktx2_create_transcode_state")
        .allowlist_function("bt_ktx2_destroy_transcode_state")
        .allowlist_function("bt_ktx2_transcode_image_level")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .parse_callbacks(Box::new(WasmBoolRenameCallbacks))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(binding_file)
        .expect("Couldn't write bindings!");
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
    build.files(SRCS).compile("basisu_c_api_vendor");
}
