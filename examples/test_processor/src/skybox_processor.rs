use std::path::Path;

use bevy::{
    asset::{AsyncReadExt, AsyncWriteExt, RenderAssetUsages, processor::Process},
    image::{CompressedImageFormats, Image},
    reflect::TypePath,
    render::render_resource::Extent3d,
};
use bevy_basisu_loader::{BasisuLoader, BasisuLoaderSettings};
use bevy_basisu_saver::sys::extra::{BasisuEncoder, BasisuEncoderParams, SourceImageFormat, types};
use bevy_basisu_saver::sys::{
    common::{BU_COMP_FLAGS_DEBUG_OUTPUT, BU_COMP_FLAGS_VALIDATE_OUTPUT},
    extra::SourceImage,
};

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
        encoder
            .set_image_slice(
                i as u32,
                SourceImage {
                    data: image.data.as_deref().unwrap(),
                    format: SourceImageFormat::Rgba8,
                    size: convert_extent3d(image.texture_descriptor.size),
                },
            )
            .unwrap();
    }
    let params = BasisuEncoderParams::new_with_srgb_defaults(
        bevy_basisu_saver::sys::BasisTextureFormat::XuastcLdr8x8,
    )
    .with_tex_type(types::TextureViewDimension::Cube);

    encoder
        .compress(if debug {
            params.with_flags(BU_COMP_FLAGS_DEBUG_OUTPUT | BU_COMP_FLAGS_VALIDATE_OUTPUT)
        } else {
            params
        })
        .unwrap()
}

fn convert_extent3d(size: Extent3d) -> types::Extent3d {
    types::Extent3d {
        width: size.width,
        height: size.height,
        depth_or_array_layers: size.depth_or_array_layers,
    }
}
