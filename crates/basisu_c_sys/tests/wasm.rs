//! wasm-capable encode/decode integration tests.
//!
//! All assets and expected outputs are embedded with [`include_bytes!`], so
//! the tests never touch the filesystem — that's what makes them runnable on
//! `wasm32-unknown-unknown`, where `std` stubs all I/O. They also run
//! natively on every host.
//!
//! The expected outputs are the binary snapshots produced by the native
//! `encoder`/`transcoder` integration tests (kept in the
//! `basisu_c_sys_asset_files` submodule), embedded instead of compared via
//! `insta`, which cannot run on wasm.
//!
//! The libtest harness cannot report pass/fail on `wasm32-unknown-unknown`
//! (std stubs stdout and `std::process::exit`), so `Cargo.toml` sets
//! `harness = false` for this target and it provides its own entry point:
//!
//! - natively: `fn main` runs every test in order; an assertion failure
//!   panics and aborts with a non-zero exit code.
//! - on wasm: `_start` is exported (wasmtime invokes `_start`, not `main`);
//!   an assertion failure panics and aborts to a wasm trap (`unreachable`),
//!   so wasmtime exits with a non-zero code.
//!
//! Run with wasmtime as the cargo test runner:
//!
//! ```sh
//! CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER="wasmtime run" \
//!   cargo test -p basisu_c_sys --test wasm --target wasm32-unknown-unknown --all-features
//! ```

use std::io::Cursor;

use basisu_c_sys::{
    BasisTextureFormat,
    common::{
        BU_COMP_FLAGS_DEBUG_OUTPUT, BU_COMP_FLAGS_GEN_MIPS_CLAMP, BU_COMP_FLAGS_VALIDATE_OUTPUT,
    },
    extra::{
        BasisuEncoder, BasisuEncoderParams, BasisuTranscoder, ChannelType, SourceImage,
        SourceImageFormat, SupportedTextureCompression, basisu_encoder_init,
        basisu_transcoder_init, types,
    },
};
use image::{DynamicImage, ImageFormat, ImageReader};

// ----- embedded source images (encode inputs) -----

/// A 256x256 RGBA texture with alpha, embedded as a PNG.
const ALPHA0_PNG: &[u8] = include_bytes!("../../../original_assets/alpha0.png");
/// A larger photo, embedded as a PNG.
const KODIM20_PNG: &[u8] = include_bytes!("../../../original_assets/kodim20.png");
/// An HDR photo, embedded as an EXR.
const DESK_EXR: &[u8] = include_bytes!("../../../original_assets/Desk_fixed_6x6.exr");

// ----- embedded ktx2 assets (decode inputs) -----

const ALPHA0_ETC1S_KTX2: &[u8] = include_bytes!("../../../assets/alpha0_etc1s_mips.basisu.ktx2");
const KODIM20_ASTC_KTX2: &[u8] =
    include_bytes!("../../../assets/kodim20_astc_ldr_8x8_mips.basisu.ktx2");
const WIKIPEDIA_XUASTC_KTX2: &[u8] =
    include_bytes!("../../../assets/wikipedia_xuastc_ldr_8x8_mips.basisu.ktx2");
const DESK_HDR_4X4_KTX2: &[u8] = include_bytes!("../../../assets/desk_uastc_hdr_4x4.basisu.ktx2");

// ----- embedded transcode snapshots (expected decode outputs) -----

