#[expect(
    non_upper_case_globals,
    non_camel_case_types,
    reason = "Generated code is OK to have non upper case globals or non camel case enums"
)]
#[cfg_attr(
    all(
        target_arch = "wasm32",
        target_vendor = "unknown",
        target_os = "unknown",
    ),
    expect(
        unused,
        reason = "On wasm32 we use js bindings thus all native functions are unused"
    )
)]
mod transcoding {
    include!(concat!(env!("OUT_DIR"), "/transcoding.rs"));
}

pub use transcoding::{
    BasisTextureFormat, ChannelType, SupportedTextureCompressionMethods, TranscodedTextureFormat,
};

use async_lock::OnceCell;

#[cfg(not(all(
    target_arch = "wasm32",
    target_vendor = "unknown",
    target_os = "unknown",
)))]
mod native;
#[cfg(not(all(
    target_arch = "wasm32",
    target_vendor = "unknown",
    target_os = "unknown",
)))]
use native::*;

#[cfg(all(
    target_arch = "wasm32",
    target_vendor = "unknown",
    target_os = "unknown",
))]
mod web;
#[cfg(all(
    target_arch = "wasm32",
    target_vendor = "unknown",
    target_os = "unknown",
))]
use web::*;

static BASISU_INITIALIZED: OnceCell<()> = OnceCell::new();

/// Init basisu global data. Must be called before transcoding.
pub async fn basisu_init() {
    BASISU_INITIALIZED
        .get_or_init(async || {
            #[cfg(all(
                target_arch = "wasm32",
                target_vendor = "unknown",
                target_os = "unknown",
            ))]
            basisu_sys_init_vendor().await;
            unsafe {
                basisu_transcoder_init();
            }
        })
        .await;
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
    pub preferred_target: TranscodedTextureFormat,
}

pub struct BasisuTranscoder {
    inner: *mut transcoding::Transcoder,
    #[cfg(not(all(
        target_arch = "wasm32",
        target_vendor = "unknown",
        target_os = "unknown",
    )))]
    data_handle: Option<Vec<u8>>,
    #[cfg(all(
        target_arch = "wasm32",
        target_vendor = "unknown",
        target_os = "unknown",
    ))]
    data_handle: usize,
}

