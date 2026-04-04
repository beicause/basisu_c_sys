use crate::common;

/// A enum that wraps `common::BTF_*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u32)]
pub enum BasisTextureFormat {
    Etc1s = common::BTF_ETC1S,
    UastcLdr4x4 = common::BTF_UASTC_LDR_4X4,
    UastcHdr4x4 = common::BTF_UASTC_HDR_4X4,
    AstcHdr6x6 = common::BTF_ASTC_HDR_6X6,
    UastcHdr6x6 = common::BTF_UASTC_HDR_6X6,
    XuastcLdr4x4 = common::BTF_XUASTC_LDR_4X4,
    XuastcLdr5x4 = common::BTF_XUASTC_LDR_5X4,
    XuastcLdr5x5 = common::BTF_XUASTC_LDR_5X5,
    XuastcLdr6x5 = common::BTF_XUASTC_LDR_6X5,
    XuastcLdr6x6 = common::BTF_XUASTC_LDR_6X6,
    XuastcLdr8x5 = common::BTF_XUASTC_LDR_8X5,
    XuastcLdr8x6 = common::BTF_XUASTC_LDR_8X6,
    XuastcLdr10x5 = common::BTF_XUASTC_LDR_10X5,
    XuastcLdr10x6 = common::BTF_XUASTC_LDR_10X6,
    XuastcLdr8x8 = common::BTF_XUASTC_LDR_8X8,
    XuastcLdr10x8 = common::BTF_XUASTC_LDR_10X8,
    XuastcLdr10x10 = common::BTF_XUASTC_LDR_10X10,
    XuastcLdr12x10 = common::BTF_XUASTC_LDR_12X10,
    XuastcLdr12x12 = common::BTF_XUASTC_LDR_12X12,
    AstcLdr4x4 = common::BTF_ASTC_LDR_4X4,
    AstcLdr5x4 = common::BTF_ASTC_LDR_5X4,
    AstcLdr5x5 = common::BTF_ASTC_LDR_5X5,
    AstcLdr6x5 = common::BTF_ASTC_LDR_6X5,
    AstcLdr6x6 = common::BTF_ASTC_LDR_6X6,
    AstcLdr8x5 = common::BTF_ASTC_LDR_8X5,
    AstcLdr8x6 = common::BTF_ASTC_LDR_8X6,
    AstcLdr10x5 = common::BTF_ASTC_LDR_10X5,
    AstcLdr10x6 = common::BTF_ASTC_LDR_10X6,
    AstcLdr8x8 = common::BTF_ASTC_LDR_8X8,
    AstcLdr10x8 = common::BTF_ASTC_LDR_10X8,
    AstcLdr10x10 = common::BTF_ASTC_LDR_10X10,
    AstcLdr12x10 = common::BTF_ASTC_LDR_12X10,
    AstcLdr12x12 = common::BTF_ASTC_LDR_12X12,
}

impl TryFrom<u32> for BasisTextureFormat {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value > common::BTF_ASTC_LDR_12X12 {
            Err(())
        } else {
            Ok(unsafe { core::mem::transmute::<u32, BasisTextureFormat>(value) })
        }
    }
}

/// A enum that wraps `common::TF_*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u32)]
pub enum TranscodeTargetFormat {
    Etc1Rgb = common::TF_ETC1_RGB,
    Etc2Rgba = common::TF_ETC2_RGBA,
    Bc1Rgb = common::TF_BC1_RGB,
    Bc3Rgba = common::TF_BC3_RGBA,
    Bc4R = common::TF_BC4_R,
    Bc5Rg = common::TF_BC5_RG,
    Bc7Rgba = common::TF_BC7_RGBA,
    Pvrtc1_4Rgb = common::TF_PVRTC1_4_RGB,
    Pvrtc1_4Rgba = common::TF_PVRTC1_4_RGBA,
    AstcLdr4x4Rgba = common::TF_ASTC_LDR_4X4_RGBA,
    AtcRgb = common::TF_ATC_RGB,
    AtcRgba = common::TF_ATC_RGBA,
    Fxt1Rgb = common::TF_FXT1_RGB,
    Pvrtc2_4Rgb = common::TF_PVRTC2_4_RGB,
    Pvrtc2_4Rgba = common::TF_PVRTC2_4_RGBA,
    Etc2EacR11 = common::TF_ETC2_EAC_R11,
    Etc2EacRg11 = common::TF_ETC2_EAC_RG11,
    Bc6H = common::TF_BC6H,
    AstcHdr4x4Rgba = common::TF_ASTC_HDR_4X4_RGBA,
    RGBA32 = common::TF_RGBA32,
    RGB565 = common::TF_RGB565,
    BGR565 = common::TF_BGR565,
    RGBA4444 = common::TF_RGBA4444,
    RgbHalf = common::TF_RGB_HALF,
    RgbaHalf = common::TF_RGBA_HALF,
    Rgb9e5 = common::TF_RGB_9E5,
    AstcHdr6x6Rgba = common::TF_ASTC_HDR_6X6_RGBA,
    AstcLdr5x4Rgba = common::TF_ASTC_LDR_5X4_RGBA,
    AstcLdr5x5Rgba = common::TF_ASTC_LDR_5X5_RGBA,
    AstcLdr6x5Rgba = common::TF_ASTC_LDR_6X5_RGBA,
    AstcLdr6x6Rgba = common::TF_ASTC_LDR_6X6_RGBA,
    AstcLdr8x5Rgba = common::TF_ASTC_LDR_8X5_RGBA,
    AstcLdr8x6Rgba = common::TF_ASTC_LDR_8X6_RGBA,
    AstcLdr10x5Rgba = common::TF_ASTC_LDR_10X5_RGBA,
    AstcLdr10x6Rgba = common::TF_ASTC_LDR_10X6_RGBA,
    AstcLdr8x8Rgba = common::TF_ASTC_LDR_8X8_RGBA,
    AstcLdr10x8Rgba = common::TF_ASTC_LDR_10X8_RGBA,
    AstcLdr10x10Rgba = common::TF_ASTC_LDR_10X10_RGBA,
    AstcLdr12x10Rgba = common::TF_ASTC_LDR_12X10_RGBA,
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
