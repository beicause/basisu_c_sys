# Bevy KTX2 BasisU texture loader

[![Build](https://github.com/beicause/bevy_basisu_loader/actions/workflows/ci.yml/badge.svg)](https://github.com/beicause/bevy_basisu_loader/actions)
[![License](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](https://github.com/beicause/bevy_basisu_loader)
[![Cargo](https://img.shields.io/crates/v/bevy_basisu_loader.svg)](https://crates.io/crates/bevy_basisu_loader)
[![Documentation](https://docs.rs/bevy_basisu_loader/badge.svg)](https://docs.rs/bevy_basisu_loader)

A lightweight, cross-platform Bevy plugin that provides a KTX2 Basis Universal texture loader.

Although Bevy's `ImageLoader` has built-in support for Basis Universal textures via the [`basis-universal-rs`](https://github.com/aclysma/basis-universal-rs) crate, it has some limitations:
1. It uses a very old version of [Basis Universal][].
2. No support for UASTC HDR yet, either ASTC, XUASTC which are added in basis universal v2.0.
3. No support for Web. Bevy can't be compiled to `wasm32-unknown-emscripten` and `basis-universal-rs` can't be compiled to `wasm32-unknown-unknown`.
4. It compiles both the encoder and transcoder and includes transcoding formats not supported by wgpu, which increases binary size.

This plugin adds a loader for Basis Universal KTX2 textures with support for all formats supported by basis universal v2.0 (ETC1S, UASTC, ASTC, XUASTC), and web support through JavaScript glue to call [Basis Universal][] C++ library compiled with Emscripten which includes only the transcoder and necessary transcoding formats.

Note: This doesn't include BasisU encoder. To encode textures to `.ktx2`, use the command line tool in [Basis Universal](https://github.com/BinomialLLC/basis_universal/?tab=readme-ov-file#compressing-and-unpacking-ktx2basis-files) repo.

Web demo: https://beicause.github.io/bevy_basisu_loader/

## Usage

1. Add the Cargo dependency:
```sh
cargo add bevy_basisu_loader
```

2. Add `BasisuLoaderPlugin`:
```rs
use bevy_basisu_loader::BasisuLoaderPlugin;

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BasisuLoaderPlugin);
}
```

1. Load ktx2 basis universal textures. Supports `D2`, `D2Array` and `Cube` texture types. Zstd supercompression is supported. Note: Only supports `.ktx2` format, `.basis` format is not supported.
```rs
    let image_handle = asset_server.load("gl_skybox_etc1s_cubemap_mips_12.basisu.ktx2");
```

⚠️Note: you have to rename the file extension to `.basisu.ktx2` to load it with this `BasisuLoader`, otherwise bevy will load `.ktx2` files with the builtin `ImageLoader`.

⚠️Note: The compressed texture dimensions must be a multiplier of block size. See https://github.com/gfx-rs/wgpu/issues/7677 for more context. Also because basisu can transcode to textures with different block size on different platforms,
the texture dimensions should satisfy all possible block sizes. For example, XUASTC 6x6 can transcode to ASTC 6x6 and BC7, so its dimensions should be a multiplier of 12.

## Run on web

TLDR: Just build your bevy application to `wasm32-unknown-unknown` normally.

The prebuilt wasm in `crates/basisu_sys/wasm` is automatically embedded in binary when building `wasm32-unknown-unknown`. It was prebuilt through CI with:
```sh
cargo r -p bevy_basisu_loader_sys --bin build-wasm-cli --features build-wasm-cli -- --emcc-flags="-Os -msimd128 -flto=full -sEVAL_CTORS" --wasm-opt-flags="-Os --enable-simd --enable-bulk-memory-opt --enable-nontrapping-float-to-int"
```

## Implementation details

To run on web, this repo uses a solution:

The `crates/basisu_sys/` contains a high level wrapper of the basis universal C++ library.

For native platforms, it just builds and statically links the C++ library.

For web, it contains a tool to build basisu vendor using Emscripten and produce js and wasm files. The basisu wrapper is designed so that it does not need to share memory with main Wasm module, instead its memory is copied from/into main Wasm module through javascript. When building this plugin targeting `wasm32-unknown-unknown`, the basisu vendor js and wasm files are embedded into binary and is called through `wasm-bindgen` and `js-sys`.

## Bevy version compatibility

| `bevy` | `bevy_basisu_loader` | `basis_universal` |
| ------ | -------------------- | ----------------- |
| 0.18   | 0.3, 0.4             | v2_0_2            |
| 0.17   | 0.1, 0.2             | v1_60_snapshot    |

## License

Except where noted (below and/or in individual files), all code in this repository is dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.

[Basis Universal]: https://github.com/BinomialLLC/basis_universal/