impl Default for BasisuTranscoder {
    fn default() -> Self {
        Self::new()
    }
}
impl BasisuTranscoder {
    /// Create a transcoder. Panic if [`basisu_init`] has not been called.
    pub fn new() -> Self {
        if !BASISU_INITIALIZED.is_initialized() {
            panic!("`basisu_init` must be called before transcoding.");
        }
        Self {
            inner: unsafe { ktx2_transcoder_new() },
            #[cfg(not(all(
                target_arch = "wasm32",
                target_vendor = "unknown",
                target_os = "unknown",
            )))]
            data_handle: None,
            #[cfg(all(
                target_arch = "wasm32",
                target_vendor = "unknown",
                target_os = "unknown",
            ))]
            data_handle: 0,
        }
    }

    /// Get info about the basisu ktx2 data and prepare to transcode.
    /// Return None if the data is invalid.
    pub fn start(
        &mut self,
        data: Vec<u8>,
        supported_compressed_formats: SupportedTextureCompressionMethods,
        channel_type_hint: ChannelType,
    ) -> Option<TranscodeInfo> {
        let transcoder = self.inner;
        unsafe {
            #[cfg(not(all(
                target_arch = "wasm32",
                target_vendor = "unknown",
                target_os = "unknown",
            )))]
            {
                if !ktx2_transcoder_transcode_image_get_info(
                    transcoder,
                    &data,
                    supported_compressed_formats,
                    channel_type_hint,
                ) {
                    self.data_handle = None;
                    return None;
                }
                self.data_handle = Some(data);
            };
            #[cfg(all(
                target_arch = "wasm32",
                target_vendor = "unknown",
                target_os = "unknown",
            ))]
            {
                let (success, wasm_data_handle) = ktx2_transcoder_transcode_image_get_info(
                    transcoder,
                    &data,
                    supported_compressed_formats,
                    channel_type_hint,
                );
                if self.data_handle != 0 {
                    ktx2_transcoder_transcode_image_free_wasm_data(self.data_handle);
                }
                self.data_handle = wasm_data_handle;
                if !success {
                    if self.data_handle != 0 {
                        ktx2_transcoder_transcode_image_free_wasm_data(self.data_handle);
                        self.data_handle = 0;
                    }
                    return None;
                }
            }
            Some(TranscodeInfo {
                width: ktx2_transcoder_get_r_width(transcoder),
                height: ktx2_transcoder_get_r_height(transcoder),
                levels: ktx2_transcoder_get_r_levels(transcoder),
                layers: ktx2_transcoder_get_r_layers(transcoder),
                faces: ktx2_transcoder_get_r_faces(transcoder),
                is_srgb: ktx2_transcoder_get_r_is_srgb(transcoder),
                preferred_target: ktx2_transcoder_get_r_preferred_target(transcoder),
                basis_format: ktx2_transcoder_get_r_basis_format(transcoder),
            })
        }
    }

    /// Transcode the prepared data and return the result. Return None if transcoding failed.
    /// Panic if [`BasisuTranscoder::start`] has not been called or successfully returned `Some`.
    pub fn output(&self, transcode_target: TranscodedTextureFormat) -> Option<Vec<u8>> {
        #[cfg(not(all(
            target_arch = "wasm32",
            target_vendor = "unknown",
            target_os = "unknown",
        )))]
        {
            if self.data_handle.is_none() {
                panic!(
                    "BasisuTranscoder::start must be called and succeed before calling BasisuTranscoder::output"
                );
            }
            basisu_transcode_directly_write(self, transcode_target)
        }
        #[cfg(all(
            target_arch = "wasm32",
            target_vendor = "unknown",
            target_os = "unknown",
        ))]
        {
            if self.data_handle == 0 {
                panic!(
                    "BasisuTranscoder::start must be called and succeed before calling BasisuTranscoder::output"
                );
            }
            basisu_transcode_alloc_and_fetch_dst(self, transcode_target)
        }
    }
}

impl Drop for BasisuTranscoder {
    fn drop(&mut self) {
        unsafe {
            #[cfg(all(
                target_arch = "wasm32",
                target_vendor = "unknown",
                target_os = "unknown",
            ))]
            if self.data_handle != 0 {
                ktx2_transcoder_transcode_image_free_wasm_data(self.data_handle);
                self.data_handle = 0;
            }
            ktx2_transcoder_delete(self.inner)
        };
    }
}

#[cfg(any(
    not(all(
        target_arch = "wasm32",
        target_vendor = "unknown",
        target_os = "unknown",
    )),
    test
))]
fn basisu_transcode_directly_write(
    transcoder: &BasisuTranscoder,
    target_format: TranscodedTextureFormat,
) -> Option<Vec<u8>> {
    let target_bytes = unsafe {
        if !ktx2_transcoder_transcode_image_compute_target_bytes(transcoder.inner, target_format) {
            return None;
        }
        ktx2_transcoder_get_r_dst_buf_len(transcoder.inner)
    };
    let mut buffer = vec![0u8; target_bytes as usize];
    let success = unsafe {
        ktx2_transcoder_transcode_image_write(transcoder.inner, target_format, buffer.as_mut_ptr())
    };
    if success { Some(buffer) } else { None }
}