const ALPHA0_BCN_SNAPSHOT: &[u8] = include_bytes!(
    "../../../basisu_c_sys_asset_files/tests/snapshots/transcoder__bcn_alpha0_etc1s_mips.snap.bin"
);
const ALPHA0_ASTC_SNAPSHOT: &[u8] = include_bytes!(
    "../../../basisu_c_sys_asset_files/tests/snapshots/transcoder__astc_alpha0_etc1s_mips.snap.bin"
);
const KODIM20_BCN_SNAPSHOT: &[u8] = include_bytes!(
    "../../../basisu_c_sys_asset_files/tests/snapshots/transcoder__bcn_kodim20_astc_ldr_8x8_mips.snap.bin"
);
const KODIM20_ASTC_SNAPSHOT: &[u8] = include_bytes!(
    "../../../basisu_c_sys_asset_files/tests/snapshots/transcoder__astc_kodim20_astc_ldr_8x8_mips.snap.bin"
);
const WIKIPEDIA_BCN_SNAPSHOT: &[u8] = include_bytes!(
    "../../../basisu_c_sys_asset_files/tests/snapshots/transcoder__bcn_wikipedia_xuastc_ldr_8x8_mips.snap.bin"
);
const WIKIPEDIA_ASTC_SNAPSHOT: &[u8] = include_bytes!(
    "../../../basisu_c_sys_asset_files/tests/snapshots/transcoder__astc_wikipedia_xuastc_ldr_8x8_mips.snap.bin"
);
const DESK_HDR_ASTC_SNAPSHOT: &[u8] = include_bytes!(
    "../../../basisu_c_sys_asset_files/tests/snapshots/transcoder__astc_desk_uastc_hdr_4x4.snap.bin"
);
const DESK_HDR_BC_SNAPSHOT: &[u8] = include_bytes!(
    "../../../basisu_c_sys_asset_files/tests/snapshots/transcoder__bcn_desk_uastc_hdr_4x4.snap.bin"
);
// ----- embedded encode snapshot (expected encode output) -----

const DESK_UASTC_HDR_6X6_MIPS_SNAPSHOT: &[u8] = include_bytes!(
    "../../../basisu_c_sys_asset_files/tests/snapshots/encoder__desk_uastc_hdr_6x6_mips.snap.basisu.ktx2"
);

fn main() {
    run_all();
}

/// wasmtime's entry point for `wasm32-unknown-unknown` modules.
#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
extern "C" fn _start() {
    run_all();
}

fn run_all() {
    encode_gradient_roundtrip();
    encode_embedded_png();
    encode_kodim20();
    encode_desk_hdr_snapshot();
    decode_alpha0_snapshots();
    decode_kodim20_snapshots();
    decode_wikipedia_snapshots();
    decode_desk_hdr_snapshot();
    println!("wasm test: all tests passed");
}

/// Compress an RGBA8 image through the high-level `extra` encoder with sRGB
/// defaults.
fn encode_rgba8(
    data: &[u8],
    width: u32,
    height: u32,
    format: BasisTextureFormat,
    flags: u64,
) -> Vec<u8> {
    basisu_encoder_init();
    let mut encoder = BasisuEncoder::new();
    let source = SourceImage {
        data,
        format: SourceImageFormat::Rgba8,
        size: types::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    };
    encoder
        .set_image(source)
        .expect("BasisuEncoder::set_image failed");
    encoder
        .compress(BasisuEncoderParams::new_with_srgb_defaults(format).with_flags(flags))
        .expect("compress failed")
}

/// Compress an RGBA32F image through the high-level `extra` encoder with
/// sRGB defaults.
fn encode_rgba32f(
    data: &[u8],
    width: u32,
    height: u32,
    format: BasisTextureFormat,
    flags: u64,
) -> Vec<u8> {
    basisu_encoder_init();
    let mut encoder = BasisuEncoder::new();
    let source = SourceImage {
        data,
        format: SourceImageFormat::Rgba32Float,
        size: types::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    };
    encoder
        .set_image(source)
        .expect("BasisuEncoder::set_image failed");
    encoder
        .compress(BasisuEncoderParams::new_with_srgb_defaults(format).with_flags(flags))
        .expect("compress failed")
}

/// Transcode an embedded ktx2 and compare the output against the embedded
/// native snapshot, byte for byte.
fn transcode_and_compare(
    ktx2: &[u8],
    supported_compressed_formats: SupportedTextureCompression,
    snapshot: &[u8],
    name: &str,
) {
    basisu_transcoder_init();
    let transcoder = BasisuTranscoder::new(ktx2, supported_compressed_formats, ChannelType::Auto)
        .expect("failed to open the embedded ktx2");
    let image = transcoder.transcode(None, None).expect("transcode failed");
    assert_eq!(
        image.data.len(),
        snapshot.len(),
        "{name} transcode size mismatch"
    );
    assert_eq!(image.data, snapshot, "{name} transcode mismatch");
    println!("{name} snapshot matched: {} bytes", image.data.len());
}

