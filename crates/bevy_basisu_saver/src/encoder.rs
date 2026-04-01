use basisu_c_sys::encoder;
use bevy::{
    image::Image,
    render::render_resource::{TextureDimension, TextureFormat, TextureViewDimension},
};
use serde::{Deserialize, Serialize};

use std::sync::OnceLock;

static BASISU_INITIALIZED: OnceLock<()> = OnceLock::new();

pub fn basisu_init() {
    BASISU_INITIALIZED.get_or_init(|| {
        unsafe { encoder::bu_init() };
    });
}

pub fn basisu_enable_debug_printf(enable: bool) {
    unsafe { encoder::bu_enable_debug_printf(enable as u32) };
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(u32)]
pub enum BasisTextureFormat {
    Etc1s = encoder::BTF_ETC1S,
    UastcLdr4x4 = encoder::BTF_UASTC_LDR_4X4,
    UastcHdr4x4 = encoder::BTF_UASTC_HDR_4X4,
    AstcHdr6x6 = encoder::BTF_ASTC_HDR_6X6,
    UastcHdr6x6 = encoder::BTF_UASTC_HDR_6X6,
    XuastcLdr4x4 = encoder::BTF_XUASTC_LDR_4X4,
    XuastcLdr5x4 = encoder::BTF_XUASTC_LDR_5X4,
    XuastcLdr5x5 = encoder::BTF_XUASTC_LDR_5X5,
    XuastcLdr6x5 = encoder::BTF_XUASTC_LDR_6X5,
    XuastcLdr6x6 = encoder::BTF_XUASTC_LDR_6X6,
    XuastcLdr8x5 = encoder::BTF_XUASTC_LDR_8X5,
    XuastcLdr8x6 = encoder::BTF_XUASTC_LDR_8X6,
    XuastcLdr10x5 = encoder::BTF_XUASTC_LDR_10X5,
    XuastcLdr10x6 = encoder::BTF_XUASTC_LDR_10X6,
    XuastcLdr8x8 = encoder::BTF_XUASTC_LDR_8X8,
    XuastcLdr10x8 = encoder::BTF_XUASTC_LDR_10X8,
    XuastcLdr10x10 = encoder::BTF_XUASTC_LDR_10X10,
    XuastcLdr12x10 = encoder::BTF_XUASTC_LDR_12X10,
    XuastcLdr12x12 = encoder::BTF_XUASTC_LDR_12X12,
    AstcLdr4x4 = encoder::BTF_ASTC_LDR_4X4,
    AstcLdr5x4 = encoder::BTF_ASTC_LDR_5X4,
    AstcLdr5x5 = encoder::BTF_ASTC_LDR_5X5,
    AstcLdr6x5 = encoder::BTF_ASTC_LDR_6X5,
    AstcLdr6x6 = encoder::BTF_ASTC_LDR_6X6,
    AstcLdr8x5 = encoder::BTF_ASTC_LDR_8X5,
    AstcLdr8x6 = encoder::BTF_ASTC_LDR_8X6,
    AstcLdr10x5 = encoder::BTF_ASTC_LDR_10X5,
    AstcLdr10x6 = encoder::BTF_ASTC_LDR_10X6,
    AstcLdr8x8 = encoder::BTF_ASTC_LDR_8X8,
    AstcLdr10x8 = encoder::BTF_ASTC_LDR_10X8,
    AstcLdr10x10 = encoder::BTF_ASTC_LDR_10X10,
    AstcLdr12x10 = encoder::BTF_ASTC_LDR_12X10,
    AstcLdr12x12 = encoder::BTF_ASTC_LDR_12X12,
}

pub const BU_COMP_FLAGS_DEBUG_IMAGES: u64 = encoder::BU_COMP_FLAGS_DEBUG_IMAGES as u64;
pub const BU_COMP_FLAGS_DEBUG_OUTPUT: u64 = encoder::BU_COMP_FLAGS_DEBUG_OUTPUT as u64;
pub const BU_COMP_FLAGS_GEN_MIPS_CLAMP: u64 = encoder::BU_COMP_FLAGS_GEN_MIPS_CLAMP as u64;
pub const BU_COMP_FLAGS_GEN_MIPS_WRAP: u64 = encoder::BU_COMP_FLAGS_GEN_MIPS_WRAP as u64;
pub const BU_COMP_FLAGS_KTX2_OUTPUT: u64 = encoder::BU_COMP_FLAGS_KTX2_OUTPUT as u64;
pub const BU_COMP_FLAGS_KTX2_UASTC_ZSTD: u64 = encoder::BU_COMP_FLAGS_KTX2_UASTC_ZSTD as u64;
pub const BU_COMP_FLAGS_NONE: u64 = encoder::BU_COMP_FLAGS_NONE as u64;
pub const BU_COMP_FLAGS_PRINT_STATS: u64 = encoder::BU_COMP_FLAGS_PRINT_STATS as u64;
pub const BU_COMP_FLAGS_PRINT_STATUS: u64 = encoder::BU_COMP_FLAGS_PRINT_STATUS as u64;
pub const BU_COMP_FLAGS_REC2020: u64 = encoder::BU_COMP_FLAGS_REC2020 as u64;
pub const BU_COMP_FLAGS_SRGB: u64 = encoder::BU_COMP_FLAGS_SRGB as u64;
pub const BU_COMP_FLAGS_TEXTURE_TYPE_2D: u64 = encoder::BU_COMP_FLAGS_TEXTURE_TYPE_2D as u64;
pub const BU_COMP_FLAGS_TEXTURE_TYPE_2D_ARRAY: u64 =
    encoder::BU_COMP_FLAGS_TEXTURE_TYPE_2D_ARRAY as u64;
