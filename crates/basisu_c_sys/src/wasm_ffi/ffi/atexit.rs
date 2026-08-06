//! Rust implementation of `__cxa_atexit`.
//!
//! libc++/libc++abi and basis_universal reference it for C++ static
//! destructors. wasm32-unknown-unknown has no `exit()` (atexit handlers are
//! never actually run), so a small fixed-size registry that satisfies the
//! ABI is enough. musl's own `atexit.c` drags in libc.h/lock.h/
//! fork_impl.h/__libc_malloc machinery, which is not worth compiling
//! piecemeal.

use core::ffi::c_void;

type AtexitFn = unsafe extern "C" fn(*mut c_void);

#[derive(Copy, Clone)]
#[repr(C)]
struct AtexitEntry {
    func: Option<AtexitFn>,
    arg: *mut c_void,
    dso: *mut c_void,
}

const MAX_ATEXIT: usize = 32;

#[allow(non_upper_case_globals)]
static mut ATEXIT_ENTRIES: [AtexitEntry; MAX_ATEXIT] = [AtexitEntry {
    func: None,
    arg: core::ptr::null_mut(),
    dso: core::ptr::null_mut(),
}; MAX_ATEXIT];
#[allow(non_upper_case_globals)]
static mut ATEXIT_COUNT: usize = 0;

/// `int __cxa_atexit(void (*func)(void *), void *arg, void *dso)`
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn __cxa_atexit(func: AtexitFn, arg: *mut c_void, dso: *mut c_void) -> i32 {
    // wasm32-unknown-unknown is single-threaded, so no synchronization is
    // needed.
    unsafe {
        if ATEXIT_COUNT >= MAX_ATEXIT {
            return -1;
        }
        ATEXIT_ENTRIES[ATEXIT_COUNT] = AtexitEntry {
            func: Some(func),
            arg,
            dso,
        };
        ATEXIT_COUNT += 1;
    }
    0
}
