use crate::common;
use crate::encoder as enc_sys;
use crate::extra::BuHeap;
use crate::utils::BasisTextureFormat;
use alloc::vec::Vec;
use async_lock::OnceCell;
use bytemuck::NoUninit;
use bytemuck::PodCastError;
use wgpu_types::{Extent3d, TextureViewDimension};

#[derive(Debug, Clone)]
pub enum SourceImageData<'a> {
    /// A 32bpp RGBA data slice (4 bytes per pixel)
    Rgba8(&'a [u8]),
    /// A 64bpp float RGBA data slice (16 bytes per pixel)
    Rgba32Float(&'a [f32]),
}

impl<'a> SourceImageData<'a> {
    /// Cast the slice to [`SourceImageData::Rgba32Float`]. Return an error if the casting failed.
    pub fn rgba32float<T: NoUninit>(data: &'a [T]) -> Result<SourceImageData<'a>, PodCastError> {
        bytemuck::try_cast_slice(data).map(Self::Rgba32Float)
    }

    /// Cast the slice to [`SourceImageData::Rgba8`]. Return an error if the casting failed.
    pub fn rgba8<T: NoUninit>(data: &'a [T]) -> Result<SourceImageData<'a>, PodCastError> {
        bytemuck::try_cast_slice(data).map(Self::Rgba32Float)
    }
}

#[derive(Debug, Clone)]
pub struct SourceImage<'a> {
    /// The input data of image pixels.
    pub data: SourceImageData<'a>,
    /// The size of image.
    pub size: Extent3d,
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

/// Encoder that used to compress [`SourceImage`] to basis universal ktx2 file.
pub struct BasisuEncoder {
    params: u64,
}

impl Default for BasisuEncoder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
#[non_exhaustive]
pub enum BasisuEncodeError {
    #[error("`BasisuEncoder::set_image_slice` only accepts image with 1 layer")]
    SetImageSliceOnlyAcceptsOneLayer,
    #[error("Image data is empty")]
    EmptyImageData,
    #[error("Image {image_size:?} Expects data length {expected_len}, got {data_len}")]
    ImageUnmatchedDataAndSize {
        image_size: Extent3d,
        expected_len: usize,
        data_len: usize,
    },
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
    /// All the layers of the image will be set. To compress it as a cubemap or texture array,
    /// you will need to add flag to [`BasisuEncoderParams::flags_and_quality`] by calling [`BasisuEncoderParams::with_tex_type`].
    ///
    /// If you already have continuous data for the cubemap or texture array, this should be faster than [`Self::set_image_slice`] .
    pub fn set_image(&mut self, image: SourceImage) -> Result<(), BasisuEncodeError> {
        self.clear_image();

        match image.data {
            SourceImageData::Rgba8(data) => unsafe {
                let pixel_bytes = 4;
                let expected_len = (image.size.width
                    * image.size.height
                    * image.size.depth_or_array_layers
                    * pixel_bytes) as usize;
                if data.len() != expected_len {
                    return Err(BasisuEncodeError::ImageUnmatchedDataAndSize {
                        image_size: image.size,
                        expected_len,
                        data_len: data.len(),
                    });
                }

                let Some(bu_heap) = BuHeap::new(data) else {
                    return Err(BasisuEncodeError::EmptyImageData);
                };
                let ptr = u64::from(bu_heap.ptr());
                for i in 0..image.size.depth_or_array_layers {
                    if enc_sys::bu_comp_params_set_image_rgba32(
                        self.params,
                        i,
                        ptr + (i * image.size.width * image.size.height * pixel_bytes) as u64,
                        image.size.width,
                        image.size.height,
                        image.size.width * pixel_bytes,
                    )
                    .is_err()
                    {
                        return Err(BasisuEncodeError::BuSetImageFailed);
                    }
                }
            },
            SourceImageData::Rgba32Float(data) => unsafe {
                let pixel_bytes = 16;
                let expected_len = (image.size.width
                    * image.size.height
                    * image.size.depth_or_array_layers
                    * pixel_bytes) as usize;
                if data.len() != expected_len {
                    return Err(BasisuEncodeError::ImageUnmatchedDataAndSize {
                        image_size: image.size,
                        expected_len,
                        data_len: data.len(),
                    });
                }

                let Some(bu_heap) = BuHeap::new(data) else {
                    return Err(BasisuEncodeError::EmptyImageData);
                };
                let ptr = u64::from(bu_heap.ptr());
                for i in 0..image.size.depth_or_array_layers {
                    if enc_sys::bu_comp_params_set_image_float_rgba(
                        self.params,
                        i,
                        ptr + (i * image.size.width * image.size.height * pixel_bytes) as u64,
                        image.size.width,
                        image.size.height,
                        image.size.width * pixel_bytes,
                    )
                    .is_err()
                    {
                        return Err(BasisuEncodeError::BuSetImageFailed);
                    }
                }
            },
        }
        Ok(())
    }

    /// Clear the input image of encoder that was set.
    pub fn clear_image(&mut self) {
        assert!(unsafe { enc_sys::bu_comp_params_clear(self.params) }.is_ok());
    }

    /// Set a image slice at index. Other image set before is not cleared.
    ///
    /// After set all the layers of the image, to compress it as a cubemap or texture array,
    /// you will need to add flag to [`BasisuEncoderParams::flags_and_quality`] by calling [`BasisuEncoderParams::with_tex_type`].
    ///
    /// The input image array layer count must be 1, otherwise an error will be returned.
    ///
    /// If you already have continuous data for the cubemap or texture array, [`Self::set_image`] should faster.
    pub fn set_image_slice(
        &mut self,
        index: u32,
        image: SourceImage,
    ) -> Result<(), BasisuEncodeError> {
        if image.size.depth_or_array_layers != 1 {
            return Err(BasisuEncodeError::SetImageSliceOnlyAcceptsOneLayer);
        }

        match image.data {
            SourceImageData::Rgba8(data) => unsafe {
                let pixel_bytes = 4;
                let expected_len = (image.size.width
                    * image.size.height
                    * image.size.depth_or_array_layers
                    * pixel_bytes) as usize;
                if data.len() != expected_len {
                    return Err(BasisuEncodeError::ImageUnmatchedDataAndSize {
                        image_size: image.size,
                        expected_len,
                        data_len: data.len(),
                    });
                }

                let Some(bu_heap) = BuHeap::new(data) else {
                    return Err(BasisuEncodeError::EmptyImageData);
                };
                let ptr = u64::from(bu_heap.ptr());
                if enc_sys::bu_comp_params_set_image_rgba32(
                    self.params,
                    index,
                    ptr,
                    image.size.width,
                    image.size.height,
                    image.size.width * pixel_bytes,
                )
                .is_err()
                {
                    return Err(BasisuEncodeError::BuSetImageFailed);
                }
            },
            SourceImageData::Rgba32Float(data) => unsafe {
                let pixel_bytes = 16;
                let expected_len = (image.size.width
                    * image.size.height
                    * image.size.depth_or_array_layers
                    * pixel_bytes) as usize;
                if data.len() != expected_len {
                    return Err(BasisuEncodeError::ImageUnmatchedDataAndSize {
                        image_size: image.size,
                        expected_len,
                        data_len: data.len(),
                    });
                }

                let Some(bu_heap) = BuHeap::new(data) else {
                    return Err(BasisuEncodeError::EmptyImageData);
                };
                let ptr = u64::from(bu_heap.ptr());
                if enc_sys::bu_comp_params_set_image_float_rgba(
                    self.params,
                    index,
                    ptr,
                    image.size.width,
                    image.size.height,
                    image.size.width * pixel_bytes,
                )
                .is_err()
                {
                    return Err(BasisuEncodeError::BuSetImageFailed);
                }
            },
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
    use wgpu_types::Extent3d;

    use crate::extra::{
        BasisuEncodeError, BasisuEncoder, SourceImage, SourceImageData, basisu_encoder_init,
        encoder::BASISU_ENCODER_INITIALIZED,
    };

    #[test]
    #[should_panic]
    fn encoder_create_before_init() {
        if BASISU_ENCODER_INITIALIZED.is_initialized() {
            panic!("Basisu is already initialized, panic to skip this test");
        } else {
            BasisuEncoder::new();
        }
    }

    #[test]
    fn invalid_image_data() {
        block_on(basisu_encoder_init());
        let mut encoder = BasisuEncoder::new();
        assert_eq!(
            encoder.set_image(SourceImage {
                data: SourceImageData::Rgba8(&[]),
                size: Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1
                },
            }),
            Err(BasisuEncodeError::ImageUnmatchedDataAndSize {
                image_size: Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1
                },
                expected_len: 4,
                data_len: 0
            })
        );
    }

    /// Blocks on the supplied `future`.
    /// This implementation will busy-wait until it is completed.
    /// Consider enabling the `async-io` or `futures-lite` features.
    pub fn block_on<T>(future: impl Future<Output = T>) -> T {
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
