use crate::{
    extra::types,
    transcoder as trans_sys,
    utils::{BasisTextureFormat, TranscodeTargetFormat},
};
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use core::num::NonZero;
#[cfg(not(feature = "std"))]
use spin::Once;
#[cfg(feature = "std")]
use std::sync::Once;

#[derive(Debug, Clone, PartialEq)]
pub struct TranscodedImage {
    /// The output data of image pixels.
    /// The data order currently is always `TextureDataOrder::MipMajor`.
    pub data: Vec<u8>,
    /// The size of transcoded image.
    pub size: types::Extent3d,
    /// The format of transcoded image.
    pub format: types::TextureFormat,
    /// The mip level count of transcoded image.
    pub mip_level_count: u32,
    /// The texture view dimension of transcoded image. It can be D2, D2Array, Cube or CubeArray.
    pub view_dimension: types::TextureViewDimension,
}

static BASISU_TRANSCODER_INITIALIZED: Once = Once::new();

pub(crate) fn transcoder_is_init() -> bool {
    #[cfg(not(feature = "std"))]
    return BASISU_TRANSCODER_INITIALIZED.get().is_some();
    #[cfg(feature = "std")]
    return BASISU_TRANSCODER_INITIALIZED.is_completed();
}

pub(crate) fn transcoder_is_uninit() -> bool {
    !transcoder_is_init()
}

/// Init global data of transcoder ([`trans_sys::bt_init`]).
pub fn basisu_transcoder_init() {
    BASISU_TRANSCODER_INITIALIZED.call_once(|| unsafe {
        trans_sys::bt_init();
    });
}

/// A wrapper of [`trans_sys::bt_enable_debug_printf`].
pub fn basisu_transcoder_enable_debug_printf(enable: bool) {
    unsafe { trans_sys::bt_enable_debug_printf(enable as u32) };
}

#[derive(Debug, thiserror::Error, PartialEq)]
#[non_exhaustive]
pub enum BasisuTranscodeError {
    #[error("Input data is empty")]
    EmptyInputData,
    #[error("Failed to load ktx2 data, likely the input data isn't valid")]
    LoadKtx2DataFailed,
    #[error("Invalid ktx2 face count. It must be 1 or 6, got {0}")]
    InvalidFaceCount(u32),
    #[error("`BasisuTranscoder::prepare` isn't called before transcoding")]
    UnsupportedTranscodeTarget,
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
    ktx2_data: Ktx2Data,
    info: TranscodeInfo,
}

struct Ktx2Data {
    #[expect(
        unused,
        reason = "This is kept to remain memory, which is referenced by ktx2 handle"
    )]
    data: Box<[u8]>,
    ktx2_handle: NonZero<u64>,
}

impl Ktx2Data {
    fn new(data: Box<[u8]>) -> Option<Self> {
        NonZero::try_from(unsafe {
            trans_sys::bt_ktx2_open(data.as_ptr().addr() as u64, u32::try_from(data.len()).ok()?)
        })
        .ok()
        .map(|ktx2_handle| Self { data, ktx2_handle })
    }
    #[inline]
    fn ktx2_handle(&self) -> NonZero<u64> {
        self.ktx2_handle
    }
}

impl Drop for Ktx2Data {
    fn drop(&mut self) {
        unsafe { trans_sys::bt_ktx2_close(self.ktx2_handle().into()) };
    }
}

