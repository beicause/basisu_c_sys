use bevy::asset::{AssetLoader, RenderAssetUsages};
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{
    AstcBlock, AstcChannel, Extent3d, TextureDataOrder, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages, TextureViewDescriptor, TextureViewDimension,
    WgpuFeatures as Features,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct SupportedTextureCompression: u8 {
        const ETC2 = 1;
        const BC = 1 << 1;
        const ASTC_LDR = 1 << 2;
        const ASTC_HDR = 1 << 3;
    }
}

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
}

/// An error when loading an image using [`BasisuLoader`].
#[derive(Debug, Error)]
#[non_exhaustive]
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

fn texture_transcode_format_to_wgpu_format(transcoded: u32, is_srgb: bool) -> TextureFormat {
    let mut fmt = match transcoded {
        basisu_c_sys::common::TF_ETC1_RGB => TextureFormat::Etc2Rgb8Unorm,
        basisu_c_sys::common::TF_ETC2_RGBA => TextureFormat::Etc2Rgba8Unorm,
        basisu_c_sys::common::TF_BC1_RGB => TextureFormat::Bc1RgbaUnorm,
        basisu_c_sys::common::TF_BC3_RGBA => TextureFormat::Bc3RgbaUnorm,
        basisu_c_sys::common::TF_BC4_R => TextureFormat::Bc4RUnorm,
        basisu_c_sys::common::TF_BC5_RG => TextureFormat::Bc5RgUnorm,
        basisu_c_sys::common::TF_BC7_RGBA => TextureFormat::Bc7RgbaUnorm,
        basisu_c_sys::common::TF_PVRTC1_4_RGB => unreachable!(),
        basisu_c_sys::common::TF_PVRTC1_4_RGBA => unreachable!(),
        basisu_c_sys::common::TF_ASTC_LDR_4X4_RGBA => TextureFormat::Astc {
            block: AstcBlock::B4x4,
            channel: AstcChannel::Unorm,
        },
        basisu_c_sys::common::TF_ATC_RGB => unreachable!(),
        basisu_c_sys::common::TF_ATC_RGBA => unreachable!(),
        basisu_c_sys::common::TF_FXT1_RGB => unreachable!(),
        basisu_c_sys::common::TF_PVRTC2_4_RGB => unreachable!(),
        basisu_c_sys::common::TF_PVRTC2_4_RGBA => unreachable!(),
        basisu_c_sys::common::TF_ETC2_EAC_R11 => TextureFormat::EacR11Unorm,
        basisu_c_sys::common::TF_ETC2_EAC_RG11 => TextureFormat::EacRg11Unorm,
        basisu_c_sys::common::TF_BC6H => TextureFormat::Bc6hRgbUfloat,
        basisu_c_sys::common::TF_ASTC_HDR_4X4_RGBA => TextureFormat::Astc {
            block: AstcBlock::B4x4,
            channel: AstcChannel::Hdr,
        },
        basisu_c_sys::common::TF_RGBA32 => TextureFormat::Rgba8Unorm,
        basisu_c_sys::common::TF_RGB565 => unreachable!(),
        basisu_c_sys::common::TF_BGR565 => unreachable!(),
        basisu_c_sys::common::TF_RGBA4444 => unreachable!(),
        basisu_c_sys::common::TF_RGB_HALF => unreachable!(),
        basisu_c_sys::common::TF_RGBA_HALF => TextureFormat::Rgba16Float,
        basisu_c_sys::common::TF_RGB_9E5 => TextureFormat::Rgb9e5Ufloat,
        basisu_c_sys::common::TF_ASTC_HDR_6X6_RGBA => TextureFormat::Astc {
            block: AstcBlock::B6x6,
            channel: AstcChannel::Hdr,
        },
        basisu_c_sys::common::TF_ASTC_LDR_5X4_RGBA => TextureFormat::Astc {
            block: AstcBlock::B5x4,
            channel: AstcChannel::Unorm,
        },
        basisu_c_sys::common::TF_ASTC_LDR_5X5_RGBA => TextureFormat::Astc {
            block: AstcBlock::B5x5,
            channel: AstcChannel::Unorm,
        },
        basisu_c_sys::common::TF_ASTC_LDR_6X5_RGBA => TextureFormat::Astc {
            block: AstcBlock::B6x5,
            channel: AstcChannel::Unorm,
        },
        basisu_c_sys::common::TF_ASTC_LDR_6X6_RGBA => TextureFormat::Astc {
            block: AstcBlock::B6x6,
            channel: AstcChannel::Unorm,
        },
        basisu_c_sys::common::TF_ASTC_LDR_8X5_RGBA => TextureFormat::Astc {
            block: AstcBlock::B8x5,
            channel: AstcChannel::Unorm,
        },
        basisu_c_sys::common::TF_ASTC_LDR_8X6_RGBA => TextureFormat::Astc {
            block: AstcBlock::B8x6,
            channel: AstcChannel::Unorm,
        },
        basisu_c_sys::common::TF_ASTC_LDR_10X5_RGBA => TextureFormat::Astc {
            block: AstcBlock::B10x5,
            channel: AstcChannel::Unorm,
        },
        basisu_c_sys::common::TF_ASTC_LDR_10X6_RGBA => TextureFormat::Astc {
            block: AstcBlock::B10x6,
            channel: AstcChannel::Unorm,
        },
        basisu_c_sys::common::TF_ASTC_LDR_8X8_RGBA => TextureFormat::Astc {
            block: AstcBlock::B8x8,
            channel: AstcChannel::Unorm,
        },
        basisu_c_sys::common::TF_ASTC_LDR_10X8_RGBA => TextureFormat::Astc {
            block: AstcBlock::B10x8,
            channel: AstcChannel::Unorm,
        },
        basisu_c_sys::common::TF_ASTC_LDR_10X10_RGBA => TextureFormat::Astc {
            block: AstcBlock::B10x10,
            channel: AstcChannel::Unorm,
        },
        basisu_c_sys::common::TF_ASTC_LDR_12X10_RGBA => TextureFormat::Astc {
            block: AstcBlock::B12x10,
            channel: AstcChannel::Unorm,
        },
        basisu_c_sys::common::TF_ASTC_LDR_12X12_RGBA => TextureFormat::Astc {
            block: AstcBlock::B12x12,
            channel: AstcChannel::Unorm,
        },
        _ => unreachable!(),
    };
    if is_srgb {
        fmt = fmt.add_srgb_suffix();
    }
    fmt
}
