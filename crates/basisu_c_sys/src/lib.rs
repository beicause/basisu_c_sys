#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg), doc(auto_cfg = false))]
#![no_std]
extern crate alloc;
#[cfg(test)]
extern crate std;

#[cfg(feature = "extra")]
#[cfg_attr(docsrs, doc(cfg(feature = "extra")))]
pub mod extra;

pub mod common {
    include!(concat!(env!("OUT_DIR"), "/basisu_api_common.rs"));
}

#[cfg(any(
    test,
    all(target_arch = "wasm32", any(target_os = "unknown", target_os = "none"),),
))]
mod wasm_ffi;

mod utils;
pub use utils::*;

#[cfg(feature = "encoder")]
#[cfg_attr(docsrs, doc(cfg(feature = "encoder")))]
pub mod encoder {
    include!(concat!(env!("OUT_DIR"), "/basisu_c_api.rs"));
    include!("bool32.rs");
}

pub mod transcoder {
    include!(concat!(env!("OUT_DIR"), "/basisu_c_transcoder_api.rs"));
    include!("bool32.rs");
}
