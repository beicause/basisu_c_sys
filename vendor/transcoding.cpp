#include "transcoding.hpp"
#include "basis_universal/transcoder/basisu_transcoder.h"

enum TextureCompressionMethod;

enum ChannelType {
	CHANNEL_RGB,
	CHANNEL_RGBA,
	CHANNEL_R,
	CHANNEL_RG,
};

static ChannelType channel_id_to_type(bool is_uastc,
		basist::ktx2_df_channel_id channel_id0,
		basist::ktx2_df_channel_id channel_id1);

static basist::transcoder_texture_format get_target_texture_format(
		basist::basis_tex_format basis_format, ChannelType channel_type,
		TextureCompressionMethod supported_compressed_formats);

extern "C" {
void basisu_transcoder_init() {
	basist::basisu_transcoder_init();
}

basist::ktx2_transcoder *ktx2_transcoder_new() {
	basist::ktx2_transcoder *transcoder = new basist::ktx2_transcoder();
	return transcoder;
}

void ktx2_transcoder_delete(basist::ktx2_transcoder *transcoder) {
	delete transcoder;
}

void ktx2_transcoder_clear(basist::ktx2_transcoder *transcoder) {
	transcoder->clear();
}

bool ktx2_transcoder_init(basist::ktx2_transcoder *transcoder, const unsigned char *data, unsigned int data_size) {
	return transcoder->init(data, data_size);
}

bool ktx2_transcoder_start(basist::ktx2_transcoder *transcoder) {
	return transcoder->start_transcoding();
}

bool ktx2_transcoder_get_texture_info(basist::ktx2_transcoder *transcoder, TextureTranscodedFormat target_format, unsigned int *r_width, unsigned int *r_height, unsigned int *r_levels, unsigned int *r_layers, unsigned int *r_faces, unsigned int *r_total_bytes) {
	*r_width = transcoder->get_width();
	*r_height = transcoder->get_height();
	*r_levels = transcoder->get_levels();
	*r_layers = transcoder->get_layers();
	*r_faces = transcoder->get_faces();

	const basist::transcoder_texture_format transcode_format = static_cast<basist::transcoder_texture_format>(static_cast<uint32_t>(target_format));

	uint32_t total_bytes = 0;
	uint32_t total_layers = basisu::maximumu(transcoder->get_layers(), 1u);
	for (uint32_t level_index = 0; level_index < transcoder->get_levels(); level_index++) {
		for (uint32_t layer_index = 0; layer_index < total_layers; layer_index++) {
			for (uint32_t face_index = 0; face_index < transcoder->get_faces(); face_index++) {
				basist::ktx2_image_level_info level_info;
				if (!transcoder->get_image_level_info(level_info, level_index, layer_index, face_index)) {
					return false;
				}
				uint32_t bytes = basist::basis_compute_transcoded_image_size_in_bytes(transcode_format, level_info.m_orig_width, level_info.m_orig_height);

				total_bytes += bytes;
			}
		}
	}
	*r_total_bytes = total_bytes;
	return true;
}

TextureTranscodedFormat ktx2_transcoder_get_transcoded_format(basist::ktx2_transcoder *transcoder, TextureCompressionMethod supported_compressed_formats, bool *r_is_srgb) {
	basist::ktx2_df_channel_id channel_id0 = transcoder->get_dfd_channel_id0();
	basist::ktx2_df_channel_id channel_id1 = transcoder->get_dfd_channel_id1();
	basist::basis_tex_format basis_format = transcoder->get_basis_tex_format();
	ChannelType channel_type = channel_id_to_type(transcoder->is_uastc(), channel_id0, channel_id1);
	basist::transcoder_texture_format target_format = get_target_texture_format(basis_format, channel_type, supported_compressed_formats);
	*r_is_srgb = transcoder->get_dfd_transfer_func() == basist::KTX2_KHR_DF_TRANSFER_SRGB;
	return static_cast<TextureTranscodedFormat>(static_cast<uint32_t>(target_format));
}

bool ktx2_transcoder_transcode_image(
		basist::ktx2_transcoder *transcoder,
		const uint8_t *data, uint32_t data_size, TextureTranscodedFormat target_format, uint8_t *r_dst_data, uint32_t r_dst_data_size) {
	const basist::transcoder_texture_format transcode_format = static_cast<basist::transcoder_texture_format>(static_cast<uint32_t>(target_format));

	uint32_t total_layers = basisu::maximumu(transcoder->get_layers(), 1u);
	uint8_t *out = r_dst_data;
	for (uint32_t level_index = 0; level_index < transcoder->get_levels(); level_index++) {
		for (uint32_t layer_index = 0; layer_index < total_layers; layer_index++) {
			for (uint32_t face_index = 0; face_index < transcoder->get_faces(); face_index++) {
				basist::ktx2_image_level_info level_info;
				if (!transcoder->get_image_level_info(level_info, level_index, layer_index, face_index)) {
					return false;
				}

				uint32_t total_dst_blocks_or_pixels;
				if (basist::basis_transcoder_format_is_uncompressed(transcode_format)) {
					total_dst_blocks_or_pixels = level_info.m_orig_width * level_info.m_orig_height;
				} else {
					const uint32_t dst_block_width = basist::basis_get_block_width(transcode_format);
					const uint32_t dst_block_height = basist::basis_get_block_height(transcode_format);

					// Take into account the destination format's block width/height.
					const uint32_t num_dst_blocks_x = (level_info.m_orig_width + dst_block_width - 1) / dst_block_width;
					const uint32_t num_dst_blocks_y = (level_info.m_orig_height + dst_block_height - 1) / dst_block_height;
					total_dst_blocks_or_pixels = num_dst_blocks_x * num_dst_blocks_y;
				}

				if (!transcoder->transcode_image_level(level_index, layer_index, face_index, out, total_dst_blocks_or_pixels, transcode_format)) {
					return false;
				}

				uint32_t total_bytes = basist::basis_compute_transcoded_image_size_in_bytes(transcode_format, level_info.m_orig_width, level_info.m_orig_height);
				out += total_bytes;
			}
		}
	}

	return true;
}
}

