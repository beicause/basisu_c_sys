#![cfg_attr(
    not(all(
        target_arch = "wasm32",
        target_vendor = "unknown",
        target_os = "unknown",
    )),
    no_std
)]
extern crate alloc;

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
#[cfg_attr(
    not(test),
    expect(
        unused,
        reason = "On native we don't alloc dst buffer on cpp side thus \
	`c_ktx2_transcoder_transcode_image_alloc_dst` and `c_ktx2_transcoder_get_r_dst_buf` are unused"
    )
)]
mod transcoding {
    include!(concat!(env!("OUT_DIR"), "/transcoding.rs"));
}

use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
pub use transcoding::{
    BasisTextureFormat, ChannelType, SupportedTextureCompressionMethods, TranscodedTextureFormat,
};

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

static BASISU_INITIALIZED: AtomicUsize = AtomicUsize::new(0);

/// Init basisu global data. Must be called before transcoding.
pub async fn basisu_init() {
    if BASISU_INITIALIZED.load(Ordering::Acquire) != 0 {
        return;
    }

    #[cfg(all(
        target_arch = "wasm32",
        target_vendor = "unknown",
        target_os = "unknown",
    ))]
    basisu_sys_init_vendor().await;
    unsafe {
        basisu_transcoder_init();
    }
    BASISU_INITIALIZED.store(1, Ordering::Release);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranscodeResult {
    pub width: u32,
    pub height: u32,
    pub levels: u32,
    pub layers: u32,
    pub faces: u32,
    pub is_srgb: bool,
    pub basis_format: BasisTextureFormat,
    pub target_format: TranscodedTextureFormat,
    pub data: Vec<u8>,
}

/// Transcode the basisu ktx2 data.
/// Panic if [`basisu_init`] has not been called.
pub fn basisu_transcode(
    data: Vec<u8>,
    supported_compressed_formats: SupportedTextureCompressionMethods,
    channel_type_hint: ChannelType,
    force_transcode_target: TranscodedTextureFormat,
) -> Option<TranscodeResult> {
    #[cfg(not(all(
        target_arch = "wasm32",
        target_vendor = "unknown",
        target_os = "unknown",
    )))]
    return basisu_transcode_direct_dst(
        data,
        supported_compressed_formats,
        channel_type_hint,
        force_transcode_target,
    );

    #[cfg(all(
        target_arch = "wasm32",
        target_vendor = "unknown",
        target_os = "unknown",
    ))]
    return basisu_transcode_alloc_and_fetch_dst(
        data,
        supported_compressed_formats,
        channel_type_hint,
        force_transcode_target,
    );
}

