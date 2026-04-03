use js_sys::{Object, Reflect, Uint8Array};
use std::cell::OnceCell;

mod binding {
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
const BASISU_WASM: &[u8] = include_bytes!("../wasm/basisu_encoder.wasm");

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

pub async fn basisu_builtin_wasm_instantiate() {
    let binary = Uint8Array::new_from_slice(BASISU_WASM);
    let args = Object::new();
    Reflect::set(&args, &"wasmBinary".into(), &binary).unwrap();
    let instance = instance::new_instance(&args).await;
    BASISU_INSTANCE.with(|cell| {
        cell.set(instance).unwrap();
    });
}
