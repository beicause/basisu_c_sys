# Change Log

## Unreleased

-

## v0.4.4

- `bevy_basisu_loader_sys`: Fix some potential unsoundness.

## v0.4.3

- Update basis universal to v2.1.0
- `bevy_basisu_loader_sys` crate APIs are simplified and safe.
- `bevy_basisu_loader_sys` crate gets snapshot tests.

## v0.4.2

- Guarantee basisu wasm is initialized during plugin adding.

## v0.4.1

- Update README.md

## v0.4.0

- The supported file extension of `BasisuLoader` is changed from `.basisu_ktx2` to `.basisu.ktx2`
- `bevy_basisu_loader_sys/build-wasm-cli` doesn't pass emcc `-msimd128` and wasm-opt `--enable-simd --enable-bulk-memory-opt --enable-nontrapping-float-to-int` flags by default.

## v0.3.2

- Serde `BasisuLoaderSettings::force_transcode_target`

## v0.3.0

- Update bevy to 0.18
- Update basis_universal to v2.0.2, support the new ASTC LDR 4x4-12x12 and XUASTC LDR 4x4-12x12 formats in basis_universal v2.0.
