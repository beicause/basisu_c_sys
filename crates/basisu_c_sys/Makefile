
CXX = em++
CXXFLAGS = -xc++ -std=c++17 -Wno-unused-variable -Wno-type-limits -Wno-unused-but-set-variable -Wno-unused-function -Wno-misleading-indentation -fno-exceptions -fno-strict-aliasing -DBASISD_SUPPORT_DXT1=0 -DBASISD_SUPPORT_PVRTC1=0 -DBASISD_SUPPORT_ATC=0 -DBASISD_SUPPORT_FXT1=0 -DBASISD_SUPPORT_PVRTC2=0 -sSTRICT -Os -msimd128 -flto=full

ENCODER_LINKFLAGS = -std=c++17 -Wno-unused-variable -Wno-type-limits -Wno-unused-but-set-variable -Wno-unused-function -Wno-misleading-indentation -fno-exceptions -fno-strict-aliasing -DBASISD_SUPPORT_DXT1=0 -DBASISD_SUPPORT_PVRTC1=0 -DBASISD_SUPPORT_ATC=0 -DBASISD_SUPPORT_FXT1=0 -DBASISD_SUPPORT_PVRTC2=0 -sSTRICT -sEXPORT_ES6 -sINCOMING_MODULE_JS_API=wasmBinary -sALLOW_MEMORY_GROWTH -sEXPORTED_RUNTIME_METHODS=HEAPU8 -sEXPORTED_FUNCTIONS=_bu_get_version,_bu_enable_debug_printf,_bu_init,_bu_alloc,_bu_free,_bu_new_comp_params,_bu_delete_comp_params,_bu_comp_params_get_comp_data_size,_bu_comp_params_get_comp_data_ofs,_bu_comp_params_clear,_bu_comp_params_set_image_rgba32,_bu_comp_params_set_image_float_rgba,_bu_compress_texture,_bt_get_version,_bt_enable_debug_printf,_bt_init,_bt_alloc,_bt_free,_bt_basis_tex_format_is_xuastc_ldr,_bt_basis_tex_format_is_astc_ldr,_bt_basis_tex_format_get_block_width,_bt_basis_tex_format_get_block_height,_bt_basis_tex_format_is_hdr,_bt_basis_tex_format_is_ldr,_bt_basis_get_bytes_per_block_or_pixel,_bt_basis_transcoder_format_has_alpha,_bt_basis_transcoder_format_is_hdr,_bt_basis_transcoder_format_is_ldr,_bt_basis_transcoder_texture_format_is_astc,_bt_basis_transcoder_format_is_uncompressed,_bt_basis_get_uncompressed_bytes_per_pixel,_bt_basis_get_block_width,_bt_basis_get_block_height,_bt_basis_get_transcoder_texture_format_from_basis_tex_format,_bt_basis_is_format_supported,_bt_basis_compute_transcoded_image_size_in_bytes,_bt_ktx2_open,_bt_ktx2_close,_bt_ktx2_get_width,_bt_ktx2_get_height,_bt_ktx2_get_levels,_bt_ktx2_get_faces,_bt_ktx2_get_layers,_bt_ktx2_get_basis_tex_format,_bt_ktx2_is_etc1s,_bt_ktx2_is_uastc_ldr_4x4,_bt_ktx2_is_hdr,_bt_ktx2_is_hdr_4x4,_bt_ktx2_is_hdr_6x6,_bt_ktx2_is_ldr,_bt_ktx2_is_astc_ldr,_bt_ktx2_is_xuastc_ldr,_bt_ktx2_get_block_width,_bt_ktx2_get_block_height,_bt_ktx2_has_alpha,_bt_ktx2_get_dfd_color_model,_bt_ktx2_get_dfd_color_primaries,_bt_ktx2_get_dfd_transfer_func,_bt_ktx2_is_srgb,_bt_ktx2_get_dfd_flags,_bt_ktx2_get_dfd_total_samples,_bt_ktx2_get_dfd_channel_id0,_bt_ktx2_get_dfd_channel_id1,_bt_ktx2_is_video,_bt_ktx2_get_ldr_hdr_upconversion_nit_multiplier,_bt_ktx2_get_level_orig_width,_bt_ktx2_get_level_orig_height,_bt_ktx2_get_level_actual_width,_bt_ktx2_get_level_actual_height,_bt_ktx2_get_level_num_blocks_x,_bt_ktx2_get_level_num_blocks_y,_bt_ktx2_get_level_total_blocks,_bt_ktx2_get_level_alpha_flag,_bt_ktx2_get_level_iframe_flag,_bt_ktx2_start_transcoding,_bt_ktx2_create_transcode_state,_bt_ktx2_destroy_transcode_state,_bt_ktx2_transcode_image_level -Os -msimd128 -flto=full -sEVAL_CTORS
TRANSCODER_LINKFLAGS = -std=c++17 -Wno-unused-variable -Wno-type-limits -Wno-unused-but-set-variable -Wno-unused-function -Wno-misleading-indentation -fno-exceptions -fno-strict-aliasing -DBASISD_SUPPORT_DXT1=0 -DBASISD_SUPPORT_PVRTC1=0 -DBASISD_SUPPORT_ATC=0 -DBASISD_SUPPORT_FXT1=0 -DBASISD_SUPPORT_PVRTC2=0 -sSTRICT -sEXPORT_ES6 -sINCOMING_MODULE_JS_API=wasmBinary -sALLOW_MEMORY_GROWTH -sEXPORTED_RUNTIME_METHODS=HEAPU8 -sEXPORTED_FUNCTIONS=_bt_get_version,_bt_enable_debug_printf,_bt_init,_bt_alloc,_bt_free,_bt_basis_tex_format_is_xuastc_ldr,_bt_basis_tex_format_is_astc_ldr,_bt_basis_tex_format_get_block_width,_bt_basis_tex_format_get_block_height,_bt_basis_tex_format_is_hdr,_bt_basis_tex_format_is_ldr,_bt_basis_get_bytes_per_block_or_pixel,_bt_basis_transcoder_format_has_alpha,_bt_basis_transcoder_format_is_hdr,_bt_basis_transcoder_format_is_ldr,_bt_basis_transcoder_texture_format_is_astc,_bt_basis_transcoder_format_is_uncompressed,_bt_basis_get_uncompressed_bytes_per_pixel,_bt_basis_get_block_width,_bt_basis_get_block_height,_bt_basis_get_transcoder_texture_format_from_basis_tex_format,_bt_basis_is_format_supported,_bt_basis_compute_transcoded_image_size_in_bytes,_bt_ktx2_open,_bt_ktx2_close,_bt_ktx2_get_width,_bt_ktx2_get_height,_bt_ktx2_get_levels,_bt_ktx2_get_faces,_bt_ktx2_get_layers,_bt_ktx2_get_basis_tex_format,_bt_ktx2_is_etc1s,_bt_ktx2_is_uastc_ldr_4x4,_bt_ktx2_is_hdr,_bt_ktx2_is_hdr_4x4,_bt_ktx2_is_hdr_6x6,_bt_ktx2_is_ldr,_bt_ktx2_is_astc_ldr,_bt_ktx2_is_xuastc_ldr,_bt_ktx2_get_block_width,_bt_ktx2_get_block_height,_bt_ktx2_has_alpha,_bt_ktx2_get_dfd_color_model,_bt_ktx2_get_dfd_color_primaries,_bt_ktx2_get_dfd_transfer_func,_bt_ktx2_is_srgb,_bt_ktx2_get_dfd_flags,_bt_ktx2_get_dfd_total_samples,_bt_ktx2_get_dfd_channel_id0,_bt_ktx2_get_dfd_channel_id1,_bt_ktx2_is_video,_bt_ktx2_get_ldr_hdr_upconversion_nit_multiplier,_bt_ktx2_get_level_orig_width,_bt_ktx2_get_level_orig_height,_bt_ktx2_get_level_actual_width,_bt_ktx2_get_level_actual_height,_bt_ktx2_get_level_num_blocks_x,_bt_ktx2_get_level_num_blocks_y,_bt_ktx2_get_level_total_blocks,_bt_ktx2_get_level_alpha_flag,_bt_ktx2_get_level_iframe_flag,_bt_ktx2_start_transcoding,_bt_ktx2_create_transcode_state,_bt_ktx2_destroy_transcode_state,_bt_ktx2_transcode_image_level -Os -msimd128 -flto=full -sEVAL_CTORS


