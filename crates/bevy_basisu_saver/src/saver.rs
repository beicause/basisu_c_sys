use basisu_c_sys::extra::{BasisuEncodeError, BasisuEncoder, BasisuEncoderParams};
use bevy::{
    asset::{
        AsyncWriteExt, processor::LoadTransformAndSave, saver::AssetSaver,
        transformer::IdentityAssetTransformer,
    },
    image::{Image, ImageLoader},
    reflect::TypePath,
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
        let _span = bevy::log::info_span!("Encoding basisu texture").entered();
        let time = if log::STATIC_MAX_LEVEL >= log::LevelFilter::Debug {
            Some(bevy::platform::time::Instant::now())
        } else {
            None
        };

        let mut encoder = BasisuEncoder::new();
        encoder.set_image(basisu_c_sys::extra::SourceImage {
            data: asset.data.as_deref().unwrap_or(&[]),
            texture_descriptor: asset.texture_descriptor.clone(),
            texture_view_descriptor: asset.texture_view_descriptor.clone(),
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
