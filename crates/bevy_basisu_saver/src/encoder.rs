use async_lock::OnceCell;
use basisu_c_sys::BasisTextureFormat;
use basisu_c_sys::common;
use basisu_c_sys::encoder as enc_sys;
use bevy::{
    image::Image,
    render::render_resource::{TextureDimension, TextureFormat, TextureViewDimension},
};
use serde::{Deserialize, Serialize};

static BASISU_INITIALIZED: OnceCell<()> = OnceCell::new();

pub async fn basisu_encoder_init() {
    BASISU_INITIALIZED
        .get_or_init(async || {
            basisu_c_sys::instantiate_embedded_basisu_wasm().await;
            unsafe { enc_sys::bu_init() };
        })
        .await;
}

pub fn basisu_encoder_enable_debug_printf(enable: bool) {
    unsafe { enc_sys::bu_enable_debug_printf(enable as u32) };
}

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
    ImageDataIsNone,
    #[error("Mip level count must be 1")]
    MipLevelCountNotOne,
    #[error("Unsupported texture format: {0:?}")]
    UnsupportedTextureFormat(TextureFormat),
    #[error("Unsupported texture dimension: {0:?}")]
    UnsupportedTextureDimension(TextureDimension),
    #[error("Unsupported texture view dimension: {0:?}")]
    UnsupportedTextureViewDimension(TextureViewDimension),
    #[error("`BasisuEncoder::set_image_slice` only accepts image with 1 layer or depth")]
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
    /// Unified Quality level [1, 100]. See [`common::BU_QUALITY_MIN`], [`common::BU_QUALITY_MAX`]. Note the recommended usable unified quality range is [1, 100], but the C API accepts [0, 100]. Use -1 to use older non-unified/direct codec-specific quality level or lambda (low 8-bits of flags_and_quality, or via low_level_uastc_rdo_or_dct_quality).
    pub quality_level: i32,
    /// Unified Encoder effort [0, 10]. See [`common::BU_EFFORT_MIN`], [`common::BU_EFFORT_MAX`]. See `BU_EFFORT_*` presets. Use -1 to use older non-unified/direct codec-specific effort level (low 8-bits of flags_and_quality for some codecs).
    pub effort_level: i32,
    /// Bitwise OR of `BU_COMP_FLAGS_*` constants. Controls output format, mipmaps, color space, etc. Low 8-bits are either the older non-unified quality level, or for some codecs the non-unified effort level.
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
            flags_and_quality: common::BU_COMP_FLAGS_THREADED
                | common::BU_COMP_FLAGS_SRGB
                | common::BU_COMP_FLAGS_KTX2_OUTPUT
                | common::BU_COMP_FLAGS_KTX2_UASTC_ZSTD,
            low_level_uastc_rdo_or_dct_quality: 0.0,
        }
    }

    pub const fn new_with_linear_defaults(basis_tex_format: BasisTextureFormat) -> Self {
        Self {
            basis_tex_format,
            quality_level: 75,
            effort_level: 2,
            flags_and_quality: common::BU_COMP_FLAGS_THREADED
                | common::BU_COMP_FLAGS_KTX2_OUTPUT
                | common::BU_COMP_FLAGS_KTX2_UASTC_ZSTD,
            low_level_uastc_rdo_or_dct_quality: 0.0,
        }
    }

    pub const fn with_tex_type(mut self, tex_type: TextureViewDimension) -> Self {
        self.flags_and_quality = self.flags_and_quality
            & !(common::BU_COMP_FLAGS_TEXTURE_TYPE_MASK
                << common::BU_COMP_FLAGS_TEXTURE_TYPE_SHIFT);

        self.flags_and_quality = self.flags_and_quality
            | match tex_type {
                TextureViewDimension::D2 => common::BU_COMP_FLAGS_TEXTURE_TYPE_2D,
                TextureViewDimension::D2Array => common::BU_COMP_FLAGS_TEXTURE_TYPE_2D_ARRAY,
                TextureViewDimension::Cube | TextureViewDimension::CubeArray => {
                    common::BU_COMP_FLAGS_TEXTURE_TYPE_CUBEMAP_ARRAY
                }
                TextureViewDimension::D1 | TextureViewDimension::D3 => {
                    panic!("Compressing 1D or 3D texture is unsupported")
                }
            };
        self
    }

    /// Bitwise OR the flags (See `BU_COMP_FLAGS_*`) to `self`.
    pub const fn with_flags(mut self, flags: u64) -> Self {
        self.flags_and_quality |= flags;
        self
    }
}

