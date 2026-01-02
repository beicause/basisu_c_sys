#![expect(clippy::missing_safety_doc, reason = "TODO")]

use std::cell::OnceCell;

pub use crate::transcoding::{TextureCompressionMethod, TextureTranscodedFormat, Transcoder};

use js_sys::Object;
use js_sys::Reflect;
use js_sys::Uint8Array;

mod bindings_sys {
    use super::Transcoder;
    use js_sys::Uint8Array;
    use wasm_bindgen::prelude::wasm_bindgen;
    type TextureCompressionMethodRepr = u8;
    type TextureTranscodedFormatRepr = u32;

    #[wasm_bindgen]
    extern "C" {
        #[derive(Debug)]
        pub type BasisuVendor;

        #[wasm_bindgen(method,getter,js_name=HEAPU8)]
        pub fn js_basisu_heapu8(this: &BasisuVendor) -> Uint8Array;
        #[wasm_bindgen(method,js_name=_malloc)]
        pub fn js_basisu_malloc(this: &BasisuVendor, size: usize) -> usize;
        #[wasm_bindgen(method,js_name=_free)]
        pub fn js_basisu_free(this: &BasisuVendor, ptr: usize);

        #[wasm_bindgen(method,js_name=_c_basisu_transcoder_init)]
        pub fn js_basisu_transcoder_init(this: &BasisuVendor);
        #[wasm_bindgen(method,js_name=_c_ktx2_transcoder_new)]
        pub fn js_ktx2_transcoder_new(this: &BasisuVendor) -> *mut Transcoder;
        #[wasm_bindgen(method,js_name=_c_ktx2_transcoder_delete)]
        pub fn js_ktx2_transcoder_delete(this: &BasisuVendor, transcoder: *mut Transcoder);
        #[wasm_bindgen(method,js_name=_c_ktx2_transcoder_transcode_image)]
        pub fn js_ktx2_transcoder_transcode_image(
            this: &BasisuVendor,
            transcoder: *mut Transcoder,
            data: usize,
            data_len: u32,
            supported_compressed_formats: TextureCompressionMethodRepr,
        ) -> bool;
        #[wasm_bindgen(method,js_name=_c_ktx2_transcoder_get_r_dst_buf)]
        pub fn js_ktx2_transcoder_get_r_dst_buf(
            this: &BasisuVendor,
            transcoder: *mut Transcoder,
        ) -> u32;
        #[wasm_bindgen(method,js_name=_c_ktx2_transcoder_get_r_dst_buf_len)]
        pub fn js_ktx2_transcoder_get_r_dst_buf_len(
            this: &BasisuVendor,
            transcoder: *mut Transcoder,
        ) -> u32;
        #[wasm_bindgen(method,js_name=_c_ktx2_transcoder_get_r_width)]
        pub fn js_ktx2_transcoder_get_r_width(
            this: &BasisuVendor,
            transcoder: *mut Transcoder,
        ) -> ::std::os::raw::c_uint;
        #[wasm_bindgen(method,js_name=_c_ktx2_transcoder_get_r_height)]
        pub fn js_ktx2_transcoder_get_r_height(
            this: &BasisuVendor,
            transcoder: *mut Transcoder,
        ) -> ::std::os::raw::c_uint;
        #[wasm_bindgen(method,js_name=_c_ktx2_transcoder_get_r_levels)]
        pub fn js_ktx2_transcoder_get_r_levels(
            this: &BasisuVendor,
            transcoder: *mut Transcoder,
        ) -> ::std::os::raw::c_uint;
        #[wasm_bindgen(method,js_name=_c_ktx2_transcoder_get_r_layers)]
        pub fn js_ktx2_transcoder_get_r_layers(
            this: &BasisuVendor,
            transcoder: *mut Transcoder,
        ) -> ::std::os::raw::c_uint;
        #[wasm_bindgen(method,js_name=_c_ktx2_transcoder_get_r_faces)]
        pub fn js_ktx2_transcoder_get_r_faces(
            this: &BasisuVendor,
            transcoder: *mut Transcoder,
        ) -> ::std::os::raw::c_uint;
        #[wasm_bindgen(method,js_name=_c_ktx2_transcoder_get_r_target_format)]
        pub fn js_ktx2_transcoder_get_r_target_format(
            this: &BasisuVendor,
            transcoder: *mut Transcoder,
        ) -> TextureTranscodedFormatRepr;
        #[wasm_bindgen(method,js_name=_c_ktx2_transcoder_get_r_is_srgb)]
        pub fn js_ktx2_transcoder_get_r_is_srgb(
            this: &BasisuVendor,
            transcoder: *mut Transcoder,
        ) -> bool;
    }
}

mod bindings_vendor {
    use super::bindings_sys::BasisuVendor;
    use js_sys::Object;
    use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen(module = "/wasm/basisu_vendor.js")]
    extern "C" {
        #[wasm_bindgen(js_name = "default")]
        pub async fn new_instance(args: &Object) -> BasisuVendor;
    }
}

