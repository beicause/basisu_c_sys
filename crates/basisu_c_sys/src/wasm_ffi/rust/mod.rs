//! Rust parts of the tiny wasm libc — only what musl C cannot provide on
//! bare-metal wasm32 (allocator, signals, basisu's itoa).
//!
//! The string/ctype/math/stdlib/multibyte functions and the atexit
//! registry are compiled from the vendored musl sources instead (see
//! `wasm_libc.rs`), mirroring how sqlite-wasm-rs builds its libc shim.
//!
//! This file is Copyright (c) Jonathan 'theJPster' Pallant 2019
//! Licensed under the Blue Oak Model License 1.0.0
//!
//! See each module for its respective license.

#![allow(clippy::missing_safety_doc)]
// On native test builds nothing references these functions: they are only
// used via the `no_mangle` forwards in `export.rs`, which is
// wasm32-unknown-unknown-only (compiled on wasm, not on native test).
// On wasm there is no dead code, so `expect` must not apply there (it
// would warn about an unfulfilled expectation).
#![cfg_attr(
    all(
        test,
        not(all(target_arch = "wasm32", any(target_os = "unknown", target_os = "none"),)),
    ),
    expect(dead_code)
)]

// The re-exports are consumed by `export.rs`, which is compiled only for
// bare-metal wasm targets; on native test builds they would be unused
// imports (the functions stay alive via the `expect(dead_code)` above).
mod malloc;
#[cfg(all(target_arch = "wasm32", any(target_os = "unknown", target_os = "none")))]
pub use self::malloc::{calloc, free, malloc, realloc};

mod signal;
#[cfg(all(target_arch = "wasm32", any(target_os = "unknown", target_os = "none")))]
pub use self::signal::{abort, raise, signal};
