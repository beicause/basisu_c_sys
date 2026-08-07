//! Encoder smoke test — build/link regression for the wasm build.
//!
//! Compresses a tiny RGBA texture through the high-level `extra` encoder
//! API with [`common::BU_COMP_FLAGS_VALIDATE_OUTPUT`] enabled and checks
//! that a non-empty ktx2 payload comes back. The encoder (C++) pulls in
//! extra libc symbols (`wcslen`, `aligned_alloc`, pthread stubs, `stdout`,
//! `fputs`, ...) that the vendored minimal wasm libc has to provide, so
//! building this example with `--all-features` on `wasm32-unknown-unknown`
//! is the link regression test for those symbols:
//!
//! ```sh
//! cargo build -p basisu_c_sys --target wasm32-unknown-unknown --all-features --examples
//! ```
//!
//! The example only builds when the `encoder` and `extra` features are
//! enabled (see `required-features` in `Cargo.toml`).

use basisu_c_sys::common;
use basisu_c_sys::extra::types::Extent3d;
use basisu_c_sys::extra::{
    basisu_encoder_init, BasisuEncoder, BasisuEncoderParams, SourceImage, SourceImageFormat,
};
use basisu_c_sys::BasisTextureFormat;

fn main() {
    // A 4x4 RGBA8 gradient, enough for one 4x4 block.
    const W: u32 = 4;
    const H: u32 = 4;
    let mut pixels = Vec::with_capacity((W * H * 4) as usize);
    for y in 0..H {
        for x in 0..W {
            pixels.extend_from_slice(&[x as u8, y as u8, (x + y) as u8, 255]);
        }
    }

    basisu_encoder_init();

    let image = SourceImage {
        data: &pixels,
        format: SourceImageFormat::Rgba8,
        size: Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
    };

    let mut encoder = BasisuEncoder::new();
    encoder
        .set_image(image)
        .expect("BasisuEncoder::set_image failed");

    // ETC1S with output validation on, so the encoder verifies the
    // compressed data and the vendored-libc symbol surface is exercised.
    let params = BasisuEncoderParams::new_with_linear_defaults(BasisTextureFormat::Etc1s)
        .with_flags(common::BU_COMP_FLAGS_VALIDATE_OUTPUT);

    let ktx2 = encoder.compress(params).expect("compress failed");
    assert!(!ktx2.is_empty(), "compressed payload is empty");

    println!("encoder_smoke example: compressed {} bytes", ktx2.len());
}
