//! Rust parts of the tiny wasm libc — only what musl C cannot provide on
//! bare-metal wasm32 (allocator, atexit, signals, basisu's itoa).
//!
//! The string/ctype/math/stdlib/multibyte functions are compiled from the
//! vendored musl sources instead (see `wasm_libc.rs`), mirroring how
//! sqlite-wasm-rs builds its libc shim.
//!
//! This file is Copyright (c) Jonathan 'theJPster' Pallant 2019
//! Licensed under the Blue Oak Model License 1.0.0
//!
//! See each module for its respective license.

#![allow(clippy::missing_safety_doc)]
#![allow(unused_imports)]
// On native test builds nothing references these functions: they are only
// used via the `no_mangle` forwards in `export.rs`, which is
// wasm32-unknown-unknown-only (compiled on wasm, not on native test).
// On wasm there is no dead code, so `expect` must not apply there (it
// would warn about an unfulfilled expectation).
#![cfg_attr(
    all(
        test,
        not(all(
            target_arch = "wasm32",
            any(target_os = "unknown", target_os = "none"),
        )),
    ),
    expect(dead_code)
)]

mod malloc;
pub use self::malloc::{calloc, free, malloc, realloc};

mod itoa;
pub use self::itoa::itoa;
pub use self::itoa::utoa;

mod signal;
pub use self::signal::{abort, raise, signal};

mod atexit;
pub use self::atexit::__cxa_atexit;

mod ctype;
pub use self::ctype::*;
