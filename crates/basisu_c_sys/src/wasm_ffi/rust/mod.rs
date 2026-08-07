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
    expect(
        dead_code,
        reason = "Native test builds compile the shim only so the unit tests run on the host; the functions are reached via the wasm-only no_mangle forwards"
    )
)]

// No `expect(clippy::missing_safety_doc)` here: `wasm_ffi` is a private
// module (`mod wasm_ffi;` in lib.rs), so none of these functions are
// exported from the crate and that lint never fires on them, on any
// target. The crate-visible safety contract lives on the `no_mangle`
// forwards in `export.rs` instead.

// The re-exports are consumed by `export.rs`, which is compiled only for
// bare-metal wasm targets; on native test builds they would be unused
// imports (the functions stay alive via the `expect(dead_code)` above).
mod malloc;
#[cfg(all(target_arch = "wasm32", any(target_os = "unknown", target_os = "none")))]
pub use self::malloc::{aligned_alloc, calloc, free, malloc, realloc};

mod signal;
#[cfg(all(target_arch = "wasm32", any(target_os = "unknown", target_os = "none")))]
pub use self::signal::{abort, raise, signal};