ENCODER_SOURCES = vendor/basis_universal/encoder/basisu_astc_hdr_6x6_enc.cpp vendor/basis_universal/encoder/basisu_astc_hdr_common.cpp vendor/basis_universal/encoder/basisu_astc_ldr_common.cpp vendor/basis_universal/encoder/basisu_astc_ldr_encode.cpp vendor/basis_universal/encoder/basisu_backend.cpp vendor/basis_universal/encoder/basisu_basis_file.cpp vendor/basis_universal/encoder/basisu_bc7enc.cpp vendor/basis_universal/encoder/basisu_comp.cpp vendor/basis_universal/encoder/basisu_enc.cpp vendor/basis_universal/encoder/basisu_etc.cpp vendor/basis_universal/encoder/basisu_frontend.cpp vendor/basis_universal/encoder/basisu_gpu_texture.cpp vendor/basis_universal/encoder/basisu_kernels_sse.cpp vendor/basis_universal/encoder/basisu_opencl.cpp vendor/basis_universal/encoder/basisu_pvrtc1_4.cpp vendor/basis_universal/encoder/basisu_resample_filters.cpp vendor/basis_universal/encoder/basisu_resampler.cpp vendor/basis_universal/encoder/basisu_ssim.cpp vendor/basis_universal/encoder/basisu_uastc_enc.cpp vendor/basis_universal/encoder/basisu_uastc_hdr_4x4_enc.cpp vendor/basis_universal/encoder/basisu_wasm_api.cpp vendor/basis_universal/encoder/basisu_wasm_transcoder_api.cpp vendor/basis_universal/encoder/jpgd.cpp vendor/basis_universal/encoder/pvpngreader.cpp vendor/basis_universal/encoder/3rdparty/android_astc_decomp.cpp vendor/basis_universal/encoder/3rdparty/tinyexr.cpp vendor/basis_universal/transcoder/basisu_transcoder.cpp vendor/basis_universal/zstd/zstd.c
TRANSCODER_SOURCES = vendor/basis_universal/encoder/basisu_wasm_transcoder_api.cpp vendor/basis_universal/transcoder/basisu_transcoder.cpp vendor/basis_universal/zstd/zstddeclib.c