#[cfg(not(all(
    target_arch = "wasm32",
    target_vendor = "unknown",
    target_os = "unknown",
)))]
pub fn basisu_transcode_direct_dst(
    data: Vec<u8>,
    supported_compressed_formats: SupportedTextureCompressionMethods,
    channel_type_hint: ChannelType,
    force_transcode_target: TranscodedTextureFormat,
) -> Option<TranscodeResult> {
    if BASISU_INITIALIZED.load(Ordering::Acquire) == 0 {
        panic!("`basisu_init` must be called before transcoding.");
    }

    unsafe {
        let transcoder = ktx2_transcoder_new();
        let result = {
            let success = ktx2_transcoder_transcode_image_get_info(
                transcoder,
                data.as_ptr(),
                u32::try_from(data.len()).unwrap(),
                supported_compressed_formats,
                channel_type_hint,
                force_transcode_target,
            );
            if !success {
                ktx2_transcoder_delete(transcoder);
                return None;
            }
            let mut buffer =
                alloc::vec![0u8; ktx2_transcoder_get_r_dst_buf_len(transcoder) as usize];
            ktx2_transcoder_transcode_image_write_buffer(transcoder, buffer.as_mut_ptr());
            buffer
        };

        let res = Some(TranscodeResult {
            data: result,
            width: ktx2_transcoder_get_r_width(transcoder),
            height: ktx2_transcoder_get_r_height(transcoder),
            levels: ktx2_transcoder_get_r_levels(transcoder),
            layers: ktx2_transcoder_get_r_layers(transcoder),
            faces: ktx2_transcoder_get_r_faces(transcoder),
            is_srgb: ktx2_transcoder_get_r_is_srgb(transcoder),
            target_format: ktx2_transcoder_get_r_target_format(transcoder),
            basis_format: ktx2_transcoder_get_r_basis_format(transcoder),
        });
        ktx2_transcoder_delete(transcoder);
        res
    }
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
    data: Vec<u8>,
    supported_compressed_formats: SupportedTextureCompressionMethods,
    channel_type_hint: ChannelType,
    force_transcode_target: TranscodedTextureFormat,
) -> Option<TranscodeResult> {
    if BASISU_INITIALIZED.load(Ordering::Acquire) == 0 {
        panic!("`basisu_init` must be called before transcoding.");
    }
    unsafe {
        let transcoder = ktx2_transcoder_new();
        let result = {
            let success = ktx2_transcoder_transcode_image_alloc_dst(
                transcoder,
                data,
                supported_compressed_formats,
                channel_type_hint,
                force_transcode_target,
            );
            if !success {
                ktx2_transcoder_delete(transcoder);
                return None;
            }
            ktx2_transcoder_get_r_dst_buf(transcoder)
        };
        let res = Some(TranscodeResult {
            data: result,
            width: ktx2_transcoder_get_r_width(transcoder),
            height: ktx2_transcoder_get_r_height(transcoder),
            levels: ktx2_transcoder_get_r_levels(transcoder),
            layers: ktx2_transcoder_get_r_layers(transcoder),
            faces: ktx2_transcoder_get_r_faces(transcoder),
            is_srgb: ktx2_transcoder_get_r_is_srgb(transcoder),
            target_format: ktx2_transcoder_get_r_target_format(transcoder),
            basis_format: ktx2_transcoder_get_r_basis_format(transcoder),
        });
        ktx2_transcoder_delete(transcoder);
        res
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use crate::{
        BasisTextureFormat, ChannelType, SupportedTextureCompressionMethods, TranscodeResult,
        TranscodedTextureFormat,
    };
    use alloc::vec;
    use alloc::{string::ToString, vec::Vec};

    #[test]
    #[should_panic]
    fn transcode_before_init() {
        crate::basisu_transcode(
            vec![],
            SupportedTextureCompressionMethods::NONE,
            ChannelType::CHANNEL_UNDEFINED,
            TranscodedTextureFormat::cTFTotalTextureFormats,
        );
    }

    #[test]
    #[cfg(not(all(
        target_arch = "wasm32",
        target_vendor = "unknown",
        target_os = "unknown",
    )))]
    fn transcode_wasm_and_native_eq() {
        block_on(crate::basisu_init());
        assert_eq!(
            crate::basisu_transcode_alloc_and_fetch_dst(
                vec![],
                SupportedTextureCompressionMethods::NONE,
                ChannelType::CHANNEL_UNDEFINED,
                TranscodedTextureFormat::cTFTotalTextureFormats,
            ),
            crate::basisu_transcode_direct_dst(
                vec![],
                SupportedTextureCompressionMethods::NONE,
                ChannelType::CHANNEL_UNDEFINED,
                TranscodedTextureFormat::cTFTotalTextureFormats,
            )
        );
        assert_eq!(
            crate::basisu_transcode_alloc_and_fetch_dst(
                vec![1, 2, 1],
                SupportedTextureCompressionMethods::BC
                    | SupportedTextureCompressionMethods::ASTC_LDR
                    | SupportedTextureCompressionMethods::ASTC_HDR
                    | SupportedTextureCompressionMethods::ETC2,
                ChannelType::CHANNEL_UNDEFINED,
                TranscodedTextureFormat::cTFTotalTextureFormats,
            ),
            crate::basisu_transcode_direct_dst(
                vec![1, 2, 1],
                SupportedTextureCompressionMethods::BC
                    | SupportedTextureCompressionMethods::ASTC_LDR
                    | SupportedTextureCompressionMethods::ASTC_HDR
                    | SupportedTextureCompressionMethods::ETC2,
                ChannelType::CHANNEL_UNDEFINED,
                TranscodedTextureFormat::cTFTotalTextureFormats,
            )
        );
    }

    #[test]
    fn transcode_simple_data() {
        block_on(crate::basisu_init());
        let res = crate::basisu_transcode(
            vec![],
            SupportedTextureCompressionMethods::NONE,
            ChannelType::CHANNEL_UNDEFINED,
            TranscodedTextureFormat::cTFTotalTextureFormats,
        );
        assert_eq!(
            res,
            Some(TranscodeResult {
                data: vec![],
                width: 0,
                height: 0,
                levels: 0,
                layers: 0,
                faces: 0,
                is_srgb: false,
                basis_format: BasisTextureFormat::cETC1S,
                target_format: TranscodedTextureFormat::cTFRGBA32
            })
        );
        let res = crate::basisu_transcode(
            vec![1, 2, 1],
            SupportedTextureCompressionMethods::BC
                | SupportedTextureCompressionMethods::ASTC_LDR
                | SupportedTextureCompressionMethods::ASTC_HDR
                | SupportedTextureCompressionMethods::ETC2,
            ChannelType::CHANNEL_UNDEFINED,
            TranscodedTextureFormat::cTFTotalTextureFormats,
        );
        assert_eq!(
            res,
            Some(TranscodeResult {
                data: vec![],
                width: 0,
                height: 0,
                levels: 0,
                layers: 0,
                faces: 0,
                is_srgb: false,
                basis_format: BasisTextureFormat::cETC1S,
                target_format: TranscodedTextureFormat::cTFBC7_RGBA
            })
        );
    }

    #[test]
    fn transcode_assets_bcn() {
        let mut path = std::path::PathBuf::new();
        path.push(std::env!("CARGO_MANIFEST_DIR"));
        path.push("../../assets");
        block_on(crate::basisu_init());
        let mut results = Vec::new();
        for file in std::fs::read_dir(path).unwrap() {
            let file = file.unwrap();
            let file_name = file.file_name().into_string().unwrap();
            if !file_name.ends_with(".basisu.ktx2") {
                continue;
            }
            let data = std::fs::read(file.path()).unwrap();
            let result = crate::basisu_transcode(
                data,
                SupportedTextureCompressionMethods::BC,
                ChannelType::CHANNEL_UNDEFINED,
                TranscodedTextureFormat::cTFTotalTextureFormats,
            )
            .unwrap();
            insta::assert_binary_snapshot!(&("bcn_".to_string() + &file_name), result.data);
            results.push(TranscodeResult {
                data: Vec::new(),
                ..result
            });
        }
        insta::assert_debug_snapshot!(results);
    }

    #[test]
    fn transcode_assets_astc() {
        let mut path = std::path::PathBuf::new();
        path.push(std::env!("CARGO_MANIFEST_DIR"));
        path.push("../../assets");
        block_on(crate::basisu_init());
        let mut results = Vec::new();
        for file in std::fs::read_dir(path).unwrap() {
            let file = file.unwrap();
            let file_name = file.file_name().into_string().unwrap();
            if !file_name.ends_with(".basisu.ktx2") {
                continue;
            }
            let data = std::fs::read(file.path()).unwrap();
            let result = crate::basisu_transcode(
                data,
                SupportedTextureCompressionMethods::ASTC_LDR
                    | SupportedTextureCompressionMethods::ASTC_HDR,
                ChannelType::CHANNEL_UNDEFINED,
                TranscodedTextureFormat::cTFTotalTextureFormats,
            )
            .unwrap();
            insta::assert_binary_snapshot!(&("astc_".to_string() + &file_name), result.data);
            results.push(TranscodeResult {
                data: Vec::new(),
                ..result
            });
        }
        insta::assert_debug_snapshot!(results);
    }

    #[test]
    fn transcode_assets_uncompressed() {
        let mut path = std::path::PathBuf::new();
        path.push(std::env!("CARGO_MANIFEST_DIR"));
        path.push("../../assets");
        block_on(crate::basisu_init());
        let mut results = Vec::new();
        for file in std::fs::read_dir(path).unwrap() {
            let file = file.unwrap();
            let file_name = file.file_name().into_string().unwrap();
            if !file_name.ends_with(".basisu.ktx2") {
                continue;
            }
            let data = std::fs::read(file.path()).unwrap();
            let result = crate::basisu_transcode(
                data,
                SupportedTextureCompressionMethods::NONE,
                ChannelType::CHANNEL_UNDEFINED,
                TranscodedTextureFormat::cTFTotalTextureFormats,
            )
            .unwrap();
            insta::assert_binary_snapshot!(
                &("uncompressed_".to_string() + &file_name),
                result.data
            );
            results.push(TranscodeResult {
                data: Vec::new(),
                ..result
            });
        }
        insta::assert_debug_snapshot!(results);
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
