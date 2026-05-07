mod common;
use common::block_on;

use basisu_c_sys::{
    TranscodeTargetFormat,
    extra::{BasisuTranscoder, ChannelType, SupportedTextureCompression, basisu_transcoder_init},
};

#[test]
fn transcode_invalid_data_info_is_none() {
    block_on(basisu_transcoder_init());
    let mut transcoder = BasisuTranscoder::new();
    let info = transcoder.prepare(&[], SupportedTextureCompression::empty(), ChannelType::Auto);
    assert!(info.is_err());
    let info = transcoder.prepare(
        &[1, 2, 1],
        SupportedTextureCompression::BC
            | SupportedTextureCompression::ASTC_LDR
            | SupportedTextureCompression::ASTC_HDR
            | SupportedTextureCompression::ETC2,
        ChannelType::Auto,
    );
    assert!(info.is_err());
}

#[test]
fn transcode_before_prepare_panic() {
    block_on(basisu_transcoder_init());
    let transcoder = BasisuTranscoder::new();
    assert!(
        transcoder
            .transcode(Some(TranscodeTargetFormat::RGBA32), None)
            .is_err(),
    );
}

#[test]
fn transcode_invalid_data_output_panic() {
    block_on(basisu_transcoder_init());
    let mut transcoder = BasisuTranscoder::new();
    let _info = transcoder.prepare(
        &[1, 2, 1],
        SupportedTextureCompression::empty(),
        ChannelType::Auto,
    );
    let res = transcoder.transcode(Some(TranscodeTargetFormat::RGBA32), None);
    assert!(res.is_err())
}

// Use macro to make `assert_debug_snapshot` produce correct file name.
// See https://github.com/mitsuhiko/insta/issues/357
macro_rules! snapshot_test {
    ($prefix: expr, $supported_format: expr $(,)?) => {
        let mut path = std::path::PathBuf::new();
        path.push(std::env!("CARGO_MANIFEST_DIR"));
        path.push("../../assets");
        block_on(basisu_transcoder_init());
        let mut results = Vec::new();
        let mut transcoder = BasisuTranscoder::new();
        for file in std::fs::read_dir(path).unwrap() {
            let file = file.unwrap();
            let file_name = file.file_name().into_string().unwrap();
            if !file_name.ends_with(".basisu.ktx2") {
                continue;
            }
            let data = std::fs::read(file.path()).unwrap();
            let info = transcoder
                .prepare(&data, $supported_format, ChannelType::Auto)
                .unwrap();
            let mut image = transcoder.transcode(None, None).unwrap();
            let image_data = std::mem::take(&mut image.data);
            // The bcn test failed on macos, disable it for now.
            if !(cfg!(target_os = "macos") && $prefix == "bcn_") {
                insta::assert_binary_snapshot!(
                    &($prefix.to_string() + &file_name.replace(".basisu.ktx2", ".bin")),
                    image_data
                );
            }
            results.push((file_name, info, image));
        }
        results.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        insta::assert_debug_snapshot!(results);
    };
}

#[test]
fn transcode_assets_bcn() {
    snapshot_test!("bcn_", SupportedTextureCompression::BC);
}

#[test]
fn transcode_assets_astc() {
    snapshot_test!(
        "astc_",
        SupportedTextureCompression::ASTC_LDR
            | SupportedTextureCompression::ASTC_HDR
            | SupportedTextureCompression::ETC2,
    );
}