/// Encode the embedded HDR EXR to UASTC HDR with mips and compare against the
/// native snapshot, byte for byte.
fn encode_desk_hdr_snapshot() {
    let img = decode_exr(DESK_EXR);
    assert_eq!(img.width(), 648, "unexpected exr width");
    assert_eq!(img.height(), 876, "unexpected exr height");

    let ktx2 = encode_rgba32f(
        img.as_bytes(),
        img.width(),
        img.height(),
        BasisTextureFormat::UastcHdr6x6,
        BU_COMP_FLAGS_DEBUG_OUTPUT | BU_COMP_FLAGS_VALIDATE_OUTPUT | BU_COMP_FLAGS_GEN_MIPS_CLAMP,
    );
    assert_eq!(
        ktx2, DESK_UASTC_HDR_6X6_MIPS_SNAPSHOT,
        "desk uastc hdr encode mismatch"
    );
    println!("desk hdr encode snapshot matched: {} bytes", ktx2.len());
}

/// Decode an embedded PNG with the `image` crate, then encode it.
fn encode_embedded_png() {
    let img = decode_png(ALPHA0_PNG);
    assert_eq!(img.width(), 256, "unexpected png width");
    assert_eq!(img.height(), 256, "unexpected png height");
    let rgba = img.to_rgba8();

    let ktx2 = encode_rgba8(
        rgba.as_raw(),
        rgba.width(),
        rgba.height(),
        BasisTextureFormat::XuastcLdr4x4,
        BU_COMP_FLAGS_DEBUG_OUTPUT | BU_COMP_FLAGS_VALIDATE_OUTPUT | BU_COMP_FLAGS_GEN_MIPS_CLAMP,
    );
    assert!(!ktx2.is_empty(), "compressed payload is empty");
    println!("encoded png: {} bytes", ktx2.len());
}

/// Encode a larger embedded photo to ASTC with mips.
fn encode_kodim20() {
    let rgba = decode_png(KODIM20_PNG).to_rgba8();
    assert_eq!(rgba.width(), 768, "unexpected png width");
    assert_eq!(rgba.height(), 512, "unexpected png height");

    let ktx2 = encode_rgba8(
        rgba.as_raw(),
        rgba.width(),
        rgba.height(),
        BasisTextureFormat::AstcLdr8x8,
        BU_COMP_FLAGS_DEBUG_OUTPUT | BU_COMP_FLAGS_VALIDATE_OUTPUT | BU_COMP_FLAGS_GEN_MIPS_CLAMP,
    );
    assert!(!ktx2.is_empty(), "compressed payload is empty");
    println!("encoded kodim20: {} bytes", ktx2.len());

    // Decode it back and check that the dimensions survive the round trip.
    basisu_transcoder_init();
    let transcoder = BasisuTranscoder::new(
        &ktx2,
        SupportedTextureCompression::ASTC_LDR
            | SupportedTextureCompression::ASTC_HDR
            | SupportedTextureCompression::ETC2,
        ChannelType::Auto,
    )
    .expect("failed to open the encoded ktx2");
    let info = transcoder.get_info();
    assert_eq!(info.width, rgba.width(), "decoded width mismatch");
    assert_eq!(info.height, rgba.height(), "decoded height mismatch");
    let image = transcoder.transcode(None, None).expect("transcode failed");
    assert!(!image.data.is_empty(), "transcoded image is empty");
    println!("decoded kodim20: {} bytes", image.data.len());
}

