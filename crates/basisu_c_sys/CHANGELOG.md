# Change Log

## Unreleased

-

## basisu_c_sys v0.9.0 (2026-08-10)

- Overhaul the wasm compilation implementation, replacing the emscripten and wasm-bindgen bridging with direct compilation of basis universal to wasm32-unknown-unknown by integrating musl libc and emscripten libc++. You no longer need to install emsdk to compile it to wasm32, but a relatively new clang version is required (clang 21 is tested in CI).
- `wgpu-types` dependency is removed and replaced with this crate's own types which mirror `wgpu-types`, so you will have to perform the conversion yourself but this should be straightforward.
