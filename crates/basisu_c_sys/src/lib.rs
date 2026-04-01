//! Raw Rust ffi binding for the Basis Universal pure C API.

#[expect(nonstandard_style, reason = "Generated code is ok")]
pub mod encoder {
    include!(concat!(env!("OUT_DIR"), "/basisu_c_api.rs"));
}

#[expect(nonstandard_style, reason = "Generated code is ok")]
pub mod transcoder {
    include!(concat!(env!("OUT_DIR"), "/basisu_c_transcoder_api.rs"));
}
