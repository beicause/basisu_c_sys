#![expect(clippy::missing_safety_doc, reason = "TODO")]

pub use basisu_bindgen::*;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_basisu_transcoder_init() {
    unsafe { basisu_bindgen::c_basisu_transcoder_init() };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_ktx2_transcoder_new() -> *mut Transcoder {
    unsafe { basisu_bindgen::c_ktx2_transcoder_new() }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_ktx2_transcoder_delete(transcoder: *mut Transcoder) {
    unsafe { basisu_bindgen::c_ktx2_transcoder_delete(transcoder) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_ktx2_transcoder_transcode_image(
    transcoder: *mut Transcoder,
    data: *const ::std::os::raw::c_uchar,
    data_size: ::std::os::raw::c_uint,
    supported_compressed_formats: TextureCompressionMethod,
) -> bool {
    unsafe {
        basisu_bindgen::c_ktx2_transcoder_transcode_image(
            transcoder,
            data,
            data_size,
            supported_compressed_formats,
        )
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_ktx2_transcoder_get_r_dst_buf(
    transcoder: *mut Transcoder,
) -> *mut ::std::os::raw::c_uchar {
    unsafe { basisu_bindgen::c_ktx2_transcoder_get_r_dst_buf(transcoder) }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_ktx2_transcoder_get_r_dst_buf_len(
    transcoder: *mut Transcoder,
) -> ::std::os::raw::c_uint {
    unsafe { basisu_bindgen::c_ktx2_transcoder_get_r_dst_buf_len(transcoder) }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_ktx2_transcoder_get_r_width(
    transcoder: *mut Transcoder,
) -> ::std::os::raw::c_uint {
    unsafe { basisu_bindgen::c_ktx2_transcoder_get_r_width(transcoder) }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_ktx2_transcoder_get_r_height(
    transcoder: *mut Transcoder,
) -> ::std::os::raw::c_uint {
    unsafe { basisu_bindgen::c_ktx2_transcoder_get_r_height(transcoder) }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_ktx2_transcoder_get_r_levels(
    transcoder: *mut Transcoder,
) -> ::std::os::raw::c_uint {
    unsafe { basisu_bindgen::c_ktx2_transcoder_get_r_levels(transcoder) }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_ktx2_transcoder_get_r_layers(
    transcoder: *mut Transcoder,
) -> ::std::os::raw::c_uint {
    unsafe { basisu_bindgen::c_ktx2_transcoder_get_r_layers(transcoder) }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_ktx2_transcoder_get_r_faces(
    transcoder: *mut Transcoder,
) -> ::std::os::raw::c_uint {
    unsafe { basisu_bindgen::c_ktx2_transcoder_get_r_faces(transcoder) }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_ktx2_transcoder_get_r_target_format(
    transcoder: *mut Transcoder,
) -> TextureTranscodedFormat {
    unsafe { basisu_bindgen::c_ktx2_transcoder_get_r_target_format(transcoder) }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_ktx2_transcoder_get_r_is_srgb(transcoder: *mut Transcoder) -> bool {
    unsafe { basisu_bindgen::c_ktx2_transcoder_get_r_is_srgb(transcoder) }
}
