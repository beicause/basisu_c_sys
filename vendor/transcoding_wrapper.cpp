#include "transcoding_wrapper.hpp"
#include "basis_universal/transcoder/basisu_transcoder.h"

enum TextureCompressionMethod : unsigned char;

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
void c_basisu_transcoder_init() {
	basist::basisu_transcoder_init();
}

Transcoder *c_ktx2_transcoder_new() {
	basist::ktx2_transcoder *inner = new basist::ktx2_transcoder();
	Transcoder *transcoder = new Transcoder();
	transcoder->inner = inner;
	return transcoder;
}

void c_ktx2_transcoder_delete(Transcoder *transcoder) {
	if (transcoder->r_dst_buf) {
		free(transcoder->r_dst_buf);
		transcoder->r_dst_buf = nullptr;
		transcoder->r_dst_buf_len = 0;
	}
	delete transcoder;
}

static bool c_ktx2_transcoder_get_texture_info(Transcoder *transcoder, TextureTranscodedFormat target_format, unsigned int *r_width, unsigned int *r_height, unsigned int *r_levels, unsigned int *r_layers, unsigned int *r_faces, unsigned int *r_total_bytes) {
	basist::ktx2_transcoder *inner = transcoder->inner;

	*r_width = inner->get_width();
	*r_height = inner->get_height();
	*r_levels = inner->get_levels();
	*r_layers = inner->get_layers();
	*r_faces = inner->get_faces();

	const basist::transcoder_texture_format transcode_format = static_cast<basist::transcoder_texture_format>(static_cast<uint32_t>(target_format));

	uint32_t total_bytes = 0;
	uint32_t total_layers = basisu::maximumu(inner->get_layers(), 1u);
	for (uint32_t level_index = 0; level_index < inner->get_levels(); level_index++) {
		for (uint32_t layer_index = 0; layer_index < total_layers; layer_index++) {
			for (uint32_t face_index = 0; face_index < inner->get_faces(); face_index++) {
				basist::ktx2_image_level_info level_info;
				if (!inner->get_image_level_info(level_info, level_index, layer_index, face_index)) {
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

static void c_ktx2_transcoder_get_target_format(Transcoder *transcoder, TextureCompressionMethod supported_compressed_formats, bool *r_is_srgb, TextureTranscodedFormat *r_format) {
	basist::ktx2_transcoder *inner = transcoder->inner;

	basist::ktx2_df_channel_id channel_id0 = inner->get_dfd_channel_id0();
	basist::ktx2_df_channel_id channel_id1 = inner->get_dfd_channel_id1();
	basist::basis_tex_format basis_format = inner->get_basis_tex_format();
	ChannelType channel_type = channel_id_to_type(inner->is_uastc(), channel_id0, channel_id1);
	basist::transcoder_texture_format target_format = get_target_texture_format(basis_format, channel_type, supported_compressed_formats);
	*r_is_srgb = inner->get_dfd_transfer_func() == basist::KTX2_KHR_DF_TRANSFER_SRGB;
	*r_format = static_cast<TextureTranscodedFormat>(static_cast<uint32_t>(target_format));
}

bool c_ktx2_transcoder_transcode_image(
		Transcoder *transcoder, const unsigned char *data, unsigned int data_size,
		TextureCompressionMethod supported_compressed_formats) {
	basist::ktx2_transcoder *inner = transcoder->inner;
	inner->init(data, data_size);
	inner->start_transcoding();

	c_ktx2_transcoder_get_target_format(transcoder, supported_compressed_formats, &transcoder->r_is_srgb, &transcoder->r_target_format);
	const basist::transcoder_texture_format transcode_format = static_cast<basist::transcoder_texture_format>(static_cast<uint32_t>(transcoder->r_target_format));

	if (!c_ktx2_transcoder_get_texture_info(transcoder, transcoder->r_target_format, &transcoder->r_width, &transcoder->r_height, &transcoder->r_levels, &transcoder->r_layers, &transcoder->r_faces, &transcoder->r_dst_buf_len)) {
		return false;
	}
	transcoder->r_dst_buf = (unsigned char *)malloc(transcoder->r_dst_buf_len);

	uint32_t total_layers = basisu::maximumu(inner->get_layers(), 1u);
	uint8_t *out = transcoder->r_dst_buf;
	for (uint32_t level_index = 0; level_index < inner->get_levels(); level_index++) {
		for (uint32_t layer_index = 0; layer_index < total_layers; layer_index++) {
			for (uint32_t face_index = 0; face_index < inner->get_faces(); face_index++) {
				basist::ktx2_image_level_info level_info;
				if (!inner->get_image_level_info(level_info, level_index, layer_index, face_index)) {
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

				if (!inner->transcode_image_level(level_index, layer_index, face_index, out, total_dst_blocks_or_pixels, transcode_format)) {
					return false;
				}

				uint32_t total_bytes = basist::basis_compute_transcoded_image_size_in_bytes(transcode_format, level_info.m_orig_width, level_info.m_orig_height);
				out += total_bytes;
			}
		}
	}
	transcoder->inner->clear();

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

// Select target format according to https://github.com/KhronosGroup/3D-Formats-Guidelines/blob/main/KTXDeveloperGuide.md.
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
			// Prefer BC7 over ETC2 because on some desktop platforms ETC2 is really slow.
			if (supported_compressed_formats & TextureCompressionMethod::BC) {
				switch (channel_type) {
					case CHANNEL_RGB: {
						return basist::transcoder_texture_format::cTFBC7_RGBA;
					} break;
					case CHANNEL_RGBA: {
						return basist::transcoder_texture_format::cTFBC7_RGBA;
					} break;
					case CHANNEL_R: {
						return basist::transcoder_texture_format::cTFBC4_R;
					} break;
					case CHANNEL_RG: {
						return basist::transcoder_texture_format::cTFBC5_RG;
					} break;
				}
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
		default:
			return basist::transcoder_texture_format::cTFTotalTextureFormats;
	}
	return basist::transcoder_texture_format::cTFTotalTextureFormats;
}
unsigned char *c_ktx2_transcoder_get_r_dst_buf(Transcoder *transcoder) {
	return transcoder->r_dst_buf;
}
unsigned int c_ktx2_transcoder_get_r_dst_buf_len(Transcoder *transcoder) {
	return transcoder->r_dst_buf_len;
}
unsigned int c_ktx2_transcoder_get_r_width(Transcoder *transcoder) {
	return transcoder->r_width;
}
unsigned int c_ktx2_transcoder_get_r_height(Transcoder *transcoder) {
	return transcoder->r_height;
}
unsigned int c_ktx2_transcoder_get_r_levels(Transcoder *transcoder) {
	return transcoder->r_levels;
}
unsigned int c_ktx2_transcoder_get_r_layers(Transcoder *transcoder) {
	return transcoder->r_layers;
}
unsigned int c_ktx2_transcoder_get_r_faces(Transcoder *transcoder) {
	return transcoder->r_faces;
}
TextureTranscodedFormat c_ktx2_transcoder_get_r_target_format(Transcoder *transcoder) {
	return transcoder->r_target_format;
}
bool c_ktx2_transcoder_get_r_is_srgb(Transcoder *transcoder) {
	return transcoder->r_is_srgb;
}