ENCODER_BUILD_DIR = build/encoder
ENCODER_OBJECT_FILENAMES = $(patsubst %.c, %.o, $(patsubst %.cpp, %.o, $(notdir $(ENCODER_SOURCES))))
ENCODER_OBJECTS = $(addprefix $(ENCODER_BUILD_DIR)/, $(ENCODER_OBJECT_FILENAMES))

TRANSCODER_BUILD_DIR = build/transcoder
TRANSCODER_OBJECT_FILENAMES = $(patsubst %.c, %.o, $(patsubst %.cpp, %.o, $(notdir $(TRANSCODER_SOURCES))))
TRANSCODER_OBJECTS = $(addprefix $(TRANSCODER_BUILD_DIR)/, $(TRANSCODER_OBJECT_FILENAMES))

all: wasm/basisu_encoder.wasm wasm/basisu_encoder.js wasm/basisu_transcoder.wasm wasm/basisu_transcoder.js

.PHONY: clean
clean:
	rm -rf build

build_dir:
	mkdir -p build/encoder
	mkdir -p build/transcoder

build/basisu_encoder.wasm: $(ENCODER_OBJECTS) | build_dir
	$(CXX) $(ENCODER_LINKFLAGS) $^ -o $(patsubst %.wasm, %.js, $@)

build/basisu_transcoder.wasm: $(TRANSCODER_OBJECTS) | build_dir
	$(CXX) $(TRANSCODER_LINKFLAGS) $^ -o $(patsubst %.wasm, %.js, $@)

build/encoder/basisu_astc_hdr_6x6_enc.o: vendor/basis_universal/encoder/basisu_astc_hdr_6x6_enc.cpp | build_dir
	$(CXX) $(CXXFLAGS) -c $< -o $@

build/encoder/basisu_astc_hdr_common.o: vendor/basis_universal/encoder/basisu_astc_hdr_common.cpp | build_dir
	$(CXX) $(CXXFLAGS) -c $< -o $@

build/encoder/basisu_astc_ldr_common.o: vendor/basis_universal/encoder/basisu_astc_ldr_common.cpp | build_dir
	$(CXX) $(CXXFLAGS) -c $< -o $@

build/encoder/basisu_astc_ldr_encode.o: vendor/basis_universal/encoder/basisu_astc_ldr_encode.cpp | build_dir
	$(CXX) $(CXXFLAGS) -c $< -o $@

build/encoder/basisu_backend.o: vendor/basis_universal/encoder/basisu_backend.cpp | build_dir
	$(CXX) $(CXXFLAGS) -c $< -o $@

build/encoder/basisu_basis_file.o: vendor/basis_universal/encoder/basisu_basis_file.cpp | build_dir
	$(CXX) $(CXXFLAGS) -c $< -o $@

build/encoder/basisu_bc7enc.o: vendor/basis_universal/encoder/basisu_bc7enc.cpp | build_dir
	$(CXX) $(CXXFLAGS) -c $< -o $@

build/encoder/basisu_comp.o: vendor/basis_universal/encoder/basisu_comp.cpp | build_dir
	$(CXX) $(CXXFLAGS) -c $< -o $@

build/encoder/basisu_enc.o: vendor/basis_universal/encoder/basisu_enc.cpp | build_dir
	$(CXX) $(CXXFLAGS) -c $< -o $@

build/encoder/basisu_etc.o: vendor/basis_universal/encoder/basisu_etc.cpp | build_dir
	$(CXX) $(CXXFLAGS) -c $< -o $@

