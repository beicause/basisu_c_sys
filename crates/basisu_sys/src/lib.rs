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
#[expect(
    unused,
    reason = "On wasm32 we use js bindings thus all native functions are expected to be unused. \
    On native `c_ktx2_transcoder_transcode_image_alloc_dst` and `c_ktx2_transcoder_get_r_dst_buf` are unused"
)]
mod transcoding {
    include!(concat!(env!("OUT_DIR"), "/transcoding.rs"));
}

use alloc::vec::Vec;
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

/// Init basisu global data. Must be called before transcoding.
pub async fn basisu_init() {
    #[cfg(all(
        target_arch = "wasm32",
        target_vendor = "unknown",
        target_os = "unknown",
    ))]
    basisu_sys_init_vendor().await;
    unsafe {
        basisu_transcoder_init();
    }
}

pub struct TranscodeResult {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub levels: u32,
    pub layers: u32,
    pub faces: u32,
    pub is_srgb: bool,
    pub basis_format: BasisTextureFormat,
    pub target_format: TranscodedTextureFormat,
}

/// Transcode the basisu ktx2 data.
pub fn basisu_transcode(
    data: Vec<u8>,
    supported_compressed_formats: SupportedTextureCompressionMethods,
    channel_type_hint: ChannelType,
    force_transcode_target: TranscodedTextureFormat,
) -> Option<TranscodeResult> {
    unsafe {
        let transcoder = ktx2_transcoder_new();
        #[cfg(all(
            target_arch = "wasm32",
            target_vendor = "unknown",
            target_os = "unknown",
        ))]
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

        #[cfg(not(all(
            target_arch = "wasm32",
            target_vendor = "unknown",
            target_os = "unknown",
        )))]
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
