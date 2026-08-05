use basisu_c_sys::TranscodeTargetFormat;
use basisu_c_sys::extra::{
    BasisuTranscodeError, BasisuTranscoder, ChannelType, SupportedTextureCompression, types,
};
use bevy::asset::{AssetLoader, RenderAssetUsages};
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{
    AstcBlock, AstcChannel, Extent3d, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages, TextureViewDescriptor, TextureViewDimension, WgpuFeatures as Features,
};
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
        let transcoder = BasisuTranscoder::new(
            &data,
            self.supported_compressed_formats,
            settings.channel_type_hint,
        )?;
        let info = transcoder.get_info();

        let out_image = transcoder.transcode(settings.force_transcode_target, settings.is_srgb)?;

        if log::STATIC_MAX_LEVEL >= log::LevelFilter::Debug {
            bevy::log::debug!(
                "Transcoded a basisu texture {:?} -> {:?}, {:?}kb -> {:?}kb, preferred_target {:?}, extents {:?}, levels {:?}, view_dimension {:?}, in {:?}",
                info.basis_format,
                out_image.format,
                src_bytes as f32 / 1000.0,
                out_image.data.len() as f32 / 1000.0,
                info.preferred_target,
                out_image.size,
                info.levels,
                out_image.view_dimension,
                time.unwrap().elapsed(),
            );
        }
        Ok(Image {
            data: Some(out_image.data),
            data_order: bevy::render::render_resource::TextureDataOrder::MipMajor,
            texture_descriptor: TextureDescriptor {
                label: None,
                size: convert_extent3d(out_image.size),
                mip_level_count: out_image.mip_level_count,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: convert_format(out_image.format),
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::COPY_DST,
                view_formats: &[],
            },
            texture_view_descriptor: Some(TextureViewDescriptor {
                dimension: Some(convert_view_dimension(out_image.view_dimension)),
                ..Default::default()
            }),
            copy_on_resize: false,
            sampler: settings.sampler.clone(),
            asset_usage: settings.asset_usage,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["basisu.ktx2"]
    }
}

fn convert_extent3d(size: types::Extent3d) -> Extent3d {
    Extent3d {
        width: size.width,
        height: size.height,
        depth_or_array_layers: size.depth_or_array_layers,
    }
}

fn convert_astc_channel(channel: types::AstcChannel) -> AstcChannel {
    match channel {
        types::AstcChannel::Unorm => AstcChannel::Unorm,
        types::AstcChannel::UnormSrgb => AstcChannel::UnormSrgb,
        types::AstcChannel::Hdr => AstcChannel::Hdr,
    }
}
fn convert_astc_block(block: types::AstcBlock) -> AstcBlock {
    match block {
        types::AstcBlock::B4x4 => AstcBlock::B4x4,
        types::AstcBlock::B5x4 => AstcBlock::B5x4,
        types::AstcBlock::B5x5 => AstcBlock::B5x5,
        types::AstcBlock::B6x5 => AstcBlock::B6x5,
        types::AstcBlock::B6x6 => AstcBlock::B6x6,
        types::AstcBlock::B8x5 => AstcBlock::B8x5,
        types::AstcBlock::B8x6 => AstcBlock::B8x6,
        types::AstcBlock::B8x8 => AstcBlock::B8x8,
        types::AstcBlock::B10x5 => AstcBlock::B10x5,
        types::AstcBlock::B10x6 => AstcBlock::B10x6,
        types::AstcBlock::B10x8 => AstcBlock::B10x8,
        types::AstcBlock::B10x10 => AstcBlock::B10x10,
        types::AstcBlock::B12x10 => AstcBlock::B12x10,
        types::AstcBlock::B12x12 => AstcBlock::B12x12,
    }
}

