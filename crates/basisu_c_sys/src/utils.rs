use crate::common;

/// An enum that wraps the `common::BTF_*` texture formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u32)]
pub enum BasisTextureFormat {
    /// ETC1S: a low-bitrate LDR format whose RGB and alpha slices are decoded separately.
    Etc1s = common::BTF_ETC1S,
    /// UASTC LDR with 4x4 blocks (16 bytes per block).
    UastcLdr4x4 = common::BTF_UASTC_LDR_4X4,
    /// UASTC HDR with 4x4 blocks; transcodes to ASTC HDR or BC6H.
    UastcHdr4x4 = common::BTF_UASTC_HDR_4X4,
    /// ASTC HDR with 6x6 blocks; passed through unchanged.
    AstcHdr6x6 = common::BTF_ASTC_HDR_6X6,
    /// UASTC HDR with 6x6 blocks; transcodes to ASTC HDR or BC6H.
    UastcHdr6x6 = common::BTF_UASTC_HDR_6X6,
    /// XUASTC LDR with 4x4 blocks.
    XuastcLdr4x4 = common::BTF_XUASTC_LDR_4X4,
    /// XUASTC LDR with 5x4 blocks.
    XuastcLdr5x4 = common::BTF_XUASTC_LDR_5X4,
    /// XUASTC LDR with 5x5 blocks.
    XuastcLdr5x5 = common::BTF_XUASTC_LDR_5X5,
    /// XUASTC LDR with 6x5 blocks.
    XuastcLdr6x5 = common::BTF_XUASTC_LDR_6X5,
    /// XUASTC LDR with 6x6 blocks.
    XuastcLdr6x6 = common::BTF_XUASTC_LDR_6X6,
    /// XUASTC LDR with 8x5 blocks.
    XuastcLdr8x5 = common::BTF_XUASTC_LDR_8X5,
    /// XUASTC LDR with 8x6 blocks.
    XuastcLdr8x6 = common::BTF_XUASTC_LDR_8X6,
    /// XUASTC LDR with 10x5 blocks.
    XuastcLdr10x5 = common::BTF_XUASTC_LDR_10X5,
    /// XUASTC LDR with 10x6 blocks.
    XuastcLdr10x6 = common::BTF_XUASTC_LDR_10X6,
    /// XUASTC LDR with 8x8 blocks.
    XuastcLdr8x8 = common::BTF_XUASTC_LDR_8X8,
    /// XUASTC LDR with 10x8 blocks.
    XuastcLdr10x8 = common::BTF_XUASTC_LDR_10X8,
    /// XUASTC LDR with 10x10 blocks.
    XuastcLdr10x10 = common::BTF_XUASTC_LDR_10X10,
    /// XUASTC LDR with 12x10 blocks.
    XuastcLdr12x10 = common::BTF_XUASTC_LDR_12X10,
    /// XUASTC LDR with 12x12 blocks.
    XuastcLdr12x12 = common::BTF_XUASTC_LDR_12X12,
    /// ASTC LDR with 4x4 blocks; passed through unchanged.
    AstcLdr4x4 = common::BTF_ASTC_LDR_4X4,
    /// ASTC LDR with 5x4 blocks; passed through unchanged.
    AstcLdr5x4 = common::BTF_ASTC_LDR_5X4,
    /// ASTC LDR with 5x5 blocks; passed through unchanged.
    AstcLdr5x5 = common::BTF_ASTC_LDR_5X5,
    /// ASTC LDR with 6x5 blocks; passed through unchanged.
    AstcLdr6x5 = common::BTF_ASTC_LDR_6X5,
    /// ASTC LDR with 6x6 blocks; passed through unchanged.
    AstcLdr6x6 = common::BTF_ASTC_LDR_6X6,
    /// ASTC LDR with 8x5 blocks; passed through unchanged.
    AstcLdr8x5 = common::BTF_ASTC_LDR_8X5,
    /// ASTC LDR with 8x6 blocks; passed through unchanged.
    AstcLdr8x6 = common::BTF_ASTC_LDR_8X6,
    /// ASTC LDR with 10x5 blocks; passed through unchanged.
    AstcLdr10x5 = common::BTF_ASTC_LDR_10X5,
    /// ASTC LDR with 10x6 blocks; passed through unchanged.
    AstcLdr10x6 = common::BTF_ASTC_LDR_10X6,
    /// ASTC LDR with 8x8 blocks; passed through unchanged.
    AstcLdr8x8 = common::BTF_ASTC_LDR_8X8,
    /// ASTC LDR with 10x8 blocks; passed through unchanged.
    AstcLdr10x8 = common::BTF_ASTC_LDR_10X8,
    /// ASTC LDR with 10x10 blocks; passed through unchanged.
    AstcLdr10x10 = common::BTF_ASTC_LDR_10X10,
    /// ASTC LDR with 12x10 blocks; passed through unchanged.
    AstcLdr12x10 = common::BTF_ASTC_LDR_12X10,
    /// ASTC LDR with 12x12 blocks; passed through unchanged.
    AstcLdr12x12 = common::BTF_ASTC_LDR_12X12,
    /// XUBC7: a BC7-family LDR format that transcodes to BC7 or ASTC.
    Xubc7 = common::BTF_XUBC7,
}

impl TryFrom<u32> for BasisTextureFormat {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value >= common::BTF_TOTAL_FORMATS {
            Err(())
        } else {
            // SAFETY: `value` within [0, `common::BTF_TOTAL_FORMATS`] is a valid enum.
            Ok(unsafe { core::mem::transmute::<u32, BasisTextureFormat>(value) })
        }
    }
}

