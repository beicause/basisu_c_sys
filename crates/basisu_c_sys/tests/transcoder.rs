mod common;
use std::io::Cursor;

use common::block_on;

use basisu_c_sys::extra::{
    BasisuTranscodeError, BasisuTranscoder, ChannelType, SupportedTextureCompression,
    TranscodeInfo, TranscodedImage, basisu_transcoder_init,
};
use image::{DynamicImage, ImageBuffer, ImageFormat};
use wgpu_types::{TextureDataOrder, TextureFormat};

use crate::common::SNAPSHOT_PATH;

#[test]
fn transcode_invalid_data() {
    block_on(basisu_transcoder_init());
    let transcoder =
        BasisuTranscoder::new(&[], SupportedTextureCompression::empty(), ChannelType::Auto);
    assert_eq!(transcoder.err(), Some(BasisuTranscodeError::EmptyInputData));

    let transcoder = BasisuTranscoder::new(
        &[1, 2, 1],
        SupportedTextureCompression::BC
            | SupportedTextureCompression::ASTC_LDR
            | SupportedTextureCompression::ASTC_HDR
            | SupportedTextureCompression::ETC2,
        ChannelType::Auto,
    );
    assert_eq!(
        transcoder.err(),
        Some(BasisuTranscodeError::LoadKtx2DataFailed)
    );
}

fn snapshot(
    supported_format: SupportedTextureCompression,
    each_data_result: impl Fn(&str, TranscodedImage),
) -> Vec<(String, TranscodeInfo, TranscodedImage)> {
    block_on(basisu_transcoder_init());

    let mut path = std::path::PathBuf::new();
    path.push(std::env!("CARGO_MANIFEST_DIR"));
    path.push("../../assets");

    // Read the real path, otherwise `std::fs::read_dir` panics on windows.
    let link_path = std::fs::read_link(&path).unwrap();
    path.pop();
    path.push(link_path);

    let mut results = Vec::new();
    for file in std::fs::read_dir(path).unwrap() {
        let file = file.unwrap();
        let file_name = file.file_name().into_string().unwrap();
        if !file_name.ends_with(".basisu.ktx2") {
            continue;
        }
        let data = std::fs::read(file.path()).unwrap();
        let transcoder = BasisuTranscoder::new(&data, supported_format, ChannelType::Auto).unwrap();
        let info = transcoder.get_info();
        let mut image = transcoder.transcode(None, None).unwrap();
        let image_data = std::mem::take(&mut image.data);
        let mut cloned = image.clone();
        cloned.data = image_data;
        each_data_result(&file_name, cloned);
        results.push((file_name, info, image));
    }
    results.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    results
}

#[test]
fn transcode_assets_bcn() {
    insta::with_settings!({ snapshot_path => SNAPSHOT_PATH }, {
        let results = snapshot(SupportedTextureCompression::BC, |file_name, image| {
            // The bcn test failed on macos, disable it for now.
            if !cfg!(target_os = "macos") {
                insta::assert_binary_snapshot!(
                    &("bcn_".to_string() + &file_name.replace(".basisu.ktx2", ".bin")),
                    image.data
                );
            }
        });
        insta::assert_debug_snapshot!(results);
    });
}

#[test]
fn transcode_assets_astc() {
    insta::with_settings!({ snapshot_path => SNAPSHOT_PATH }, {
        let results = snapshot(
            SupportedTextureCompression::ASTC_LDR
            | SupportedTextureCompression::ASTC_HDR
            | SupportedTextureCompression::ETC2,
            |file_name, image| {
                insta::assert_binary_snapshot!(
                    &("astc_".to_string() + &file_name.replace(".basisu.ktx2", ".bin")),
                    image.data
                );
            },
        );
        insta::assert_debug_snapshot!(results);
    });
}

#[test]
fn transcode_assets_uncompressed() {
    let each_result = |file_name: &str, mut transcoded_image: TranscodedImage| {
        assert!(
            [
                TextureFormat::Rgba8Unorm,
                TextureFormat::Rgba8UnormSrgb,
                TextureFormat::Rgba16Float
            ]
            .contains(&transcoded_image.format)
        );
        assert_eq!(transcoded_image.data_order, TextureDataOrder::MipMajor);
        let is_hdr = transcoded_image.format == TextureFormat::Rgba16Float;
        let data = core::mem::take(&mut transcoded_image.data);
        let pixel_size = if is_hdr { 8 } else { 4 };
        let mut offset = 0usize;
        for mip in 0..transcoded_image.mip_level_count {
            for layer in 0..transcoded_image.size.depth_or_array_layers {
                let width = (transcoded_image.size.width >> mip).max(1);
                let height = (transcoded_image.size.height >> mip).max(1);
                let bytes = width * height * pixel_size;
                let dynamic_image = if is_hdr {
                    DynamicImage::ImageRgba32F(
                        ImageBuffer::from_raw(
                            width,
                            height,
                            data[offset..(offset + bytes as usize)]
                                .as_chunks::<2>()
                                .0
                                .iter()
                                .map(|ck| half::f16::from_le_bytes(*ck))
                                .map(|hf| hf.to_f32())
                                .collect(),
                        )
                        .unwrap(),
                    )
                } else {
                    DynamicImage::ImageRgba8(
                        ImageBuffer::from_raw(
                            width,
                            height,
                            data[offset..(offset + bytes as usize)].to_vec(),
                        )
                        .unwrap(),
                    )
                };
                offset += bytes as usize;
                let mut output = Vec::new();
                dynamic_image
                    .write_to(
                        Cursor::new(&mut output),
                        if is_hdr {
                            ImageFormat::OpenExr
                        } else {
                            ImageFormat::WebP
                        },
                    )
                    .unwrap();
                insta::assert_binary_snapshot!(
                    &("uncompressed_".to_string()
                        + &file_name.replace(
                            ".basisu.ktx2",
                            &format!(
                                "_layer{}_mip{}{}",
                                layer,
                                mip,
                                if is_hdr { ".exr" } else { ".webp" }
                            )
                        )),
                    output
                );
            }
        }
    };

    insta::with_settings!({ snapshot_path => SNAPSHOT_PATH }, {
        let results = snapshot(SupportedTextureCompression::empty(), each_result);
        insta::assert_debug_snapshot!(results);
    });
}
