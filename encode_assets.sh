#!/bin/sh

$BASISU_TOOL -etc1s -mipmap ./original_assets/alpha0.png -output_file ./assets/alpha0_etc1s_mips.basisu.ktx2
$BASISU_TOOL -uastc_hdr_4x4 ./original_assets/Desk_fixed_6x6.exr -output_file ./assets/desk_uastc_hdr_4x4.basisu.ktx2
$BASISU_TOOL -uastc_hdr_6x6 -mipmap ./original_assets/Desk_fixed_6x6.exr -output_file ./assets/desk_uastc_hdr_6x6_mips.basisu.ktx2
$BASISU_TOOL -xuastc_ldr_8x8 -mipmap -cubemap ./original_assets/skybox/right.jpg ./original_assets/skybox/left.jpg ./original_assets/skybox/top.jpg ./original_assets/skybox/bottom.jpg ./original_assets/skybox/front.jpg ./original_assets/skybox/back.jpg -output_file ./assets/skybox_xuastc_ldr_8x8_cubemap_mips.basisu.ktx2
$BASISU_TOOL -astc_ldr_8x8 -mipmap ./original_assets/kodim20.png -output_file ./assets/kodim20_astc_ldr_8x8_mips.basisu.ktx2
$BASISU_TOOL -uastc_ldr_4x4 ./original_assets/tough_fixed.png -output_file ./assets/tough_uastc_ldr_4x4.basisu.ktx2
$BASISU_TOOL -xuastc_ldr_6x6 -mipmap ./original_assets/wikipedia_fixed_6x6.png -output_file ./assets/wikipedia_xuastc_ldr_6x6_mips.basisu.ktx2