static ChannelType channel_id_to_type(bool is_uastc,
		basist::ktx2_df_channel_id channel_id0,
		basist::ktx2_df_channel_id channel_id1) {
	if (is_uastc) {
		switch (channel_id0) {
			case basist::KTX2_DF_CHANNEL_UASTC_RGB: {
				return ChannelType::CHANNEL_RGB;
			} break;
			case basist::KTX2_DF_CHANNEL_UASTC_RGBA: {
				return ChannelType::CHANNEL_RGBA;
			} break;
			case basist::KTX2_DF_CHANNEL_UASTC_RRR: {
				return ChannelType::CHANNEL_R;
			} break;
			case basist::KTX2_DF_CHANNEL_UASTC_RRRG: {
				return ChannelType::CHANNEL_RG;
			} break;
			case basist::KTX2_DF_CHANNEL_UASTC_RG: {
				return ChannelType::CHANNEL_RG;
			} break;
			default: {
				return ChannelType::CHANNEL_RGBA;
			}
		}
	} else {
		if (channel_id0 == basist::KTX2_DF_CHANNEL_ETC1S_RGB &&
				channel_id1 != basist::KTX2_DF_CHANNEL_ETC1S_AAA) {
			return ChannelType::CHANNEL_RGB;
		} else if (channel_id0 == basist::KTX2_DF_CHANNEL_ETC1S_RGB &&
				channel_id1 == basist::KTX2_DF_CHANNEL_ETC1S_AAA) {
			return ChannelType::CHANNEL_RGBA;
		} else if (channel_id0 == basist::KTX2_DF_CHANNEL_ETC1S_RRR &&
				channel_id1 != basist::KTX2_DF_CHANNEL_ETC1S_GGG) {
			return ChannelType::CHANNEL_R;
		} else if (channel_id0 == basist::KTX2_DF_CHANNEL_ETC1S_RRR &&
				channel_id1 == basist::KTX2_DF_CHANNEL_ETC1S_GGG) {
			return ChannelType::CHANNEL_RG;
		}
		return ChannelType::CHANNEL_RGBA;
	}
}

