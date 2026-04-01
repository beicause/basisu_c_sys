//! A bevy asset processor to transform images to basisu ktx2 textures
//!
//! This is based on [basisu_c_sys](https://crates.io/crates/basisu_c_sys) and [bevy_basisu_loader](https://crates.io/crates/bevy_basisu_loader).

use bevy::{
    app::Plugin,
    asset::{processor::LoadTransformAndSave, transformer::IdentityAssetTransformer},
    image::{Image, ImageLoader},
};
use bevy_basisu_loader::BasisuLoaderPlugin;

use crate::{encoder::basisu_init, saver::BasisuTextureSaver};

pub mod encoder;
pub mod saver;

pub type BasisuProcessor =
    LoadTransformAndSave<ImageLoader, IdentityAssetTransformer<Image>, BasisuTextureSaver>;

pub struct BasisuSaverPlugin {
    /// The file extensions handled by the processor.
    pub file_extensions: Vec<String>,
}

impl Default for BasisuSaverPlugin {
    fn default() -> Self {
        Self {
            file_extensions: ImageLoader::SUPPORTED_FILE_EXTENSIONS
                .iter()
                .filter(|s| !["basis", "ktx2", "dds"].contains(s))
                .map(|s| s.to_string())
                .collect(),
        }
    }
}

impl Plugin for BasisuSaverPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        basisu_init();

        if !app.is_plugin_added::<BasisuLoaderPlugin>() {
            app.add_plugins(BasisuLoaderPlugin);
        }

        if let Some(asset_processor) = app
            .world()
            .get_resource::<bevy::asset::processor::AssetProcessor>()
        {
            asset_processor.register_processor::<BasisuProcessor>(BasisuTextureSaver.into());
            for ext in &self.file_extensions {
                asset_processor.set_default_processor::<BasisuProcessor>(ext.as_str());
            }
        }
    }
}