const BASISU_VENDOR_WASM: &[u8] = include_bytes!("../wasm/basisu_vendor.wasm");

thread_local! {
    static BASISU_VENDOR_INSTANCE: OnceCell<bindings_sys::BasisuVendor> = const{ OnceCell::new() };
}

pub async fn basisu_sys_init_vendor() {
    let binary = Uint8Array::new_from_slice(BASISU_VENDOR_WASM);
    let args = Object::new();
    Reflect::set(&args, &"wasmBinary".into(), &binary).unwrap();
    let instance = bindings_vendor::new_instance(&args).await;
    BASISU_VENDOR_INSTANCE.with(|cell| {
        cell.set(instance).unwrap();
    });
}

pub unsafe fn basisu_transcoder_init() {
    BASISU_VENDOR_INSTANCE.with(|inst| {
        let inst = inst.get().unwrap();
        inst.js_basisu_transcoder_init()
    })
}
pub unsafe fn ktx2_transcoder_delete(transcoder: *mut Transcoder) {
    BASISU_VENDOR_INSTANCE.with(|inst| {
        let inst = inst.get().unwrap();
        inst.js_ktx2_transcoder_delete(transcoder)
    })
}
pub unsafe fn ktx2_transcoder_get_r_faces(transcoder: *mut Transcoder) -> u32 {
    BASISU_VENDOR_INSTANCE.with(|inst| {
        let inst = inst.get().unwrap();
        inst.js_ktx2_transcoder_get_r_faces(transcoder)
    })
}
pub unsafe fn ktx2_transcoder_get_r_height(transcoder: *mut Transcoder) -> u32 {
    BASISU_VENDOR_INSTANCE.with(|inst| {
        let inst = inst.get().unwrap();
        inst.js_ktx2_transcoder_get_r_height(transcoder)
    })
}
pub unsafe fn ktx2_transcoder_get_r_is_srgb(transcoder: *mut Transcoder) -> bool {
    BASISU_VENDOR_INSTANCE.with(|inst| {
        let inst = inst.get().unwrap();
        inst.js_ktx2_transcoder_get_r_is_srgb(transcoder)
    })
}
pub unsafe fn ktx2_transcoder_get_r_layers(transcoder: *mut Transcoder) -> u32 {
    BASISU_VENDOR_INSTANCE.with(|inst| {
        let inst = inst.get().unwrap();
        inst.js_ktx2_transcoder_get_r_layers(transcoder)
    })
}
pub unsafe fn ktx2_transcoder_get_r_levels(transcoder: *mut Transcoder) -> u32 {
    BASISU_VENDOR_INSTANCE.with(|inst| {
        let inst = inst.get().unwrap();
        inst.js_ktx2_transcoder_get_r_levels(transcoder)
    })
}
pub unsafe fn ktx2_transcoder_get_r_target_format(
    transcoder: *mut Transcoder,
) -> TextureTranscodedFormat {
    TextureTranscodedFormat(BASISU_VENDOR_INSTANCE.with(|inst| {
        let inst = inst.get().unwrap();
        inst.js_ktx2_transcoder_get_r_target_format(transcoder)
    }))
}
pub unsafe fn ktx2_transcoder_get_r_width(transcoder: *mut Transcoder) -> u32 {
    BASISU_VENDOR_INSTANCE.with(|inst| {
        let inst = inst.get().unwrap();
        inst.js_ktx2_transcoder_get_r_width(transcoder)
    })
}
pub unsafe fn ktx2_transcoder_new() -> *mut Transcoder {
    BASISU_VENDOR_INSTANCE.with(|inst| {
        let inst = inst.get().unwrap();
        inst.js_ktx2_transcoder_new()
    })
}

pub unsafe fn ktx2_transcoder_transcode_image(
    transcoder: *mut Transcoder,
    data: Vec<u8>,
    supported_compressed_formats: TextureCompressionMethod,
) -> bool {
    BASISU_VENDOR_INSTANCE.with(|inst| {
        let inst = inst.get().unwrap();
        let heap = inst.js_basisu_heapu8();
        let len = u32::try_from(data.len()).unwrap();
        let ptr = inst.js_basisu_malloc(len as usize);
        heap.set(&data.into(), ptr as u32);
        let result = inst.js_ktx2_transcoder_transcode_image(
            transcoder,
            ptr,
            len,
            supported_compressed_formats.0,
        );
        inst.js_basisu_free(ptr);
        result
    })
}

pub unsafe fn ktx2_transcoder_get_r_dst_buf(transcoder: *mut Transcoder) -> Vec<u8> {
    BASISU_VENDOR_INSTANCE.with(|inst| {
        let inst = inst.get().unwrap();
        let dst_buf = inst.js_ktx2_transcoder_get_r_dst_buf(transcoder);
        let dst_len = inst.js_ktx2_transcoder_get_r_dst_buf_len(transcoder);
        inst.js_basisu_heapu8()
            .subarray(dst_buf, dst_buf + dst_len)
            .to_vec()
    })
}