#[cfg(any(
    all(
        target_arch = "wasm32",
        target_vendor = "unknown",
        target_os = "unknown",
    ),
    test
))]
fn basisu_transcode_alloc_and_fetch_dst(
    transcoder: &BasisuTranscoder,
    target_format: TranscodedTextureFormat,
) -> Option<Vec<u8>> {
    unsafe {
        if !ktx2_transcoder_transcode_image_compute_target_bytes(transcoder.inner, target_format) {
            return None;
        }
        let result = {
            let success =
                ktx2_transcoder_transcode_image_alloc_and_write(transcoder.inner, target_format);
            if !success {
                return None;
            }
            ktx2_transcoder_get_r_dst_buf(transcoder.inner)
        };
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        BASISU_INITIALIZED, BasisTextureFormat, BasisuTranscoder, ChannelType,
        SupportedTextureCompressionMethods, TranscodeInfo,
    };

    const XUASTC_FILE: &[u8] =
        include_bytes!("../../../assets/wikipedia_xuastc_ldr_8x8_mips.basisu.ktx2");

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
        block_on(crate::basisu_init());
        let mut transcoder = BasisuTranscoder::new();
        let info = transcoder.start(
            vec![],
            SupportedTextureCompressionMethods::NONE,
            ChannelType::CHANNEL_UNDEFINED,
        );
        assert_eq!(info, None);
        let info = transcoder.start(
            vec![1, 2, 1],
            SupportedTextureCompressionMethods::BC
                | SupportedTextureCompressionMethods::ASTC_LDR
                | SupportedTextureCompressionMethods::ASTC_HDR
                | SupportedTextureCompressionMethods::ETC2,
            ChannelType::CHANNEL_UNDEFINED,
        );
        assert_eq!(info, None);
    }

    #[test]
    #[should_panic]
    fn transcode_before_start_panic() {
        block_on(crate::basisu_init());
        let transcoder = BasisuTranscoder::new();
        transcoder.output(crate::TranscodedTextureFormat::cTFRGBA32);
    }

    #[test]
    #[should_panic]
    fn transcode_invalid_data_output_panic() {
        block_on(crate::basisu_init());
        let mut transcoder = BasisuTranscoder::new();
        let _info = transcoder.start(
            vec![1, 2, 1],
            SupportedTextureCompressionMethods::NONE,
            ChannelType::CHANNEL_UNDEFINED,
        );
        transcoder.output(crate::TranscodedTextureFormat::cTFRGBA32);
    }

    #[test]
    fn transcode_xuastc_sample_two_path() {
        for _ in 0..10 {
            block_on(crate::basisu_init());
            let mut transcoder = BasisuTranscoder::new();
            let info = transcoder
                .start(
                    XUASTC_FILE.to_vec(),
                    SupportedTextureCompressionMethods::BC
                        | SupportedTextureCompressionMethods::ASTC_LDR
                        | SupportedTextureCompressionMethods::ASTC_HDR
                        | SupportedTextureCompressionMethods::ETC2,
                    ChannelType::CHANNEL_UNDEFINED,
                )
                .unwrap();
            assert_eq!(
                info,
                TranscodeInfo {
                    width: 1848,
                    height: 888,
                    levels: 11,
                    layers: 0,
                    faces: 1,
                    is_srgb: true,
                    basis_format: BasisTextureFormat::cXUASTC_LDR_8x8,
                    preferred_target: crate::TranscodedTextureFormat::cTFASTC_LDR_8x8_RGBA
                }
            );
            assert_eq!(
                crate::basisu_transcode_alloc_and_fetch_dst(&transcoder, info.preferred_target),
                crate::basisu_transcode_directly_write(&transcoder, info.preferred_target)
            );
        }
    }

    // Use macro to make `assert_debug_snapshot` produce correct file name.
    // See https://github.com/mitsuhiko/insta/issues/357
    macro_rules! snapshot_test {
        ($prefix: expr, $supported_format: expr $(,)?) => {
            let mut path = std::path::PathBuf::new();
            path.push(std::env!("CARGO_MANIFEST_DIR"));
            path.push("../../assets");
            block_on(crate::basisu_init());
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
                    .start(data, $supported_format, ChannelType::CHANNEL_UNDEFINED)
                    .unwrap();
                insta::assert_binary_snapshot!(
                    &($prefix.to_string() + &file_name.replace(".basisu.ktx2", ".bin")),
                    transcoder.output(info.preferred_target).unwrap()
                );
                results.push((file_name, info));
            }
            results.sort_unstable_by(|a, b| a.0.cmp(&b.0));
            insta::assert_debug_snapshot!(results);
        };
    }

    #[test]
    fn transcode_assets_bcn() {
        snapshot_test!("bcn_", SupportedTextureCompressionMethods::BC);
    }

    #[test]
    fn transcode_assets_astc() {
        snapshot_test!(
            "astc_",
            SupportedTextureCompressionMethods::ASTC_LDR
                | SupportedTextureCompressionMethods::ASTC_HDR,
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
