# Bevy KTX2 BasisU texture loader

This is a bevy plugin that provides a loader for ktx2 basis universal textures.

Although Bevy includes built-in support for Basis Universal textures via the [`basis-universal-rs`](https://github.com/aclysma/basis-universal-rs) crate, it has some limitations:
1. It uses a relatively older version of Basis Universal
2. No support for UASTC HDR
3. No support for WASM
4. It compiles both the encoder and transcoder and includes transcoding formats not supported by wgpu, which increases binary size

This plugin adds supports for USATC HDR, and WASM through using JavaScript to call BasisU compiled with `wasm32-unknown-emscripten`, and includes only the transcoder and necessary transcoding formats.

## Usage

1. Add Cargo dependency:
```toml
bevy_basisu_loader = { version = "0.1", git = "https://github.com/beicause/bevy_basisu_loader" }
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
3. Load ktx2 basis universal textures. Supports ETC1S and UASTC formats and `D2`, `D2Array` and `Cube` texture types. Only supports KTX2 format (zstd compression is supported). `.basis` is not supported.
```rs
    let skybox_handle = asset_server.load("gl_skybox_etc1s_cubemap_mips_12.basisu_ktx2");
```

⚠️Note: you have to rename the file extension to `.basisu_ktx2` to load it with this `BasisuLoader`. This is a limitations of bevy because otherwise bevy will load `.ktx2` file with its `ImageLoader`.

⚠️Note: The ompressed texture dimensions must be a multiplier of block size. See https://github.com/gfx-rs/wgpu/issues/7677 for more context.
block_size = 4, for etc1s/uastc_ldr/uastc_hdr_4x4
block_size = 4 or 6, for uastc_hdr_6x6

## Implementation details

This plugin supports WebGL2 and WebGPU. To run on web this repo uses a solution:

The `crates/basisu-bindgen` uses `bindgen` to generate Rust binding of the C++ wrapper.

The `crates/basisu-vendor` builds a high level wrapper of the basis universal C++ library. For native platforms it builds and links the C++ dependency. For web, it's not a cargo dependency and needs to be build manually with `cargo b -p basisu-vendor --target wasm32-unknown-emscripten`. The wrapper interface is designed so that it does not need to share memory with the main WASM.

The `crates/basisu-sys`. For native platforms it re-exports the APIs of `basisu-vendor`. For web it calls a JavaScript wrapper which calls the `basisu-vendor.js` and `basisu_vendor.wasm`. Now we can use it on all platforms.

## Run on web

TLDR: Copy the `basisu-vendor.js` and `basisu_vendor.wasm` to the root dir of your webpage. The prebuilt wasm can be found in `prebuild/`.

Or Clone this repo and build them from source with the config in `.cargo/config.toml`:
```sh
cargo b -p basisu-vendor --target wasm32-unknown-emscripten --profile web_release
```
And copy them from `target/wasm32-unknown-emscripten/web_release/`

## License

Except where noted (below and/or in individual files), all code in this repository is dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.
