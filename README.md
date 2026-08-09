# Basis Universal Rust binding and Bevy integration

[![Build](https://github.com/beicause/basisu_c_sys/actions/workflows/ci.yml/badge.svg)](https://github.com/beicause/basisu_c_sys/actions)
[![License](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](https://github.com/beicause/basisu_c_sys)
[![Cargo](https://img.shields.io/crates/v/basisu_c_sys.svg)](https://crates.io/crates/basisu_c_sys)
[![Documentation](https://docs.rs/basisu_c_sys/badge.svg)](https://docs.rs/basisu_c_sys)

| crate | description |
| --- | ------------- |
| [basisu_c_sys](./crates/basisu_c_sys)| Rust binding and wrappers for Basis Universal C API, through FFI on native and wasm32 |
| [bevy_basisu_loader](./crates/bevy_basisu_loader)| Basisu texture loader for bevy |
| [bevy_basisu_saver](./crates/bevy_basisu_saver/)| Basisu saver and asset processor for bevy |

Documents and live WebGPU example: <https://beicause.github.io/basisu_c_sys>

Note: `bevy_basisu_loader` and `bevy_basisu_saver` are currently considered experimental, with some limitations due to Bevy's asset system, such as the loader currently being unable to be used with `GltfLoader`, and the asset processor and saver not being mature enough.
