//! The bare-metal wasm symbol export table (`wasm32-unknown-unknown`,
//! `wasm32-unknown-none`, `wasm32v1-none`).
//!
//! Every `no_mangle` symbol of this crate lives here, in one place,
//! compiled only for the wasm target. The implementations in `rust/` are
//! plain Rust functions; native test builds compile `rust/` but not this
//! module, so the unit tests run on the host without colliding with libc
//! symbols of the same name.
//!
//! Most libc symbols (`strcmp`, `strlen`, `tolower`, ...) are not defined
//! here — they come from the vendored musl C sources compiled by
//! `wasm_libc.rs`. Only symbols musl cannot provide on bare-metal wasm
//! stay in Rust:
//!
//! - `malloc`/`calloc`/`realloc`/`free` — dlmalloc (musl's malloc needs
//!   brk/mmap syscalls)
//! - `itoa`/`utoa` — basisu-specific, not part of musl
//! - `signal`/`raise`/`abort` — musl's need sigaction syscalls
//! - `__assert_fail` — traps immediately (no stderr on bare-metal wasm)

use super::rust as ffi;
use super::rust::{CChar, CInt, CSizeT};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn itoa(i: i64, s: *mut CChar, s_len: usize, radix: u8) -> i32 {
    unsafe { ffi::itoa(i, s, s_len, radix) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn utoa(u: u64, s: *mut CChar, s_len: usize, radix: u8) -> i32 {
    unsafe { ffi::utoa(u, s, s_len, radix) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn malloc(size: CSizeT) -> *mut u8 {
    unsafe { ffi::malloc(size) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn calloc(nmemb: CSizeT, size: CSizeT) -> *mut u8 {
    unsafe { ffi::calloc(nmemb, size) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn realloc(ptr: *mut u8, size: CSizeT) -> *mut u8 {
    unsafe { ffi::realloc(ptr, size) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn free(ptr: *mut u8) {
    unsafe { ffi::free(ptr) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn signal(sig: i32, handler: usize) -> usize {
    unsafe { ffi::signal(sig, handler) }
}

#[unsafe(no_mangle)]
pub extern "C" fn raise(sig: i32) -> i32 {
    ffi::raise(sig)
}

#[unsafe(no_mangle)]
pub extern "C" fn abort() {
    ffi::abort()
}

/// `__assert_fail` — called by the C side on assertion failure.
/// wasm32-unknown-unknown has no stderr and no std to format the message
/// with — trap immediately.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __assert_fail(
    _assertion: *const CChar,
    _file: *const CChar,
    _line: CInt,
    _function: *const CChar,
) {
    core::arch::wasm32::unreachable();
}
