# Raw Rust binding for the Basis Universal pure C API.

[![Build](https://github.com/beicause/bevy_basisu_loader/actions/workflows/ci.yml/badge.svg)](https://github.com/beicause/bevy_basisu_loader/actions)
[![License](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](https://github.com/beicause/bevy_basisu_loader)
[![Cargo](https://img.shields.io/crates/v/basisu_c_sys.svg)](https://crates.io/crates/basisu_c_sys)
[![Documentation](https://docs.rs/basisu_c_sys/badge.svg)](https://docs.rs/basisu_c_sys)

Raw Rust binding for the basisu pure C API, through FFI on native and wasm-bindgen on web. See also <https://github.com/BinomialLLC/basis_universal/wiki#encoder-and-transcoding-c-api-documentation>. This is used by [bevy_basisu_loader](../bevy_basisu_loader/) and [bevy_basisu_saver](../bevy_basisu_saver/).

Note that BC1, PVRTC1, ATC, FXT1, PVRTC2 are not compiled to reduce size as they are rarely used.

This supports `wasm32-unknown-unknown` by embedding prebuilt basisu wasm binary and `wasm-bindgen`. You need to call `instantiate_embedded_basisu_wasm` before calling other functions if running on web. The prebuilt wasm was prebuilt through:
```sh
cargo r -p basisu_c_sys --bin gen_make_wasm --features __gen_make_wasm -- --emcc-flags="-Os -msimd128 -flto=full -sEVAL_CTORS" --wasm-opt-flags="-Os --enable-simd --enable-bulk-memory-opt --enable-nontrapping-float-to-int"
make -j$(nproc)
```

Feature flags:
- `encoder`: Enable basisu encoder. By default only transcoder is enabled.
- `serde`: Enable serde on structs.
- `extra`: Enable extra high level API for easier usage.