fn convert_format(format: types::TextureFormat) -> TextureFormat {
    match format {
        types::TextureFormat::R8Unorm => TextureFormat::R8Unorm,
        types::TextureFormat::R16Float => TextureFormat::R16Float,
        types::TextureFormat::Rg8Unorm => TextureFormat::Rg8Unorm,
        types::TextureFormat::Rg16Float => TextureFormat::Rg16Float,
        types::TextureFormat::Rgba8Unorm => TextureFormat::Rgba8Unorm,
        types::TextureFormat::Rgba8UnormSrgb => TextureFormat::Rgba8UnormSrgb,
        types::TextureFormat::Rgb9e5Ufloat => TextureFormat::Rgb9e5Ufloat,
        types::TextureFormat::Rgba16Float => TextureFormat::Rgba16Float,
        types::TextureFormat::Bc1RgbaUnorm => TextureFormat::Bc1RgbaUnorm,
        types::TextureFormat::Bc1RgbaUnormSrgb => TextureFormat::Bc1RgbaUnormSrgb,
        types::TextureFormat::Bc2RgbaUnorm => TextureFormat::Bc2RgbaUnorm,
        types::TextureFormat::Bc2RgbaUnormSrgb => TextureFormat::Bc2RgbaUnormSrgb,
        types::TextureFormat::Bc3RgbaUnorm => TextureFormat::Bc3RgbaUnorm,
        types::TextureFormat::Bc3RgbaUnormSrgb => TextureFormat::Bc3RgbaUnormSrgb,
        types::TextureFormat::Bc4RUnorm => TextureFormat::Bc4RUnorm,
        types::TextureFormat::Bc4RSnorm => TextureFormat::Bc4RSnorm,
        types::TextureFormat::Bc5RgUnorm => TextureFormat::Bc5RgUnorm,
        types::TextureFormat::Bc5RgSnorm => TextureFormat::Bc5RgSnorm,
        types::TextureFormat::Bc6hRgbUfloat => TextureFormat::Bc6hRgbUfloat,
        types::TextureFormat::Bc6hRgbFloat => TextureFormat::Bc6hRgbFloat,
        types::TextureFormat::Bc7RgbaUnorm => TextureFormat::Bc7RgbaUnorm,
        types::TextureFormat::Bc7RgbaUnormSrgb => TextureFormat::Bc7RgbaUnormSrgb,
        types::TextureFormat::Etc2Rgb8Unorm => TextureFormat::Etc2Rgb8Unorm,
        types::TextureFormat::Etc2Rgb8UnormSrgb => TextureFormat::Etc2Rgb8UnormSrgb,
        types::TextureFormat::Etc2Rgb8A1Unorm => TextureFormat::Etc2Rgb8A1Unorm,
        types::TextureFormat::Etc2Rgb8A1UnormSrgb => TextureFormat::Etc2Rgb8A1UnormSrgb,
        types::TextureFormat::Etc2Rgba8Unorm => TextureFormat::Etc2Rgba8Unorm,
        types::TextureFormat::Etc2Rgba8UnormSrgb => TextureFormat::Etc2Rgba8UnormSrgb,
        types::TextureFormat::EacR11Unorm => TextureFormat::EacR11Unorm,
        types::TextureFormat::EacR11Snorm => TextureFormat::EacR11Snorm,
        types::TextureFormat::EacRg11Unorm => TextureFormat::EacRg11Unorm,
        types::TextureFormat::EacRg11Snorm => TextureFormat::EacRg11Snorm,
        types::TextureFormat::Astc { block, channel } => TextureFormat::Astc {
            block: convert_astc_block(block),
            channel: convert_astc_channel(channel),
        },
    }
}

fn convert_view_dimension(dim: types::TextureViewDimension) -> TextureViewDimension {
    match dim {
        types::TextureViewDimension::D2 => TextureViewDimension::D2,
        types::TextureViewDimension::D2Array => TextureViewDimension::D2Array,
        types::TextureViewDimension::Cube => TextureViewDimension::Cube,
        types::TextureViewDimension::CubeArray => TextureViewDimension::CubeArray,
    }
}