impl BasisuTranscoder {
    /// Create a transcoder and set the input data of ktx2 basisu file in bytes.
    ///
    /// Panic if [`basisu_transcoder_init`] hasn't been called.
    ///
    /// `supported_compressed_formats` and `channel_type_hint` affect the preferred transcode target.
    ///
    /// `channel_type_hint` only has effect if the basisu texture is ETC1S, or device only supports ETC2.  If it's [`ChannelType::Auto`], it will be determined automatically according to ktx2 dfd channel ids.
    ///
    /// Default transcode target selection:
    ///
    /// | BasisU format                  | Target selection                                               |
    /// | ------------------------------ | -------------------------------------------------------------- |
    /// | ETC1S                          | Bc7Rgba/Bc5Rg/Bc4R > Etc2Rgba8/Etc2Rgb8/EacRg11/EacR11 > Rgba8 |
    /// | UASTC_LDR, ASTC_LDR, XUASTC_LDR| Astc > Bc7Rgba > Etc2Rgba8/Etc2Rgb8/EacRg11/EacR11 > Rgba8     |
    /// | UASTC_HDR, ASTC_HDR            | Astc > Bc6hRgbUfloat > Rgba16Float                             |
    pub fn new(
        input: &[u8],
        supported_compressed_formats: SupportedTextureCompression,
        channel_type_hint: ChannelType,
    ) -> Result<Self, BasisuTranscodeError> {
        if transcoder_is_uninit() {
            panic!("`basisu_transcoder_init` must be called before create transcoder");
        }
        unsafe {
            if input.is_empty() {
                return Err(BasisuTranscodeError::EmptyInputData);
            }
            let Some(ktx2_data) = Ktx2Data::new(input.into()) else {
                return Err(BasisuTranscodeError::LoadKtx2DataFailed);
            };
            let ktx2_handle_ptr = u64::from(ktx2_data.ktx2_handle());
            if trans_sys::bt_ktx2_start_transcoding(ktx2_handle_ptr).is_err() {
                return Err(BasisuTranscodeError::BtStartTranscodingFailed);
            }
            let faces = trans_sys::bt_ktx2_get_faces(ktx2_handle_ptr);
            if faces != 1 && faces != 6 {
                return Err(BasisuTranscodeError::InvalidFaceCount(faces));
            }
            let width = trans_sys::bt_ktx2_get_width(ktx2_handle_ptr);
            let height = trans_sys::bt_ktx2_get_height(ktx2_handle_ptr);
            let layers = trans_sys::bt_ktx2_get_layers(ktx2_handle_ptr);
            let levels = trans_sys::bt_ktx2_get_levels(ktx2_handle_ptr);
            let is_srgb = trans_sys::bt_ktx2_is_srgb(ktx2_handle_ptr).is_ok();
            let basis_format = BasisTextureFormat::try_from(
                trans_sys::bt_ktx2_get_basis_tex_format(ktx2_handle_ptr),
            )
            .unwrap();
            let channel_id0 = trans_sys::bt_ktx2_get_dfd_channel_id0(ktx2_handle_ptr);
            let channel_id1 = trans_sys::bt_ktx2_get_dfd_channel_id1(ktx2_handle_ptr);
            let is_uastc = trans_sys::bt_ktx2_is_uastc_ldr_4x4(ktx2_handle_ptr).is_ok();
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
            Ok(Self { ktx2_data, info })
        }
    }

    /// Get the input info [`TranscodeInfo`].
    pub fn get_info(&self) -> TranscodeInfo {
        self.info
    }

