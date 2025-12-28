fn main() {
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .warnings(false)
        .flag("-x")
        .flag("c++")
        .flag("-std=c++17")
        .define("BASISU_FORCE_DEVEL_MESSAGES", "1")
        // .define("BASISD_SUPPORT_KTX2", "1")
        // .define("BASISD_SUPPORT_KTX2_ZSTD", "1")
        // .define("BASISD_SUPPORT_UASTC", "1")
        // .define("BASISD_SUPPORT_DXT1", "1") //(BC1)
        // .define("BASISD_SUPPORT_DXT5A", "1") //(BC3 / 4 / 5)
        // .define("BASISD_SUPPORT_BC7", "1")
        // .define("BASISD_SUPPORT_BC7_MODE5", "1")
        .define("BASISD_SUPPORT_PVRTC1", "0")
        // .define("BASISD_SUPPORT_ETC2_EAC_A8", "1")
        // .define("BASISD_SUPPORT_ASTC", "1")
        .define("BASISD_SUPPORT_ATC", "0")
        // .define("BASISD_SUPPORT_ASTC_HIGHER_OPAQUE_QUALITY", "1")
        // .define("BASISD_SUPPORT_ETC2_EAC_RG11", "1")
        .define("BASISD_SUPPORT_FXT1", "0")
        .define("BASISD_SUPPORT_PVRTC2", "0")
        // .define("BASISD_SUPPORT_UASTC_HDR", "1")
        .file("../../vendor/basis_universal/transcoder/basisu_transcoder.cpp")
        .file("../../vendor/transcoding_wrapper.cpp")
        .file("../../vendor/basis_universal/zstd/zstddeclib.c")
        .compile("basisuniversal");

    println!("cargo::rerun-if-changed=../../vendor/");
}
