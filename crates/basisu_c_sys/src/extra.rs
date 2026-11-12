//! Safe wrappers around the [encoder](crate::encoder) and
//! [transcoder](crate::transcoder) C APIs.

#[cfg(feature = "encoder")]
#[cfg_attr(docsrs, doc(cfg(feature = "encoder")))]
mod encoder;
mod transcoder;

#[cfg(feature = "encoder")]
#[cfg_attr(docsrs, doc(cfg(feature = "encoder")))]
pub use encoder::*;
pub use transcoder::*;

/// Texture and image types shared with the encoder and transcoder wrappers.
pub mod types;
