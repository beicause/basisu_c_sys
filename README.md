# Bevy KTX2 BasisU texture loader and saver

| crate | description |
| --- | ------------- |
| [basisu_c_sys](./crates/basisu_c_sys)| Raw Rust binding for the basisu pure C API, through FFI on native and wasm-bindgen on web|
| [bevy_basisu_loader](./crates/bevy_basisu_loader)| Basisu texture loader for bevy |
| [bevy_basisu_saver](./crates/bevy_basisu_saver/)| Basisu saver and asset processor for bevy |

## For developers

To run the examples and tests in this repository, please make sure git `core.symlinks` is enabled,
and clone https://github.com/beicause/basisu_c_sys_asset_files to the project root directory.
