#![cfg_attr(
    not(all(
        target_arch = "wasm32",
        target_vendor = "unknown",
        target_os = "unknown",
    )),
    no_std
)]
extern crate alloc;

#[expect(
    non_upper_case_globals,
    non_camel_case_types,
    reason = "Generated code is OK to have non upper case globals or non camel case enums"
)]
#[cfg_attr(
    all(
        target_arch = "wasm32",
        target_vendor = "unknown",
        target_os = "unknown"
    ),
    expect(
        unused,
        reason = "On wasm32 we use js bindings thus native functions are expected to be unused"
    )
)]
mod transcoding {
    include!(concat!(env!("OUT_DIR"), "/transcoding.rs"));
}

pub use transcoding::{
    ChannelType, SupportedTextureCompressionMethods, TextureTranscodedFormat, Transcoder,
};

#[cfg(not(all(
    target_arch = "wasm32",
    target_vendor = "unknown",
    target_os = "unknown",
)))]
mod native;
#[cfg(not(all(
    target_arch = "wasm32",
    target_vendor = "unknown",
    target_os = "unknown",
)))]
pub use native::*;

#[cfg(all(
    target_arch = "wasm32",
    target_vendor = "unknown",
    target_os = "unknown",
))]
mod web;
#[cfg(all(
    target_arch = "wasm32",
    target_vendor = "unknown",
    target_os = "unknown",
))]
pub use web::*;
