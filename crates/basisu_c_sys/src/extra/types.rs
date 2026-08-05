/// ASTC block dimensions
#[repr(C)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AstcBlock {
    /// 4x4 block compressed texture. 16 bytes per block (8 bit/px).
    B4x4,
    /// 5x4 block compressed texture. 16 bytes per block (6.4 bit/px).
    B5x4,
    /// 5x5 block compressed texture. 16 bytes per block (5.12 bit/px).
    B5x5,
    /// 6x5 block compressed texture. 16 bytes per block (4.27 bit/px).
    B6x5,
    /// 6x6 block compressed texture. 16 bytes per block (3.56 bit/px).
    B6x6,
    /// 8x5 block compressed texture. 16 bytes per block (3.2 bit/px).
    B8x5,
    /// 8x6 block compressed texture. 16 bytes per block (2.67 bit/px).
    B8x6,
    /// 8x8 block compressed texture. 16 bytes per block (2 bit/px).
    B8x8,
    /// 10x5 block compressed texture. 16 bytes per block (2.56 bit/px).
    B10x5,
    /// 10x6 block compressed texture. 16 bytes per block (2.13 bit/px).
    B10x6,
    /// 10x8 block compressed texture. 16 bytes per block (1.6 bit/px).
    B10x8,
    /// 10x10 block compressed texture. 16 bytes per block (1.28 bit/px).
    B10x10,
    /// 12x10 block compressed texture. 16 bytes per block (1.07 bit/px).
    B12x10,
    /// 12x12 block compressed texture. 16 bytes per block (0.89 bit/px).
    B12x12,
}

/// ASTC RGBA channel
#[repr(C)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AstcChannel {
    /// 8 bit integer RGBA, [0, 255] converted to/from linear-color float [0, 1] in shader.
    ///
    /// [`Features::TEXTURE_COMPRESSION_ASTC`] must be enabled to use this channel.
    Unorm,
    /// 8 bit integer RGBA, Srgb-color [0, 255] converted to/from linear-color float [0, 1] in shader.
    ///
    /// [`Features::TEXTURE_COMPRESSION_ASTC`] must be enabled to use this channel.
    UnormSrgb,
    /// floating-point RGBA, linear-color float can be outside of the [0, 1] range.
    ///
    /// [`Features::TEXTURE_COMPRESSION_ASTC_HDR`] must be enabled to use this channel.
    Hdr,
}

/// Dimensions of a particular texture view.
///
/// Corresponds to [WebGPU `GPUTextureViewDimension`](
/// https://gpuweb.github.io/gpuweb/#enumdef-gputextureviewdimension).
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TextureViewDimension {
    // /// A one dimensional texture. `texture_1d` in WGSL and `texture1D` in GLSL.
    // #[cfg_attr(feature = "serde", serde(rename = "1d"))]
    // D1,
    /// A two dimensional texture. `texture_2d` in WGSL and `texture2D` in GLSL.
    #[cfg_attr(feature = "serde", serde(rename = "2d"))]
    #[default]
    D2,
    /// A two dimensional array texture. `texture_2d_array` in WGSL and `texture2DArray` in GLSL.
    #[cfg_attr(feature = "serde", serde(rename = "2d-array"))]
    D2Array,
    /// A cubemap texture. `texture_cube` in WGSL and `textureCube` in GLSL.
    #[cfg_attr(feature = "serde", serde(rename = "cube"))]
    Cube,
    /// A cubemap array texture. `texture_cube_array` in WGSL and `textureCubeArray` in GLSL.
    #[cfg_attr(feature = "serde", serde(rename = "cube-array"))]
    CubeArray,
    // /// A three dimensional texture. `texture_3d` in WGSL and `texture3D` in GLSL.
    // #[cfg_attr(feature = "serde", serde(rename = "3d"))]
    // D3,
}

/// Extent of a texture related operation.
///
/// Corresponds to [WebGPU `GPUExtent3D`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpuextent3ddict).
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Extent3d {
    /// Width of the extent
    pub width: u32,
    /// Height of the extent
    pub height: u32,
    /// The depth of the extent or the number of array layers
    #[cfg_attr(feature = "serde", serde(default = "default_depth"))]
    pub depth_or_array_layers: u32,
}

