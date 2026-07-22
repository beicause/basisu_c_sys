# Rust binding and wrappers for the Basis Universal pure C API.

[![Build](https://github.com/beicause/basisu_c_sys/actions/workflows/ci.yml/badge.svg)](https://github.com/beicause/basisu_c_sys/actions)
[![License](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](https://github.com/beicause/basisu_c_sys)
[![Cargo](https://img.shields.io/crates/v/basisu_c_sys.svg)](https://crates.io/crates/basisu_c_sys)
[![Documentation](https://docs.rs/basisu_c_sys/badge.svg)](https://docs.rs/basisu_c_sys)

Rust binding and wrappers for the basisu pure C API, through FFI on native and wasm-bindgen on web. See also <https://github.com/BinomialLLC/basis_universal/wiki#encoder-and-transcoding-c-api-documentation>.

This crate also contains optional high level API that is easier to use with `wgpu-types`. Enabling the `extra` cargo feature to use the high level `BasisuEncoder` and `BasisuTranscoder`.

This supports `wasm32-unknown-unknown` by bundling basisu wasm binary and `wasm-bindgen`. You need to install [Emscripten SDK](https://emscripten.org/) and [CMake](https://cmake.org/) to compile it. And in your application, call `instantiate_basisu_wasm`(or `instantiate_custom_basisu_wasm`) function before calling other functions.

Feature flags:
- `encoder`: Enable basisu encoder, which will significantly increase the binary size. By default only transcoder is enabled.
- `serde`: Enable `serde` on some structs.
- `extra`: Enable extra high level encoder and transcoder API that is easier to use with `wgpu-types`.

Feature flags to enable specific transcode target:
- `transcode_etc1s_bc3`
- `transcode_etc1s_bc1` 
- `transcode_etc1s_bc4_5` 
- `transcode_etc1s_bc7` 
- `transcode_etc1s_etc2` 
- `transcode_uastc` 
- `transcode_uastc_hdr` 
- `transcode_xuastc` 
- `transcode_astc`

It's recommended to disable unused targets (especially for ETC1S) to reduce binary size.

PVRTC1, ATC, FXT1, PVRTC2 are always disabled since they are rarely used. Note: Some combinations may fail to compile due to upstream bugs.
