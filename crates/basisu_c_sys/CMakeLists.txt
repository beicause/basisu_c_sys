cmake_minimum_required(VERSION 3.10)
project(basisu_c_sys_wasm)

set(BUILD_ARGS_DIR "${CMAKE_CURRENT_SOURCE_DIR}/build_args" CACHE STRING "build args dir")

if (EMSCRIPTEN)

  set(CMAKE_CXX_STANDARD 17)

  file(STRINGS "${BUILD_ARGS_DIR}/encoder_srcs.txt" ENCODER_SRCS)
  file(STRINGS "${BUILD_ARGS_DIR}/encoder_compile_flags.txt" ENCODER_COMPILE_FLAGS)
  file(STRINGS "${BUILD_ARGS_DIR}/encoder_link_flags.txt" ENCODER_LINK_FLAGS)

  file(STRINGS "${BUILD_ARGS_DIR}/transcoder_srcs.txt" TRANSCODER_SRCS)
  file(STRINGS "${BUILD_ARGS_DIR}/transcoder_compile_flags.txt" TRANSCODER_COMPILE_FLAGS)
  file(STRINGS "${BUILD_ARGS_DIR}/transcoder_link_flags.txt" TRANSCODER_LINK_FLAGS)

  add_executable(encoder ${ENCODER_SRCS})
  add_executable(transcoder ${TRANSCODER_SRCS})

  target_compile_options(encoder PRIVATE ${ENCODER_COMPILE_FLAGS})
  target_compile_options(transcoder PRIVATE ${TRANSCODER_COMPILE_FLAGS})

  target_link_options(encoder PRIVATE ${ENCODER_LINK_FLAGS})
  target_link_options(transcoder PRIVATE ${TRANSCODER_LINK_FLAGS})

  set_target_properties(encoder PROPERTIES
    OUTPUT_NAME "basisu_encoder"
    SUFFIX ".js"
  )

  set_target_properties(transcoder PROPERTIES
    OUTPUT_NAME "basisu_transcoder"
    SUFFIX ".js"
  )

endif()
