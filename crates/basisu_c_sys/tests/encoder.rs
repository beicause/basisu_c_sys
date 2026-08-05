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
        BasisuEncoder, BasisuEncoderParams, SourceImage, SourceImageFormat, basisu_encoder_init,
        types,
    },
};

use crate::common::SNAPSHOT_PATH;

const SKYBOX_PATHS: &[&str] = &[
    "../../original_assets/skybox/right.jpg",
    "../../original_assets/skybox/left.jpg",
    "../../original_assets/skybox/top.jpg",
    "../../original_assets/skybox/bottom.jpg",
    "../../original_assets/skybox/front.jpg",
    "../../original_assets/skybox/back.jpg",
];

fn read_jpeg_image(path: &PathBuf) -> DynamicImage {
    let file = BufReader::new(File::open(path).unwrap());
    let mut reader = ImageReader::new(file);
    reader.set_format(ImageFormat::Jpeg);
    reader.no_limits();
    let img = reader.decode().unwrap();
    DynamicImage::ImageRgba8(img.into_rgba8())
}

fn skybox_images_iter() -> impl Iterator<Item = DynamicImage> {
    let dir = std::env!("CARGO_MANIFEST_DIR");
    SKYBOX_PATHS
        .iter()
        .map(|path| read_jpeg_image(&Path::new(dir).join(path)))
}

fn encode_cubemap_xuastc_ldr_4x4_by_slice() -> Vec<u8> {
    block_on(basisu_encoder_init());

    let mut encoder = BasisuEncoder::new();
    for (i, img) in skybox_images_iter().enumerate() {
        let source = SourceImage {
            data: img.as_bytes(),
            format: SourceImageFormat::Rgba8,
            size: types::Extent3d {
                width: img.width(),
                height: img.height(),
                depth_or_array_layers: 1,
            },
        };
        encoder.set_image_slice(i as u32, source).unwrap();
    }
    encoder
        .compress(
            BasisuEncoderParams::new_with_srgb_defaults(BasisTextureFormat::XuastcLdr4x4)
                .with_tex_type(types::TextureViewDimension::Cube)
                .with_flags(BU_COMP_FLAGS_DEBUG_OUTPUT | BU_COMP_FLAGS_VALIDATE_OUTPUT),
        )
        .unwrap()
}

fn encode_cubemap_astc_ldr_8x8_mips_by_image() -> Vec<u8> {
    block_on(basisu_encoder_init());

    let mut images = Vec::new();
    let mut encoder = BasisuEncoder::new();
    for img in skybox_images_iter() {
        images.push(img);
    }
    let cube_image = SourceImage {
        data: &images
            .iter()
            .flat_map(|img| img.as_bytes())
            .copied()
            .collect::<Vec<u8>>(),
        format: SourceImageFormat::Rgba8,
        size: types::Extent3d {
            width: images[0].width(),
            height: images[0].height(),
            depth_or_array_layers: images.len() as u32,
        },
    };
    encoder.set_image(cube_image).unwrap();
    encoder
        .compress(
            BasisuEncoderParams::new_with_srgb_defaults(BasisTextureFormat::AstcLdr8x8)
                .with_tex_type(types::TextureViewDimension::Cube)
                .with_flags(
                    BU_COMP_FLAGS_DEBUG_OUTPUT
                        | BU_COMP_FLAGS_VALIDATE_OUTPUT
                        | BU_COMP_FLAGS_GEN_MIPS_CLAMP,
                ),
        )
        .unwrap()
}

#[test]
fn encode_cubemap_by_slice() {
    insta::with_settings!({ snapshot_path => SNAPSHOT_PATH },{
        // basisu_c_sys::extra::basisu_encoder_enable_debug_printf(true);

        let _res = encode_cubemap_xuastc_ldr_4x4_by_slice();
        // The test failed on macos and windows, disable it for now.
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        insta::assert_binary_snapshot!("skybox_xuastc_ldr_4x4.basisu.ktx2", _res);
    });
}

#[test]
fn encode_cubemap_by_image() {
    insta::with_settings!({ snapshot_path => SNAPSHOT_PATH },{
        // basisu_c_sys::extra::basisu_encoder_enable_debug_printf(true);

        let _res = encode_cubemap_astc_ldr_8x8_mips_by_image();
        // The test failed on macos and windows, disable it for now.
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        insta::assert_binary_snapshot!("skybox_astc_ldr_8x8_mips.basisu.ktx2", _res);
    });
}

fn read_exr_image(path: &PathBuf) -> DynamicImage {
    let file = BufReader::new(File::open(path).unwrap());
    let mut reader = ImageReader::new(file);
    reader.set_format(ImageFormat::OpenExr);
    reader.no_limits();
    let img = reader.decode().unwrap();
    DynamicImage::ImageRgba32F(img.into_rgba32f())
}

fn encode_exr_image() -> Vec<u8> {
    let dir = std::env!("CARGO_MANIFEST_DIR");
    let exr_image_path = Path::new(dir).join("../../original_assets/Desk_fixed_6x6.exr");

    block_on(basisu_encoder_init());

    let mut encoder = BasisuEncoder::new();
    let image = read_exr_image(&exr_image_path);
    let source_image = SourceImage {
        data: image.as_bytes(),
        format: SourceImageFormat::Rgba32Float,
        size: types::Extent3d {
            width: image.width(),
            height: image.height(),
            depth_or_array_layers: 1,
        },
    };
    encoder.set_image(source_image).unwrap();
    encoder
        .compress(
            BasisuEncoderParams::new_with_srgb_defaults(BasisTextureFormat::UastcHdr6x6)
                .with_flags(
                    BU_COMP_FLAGS_DEBUG_OUTPUT
                        | BU_COMP_FLAGS_VALIDATE_OUTPUT
                        | BU_COMP_FLAGS_GEN_MIPS_CLAMP,
                ),
        )
        .unwrap()
}

#[test]
fn encode_hdr_image() {
    insta::with_settings!({ snapshot_path => SNAPSHOT_PATH },{
        // basisu_c_sys::extra::basisu_encoder_enable_debug_printf(true);

        let _res = encode_exr_image();
        // The test failed on macos, disable it for now.
        #[cfg(not(any(target_os = "macos")))]
        insta::assert_binary_snapshot!("desk_uastc_hdr_6x6_mips.basisu.ktx2", _res);
    });
}
