# Change Log

## basisu_c_sys v0.9.1 (2026-09-21)

- Extract the bare-metal wasm32 libc/libc++ compilation into the new `basisu_wasm_libcxx` crate. The wasm build behavior is unchanged.
- Enable the `missing_docs` lint and document all public APIs.
- Update branch references from master to main.

## basisu_c_sys v0.9.0 (2026-08-10)

- Overhaul the wasm compilation implementation, replacing the emscripten and wasm-bindgen bridging with direct compilation of basis universal to wasm32-unknown-unknown by integrating musl libc and emscripten libc++. You no longer need to install emsdk to compile it to wasm32, but a relatively new clang version is required (clang 21 is tested in CI).
- `wgpu-types` dependency is removed and replaced with this crate's own types which mirror `wgpu-types`, so you will have to perform the conversion yourself but this should be straightforward.
