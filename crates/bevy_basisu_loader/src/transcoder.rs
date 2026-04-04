use async_lock::OnceCell;
use basisu_c_sys::{BasisTextureFormat, TranscodeTargetFormat, transcoder as trans_sys};
use bevy::{
    image::Image,
    render::render_resource::{
        AstcBlock, AstcChannel, Extent3d, TextureDataOrder, TextureDescriptor, TextureDimension,
        TextureFormat, TextureViewDescriptor, TextureViewDimension,
    },
};

static BASISU_INITIALIZED: OnceCell<()> = OnceCell::new();

pub async fn basisu_transcoder_init() {
    BASISU_INITIALIZED
        .get_or_init(async || {
            basisu_c_sys::instantiate_embedded_basisu_wasm().await;
            unsafe { trans_sys::bt_init() };
        })
        .await;
}

#[derive(Debug, thiserror::Error)]
pub enum BasisuTranscodeError {
    #[error("Failed to load ktx2 data, likely the input data isn't valid")]
    LoadKtx2DataFailed,
    #[error("`BasisuTranscoder::prepare` isn't called before transcoding")]
    UnsupportedTranscodeTarget,
    #[error("`BasisuTranscoder::prepare` isn't called before transcoding")]
    TranscodingBeforePrepared,
    #[error("`bt_ktx2_start_transcoding` failed")]
    BtStartTranscodingFailed,
    #[error("`bt_ktx2_transcode_image_level` failed")]
    BtTranscodeImageLevelFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TranscodeInfo {
    pub width: u32,
    pub height: u32,
    pub levels: u32,
    pub layers: u32,
    pub faces: u32,
    pub is_srgb: bool,
    pub basis_format: BasisTextureFormat,
    pub preferred_target: TranscodeTargetFormat,
}

pub struct BasisuTranscoder {
    data_ptr: u64,
    ktx2_handle: u64,
    info: Option<TranscodeInfo>,
}

impl Drop for BasisuTranscoder {
    fn drop(&mut self) {
        if self.ktx2_handle != 0 {
            unsafe { trans_sys::bt_ktx2_close(self.ktx2_handle) };
        }
        if self.data_ptr != 0 {
            unsafe { trans_sys::bt_free(self.data_ptr) };
        }
    }
}
impl Default for BasisuTranscoder {
    fn default() -> Self {
        Self::new()
    }
}

impl BasisuTranscoder {
    pub fn new() -> Self {
        if !BASISU_INITIALIZED.is_initialized() {
            panic!("`basisu_transcoder_init` must be called before create transcoder");
        }
        Self {
            data_ptr: 0,
            ktx2_handle: 0,
            info: None,
        }
    }

