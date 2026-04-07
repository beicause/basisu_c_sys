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

pub async fn basisu_encoder_init() {
    BASISU_ENCODER_INITIALIZED
        .get_or_init(async || {
            crate::instantiate_embedded_basisu_wasm().await;
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
        if !BASISU_ENCODER_INITIALIZED.is_initialized() {
            panic!("`basisu_encoder_init` must be called before create encoder");
        }
        Self {
            params: unsafe { enc_sys::bu_new_comp_params() },
        }
    }

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

    pub fn clear_image(&mut self) {
        assert!(unsafe { enc_sys::bu_comp_params_clear(self.params) }.is_ok());
    }

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
    use std::path::Path;

    use crate::{
        common::{
            BU_COMP_FLAGS_DEBUG_OUTPUT, BU_COMP_FLAGS_GEN_MIPS_CLAMP, BU_COMP_FLAGS_VALIDATE_OUTPUT,
        },
        extra::{
            BasisuEncoder, BasisuEncoderParams, SourceImage, basisu_encoder_enable_debug_printf,
            basisu_encoder_init,
        },
        utils::BasisTextureFormat,
    };
    use bevy_image::{CompressedImageFormats, Image};
    use wgpu_types::TextureViewDimension;

    const SKYBOX_PATHS: &[&str] = &[
        "../../original_assets/skybox/right.jpg",
        "../../original_assets/skybox/left.jpg",
        "../../original_assets/skybox/top.jpg",
        "../../original_assets/skybox/bottom.jpg",
        "../../original_assets/skybox/front.jpg",
        "../../original_assets/skybox/back.jpg",
    ];

    impl<'a> From<&'a Image> for SourceImage<'a> {
        fn from(value: &'a Image) -> Self {
            Self {
                data: value.data.as_deref().unwrap_or(&[]),
                texture_descriptor: &value.texture_descriptor,
                texture_view_descriptor: &value.texture_view_descriptor,
            }
        }
    }

    #[test]
    fn encode_cubemap_xuastc_ldr_4x4_by_slice() {
        block_on(basisu_encoder_init());
        basisu_encoder_enable_debug_printf(true);

        let dir = std::env!("CARGO_MANIFEST_DIR");
        let mut encoder = BasisuEncoder::new();
        for (i, path) in SKYBOX_PATHS.iter().enumerate() {
            let image = Image::from_buffer(
                &std::fs::read(Path::new(dir).join(path)).unwrap(),
                bevy_image::ImageType::Extension(
                    Path::new(path).extension().unwrap().to_str().unwrap(),
                ),
                CompressedImageFormats::empty(),
                true,
                bevy_image::ImageSampler::Default,
                Default::default(),
            )
            .unwrap();
            encoder.set_image_slice(i as u32, (&image).into()).unwrap();
        }
        let params = BasisuEncoderParams::new_with_srgb_defaults(BasisTextureFormat::XuastcLdr4x4)
            .with_tex_type(TextureViewDimension::Cube);

        let res = encoder
            .compress(params.with_flags(BU_COMP_FLAGS_DEBUG_OUTPUT | BU_COMP_FLAGS_VALIDATE_OUTPUT))
            .unwrap();
        // The test failed on macos, disable it for now.
        #[cfg(not(target_os = "macos"))]
        insta::assert_binary_snapshot!("skybox_astc_ldr_8x8.basisu.ktx2", res);
    }

    #[test]
    fn encode_cubemap_astc_ldr_8x8_mips_by_image() {
        block_on(basisu_encoder_init());
        basisu_encoder_enable_debug_printf(true);

        let dir = std::env!("CARGO_MANIFEST_DIR");
        let mut images = Vec::new();
        let mut encoder = BasisuEncoder::new();
        for path in SKYBOX_PATHS {
            let image = Image::from_buffer(
                &std::fs::read(Path::new(dir).join(path)).unwrap(),
                bevy_image::ImageType::Extension(
                    Path::new(path).extension().unwrap().to_str().unwrap(),
                ),
                CompressedImageFormats::empty(),
                true,
                bevy_image::ImageSampler::Default,
                Default::default(),
            )
            .unwrap();
            images.push(image);
        }
        let cube_image = Image {
            data: Some(
                images
                    .iter_mut()
                    .flat_map(|img| img.data.take().unwrap())
                    .collect(),
            ),
            texture_descriptor: wgpu_types::TextureDescriptor {
                size: wgpu_types::Extent3d {
                    width: images[0].width(),
                    height: images[0].height(),
                    depth_or_array_layers: images.len() as u32,
                },
                ..images[0].texture_descriptor
            },
            texture_view_descriptor: Some(wgpu_types::TextureViewDescriptor {
                dimension: Some(TextureViewDimension::Cube),
                ..Default::default()
            }),
            ..Default::default()
        };
        encoder.set_image((&cube_image).into()).unwrap();
        let res = encoder
            .compress(
                BasisuEncoderParams::new_with_srgb_defaults(BasisTextureFormat::AstcLdr8x8)
                    .with_tex_type(TextureViewDimension::Cube)
                    .with_flags(
                        BU_COMP_FLAGS_DEBUG_OUTPUT
                            | BU_COMP_FLAGS_VALIDATE_OUTPUT
                            | BU_COMP_FLAGS_GEN_MIPS_CLAMP,
                    ),
            )
            .unwrap();
        // The test failed on macos, disable it for now.
        #[cfg(not(target_os = "macos"))]
        insta::assert_binary_snapshot!("skybox_astc_ldr_8x8_mips.basisu.ktx2", res);
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
