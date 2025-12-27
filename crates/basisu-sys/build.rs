use std::path::PathBuf;

fn main() {
    let bindings_out_file = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("transcoding.rs");
    bindgen::Builder::default()
        .header("../../vendor/transcoding.hpp")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .opaque_type("basist::ktx2_transcoder")
        .bitfield_enum("TextureCompressionMethod")
        .newtype_enum("TextureTranscodedFormat")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(bindings_out_file)
        .expect("Couldn't write bindings!");

    let mut build = cc::Build::new();
    build
        .flag_if_supported("-fvisibility=hidden")
        .flag_if_supported("-fno-strict-aliasing")
        .flag_if_supported("-Wall")
        .flag_if_supported("-Wextra")
        .flag_if_supported("-Wno-unused-local-typedefs")
        .flag_if_supported("-Wno-unused-value")
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-unused-variable")
        .flag_if_supported("-Wno-unused-but-set-variable")
        .cpp(true)
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
        .flag_if_supported("--std=c++17")
        .file("../../vendor/basis_universal/transcoder/basisu_transcoder.cpp")
        .file("../../vendor/basis_universal/zstd/zstddeclib.c")
        .file("../../vendor/transcoding.cpp")
        .compile("basisuniversal");

    println!("cargo::rerun-if-changed=../../vendor/");
}
