use basisu_c_sys::extra::{
    BasisuEncodeError, BasisuEncoder, BasisuEncoderParams, SourceImageFormat, types,
};
use bevy::{
    asset::{
        AsyncWriteExt, processor::LoadTransformAndSave, saver::AssetSaver,
        transformer::IdentityAssetTransformer,
    },
    image::{Image, ImageLoader},
    reflect::TypePath,
    render::render_resource::{Extent3d, TextureFormat},
};
use bevy_basisu_loader::{BasisuLoader, BasisuLoaderSettings};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Basis universal asset processor.
pub type BasisuProcessor =
    LoadTransformAndSave<ImageLoader, IdentityAssetTransformer<Image>, BasisuSaver>;

/// Basis universal texture saver.
#[derive(TypePath)]
pub struct BasisuSaver {
    /// Default basisu encoder params.
    /// See the documents and `BU_COMP_FLAGS_*` in [`basisu_c_sys`] if you want more controls,
    /// like mipmap generation.
    pub default_encoder_params: BasisuEncoderParams,
}

/// Basis universal texture saver settings.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct BasisuSaverSettings {
    /// Basisu encoder params. If it's None the [`BasisuSaver::default_encoder_params`] will be used.
    pub encoder_params: Option<BasisuEncoderParams>,
}

/// An error when encoding an image using [`BasisuSaver`].
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum BasisuSaverError {
    /// An error occurred while trying to load the bytes.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Image data is empty")]
    EmptyData,
    #[error("Image texture format is unsupported by the encoder")]
    UnsupportedTextureFormat(TextureFormat),
    #[error("Image format is Rgba32Float, but the data bytes is not a multiple of 16")]
    UnalignedRgba32Float,
    #[error("Image with mipmaps ({0}) is unsupported")]
    Mipmaps(u32),
    /// An error occurred while trying to encode the image.
    #[error(transparent)]
    BasisuEncodeError(#[from] BasisuEncodeError),
}

impl AssetSaver for BasisuSaver {
    type Asset = Image;
    type Settings = BasisuSaverSettings;
    type OutputLoader = BasisuLoader;
    type Error = BasisuSaverError;

    async fn save(
        &self,
        writer: &mut bevy::asset::io::Writer,
        asset: bevy::asset::saver::SavedAsset<'_, '_, Self::Asset>,
        settings: &Self::Settings,
        asset_path: bevy::asset::AssetPath<'_>,
    ) -> Result<<Self::OutputLoader as bevy::asset::AssetLoader>::Settings, Self::Error> {
        if asset.texture_descriptor.mip_level_count != 1 {
            return Err(BasisuSaverError::Mipmaps(
                asset.texture_descriptor.mip_level_count,
            ));
        }

        let _span = bevy::log::info_span!("Encoding basisu texture").entered();
        let time = if log::STATIC_MAX_LEVEL >= log::LevelFilter::Debug {
            Some(bevy::platform::time::Instant::now())
        } else {
            None
        };

        let mut encoder = BasisuEncoder::new();
        let (format, data) = match (&asset.data, asset.texture_descriptor.format) {
            (None, _) => return Err(BasisuSaverError::EmptyData),
            (Some(data), TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb) => {
                (SourceImageFormat::Rgba8, data)
            }
            (Some(data), TextureFormat::Rgba32Float) => (SourceImageFormat::Rgba32Float, data),
            (_, format) => return Err(BasisuSaverError::UnsupportedTextureFormat(format)),
        };
        encoder.set_image(basisu_c_sys::extra::SourceImage {
            data,
            format,
            size: convert_extent3d(asset.texture_descriptor.size),
        })?;
        let result = encoder.compress(
            settings
                .encoder_params
                .unwrap_or(self.default_encoder_params),
        )?;

        if log::STATIC_MAX_LEVEL >= log::LevelFilter::Debug {
            bevy::log::debug!(
                "Encoded basisu texture \"{}\", {}kb -> {}kb in {:?}",
                asset_path,
                asset.data.as_deref().unwrap_or(&[]).len() as f32 / 1000.0,
                result.len() as f32 / 1000.0,
                time.unwrap().elapsed(),
            );
        }
        drop(_span);

        writer.write_all(&result).await?;

        Ok(BasisuLoaderSettings {
            asset_usage: asset.asset_usage,
            sampler: asset.sampler.clone(),
            ..Default::default()
        })
    }
}

fn convert_extent3d(size: Extent3d) -> types::Extent3d {
    types::Extent3d {
        width: size.width,
        height: size.height,
        depth_or_array_layers: size.depth_or_array_layers,
    }
}