    /// Transcode the input data and return the image. Return error if it's not prepared or transcoding failed.
    ///
    /// If `transcode_target`/`is_srgb` is None, it will use [`TranscodeInfo::preferred_target`]/[`TranscodeInfo::is_srgb`].
    pub fn transcode(
        &self,
        transcode_target: Option<TranscodeTargetFormat>,
        is_srgb: Option<bool>,
    ) -> Result<TranscodedImage, BasisuTranscodeError> {
        let info = self.info;
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
        let ktx2_handle_ptr = u64::from(self.ktx2_data.ktx2_handle());
        let data = unsafe {
            for level_index in 0..info.levels {
                for layer_index in 0..total_layers {
                    for face_index in 0..info.faces {
                        let orig_width = trans_sys::bt_ktx2_get_level_orig_width(
                            ktx2_handle_ptr,
                            level_index,
                            layer_index,
                            face_index,
                        );
                        let orig_height = trans_sys::bt_ktx2_get_level_orig_height(
                            ktx2_handle_ptr,
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

            let mut basisu_heap = vec![0u8; total_bytes as usize];
            let basisu_ptr = basisu_heap.as_mut_ptr().addr() as u64;
            let mut offset = 0u64;
            for level_index in 0..info.levels {
                for layer_index in 0..total_layers {
                    for face_index in 0..info.faces {
                        let bytes_per_block_or_pixel =
                            trans_sys::bt_basis_get_bytes_per_block_or_pixel(
                                transcode_target as u32,
                            );
                        let orig_width = trans_sys::bt_ktx2_get_level_orig_width(
                            ktx2_handle_ptr,
                            level_index,
                            layer_index,
                            face_index,
                        );
                        let orig_height = trans_sys::bt_ktx2_get_level_orig_height(
                            ktx2_handle_ptr,
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
                            ktx2_handle_ptr,
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
                            return Err(BasisuTranscodeError::BtTranscodeImageLevelFailed);
                        }
                        offset += bytes as u64;
                    }
                }
            }
            basisu_heap
        };

        let view_dimension = if info.layers == 0 {
            if info.faces == 1 {
                types::TextureViewDimension::D2
            } else if info.faces == 6 {
                types::TextureViewDimension::Cube
            } else {
                return Err(BasisuTranscodeError::InvalidFaceCount(info.faces));
            }
        } else if info.faces == 1 {
            types::TextureViewDimension::D2Array
        } else if info.faces == 6 {
            types::TextureViewDimension::CubeArray
        } else {
            return Err(BasisuTranscodeError::InvalidFaceCount(info.faces));
        };
        let extent = types::Extent3d {
            width: info.width,
            height: info.height,
            depth_or_array_layers: total_layers * info.faces,
        };
        let out_format = if is_srgb.unwrap_or(info.is_srgb) {
            out_format.add_srgb_suffix()
        } else {
            out_format.remove_srgb_suffix()
        };
        Ok(TranscodedImage {
            data,
            // Note: we must give wgpu the logical texture dimensions, so it can correctly compute mip sizes.
            // However this currently causes wgpu to panic if the dimensions aren't a multiple of blocksize.
            // See https://github.com/gfx-rs/wgpu/issues/7677 for more context.
            size: extent,
            format: out_format,
            mip_level_count: info.levels,
            view_dimension,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    let select_xubc7 = || {
        if supported_compressed_formats.contains(SupportedTextureCompression::BC) {
            TranscodeTargetFormat::Bc7Rgba
        } else if supported_compressed_formats.contains(SupportedTextureCompression::ASTC_LDR) {
            TranscodeTargetFormat::AstcLdr4x4Rgba
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
    match basis_format {
        BasisTextureFormat::Etc1s => select_etc1s(),
        BasisTextureFormat::UastcLdr4x4 => select_astc_ldr(),
        BasisTextureFormat::UastcHdr4x4 => select_hdr_4x4(),
        BasisTextureFormat::AstcHdr6x6 => select_hdr_6x6(),
        BasisTextureFormat::UastcHdr6x6 => select_hdr_6x6(),
        BasisTextureFormat::Xubc7 => select_xubc7(),
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

pub fn transcode_target_to_wgpu_format(
    transcode: TranscodeTargetFormat,
) -> Option<types::TextureFormat> {
    use types::{AstcBlock, AstcChannel, TextureFormat};

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

    #[test]
    #[should_panic]
    fn transcoder_create_before_init() {
        if super::transcoder_is_init() {
            panic!("Basisu is already initialized, panic to skip this test");
        } else {
            let _ = super::BasisuTranscoder::new(
                &[],
                super::SupportedTextureCompression::empty(),
                super::ChannelType::Auto,
            );
        }
    }
}
