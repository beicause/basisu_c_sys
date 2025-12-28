import Vendor from "/basisu-vendor.js";

const vendor = await Vendor();

export function js_basisu_transcoder_init() {
	vendor._rust_basisu_transcoder_init();
}

export function js_ktx2_transcoder_new() {
	return vendor._rust_ktx2_transcoder_new();
}

export function js_ktx2_transcoder_delete(transcoder) {
	vendor._rust_ktx2_transcoder_delete(transcoder);
}

/**
 *
 * @param {number} transcoder
 * @param {Uint8Array} u8_arr
 * @param {number} supported_compressed_formats
 * @returns {boolean}
 */
export function js_ktx2_transcoder_transcode_image(
	transcoder,
	u8_arr,
	supported_compressed_formats,
) {
	const len = u8_arr.length * u8_arr.BYTES_PER_ELEMENT;
	const src_buf = vendor._malloc(len);
	vendor.HEAPU8.set(u8_arr, src_buf);

	let success = vendor._rust_ktx2_transcoder_transcode_image(
		transcoder,
		src_buf,
		len,
		supported_compressed_formats,
	);
	vendor._free(src_buf);
	return success;
}

/**
 *
 * @param {number} transcoder
 * @returns {Uint8Array}
 */
export function js_ktx2_transcoder_get_r_dst_buf(transcoder) {
	const dst_buf = vendor._rust_ktx2_transcoder_get_r_dst_buf(transcoder);
	/** @type {number} */
	const dst_len = vendor._rust_ktx2_transcoder_get_r_dst_buf_len(transcoder);
	const res_arr = new Uint8Array(dst_len);
	res_arr.set(vendor.HEAPU8.subarray(dst_buf, dst_buf + dst_len));
	return res_arr;
}

export function js_ktx2_transcoder_get_r_width(transcoder) {
	return vendor._rust_ktx2_transcoder_get_r_width(transcoder);
}
export function js_ktx2_transcoder_get_r_height(transcoder) {
	return vendor._rust_ktx2_transcoder_get_r_height(transcoder);
}
export function js_ktx2_transcoder_get_r_levels(transcoder) {
	return vendor._rust_ktx2_transcoder_get_r_levels(transcoder);
}
export function js_ktx2_transcoder_get_r_layers(transcoder) {
	return vendor._rust_ktx2_transcoder_get_r_layers(transcoder);
}
export function js_ktx2_transcoder_get_r_faces(transcoder) {
	return vendor._rust_ktx2_transcoder_get_r_faces(transcoder);
}
export function js_ktx2_transcoder_get_r_target_format(transcoder) {
	return vendor._rust_ktx2_transcoder_get_r_target_format(transcoder);
}
export function js_ktx2_transcoder_get_r_is_srgb(transcoder) {
	return vendor._rust_ktx2_transcoder_get_r_is_srgb(transcoder);
}
