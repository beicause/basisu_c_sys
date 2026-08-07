//! Rust implementation of C library `malloc`, `calloc`, `realloc`, `free`,
//! and `aligned_alloc`.
//!
//! Every function returns a pointer whose 16 bytes immediately before it
//! hold a two-word header:
//!
//!   [base_ptr] [alloc_size]
//!
//! `base_ptr` is the original allocation (for `dealloc`), `alloc_size` is
//! the total allocated size (for the dealloc layout). `aligned_alloc`
//! pads the payload up to the requested alignment, so the header moves
//! with it; `free`/`realloc` work uniformly on any of the five functions.

extern crate alloc;

use core::ptr;

const MAX_ALIGN: usize = 16;

const HDR_BASE: usize = 0;
const HDR_SIZE: usize = 8;

/// Total size to allocate for a payload of `size` bytes, or `None` if the
/// total overflows `usize`. C `malloc` must report failure (NULL) on
/// overflow, not wrap around.
fn total_for(size: usize) -> Option<usize> {
    size.checked_add(MAX_ALIGN)
}

/// Write the two-word header directly before `payload` (16 bytes before).
unsafe fn set_header(payload: *mut u8, base: *mut u8, alloc_size: usize) {
    unsafe {
        let hdr = payload.sub(MAX_ALIGN) as *mut usize;
        *hdr.add(HDR_BASE / 8) = base as usize;
        *hdr.add(HDR_SIZE / 8) = alloc_size;
    }
}

/// Read the header directly before `ptr`, returning (base, alloc_size).
unsafe fn read_header(ptr: *mut u8) -> (*mut u8, usize) {
    unsafe {
        let hdr = ptr.sub(MAX_ALIGN) as *mut usize;
        (*hdr.add(HDR_BASE / 8) as *mut u8, *hdr.add(HDR_SIZE / 8))
    }
}

/// Base allocation with 16-byte alignment and header at payload-16.
unsafe fn alloc_with_header(alloc_size: usize) -> *mut u8 {
    let layout = alloc::alloc::Layout::from_size_align(alloc_size, MAX_ALIGN).unwrap();
    let base = unsafe { alloc::alloc::alloc(layout) };
    if base.is_null() {
        return base;
    }
    let payload = unsafe { base.add(MAX_ALIGN) };
    unsafe {
        set_header(payload, base, alloc_size);
    }
    payload
}

pub unsafe extern "C" fn malloc(size: usize) -> *mut u8 {
    let Some(total) = total_for(size) else {
        return ptr::null_mut();
    };
    unsafe { alloc_with_header(total) }
}

pub unsafe extern "C" fn calloc(nmemb: usize, size: usize) -> *mut u8 {
    // C11: `nmemb * size` overflow is undefined behavior; fail like a
    // conforming libc (return NULL) instead of wrapping.
    let Some(total_size) = nmemb.checked_mul(size) else {
        return ptr::null_mut();
    };
    let Some(total) = total_for(total_size) else {
        return ptr::null_mut();
    };
    let layout = alloc::alloc::Layout::from_size_align(total, MAX_ALIGN).unwrap();
    let base = unsafe { alloc::alloc::alloc_zeroed(layout) };
    if base.is_null() {
        return base;
    }
    let payload = unsafe { base.add(MAX_ALIGN) };
    unsafe {
        set_header(payload, base, total);
    }
    payload
}

pub unsafe extern "C" fn realloc(ptr: *mut u8, size: usize) -> *mut u8 {
    unsafe {
        if ptr.is_null() {
            return malloc(size);
        }
        let (base, old_total) = read_header(ptr);
        let layout = alloc::alloc::Layout::from_size_align(old_total, MAX_ALIGN).unwrap();
        let Some(new_total) = total_for(size) else {
            return ptr::null_mut();
        };
        let new_base = alloc::alloc::realloc(base, layout, new_total);
        if new_base.is_null() {
            return new_base;
        }
        let payload = new_base.add(MAX_ALIGN);
        set_header(payload, new_base, new_total);
        payload
    }
}

pub unsafe extern "C" fn free(ptr: *mut u8) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let (base, alloc_size) = read_header(ptr);
        let layout = alloc::alloc::Layout::from_size_align(alloc_size, MAX_ALIGN).unwrap();
        alloc::alloc::dealloc(base, layout);
    }
}

