
namespace basist {
class ktx2_transcoder;
} //namespace basist

extern "C" {

// This enum must be in sync with the `basist::transcoder_texture_format`.
enum class TextureTranscodedFormat {
	// Compressed formats

	// ETC1-2
	cTFETC1_RGB = 0, // Opaque only, returns RGB or alpha data if cDecodeFlagsTranscodeAlphaDataToOpaqueFormats flag is specified
	cTFETC2_RGBA = 1, // Opaque+alpha, ETC2_EAC_A8 block followed by a ETC1 block, alpha channel will be opaque for opaque .basis files

	// BC1-5, BC7 (desktop, some mobile devices)
	cTFBC1_RGB = 2, // Opaque only, no punchthrough alpha support yet, transcodes alpha slice if cDecodeFlagsTranscodeAlphaDataToOpaqueFormats flag is specified
	cTFBC3_RGBA = 3, // Opaque+alpha, BC4 followed by a BC1 block, alpha channel will be opaque for opaque .basis files
	cTFBC4_R = 4, // Red only, alpha slice is transcoded to output if cDecodeFlagsTranscodeAlphaDataToOpaqueFormats flag is specified
	cTFBC5_RG = 5, // XY: Two BC4 blocks, X=R and Y=Alpha, .basis file should have alpha data (if not Y will be all 255's)
	cTFBC7_RGBA = 6, // RGB or RGBA, mode 5 for ETC1S, modes (1,2,3,5,6,7) for UASTC

	// PVRTC1 4bpp (mobile, PowerVR devices)
	cTFPVRTC1_4_RGB = 8, // Opaque only, RGB or alpha if cDecodeFlagsTranscodeAlphaDataToOpaqueFormats flag is specified, nearly lowest quality of any texture format.
	cTFPVRTC1_4_RGBA = 9, // Opaque+alpha, most useful for simple opacity maps. If .basis file doesn't have alpha cTFPVRTC1_4_RGB will be used instead. Lowest quality of any supported texture format.

	// ASTC (mobile, Intel devices, hopefully all desktop GPU's one day)
	cTFASTC_4x4_RGBA = 10, // LDR. Opaque+alpha, ASTC 4x4, alpha channel will be opaque for opaque .basis files.
	// LDR: Transcoder uses RGB/RGBA/L/LA modes, void extent, and up to two ([0,47] and [0,255]) endpoint precisions.

	// ATC (mobile, Adreno devices, this is a niche format)
	cTFATC_RGB = 11, // Opaque, RGB or alpha if cDecodeFlagsTranscodeAlphaDataToOpaqueFormats flag is specified. ATI ATC (GL_ATC_RGB_AMD)
	cTFATC_RGBA = 12, // Opaque+alpha, alpha channel will be opaque for opaque .basis files. ATI ATC (GL_ATC_RGBA_INTERPOLATED_ALPHA_AMD)

	// FXT1 (desktop, Intel devices, this is a super obscure format)
	cTFFXT1_RGB = 17, // Opaque only, uses exclusively CC_MIXED blocks. Notable for having a 8x4 block size. GL_3DFX_texture_compression_FXT1 is supported on Intel integrated GPU's (such as HD 630).
	// Punch-through alpha is relatively easy to support, but full alpha is harder. This format is only here for completeness so opaque-only is fine for now.
	// See the BASISU_USE_ORIGINAL_3DFX_FXT1_ENCODING macro in basisu_transcoder_internal.h.

	cTFPVRTC2_4_RGB = 18, // Opaque-only, almost BC1 quality, much faster to transcode and supports arbitrary texture dimensions (unlike PVRTC1 RGB).
	cTFPVRTC2_4_RGBA = 19, // Opaque+alpha, slower to encode than cTFPVRTC2_4_RGB. Premultiplied alpha is highly recommended, otherwise the color channel can leak into the alpha channel on transparent blocks.

	cTFETC2_EAC_R11 = 20, // R only (ETC2 EAC R11 unsigned)
	cTFETC2_EAC_RG11 = 21, // RG only (ETC2 EAC RG11 unsigned), R=opaque.r, G=alpha - for tangent space normal maps

	cTFBC6H = 22, // HDR, RGB only, unsigned
	cTFASTC_HDR_4x4_RGBA = 23, // HDR, RGBA (currently UASTC HDR 4x4 encoders are only RGB), unsigned

	// Uncompressed (raw pixel) formats
	// Note these uncompressed formats (RGBA32, 565, and 4444) can only be transcoded to from LDR input files (ETC1S or UASTC LDR).
	cTFRGBA32 = 13, // 32bpp RGBA image stored in raster (not block) order in memory, R is first byte, A is last byte.
	cTFRGB565 = 14, // 16bpp RGB image stored in raster (not block) order in memory, R at bit position 11
	cTFBGR565 = 15, // 16bpp RGB image stored in raster (not block) order in memory, R at bit position 0
	cTFRGBA4444 = 16, // 16bpp RGBA image stored in raster (not block) order in memory, R at bit position 12, A at bit position 0

	// Note these uncompressed formats (HALF and 9E5) can only be transcoded to from HDR input files (UASTC HDR 4x4 or ASTC HDR 6x6).
	cTFRGB_HALF = 24, // 48bpp RGB half (16-bits/component, 3 components)
	cTFRGBA_HALF = 25, // 64bpp RGBA half (16-bits/component, 4 components) (A will always currently 1.0, UASTC_HDR doesn't support alpha)
	cTFRGB_9E5 = 26, // 32bpp RGB 9E5 (shared exponent, positive only, see GL_EXT_texture_shared_exponent)

	cTFASTC_HDR_6x6_RGBA = 27, // HDR, RGBA (currently our ASTC HDR 6x6 encodes are only RGB), unsigned

	cTFTotalTextureFormats = 28,

	// ----- The following are old/legacy enums for compatibility with code compiled against previous versions
	cTFETC1 = cTFETC1_RGB,
	cTFETC2 = cTFETC2_RGBA,
	cTFBC1 = cTFBC1_RGB,
	cTFBC3 = cTFBC3_RGBA,
	cTFBC4 = cTFBC4_R,
	cTFBC5 = cTFBC5_RG,

	// Previously, the caller had some control over which BC7 mode the transcoder output. We've simplified this due to UASTC, which supports numerous modes.
	cTFBC7_M6_RGB = cTFBC7_RGBA, // Opaque only, RGB or alpha if cDecodeFlagsTranscodeAlphaDataToOpaqueFormats flag is specified. Highest quality of all the non-ETC1 formats.
	cTFBC7_M5_RGBA = cTFBC7_RGBA, // Opaque+alpha, alpha channel will be opaque for opaque .basis files
	cTFBC7_M6_OPAQUE_ONLY = cTFBC7_RGBA,
	cTFBC7_M5 = cTFBC7_RGBA,
	cTFBC7_ALT = 7,

	cTFASTC_4x4 = cTFASTC_4x4_RGBA,

	cTFATC_RGBA_INTERPOLATED_ALPHA = cTFATC_RGBA,
};

enum TextureCompressionMethod {
	NONE = 0,
	ASTC_LDR = 1 << 0,
	ASTC_HDR = 1 << 1,
	BC = 1 << 2,
	ETC2 = 1 << 3
};

void basisu_transcoder_init();

basist::ktx2_transcoder *ktx2_transcoder_new();

void ktx2_transcoder_delete(basist::ktx2_transcoder *transcoder);

bool ktx2_transcoder_start(basist::ktx2_transcoder *transcoder);

void ktx2_transcoder_clear(basist::ktx2_transcoder *transcoder);

bool ktx2_transcoder_init(basist::ktx2_transcoder *transcoder, const unsigned char *data, unsigned int data_size);

bool ktx2_transcoder_get_texture_info(basist::ktx2_transcoder *transcoder, TextureTranscodedFormat target_format, unsigned int *r_width, unsigned int *r_height, unsigned int *r_levels, unsigned int *r_layers, unsigned int *r_faces, unsigned int *r_total_bytes);

TextureTranscodedFormat ktx2_transcoder_get_transcoded_format(basist::ktx2_transcoder *transcoder, TextureCompressionMethod supported_compressed_formats, bool *r_is_srgb);

bool ktx2_transcoder_transcode_image(
		basist::ktx2_transcoder *transcoder,
		const unsigned char *data, unsigned int data_size, TextureTranscodedFormat target_format, unsigned char *r_dst_data, unsigned int r_dst_data_size);
}
