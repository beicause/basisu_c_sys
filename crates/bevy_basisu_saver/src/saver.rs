use bevy::{
    asset::{AsyncWriteExt, saver::AssetSaver},
    image::Image,
    reflect::TypePath,
};
use bevy_basisu_loader::{BasisuLoader, BasisuLoaderSettings};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::encoder::{BasisuEncodeError, BasisuEncoder, BasisuEncoderParams};

#[derive(TypePath)]
pub struct BasisuTextureSaver;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BasisuTextureSaverSettings {
    pub params: BasisuEncoderParams,
}

impl Default for BasisuTextureSaverSettings {
    fn default() -> Self {
        Self {
            params: BasisuEncoderParams::new_with_srgb_defaults(
                crate::encoder::BasisTextureFormat::XuastcLdr4x4,
            ),
        }
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum BasisuTextureSaverError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    BasisuEncodeError(#[from] BasisuEncodeError),
}

impl AssetSaver for BasisuTextureSaver {
    type Asset = Image;
    type Settings = BasisuTextureSaverSettings;
    type OutputLoader = BasisuLoader;
    type Error = BasisuTextureSaverError;

    async fn save(
        &self,
        writer: &mut bevy::asset::io::Writer,
        asset: bevy::asset::saver::SavedAsset<'_, Self::Asset>,
        settings: &Self::Settings,
    ) -> Result<<Self::OutputLoader as bevy::asset::AssetLoader>::Settings, Self::Error> {
        let mut encoder = BasisuEncoder::new();
        encoder.set_image(&asset)?;
        let result = encoder.compress(settings.params)?;
        writer.write_all(&result).await?;

        Ok(BasisuLoaderSettings {
            asset_usage: asset.asset_usage,
            sampler: asset.sampler.clone(),
            ..Default::default()
        })
    }
}
