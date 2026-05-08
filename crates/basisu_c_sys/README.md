# Rust binding and wrappers for the Basis Universal pure C API.

[![Build](https://github.com/beicause/bevy_basisu_loader/actions/workflows/ci.yml/badge.svg)](https://github.com/beicause/bevy_basisu_loader/actions)
[![License](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](https://github.com/beicause/bevy_basisu_loader)
[![Cargo](https://img.shields.io/crates/v/basisu_c_sys.svg)](https://crates.io/crates/basisu_c_sys)
[![Documentation](https://docs.rs/basisu_c_sys/badge.svg)](https://docs.rs/basisu_c_sys)

Rust binding and wrappers for the basisu pure C API, through FFI on native and wasm-bindgen on web. See also <https://github.com/BinomialLLC/basis_universal/wiki#encoder-and-transcoding-c-api-documentation>.

This crate also contains optional high level API that is easier to use with `wgpu-types`. Enabling the `extra` cargo feature to use the high level `BasisuEncoder` and `BasisuTranscoder`.

Note that BC1, PVRTC1, ATC, FXT1, PVRTC2 are not compiled to reduce binary size, as they are not transcode targets by default.

This supports `wasm32-unknown-unknown` by embedding prebuilt basisu wasm binary and `wasm-bindgen`. You need to call `instantiate_embedded_basisu_wasm` function before calling other functions if running on the web.

By default, the prebuilt basisu wasm is from [Github artifact in CI](https://github.com/beicause/bevy_basisu_loader/actions/workflows/prebuild.yml). It is built with:
```sh
cargo r -p basisu_c_sys --bin gen_make_wasm --features __gen_make_wasm -- --emcc-flags="-Os -msimd128 -flto=full -sEVAL_CTORS"
mkdir build && cd build
emcmake cmake ..
emmake make -j$(nproc)
```

Feature flags:
- `encoder`: Enable basisu encoder, which will significantly increase the binary size. By default only transcoder is enabled.
- `serde`: Enable `serde` on some structs.
- `extra`: Enable extra high level encoder and transcoder API that is easier to use with `wgpu-types`.