/// Encode an in-memory RGBA gradient, then decode it back and check that the
/// dimensions survive the round trip.
fn encode_gradient_roundtrip() {
    const W: u32 = 128;
    const H: u32 = 128;
    let mut pixels = Vec::with_capacity((W * H * 4) as usize);
    for y in 0..H {
        for x in 0..W {
            pixels.extend_from_slice(&[x as u8, y as u8, (x + y) as u8, 255]);
        }
    }

    basisu_encoder_init();
    let mut encoder = BasisuEncoder::new();
    let source = SourceImage {
        data: &pixels,
        format: SourceImageFormat::Rgba8,
        size: types::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
    };
    encoder
        .set_image(source)
        .expect("BasisuEncoder::set_image failed");
    let ktx2 = encoder
        .compress(
            BasisuEncoderParams::new_with_linear_defaults(BasisTextureFormat::XuastcLdr4x4)
                .with_flags(
                    BU_COMP_FLAGS_DEBUG_OUTPUT
                        | BU_COMP_FLAGS_VALIDATE_OUTPUT
                        | BU_COMP_FLAGS_GEN_MIPS_CLAMP,
                ),
        )
        .expect("compress failed");
    assert!(!ktx2.is_empty(), "compressed payload is empty");
    println!("encoded gradient: {} bytes", ktx2.len());

    basisu_transcoder_init();
    let transcoder = BasisuTranscoder::new(
        &ktx2,
        SupportedTextureCompression::empty(),
        ChannelType::Auto,
    )
    .expect("failed to open the encoded ktx2");
    let info = transcoder.get_info();
    assert_eq!(info.width, W, "decoded width mismatch");
    assert_eq!(info.height, H, "decoded height mismatch");
    let image = transcoder.transcode(None, None).expect("transcode failed");
    assert!(!image.data.is_empty(), "transcoded image is empty");
    println!("decoded gradient: {} bytes", image.data.len());
}

/// Transcode the ETC1S ktx2 to BC and ASTC, comparing against the native
/// snapshots.
fn decode_alpha0_snapshots() {
    transcode_and_compare(
        ALPHA0_ETC1S_KTX2,
        SupportedTextureCompression::BC,
        ALPHA0_BCN_SNAPSHOT,
        "alpha0 bcn",
    );
    transcode_and_compare(
        ALPHA0_ETC1S_KTX2,
        SupportedTextureCompression::ASTC_LDR
            | SupportedTextureCompression::ASTC_HDR
            | SupportedTextureCompression::ETC2,
        ALPHA0_ASTC_SNAPSHOT,
        "alpha0 astc",
    );
}

/// Transcode the ASTC ktx2 to BC and ASTC, comparing against the native
/// snapshots.
fn decode_kodim20_snapshots() {
    transcode_and_compare(
        KODIM20_ASTC_KTX2,
        SupportedTextureCompression::BC,
        KODIM20_BCN_SNAPSHOT,
        "kodim20 bcn",
    );
    transcode_and_compare(
        KODIM20_ASTC_KTX2,
        SupportedTextureCompression::ASTC_LDR
            | SupportedTextureCompression::ASTC_HDR
            | SupportedTextureCompression::ETC2,
        KODIM20_ASTC_SNAPSHOT,
        "kodim20 astc",
    );
}

/// Transcode the XUASTC ktx2 to BC and ASTC, comparing against the native
/// snapshots.
fn decode_wikipedia_snapshots() {
    transcode_and_compare(
        WIKIPEDIA_XUASTC_KTX2,
        SupportedTextureCompression::BC,
        WIKIPEDIA_BCN_SNAPSHOT,
        "wikipedia bcn",
    );
    transcode_and_compare(
        WIKIPEDIA_XUASTC_KTX2,
        SupportedTextureCompression::ASTC_LDR
            | SupportedTextureCompression::ASTC_HDR
            | SupportedTextureCompression::ETC2,
        WIKIPEDIA_ASTC_SNAPSHOT,
        "wikipedia astc",
    );
}

/// Transcode the UASTC HDR ktx2 to ASTC, comparing against the native
/// snapshot.
fn decode_desk_hdr_snapshot() {
    transcode_and_compare(
        DESK_HDR_4X4_KTX2,
        SupportedTextureCompression::BC,
        DESK_HDR_BC_SNAPSHOT,
        "desk hdr bc",
    );
    transcode_and_compare(
        DESK_HDR_4X4_KTX2,
        SupportedTextureCompression::ASTC_LDR
            | SupportedTextureCompression::ASTC_HDR
            | SupportedTextureCompression::ETC2,
        DESK_HDR_ASTC_SNAPSHOT,
        "desk hdr astc",
    );
}

fn decode_png(data: &[u8]) -> DynamicImage {
    let mut reader = ImageReader::new(Cursor::new(data));
    reader.set_format(ImageFormat::Png);
    reader.decode().expect("failed to decode the embedded png")
}

fn decode_exr(data: &[u8]) -> DynamicImage {
    let mut reader = ImageReader::new(Cursor::new(data));
    reader.set_format(ImageFormat::OpenExr);
    reader.decode().expect("failed to decode the embedded exr")
}
