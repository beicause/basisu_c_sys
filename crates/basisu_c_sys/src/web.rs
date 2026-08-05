use async_lock::OnceCell as AsyncOnceCell;
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
const BASISU_WASM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/build/basisu_encoder.wasm"));
#[cfg(not(feature = "encoder"))]
const BASISU_WASM: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/build/basisu_transcoder.wasm"));

thread_local! {
    static BASISU_INSTANCE: OnceCell<Basisu> = const { OnceCell::new() };
}

static BASISU_INITIALIZED: AsyncOnceCell<()> = AsyncOnceCell::new();

mod instance {
    use js_sys::Object;
    use wasm_bindgen::prelude::wasm_bindgen;

    use crate::web::binding::Basisu;

    #[cfg(feature = "encoder")]
    include!(concat!(env!("OUT_DIR"), "/wasm_encoder_inline_js.rs"));

    #[cfg(not(feature = "encoder"))]
    include!(concat!(env!("OUT_DIR"), "/wasm_transcoder_inline_js.rs"));
}

#[cfg_attr(
    docsrs,
    doc(cfg(all(
        target_arch = "wasm32",
        target_vendor = "unknown",
        target_os = "unknown",
    )))
)]
/// Instantiate the basisu wasm, required on web before calling other functions.
///
/// Once [`instantiate_basisu_wasm`]/[`instantiate_custom_basisu_wasm`] is called, the wasm can't be changed.
pub async fn instantiate_basisu_wasm() {
    instantiate_custom_basisu_wasm(BASISU_WASM).await;
}

#[cfg_attr(
    docsrs,
    doc(cfg(all(
        target_arch = "wasm32",
        target_vendor = "unknown",
        target_os = "unknown",
    )))
)]
/// Instantiate the you custom basisu wasm, required on web before calling other functions. The wasm must be compatible with basisu C API.
///
/// Once [`instantiate_basisu_wasm`]/[`instantiate_custom_basisu_wasm`] is called, the wasm can't be changed.
pub async fn instantiate_custom_basisu_wasm(wasm_data: &[u8]) {
    BASISU_INITIALIZED
        .get_or_init(async || {
            let binary = Uint8Array::new_from_slice(wasm_data);
            let args = Object::new();
            Reflect::set(&args, &"wasmBinary".into(), &binary).unwrap();
            let instance = instance::new_instance(&args).await;
            BASISU_INSTANCE.with(|cell| {
                cell.set(instance).unwrap();
            });
        })
        .await;
}

#[cfg(feature = "encoder")]
#[cfg_attr(docsrs, doc(cfg(feature = "encoder")))]
#[expect(
    clippy::missing_safety_doc,
    reason = "Generated basisu C API binding doesn't have safety doc"
)]
pub mod encoder {
    use super::BASISU_INSTANCE;
    use super::Bool32;
    include!(concat!(env!("OUT_DIR"), "/wasm_encoder_pub_funcs.rs"));
}

#[expect(
    clippy::missing_safety_doc,
    clippy::too_many_arguments,
    reason = "The binding is generated"
)]
pub mod transcoder {
    use super::BASISU_INSTANCE;
    use super::Bool32;
    include!(concat!(env!("OUT_DIR"), "/wasm_transcoder_pub_funcs.rs"));
}

pub(crate) unsafe fn copy_host_memory_to_basisu_impl(data: &[u8], basisu_ptr: u64) {
    BASISU_INSTANCE.with(|inst| {
        let inst = inst.get().unwrap();
        inst.wasm_heap_memory()
            .set(&Uint8Array::from(data), basisu_ptr as u32);
    });
}

pub(crate) unsafe fn copy_basisu_memory_to_host_impl(
    basisu_ptr: u64,
    count: u64,
) -> alloc::vec::Vec<u8> {
    BASISU_INSTANCE.with(|inst| {
        let inst = inst.get().unwrap();
        inst.wasm_heap_memory()
            .subarray(basisu_ptr as u32, (basisu_ptr + count) as u32)
            .to_vec()
    })
}