build/encoder/basisu_frontend.o: vendor/basis_universal/encoder/basisu_frontend.cpp | build_dir
	$(CXX) $(CXXFLAGS) -c $< -o $@

build/encoder/basisu_gpu_texture.o: vendor/basis_universal/encoder/basisu_gpu_texture.cpp | build_dir
	$(CXX) $(CXXFLAGS) -c $< -o $@

build/encoder/basisu_kernels_sse.o: vendor/basis_universal/encoder/basisu_kernels_sse.cpp | build_dir
	$(CXX) $(CXXFLAGS) -c $< -o $@

build/encoder/basisu_opencl.o: vendor/basis_universal/encoder/basisu_opencl.cpp | build_dir
	$(CXX) $(CXXFLAGS) -c $< -o $@

build/encoder/basisu_pvrtc1_4.o: vendor/basis_universal/encoder/basisu_pvrtc1_4.cpp | build_dir
	$(CXX) $(CXXFLAGS) -c $< -o $@

build/encoder/basisu_resample_filters.o: vendor/basis_universal/encoder/basisu_resample_filters.cpp | build_dir
	$(CXX) $(CXXFLAGS) -c $< -o $@

build/encoder/basisu_resampler.o: vendor/basis_universal/encoder/basisu_resampler.cpp | build_dir
	$(CXX) $(CXXFLAGS) -c $< -o $@

build/encoder/basisu_ssim.o: vendor/basis_universal/encoder/basisu_ssim.cpp | build_dir
	$(CXX) $(CXXFLAGS) -c $< -o $@

build/encoder/basisu_uastc_enc.o: vendor/basis_universal/encoder/basisu_uastc_enc.cpp | build_dir
	$(CXX) $(CXXFLAGS) -c $< -o $@

build/encoder/basisu_uastc_hdr_4x4_enc.o: vendor/basis_universal/encoder/basisu_uastc_hdr_4x4_enc.cpp | build_dir
	$(CXX) $(CXXFLAGS) -c $< -o $@

build/encoder/basisu_wasm_api.o: vendor/basis_universal/encoder/basisu_wasm_api.cpp | build_dir
	$(CXX) $(CXXFLAGS) -c $< -o $@

build/encoder/basisu_wasm_transcoder_api.o: vendor/basis_universal/encoder/basisu_wasm_transcoder_api.cpp | build_dir
	$(CXX) $(CXXFLAGS) -c $< -o $@

build/encoder/jpgd.o: vendor/basis_universal/encoder/jpgd.cpp | build_dir
	$(CXX) $(CXXFLAGS) -c $< -o $@

build/encoder/pvpngreader.o: vendor/basis_universal/encoder/pvpngreader.cpp | build_dir
	$(CXX) $(CXXFLAGS) -c $< -o $@

build/encoder/android_astc_decomp.o: vendor/basis_universal/encoder/3rdparty/android_astc_decomp.cpp | build_dir
	$(CXX) $(CXXFLAGS) -c $< -o $@

build/encoder/tinyexr.o: vendor/basis_universal/encoder/3rdparty/tinyexr.cpp | build_dir
	$(CXX) $(CXXFLAGS) -c $< -o $@

build/encoder/basisu_transcoder.o: vendor/basis_universal/transcoder/basisu_transcoder.cpp | build_dir
	$(CXX) $(CXXFLAGS) -c $< -o $@

build/encoder/zstd.o: vendor/basis_universal/zstd/zstd.c | build_dir
	$(CXX) $(CXXFLAGS) -c $< -o $@

build/transcoder/basisu_wasm_transcoder_api.o: vendor/basis_universal/encoder/basisu_wasm_transcoder_api.cpp | build_dir
	$(CXX) $(CXXFLAGS) -c $< -o $@

build/transcoder/basisu_transcoder.o: vendor/basis_universal/transcoder/basisu_transcoder.cpp | build_dir
	$(CXX) $(CXXFLAGS) -c $< -o $@

build/transcoder/zstddeclib.o: vendor/basis_universal/zstd/zstddeclib.c | build_dir
	$(CXX) $(CXXFLAGS) -c $< -o $@

WASM_OPT_FLAGS = -Os --enable-simd --enable-bulk-memory-opt --enable-nontrapping-float-to-int

wasm/basisu_encoder.wasm: build/basisu_encoder.wasm
	wasm-opt $(WASM_OPT_FLAGS) $^ -o $@

wasm/basisu_transcoder.wasm: build/basisu_transcoder.wasm
	wasm-opt $(WASM_OPT_FLAGS) $^ -o $@
        
wasm/basisu_encoder.js: build/basisu_encoder.wasm
	cp $(patsubst %.wasm, %.js, $^) $@

wasm/basisu_transcoder.js: build/basisu_transcoder.wasm
	cp $(patsubst %.wasm, %.js, $^) $@
