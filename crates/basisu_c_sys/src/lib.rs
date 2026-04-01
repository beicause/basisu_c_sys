//! Raw Rust ffi binding for the Basis Universal pure C API.

#[expect(
    non_upper_case_globals,
    non_camel_case_types,
    reason = "Generated code is ok"
)]
pub mod encoder {
    include!(concat!(env!("OUT_DIR"), "/basisu_c_api.rs"));
}

#[expect(
    non_upper_case_globals,
    non_camel_case_types,
    reason = "Generated code is ok"
)]
pub mod transcoder {
    include!(concat!(env!("OUT_DIR"), "/basisu_c_transcoder_api.rs"));
}