#[cfg(feature = "serde")]
fn default_depth() -> u32 {
    1
}

impl core::fmt::Debug for Extent3d {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        (self.width, self.height, self.depth_or_array_layers).fmt(f)
    }
}

/// Format in which a texture’s texels are stored in GPU memory.
///
/// Certain formats additionally specify a conversion.
/// When these formats are used in a shader, the conversion automatically takes place when loading
/// from or storing to the texture.
///
/// * `Unorm` formats linearly scale the integer range of the storage format to a floating-point
///   range of 0 to 1, inclusive.
/// * `Snorm` formats linearly scale the integer range of the storage format to a floating-point
///   range of &minus;1 to 1, inclusive, except that the most negative value
///   (&minus;128 for 8-bit, &minus;32768 for 16-bit) is excluded; on conversion,
///   it is treated as identical to the second most negative
///   (&minus;127 for 8-bit, &minus;32767 for 16-bit),
///   so that the positive and negative ranges are symmetric.
/// * `UnormSrgb` formats apply the [sRGB transfer function] so that the storage is sRGB encoded
///   while the shader works with linear intensity values.
/// * `Uint`, `Sint`, and `Float` formats perform no conversion.
///
/// Corresponds to [WebGPU `GPUTextureFormat`](
/// https://gpuweb.github.io/gpuweb/#enumdef-gputextureformat).
///
/// [sRGB transfer function]: https://en.wikipedia.org/wiki/SRGB#Transfer_function_(%22gamma%22)
#[repr(C)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum TextureFormat {
    // Normal 8 bit formats
    /// Red channel only. 8 bit integer per channel. [0, 255] converted to/from float [0, 1] in shader.
    R8Unorm,
    // /// Red channel only. 8 bit integer per channel. [&minus;127, 127] converted to/from float [&minus;1, 1] in shader.
    // R8Snorm,
    // /// Red channel only. 8 bit integer per channel. Unsigned in shader.
    // R8Uint,
    // /// Red channel only. 8 bit integer per channel. Signed in shader.
    // R8Sint,

    // // Normal 16 bit formats
    // /// Red channel only. 16 bit integer per channel. Unsigned in shader.
    // R16Uint,
    // /// Red channel only. 16 bit integer per channel. Signed in shader.
    // R16Sint,
    // /// Red channel only. 16 bit integer per channel. [0, 65535] converted to/from float [0, 1] in shader.
    // ///
    // /// [`Features::TEXTURE_FORMAT_16BIT_NORM`] must be enabled to use this texture format.
    // R16Unorm,
    // /// Red channel only. 16 bit integer per channel. [&minus;32767, 32767] converted to/from float [&minus;1, 1] in shader.
    // ///
    // /// [`Features::TEXTURE_FORMAT_16BIT_NORM`] must be enabled to use this texture format.
    // R16Snorm,
    /// Red channel only. 16 bit float per channel. Float in shader.
    R16Float,
    /// Red and green channels. 8 bit integer per channel. [0, 255] converted to/from float [0, 1] in shader.
    Rg8Unorm,
    // /// Red and green channels. 8 bit integer per channel. [&minus;127, 127] converted to/from float [&minus;1, 1] in shader.
    // Rg8Snorm,
    // /// Red and green channels. 8 bit integer per channel. Unsigned in shader.
    // Rg8Uint,
    // /// Red and green channels. 8 bit integer per channel. Signed in shader.
    // Rg8Sint,

    // // Normal 32 bit formats
    // /// Red channel only. 32 bit integer per channel. Unsigned in shader.
    // R32Uint,
    // /// Red channel only. 32 bit integer per channel. Signed in shader.
    // R32Sint,
    // /// Red channel only. 32 bit float per channel. Float in shader.
    // R32Float,
    // /// Red and green channels. 16 bit integer per channel. Unsigned in shader.
    // Rg16Uint,
    // /// Red and green channels. 16 bit integer per channel. Signed in shader.
    // Rg16Sint,
    // /// Red and green channels. 16 bit integer per channel. [0, 65535] converted to/from float [0, 1] in shader.
    // ///
    // /// [`Features::TEXTURE_FORMAT_16BIT_NORM`] must be enabled to use this texture format.
    // Rg16Unorm,
    // /// Red and green channels. 16 bit integer per channel. [&minus;32767, 32767] converted to/from float [&minus;1, 1] in shader.
    // ///
    // /// [`Features::TEXTURE_FORMAT_16BIT_NORM`] must be enabled to use this texture format.
    // Rg16Snorm,
    /// Red and green channels. 16 bit float per channel. Float in shader.
    Rg16Float,
    /// Red, green, blue, and alpha channels. 8 bit integer per channel. [0, 255] converted to/from float [0, 1] in shader.
    Rgba8Unorm,
    /// Red, green, blue, and alpha channels. 8 bit integer per channel. Srgb-color [0, 255] converted to/from linear-color float [0, 1] in shader.
    Rgba8UnormSrgb,
    // /// Red, green, blue, and alpha channels. 8 bit integer per channel. [&minus;127, 127] converted to/from float [&minus;1, 1] in shader.
    // Rgba8Snorm,
    // /// Red, green, blue, and alpha channels. 8 bit integer per channel. Unsigned in shader.
    // Rgba8Uint,
    // /// Red, green, blue, and alpha channels. 8 bit integer per channel. Signed in shader.
    // Rgba8Sint,
    // /// Blue, green, red, and alpha channels. 8 bit integer per channel. [0, 255] converted to/from float [0, 1] in shader.
    // Bgra8Unorm,
    // /// Blue, green, red, and alpha channels. 8 bit integer per channel. Srgb-color [0, 255] converted to/from linear-color float [0, 1] in shader.
    // Bgra8UnormSrgb,

    // Packed 32 bit formats
    /// Packed unsigned float with 9 bits mantisa for each RGB component, then a common 5 bits exponent
    Rgb9e5Ufloat,
    // /// Red, green, blue, and alpha channels. 10 bit integer for RGB channels, 2 bit integer for alpha channel. Unsigned in shader.
    // Rgb10a2Uint,
    // /// Red, green, blue, and alpha channels. 10 bit integer for RGB channels, 2 bit integer for alpha channel. [0, 1023] ([0, 3] for alpha) converted to/from float [0, 1] in shader.
    // Rgb10a2Unorm,
    // /// Red, green, and blue channels. 11 bit float with no sign bit for RG channels. 10 bit float with no sign bit for blue channel. Float in shader.
    // Rg11b10Ufloat,

    // // Normal 64 bit formats
    // /// Red channel only. 64 bit integer per channel. Unsigned in shader.
    // ///
    // /// [`Features::TEXTURE_INT64_ATOMIC`] must be enabled to use this texture format.
    // R64Uint,
    // /// Red and green channels. 32 bit integer per channel. Unsigned in shader.
    // Rg32Uint,
    // /// Red and green channels. 32 bit integer per channel. Signed in shader.
    // Rg32Sint,
    // /// Red and green channels. 32 bit float per channel. Float in shader.
    // Rg32Float,
    // /// Red, green, blue, and alpha channels. 16 bit integer per channel. Unsigned in shader.
    // Rgba16Uint,
    // /// Red, green, blue, and alpha channels. 16 bit integer per channel. Signed in shader.
    // Rgba16Sint,
    // /// Red, green, blue, and alpha channels. 16 bit integer per channel. [0, 65535] converted to/from float [0, 1] in shader.
    // ///
    // /// [`Features::TEXTURE_FORMAT_16BIT_NORM`] must be enabled to use this texture format.
    // Rgba16Unorm,
    // /// Red, green, blue, and alpha. 16 bit integer per channel. [&minus;32767, 32767] converted to/from float [&minus;1, 1] in shader.
    // ///
    // /// [`Features::TEXTURE_FORMAT_16BIT_NORM`] must be enabled to use this texture format.
    // Rgba16Snorm,
    /// Red, green, blue, and alpha channels. 16 bit float per channel. Float in shader.
    Rgba16Float,

    // // Normal 128 bit formats
    // /// Red, green, blue, and alpha channels. 32 bit integer per channel. Unsigned in shader.
    // Rgba32Uint,
    // /// Red, green, blue, and alpha channels. 32 bit integer per channel. Signed in shader.
    // Rgba32Sint,
    // /// Red, green, blue, and alpha channels. 32 bit float per channel. Float in shader.
    // Rgba32Float,

    // // Depth and stencil formats
    // /// Stencil format with 8 bit integer stencil.
    // Stencil8,
    // /// Special depth format with 16 bit integer depth.
    // Depth16Unorm,
    // /// Special depth format with at least 24 bit integer depth.
    // Depth24Plus,
    // /// Special depth/stencil format with at least 24 bit integer depth and 8 bits integer stencil.
    // Depth24PlusStencil8,
    // /// Special depth format with 32 bit floating point depth.
    // Depth32Float,
    // /// Special depth/stencil format with 32 bit floating point depth and 8 bits integer stencil.
    // ///
    // /// [`Features::DEPTH32FLOAT_STENCIL8`] must be enabled to use this texture format.
    // Depth32FloatStencil8,

    // /// YUV 4:2:0 chroma subsampled format.
    // ///
    // /// Contains two planes:
    // /// - 0: Single 8 bit channel luminance.
    // /// - 1: Dual 8 bit channel chrominance at half width and half height.
    // ///
    // /// Valid view formats for luminance are [`TextureFormat::R8Unorm`].
    // ///
    // /// Valid view formats for chrominance are [`TextureFormat::Rg8Unorm`].
    // ///
    // /// Width and height must be even.
    // ///
    // /// [`Features::TEXTURE_FORMAT_NV12`] must be enabled to use this texture format.
    // NV12,

    // /// YUV 4:2:0 chroma subsampled format.
    // ///
    // /// Contains two planes:
    // /// - 0: Single 16 bit channel luminance, of which only the high 10 bits
    // ///   are used.
    // /// - 1: Dual 16 bit channel chrominance at half width and half height, of
    // ///   which only the high 10 bits are used.
    // ///
    // /// Valid view formats for luminance are [`TextureFormat::R16Unorm`].
    // ///
    // /// Valid view formats for chrominance are [`TextureFormat::Rg16Unorm`].
    // ///
    // /// Width and height must be even.
    // ///
    // /// [`Features::TEXTURE_FORMAT_P010`] must be enabled to use this texture format.
    // P010,

    // Compressed textures usable with `TEXTURE_COMPRESSION_BC` feature. `TEXTURE_COMPRESSION_SLICED_3D` is required to use with 3D textures.
    /// 4x4 block compressed texture. 8 bytes per block (4 bit/px). 4 color + alpha pallet. 5 bit R + 6 bit G + 5 bit B + 1 bit alpha.
    /// [0, 63] ([0, 1] for alpha) converted to/from float [0, 1] in shader.
    ///
    /// Also known as DXT1.
    ///
    /// [`Features::TEXTURE_COMPRESSION_BC`] must be enabled to use this texture format.
    /// [`Features::TEXTURE_COMPRESSION_BC_SLICED_3D`] must be enabled to use this texture format with 3D dimension.
    Bc1RgbaUnorm,
    /// 4x4 block compressed texture. 8 bytes per block (4 bit/px). 4 color + alpha pallet. 5 bit R + 6 bit G + 5 bit B + 1 bit alpha.
    /// Srgb-color [0, 63] ([0, 1] for alpha) converted to/from linear-color float [0, 1] in shader.
    ///
    /// Also known as DXT1.
    ///
    /// [`Features::TEXTURE_COMPRESSION_BC`] must be enabled to use this texture format.
    /// [`Features::TEXTURE_COMPRESSION_BC_SLICED_3D`] must be enabled to use this texture format with 3D dimension.
    Bc1RgbaUnormSrgb,
    /// 4x4 block compressed texture. 16 bytes per block (8 bit/px). 4 color pallet. 5 bit R + 6 bit G + 5 bit B + 4 bit alpha.
    /// [0, 63] ([0, 15] for alpha) converted to/from float [0, 1] in shader.
    ///
    /// Also known as DXT3.
    ///
    /// [`Features::TEXTURE_COMPRESSION_BC`] must be enabled to use this texture format.
    /// [`Features::TEXTURE_COMPRESSION_BC_SLICED_3D`] must be enabled to use this texture format with 3D dimension.
    Bc2RgbaUnorm,
    /// 4x4 block compressed texture. 16 bytes per block (8 bit/px). 4 color pallet. 5 bit R + 6 bit G + 5 bit B + 4 bit alpha.
    /// Srgb-color [0, 63] ([0, 255] for alpha) converted to/from linear-color float [0, 1] in shader.
    ///
    /// Also known as DXT3.
    ///
    /// [`Features::TEXTURE_COMPRESSION_BC`] must be enabled to use this texture format.
    /// [`Features::TEXTURE_COMPRESSION_BC_SLICED_3D`] must be enabled to use this texture format with 3D dimension.
    Bc2RgbaUnormSrgb,
    /// 4x4 block compressed texture. 16 bytes per block (8 bit/px). 4 color pallet + 8 alpha pallet. 5 bit R + 6 bit G + 5 bit B + 8 bit alpha.
    /// [0, 63] ([0, 255] for alpha) converted to/from float [0, 1] in shader.
    ///
    /// Also known as DXT5.
    ///
    /// [`Features::TEXTURE_COMPRESSION_BC`] must be enabled to use this texture format.
    /// [`Features::TEXTURE_COMPRESSION_BC_SLICED_3D`] must be enabled to use this texture format with 3D dimension.
    Bc3RgbaUnorm,
    /// 4x4 block compressed texture. 16 bytes per block (8 bit/px). 4 color pallet + 8 alpha pallet. 5 bit R + 6 bit G + 5 bit B + 8 bit alpha.
    /// Srgb-color [0, 63] ([0, 255] for alpha) converted to/from linear-color float [0, 1] in shader.
    ///
    /// Also known as DXT5.
    ///
    /// [`Features::TEXTURE_COMPRESSION_BC`] must be enabled to use this texture format.
    /// [`Features::TEXTURE_COMPRESSION_BC_SLICED_3D`] must be enabled to use this texture format with 3D dimension.
    Bc3RgbaUnormSrgb,
    /// 4x4 block compressed texture. 8 bytes per block (4 bit/px). 8 color pallet. 8 bit R.
    /// [0, 255] converted to/from float [0, 1] in shader.
    ///
    /// Also known as RGTC1.
    ///
    /// [`Features::TEXTURE_COMPRESSION_BC`] must be enabled to use this texture format.
    /// [`Features::TEXTURE_COMPRESSION_BC_SLICED_3D`] must be enabled to use this texture format with 3D dimension.
    Bc4RUnorm,
    /// 4x4 block compressed texture. 8 bytes per block (4 bit/px). 8 color pallet. 8 bit R.
    /// [&minus;127, 127] converted to/from float [&minus;1, 1] in shader.
    ///
    /// Also known as RGTC1.
    ///
    /// [`Features::TEXTURE_COMPRESSION_BC`] must be enabled to use this texture format.
    /// [`Features::TEXTURE_COMPRESSION_BC_SLICED_3D`] must be enabled to use this texture format with 3D dimension.
    Bc4RSnorm,
    /// 4x4 block compressed texture. 16 bytes per block (8 bit/px). 8 color red pallet + 8 color green pallet. 8 bit RG.
    /// [0, 255] converted to/from float [0, 1] in shader.
    ///
    /// Also known as RGTC2.
    ///
    /// [`Features::TEXTURE_COMPRESSION_BC`] must be enabled to use this texture format.
    /// [`Features::TEXTURE_COMPRESSION_BC_SLICED_3D`] must be enabled to use this texture format with 3D dimension.
    Bc5RgUnorm,
    /// 4x4 block compressed texture. 16 bytes per block (8 bit/px). 8 color red pallet + 8 color green pallet. 8 bit RG.
    /// [&minus;127, 127] converted to/from float [&minus;1, 1] in shader.
    ///
    /// Also known as RGTC2.
    ///
    /// [`Features::TEXTURE_COMPRESSION_BC`] must be enabled to use this texture format.
    /// [`Features::TEXTURE_COMPRESSION_BC_SLICED_3D`] must be enabled to use this texture format with 3D dimension.
    Bc5RgSnorm,
    /// 4x4 block compressed texture. 16 bytes per block (8 bit/px). Variable sized pallet. 16 bit unsigned float RGB. Float in shader.
    ///
    /// Also known as BPTC (float).
    ///
    /// [`Features::TEXTURE_COMPRESSION_BC`] must be enabled to use this texture format.
    /// [`Features::TEXTURE_COMPRESSION_BC_SLICED_3D`] must be enabled to use this texture format with 3D dimension.
    Bc6hRgbUfloat,
    /// 4x4 block compressed texture. 16 bytes per block (8 bit/px). Variable sized pallet. 16 bit signed float RGB. Float in shader.
    ///
    /// Also known as BPTC (float).
    ///
    /// [`Features::TEXTURE_COMPRESSION_BC`] must be enabled to use this texture format.
    /// [`Features::TEXTURE_COMPRESSION_BC_SLICED_3D`] must be enabled to use this texture format with 3D dimension.
    Bc6hRgbFloat,
    /// 4x4 block compressed texture. 16 bytes per block (8 bit/px). Variable sized pallet. 8 bit integer RGBA.
    /// [0, 255] converted to/from float [0, 1] in shader.
    ///
    /// Also known as BPTC (unorm).
    ///
    /// [`Features::TEXTURE_COMPRESSION_BC`] must be enabled to use this texture format.
    /// [`Features::TEXTURE_COMPRESSION_BC_SLICED_3D`] must be enabled to use this texture format with 3D dimension.
    Bc7RgbaUnorm,
    /// 4x4 block compressed texture. 16 bytes per block (8 bit/px). Variable sized pallet. 8 bit integer RGBA.
    /// Srgb-color [0, 255] converted to/from linear-color float [0, 1] in shader.
    ///
    /// Also known as BPTC (unorm).
    ///
    /// [`Features::TEXTURE_COMPRESSION_BC`] must be enabled to use this texture format.
    /// [`Features::TEXTURE_COMPRESSION_BC_SLICED_3D`] must be enabled to use this texture format with 3D dimension.
    Bc7RgbaUnormSrgb,
    /// 4x4 block compressed texture. 8 bytes per block (4 bit/px). Complex pallet. 8 bit integer RGB.
    /// [0, 255] converted to/from float [0, 1] in shader.
    ///
    /// [`Features::TEXTURE_COMPRESSION_ETC2`] must be enabled to use this texture format.
    Etc2Rgb8Unorm,
    /// 4x4 block compressed texture. 8 bytes per block (4 bit/px). Complex pallet. 8 bit integer RGB.
    /// Srgb-color [0, 255] converted to/from linear-color float [0, 1] in shader.
    ///
    /// [`Features::TEXTURE_COMPRESSION_ETC2`] must be enabled to use this texture format.
    Etc2Rgb8UnormSrgb,
    /// 4x4 block compressed texture. 8 bytes per block (4 bit/px). Complex pallet. 8 bit integer RGB + 1 bit alpha.
    /// [0, 255] ([0, 1] for alpha) converted to/from float [0, 1] in shader.
    ///
    /// [`Features::TEXTURE_COMPRESSION_ETC2`] must be enabled to use this texture format.
    Etc2Rgb8A1Unorm,
    /// 4x4 block compressed texture. 8 bytes per block (4 bit/px). Complex pallet. 8 bit integer RGB + 1 bit alpha.
    /// Srgb-color [0, 255] ([0, 1] for alpha) converted to/from linear-color float [0, 1] in shader.
    ///
    /// [`Features::TEXTURE_COMPRESSION_ETC2`] must be enabled to use this texture format.
    Etc2Rgb8A1UnormSrgb,
    /// 4x4 block compressed texture. 16 bytes per block (8 bit/px). Complex pallet. 8 bit integer RGB + 8 bit alpha.
    /// [0, 255] converted to/from float [0, 1] in shader.
    ///
    /// [`Features::TEXTURE_COMPRESSION_ETC2`] must be enabled to use this texture format.
    Etc2Rgba8Unorm,
    /// 4x4 block compressed texture. 16 bytes per block (8 bit/px). Complex pallet. 8 bit integer RGB + 8 bit alpha.
    /// Srgb-color [0, 255] converted to/from linear-color float [0, 1] in shader.
    ///
    /// [`Features::TEXTURE_COMPRESSION_ETC2`] must be enabled to use this texture format.
    Etc2Rgba8UnormSrgb,
    /// 4x4 block compressed texture. 8 bytes per block (4 bit/px). Complex pallet. 11 bit integer R.
    /// [0, 255] converted to/from float [0, 1] in shader.
    ///
    /// [`Features::TEXTURE_COMPRESSION_ETC2`] must be enabled to use this texture format.
    EacR11Unorm,
    /// 4x4 block compressed texture. 8 bytes per block (4 bit/px). Complex pallet. 11 bit integer R.
    /// [&minus;127, 127] converted to/from float [&minus;1, 1] in shader.
    ///
    /// [`Features::TEXTURE_COMPRESSION_ETC2`] must be enabled to use this texture format.
    EacR11Snorm,
    /// 4x4 block compressed texture. 16 bytes per block (8 bit/px). Complex pallet. 11 bit integer R + 11 bit integer G.
    /// [0, 255] converted to/from float [0, 1] in shader.
    ///
    /// [`Features::TEXTURE_COMPRESSION_ETC2`] must be enabled to use this texture format.
    EacRg11Unorm,
    /// 4x4 block compressed texture. 16 bytes per block (8 bit/px). Complex pallet. 11 bit integer R + 11 bit integer G.
    /// [&minus;127, 127] converted to/from float [&minus;1, 1] in shader.
    ///
    /// [`Features::TEXTURE_COMPRESSION_ETC2`] must be enabled to use this texture format.
    EacRg11Snorm,
    /// block compressed texture. 16 bytes per block.
    ///
    /// Features [`TEXTURE_COMPRESSION_ASTC`] or [`TEXTURE_COMPRESSION_ASTC_HDR`]
    /// must be enabled to use this texture format.
    ///
    /// [`TEXTURE_COMPRESSION_ASTC`]: Features::TEXTURE_COMPRESSION_ASTC
    /// [`TEXTURE_COMPRESSION_ASTC_HDR`]: Features::TEXTURE_COMPRESSION_ASTC_HDR
    Astc {
        /// compressed block dimensions
        block: AstcBlock,
        /// ASTC RGBA channel
        channel: AstcChannel,
    },
}

