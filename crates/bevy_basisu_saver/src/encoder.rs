use basisu_c_sys::encoder;
use bevy::{
    image::Image,
    render::render_resource::{TextureFormat, TextureViewDimension},
};
use serde::{Deserialize, Serialize};

use std::sync::OnceLock;

static BASISU_INITIALIZED: OnceLock<()> = OnceLock::new();

pub fn basisu_init() {
    BASISU_INITIALIZED.get_or_init(|| {
        unsafe { encoder::bu_init() };
    });
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

pub use encoder::BU_COMP_FLAGS_DEBUG_IMAGES;
pub use encoder::BU_COMP_FLAGS_DEBUG_OUTPUT;
pub use encoder::BU_COMP_FLAGS_GEN_MIPS_CLAMP;
pub use encoder::BU_COMP_FLAGS_GEN_MIPS_WRAP;
pub use encoder::BU_COMP_FLAGS_KTX2_OUTPUT;
pub use encoder::BU_COMP_FLAGS_KTX2_UASTC_ZSTD;
pub use encoder::BU_COMP_FLAGS_NONE;
pub use encoder::BU_COMP_FLAGS_PRINT_STATS;
pub use encoder::BU_COMP_FLAGS_PRINT_STATUS;
pub use encoder::BU_COMP_FLAGS_REC2020;
pub use encoder::BU_COMP_FLAGS_SRGB;
pub use encoder::BU_COMP_FLAGS_TEXTURE_TYPE_2D;
pub use encoder::BU_COMP_FLAGS_TEXTURE_TYPE_2D_ARRAY;
pub use encoder::BU_COMP_FLAGS_TEXTURE_TYPE_CUBEMAP_ARRAY;
pub use encoder::BU_COMP_FLAGS_TEXTURE_TYPE_MASK;
pub use encoder::BU_COMP_FLAGS_TEXTURE_TYPE_SHIFT;
pub use encoder::BU_COMP_FLAGS_TEXTURE_TYPE_VIDEO_FRAMES;
pub use encoder::BU_COMP_FLAGS_THREADED;
pub use encoder::BU_COMP_FLAGS_USE_OPENCL;
pub use encoder::BU_COMP_FLAGS_VALIDATE_OUTPUT;
pub use encoder::BU_COMP_FLAGS_VERBOSE;
pub use encoder::BU_COMP_FLAGS_XUASTC_LDR_FULL_ARITH;
pub use encoder::BU_COMP_FLAGS_XUASTC_LDR_FULL_ZSTD;
pub use encoder::BU_COMP_FLAGS_XUASTC_LDR_HYBRID;
pub use encoder::BU_COMP_FLAGS_XUASTC_LDR_SYNTAX_MASK;
pub use encoder::BU_COMP_FLAGS_XUASTC_LDR_SYNTAX_SHIFT;
pub use encoder::BU_COMP_FLAGS_Y_FLIP;

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
    #[error("Unsupported textire format: {0:?}")]
    UnsupportedTextureFormat(TextureFormat),
    #[error("Unsupported textire view dimension: {0:?}")]
    UnsupportedTextureViewDimension(TextureViewDimension),
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
            flags_and_quality: (encoder::BU_COMP_FLAGS_THREADED
                | encoder::BU_COMP_FLAGS_SRGB
                | encoder::BU_COMP_FLAGS_KTX2_OUTPUT
                | encoder::BU_COMP_FLAGS_KTX2_UASTC_ZSTD) as u64,
            low_level_uastc_rdo_or_dct_quality: 0.0,
        }
    }

    pub const fn new_with_linear_defaults(basis_tex_format: BasisTextureFormat) -> Self {
        Self {
            basis_tex_format,
            quality_level: 75,
            effort_level: 2,
            flags_and_quality: (encoder::BU_COMP_FLAGS_THREADED
                | encoder::BU_COMP_FLAGS_KTX2_OUTPUT
                | encoder::BU_COMP_FLAGS_KTX2_UASTC_ZSTD) as u64,
            low_level_uastc_rdo_or_dct_quality: 0.0,
        }
    }
}

impl BasisuEncoder {
    pub fn new() -> Self {
        Self {
            params: unsafe { encoder::bu_new_comp_params() },
        }
    }

    pub fn set_image(&mut self, image: &Image) -> Result<(), BasisuEncodeError> {
        assert!(unsafe { encoder::bu_comp_params_clear(self.params) } != 0);
        let Some(data) = image.data.as_ref() else {
            return Err(BasisuEncodeError::DataIsNone);
        };
        if image.texture_descriptor.mip_level_count != 1 {
            return Err(BasisuEncodeError::MipLevelCountNotOne);
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
            TextureFormat::Rgba8Unorm => unsafe {
                for i in 0..image.texture_descriptor.array_layer_count() {
                    if encoder::bu_comp_params_set_image_rgba32(
                        self.params,
                        i,
                        data.as_ptr() as u64,
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
                        data.as_ptr() as u64,
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
