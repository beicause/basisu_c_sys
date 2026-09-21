#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg), doc(auto_cfg = false))]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
#[cfg(test)]
extern crate std;

// Brings the vendored musl libc / libc++ and the `#[no_mangle]` libc
// forwarders into the link. It is only a dependency on the bare-metal wasm
// targets, so on every other target this import does not exist either.
#[cfg(all(target_arch = "wasm32", any(target_os = "unknown", target_os = "none"),))]
use basisu_wasm_libcxx as _;

#[cfg(feature = "extra")]
#[cfg_attr(docsrs, doc(cfg(feature = "extra")))]
pub mod extra;

pub mod common {
    include!(concat!(env!("OUT_DIR"), "/basisu_api_common.rs"));
}

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
