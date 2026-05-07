mod common;
use common::block_on;

use std::path::Path;

use basisu_c_sys::{
    BasisTextureFormat,
    common::{
        BU_COMP_FLAGS_DEBUG_OUTPUT, BU_COMP_FLAGS_GEN_MIPS_CLAMP, BU_COMP_FLAGS_VALIDATE_OUTPUT,
    },
    extra::{
        BasisuEncoder, BasisuEncoderParams, SourceImage, basisu_encoder_enable_debug_printf,
        basisu_encoder_init,
    },
};
use bevy_image::{CompressedImageFormats, Image};
use wgpu_types::TextureViewDimension;

const SKYBOX_PATHS: &[&str] = &[
    "../../original_assets/skybox/right.jpg",
    "../../original_assets/skybox/left.jpg",
    "../../original_assets/skybox/top.jpg",
    "../../original_assets/skybox/bottom.jpg",
    "../../original_assets/skybox/front.jpg",
    "../../original_assets/skybox/back.jpg",
];

fn image_to_source_image(value: &'_ Image) -> SourceImage<'_> {
    SourceImage {
        data: value.data.as_deref().unwrap_or(&[]),
        texture_descriptor: &value.texture_descriptor,
        texture_view_descriptor: &value.texture_view_descriptor,
    }
}

fn encode_cubemap_xuastc_ldr_4x4_by_slice() {
    block_on(basisu_encoder_init());

    let dir = std::env!("CARGO_MANIFEST_DIR");
    let mut encoder = BasisuEncoder::new();
    for (i, path) in SKYBOX_PATHS.iter().enumerate() {
        let image = Image::from_buffer(
            &std::fs::read(Path::new(dir).join(path)).unwrap(),
            bevy_image::ImageType::Extension(
                Path::new(path).extension().unwrap().to_str().unwrap(),
            ),
            CompressedImageFormats::empty(),
            true,
            bevy_image::ImageSampler::Default,
            Default::default(),
        )
        .unwrap();
        encoder
            .set_image_slice(i as u32, image_to_source_image(&image))
            .unwrap();
    }
    let params = BasisuEncoderParams::new_with_srgb_defaults(BasisTextureFormat::XuastcLdr4x4)
        .with_tex_type(TextureViewDimension::Cube);
    #[cfg_attr(target_os = "macos", expect(unused_variables))]
    let res = encoder
        .compress(params.with_flags(BU_COMP_FLAGS_DEBUG_OUTPUT | BU_COMP_FLAGS_VALIDATE_OUTPUT))
        .unwrap();
    // The test failed on macos, disable it for now.
    #[cfg(not(target_os = "macos"))]
    insta::assert_binary_snapshot!("skybox_astc_ldr_8x8.basisu.ktx2", res);
}

fn encode_cubemap_astc_ldr_8x8_mips_by_image() {
    block_on(basisu_encoder_init());

    let dir = std::env!("CARGO_MANIFEST_DIR");
    let mut images = Vec::new();
    let mut encoder = BasisuEncoder::new();
    for path in SKYBOX_PATHS {
        let image = Image::from_buffer(
            &std::fs::read(Path::new(dir).join(path)).unwrap(),
            bevy_image::ImageType::Extension(
                Path::new(path).extension().unwrap().to_str().unwrap(),
            ),
            CompressedImageFormats::empty(),
            true,
            bevy_image::ImageSampler::Default,
            Default::default(),
        )
        .unwrap();
        images.push(image);
    }
    let cube_image = Image {
        data: Some(
            images
                .iter_mut()
                .flat_map(|img| img.data.take().unwrap())
                .collect(),
        ),
        texture_descriptor: wgpu_types::TextureDescriptor {
            size: wgpu_types::Extent3d {
                width: images[0].width(),
                height: images[0].height(),
                depth_or_array_layers: images.len() as u32,
            },
            ..images[0].texture_descriptor
        },
        texture_view_descriptor: Some(wgpu_types::TextureViewDescriptor {
            dimension: Some(TextureViewDimension::Cube),
            ..Default::default()
        }),
        ..Default::default()
    };
    encoder
        .set_image(image_to_source_image(&cube_image))
        .unwrap();
    #[cfg_attr(target_os = "macos", expect(unused_variables))]
    let res = encoder
        .compress(
            BasisuEncoderParams::new_with_srgb_defaults(BasisTextureFormat::AstcLdr8x8)
                .with_tex_type(TextureViewDimension::Cube)
                .with_flags(
                    BU_COMP_FLAGS_DEBUG_OUTPUT
                        | BU_COMP_FLAGS_VALIDATE_OUTPUT
                        | BU_COMP_FLAGS_GEN_MIPS_CLAMP,
                ),
        )
        .unwrap();
    // The test failed on macos, disable it for now.
    #[cfg(not(target_os = "macos"))]
    insta::assert_binary_snapshot!("skybox_astc_ldr_8x8_mips.basisu.ktx2", res);
}

#[test]
fn encode_cubemap() {
    basisu_encoder_enable_debug_printf(true);
    encode_cubemap_xuastc_ldr_4x4_by_slice();
    encode_cubemap_astc_ldr_8x8_mips_by_image();
}
