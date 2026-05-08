/// Instantiate the embedded basisu wasm, required on web before calling other functions.
/// This is no-op on native platforms.
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

pub(crate) unsafe fn copy_host_memory_to_basisu_impl(data: &[u8], basisu_ptr: u64) {
    unsafe { core::ptr::copy_nonoverlapping(data.as_ptr(), basisu_ptr as *mut u8, data.len()) };
}

pub(crate) unsafe fn copy_basisu_memory_to_host_impl(
    basisu_ptr: u64,
    count: u64,
) -> alloc::vec::Vec<u8> {
    let mut dst = alloc::vec![0u8; count as usize];
    unsafe {
        core::ptr::copy_nonoverlapping(basisu_ptr as *mut u8, dst.as_mut_ptr(), count as usize)
    };
    dst
}
