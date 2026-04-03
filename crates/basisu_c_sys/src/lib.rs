//! Raw Rust ffi binding for the basis universal pure C API.
//!
//! See also <https://github.com/BinomialLLC/basis_universal/wiki#encoder-and-transcoding-c-api-documentation>.

#![cfg_attr(
    not(all(
        target_arch = "wasm32",
        target_vendor = "unknown",
        target_os = "unknown",
    )),
    no_std
)]

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
pub use web::*;

#[cfg(not(all(
    target_arch = "wasm32",
    target_vendor = "unknown",
    target_os = "unknown",
)))]
pub async fn basisu_builtin_wasm_instantiate() {}

pub mod common {
    include!(concat!(env!("OUT_DIR"), "/basisu_api_common.rs"));
}

#[cfg(all(
    not(all(
        target_arch = "wasm32",
        target_vendor = "unknown",
        target_os = "unknown",
    )),
    feature = "encoder"
))]
pub mod encoder {
    include!(concat!(env!("OUT_DIR"), "/basisu_c_api.rs"));

    impl Bool32 {
        pub fn is_ok(&self) -> bool {
            self.0 != 0
        }
        pub fn is_err(&self) -> bool {
            !self.is_ok()
        }
    }
}

#[cfg(not(all(
    target_arch = "wasm32",
    target_vendor = "unknown",
    target_os = "unknown",
)))]
pub mod transcoder {
    include!(concat!(env!("OUT_DIR"), "/basisu_c_transcoder_api.rs"));

    impl Bool32 {
        pub fn is_ok(&self) -> bool {
            self.0 != 0
        }
        pub fn is_err(&self) -> bool {
            !self.is_ok()
        }
    }
}