    pub fn prepare(
        &mut self,
        input: &[u8],
        supported_compressed_formats: SupportedTextureCompression,
        channel_type_hint: ChannelType,
    ) -> Result<TranscodeInfo, BasisuTranscodeError> {
        unsafe {
            if self.data_ptr != 0 {
                trans_sys::bt_free(self.data_ptr);
                self.data_ptr = 0;
            }
            if self.ktx2_handle != 0 {
                trans_sys::bt_ktx2_close(self.ktx2_handle);
                self.ktx2_handle = 0;
            }

            let basisu_ptr = trans_sys::bt_alloc(input.len() as u64);
            basisu_c_sys::copy_host_memory_to_basisu(input, basisu_ptr);
            let ktx2_handle = trans_sys::bt_ktx2_open(basisu_ptr, input.len() as u32);
            if ktx2_handle == 0 {
                return Err(BasisuTranscodeError::LoadKtx2DataFailed);
            }
            if trans_sys::bt_ktx2_start_transcoding(ktx2_handle).is_err() {
                trans_sys::bt_free(basisu_ptr);
                trans_sys::bt_ktx2_close(ktx2_handle);
                return Err(BasisuTranscodeError::BtStartTranscodingFailed);
            }
            let width = trans_sys::bt_ktx2_get_width(ktx2_handle);
            let height = trans_sys::bt_ktx2_get_height(ktx2_handle);
            let layers = trans_sys::bt_ktx2_get_layers(ktx2_handle);
            let levels = trans_sys::bt_ktx2_get_levels(ktx2_handle);
            let faces = trans_sys::bt_ktx2_get_faces(ktx2_handle);
            let is_srgb = trans_sys::bt_ktx2_is_srgb(ktx2_handle).is_ok();
            let basis_format =
                BasisTextureFormat::try_from(trans_sys::bt_ktx2_get_basis_tex_format(ktx2_handle))
                    .unwrap();
            let channel_id0 = trans_sys::bt_ktx2_get_dfd_channel_id0(ktx2_handle);
            let channel_id1 = trans_sys::bt_ktx2_get_dfd_channel_id1(ktx2_handle);
            let is_uastc = trans_sys::bt_ktx2_is_uastc_ldr_4x4(ktx2_handle).is_ok();
            let channel_type = if channel_type_hint != ChannelType::Auto {
                channel_type_hint
            } else {
                channel_id_to_type(is_uastc, channel_id0, channel_id1)
            };
            let preferred_target = select_preferred_transcode_target(
                basis_format,
                channel_type,
                supported_compressed_formats,
            );

            let info = TranscodeInfo {
                width,
                height,
                levels,
                layers,
                faces,
                is_srgb,
                basis_format,
                preferred_target,
            };

            self.data_ptr = basisu_ptr;
            self.ktx2_handle = ktx2_handle;
            self.info = Some(info);
            Ok(info)
        }
    }

    pub fn get_prepared_info(&self) -> Option<TranscodeInfo> {
        self.info
    }

