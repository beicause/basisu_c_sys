use bevy::asset::{AssetLoader, RenderAssetUsages};
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{
    AstcBlock, AstcChannel, Extent3d, TextureDataOrder, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages, TextureViewDescriptor, TextureViewDimension,
    WgpuFeatures as Features,
};
use bevy_basisu_loader_sys::{
    BasisuTranscoder, ChannelType as ChannelTypeSys, SupportedTextureCompressionMethods,
    TranscodedTextureFormat,
};
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

const fn channel_type_to_channel_type_sys(t: ChannelType) -> ChannelTypeSys {
    // SAFETY: Both repr are u8 and always valid.
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
    ///
    /// Note: This will be ignored when the transcode target isn't single-channel or dual-channel (like ETC2 or BC4/BC5), so this usually only has effect for ETC1S textures. See [`BasisuLoaderPlugin`](crate::BasisuLoaderPlugin) for more information about target selection.
    pub channel_type_hint: ChannelType,
    /// Forcibly transcode to a specific [`TextureFormat`]. If `None` the target format is selected automatically.
    ///
    /// Use this with caution! It will panic if the target format is not supported by the device,
    /// and will crash if it can't be transcoded by Basis Universal (TODO: This is unsafe).
    ///
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
            let _span = bevy::log::info_span!("transcoding basisu texture").entered();
            let time = if log::STATIC_MAX_LEVEL >= log::LevelFilter::Debug {
                Some(bevy::platform::time::Instant::now())
            } else {
                None
            };
            let mut transcoder = BasisuTranscoder::new();
            let Some(info) = transcoder.start(
                data,
                self.supported_compressed_formats,
                channel_type_to_channel_type_sys(settings.channel_type_hint),
            ) else {
                return Err(BasisuLoaderError::TranscodingError("transcoder.start"));
            };

            let view_dimension = if info.layers == 0 {
                if info.faces == 1 {
                    TextureViewDimension::D2
                } else if info.faces == 6 {
                    TextureViewDimension::Cube
                } else {
                    unreachable!()
                }
            } else if info.faces == 1 {
                TextureViewDimension::D2Array
            } else if info.faces == 6 {
                TextureViewDimension::CubeArray
            } else {
                unreachable!()
            };
            let extent = Extent3d {
                width: info.width,
                height: info.height,
                depth_or_array_layers: info.layers.max(1) * info.faces,
            };
            let target = validate_transcode_target_format(
                settings.force_transcode_target,
                self.supported_compressed_formats,
            )
            .unwrap_or(info.preferred_target);
            let out_format = texture_transcode_format_to_wgpu_format(
                target,
                settings.is_srgb.unwrap_or(info.is_srgb),
            );
            let Some(out_data) = transcoder.output(target) else {
                return Err(BasisuLoaderError::TranscodingError("transcoder.output"));
            };

            if log::STATIC_MAX_LEVEL >= log::LevelFilter::Debug {
                bevy::log::debug!(
                    "Transcoded a basisu texture {:?} -> {:?}, {:?}kb -> {:?}kb, preferred_target {:?}, extents {:?}, levels {:?}, view_dimension {:?}, in {:?}",
                    info.basis_format,
                    out_format,
                    src_bytes as f32 / 1000.0,
                    out_data.len() as f32 / 1000.0,
                    info.preferred_target,
                    extent,
                    info.levels,
                    view_dimension,
                    time.unwrap().elapsed(),
                );
            }

            (out_data, out_format, extent, info.levels, view_dimension)
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

fn texture_transcode_format_to_wgpu_format(
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

fn validate_transcode_target_format(
    format: Option<TextureFormat>,
    supported_methods: SupportedTextureCompressionMethods,
) -> Option<TranscodedTextureFormat> {
    let format = format?.remove_srgb_suffix();
    let target = match format {
        TextureFormat::Etc2Rgb8Unorm
            if supported_methods.0 & SupportedTextureCompressionMethods::ETC2.0 != 0 =>
        {
            TranscodedTextureFormat::cTFETC1_RGB
        }
        TextureFormat::Etc2Rgba8Unorm
            if supported_methods.0 & SupportedTextureCompressionMethods::ETC2.0 != 0 =>
        {
            TranscodedTextureFormat::cTFETC2_RGBA
        }
        TextureFormat::Bc1RgbaUnorm
            if supported_methods.0 & SupportedTextureCompressionMethods::BC.0 != 0 =>
        {
            TranscodedTextureFormat::cTFBC1_RGB
        }
        TextureFormat::Bc3RgbaUnorm
            if supported_methods.0 & SupportedTextureCompressionMethods::BC.0 != 0 =>
        {
            TranscodedTextureFormat::cTFBC3_RGBA
        }
        TextureFormat::Bc4RUnorm
            if supported_methods.0 & SupportedTextureCompressionMethods::BC.0 != 0 =>
        {
            TranscodedTextureFormat::cTFBC4_R
        }
        TextureFormat::Bc5RgUnorm
            if supported_methods.0 & SupportedTextureCompressionMethods::BC.0 != 0 =>
        {
            TranscodedTextureFormat::cTFBC5_RG
        }
        TextureFormat::Bc7RgbaUnorm
            if supported_methods.0 & SupportedTextureCompressionMethods::BC.0 != 0 =>
        {
            TranscodedTextureFormat::cTFBC7_RGBA
        }
        TextureFormat::Astc {
            block: AstcBlock::B4x4,
            channel: AstcChannel::Unorm,
        } if supported_methods.0 & SupportedTextureCompressionMethods::ASTC_LDR.0 != 0 => {
            TranscodedTextureFormat::cTFASTC_4x4_RGBA
        }
        TextureFormat::EacR11Unorm
            if supported_methods.0 & SupportedTextureCompressionMethods::ETC2.0 != 0 =>
        {
            TranscodedTextureFormat::cTFETC2_EAC_R11
        }
        TextureFormat::EacRg11Unorm
            if supported_methods.0 & SupportedTextureCompressionMethods::ETC2.0 != 0 =>
        {
            TranscodedTextureFormat::cTFETC2_EAC_RG11
        }
        TextureFormat::Bc6hRgbUfloat
            if supported_methods.0 & SupportedTextureCompressionMethods::BC.0 != 0 =>
        {
            TranscodedTextureFormat::cTFBC6H
        }
        TextureFormat::Astc {
            block: AstcBlock::B4x4,
            channel: AstcChannel::Hdr,
        } if supported_methods.0 & SupportedTextureCompressionMethods::ASTC_HDR.0 != 0 => {
            TranscodedTextureFormat::cTFASTC_HDR_4x4_RGBA
        }
        TextureFormat::Rgba8Unorm => TranscodedTextureFormat::cTFRGBA32,
        TextureFormat::Rgba16Float => TranscodedTextureFormat::cTFRGBA_HALF,
        TextureFormat::Rgb9e5Ufloat => TranscodedTextureFormat::cTFRGB_9E5,
        TextureFormat::Astc {
            block: AstcBlock::B6x6,
            channel: AstcChannel::Hdr,
        } if supported_methods.0 & SupportedTextureCompressionMethods::ASTC_HDR.0 != 0 => {
            TranscodedTextureFormat::cTFASTC_HDR_6x6_RGBA
        }
        _ => unreachable!(),
    };
    Some(target)
}
