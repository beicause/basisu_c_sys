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

Web demo: https://beicause.github.io/bevy_basisu_loader

To encode textures to basisu `.ktx2`,  use [bevy_basisu_saver](../bevy_basisu_saver/) or the command line tool in [Basis Universal](https://github.com/BinomialLLC/basis_universal/?tab=readme-ov-file#compressing-and-unpacking-ktx2basis-files) repo.

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

## Bevy version compatibility

| `bevy` | `bevy_basisu_loader` | `basis_universal` |
| ------ | -------------------- | ----------------- |
| 0.18   | 0.3, 0.4, 0.5, 0.6   | v2_1_0            |
| 0.17   | 0.1, 0.2             | v1_60_snapshot    |

## License

Except where noted (below and/or in individual files), all code in this repository is dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.

[Basis Universal]: https://github.com/BinomialLLC/basis_universal/
