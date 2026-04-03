//! A bevy asset processor to transform images to basisu ktx2 textures
//!
//! This is based on [basisu_c_sys](https://crates.io/crates/basisu_c_sys) and [bevy_basisu_loader](https://crates.io/crates/bevy_basisu_loader).

use crate::{encoder::basisu_encoder_init, saver::BasisuTextureSaver};
#[cfg(all(
    target_arch = "wasm32",
    target_vendor = "unknown",
    target_os = "unknown",
))]
use bevy::platform::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use bevy::{
    app::Plugin,
    asset::{processor::LoadTransformAndSave, transformer::IdentityAssetTransformer},
    image::{Image, ImageLoader},
    prelude::*,
};
use bevy_basisu_loader::BasisuLoaderPlugin;

pub mod encoder;
pub mod saver;
pub use basisu_c_sys as c_sys;

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

#[cfg(all(
    target_arch = "wasm32",
    target_vendor = "unknown",
    target_os = "unknown",
))]
#[derive(Resource, Clone, Deref)]
struct BasisuWasmReady(Arc<AtomicUsize>);

impl Plugin for BasisuSaverPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        #[cfg(all(
            target_arch = "wasm32",
            target_vendor = "unknown",
            target_os = "unknown",
        ))]
        {
            let ready = BasisuWasmReady(Arc::new(AtomicUsize::new(0)));
            let r = ready.clone();
            bevy::tasks::IoTaskPool::get()
                .spawn_local(async move {
                    basisu_encoder_init().await;
                    r.store(1, Ordering::Release);
                    bevy::log::debug!("Basisu wasm initialized")
                })
                .detach();
            app.insert_resource(ready);
        }
        #[cfg(not(all(
            target_arch = "wasm32",
            target_vendor = "unknown",
            target_os = "unknown",
        )))]
        bevy::tasks::block_on(basisu_encoder_init());

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