    pub fn transcode(
        &self,
        transcode_target: Option<TranscodeTargetFormat>,
        is_srgb: Option<bool>,
    ) -> Result<Image, BasisuTranscodeError> {
        if self.data_ptr == 0 || self.ktx2_handle == 0 {
            return Err(BasisuTranscodeError::TranscodingBeforePrepared);
        }
        let info = self
            .info
            .ok_or(BasisuTranscodeError::TranscodingBeforePrepared)?;
        let transcode_target = transcode_target.unwrap_or(info.preferred_target);
        if unsafe {
            trans_sys::bt_basis_is_format_supported(
                transcode_target as u32,
                info.basis_format as u32,
            )
            .is_err()
        } {
            return Err(BasisuTranscodeError::UnsupportedTranscodeTarget);
        }
        let out_format = transcode_target_to_wgpu_format(transcode_target)
            .ok_or(BasisuTranscodeError::UnsupportedTranscodeTarget)?;
        let total_layers = info.layers.max(1);
        let mut total_bytes = 0;
        let data = unsafe {
            for level_index in 0..info.levels {
                for layer_index in 0..total_layers {
                    for face_index in 0..info.faces {
                        let orig_width = trans_sys::bt_ktx2_get_level_orig_width(
                            self.ktx2_handle,
                            level_index,
                            layer_index,
                            face_index,
                        );
                        let orig_height = trans_sys::bt_ktx2_get_level_orig_height(
                            self.ktx2_handle,
                            level_index,
                            layer_index,
                            face_index,
                        );
                        let bytes = trans_sys::bt_basis_compute_transcoded_image_size_in_bytes(
                            transcode_target as u32,
                            orig_width,
                            orig_height,
                        );
                        total_bytes += bytes;
                    }
                }
            }

            let basisu_ptr = trans_sys::bt_alloc(total_bytes.into());
            let mut offset = 0u64;
            for level_index in 0..info.levels {
                for layer_index in 0..total_layers {
                    for face_index in 0..info.faces {
                        let bytes_per_block_or_pixel =
                            trans_sys::bt_basis_get_bytes_per_block_or_pixel(
                                transcode_target as u32,
                            );
                        let orig_width = trans_sys::bt_ktx2_get_level_orig_width(
                            self.ktx2_handle,
                            level_index,
                            layer_index,
                            face_index,
                        );
                        let orig_height = trans_sys::bt_ktx2_get_level_orig_height(
                            self.ktx2_handle,
                            level_index,
                            layer_index,
                            face_index,
                        );
                        let bytes = trans_sys::bt_basis_compute_transcoded_image_size_in_bytes(
                            transcode_target as u32,
                            orig_width,
                            orig_height,
                        );
                        let blocks = bytes / bytes_per_block_or_pixel;
                        if trans_sys::bt_ktx2_transcode_image_level(
                            self.ktx2_handle,
                            level_index,
                            layer_index,
                            face_index,
                            basisu_ptr + offset,
                            blocks,
                            transcode_target as u32,
                            0,
                            0,
                            0,
                            -1,
                            -1,
                            0,
                        )
                        .is_err()
                        {
                            trans_sys::bt_free(basisu_ptr);
                            return Err(BasisuTranscodeError::BtTranscodeImageLevelFailed);
                        }
                        offset += bytes as u64;
                    }
                }
            }
            let data = basisu_c_sys::copy_basisu_memory_to_host(basisu_ptr, total_bytes.into());
            trans_sys::bt_free(basisu_ptr);
            data
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
            depth_or_array_layers: total_layers * info.faces,
        };
        let out_format = if is_srgb.unwrap_or(info.is_srgb) {
            out_format.add_srgb_suffix()
        } else {
            out_format.remove_srgb_suffix()
        };
        let default = Image::default();
        Ok(Image {
            data: Some(data),
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
                mip_level_count: info.levels,
                ..default.texture_descriptor
            },
            texture_view_descriptor: Some(TextureViewDescriptor {
                dimension: Some(view_dimension),
                ..Default::default()
            }),
            ..default
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ChannelType {
    #[default]
    Auto,
    Rgba,
    Rgb,
    Rg,
    R,
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct SupportedTextureCompression: u8 {
        const ETC2 = 1;
        const BC = 1 << 1;
        const ASTC_LDR = 1 << 2;
        const ASTC_HDR = 1 << 3;
    }
}

const KTX2_DF_CHANNEL_ETC1S_RGB: u32 = 0;
const KTX2_DF_CHANNEL_ETC1S_RRR: u32 = 3;
const KTX2_DF_CHANNEL_ETC1S_GGG: u32 = 4;
const KTX2_DF_CHANNEL_ETC1S_AAA: u32 = 15;

const KTX2_DF_CHANNEL_UASTC_RGB: u32 = 0;
const KTX2_DF_CHANNEL_UASTC_RGBA: u32 = 3;
const KTX2_DF_CHANNEL_UASTC_RRR: u32 = 4;
const KTX2_DF_CHANNEL_UASTC_RRRG: u32 = 5;
const KTX2_DF_CHANNEL_UASTC_RG: u32 = 6;

fn channel_id_to_type(is_uastc: bool, channel_id0: u32, channel_id1: u32) -> ChannelType {
    if is_uastc {
        match channel_id0 {
            KTX2_DF_CHANNEL_UASTC_RGB => ChannelType::Rgb,
            KTX2_DF_CHANNEL_UASTC_RGBA => ChannelType::Rgba,
            KTX2_DF_CHANNEL_UASTC_RRR => ChannelType::R,
            KTX2_DF_CHANNEL_UASTC_RRRG => ChannelType::Rg,
            KTX2_DF_CHANNEL_UASTC_RG => ChannelType::Rg,
            _ => ChannelType::Rgba,
        }
    } else {
        if channel_id0 == KTX2_DF_CHANNEL_ETC1S_RGB && channel_id1 != KTX2_DF_CHANNEL_ETC1S_AAA {
            ChannelType::Rgb
        } else if channel_id0 == KTX2_DF_CHANNEL_ETC1S_RGB
            && channel_id1 == KTX2_DF_CHANNEL_ETC1S_AAA
        {
            ChannelType::Rgba
        } else if channel_id0 == KTX2_DF_CHANNEL_ETC1S_RRR
            && channel_id1 != KTX2_DF_CHANNEL_ETC1S_GGG
        {
            ChannelType::R
        } else if channel_id0 == KTX2_DF_CHANNEL_ETC1S_RRR
            && channel_id1 == KTX2_DF_CHANNEL_ETC1S_GGG
        {
            ChannelType::Rg
        } else {
            ChannelType::Rgba
        }
    }
}

// Select target format according to https://github.com/KhronosGroup/3D-Formats-Guidelines/blob/main/KTXDeveloperGuide.md.
fn select_preferred_transcode_target(
    basis_format: BasisTextureFormat,
    channel_type: ChannelType,
    supported_compressed_formats: SupportedTextureCompression,
) -> TranscodeTargetFormat {
    let select_hdr_4x4 = || {
        if supported_compressed_formats.contains(SupportedTextureCompression::ASTC_HDR) {
            TranscodeTargetFormat::AstcHdr4x4Rgba
        } else if supported_compressed_formats.contains(SupportedTextureCompression::BC) {
            TranscodeTargetFormat::Bc6H
        } else {
            TranscodeTargetFormat::RgbaHalf
        }
    };
    let select_hdr_6x6 = || {
        if supported_compressed_formats.contains(SupportedTextureCompression::ASTC_HDR) {
            TranscodeTargetFormat::AstcHdr6x6Rgba
        } else if supported_compressed_formats.contains(SupportedTextureCompression::BC) {
            TranscodeTargetFormat::Bc6H
        } else {
            TranscodeTargetFormat::RgbaHalf
        }
    };
    let select_astc_ldr = || {
        if supported_compressed_formats.contains(SupportedTextureCompression::ASTC_LDR) {
            TranscodeTargetFormat::try_from(unsafe {
                trans_sys::bt_basis_get_transcoder_texture_format_from_basis_tex_format(
                    basis_format as u32,
                )
            })
            .unwrap()
        } else if supported_compressed_formats.contains(SupportedTextureCompression::BC) {
            TranscodeTargetFormat::Bc7Rgba
        } else if supported_compressed_formats.contains(SupportedTextureCompression::ETC2) {
            match channel_type {
                ChannelType::Rgb => TranscodeTargetFormat::Etc1Rgb,
                ChannelType::Rgba | ChannelType::Auto => TranscodeTargetFormat::Etc2Rgba,
                ChannelType::R => TranscodeTargetFormat::Etc2EacR11,
                ChannelType::Rg => TranscodeTargetFormat::Etc2EacRg11,
            }
        } else {
            TranscodeTargetFormat::RGBA32
        }
    };
    let select_etc1s = || {
        if supported_compressed_formats.contains(SupportedTextureCompression::ETC2) {
            match channel_type {
                ChannelType::Rgb => TranscodeTargetFormat::Etc1Rgb,
                ChannelType::Rgba | ChannelType::Auto => TranscodeTargetFormat::Etc2Rgba,
                ChannelType::R => TranscodeTargetFormat::Etc2EacR11,
                ChannelType::Rg => TranscodeTargetFormat::Etc2EacRg11,
            }
        } else if supported_compressed_formats.contains(SupportedTextureCompression::BC) {
            match channel_type {
                ChannelType::Rgb => TranscodeTargetFormat::Bc7Rgba,
                ChannelType::Rgba | ChannelType::Auto => TranscodeTargetFormat::Bc7Rgba,
                ChannelType::R => TranscodeTargetFormat::Bc4R,
                ChannelType::Rg => TranscodeTargetFormat::Bc5Rg,
            }
        } else {
            TranscodeTargetFormat::RGBA32
        }
    };
    match basis_format {
        BasisTextureFormat::Etc1s => select_etc1s(),
        BasisTextureFormat::UastcLdr4x4 => select_astc_ldr(),
        BasisTextureFormat::UastcHdr4x4 => select_hdr_4x4(),
        BasisTextureFormat::AstcHdr6x6 => select_hdr_6x6(),
        BasisTextureFormat::UastcHdr6x6 => select_hdr_6x6(),
        BasisTextureFormat::XuastcLdr4x4
        | BasisTextureFormat::XuastcLdr5x4
        | BasisTextureFormat::XuastcLdr5x5
        | BasisTextureFormat::XuastcLdr6x5
        | BasisTextureFormat::XuastcLdr6x6
        | BasisTextureFormat::XuastcLdr8x5
        | BasisTextureFormat::XuastcLdr8x6
        | BasisTextureFormat::XuastcLdr10x5
        | BasisTextureFormat::XuastcLdr10x6
        | BasisTextureFormat::XuastcLdr8x8
        | BasisTextureFormat::XuastcLdr10x8
        | BasisTextureFormat::XuastcLdr10x10
        | BasisTextureFormat::XuastcLdr12x10
        | BasisTextureFormat::XuastcLdr12x12
        | BasisTextureFormat::AstcLdr4x4
        | BasisTextureFormat::AstcLdr5x4
        | BasisTextureFormat::AstcLdr5x5
        | BasisTextureFormat::AstcLdr6x5
        | BasisTextureFormat::AstcLdr6x6
        | BasisTextureFormat::AstcLdr8x5
        | BasisTextureFormat::AstcLdr8x6
        | BasisTextureFormat::AstcLdr10x5
        | BasisTextureFormat::AstcLdr10x6
        | BasisTextureFormat::AstcLdr8x8
        | BasisTextureFormat::AstcLdr10x8
        | BasisTextureFormat::AstcLdr10x10
        | BasisTextureFormat::AstcLdr12x10
        | BasisTextureFormat::AstcLdr12x12 => select_astc_ldr(),
    }
}

pub fn transcode_target_to_wgpu_format(transcode: TranscodeTargetFormat) -> Option<TextureFormat> {
    Some(match transcode {
        TranscodeTargetFormat::Etc1Rgb => TextureFormat::Etc2Rgb8Unorm,
        TranscodeTargetFormat::Etc2Rgba => TextureFormat::Etc2Rgba8Unorm,
        TranscodeTargetFormat::Bc1Rgb => TextureFormat::Bc1RgbaUnorm,
        TranscodeTargetFormat::Bc3Rgba => TextureFormat::Bc3RgbaUnorm,
        TranscodeTargetFormat::Bc4R => TextureFormat::Bc4RUnorm,
        TranscodeTargetFormat::Bc5Rg => TextureFormat::Bc5RgUnorm,
        TranscodeTargetFormat::Bc7Rgba => TextureFormat::Bc7RgbaUnorm,
        TranscodeTargetFormat::Pvrtc1_4Rgb => return None,
        TranscodeTargetFormat::Pvrtc1_4Rgba => return None,
        TranscodeTargetFormat::AstcLdr4x4Rgba => TextureFormat::Astc {
            block: AstcBlock::B4x4,
            channel: AstcChannel::Unorm,
        },
        TranscodeTargetFormat::AtcRgb => return None,
        TranscodeTargetFormat::AtcRgba => return None,
        TranscodeTargetFormat::Fxt1Rgb => return None,
        TranscodeTargetFormat::Pvrtc2_4Rgb => return None,
        TranscodeTargetFormat::Pvrtc2_4Rgba => return None,
        TranscodeTargetFormat::Etc2EacR11 => TextureFormat::EacR11Unorm,
        TranscodeTargetFormat::Etc2EacRg11 => TextureFormat::EacRg11Unorm,
        TranscodeTargetFormat::Bc6H => TextureFormat::Bc6hRgbUfloat,
        TranscodeTargetFormat::AstcHdr4x4Rgba => TextureFormat::Astc {
            block: AstcBlock::B4x4,
            channel: AstcChannel::Hdr,
        },
        TranscodeTargetFormat::RGBA32 => TextureFormat::Rgba8Unorm,
        TranscodeTargetFormat::RGB565 => return None,
        TranscodeTargetFormat::BGR565 => return None,
        TranscodeTargetFormat::RGBA4444 => return None,
        TranscodeTargetFormat::RgbHalf => return None,
        TranscodeTargetFormat::RgbaHalf => TextureFormat::Rgba16Float,
        TranscodeTargetFormat::Rgb9e5 => TextureFormat::Rgb9e5Ufloat,
        TranscodeTargetFormat::AstcHdr6x6Rgba => TextureFormat::Astc {
            block: AstcBlock::B6x6,
            channel: AstcChannel::Hdr,
        },
        TranscodeTargetFormat::AstcLdr5x4Rgba => TextureFormat::Astc {
            block: AstcBlock::B5x4,
            channel: AstcChannel::Unorm,
        },
        TranscodeTargetFormat::AstcLdr5x5Rgba => TextureFormat::Astc {
            block: AstcBlock::B5x5,
            channel: AstcChannel::Unorm,
        },
        TranscodeTargetFormat::AstcLdr6x5Rgba => TextureFormat::Astc {
            block: AstcBlock::B6x5,
            channel: AstcChannel::Unorm,
        },
        TranscodeTargetFormat::AstcLdr6x6Rgba => TextureFormat::Astc {
            block: AstcBlock::B6x6,
            channel: AstcChannel::Unorm,
        },
        TranscodeTargetFormat::AstcLdr8x5Rgba => TextureFormat::Astc {
            block: AstcBlock::B8x5,
            channel: AstcChannel::Unorm,
        },
        TranscodeTargetFormat::AstcLdr8x6Rgba => TextureFormat::Astc {
            block: AstcBlock::B8x6,
            channel: AstcChannel::Unorm,
        },
        TranscodeTargetFormat::AstcLdr10x5Rgba => TextureFormat::Astc {
            block: AstcBlock::B10x5,
            channel: AstcChannel::Unorm,
        },
        TranscodeTargetFormat::AstcLdr10x6Rgba => TextureFormat::Astc {
            block: AstcBlock::B10x6,
            channel: AstcChannel::Unorm,
        },
        TranscodeTargetFormat::AstcLdr8x8Rgba => TextureFormat::Astc {
            block: AstcBlock::B8x8,
            channel: AstcChannel::Unorm,
        },
        TranscodeTargetFormat::AstcLdr10x8Rgba => TextureFormat::Astc {
            block: AstcBlock::B10x8,
            channel: AstcChannel::Unorm,
        },
        TranscodeTargetFormat::AstcLdr10x10Rgba => TextureFormat::Astc {
            block: AstcBlock::B10x10,
            channel: AstcChannel::Unorm,
        },
        TranscodeTargetFormat::AstcLdr12x10Rgba => TextureFormat::Astc {
            block: AstcBlock::B12x10,
            channel: AstcChannel::Unorm,
        },
        TranscodeTargetFormat::AstcLdr12x12Rgba => TextureFormat::Astc {
            block: AstcBlock::B12x12,
            channel: AstcChannel::Unorm,
        },
    })
}

#[cfg(test)]
mod tests {
    use basisu_c_sys::TranscodeTargetFormat;

    use crate::transcoder::{
        BASISU_INITIALIZED, BasisuTranscoder, ChannelType, SupportedTextureCompression,
        basisu_transcoder_init,
    };

    #[test]
    #[should_panic]
    fn transcoder_create_before_init() {
        if BASISU_INITIALIZED.is_initialized() {
            panic!("Basisu is already initialized, panic to skip this test");
        } else {
            BasisuTranscoder::new();
        }
    }

    #[test]
    fn transcode_invalid_data_info_is_none() {
        block_on(basisu_transcoder_init());
        let mut transcoder = BasisuTranscoder::new();
        let info = transcoder.prepare(&[], SupportedTextureCompression::empty(), ChannelType::Auto);
        assert!(info.is_err());
        let info = transcoder.prepare(
            &[1, 2, 1],
            SupportedTextureCompression::BC
                | SupportedTextureCompression::ASTC_LDR
                | SupportedTextureCompression::ASTC_HDR
                | SupportedTextureCompression::ETC2,
            ChannelType::Auto,
        );
        assert!(info.is_err());
    }

    #[test]
    fn transcode_before_prepare_panic() {
        block_on(basisu_transcoder_init());
        let transcoder = BasisuTranscoder::new();
        assert!(
            transcoder
                .transcode(Some(TranscodeTargetFormat::RGBA32), None)
                .is_err(),
        );
    }

    #[test]
    fn transcode_invalid_data_output_panic() {
        block_on(basisu_transcoder_init());
        let mut transcoder = BasisuTranscoder::new();
        let _info = transcoder.prepare(
            &[1, 2, 1],
            SupportedTextureCompression::empty(),
            ChannelType::Auto,
        );
        let res = transcoder.transcode(Some(TranscodeTargetFormat::RGBA32), None);
        assert!(res.is_err())
    }

    // Use macro to make `assert_debug_snapshot` produce correct file name.
    // See https://github.com/mitsuhiko/insta/issues/357
    macro_rules! snapshot_test {
        ($prefix: expr, $supported_format: expr $(,)?) => {
            let mut path = std::path::PathBuf::new();
            path.push(std::env!("CARGO_MANIFEST_DIR"));
            path.push("../../assets");
            block_on(basisu_transcoder_init());
            let mut results = Vec::new();
            let mut transcoder = BasisuTranscoder::new();
            for file in std::fs::read_dir(path).unwrap() {
                let file = file.unwrap();
                let file_name = file.file_name().into_string().unwrap();
                if !file_name.ends_with(".basisu.ktx2") {
                    continue;
                }
                let data = std::fs::read(file.path()).unwrap();
                let info = transcoder
                    .prepare(&data, $supported_format, ChannelType::Auto)
                    .unwrap();
                let mut image = transcoder.transcode(None, None).unwrap();
                insta::assert_binary_snapshot!(
                    &($prefix.to_string() + &file_name.replace(".basisu.ktx2", ".bin")),
                    image.data.take().unwrap()
                );
                results.push((file_name, info, image));
            }
            results.sort_unstable_by(|a, b| a.0.cmp(&b.0));
            insta::assert_debug_snapshot!(results);
        };
    }

    #[test]
    #[cfg(not(target_os = "macos"))] // This test failed on macos, disable it for now.
    fn transcode_assets_bcn() {
        snapshot_test!("bcn_", SupportedTextureCompression::BC);
    }

    #[test]
    fn transcode_assets_astc() {
        snapshot_test!(
            "astc_",
            SupportedTextureCompression::ASTC_LDR | SupportedTextureCompression::ASTC_HDR,
        );
    }

    /// Blocks on the supplied `future`.
    /// This implementation will busy-wait until it is completed.
    /// Consider enabling the `async-io` or `futures-lite` features.
    fn block_on<T>(future: impl Future<Output = T>) -> T {
        use core::task::{Context, Poll};

        // Pin the future on the stack.
        let mut future = core::pin::pin!(future);

        // We don't care about the waker as we're just going to poll as fast as possible.
        let cx = &mut Context::from_waker(core::task::Waker::noop());

        // Keep polling until the future is ready.
        loop {
            match future.as_mut().poll(cx) {
                Poll::Ready(output) => return output,
                Poll::Pending => core::hint::spin_loop(),
            }
        }
    }
}
