use basisu_sys::{TextureCompressionMethod, TextureTranscodedFormat};
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

pub struct BasisuLoader {
    supported_compressed_formats: TextureCompressionMethod,
}

impl BasisuLoader {
    pub fn from_features(features: Features) -> Self {
        let mut supported_compressed_formats = TextureCompressionMethod::NONE;
        if features.contains(Features::TEXTURE_COMPRESSION_ASTC) {
            supported_compressed_formats |= TextureCompressionMethod::ASTC_LDR;
        }
        if features.contains(Features::TEXTURE_COMPRESSION_ASTC_HDR) {
            supported_compressed_formats |= TextureCompressionMethod::ASTC_HDR;
        }
        if features.contains(Features::TEXTURE_COMPRESSION_BC) {
            supported_compressed_formats |= TextureCompressionMethod::BC;
        }
        if features.contains(Features::TEXTURE_COMPRESSION_ETC2) {
            supported_compressed_formats |= TextureCompressionMethod::ETC2;
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
        // SAFETY: Ensure the transcoding code is correct.
        let (out_data, out_format, extent, levels, view_dimension) = unsafe {
            let transcoder = basisu_sys::ktx2_transcoder_new();
            if transcoder.is_null() {
                return Err(BasisuLoaderError::TranscodingError("ktx2_transcoder_new"));
            }
            if !basisu_sys::ktx2_transcoder_transcode_image(
                transcoder,
                data,
                self.supported_compressed_formats,
            ) {
                return Err(BasisuLoaderError::TranscodingError(
                    "ktx2_transcoder_transcode_image",
                ));
            }

            let is_srgb = basisu_sys::ktx2_transcoder_get_r_is_srgb(transcoder);
            let target_format = basisu_sys::ktx2_transcoder_get_r_target_format(transcoder);

            let width = basisu_sys::ktx2_transcoder_get_r_width(transcoder);
            let height = basisu_sys::ktx2_transcoder_get_r_height(transcoder);
            let levels = basisu_sys::ktx2_transcoder_get_r_levels(transcoder);
            let layers = basisu_sys::ktx2_transcoder_get_r_layers(transcoder);
            let faces = basisu_sys::ktx2_transcoder_get_r_faces(transcoder);
            let dst_bytes = basisu_sys::ktx2_transcoder_get_r_dst_buf(transcoder);

            let view_dimension = if layers == 0 {
                if faces == 1 {
                    TextureViewDimension::D2
                } else if faces == 6 {
                    TextureViewDimension::Cube
                } else {
                    unreachable!()
                }
            } else if faces == 1 {
                TextureViewDimension::D2Array
            } else if faces == 6 {
                TextureViewDimension::CubeArray
            } else {
                unreachable!()
            };
            let extent = Extent3d {
                width,
                height,
                depth_or_array_layers: layers.max(1) * faces,
            };

            basisu_sys::ktx2_transcoder_delete(transcoder);
            (
                dst_bytes,
                texture_transcode_format_to_bevy_format(target_format, is_srgb),
                extent,
                levels,
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
        &["basisu_ktx2"]
    }
}

fn texture_transcode_format_to_bevy_format(
    transcoded: TextureTranscodedFormat,
    is_srgb: bool,
) -> TextureFormat {
    let mut fmt = match transcoded {
        TextureTranscodedFormat::cTFETC1_RGB => unreachable!(),
        TextureTranscodedFormat::cTFETC2_RGBA => TextureFormat::Etc2Rgba8Unorm,
        TextureTranscodedFormat::cTFBC1_RGB => TextureFormat::Bc1RgbaUnorm,
        TextureTranscodedFormat::cTFBC3_RGBA => TextureFormat::Bc3RgbaUnorm,
        TextureTranscodedFormat::cTFBC4_R => TextureFormat::Bc4RUnorm,
        TextureTranscodedFormat::cTFBC5_RG => TextureFormat::Bc5RgUnorm,
        TextureTranscodedFormat::cTFBC7_RGBA => TextureFormat::Bc7RgbaUnorm,
        TextureTranscodedFormat::cTFPVRTC1_4_RGB => unreachable!(),
        TextureTranscodedFormat::cTFPVRTC1_4_RGBA => unreachable!(),
        TextureTranscodedFormat::cTFASTC_4x4_RGBA => TextureFormat::Astc {
            block: AstcBlock::B4x4,
            channel: AstcChannel::Unorm,
        },
        TextureTranscodedFormat::cTFATC_RGB => unreachable!(),
        TextureTranscodedFormat::cTFATC_RGBA => unreachable!(),
        TextureTranscodedFormat::cTFFXT1_RGB => unreachable!(),
        TextureTranscodedFormat::cTFPVRTC2_4_RGB => unreachable!(),
        TextureTranscodedFormat::cTFPVRTC2_4_RGBA => unreachable!(),
        TextureTranscodedFormat::cTFETC2_EAC_R11 => TextureFormat::EacR11Unorm,
        TextureTranscodedFormat::cTFETC2_EAC_RG11 => TextureFormat::EacRg11Unorm,
        TextureTranscodedFormat::cTFBC6H => TextureFormat::Bc6hRgbUfloat,
        TextureTranscodedFormat::cTFASTC_HDR_4x4_RGBA => TextureFormat::Astc {
            block: AstcBlock::B4x4,
            channel: AstcChannel::Hdr,
        },
        TextureTranscodedFormat::cTFRGBA32 => TextureFormat::Rgba8Unorm,
        TextureTranscodedFormat::cTFRGB565 => unreachable!(),
        TextureTranscodedFormat::cTFBGR565 => unreachable!(),
        TextureTranscodedFormat::cTFRGBA4444 => unreachable!(),
        TextureTranscodedFormat::cTFRGB_HALF => unreachable!(),
        TextureTranscodedFormat::cTFRGBA_HALF => TextureFormat::Rgba16Float,
        TextureTranscodedFormat::cTFRGB_9E5 => TextureFormat::Rgb9e5Ufloat,
        TextureTranscodedFormat::cTFASTC_HDR_6x6_RGBA => TextureFormat::Astc {
            block: AstcBlock::B6x6,
            channel: AstcChannel::Hdr,
        },
        TextureTranscodedFormat::cTFTotalTextureFormats => unreachable!(),
        TextureTranscodedFormat::cTFBC7_ALT => unreachable!(),
        _ => unreachable!(),
    };
    if is_srgb {
        fmt = fmt.add_srgb_suffix();
    }
    fmt
}
