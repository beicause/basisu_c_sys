#[expect(
    non_upper_case_globals,
    reason = "Generated rust bindings are OK to be unused or non upper case"
)]
mod transcoding {
    include!(concat!(env!("OUT_DIR"), "/transcoding.rs"));
}

pub use transcoding::*;
