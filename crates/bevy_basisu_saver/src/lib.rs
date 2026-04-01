//! ⚠️ This is WIP.
//! A bevy asset processor to transform images to basisu ktx2 textures
//!
//! This is based on [basisu_c_sys](https://crates.io/crates/basisu_c_sys) and [bevy_basisu_loader](https://crates.io/crates/bevy_basisu_loader).

use bevy::{
    app::Plugin,
    asset::{processor::LoadTransformAndSave, transformer::IdentityAssetTransformer},
    image::{Image, ImageLoader},
};
use bevy_basisu_loader::BasisuLoaderPlugin;

use crate::saver::BasisuTextureSaver;

pub mod encoder;
pub mod saver;

pub type BasisuTextureProcessor =
    LoadTransformAndSave<ImageLoader, IdentityAssetTransformer<Image>, BasisuTextureSaver>;

pub struct BasisuTextureProcessorPlugin {
    pub extensions: Vec<String>,
}

impl Plugin for BasisuTextureProcessorPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        if !app.is_plugin_added::<BasisuLoaderPlugin>() {
            app.add_plugins(BasisuLoaderPlugin);
        }

        if let Some(asset_processor) = app
            .world()
            .get_resource::<bevy::asset::processor::AssetProcessor>()
        {
            asset_processor.register_processor::<BasisuTextureProcessor>(BasisuTextureSaver.into());
            for ext in &self.extensions {
                asset_processor.set_default_processor::<BasisuTextureProcessor>(ext.as_str());
            }
        }
    }
}
