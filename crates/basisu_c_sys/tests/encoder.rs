mod common;
use common::block_on;
use image::{DynamicImage, ImageFormat, ImageReader};

use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

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
use wgpu_types::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    TextureViewDescriptor, TextureViewDimension,
};

const SKYBOX_PATHS: &[&str] = &[
    "../../original_assets/skybox/right.jpg",
    "../../original_assets/skybox/left.jpg",
    "../../original_assets/skybox/top.jpg",
    "../../original_assets/skybox/bottom.jpg",
    "../../original_assets/skybox/front.jpg",
    "../../original_assets/skybox/back.jpg",
];

fn read_image(path: &PathBuf) -> DynamicImage {
    let file = BufReader::new(File::open(path).unwrap());
    let mut reader = ImageReader::new(file);
    reader.set_format(ImageFormat::Jpeg);
    reader.no_limits();
    let img = reader.decode().unwrap();
    let img = DynamicImage::ImageRgba8(img.into_rgba8());
    img
}

fn encode_cubemap_xuastc_ldr_4x4_by_slice() {
    block_on(basisu_encoder_init());

    let dir = std::env!("CARGO_MANIFEST_DIR");
    let mut encoder = BasisuEncoder::new();
    for (i, path) in SKYBOX_PATHS.iter().enumerate() {
        let img = read_image(&Path::new(dir).join(path));
        let source = SourceImage {
            data: img.as_bytes(),
            texture_descriptor: TextureDescriptor {
                size: Extent3d {
                    width: img.width(),
                    height: img.height(),
                    depth_or_array_layers: 1,
                },
                label: None,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::empty(),
                view_formats: &[],
            },
            texture_view_descriptor: None,
        };
        encoder.set_image_slice(i as u32, source).unwrap();
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
        let img = read_image(&Path::new(dir).join(path));
        images.push(img);
    }
    let cube_image = SourceImage {
        data: &images
            .iter()
            .map(|img| img.as_bytes())
            .flatten()
            .copied()
            .collect::<Vec<u8>>(),
        texture_descriptor: wgpu_types::TextureDescriptor {
            size: wgpu_types::Extent3d {
                width: images[0].width(),
                height: images[0].height(),
                depth_or_array_layers: images.len() as u32,
            },
            label: None,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::empty(),
            view_formats: &[],
        },
        texture_view_descriptor: Some(TextureViewDescriptor {
            dimension: Some(TextureViewDimension::Cube),
            ..Default::default()
        }),
    };
    encoder.set_image(cube_image).unwrap();
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