pub const BU_COMP_FLAGS_TEXTURE_TYPE_CUBEMAP_ARRAY: u64 =
    encoder::BU_COMP_FLAGS_TEXTURE_TYPE_CUBEMAP_ARRAY as u64;
pub const BU_COMP_FLAGS_TEXTURE_TYPE_MASK: u64 = encoder::BU_COMP_FLAGS_TEXTURE_TYPE_MASK as u64;
pub const BU_COMP_FLAGS_TEXTURE_TYPE_SHIFT: u64 = encoder::BU_COMP_FLAGS_TEXTURE_TYPE_SHIFT as u64;
pub const BU_COMP_FLAGS_TEXTURE_TYPE_VIDEO_FRAMES: u64 =
    encoder::BU_COMP_FLAGS_TEXTURE_TYPE_VIDEO_FRAMES as u64;
pub const BU_COMP_FLAGS_THREADED: u64 = encoder::BU_COMP_FLAGS_THREADED as u64;
pub const BU_COMP_FLAGS_USE_OPENCL: u64 = encoder::BU_COMP_FLAGS_USE_OPENCL as u64;
pub const BU_COMP_FLAGS_VALIDATE_OUTPUT: u64 = encoder::BU_COMP_FLAGS_VALIDATE_OUTPUT as u64;
pub const BU_COMP_FLAGS_VERBOSE: u64 = encoder::BU_COMP_FLAGS_VERBOSE as u64;
pub const BU_COMP_FLAGS_XUASTC_LDR_FULL_ARITH: u64 =
    encoder::BU_COMP_FLAGS_XUASTC_LDR_FULL_ARITH as u64;
pub const BU_COMP_FLAGS_XUASTC_LDR_FULL_ZSTD: u64 =
    encoder::BU_COMP_FLAGS_XUASTC_LDR_FULL_ZSTD as u64;
pub const BU_COMP_FLAGS_XUASTC_LDR_HYBRID: u64 = encoder::BU_COMP_FLAGS_XUASTC_LDR_HYBRID as u64;
pub const BU_COMP_FLAGS_XUASTC_LDR_SYNTAX_MASK: u64 =
    encoder::BU_COMP_FLAGS_XUASTC_LDR_SYNTAX_MASK as u64;
pub const BU_COMP_FLAGS_XUASTC_LDR_SYNTAX_SHIFT: u64 =
    encoder::BU_COMP_FLAGS_XUASTC_LDR_SYNTAX_SHIFT as u64;
pub const BU_COMP_FLAGS_Y_FLIP: u64 = encoder::BU_COMP_FLAGS_Y_FLIP as u64;

pub struct BasisuEncoder {
    params: u64,
}