static basist::transcoder_texture_format get_target_texture_format(
		basist::basis_tex_format basis_format, ChannelType channel_type,
		TextureCompressionMethod supported_compressed_formats) {
	switch (basis_format) {
		case basist::basis_tex_format::cUASTC_HDR_4x4: {
			if (supported_compressed_formats & TextureCompressionMethod::ASTC_HDR) {
				return basist::transcoder_texture_format::cTFASTC_HDR_4x4_RGBA;
			} else if (supported_compressed_formats & TextureCompressionMethod::BC) {
				return basist::transcoder_texture_format::cTFBC6H;
			} else {
				return basist::transcoder_texture_format::cTFRGBA_HALF;
			}
		} break;

		case basist::basis_tex_format::cASTC_HDR_6x6_INTERMEDIATE:
		case basist::basis_tex_format::cASTC_HDR_6x6: {
			if (supported_compressed_formats & TextureCompressionMethod::ASTC_HDR) {
				return basist::transcoder_texture_format::cTFASTC_HDR_6x6_RGBA;
			} else if (supported_compressed_formats & TextureCompressionMethod::BC) {
				return basist::transcoder_texture_format::cTFBC6H;
			} else {
				return basist::transcoder_texture_format::cTFRGBA_HALF;
			}
		} break;

		case basist::basis_tex_format::cUASTC4x4: {
			if (supported_compressed_formats & TextureCompressionMethod::ASTC_LDR) {
				return basist::transcoder_texture_format::cTFASTC_4x4_RGBA;
			} else if (supported_compressed_formats & TextureCompressionMethod::BC) {
				return basist::transcoder_texture_format::cTFBC7_RGBA;
			} else if (supported_compressed_formats & TextureCompressionMethod::ETC2) {
				switch (channel_type) {
					case CHANNEL_RGB: {
						return basist::transcoder_texture_format::cTFETC1_RGB;
					} break;
					case CHANNEL_RGBA: {
						return basist::transcoder_texture_format::cTFETC2_RGBA;
					} break;
					case CHANNEL_R: {
						return basist::transcoder_texture_format::cTFETC2_EAC_R11;
					} break;
					case CHANNEL_RG: {
						return basist::transcoder_texture_format::cTFETC2_EAC_RG11;
					} break;
				}
			} else {
				return basist::transcoder_texture_format::cTFRGBA32;
			}
		} break;
		case basist::basis_tex_format::cETC1S: {
			if (supported_compressed_formats & TextureCompressionMethod::ETC2) {
				switch (channel_type) {
					case CHANNEL_RGB: {
						return basist::transcoder_texture_format::cTFETC1_RGB;
					} break;
					case CHANNEL_RGBA: {
						return basist::transcoder_texture_format::cTFETC2_RGBA;
					} break;
					case CHANNEL_R: {
						return basist::transcoder_texture_format::cTFETC2_EAC_R11;
					} break;
					case CHANNEL_RG: {
						return basist::transcoder_texture_format::cTFETC2_EAC_RG11;
					} break;
				}
			} else if (supported_compressed_formats & TextureCompressionMethod::BC) {
				return basist::transcoder_texture_format::cTFBC7_RGBA;
			} else {
				return basist::transcoder_texture_format::cTFRGBA32;
			}
		} break;
		default:
			return basist::transcoder_texture_format::cTFTotalTextureFormats;
	}
	return basist::transcoder_texture_format::cTFTotalTextureFormats;
}
