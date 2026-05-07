use crate::common;
use crate::encoder as enc_sys;
use crate::utils::BasisTextureFormat;
use alloc::vec::Vec;
use async_lock::OnceCell;
use wgpu_types::{
    TextureDescriptor, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension,
};

#[derive(Debug, Clone, PartialEq)]
pub struct SourceImage<'a> {
    /// The input data of image pixels.
    pub data: &'a [u8],
    pub texture_descriptor: &'a TextureDescriptor<Option<&'static str>, &'static [TextureFormat]>,
    pub texture_view_descriptor: &'a Option<TextureViewDescriptor<Option<&'static str>>>,
}

impl SourceImage<'_> {
    /// Returns the width of a 2D image.
    #[inline]
    pub fn width(&self) -> u32 {
        self.texture_descriptor.size.width
    }

    /// Returns the height of a 2D image.
    #[inline]
    pub fn height(&self) -> u32 {
        self.texture_descriptor.size.height
    }
}

static BASISU_ENCODER_INITIALIZED: OnceCell<()> = OnceCell::new();

/// Init global data of encoder ([`enc_sys::bu_init`]), and basisu wasm if on web.
pub async fn basisu_encoder_init() {
    BASISU_ENCODER_INITIALIZED
        .get_or_init(async || {
            crate::instantiate_embedded_basisu_wasm().await;
            unsafe { enc_sys::bu_init() };
        })
        .await;
}

/// A wrapper of [`enc_sys::bu_enable_debug_printf`].
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

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

    /// Return [`Self`] with `common::BU_COMP_FLAGS_TEXTURE_TYPE_*` set according to the view dimension.
    ///
    /// Panic if the view dimension is D1 or D3.
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
    /// Create a encoder. Panic if [`basisu_encoder_init`] hasn't been called.
    pub fn new() -> Self {
        if !BASISU_ENCODER_INITIALIZED.is_initialized() {
            panic!("`basisu_encoder_init` must be called before create encoder");
        }
        Self {
            params: unsafe { enc_sys::bu_new_comp_params() },
        }
    }

    /// Set the input image of the encoder and clear other image set before.
    ///
    /// This support setting image that has multiple layers at once to compress cubemap or texture array.
    ///
    /// A error will be returned if the input image doesn't meet:
    /// - Mip level count is 1
    /// - Dimension can't be D1 or D3
    /// - Format is [`TextureFormat::Rgba8Unorm`], [`TextureFormat::Rgba8UnormSrgb`] or [`TextureFormat::Rgba32Float`]
    pub fn set_image(&mut self, image: SourceImage) -> Result<(), BasisuEncodeError> {
        self.clear_image();

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
        let data = image.data;
        match image.texture_descriptor.format {
            TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb => unsafe {
                let basisu_ptr = enc_sys::bu_alloc(data.len() as u64);
                crate::copy_host_memory_to_basisu(data, basisu_ptr);
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
                crate::copy_host_memory_to_basisu(data, basisu_ptr);
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

    /// Clear the input image of encoder that was set.
    pub fn clear_image(&mut self) {
        assert!(unsafe { enc_sys::bu_comp_params_clear(self.params) }.is_ok());
    }

    /// Set a image slice at index. Other image set before is not cleared.
    ///
    /// This is mainly used to compress cubemap or texture array from a list of 2D images.
    /// If you already have a layered image, [`Self::set_image`] can be used instead.
    ///
    /// A error will be returned if the input image doesn't meet:
    /// - Mip level count is 1
    /// - Dimension can't be D1 or D3
    /// - Array layer count is 1
    /// - Format is [`TextureFormat::Rgba8Unorm`], [`TextureFormat::Rgba8UnormSrgb`] or [`TextureFormat::Rgba32Float`]
    pub fn set_image_slice(
        &mut self,
        index: u32,
        image: SourceImage,
    ) -> Result<(), BasisuEncodeError> {
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
        let data = image.data;
        match image.texture_descriptor.format {
            TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb => unsafe {
                let basisu_ptr = enc_sys::bu_alloc(data.len() as u64);
                crate::copy_host_memory_to_basisu(data, basisu_ptr);
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
                crate::copy_host_memory_to_basisu(data, basisu_ptr);
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

    /// Compress the inputted image and return the bytes of ktx2 file result.
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
            let result = crate::copy_basisu_memory_to_host(out_ptr, out_size);
            Ok(result)
        }
    }
}

impl Drop for BasisuEncoder {
    fn drop(&mut self) {
        assert!(unsafe { enc_sys::bu_delete_comp_params(self.params).is_ok() });
    }
}

#[cfg(test)]
mod tests {

    #[test]
    #[should_panic]
    fn encoder_create_before_init() {
        if super::BASISU_ENCODER_INITIALIZED.is_initialized() {
            panic!("Basisu is already initialized, panic to skip this test");
        } else {
            super::BasisuEncoder::new();
        }
    }
}
