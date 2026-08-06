//! Rust implementations of a few libc symbols needed by basis_universal and
//! libc++ on wasm32-unknown-unknown (no system libc).
//!
//! - `atof`  — basisu's HDR nit-multiplier parsing
//! - `lrintf` — basisu's ASTC HDR->LDR fixed-point conversion (round-half-to-even)
//! - `fputc`/`fwrite`/`stderr` — libc++'s verbose_abort.cpp
//!
//! The variadic `printf` family (`printf`/`vprintf`/`vfprintf`/`fprintf`)
//! cannot be defined in Rust (stable Rust cannot define C-ABI variadic
//! functions) and lives in `src/wasm_ffi/stdio_shim.c` instead.

use core::ffi::{c_char, c_void};

/// `double atof(const char *nptr)` — minimal decimal parser
/// (optional sign, digits, fraction, exponent).
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn atof(s: *const c_char) -> f64 {
    if s.is_null() {
        return 0.0;
    }
    let bytes = unsafe { core::ffi::CStr::from_ptr(s) }.to_bytes();
    let mut i = 0usize;
    while i < bytes.len() && matches!(bytes[i], b' ' | b'\t' | b'\n' | b'\r' | 0x0c | 0x0b) {
        i += 1;
    }

    let mut sign = 1.0f64;
    if i < bytes.len() && (bytes[i] == b'-' || bytes[i] == b'+') {
        if bytes[i] == b'-' {
            sign = -1.0;
        }
        i += 1;
    }

    let mut value = 0.0f64;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        value = value * 10.0 + (bytes[i] - b'0') as f64;
        i += 1;
    }

    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        let mut frac = 0.1f64;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            value += (bytes[i] - b'0') as f64 * frac;
            frac *= 0.1;
            i += 1;
        }
    }

    if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
        i += 1;
        let mut esign = 1i32;
        if i < bytes.len() && (bytes[i] == b'-' || bytes[i] == b'+') {
            if bytes[i] == b'-' {
                esign = -1;
            }
            i += 1;
        }
        let mut exp = 0i32;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            exp = exp
                .saturating_mul(10)
                .saturating_add((bytes[i] - b'0') as i32);
            i += 1;
        }
        let mut factor = 1.0f64;
        for _ in 0..exp {
            factor *= 10.0;
        }
        value = if esign < 0 { value / factor } else { value * factor };
    }

    sign * value
}

/// `long int lrintf(float x)` — round half to even, matching hardware
/// semantics. wasm32 has a 32-bit `long`, so the return type is `i32`.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn lrintf(x: f32) -> i32 {
    if x.is_nan() {
        return 0;
    }
    // Clamp so the rounding math below stays in range.
    if x >= 2147483648.0 {
        return i32::MAX;
    }
    if x <= -2147483649.0 {
        return i32::MIN;
    }

    let n = x as i32; // truncate toward zero (Rust `as` is saturating)
    let d = x - n as f32;
    if d > 0.5 {
        n + 1
    } else if d < -0.5 {
        n - 1
    } else if d == 0.5 {
        if n & 1 == 1 {
            n + 1
        } else {
            n
        }
    } else if d == -0.5 {
        if n & 1 == 1 {
            n - 1
        } else {
            n
        }
    } else {
        n
    }
}

/// `int fputc(int c, FILE *stream)` — no-op; there is no stdout on this
/// target and our stub stdio functions never dereference the stream.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn fputc(c: i32, _stream: *mut c_void) -> i32 {
    c
}

/// `size_t fwrite(const void *ptr, size_t size, size_t nmemb, FILE *stream)`
/// — pretend everything was written.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn fwrite(
    _ptr: *const c_void,
    size: usize,
    nmemb: usize,
    _stream: *mut c_void,
) -> usize {
    size * nmemb
}

/// `FILE *const stderr` — data symbol the C side expects. Null is fine:
/// every use goes through our stub `vfprintf`/`fputc`/`fwrite`, which
/// ignore the stream.
#[cfg_attr(not(test), unsafe(no_mangle))]
#[allow(non_upper_case_globals)]
pub static mut stderr: *const c_void = core::ptr::null();
