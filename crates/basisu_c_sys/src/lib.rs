#![doc = include_str!("../README.md")]
#![cfg_attr(
    all(
        not(all(
            target_arch = "wasm32",
            target_vendor = "unknown",
            target_os = "unknown",
        )),
        not(test)
    ),
    no_std
)]
extern crate alloc;

#[cfg(feature = "extra")]
pub mod extra;

pub mod common {
    include!(concat!(env!("OUT_DIR"), "/basisu_api_common.rs"));
}

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
mod native;
#[cfg(not(all(
    target_arch = "wasm32",
    target_vendor = "unknown",
    target_os = "unknown",
)))]
pub use native::*;

mod utils;
pub use utils::*;
