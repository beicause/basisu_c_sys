# Bevy BasisU texture saver and asset processor

[![Build](https://github.com/beicause/basisu_c_sys/actions/workflows/ci.yml/badge.svg)](https://github.com/beicause/basisu_c_sys/actions)
[![License](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](https://github.com/beicause/basisu_c_sys)
[![Cargo](https://img.shields.io/crates/v/bevy_basisu_saver.svg)](https://crates.io/crates/bevy_basisu_saver)
[![Documentation](https://docs.rs/bevy_basisu_saver/badge.svg)](https://docs.rs/bevy_basisu_saver)

Basis universal texture encoder and bevy asset processor to transform images to basisu ktx2 textures.

This is based on [basisu_c_sys](../basisu_c_sys/) and [bevy_basisu_loader](../bevy_basisu_loader/). `wasm32-unknown-unknown` should be supported but is less tested.

Note: `bevy_basisu_loader` and `bevy_basisu_saver` are currently considered experimental, with some limitations due to Bevy's asset system, such as the loader currently being unable to be used with `GltfLoader`, and the asset processor and saver not being mature enough.

## Usage

1. Add the Cargo dependency:
```sh
cargo add bevy_basisu_saver
```

2. Add `BasisuSaverPlugin` which registers basisu asset processor:
```rs
use bevy_basisu_saver::BasisuSaverPlugin;

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BasisuSaverPlugin);
}
```

3. To setup asset processor, see also [examples/test_processor](../../examples/test_processor)

## Bevy version compatibility

| `bevy` | `bevy_basisu_loader` | `basis_universal` |
| ------ | -------------------- | ----------------- |
| 0.18   | 0.2, 0.3, 0.4        | v2_1_0            |

## License

Except where noted (below and/or in individual files), all code in this repository is dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.

[Basis Universal]: https://github.com/BinomialLLC/basis_universal/
