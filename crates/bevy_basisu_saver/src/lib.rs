//! Asset saver and processor for Basis Universal KTX2 textures.

extern crate alloc;

use crate::saver::{BasisuProcessor, BasisuSaver};
use alloc::borrow::Cow;
use basisu_c_sys::extra::BasisuEncoderParams;
use bevy::{
    app::{App, Plugin},
    image::ImageLoader,
};

use bevy_basisu_loader::BasisuLoaderPlugin;

pub mod saver;
pub use basisu_c_sys as sys;

/// Provides basis universal asset saver and processor.
pub struct BasisuSaverPlugin {
    /// The file extensions handled by the basisu asset processor.
    ///
    /// Default is [`ImageLoader::SUPPORTED_FILE_EXTENSIONS`] except ktx2 and .dds.
    pub processor_extensions: Vec<Cow<'static, str>>,
    /// Default basisu encoder params.
    /// See the documents and `BU_COMP_FLAGS_*` in [`basisu_c_sys`] if you want more controls,
    /// like mipmap generation.
    pub default_encoder_params: BasisuEncoderParams,
}

impl Default for BasisuSaverPlugin {
    fn default() -> Self {
        Self {
            processor_extensions: ImageLoader::SUPPORTED_FILE_EXTENSIONS
                .iter()
                .filter(|s| !["ktx2", "dds"].contains(s))
                .map(|s| Cow::Borrowed(*s))
                .collect(),
            default_encoder_params: BasisuEncoderParams::new_with_srgb_defaults(
                basisu_c_sys::BasisTextureFormat::XuastcLdr4x4,
            ),
        }
    }
}

impl Plugin for BasisuSaverPlugin {
    fn build(&self, app: &mut App) {
        basisu_c_sys::extra::basisu_encoder_init();

        if !app.is_plugin_added::<BasisuLoaderPlugin>() {
            app.add_plugins(BasisuLoaderPlugin);
        }

        if let Some(asset_processor) = app
            .world()
            .get_resource::<bevy::asset::processor::AssetProcessor>()
        {
            asset_processor.register_processor::<BasisuProcessor>(
                BasisuSaver {
                    default_encoder_params: self.default_encoder_params,
                }
                .into(),
            );
            for ext in &self.processor_extensions {
                asset_processor.set_default_processor::<BasisuProcessor>(ext);
            }
        }
    }
}
