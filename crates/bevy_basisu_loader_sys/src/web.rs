use std::cell::OnceCell;

use js_sys::Object;
use js_sys::Reflect;
use js_sys::Uint8Array;

use crate::BasisTextureFormat;
use crate::ChannelType;
use crate::SupportedTextureCompressionMethods;
use crate::TranscodedTextureFormat;
use crate::transcoding::Transcoder;

mod bindings_sys {
    use super::Transcoder;
    use js_sys::Uint8Array;
    use wasm_bindgen::prelude::wasm_bindgen;
    type SupportedTextureCompressionMethodsRepr = u8;
    type TranscodedTextureFormatRepr = i32;
    type BasisTextureFormatRepr = i32;
    type ChannelTypeRepr = u8;

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
        #[wasm_bindgen(method,js_name=_c_ktx2_transcoder_transcode_image_get_info)]
        pub fn js_ktx2_transcoder_transcode_image_get_info(
            this: &BasisuVendor,
            transcoder: *mut Transcoder,
            data: usize,
            data_len: u32,
            supported_compressed_formats: SupportedTextureCompressionMethodsRepr,
            channel_type_hint: ChannelTypeRepr,
        );
        #[wasm_bindgen(method,js_name=_c_ktx2_transcoder_transcode_image_compute_target_bytes)]
        pub fn js_ktx2_transcoder_transcode_image_compute_target_bytes(
            this: &BasisuVendor,
            transcoder: *mut Transcoder,
            transcode_target: TranscodedTextureFormatRepr,
        ) -> bool;
        #[wasm_bindgen(method,js_name=_c_ktx2_transcoder_transcode_image_alloc_and_write)]
        pub fn js_ktx2_transcoder_transcode_image_alloc_and_write(
            this: &BasisuVendor,
            transcoder: *mut Transcoder,
            transcode_target: TranscodedTextureFormatRepr,
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
        ) -> ::core::ffi::c_uint;
        #[wasm_bindgen(method,js_name=_c_ktx2_transcoder_get_r_height)]
        pub fn js_ktx2_transcoder_get_r_height(
            this: &BasisuVendor,
            transcoder: *mut Transcoder,
        ) -> ::core::ffi::c_uint;
        #[wasm_bindgen(method,js_name=_c_ktx2_transcoder_get_r_levels)]
        pub fn js_ktx2_transcoder_get_r_levels(
            this: &BasisuVendor,
            transcoder: *mut Transcoder,
        ) -> ::core::ffi::c_uint;
        #[wasm_bindgen(method,js_name=_c_ktx2_transcoder_get_r_layers)]
        pub fn js_ktx2_transcoder_get_r_layers(
            this: &BasisuVendor,
            transcoder: *mut Transcoder,
        ) -> ::core::ffi::c_uint;
        #[wasm_bindgen(method,js_name=_c_ktx2_transcoder_get_r_faces)]
        pub fn js_ktx2_transcoder_get_r_faces(
            this: &BasisuVendor,
            transcoder: *mut Transcoder,
        ) -> ::core::ffi::c_uint;
        #[wasm_bindgen(method,js_name=_c_ktx2_transcoder_get_r_preferred_target)]
        pub fn js_ktx2_transcoder_get_r_preferred_target(
            this: &BasisuVendor,
            transcoder: *mut Transcoder,
        ) -> TranscodedTextureFormatRepr;
        #[wasm_bindgen(method,js_name=_c_ktx2_transcoder_get_r_basis_format)]
        pub fn js_ktx2_transcoder_get_r_basis_format(
            this: &BasisuVendor,
            transcoder: *mut Transcoder,
        ) -> BasisTextureFormatRepr;
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
    static BASISU_VENDOR_INSTANCE: OnceCell<bindings_sys::BasisuVendor> = OnceCell::new();
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
pub unsafe fn ktx2_transcoder_get_r_preferred_target(
    transcoder: *mut Transcoder,
) -> TranscodedTextureFormat {
    // SAFETY: Both repr are i32 and always valid.
    unsafe {
        core::mem::transmute(BASISU_VENDOR_INSTANCE.with(|inst| {
            let inst = inst.get().unwrap();
            inst.js_ktx2_transcoder_get_r_preferred_target(transcoder)
        }))
    }
}
pub unsafe fn ktx2_transcoder_get_r_basis_format(
    transcoder: *mut Transcoder,
) -> BasisTextureFormat {
    // SAFETY: Both repr are i32 and always valid.
    unsafe {
        core::mem::transmute(BASISU_VENDOR_INSTANCE.with(|inst| {
            let inst = inst.get().unwrap();
            inst.js_ktx2_transcoder_get_r_basis_format(transcoder)
        }))
    }
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

pub unsafe fn ktx2_transcoder_transcode_image_get_info(
    transcoder: *mut Transcoder,
    data: &[u8],
    supported_compressed_formats: SupportedTextureCompressionMethods,
    channel_type_hint: ChannelType,
) -> usize {
    BASISU_VENDOR_INSTANCE.with(|inst| {
        let inst = inst.get().unwrap();
        let data_len = data.len() as u32;
        let ptr = inst.js_basisu_malloc(data_len as usize);
        let heap = inst.js_basisu_heapu8();
        heap.set(&Uint8Array::from(data), ptr as u32);
        inst.js_ktx2_transcoder_transcode_image_get_info(
            transcoder,
            ptr,
            data_len,
            supported_compressed_formats.0,
            channel_type_hint as u8,
        );
        ptr
    })
}

pub unsafe fn ktx2_transcoder_transcode_image_free_wasm_data(data_ptr: usize) {
    BASISU_VENDOR_INSTANCE.with(|inst| {
        if data_ptr != 0 {
            let inst = inst.get().unwrap();
            inst.js_basisu_free(data_ptr);
        } else {
            panic!("Attempt to free null ptr");
        }
    })
}

pub unsafe fn ktx2_transcoder_transcode_image_compute_target_bytes(
    transcoder: *mut Transcoder,
    transcode_target: TranscodedTextureFormat,
) -> bool {
    BASISU_VENDOR_INSTANCE.with(|inst| {
        let inst = inst.get().unwrap();
        inst.js_ktx2_transcoder_transcode_image_compute_target_bytes(
            transcoder,
            transcode_target as i32,
        )
    })
}

pub unsafe fn ktx2_transcoder_transcode_image_alloc_and_write(
    transcoder: *mut Transcoder,
    transcode_target: TranscodedTextureFormat,
) -> bool {
    BASISU_VENDOR_INSTANCE.with(|inst| {
        let inst = inst.get().unwrap();
        inst.js_ktx2_transcoder_transcode_image_alloc_and_write(transcoder, transcode_target as i32)
    })
}

#[cfg(test)]
pub unsafe fn ktx2_transcoder_get_r_dst_buf_len(_transcoder: *mut Transcoder) -> u32 {
    unreachable!("This is only used for test and doesn't run on web")
}

#[cfg(test)]
pub unsafe fn ktx2_transcoder_transcode_image_write(
    _transcoder: *mut Transcoder,
    _target_format: TranscodedTextureFormat,
    _dst_buffer: *mut ::core::ffi::c_uchar,
) -> bool {
    unreachable!("This is only used for test and doesn't run on web")
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