impl Default for BasisuEncoder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BasisuEncodeError {
    #[error("Image data must not be None")]
    DataIsNone,
    #[error("Mip level count must be 1")]
    MipLevelCountNotOne,
    #[error("Unsupported texture format: {0:?}")]
    UnsupportedTextureFormat(TextureFormat),
    #[error("Unsupported texture dimension: {0:?}")]
    UnsupportedTextureDimension(TextureDimension),
    #[error("Unsupported texture view dimension: {0:?}")]
    UnsupportedTextureViewDimension(TextureViewDimension),
    #[error("`BaisuEncoder::set_image_slice` only accepts image with 1 layer or depth")]
    SetImageSliceOnlyAcceptsOneLayer,
    #[error("bu_comp_params_set_image_* failed")]
    BuSetImageFailed,
    #[error("bu_compress_texture failed")]
    BuCompressFailed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BasisuEncoderParams {
    /// Target file format — one of the BTF_* constants (e.g. BTF_ETC1S, BTF_UASTC_LDR_4X4).
    pub basis_tex_format: BasisTextureFormat,
    /// Unified Quality level [1,100] (see BU_QUALITY_MIN, BU_QUALITY_MAX). Use -1 to use older non-unified/direct codec-specific quality level or lambda (low 8-bits of flags_and_quality, or via low_level_uastc_rdo_or_dct_quality).
    pub quality_level: i32,
    /// Unified Encoder effort [0,10] (see BU_EFFORT_MIN, BU_EFFORT_MAX). See BU_EFFORT_* presets. Use -1 to use older non-unified/direct codec-specific effort level (low 8-bits of flags_and_quality for some codecs).
    pub effort_level: i32,
    /// Bitwise OR of BU_COMP_FLAGS_* constants. Controls output format, mipmaps, color space, etc. Low 8-bits are either the older non-unified quality level, or for some codecs the non-unified effort level.
    pub flags_and_quality: u64,
    /// Low-level (non-unified) quality or lambda parameter for UASTC RDO encoding. Typically 0.0 for defaults. Must be 0.0 if using unified (not -1) quality level.
    pub low_level_uastc_rdo_or_dct_quality: f32,
}

impl BasisuEncoderParams {
    pub const fn new_with_srgb_defaults(basis_tex_format: BasisTextureFormat) -> Self {
        Self {
            basis_tex_format,
            quality_level: 75,
            effort_level: 2,
            flags_and_quality: BU_COMP_FLAGS_THREADED
                | BU_COMP_FLAGS_SRGB
                | BU_COMP_FLAGS_KTX2_OUTPUT
                | BU_COMP_FLAGS_KTX2_UASTC_ZSTD,
            low_level_uastc_rdo_or_dct_quality: 0.0,
        }
    }

    pub const fn new_with_linear_defaults(basis_tex_format: BasisTextureFormat) -> Self {
        Self {
            basis_tex_format,
            quality_level: 75,
            effort_level: 2,
            flags_and_quality: BU_COMP_FLAGS_THREADED
                | BU_COMP_FLAGS_KTX2_OUTPUT
                | BU_COMP_FLAGS_KTX2_UASTC_ZSTD,
            low_level_uastc_rdo_or_dct_quality: 0.0,
        }
    }

    pub const fn with_tex_type(mut self, tex_type: TextureViewDimension) -> Self {
        self.flags_and_quality = self.flags_and_quality
            & !(BU_COMP_FLAGS_TEXTURE_TYPE_MASK << BU_COMP_FLAGS_TEXTURE_TYPE_SHIFT);

        self.flags_and_quality = self.flags_and_quality
            | match tex_type {
                TextureViewDimension::D2 => BU_COMP_FLAGS_TEXTURE_TYPE_2D,
                TextureViewDimension::D2Array => BU_COMP_FLAGS_TEXTURE_TYPE_2D_ARRAY,
                TextureViewDimension::Cube | TextureViewDimension::CubeArray => {
                    BU_COMP_FLAGS_TEXTURE_TYPE_CUBEMAP_ARRAY
                }
                TextureViewDimension::D1 | TextureViewDimension::D3 => {
                    panic!("Compressing 1D or 3D texture is unsupported")
                }
            };
        self
    }

    /// Add [`BU_COMP_FLAGS_VALIDATE_OUTPUT`] to param flags.
    pub const fn with_validate_output(mut self) -> Self {
        self.flags_and_quality |= BU_COMP_FLAGS_VALIDATE_OUTPUT;
        self
    }

    /// Add [`BU_COMP_FLAGS_DEBUG_OUTPUT`] to param flags.
    /// Remember to call [`basisu_enable_debug_printf`] to see the debug info.
    pub const fn with_debug_output(mut self) -> Self {
        self.flags_and_quality |= BU_COMP_FLAGS_DEBUG_OUTPUT;
        self
    }
}

impl BasisuEncoder {
    pub fn new() -> Self {
        Self {
            params: unsafe { encoder::bu_new_comp_params() },
        }
    }

