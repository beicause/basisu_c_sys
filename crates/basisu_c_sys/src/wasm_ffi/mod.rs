mod ffi;
pub use ffi::*;

// `__errno_location` is defined in C (src/errno.c) with the correct
// `int *` signature. `puts` and `getenv` are not provided here — if
// any linked C/C++ code references them, the link will fail loudly
// rather than silently call a wrong-ABI shim.

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __assert_fail(
    _assertion: *const CChar,
    _file: *const CChar,
    _line: CInt,
    _function: *const CChar,
) {
    // wasm32-unknown-unknown has no stderr and no std to format the message
    // with — trap immediately.
    core::arch::wasm32::unreachable();
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn strncpy_short() {
        let src = b"hi\0";
        let mut dest = *b"abcdef";
        let result = unsafe { ffi::strncpy(dest.as_mut_ptr(), src.as_ptr(), 5) };
        assert_eq!(
            unsafe { core::slice::from_raw_parts(result, 6) },
            *b"hi\0\0\0f"
        );
    }

    #[test]
    fn strncpy_two() {
        let src = b"hello\0";
        let mut dest = [0u8; 2];
        let result = unsafe { ffi::strncpy(dest.as_mut_ptr(), src.as_ptr(), dest.len()) };
        assert_eq!(unsafe { core::slice::from_raw_parts(result, 2) }, b"he");
    }
}
