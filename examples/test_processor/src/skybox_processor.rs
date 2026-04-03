use std::path::Path;

use bevy::{
    asset::{AsyncReadExt, AsyncWriteExt, RenderAssetUsages, processor::Process},
    image::{CompressedImageFormats, Image},
    reflect::TypePath,
    render::render_resource::TextureViewDimension,
};
use bevy_basisu_loader::{BasisuLoader, BasisuLoaderSettings};
use bevy_basisu_saver::c_sys::common::{BU_COMP_FLAGS_DEBUG_OUTPUT, BU_COMP_FLAGS_VALIDATE_OUTPUT};
use bevy_basisu_saver::encoder::{BasisuEncoder, BasisuEncoderParams};

#[derive(TypePath)]
pub(crate) struct SkyboxProcessor;

impl Process for SkyboxProcessor {
    type Settings = ();
    type OutputLoader = BasisuLoader;

    async fn process(
        &self,
        context: &mut bevy::asset::processor::ProcessContext<'_>,
        _settings: &Self::Settings,
        writer: &mut bevy::asset::io::Writer,
    ) -> Result<
        <Self::OutputLoader as bevy::asset::AssetLoader>::Settings,
        bevy::asset::processor::ProcessError,
    > {
        let mut ron = String::new();
        if let Err(err) = context.asset_reader().read_to_string(&mut ron).await {
            return Err(bevy::asset::processor::ProcessError::AssetReaderError {
                path: context.path().clone(),
                err: bevy::asset::io::AssetReaderError::Io(err.into()),
            });
        }
        let face_paths: [String; 6] = ron::from_str(&ron).unwrap();
        let compressed = encode_cubemap(&face_paths, false);
        writer.write_all(&compressed).await.unwrap();
        Ok(BasisuLoaderSettings::default())
    }
}

fn encode_cubemap(face_paths: &[String; 6], debug: bool) -> Vec<u8> {
    let dir = std::env!("CARGO_MANIFEST_DIR");
    let mut encoder = BasisuEncoder::new();
    for (i, path) in face_paths.iter().enumerate() {
        let image = Image::from_buffer(
            &std::fs::read(Path::new(dir).join(path)).unwrap(),
            bevy::image::ImageType::Extension(
                Path::new(path).extension().unwrap().to_str().unwrap(),
            ),
            CompressedImageFormats::empty(),
            true,
            bevy::image::ImageSampler::Default,
            RenderAssetUsages::all(),
        )
        .unwrap();
        encoder.set_image_slice(i as u32, &image).unwrap();
    }
    let params = BasisuEncoderParams::new_with_srgb_defaults(
        bevy_basisu_saver::encoder::BasisTextureFormat::XuastcLdr6x6,
    )
    .with_tex_type(TextureViewDimension::Cube);

    encoder
        .compress(if debug {
            params.with_flags(BU_COMP_FLAGS_DEBUG_OUTPUT | BU_COMP_FLAGS_VALIDATE_OUTPUT)
        } else {
            params
        })
        .unwrap()
}

#[cfg(test)]
mod tests {
    use bevy_basisu_saver::encoder::{basisu_encoder_enable_debug_printf, basisu_encoder_init};

    use super::*;

    fn encode_cubemap2(face_paths: &[String; 6]) -> Vec<u8> {
        let dir = std::env!("CARGO_MANIFEST_DIR");
        let mut images = Vec::new();
        let mut encoder = BasisuEncoder::new();
        for path in face_paths {
            let image = Image::from_buffer(
                &std::fs::read(Path::new(dir).join(path)).unwrap(),
                bevy::image::ImageType::Extension(
                    Path::new(path).extension().unwrap().to_str().unwrap(),
                ),
                CompressedImageFormats::empty(),
                true,
                bevy::image::ImageSampler::Default,
                RenderAssetUsages::all(),
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
            texture_descriptor: bevy::render::render_resource::TextureDescriptor {
                size: bevy::render::render_resource::Extent3d {
                    width: images[0].width(),
                    height: images[0].height(),
                    depth_or_array_layers: images.len() as u32,
                },
                ..images[0].texture_descriptor
            },
            texture_view_descriptor: Some(bevy::render::render_resource::TextureViewDescriptor {
                dimension: Some(TextureViewDimension::Cube),
                ..Default::default()
            }),
            ..Default::default()
        };
        encoder.set_image(&cube_image).unwrap();
        encoder
            .compress(
                BasisuEncoderParams::new_with_srgb_defaults(
                    bevy_basisu_saver::encoder::BasisTextureFormat::XuastcLdr4x4,
                )
                .with_tex_type(TextureViewDimension::Cube)
                .with_flags(BU_COMP_FLAGS_DEBUG_OUTPUT | BU_COMP_FLAGS_VALIDATE_OUTPUT),
            )
            .unwrap()
    }

    #[test]
    fn validate_encoding_via_set_image() {
        basisu_encoder_init();
        basisu_encoder_enable_debug_printf(true);

        let paths = [
            "skybox/right.jpg",
            "skybox/left.jpg",
            "skybox/top.jpg",
            "skybox/bottom.jpg",
            "skybox/front.jpg",
            "skybox/back.jpg",
        ]
        .map(|s| s.to_string());

        let _ = encode_cubemap(&paths, true);
    }

    #[test]
    fn validate_encoding_via_set_image_slice() {
        basisu_encoder_init();
        basisu_encoder_enable_debug_printf(true);

        let paths = [
            "skybox/right.jpg",
            "skybox/left.jpg",
            "skybox/top.jpg",
            "skybox/bottom.jpg",
            "skybox/front.jpg",
            "skybox/back.jpg",
        ]
        .map(|s| s.to_string());

        let _ = encode_cubemap2(&paths);
    }
}
