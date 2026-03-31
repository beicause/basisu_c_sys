pub use crate::transcoding::c_basisu_transcoder_init as basisu_transcoder_init;
pub use crate::transcoding::c_ktx2_transcoder_delete as ktx2_transcoder_delete;
pub use crate::transcoding::c_ktx2_transcoder_get_r_basis_format as ktx2_transcoder_get_r_basis_format;
pub use crate::transcoding::c_ktx2_transcoder_get_r_dst_buf_len as ktx2_transcoder_get_r_dst_buf_len;
pub use crate::transcoding::c_ktx2_transcoder_get_r_faces as ktx2_transcoder_get_r_faces;
pub use crate::transcoding::c_ktx2_transcoder_get_r_height as ktx2_transcoder_get_r_height;
pub use crate::transcoding::c_ktx2_transcoder_get_r_is_srgb as ktx2_transcoder_get_r_is_srgb;
pub use crate::transcoding::c_ktx2_transcoder_get_r_layers as ktx2_transcoder_get_r_layers;
pub use crate::transcoding::c_ktx2_transcoder_get_r_levels as ktx2_transcoder_get_r_levels;
pub use crate::transcoding::c_ktx2_transcoder_get_r_preferred_target as ktx2_transcoder_get_r_preferred_target;
pub use crate::transcoding::c_ktx2_transcoder_get_r_width as ktx2_transcoder_get_r_width;
pub use crate::transcoding::c_ktx2_transcoder_new as ktx2_transcoder_new;
pub use crate::transcoding::c_ktx2_transcoder_transcode_image_compute_target_bytes as ktx2_transcoder_transcode_image_compute_target_bytes;
pub use crate::transcoding::c_ktx2_transcoder_transcode_image_write as ktx2_transcoder_transcode_image_write;

pub unsafe fn ktx2_transcoder_transcode_image_get_info(
    transcoder: *mut crate::transcoding::Transcoder,
    data: &[u8],
    supported_compressed_formats: crate::SupportedTextureCompressionMethods,
    channel_type_hint: crate::ChannelType,
) -> bool {
    if data.is_empty() {
        return false;
    }
    unsafe {
        crate::transcoding::c_ktx2_transcoder_transcode_image_get_info(
            transcoder,
            data.as_ptr(),
            data.len() as u32,
            supported_compressed_formats,
            channel_type_hint,
        )
    }
}

#[cfg(test)]
#[cfg_attr(
    not(test),
    expect(unused, reason = "On native we use direct buffer writing")
)]
pub use crate::transcoding::c_ktx2_transcoder_transcode_image_alloc_and_write as ktx2_transcoder_transcode_image_alloc_and_write;

#[cfg(test)]
#[cfg_attr(
    not(test),
    expect(unused, reason = "On native we use direct buffer writing")
)]
pub unsafe fn ktx2_transcoder_get_r_dst_buf(
    transcoder: *mut crate::transcoding::Transcoder,
) -> Vec<u8> {
    let ptr = unsafe { crate::transcoding::c_ktx2_transcoder_get_r_dst_buf(transcoder) };
    let len = unsafe { crate::transcoding::c_ktx2_transcoder_get_r_dst_buf_len(transcoder) };
    let mut ret = vec![0; len as usize];
    unsafe { core::ptr::copy_nonoverlapping(ptr, ret.as_mut_ptr(), len as usize) };
    ret
}

#[expect(unused, reason = "On native we use direct buffer writing")]
pub use crate::transcoding::c_ktx2_transcoder_get_r_dst_buf;
#[expect(unused, reason = "On native we use direct buffer writing")]
pub use crate::transcoding::c_ktx2_transcoder_transcode_image_alloc_and_write;
