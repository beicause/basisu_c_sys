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
extern crate alloc;

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
pub use native::*;

#[cfg(not(all(
    target_arch = "wasm32",
    target_vendor = "unknown",
    target_os = "unknown",
)))]
pub async fn basisu_builtin_wasm_instantiate() {}

#[cfg(not(all(
    target_arch = "wasm32",
    target_vendor = "unknown",
    target_os = "unknown",
)))]
pub mod native {
    #[cfg(feature = "encoder")]
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

    pub unsafe fn copy_host_memory_to_basisu(data: &[u8], basisu_ptr: u64) {
        unsafe { core::ptr::copy_nonoverlapping(data.as_ptr(), basisu_ptr as *mut u8, data.len()) };
    }

    pub unsafe fn copy_basisu_memory_to_host(basisu_ptr: u64, count: u64) -> alloc::vec::Vec<u8> {
        let mut dst = alloc::vec![0u8;count as usize];
        unsafe {
            core::ptr::copy_nonoverlapping(basisu_ptr as *mut u8, dst.as_mut_ptr(), count as usize)
        };
        dst
    }
}
