use js_sys::{Object, Reflect, Uint8Array};
use std::cell::OnceCell;

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct Bool32(pub u32);

impl Bool32 {
    pub fn is_ok(&self) -> bool {
        self.0 != 0
    }
    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }
}

mod binding {
    use js_sys::Uint8Array;
    use wasm_bindgen::prelude::*;

    #[cfg(feature = "encoder")]
    include!(concat!(env!("OUT_DIR"), "/wasm_encoder_binding.rs"));
    #[cfg(not(feature = "encoder"))]
    include!(concat!(env!("OUT_DIR"), "/wasm_transcoder_binding.rs"));
}
use binding::Basisu;

#[cfg(feature = "encoder")]
const BASISU_WASM: &[u8] = include_bytes!("../wasm/basisu_encoder.wasm");
#[cfg(not(feature = "encoder"))]
const BASISU_WASM: &[u8] = include_bytes!("../wasm/basisu_transcoder.wasm");

thread_local! {
    static BASISU_INSTANCE: OnceCell<Basisu> = OnceCell::new();
}

mod instance {
    use js_sys::Object;
    use wasm_bindgen::prelude::wasm_bindgen;

    use crate::web::binding::Basisu;

    #[cfg(feature = "encoder")]
    #[wasm_bindgen(module = "/wasm/basisu_encoder.js")]
    extern "C" {
        #[wasm_bindgen(js_name = "default")]
        pub async fn new_instance(args: &Object) -> Basisu;
    }

    #[cfg(not(feature = "encoder"))]
    #[wasm_bindgen(module = "/wasm/basisu_transcoder.js")]
    extern "C" {
        #[wasm_bindgen(js_name = "default")]
        pub async fn new_instance(args: &Object) -> Basisu;
    }
}

/// Instantiate the embedded basisu wasm, required on web before calling other functions.
/// This is no-op on native platform.
pub async fn instantiate_embedded_basisu_wasm() {
    if BASISU_INSTANCE.with(|cell| cell.get().is_some()) {
        return;
    }
    let binary = Uint8Array::new_from_slice(BASISU_WASM);
    let args = Object::new();
    Reflect::set(&args, &"wasmBinary".into(), &binary).unwrap();
    let instance = instance::new_instance(&args).await;
    BASISU_INSTANCE.with(|cell| {
        cell.set(instance).unwrap();
    });
}

#[cfg(feature = "encoder")]
pub mod encoder {
    use super::BASISU_INSTANCE;
    use super::Bool32;
    include!(concat!(env!("OUT_DIR"), "/wasm_encoder_pub_funcs.rs"));
}

pub mod transcoder {
    use super::BASISU_INSTANCE;
    use super::Bool32;
    include!(concat!(env!("OUT_DIR"), "/wasm_transcoder_pub_funcs.rs"));
}

/// Use this to copy memory between host and basisu.
/// This is required on web where memory isn't shared.
/// # Safety
/// `basisu_ptr` must be valid pointer allocated by `bu_alloc` or `bt_alloc`
/// and must be valid for writes of `data.len()` bytes.
pub unsafe fn copy_host_memory_to_basisu(data: &[u8], basisu_ptr: u64) {
    BASISU_INSTANCE.with(|inst| {
        let inst = inst.get().unwrap();
        inst.wasm_heap_memory()
            .set(&Uint8Array::from(data), basisu_ptr as u32);
    });
}

/// Use this to copy memory between host and basisu.
/// This is required on web where memory isn't shared.
/// # Safety
/// `basisu_ptr` must be valid pointer allocated by `bu_alloc` or `bt_alloc`
/// and must be valid for reads of `count` bytes.
pub unsafe fn copy_basisu_memory_to_host(basisu_ptr: u64, count: u64) -> alloc::vec::Vec<u8> {
    BASISU_INSTANCE.with(|inst| {
        let inst = inst.get().unwrap();
        inst.wasm_heap_memory()
            .subarray(basisu_ptr as u32, (basisu_ptr + count) as u32)
            .to_vec()
    })
}
