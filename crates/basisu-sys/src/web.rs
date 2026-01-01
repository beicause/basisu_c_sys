#![expect(clippy::missing_safety_doc, reason = "TODO")]

pub use crate::transcoding::{TextureCompressionMethod, TextureTranscodedFormat, Transcoder};

pub use bindings_sys::js_basisu_transcoder_init as basisu_transcoder_init;
pub use bindings_sys::js_ktx2_transcoder_delete as ktx2_transcoder_delete;
pub use bindings_sys::js_ktx2_transcoder_get_r_dst_buf as ktx2_transcoder_get_r_dst_buf;
pub use bindings_sys::js_ktx2_transcoder_get_r_faces as ktx2_transcoder_get_r_faces;
pub use bindings_sys::js_ktx2_transcoder_get_r_height as ktx2_transcoder_get_r_height;
pub use bindings_sys::js_ktx2_transcoder_get_r_is_srgb as ktx2_transcoder_get_r_is_srgb;
pub use bindings_sys::js_ktx2_transcoder_get_r_layers as ktx2_transcoder_get_r_layers;
pub use bindings_sys::js_ktx2_transcoder_get_r_levels as ktx2_transcoder_get_r_levels;
pub use bindings_sys::js_ktx2_transcoder_get_r_width as ktx2_transcoder_get_r_width;
pub use bindings_sys::js_ktx2_transcoder_new as ktx2_transcoder_new;
use js_sys::Object;
use js_sys::Reflect;
use js_sys::Uint8Array;

mod bindings_sys {
    use super::Transcoder;
    use wasm_bindgen::prelude::wasm_bindgen;
    type TextureCompressionMethodRepr = u8;
    type TextureTranscodedFormatRepr = u32;

    #[wasm_bindgen(module = "/basisu_sys.js")]
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
    unsafe {
        bindings_sys::js_ktx2_transcoder_transcode_image(
            transcoder,
            data,
            supported_compressed_formats.0,
        )
    }
}

pub unsafe fn ktx2_transcoder_get_r_target_format(
    transcoder: *mut Transcoder,
) -> TextureTranscodedFormat {
    TextureTranscodedFormat(unsafe {
        bindings_sys::js_ktx2_transcoder_get_r_target_format(transcoder)
    })
}

mod bindings_vendor {
    use js_sys::Object;
    use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen(module = "/wasm/basisu_vendor.js")]
    extern "C" {
        #[wasm_bindgen(js_name = "default")]
        pub async fn new_instance(args: &Object) -> Object;
    }
}

const BASISU_VENDOR_WASM: &[u8] = include_bytes!("../wasm/basisu_vendor.wasm");

pub async fn basisu_sys_init_vendor() {
    let binary = Uint8Array::new_from_slice(BASISU_VENDOR_WASM);
    let args = Object::new();
    Reflect::set(&args, &"wasmBinary".into(), &binary).unwrap();
    let instance = bindings_vendor::new_instance(&args).await;
    let obj = Object::new();
    Reflect::set(&obj, &"vendor".into(), &instance).unwrap();
    Reflect::set(&js_sys::global(), &"$basisu_sys".into(), &obj).unwrap();
}
