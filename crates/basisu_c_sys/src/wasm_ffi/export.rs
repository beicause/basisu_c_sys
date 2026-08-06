//! The wasm32-unknown-unknown symbol export table.
//!
//! Every `no_mangle` symbol of this crate lives here, in one place,
//! compiled only for the wasm target. The implementations in `ffi/` are
//! plain Rust functions; native test builds compile `ffi/` but not this
//! module, so the unit tests run on the host without colliding with libc
//! symbols of the same name.

use core::ffi::{c_char, c_void};

use super::ffi;
use super::ffi::{
    CChar, CInt, CIntMax, CLong, CLongLong, CSizeT, CUIntMax, CULong, CULongLong, CVoid,
};

#[unsafe(no_mangle)]
pub extern "C" fn abs(i: CInt) -> CInt {
    ffi::abs(i)
}

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
pub unsafe extern "C" fn memchr(s: *const CVoid, c: CInt, n: CSizeT) -> *const CVoid {
    unsafe { ffi::memchr(s, c, n) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn qsort(
    base: *mut CVoid,
    nel: CSizeT,
    width: CSizeT,
    compar: Option<extern "C" fn(*const CVoid, *const CVoid) -> CInt>,
) {
    unsafe { ffi::qsort(base, nel, width, compar) }
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strcat(dest: *mut CChar, src: *const CChar) -> *const CChar {
    unsafe { ffi::strcat(dest, src) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strchr(haystack: *const CChar, needle: CInt) -> *const CChar {
    unsafe { ffi::strchr(haystack, needle) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strrchr(haystack: *const CChar, needle: CInt) -> *const CChar {
    unsafe { ffi::strrchr(haystack, needle) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strcmp(s1: *const CChar, s2: *const CChar) -> CInt {
    unsafe { ffi::strcmp(s1, s2) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strcpy(dest: *mut CChar, src: *const CChar) -> *const CChar {
    unsafe { ffi::strcpy(dest, src) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strlen(s: *const CChar) -> usize {
    unsafe { ffi::strlen(s) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strncasecmp(s1: *const CChar, s2: *const CChar, n: usize) -> CInt {
    unsafe { ffi::strncasecmp(s1, s2, n) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strncmp(s1: *const CChar, s2: *const CChar, n: usize) -> CInt {
    unsafe { ffi::strncmp(s1, s2, n) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strncpy(
    dest: *mut CChar,
    src: *const CChar,
    count: usize,
) -> *const CChar {
    unsafe { ffi::strncpy(dest, src, count) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strstr(haystack: *const CChar, needle: *const CChar) -> *const CChar {
    unsafe { ffi::strstr(haystack, needle) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn atoi(s: *const CChar) -> CInt {
    unsafe { ffi::atoi(s) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strtol(s: *const CChar, endptr: *mut *const CChar, base: CInt) -> CLong {
    unsafe { ffi::strtol(s, endptr, base) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strtoul(s: *const CChar, endptr: *mut *const CChar, base: CInt) -> CULong {
    unsafe { ffi::strtoul(s, endptr, base) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strtoll(
    s: *const CChar,
    endptr: *mut *const CChar,
    base: CInt,
) -> CLongLong {
    unsafe { ffi::strtoll(s, endptr, base) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strtoull(
    s: *const CChar,
    endptr: *mut *const CChar,
    base: CInt,
) -> CULongLong {
    unsafe { ffi::strtoull(s, endptr, base) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strtoimax(
    s: *const CChar,
    endptr: *mut *const CChar,
    base: CInt,
) -> CIntMax {
    unsafe { ffi::strtoimax(s, endptr, base) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strtoumax(
    s: *const CChar,
    endptr: *mut *const CChar,
    base: CInt,
) -> CUIntMax {
    unsafe { ffi::strtoumax(s, endptr, base) }
}

#[unsafe(no_mangle)]
pub extern "C" fn isspace(argument: CInt) -> CInt {
    ffi::isspace(argument)
}

#[unsafe(no_mangle)]
pub extern "C" fn isdigit(argument: CInt) -> CInt {
    ffi::isdigit(argument)
}

#[unsafe(no_mangle)]
pub extern "C" fn isalpha(argument: CInt) -> CInt {
    ffi::isalpha(argument)
}

#[unsafe(no_mangle)]
pub extern "C" fn isupper(argument: CInt) -> CInt {
    ffi::isupper(argument)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __cxa_atexit(
    func: unsafe extern "C" fn(*mut c_void),
    arg: *mut c_void,
    dso: *mut c_void,
) -> i32 {
    unsafe { ffi::__cxa_atexit(func, arg, dso) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn atof(s: *const c_char) -> f64 {
    unsafe { ffi::atof(s) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lrintf(x: f32) -> i32 {
    unsafe { ffi::lrintf(x) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fputc(c: i32, stream: *mut c_void) -> i32 {
    unsafe { ffi::fputc(c, stream) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fwrite(
    ptr: *const c_void,
    size: usize,
    nmemb: usize,
    stream: *mut c_void,
) -> usize {
    unsafe { ffi::fwrite(ptr, size, nmemb, stream) }
}

/// `FILE *const stderr` — data symbol the C side expects. Null is fine:
/// every use goes through our stub `vfprintf`/`fputc`/`fwrite`, which
/// ignore the stream.
#[unsafe(no_mangle)]
#[allow(non_upper_case_globals)]
pub static mut stderr: *const c_void = core::ptr::null();

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