/// C11 `aligned_alloc(alignment, size)`. libc++'s aligned `operator new`
/// calls this, so it must genuinely honor the alignment: the payload is
/// padded to an `alignment`-aligned address.
///
/// Any power-of-two `alignment` is supported (the payload address is
/// aligned up to it; a 2-byte request yields a 2-aligned address, 4096
/// yields a 4096-aligned one). The two-word header travels with the
/// payload — it is stored at payload-16 — so `free`/`realloc` work
/// unchanged.
pub unsafe extern "C" fn aligned_alloc(alignment: usize, size: usize) -> *mut u8 {
    if alignment == 0 || !alignment.is_power_of_two() {
        // C11: not a power of two is undefined; be defensive.
        return ptr::null_mut();
    }
    // Round the payload size up to a multiple of the alignment (the C
    // standard requires it; callers like C++ aligned new don't always do).
    let Some(rounded) = size.checked_add(alignment - 1) else {
        return ptr::null_mut();
    };
    let payload_size = rounded & !(alignment - 1);
    // Worst case: MAX_ALIGN header + (alignment - MAX_ALIGN) padding so the
    // payload can be nudged up to an alignment boundary.
    let Some(alloc_size) = payload_size
        .checked_add(MAX_ALIGN)
        .and_then(|v| v.checked_add(alignment))
    else {
        return ptr::null_mut();
    };
    let layout = alloc::alloc::Layout::from_size_align(alloc_size, MAX_ALIGN).unwrap();
    let base = unsafe { alloc::alloc::alloc(layout) };
    if base.is_null() {
        return base;
    }
    // Payload starts at base+16, aligned up to `alignment`.
    let raw = base as usize + MAX_ALIGN;
    let aligned = (raw + alignment - 1) & !(alignment - 1);
    let payload = unsafe { base.add(MAX_ALIGN + (aligned - raw)) };
    unsafe {
        set_header(payload, base, alloc_size);
    }
    payload
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_malloc() {
        let ptr = unsafe { malloc(10) };
        assert!(!ptr.is_null());
        unsafe {
            let (base, alloc_size) = read_header(ptr);
            assert_eq!(base, ptr.sub(MAX_ALIGN));
            assert_eq!(alloc_size, 10 + MAX_ALIGN);
            (0..10).for_each(|i| {
                *ptr.add(i) = i as u8;
            });
            (0..10).for_each(|i| {
                assert_eq!(*ptr.add(i), i as u8);
            });
        }
        unsafe { free(ptr) };
    }

    #[test]
    fn test_calloc() {
        let ptr = unsafe { calloc(10, 10) };
        assert!(!ptr.is_null());
        unsafe {
            let (base, alloc_size) = read_header(ptr);
            assert_eq!(base, ptr.sub(MAX_ALIGN));
            assert_eq!(alloc_size, 100 + MAX_ALIGN);
            (0..100).for_each(|i| {
                assert_eq!(*ptr.add(i), 0);
            });
        }
        unsafe { free(ptr) };
    }

    #[test]
    fn test_realloc() {
        let ptr = unsafe { malloc(10) };
        assert!(!ptr.is_null());
        unsafe {
            (0..10).for_each(|i| {
                *ptr.add(i) = i as u8;
            });
        }
        let ptr = unsafe { realloc(ptr, 20) };
        assert!(!ptr.is_null());
        unsafe {
            let (base, alloc_size) = read_header(ptr);
            assert_eq!(base, ptr.sub(MAX_ALIGN));
            assert_eq!(alloc_size, 20 + MAX_ALIGN);
            (0..10).for_each(|i| {
                assert_eq!(*ptr.add(i), i as u8);
            });
        }
        unsafe { free(ptr) };
    }

    #[test]
    fn test_aligned_alloc() {
        // Any power of two, small or large, is honored.
        for alignment in [1usize, 2, 4, 8, 16, 32, 64, 256, 4096] {
            let ptr = unsafe { aligned_alloc(alignment, 7) };
            assert!(!ptr.is_null(), "aligned_alloc({alignment}) returned null");
            assert_eq!(
                ptr as usize % alignment,
                0,
                "payload not {alignment}-aligned: {ptr:p}"
            );
            unsafe {
                (0..7).for_each(|i| {
                    *ptr.add(i) = i as u8;
                });
            }
            unsafe { free(ptr) };
        }
        // Non-power-of-two alignment is rejected.
        let ptr = unsafe { aligned_alloc(3, 8) };
        assert!(ptr.is_null());
    }

    #[test]
    fn test_size_overflow_returns_null() {
        // C `malloc`/`calloc` report overflow by returning NULL instead of
        // wrapping around (which would corrupt the header invariants).
        let ptr = unsafe { malloc(usize::MAX) };
        assert!(ptr.is_null());
        let ptr = unsafe { calloc(usize::MAX, 2) };
        assert!(ptr.is_null());
        let ptr = unsafe { calloc(usize::MAX, usize::MAX) };
        assert!(ptr.is_null());
        let ptr = unsafe { aligned_alloc(16, usize::MAX) };
        assert!(ptr.is_null());
        let ptr = unsafe { aligned_alloc(usize::MAX, 1) };
        assert!(ptr.is_null());
        // realloc with an overflowing size leaves the old block untouched.
        let ptr = unsafe { malloc(10) };
        assert!(!ptr.is_null());
        unsafe {
            (0..10).for_each(|i| {
                *ptr.add(i) = i as u8;
            });
        }
        let new_ptr = unsafe { realloc(ptr, usize::MAX) };
        assert!(new_ptr.is_null());
        unsafe {
            let (base, alloc_size) = read_header(ptr);
            assert_eq!(base, ptr.sub(MAX_ALIGN));
            assert_eq!(alloc_size, 10 + MAX_ALIGN);
            (0..10).for_each(|i| {
                assert_eq!(*ptr.add(i), i as u8);
            });
        }
        unsafe { free(ptr) };
    }
}
