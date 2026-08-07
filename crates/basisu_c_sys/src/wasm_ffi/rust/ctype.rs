//! Type aliases for C types used by the Rust-side wasm libc exports.
//!
//! Copyright (c) Jonathan 'theJPster' Pallant 2019
//! Licensed under the Blue Oak Model License 1.0.0

/// `size_t`
pub type CSizeT = usize;

/// `int`
pub type CInt = ::core::ffi::c_int;

/// Represents an 8-bit `char`.
pub type CChar = u8;
