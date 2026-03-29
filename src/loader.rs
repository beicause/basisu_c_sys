use bevy::asset::{AssetLoader, RenderAssetUsages};
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{
    AstcBlock, AstcChannel, Extent3d, TextureDataOrder, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages, TextureViewDescriptor, TextureViewDimension,
    WgpuFeatures as Features,
};
use bevy_basisu_loader_sys::{SupportedTextureCompressionMethods, TranscodedTextureFormat};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(TypePath)]
pub struct BasisuLoader {
    supported_compressed_formats: SupportedTextureCompressionMethods,
}

impl BasisuLoader {
    pub fn from_features(features: Features) -> Self {
        let mut supported_compressed_formats = SupportedTextureCompressionMethods::NONE;
        if features.contains(Features::TEXTURE_COMPRESSION_ASTC) {
            supported_compressed_formats |= SupportedTextureCompressionMethods::ASTC_LDR;
        }
        if features.contains(Features::TEXTURE_COMPRESSION_ASTC_HDR) {
            supported_compressed_formats |= SupportedTextureCompressionMethods::ASTC_HDR;
        }
        if features.contains(Features::TEXTURE_COMPRESSION_BC) {
            supported_compressed_formats |= SupportedTextureCompressionMethods::BC;
        }
        if features.contains(Features::TEXTURE_COMPRESSION_ETC2) {
            supported_compressed_formats |= SupportedTextureCompressionMethods::ETC2;
        }
        Self {
            supported_compressed_formats,
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy)]
#[repr(u8)]
pub enum ChannelType {
    #[default]
    Auto,
    Rgba,
    Rgb,
    Rg,
    R,
}

const fn channel_type_to_channel_type_sys(t: ChannelType) -> bevy_basisu_loader_sys::ChannelType {
    // SAFETY: Both repr are u8
    unsafe { core::mem::transmute(t) }
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
    /// Note: This will be ignored when the transcode target format is not ETC2 or BC4/BC5 so it usually only has effect for ETC1S textures.
    /// See [`BasisuLoaderPlugin`](crate::BasisuLoaderPlugin) for more information about target selection.
    pub channel_type_hint: ChannelType,
    /// Forcibly transcode to a specific [`TextureFormat`]. If `None` the target format is selected automatically.
    ///
    /// Use this with caution! It will fail if the transcode target is not supported by Basis Universal or the texture format is not supported by the device.
    /// One use case is transcoding HDR textures to [`TextureFormat::Rgb9e5Ufloat`].
    /// The srgb-ness of the texture format is ignored and will be determined by `is_srgb`.
    pub force_transcode_target: Option<TextureFormat>,
}

/// An error when loading an image using [`BasisuLoader`].
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum BasisuLoaderError {
    /// An error occurred while trying to load the image bytes.
    #[error("Failed to load image bytes: {0}")]
    Io(#[from] std::io::Error),
    /// An error occurred while trying to decode the image bytes.
    #[error("BasisU failed to transcode texture: {0}")]
    TranscodingError(&'static str),
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
        let (out_data, out_format, extent, levels, view_dimension) = {
            let _span = bevy::log::info_span!("Transcoding BasisU Texture").entered();
            let time = if log::STATIC_MAX_LEVEL >= log::LevelFilter::Debug {
                Some(bevy::platform::time::Instant::now())
            } else {
                None
            };

            let Some(result) = bevy_basisu_loader_sys::basisu_transcode(
                data,
                self.supported_compressed_formats,
                channel_type_to_channel_type_sys(settings.channel_type_hint),
                texture_bevy_format_to_transcode_format(settings.force_transcode_target),
            ) else {
                return Err(BasisuLoaderError::TranscodingError("basisu_transcode"));
            };

            let view_dimension = if result.layers == 0 {
                if result.faces == 1 {
                    TextureViewDimension::D2
                } else if result.faces == 6 {
                    TextureViewDimension::Cube
                } else {
                    unreachable!()
                }
            } else if result.faces == 1 {
                TextureViewDimension::D2Array
            } else if result.faces == 6 {
                TextureViewDimension::CubeArray
            } else {
                unreachable!()
            };
            let extent = Extent3d {
                width: result.width,
                height: result.height,
                depth_or_array_layers: result.layers.max(1) * result.faces,
            };

            let out_format = texture_transcode_format_to_bevy_format(
                result.target_format,
                settings.is_srgb.unwrap_or(result.is_srgb),
            );

            if log::STATIC_MAX_LEVEL >= log::LevelFilter::Debug {
                bevy::log::debug!(
                    "Transcoded a basisu texture with src_bytes: {:?}, src_format: {:?}, dst_bytes: {:?}, dst_format: {:?}, extent: {:?}, levels: {:?}, view_dimension: {:?}, in {:?}",
                    src_bytes,
                    result.basis_format,
                    result.data.len(),
                    out_format,
                    extent,
                    result.levels,
                    view_dimension,
                    time.unwrap().elapsed()
                );
            }

            (
                result.data,
                out_format,
                extent,
                result.levels,
                view_dimension,
            )
        };
        let mut image = Image {
            data: None,
            data_order: TextureDataOrder::MipMajor,
            texture_descriptor: TextureDescriptor {
                // Note: we must give wgpu the logical texture dimensions, so it can correctly compute mip sizes.
                // However this currently causes wgpu to panic if the dimensions aren't a multiple of blocksize.
                // See https://github.com/gfx-rs/wgpu/issues/7677 for more context.
                size: {
                    #[cfg(debug_assertions)]
                    if extent != extent.physical_size(out_format) {
                        bevy::log::error!(
                            "BasisU texture size has to be a multiple of block size to ensure correct mip levels transcoding, otherwise it will panic for now. This is due to a wgpu limitation, see https://github.com/gfx-rs/wgpu/issues/7677"
                        );
                    }
                    extent
                },
                format: out_format,
                dimension: TextureDimension::D2,
                label: None,
                mip_level_count: levels,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::COPY_SRC,
                view_formats: &[],
            },
            sampler: settings.sampler.clone(),
            texture_view_descriptor: Some(TextureViewDescriptor {
                dimension: Some(view_dimension),
                ..Default::default()
            }),
            asset_usage: settings.asset_usage,
            copy_on_resize: false,
        };
        image.data = Some(out_data);
        Ok(image)
    }

    fn extensions(&self) -> &[&str] {
        &["basisu.ktx2"]
    }
}

fn texture_transcode_format_to_bevy_format(
    transcoded: TranscodedTextureFormat,
    is_srgb: bool,
) -> TextureFormat {
    let mut fmt = match transcoded {
        TranscodedTextureFormat::cTFETC1_RGB => TextureFormat::Etc2Rgb8Unorm,
        TranscodedTextureFormat::cTFETC2_RGBA => TextureFormat::Etc2Rgba8Unorm,
        TranscodedTextureFormat::cTFBC1_RGB => TextureFormat::Bc1RgbaUnorm,
        TranscodedTextureFormat::cTFBC3_RGBA => TextureFormat::Bc3RgbaUnorm,
        TranscodedTextureFormat::cTFBC4_R => TextureFormat::Bc4RUnorm,
        TranscodedTextureFormat::cTFBC5_RG => TextureFormat::Bc5RgUnorm,
        TranscodedTextureFormat::cTFBC7_RGBA => TextureFormat::Bc7RgbaUnorm,
        TranscodedTextureFormat::cTFPVRTC1_4_RGB => unreachable!(),
        TranscodedTextureFormat::cTFPVRTC1_4_RGBA => unreachable!(),
        TranscodedTextureFormat::cTFASTC_4x4_RGBA => TextureFormat::Astc {
            block: AstcBlock::B4x4,
            channel: AstcChannel::Unorm,
        },
        TranscodedTextureFormat::cTFATC_RGB => unreachable!(),
        TranscodedTextureFormat::cTFATC_RGBA => unreachable!(),
        TranscodedTextureFormat::cTFFXT1_RGB => unreachable!(),
        TranscodedTextureFormat::cTFPVRTC2_4_RGB => unreachable!(),
        TranscodedTextureFormat::cTFPVRTC2_4_RGBA => unreachable!(),
        TranscodedTextureFormat::cTFETC2_EAC_R11 => TextureFormat::EacR11Unorm,
        TranscodedTextureFormat::cTFETC2_EAC_RG11 => TextureFormat::EacRg11Unorm,
        TranscodedTextureFormat::cTFBC6H => TextureFormat::Bc6hRgbUfloat,
        TranscodedTextureFormat::cTFASTC_HDR_4x4_RGBA => TextureFormat::Astc {
            block: AstcBlock::B4x4,
            channel: AstcChannel::Hdr,
        },
        TranscodedTextureFormat::cTFRGBA32 => TextureFormat::Rgba8Unorm,
        TranscodedTextureFormat::cTFRGB565 => unreachable!(),
        TranscodedTextureFormat::cTFBGR565 => unreachable!(),
        TranscodedTextureFormat::cTFRGBA4444 => unreachable!(),
        TranscodedTextureFormat::cTFRGB_HALF => unreachable!(),
        TranscodedTextureFormat::cTFRGBA_HALF => TextureFormat::Rgba16Float,
        TranscodedTextureFormat::cTFRGB_9E5 => TextureFormat::Rgb9e5Ufloat,
        TranscodedTextureFormat::cTFASTC_HDR_6x6_RGBA => TextureFormat::Astc {
            block: AstcBlock::B6x6,
            channel: AstcChannel::Hdr,
        },
        TranscodedTextureFormat::cTFASTC_LDR_5x4_RGBA => TextureFormat::Astc {
            block: AstcBlock::B5x4,
            channel: AstcChannel::Unorm,
        },
        TranscodedTextureFormat::cTFASTC_LDR_5x5_RGBA => TextureFormat::Astc {
            block: AstcBlock::B5x5,
            channel: AstcChannel::Unorm,
        },
        TranscodedTextureFormat::cTFASTC_LDR_6x5_RGBA => TextureFormat::Astc {
            block: AstcBlock::B6x5,
            channel: AstcChannel::Unorm,
        },
        TranscodedTextureFormat::cTFASTC_LDR_6x6_RGBA => TextureFormat::Astc {
            block: AstcBlock::B6x6,
            channel: AstcChannel::Unorm,
        },
        TranscodedTextureFormat::cTFASTC_LDR_8x5_RGBA => TextureFormat::Astc {
            block: AstcBlock::B8x5,
            channel: AstcChannel::Unorm,
        },
        TranscodedTextureFormat::cTFASTC_LDR_8x6_RGBA => TextureFormat::Astc {
            block: AstcBlock::B8x6,
            channel: AstcChannel::Unorm,
        },
        TranscodedTextureFormat::cTFASTC_LDR_10x5_RGBA => TextureFormat::Astc {
            block: AstcBlock::B10x5,
            channel: AstcChannel::Unorm,
        },
        TranscodedTextureFormat::cTFASTC_LDR_10x6_RGBA => TextureFormat::Astc {
            block: AstcBlock::B10x6,
            channel: AstcChannel::Unorm,
        },
        TranscodedTextureFormat::cTFASTC_LDR_8x8_RGBA => TextureFormat::Astc {
            block: AstcBlock::B8x8,
            channel: AstcChannel::Unorm,
        },
        TranscodedTextureFormat::cTFASTC_LDR_10x8_RGBA => TextureFormat::Astc {
            block: AstcBlock::B10x8,
            channel: AstcChannel::Unorm,
        },
        TranscodedTextureFormat::cTFASTC_LDR_10x10_RGBA => TextureFormat::Astc {
            block: AstcBlock::B10x10,
            channel: AstcChannel::Unorm,
        },
        TranscodedTextureFormat::cTFASTC_LDR_12x10_RGBA => TextureFormat::Astc {
            block: AstcBlock::B12x10,
            channel: AstcChannel::Unorm,
        },
        TranscodedTextureFormat::cTFASTC_LDR_12x12_RGBA => TextureFormat::Astc {
            block: AstcBlock::B12x12,
            channel: AstcChannel::Unorm,
        },
        TranscodedTextureFormat::cTFTotalTextureFormats => unreachable!(),
        TranscodedTextureFormat::cTFBC7_ALT => unreachable!(),
    };
    if is_srgb {
        fmt = fmt.add_srgb_suffix();
    }
    fmt
}

fn texture_bevy_format_to_transcode_format(
    format: Option<TextureFormat>,
) -> TranscodedTextureFormat {
    let Some(format) = format else {
        return TranscodedTextureFormat::cTFTotalTextureFormats;
    };
    let format = format.remove_srgb_suffix();
    match format {
        TextureFormat::Etc2Rgb8Unorm => TranscodedTextureFormat::cTFETC1_RGB,
        TextureFormat::Etc2Rgba8Unorm => TranscodedTextureFormat::cTFETC2_RGBA,
        TextureFormat::Bc1RgbaUnorm => TranscodedTextureFormat::cTFBC1_RGB,
        TextureFormat::Bc3RgbaUnorm => TranscodedTextureFormat::cTFBC3_RGBA,
        TextureFormat::Bc4RUnorm => TranscodedTextureFormat::cTFBC4_R,
        TextureFormat::Bc5RgUnorm => TranscodedTextureFormat::cTFBC5_RG,
        TextureFormat::Bc7RgbaUnorm => TranscodedTextureFormat::cTFBC7_RGBA,
        TextureFormat::Astc {
            block: AstcBlock::B4x4,
            channel: AstcChannel::Unorm,
        } => TranscodedTextureFormat::cTFASTC_4x4_RGBA,
        TextureFormat::EacR11Unorm => TranscodedTextureFormat::cTFETC2_EAC_R11,
        TextureFormat::EacRg11Unorm => TranscodedTextureFormat::cTFETC2_EAC_RG11,
        TextureFormat::Bc6hRgbUfloat => TranscodedTextureFormat::cTFBC6H,
        TextureFormat::Astc {
            block: AstcBlock::B4x4,
            channel: AstcChannel::Hdr,
        } => TranscodedTextureFormat::cTFASTC_HDR_4x4_RGBA,
        TextureFormat::Rgba8Unorm => TranscodedTextureFormat::cTFRGBA32,
        TextureFormat::Rgba16Float => TranscodedTextureFormat::cTFRGBA_HALF,
        TextureFormat::Rgb9e5Ufloat => TranscodedTextureFormat::cTFRGB_9E5,
        TextureFormat::Astc {
            block: AstcBlock::B6x6,
            channel: AstcChannel::Hdr,
        } => TranscodedTextureFormat::cTFASTC_HDR_6x6_RGBA,
        _ => unreachable!(),
    }
}
