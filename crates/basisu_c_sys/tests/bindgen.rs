#[expect(unused, reason = "Some functions are used in build.rs but not here")]
#[path = "../wasm_bindgen.rs"]
mod wasm_bindgen;
