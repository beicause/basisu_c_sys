/// Instantiate the embedded basisu wasm, required on web before calling other functions.
/// This is no-op on native platform.
pub async fn instantiate_embedded_basisu_wasm() {}

#[cfg(feature = "encoder")]
#[cfg_attr(docsrs, doc(cfg(feature = "encoder")))]
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

/// Use this to copy memory between host and basisu.
/// This is required on web where memory isn't shared.
/// # Safety
/// `basisu_ptr` must be valid pointer allocated by `bu_alloc` or `bt_alloc`
/// and must be valid for writes of `data.len()` bytes.
pub unsafe fn copy_host_memory_to_basisu(data: &[u8], basisu_ptr: u64) {
    unsafe { core::ptr::copy_nonoverlapping(data.as_ptr(), basisu_ptr as *mut u8, data.len()) };
}

/// Use this to copy memory between host and basisu.
/// This is required on web where memory isn't shared.
/// # Safety
/// `basisu_ptr` must be valid pointer allocated by `bu_alloc` or `bt_alloc`
/// and must be valid for reads of `count` bytes.
pub unsafe fn copy_basisu_memory_to_host(basisu_ptr: u64, count: u64) -> alloc::vec::Vec<u8> {
    let mut dst = alloc::vec![0u8; count as usize];
    unsafe {
        core::ptr::copy_nonoverlapping(basisu_ptr as *mut u8, dst.as_mut_ptr(), count as usize)
    };
    dst
}