    pub fn set_image(&mut self, image: &Image) -> Result<(), BasisuEncodeError> {
        self.clear_image();

        let Some(data) = image.data.as_ref() else {
            return Err(BasisuEncodeError::DataIsNone);
        };
        if image.texture_descriptor.mip_level_count != 1 {
            return Err(BasisuEncodeError::MipLevelCountNotOne);
        }
        match image.texture_descriptor.dimension {
            TextureDimension::D1 | TextureDimension::D3 => {
                return Err(BasisuEncodeError::UnsupportedTextureDimension(
                    image.texture_descriptor.dimension,
                ));
            }
            TextureDimension::D2 => {}
        }
        if let Some(view_desc) = &image.texture_view_descriptor
            && let Some(dimension) = view_desc.dimension
        {
            match dimension {
                TextureViewDimension::D1 | TextureViewDimension::D3 => {
                    return Err(BasisuEncodeError::UnsupportedTextureViewDimension(
                        dimension,
                    ));
                }
                _ => {}
            }
        };
        match image.texture_descriptor.format {
            TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb => unsafe {
                for i in 0..image.texture_descriptor.array_layer_count() {
                    if encoder::bu_comp_params_set_image_rgba32(
                        self.params,
                        i,
                        data.as_ptr() as u64 + (i * image.width() * image.height() * 4) as u64,
                        image.width(),
                        image.height(),
                        image.width() * 4,
                    ) == 0
                    {
                        return Err(BasisuEncodeError::BuSetImageFailed);
                    }
                }
            },
            TextureFormat::Rgba32Float => unsafe {
                for i in 0..image.texture_descriptor.array_layer_count() {
                    if encoder::bu_comp_params_set_image_float_rgba(
                        self.params,
                        i,
                        data.as_ptr() as u64 + (i * image.width() * image.height() * 16) as u64,
                        image.width(),
                        image.height(),
                        image.width() * 16,
                    ) == 0
                    {
                        return Err(BasisuEncodeError::BuSetImageFailed);
                    }
                }
            },
            _ => {
                return Err(BasisuEncodeError::UnsupportedTextureFormat(
                    image.texture_descriptor.format,
                ));
            }
        }
        Ok(())
    }

    pub fn clear_image(&mut self) {
        assert!(unsafe { encoder::bu_comp_params_clear(self.params) } != 0);
    }

    pub fn set_image_slice(&mut self, index: u32, image: &Image) -> Result<(), BasisuEncodeError> {
        let Some(data) = image.data.as_ref() else {
            return Err(BasisuEncodeError::DataIsNone);
        };
        if image.texture_descriptor.mip_level_count != 1 {
            return Err(BasisuEncodeError::MipLevelCountNotOne);
        }
        match image.texture_descriptor.dimension {
            TextureDimension::D1 | TextureDimension::D3 => {
                return Err(BasisuEncodeError::UnsupportedTextureDimension(
                    image.texture_descriptor.dimension,
                ));
            }
            TextureDimension::D2 => {}
        }
        if image.texture_descriptor.array_layer_count() != 1 {
            return Err(BasisuEncodeError::SetImageSliceOnlyAcceptsOneLayer);
        }
        if let Some(view_desc) = &image.texture_view_descriptor
            && let Some(dimension) = view_desc.dimension
        {
            match dimension {
                TextureViewDimension::D1 | TextureViewDimension::D3 => {
                    return Err(BasisuEncodeError::UnsupportedTextureViewDimension(
                        dimension,
                    ));
                }
                _ => {}
            }
        };
        match image.texture_descriptor.format {
            TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb => unsafe {
                if encoder::bu_comp_params_set_image_rgba32(
                    self.params,
                    index,
                    data.as_ptr() as u64,
                    image.width(),
                    image.height(),
                    image.width() * 4,
                ) == 0
                {
                    return Err(BasisuEncodeError::BuSetImageFailed);
                }
            },
            TextureFormat::Rgba32Float => unsafe {
                if encoder::bu_comp_params_set_image_float_rgba(
                    self.params,
                    index,
                    data.as_ptr() as u64,
                    image.width(),
                    image.height(),
                    image.width() * 16,
                ) == 0
                {
                    return Err(BasisuEncodeError::BuSetImageFailed);
                }
            },
            _ => {
                return Err(BasisuEncodeError::UnsupportedTextureFormat(
                    image.texture_descriptor.format,
                ));
            }
        }
        Ok(())
    }

    pub fn compress(&mut self, params: BasisuEncoderParams) -> Result<Vec<u8>, BasisuEncodeError> {
        unsafe {
            if encoder::bu_compress_texture(
                self.params,
                params.basis_tex_format as u32,
                params.quality_level,
                params.effort_level,
                params.flags_and_quality,
                params.low_level_uastc_rdo_or_dct_quality,
            ) == 0
            {
                return Err(BasisuEncodeError::BuCompressFailed);
            }
            let out_size = encoder::bu_comp_params_get_comp_data_size(self.params);
            let out_ptr = encoder::bu_comp_params_get_comp_data_ofs(self.params) as *const u8;
            let mut result = vec![0u8; out_size as usize];
            core::ptr::copy_nonoverlapping(out_ptr, result.as_mut_ptr(), out_size as usize);
            Ok(result)
        }
    }
}

impl Drop for BasisuEncoder {
    fn drop(&mut self) {
        unsafe {
            encoder::bu_delete_comp_params(self.params);
        }
    }
}
