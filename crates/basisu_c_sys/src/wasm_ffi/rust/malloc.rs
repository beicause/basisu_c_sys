//! Rust implementation of C library function `malloc`, `calloc`, `realloc`, and `free`.
//!
//! Copyright (c) Gyungmin Myung <gmmyung@kaist.ac.kr>
//! This file is licensed under the Blue Oak Model License 1.0.0

extern crate alloc;

const MAX_ALIGN: usize = 16;

pub unsafe extern "C" fn malloc(size: usize) -> *mut u8 {
    let layout = alloc::alloc::Layout::from_size_align(size + MAX_ALIGN, MAX_ALIGN).unwrap();
    let ptr = unsafe { alloc::alloc::alloc(layout) };
    if ptr.is_null() {
        return ptr;
    }
    unsafe {
        *(ptr as *mut usize) = size;
    }
    unsafe { ptr.add(MAX_ALIGN) }
}

pub unsafe extern "C" fn calloc(nmemb: usize, size: usize) -> *mut u8 {
    let total_size = nmemb * size;
    let layout = alloc::alloc::Layout::from_size_align(total_size + MAX_ALIGN, MAX_ALIGN).unwrap();
    let ptr = unsafe { alloc::alloc::alloc_zeroed(layout) };
    if ptr.is_null() {
        return ptr;
    }
    unsafe {
        *(ptr as *mut usize) = total_size;
    }
    unsafe { ptr.add(MAX_ALIGN) }
}

pub unsafe extern "C" fn realloc(ptr: *mut u8, size: usize) -> *mut u8 {
    unsafe {
        if ptr.is_null() {
            return malloc(size);
        }
        let old_size = *(ptr.sub(MAX_ALIGN) as *mut usize);
        let layout =
            alloc::alloc::Layout::from_size_align(old_size + MAX_ALIGN, MAX_ALIGN).unwrap();
        let new_ptr = alloc::alloc::realloc(ptr.sub(MAX_ALIGN), layout, size + MAX_ALIGN);
        if new_ptr.is_null() {
            return new_ptr;
        }
        *(new_ptr as *mut usize) = size;
        new_ptr.add(MAX_ALIGN)
    }
}

pub unsafe extern "C" fn free(ptr: *mut u8) {
    if ptr.is_null() {
        return;
    }
    let old_size = unsafe { *(ptr.sub(MAX_ALIGN) as *mut usize) };
    let layout = alloc::alloc::Layout::from_size_align(old_size + MAX_ALIGN, MAX_ALIGN).unwrap();
    unsafe { alloc::alloc::dealloc(ptr.sub(MAX_ALIGN), layout) };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_malloc() {
        let ptr = unsafe { malloc(10) };
        assert!(!ptr.is_null());
        unsafe {
            assert_eq!(*(ptr.sub(MAX_ALIGN) as *mut usize), 10);
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
            assert_eq!(*(ptr.sub(MAX_ALIGN) as *mut usize), 100);
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
            assert_eq!(*(ptr.sub(MAX_ALIGN) as *mut usize), 20);
            (0..10).for_each(|i| {
                assert_eq!(*ptr.add(i), i as u8);
            });
        }
        unsafe { free(ptr) };
    }
}
