mod rust;

// `__errno_location` is defined in C (src/wasm_ffi/c/errno.c) with the
// correct `int *` signature.

#[cfg(all(
    target_arch = "wasm32",
    any(target_os = "unknown", target_os = "none"),
))]
mod export;
