#![expect(clippy::missing_safety_doc, reason = "TODO")]

pub use basisu_bindgen::{TextureCompressionMethod, TextureTranscodedFormat, Transcoder};

pub use js::js_basisu_transcoder_init as basisu_transcoder_init;
pub use js::js_ktx2_transcoder_delete as ktx2_transcoder_delete;
pub use js::js_ktx2_transcoder_get_r_dst_buf as ktx2_transcoder_get_r_dst_buf;
pub use js::js_ktx2_transcoder_get_r_faces as ktx2_transcoder_get_r_faces;
pub use js::js_ktx2_transcoder_get_r_height as ktx2_transcoder_get_r_height;
pub use js::js_ktx2_transcoder_get_r_is_srgb as ktx2_transcoder_get_r_is_srgb;
pub use js::js_ktx2_transcoder_get_r_layers as ktx2_transcoder_get_r_layers;
pub use js::js_ktx2_transcoder_get_r_levels as ktx2_transcoder_get_r_levels;
pub use js::js_ktx2_transcoder_get_r_width as ktx2_transcoder_get_r_width;
pub use js::js_ktx2_transcoder_new as ktx2_transcoder_new;

use crate::web::js::js_ktx2_transcoder_get_r_target_format;
use crate::web::js::js_ktx2_transcoder_transcode_image;

mod js {
    use basisu_bindgen::Transcoder;
    use wasm_bindgen::prelude::wasm_bindgen;
    type TextureCompressionMethodRepr = u8;
    type TextureTranscodedFormatRepr = u32;

    #[wasm_bindgen(module = "/basisu.mjs")]
    extern "C" {
        pub unsafe fn js_basisu_transcoder_init();
        pub unsafe fn js_ktx2_transcoder_new() -> *mut Transcoder;
        pub unsafe fn js_ktx2_transcoder_delete(transcoder: *mut Transcoder);
        pub unsafe fn js_ktx2_transcoder_transcode_image(
            transcoder: *mut Transcoder,
            data: Vec<u8>,
            supported_compressed_formats: TextureCompressionMethodRepr,
        ) -> bool;
        pub unsafe fn js_ktx2_transcoder_get_r_dst_buf(transcoder: *mut Transcoder) -> Vec<u8>;
        pub unsafe fn js_ktx2_transcoder_get_r_width(
            transcoder: *mut Transcoder,
        ) -> ::std::os::raw::c_uint;
        pub unsafe fn js_ktx2_transcoder_get_r_height(
            transcoder: *mut Transcoder,
        ) -> ::std::os::raw::c_uint;
        pub unsafe fn js_ktx2_transcoder_get_r_levels(
            transcoder: *mut Transcoder,
        ) -> ::std::os::raw::c_uint;
        pub unsafe fn js_ktx2_transcoder_get_r_layers(
            transcoder: *mut Transcoder,
        ) -> ::std::os::raw::c_uint;
        pub unsafe fn js_ktx2_transcoder_get_r_faces(
            transcoder: *mut Transcoder,
        ) -> ::std::os::raw::c_uint;
        pub unsafe fn js_ktx2_transcoder_get_r_target_format(
            transcoder: *mut Transcoder,
        ) -> TextureTranscodedFormatRepr;
        pub unsafe fn js_ktx2_transcoder_get_r_is_srgb(transcoder: *mut Transcoder) -> bool;
    }
}

pub unsafe fn ktx2_transcoder_transcode_image(
    transcoder: *mut Transcoder,
    data: Vec<u8>,
    supported_compressed_formats: TextureCompressionMethod,
) -> bool {
    unsafe { js_ktx2_transcoder_transcode_image(transcoder, data, supported_compressed_formats.0) }
}

pub unsafe fn ktx2_transcoder_get_r_target_format(
    transcoder: *mut Transcoder,
) -> TextureTranscodedFormat {
    TextureTranscodedFormat(unsafe { js_ktx2_transcoder_get_r_target_format(transcoder) })
}
