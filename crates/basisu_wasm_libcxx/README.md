# basisu_wasm_libcxx

Vendored [musl](https://musl.libc.org/) libc and emscripten
[libc++/libc++abi](https://github.com/emscripten-core/emscripten) for compiling
C and C++ to the bare-metal wasm32 targets.

The bare-metal wasm targets (`wasm32-unknown-unknown`, `wasm32-unknown-none`,
`wasm32v1-none`) ship no system libc or C++ standard library. This crate's build
script compiles the vendored musl and libc++ sources, links the resulting
archives into the final module, and defines the handful of libc symbols musl
cannot provide without an OS (allocator, signals, assert).

It is intended for other `-sys` crates that need to compile C/C++ to those
targets. On any other target the crate is empty and nothing is compiled.

⚠️ A relatively new clang is required (clang 21 is tested in CI).

## Usage

Depend on the crate and let it link itself; the final binary must reference it
so its archive is pulled in:

```toml
[dependencies]
basisu_wasm_libcxx = { version = "0.1.0", path = "../basisu_wasm_libcxx" }
```

```rust
use basisu_wasm_libcxx as _;
```

The include paths are published as `cargo::metadata` and are available to a
dependent crate's build script through the `links` value
`basisu_wasm_libcxx`:

```rust
// build.rs
fn dep_paths(name: &str) -> Vec<std::path::PathBuf> {
    std::env::var(name).map_or_else(|_| Vec::new(), |value| {
        std::env::split_paths(&value).collect()
    })
}

let libc = dep_paths("DEP_BASISU_WASM_LIBCXX_INCLUDE_LIBC");
let libcxx = dep_paths("DEP_BASISU_WASM_LIBCXX_INCLUDE_LIBCXX");
```

C++ sources must include the libc++ paths before the C library paths,
otherwise libc++'s `<cstddef>`/`<cctype>`/... wrappers cannot find their own
`<stddef.h>`/`<ctype.h>`. C sources only need the libc paths.