impl TextureFormat {
    /// Strips the `Srgb` suffix from the given texture format.
    #[must_use]
    pub fn remove_srgb_suffix(&self) -> TextureFormat {
        match *self {
            Self::Rgba8UnormSrgb => Self::Rgba8Unorm,
            // Self::Bgra8UnormSrgb => Self::Bgra8Unorm,
            Self::Bc1RgbaUnormSrgb => Self::Bc1RgbaUnorm,
            Self::Bc2RgbaUnormSrgb => Self::Bc2RgbaUnorm,
            Self::Bc3RgbaUnormSrgb => Self::Bc3RgbaUnorm,
            Self::Bc7RgbaUnormSrgb => Self::Bc7RgbaUnorm,
            Self::Etc2Rgb8UnormSrgb => Self::Etc2Rgb8Unorm,
            Self::Etc2Rgb8A1UnormSrgb => Self::Etc2Rgb8A1Unorm,
            Self::Etc2Rgba8UnormSrgb => Self::Etc2Rgba8Unorm,
            Self::Astc {
                block,
                channel: AstcChannel::UnormSrgb,
            } => Self::Astc {
                block,
                channel: AstcChannel::Unorm,
            },
            _ => *self,
        }
    }

    /// Adds an `Srgb` suffix to the given texture format, if the format supports it.
    #[must_use]
    pub fn add_srgb_suffix(&self) -> TextureFormat {
        match *self {
            Self::Rgba8Unorm => Self::Rgba8UnormSrgb,
            // Self::Bgra8Unorm => Self::Bgra8UnormSrgb,
            Self::Bc1RgbaUnorm => Self::Bc1RgbaUnormSrgb,
            Self::Bc2RgbaUnorm => Self::Bc2RgbaUnormSrgb,
            Self::Bc3RgbaUnorm => Self::Bc3RgbaUnormSrgb,
            Self::Bc7RgbaUnorm => Self::Bc7RgbaUnormSrgb,
            Self::Etc2Rgb8Unorm => Self::Etc2Rgb8UnormSrgb,
            Self::Etc2Rgb8A1Unorm => Self::Etc2Rgb8A1UnormSrgb,
            Self::Etc2Rgba8Unorm => Self::Etc2Rgba8UnormSrgb,
            Self::Astc {
                block,
                channel: AstcChannel::Unorm,
            } => Self::Astc {
                block,
                channel: AstcChannel::UnormSrgb,
            },
            _ => *self,
        }
    }
}
