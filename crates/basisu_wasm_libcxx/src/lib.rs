//! musl libc, libc++/libc++abi, and the few libc symbols musl cannot provide,
//! for compiling C and C++ to the bare-metal wasm32 targets.
//!
//! The build script compiles and links the vendored libraries and publishes
//! their include paths through `cargo::metadata`; dependents read those from
//! their build script (see the crate README). The Rust `#[unsafe(no_mangle)]`
//! forwarders in `wasm_ffi` are only compiled for the bare-metal wasm
//! targets, where musl's own allocator and signal handling are unusable.
//!
//! On every other target this crate is empty and its build script does
//! nothing; the vendored sources are not compiled.
#![cfg_attr(not(test), no_std)]

#[cfg(any(
    test,
    all(target_arch = "wasm32", any(target_os = "unknown", target_os = "none"),),
))]
mod wasm_ffi;
