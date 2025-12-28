#![expect(clippy::missing_safety_doc, reason = "TODO")]

pub use basisu_vendor::{TextureCompressionMethod, TextureTranscodedFormat, Transcoder};

pub use basisu_vendor::c_basisu_transcoder_init as basisu_transcoder_init;
pub use basisu_vendor::c_ktx2_transcoder_delete as ktx2_transcoder_delete;
pub use basisu_vendor::c_ktx2_transcoder_get_r_faces as ktx2_transcoder_get_r_faces;
pub use basisu_vendor::c_ktx2_transcoder_get_r_height as ktx2_transcoder_get_r_height;
pub use basisu_vendor::c_ktx2_transcoder_get_r_is_srgb as ktx2_transcoder_get_r_is_srgb;
pub use basisu_vendor::c_ktx2_transcoder_get_r_layers as ktx2_transcoder_get_r_layers;
pub use basisu_vendor::c_ktx2_transcoder_get_r_levels as ktx2_transcoder_get_r_levels;
pub use basisu_vendor::c_ktx2_transcoder_get_r_target_format as ktx2_transcoder_get_r_target_format;
pub use basisu_vendor::c_ktx2_transcoder_get_r_width as ktx2_transcoder_get_r_width;
pub use basisu_vendor::c_ktx2_transcoder_new as ktx2_transcoder_new;

pub unsafe fn ktx2_transcoder_transcode_image(
    transcoder: *mut Transcoder,
    data: Vec<u8>,
    supported_compressed_formats: TextureCompressionMethod,
) -> bool {
    unsafe {
        basisu_vendor::c_ktx2_transcoder_transcode_image(
            transcoder,
            data.as_ptr(),
            u32::try_from(data.len()).unwrap(),
            supported_compressed_formats,
        )
    }
}

pub unsafe fn ktx2_transcoder_get_r_dst_buf(transcoder: *mut Transcoder) -> Vec<u8> {
    let ptr = unsafe { basisu_vendor::c_ktx2_transcoder_get_r_dst_buf(transcoder) };
    let len = unsafe { basisu_vendor::c_ktx2_transcoder_get_r_dst_buf_len(transcoder) };
    let mut ret = vec![0; len as usize];
    unsafe { std::ptr::copy_nonoverlapping(ptr, ret.as_mut_ptr(), len as usize) };
    ret
}
