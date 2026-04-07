use basisu_c_sys::TranscodeTargetFormat;
use basisu_c_sys::extra::{
    BasisuTranscodeError, BasisuTranscoder, ChannelType, SupportedTextureCompression,
};
use bevy::asset::{AssetLoader, RenderAssetUsages};
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::WgpuFeatures as Features;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(TypePath)]
pub struct BasisuLoader {
    supported_compressed_formats: SupportedTextureCompression,
}

impl BasisuLoader {
    pub fn from_features(features: Features) -> Self {
        let mut supported_compressed_formats = SupportedTextureCompression::empty();
        if features.contains(Features::TEXTURE_COMPRESSION_ASTC) {
            supported_compressed_formats |= SupportedTextureCompression::ASTC_LDR;
        }
        if features.contains(Features::TEXTURE_COMPRESSION_ASTC_HDR) {
            supported_compressed_formats |= SupportedTextureCompression::ASTC_HDR;
        }
        if features.contains(Features::TEXTURE_COMPRESSION_BC) {
            supported_compressed_formats |= SupportedTextureCompression::BC;
        }
        if features.contains(Features::TEXTURE_COMPRESSION_ETC2) {
            supported_compressed_formats |= SupportedTextureCompression::ETC2;
        }
        Self {
            supported_compressed_formats,
        }
    }
}

/// Settings for loading an [`Image`] using an [`BasisuLoader`].
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct BasisuLoaderSettings {
    /// [`ImageSampler`] to use when rendering - this does
    /// not affect the loading of the image data.
    pub sampler: ImageSampler,
    /// Where the asset will be used - see the docs on
    /// [`RenderAssetUsages`] for details.
    pub asset_usage: RenderAssetUsages,
    /// Whether the texture should be created as sRGB format.
    ///
    /// If `None`, it will be determined by the KTX2 data format descriptor transfer function.
    pub is_srgb: Option<bool>,
    /// The channel type hint for transcode target selection.
    ///
    /// If [`ChannelType::Auto`], it will be determined by the KTX2 data format descriptor channel type.
    ///
    /// Note: This will be ignored when the transcode target isn't single-channel or dual-channel (like ETC2 or BC4/BC5), so this usually only has effect for ETC1S textures. See [`BasisuLoaderPlugin`](crate::BasisuLoaderPlugin) for more information about target selection.
    pub channel_type_hint: ChannelType,
    /// Forcibly transcode to a specific `TF_*` in [`basisu_c_sys::common`]. If `None` the target format is selected automatically.
    ///
    /// It will fail to load if the target format is not supported by the device or it can't be transcoded by Basis Universal.
    pub force_transcode_target: Option<TranscodeTargetFormat>,
}

/// An error when loading an image using [`BasisuLoader`].
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum BasisuLoaderError {
    /// An error occurred while trying to load the image bytes.
    #[error("Failed to load image bytes: {0}")]
    Io(#[from] std::io::Error),
    #[error("BasisU failed to transcode texture: {0}")]
    TranscodeError(#[from] BasisuTranscodeError),
}

impl AssetLoader for BasisuLoader {
    type Asset = Image;

    type Settings = BasisuLoaderSettings;

    type Error = BasisuLoaderError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        settings: &Self::Settings,
        _load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data).await?;
        let src_bytes = data.len();

        let _span = bevy::log::info_span!("transcoding basisu texture").entered();
        let time = if log::STATIC_MAX_LEVEL >= log::LevelFilter::Debug {
            Some(bevy::platform::time::Instant::now())
        } else {
            None
        };
        let mut transcoder = BasisuTranscoder::new();
        let info = transcoder.prepare(
            &data,
            self.supported_compressed_formats,
            settings.channel_type_hint,
        )?;

        let out_image = transcoder.transcode(settings.force_transcode_target, settings.is_srgb)?;

        if log::STATIC_MAX_LEVEL >= log::LevelFilter::Debug {
            bevy::log::debug!(
                "Transcoded a basisu texture {:?} -> {:?}, {:?}kb -> {:?}kb, preferred_target {:?}, extents {:?}, levels {:?}, view_dimension {:?}, in {:?}",
                info.basis_format,
                out_image.texture_descriptor.format,
                src_bytes as f32 / 1000.0,
                out_image.data.len() as f32 / 1000.0,
                info.preferred_target,
                out_image.texture_descriptor.size,
                info.levels,
                out_image
                    .texture_view_descriptor
                    .as_ref()
                    .unwrap()
                    .dimension
                    .unwrap(),
                time.unwrap().elapsed(),
            );
        }
        Ok(Image {
            data: Some(out_image.data),
            data_order: out_image.data_order,
            texture_descriptor: out_image.texture_descriptor,
            texture_view_descriptor: out_image.texture_view_descriptor,
            copy_on_resize: false,
            sampler: settings.sampler.clone(),
            asset_usage: settings.asset_usage,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["basisu.ktx2"]
    }
}
