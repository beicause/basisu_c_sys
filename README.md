# Bevy KTX2 BasisU texture loader

A bevy plugin that provides a loader for ktx2 basis universal textures.

Although Bevy's `ImageLoader` has built-in support for Basis Universal textures via the [`basis-universal-rs`](https://github.com/aclysma/basis-universal-rs) crate, it has some limitations:
1. It uses a relatively old version of Basis Universal
2. No support for UASTC HDR yet
3. No support for Web. Bevy can't be compiled to `wasm32-unknown-emscripten` and `basis-universal-rs` can't be compiled to `wasm32-unknown-unknown`
4. It compiles both the encoder and transcoder and includes transcoding formats not supported by wgpu, which increases binary size

This plugin adds a loader for Basis Universal KTX2 textures with support for ETC1S, UASTC LDR and USATC HDR, and web support through JavaScript glue to call [Basis Universal](https://github.com/BinomialLLC/basis_universal/) C++ library compiled with Emscripten which includes only the transcoder and necessary transcoding formats.

This doesn't include BasisU encoder. To encode textures to `.ktx2`, use the command line tool in [Basis Universal](https://github.com/BinomialLLC/basis_universal/?tab=readme-ov-file#compressing-and-unpacking-ktx2basis-files) repo.

Web demo: https://beicause.github.io/bevy_basisu_loader/

## Usage

1. Add the Cargo dependency:
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

1. Load ktx2 basis universal textures. Supports ETC1S and UASTC and `D2`, `D2Array` and `Cube` texture types. Only supports KTX2 format (zstd compression is supported). No support for `.basis` format.
```rs
    let image_handle = asset_server.load("gl_skybox_etc1s_cubemap_mips_12.basisu_ktx2");
```

⚠️Note: you have to rename the file extension to `.basisu_ktx2` to load it with this `BasisuLoader`. This is a limitations of bevy because otherwise bevy will load `.ktx2` file with its `ImageLoader`.

⚠️Note: The compressed texture dimensions must be a multiplier of block size. See https://github.com/gfx-rs/wgpu/issues/7677 for more context.  
block_size = 4, for etc1s/uastc_ldr/uastc_hdr_4x4  
block_size = 4 or 6 (so both of them need to be satisfied), for uastc_hdr_6x6  

## Implementation details

This plugin supports WebGL2 and WebGPU. To run on web this repo uses a solution:

The `crates/basisu-bindgen` uses `bindgen` to generate Rust binding of the C++ wrapper.

The `crates/basisu-vendor` builds a high level wrapper of the basis universal C++ library. For native platforms, it builds and links the C++ dependency. For web, it's not a cargo dependency and just a cli tool to build basisu wrapper using Emscripten and produce js and wasm files. The basisu wrapper is designed so that it does not need to share memory with the main Wasm module.

The `crates/basisu-sys`. For native platforms, it re-exports the APIs of `basisu-vendor`. For web, it calls the js wrapper which calls the `basisu_vendor.js` and `basisu_vendor.wasm`. Now it can be used by the loader on all platforms!

## Run on web

TLDR: Build your bevy application to `wasm32-unknown-unknown` normally, and copy the `basisu_vendor.js` and `basisu_vendor.wasm` to your webpage assets and provide `basisu_vendor` importmap:
```html
	<script type="importmap">
		{
			"imports": {
				"basisu_vendor": "./basisu_vendor.js"
			}
		}
	</script>
```
The prebuilt wasm can be found in `prebuilt/`. The wasm is built with:
```sh
cargo r -p basisu-vendor --features build-wasm --bin build-wasm -- --emcc-flags="-Os -flto=full" --wasm-opt-flags="-Os"
```

## Bevy version compatibility

| `bevy` | `bevy_basisu_loader` |
| ------ | -------------------- |
| 0.17   | 0.1                  |

## License

Except where noted (below and/or in individual files), all code in this repository is dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.