/// An enum that wraps the `common::TF_*` transcode target formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u32)]
pub enum TranscodeTargetFormat {
    /// ETC1 RGB. Decoded as `TextureFormat::Etc2Rgb8Unorm`.
    Etc1Rgb = common::TF_ETC1_RGB,
    /// ETC2 RGBA.
    Etc2Rgba = common::TF_ETC2_RGBA,
    /// BC1 (DXT1) RGB.
    Bc1Rgb = common::TF_BC1_RGB,
    /// BC3 (DXT5) RGBA.
    Bc3Rgba = common::TF_BC3_RGBA,
    /// BC4 single-channel.
    Bc4R = common::TF_BC4_R,
    /// BC5 dual-channel.
    Bc5Rg = common::TF_BC5_RG,
    /// BC7 RGBA.
    Bc7Rgba = common::TF_BC7_RGBA,
    /// PVRTC1 4bpp RGB. Not supported by wgpu.
    Pvrtc1_4Rgb = common::TF_PVRTC1_4_RGB,
    /// PVRTC1 4bpp RGBA. Not supported by wgpu.
    Pvrtc1_4Rgba = common::TF_PVRTC1_4_RGBA,
    /// ASTC LDR 4x4 RGBA.
    AstcLdr4x4Rgba = common::TF_ASTC_LDR_4X4_RGBA,
    /// ATC RGB. Not supported by wgpu.
    AtcRgb = common::TF_ATC_RGB,
    /// ATC RGBA. Not supported by wgpu.
    AtcRgba = common::TF_ATC_RGBA,
    /// FXT1 RGB. Not supported by wgpu.
    Fxt1Rgb = common::TF_FXT1_RGB,
    /// PVRTC2 4bpp RGB. Not supported by wgpu.
    Pvrtc2_4Rgb = common::TF_PVRTC2_4_RGB,
    /// PVRTC2 4bpp RGBA. Not supported by wgpu.
    Pvrtc2_4Rgba = common::TF_PVRTC2_4_RGBA,
    /// ETC2 EAC single-channel (R11).
    Etc2EacR11 = common::TF_ETC2_EAC_R11,
    /// ETC2 EAC dual-channel (RG11).
    Etc2EacRg11 = common::TF_ETC2_EAC_RG11,
    /// BC6H RGB half-float (HDR).
    Bc6H = common::TF_BC6H,
    /// ASTC HDR 4x4 RGBA.
    AstcHdr4x4Rgba = common::TF_ASTC_HDR_4X4_RGBA,
    /// Uncompressed 8-bit RGBA.
    RGBA32 = common::TF_RGBA32,
    /// Uncompressed 16-bit RGB565.
    RGB565 = common::TF_RGB565,
    /// Uncompressed 16-bit BGR565.
    BGR565 = common::TF_BGR565,
    /// Uncompressed 16-bit RGBA4444.
    RGBA4444 = common::TF_RGBA4444,
    /// Uncompressed 16-bit half-float RGB.
    RgbHalf = common::TF_RGB_HALF,
    /// Uncompressed 16-bit half-float RGBA.
    RgbaHalf = common::TF_RGBA_HALF,
    /// Uncompressed shared-exponent 9/9/9/5-bit RGB.
    Rgb9e5 = common::TF_RGB_9E5,
    /// ASTC HDR 6x6 RGBA.
    AstcHdr6x6Rgba = common::TF_ASTC_HDR_6X6_RGBA,
    /// ASTC LDR 5x4 RGBA.
    AstcLdr5x4Rgba = common::TF_ASTC_LDR_5X4_RGBA,
    /// ASTC LDR 5x5 RGBA.
    AstcLdr5x5Rgba = common::TF_ASTC_LDR_5X5_RGBA,
    /// ASTC LDR 6x5 RGBA.
    AstcLdr6x5Rgba = common::TF_ASTC_LDR_6X5_RGBA,
    /// ASTC LDR 6x6 RGBA.
    AstcLdr6x6Rgba = common::TF_ASTC_LDR_6X6_RGBA,
    /// ASTC LDR 8x5 RGBA.
    AstcLdr8x5Rgba = common::TF_ASTC_LDR_8X5_RGBA,
    /// ASTC LDR 8x6 RGBA.
    AstcLdr8x6Rgba = common::TF_ASTC_LDR_8X6_RGBA,
    /// ASTC LDR 10x5 RGBA.
    AstcLdr10x5Rgba = common::TF_ASTC_LDR_10X5_RGBA,
    /// ASTC LDR 10x6 RGBA.
    AstcLdr10x6Rgba = common::TF_ASTC_LDR_10X6_RGBA,
    /// ASTC LDR 8x8 RGBA.
    AstcLdr8x8Rgba = common::TF_ASTC_LDR_8X8_RGBA,
    /// ASTC LDR 10x8 RGBA.
    AstcLdr10x8Rgba = common::TF_ASTC_LDR_10X8_RGBA,
    /// ASTC LDR 10x10 RGBA.
    AstcLdr10x10Rgba = common::TF_ASTC_LDR_10X10_RGBA,
    /// ASTC LDR 12x10 RGBA.
    AstcLdr12x10Rgba = common::TF_ASTC_LDR_12X10_RGBA,
    /// ASTC LDR 12x12 RGBA.
    AstcLdr12x12Rgba = common::TF_ASTC_LDR_12X12_RGBA,
}

impl TryFrom<u32> for TranscodeTargetFormat {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value >= common::TF_TOTAL_TEXTURE_FORMATS {
            Err(())
        } else {
            Ok(unsafe { core::mem::transmute::<u32, TranscodeTargetFormat>(value) })
        }
    }
}
