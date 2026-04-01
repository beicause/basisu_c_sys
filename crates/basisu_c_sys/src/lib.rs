//! Raw Rust ffi binding for the basis universal pure C API.
//!
//! See also https://github.com/BinomialLLC/basis_universal/wiki#encoder-and-transcoding-c-api-documentation

pub mod common {
    include!(concat!(env!("OUT_DIR"), "/basisu_api_common.rs"));
}

pub mod encoder {
    include!(concat!(env!("OUT_DIR"), "/basisu_c_api.rs"));
}

pub mod transcoder {
    include!(concat!(env!("OUT_DIR"), "/basisu_c_transcoder_api.rs"));
}