impl BasisuEncoder {
    pub fn new() -> Self {
        if !BASISU_INITIALIZED.is_initialized() {
            panic!("`basisu_encoder_init` must be called before create encoder");
        }
        Self {
            params: unsafe { enc_sys::bu_new_comp_params() },
        }
    }

    pub fn set_image(&mut self, image: &Image) -> Result<(), BasisuEncodeError> {
        self.clear_image();

        let Some(data) = image.data.as_ref() else {
            return Err(BasisuEncodeError::ImageDataIsNone);
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
                let basisu_ptr = enc_sys::bu_alloc(data.len() as u64);
                basisu_c_sys::copy_host_memory_to_basisu(data, basisu_ptr);
                for i in 0..image.texture_descriptor.array_layer_count() {
                    if enc_sys::bu_comp_params_set_image_rgba32(
                        self.params,
                        i,
                        basisu_ptr + (i * image.width() * image.height() * 4) as u64,
                        image.width(),
                        image.height(),
                        image.width() * 4,
                    )
                    .is_err()
                    {
                        enc_sys::bu_free(basisu_ptr);
                        return Err(BasisuEncodeError::BuSetImageFailed);
                    }
                }
                enc_sys::bu_free(basisu_ptr);
            },
            TextureFormat::Rgba32Float => unsafe {
                let basisu_ptr = enc_sys::bu_alloc(data.len() as u64);
                basisu_c_sys::copy_host_memory_to_basisu(data, basisu_ptr);
                for i in 0..image.texture_descriptor.array_layer_count() {
                    if enc_sys::bu_comp_params_set_image_float_rgba(
                        self.params,
                        i,
                        basisu_ptr + (i * image.width() * image.height() * 16) as u64,
                        image.width(),
                        image.height(),
                        image.width() * 16,
                    )
                    .is_err()
                    {
                        enc_sys::bu_free(basisu_ptr);
                        return Err(BasisuEncodeError::BuSetImageFailed);
                    }
                }
                enc_sys::bu_free(basisu_ptr);
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
        assert!(unsafe { enc_sys::bu_comp_params_clear(self.params) }.is_ok());
    }

    pub fn set_image_slice(&mut self, index: u32, image: &Image) -> Result<(), BasisuEncodeError> {
        let Some(data) = image.data.as_ref() else {
            return Err(BasisuEncodeError::ImageDataIsNone);
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
                let basisu_ptr = enc_sys::bu_alloc(data.len() as u64);
                basisu_c_sys::copy_host_memory_to_basisu(data, basisu_ptr);
                if enc_sys::bu_comp_params_set_image_rgba32(
                    self.params,
                    index,
                    basisu_ptr,
                    image.width(),
                    image.height(),
                    image.width() * 4,
                )
                .is_err()
                {
                    enc_sys::bu_free(basisu_ptr);
                    return Err(BasisuEncodeError::BuSetImageFailed);
                }
                enc_sys::bu_free(basisu_ptr);
            },
            TextureFormat::Rgba32Float => unsafe {
                let basisu_ptr = enc_sys::bu_alloc(data.len() as u64);
                basisu_c_sys::copy_host_memory_to_basisu(data, basisu_ptr);
                if enc_sys::bu_comp_params_set_image_float_rgba(
                    self.params,
                    index,
                    basisu_ptr,
                    image.width(),
                    image.height(),
                    image.width() * 16,
                )
                .is_err()
                {
                    enc_sys::bu_free(basisu_ptr);
                    return Err(BasisuEncodeError::BuSetImageFailed);
                }
                enc_sys::bu_free(basisu_ptr);
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
            if enc_sys::bu_compress_texture(
                self.params,
                params.basis_tex_format as u32,
                params.quality_level,
                params.effort_level,
                params.flags_and_quality,
                params.low_level_uastc_rdo_or_dct_quality,
            )
            .is_err()
            {
                return Err(BasisuEncodeError::BuCompressFailed);
            }
            let out_size = enc_sys::bu_comp_params_get_comp_data_size(self.params);
            let out_ptr = enc_sys::bu_comp_params_get_comp_data_ofs(self.params);
            let result = basisu_c_sys::copy_basisu_memory_to_host(out_ptr, out_size);
            Ok(result)
        }
    }
}

impl Drop for BasisuEncoder {
    fn drop(&mut self) {
        assert!(unsafe { enc_sys::bu_delete_comp_params(self.params).is_ok() });
    }
}
