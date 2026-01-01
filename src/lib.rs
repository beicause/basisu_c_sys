use bevy::prelude::*;
use bevy::render::{RenderApp, renderer::RenderDevice};

mod loader;

pub use loader::*;

pub struct BasisuLoaderPlugin;

impl Plugin for BasisuLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.preregister_asset_loader::<BasisuLoader>(&["basisu_ktx2"])
            .add_systems(PreStartup, || {
                #[cfg(all(
                    target_arch = "wasm32",
                    target_vendor = "unknown",
                    target_os = "unknown",
                ))]
                bevy::tasks::IoTaskPool::get()
                    .spawn_local(async {
                        basisu_sys::basisu_sys_init_vendor().await;
                        unsafe { basisu_sys::basisu_transcoder_init() };
                    })
                    .detach();
                #[cfg(not(all(
                    target_arch = "wasm32",
                    target_vendor = "unknown",
                    target_os = "unknown",
                )))]
                unsafe {
                    basisu_sys::basisu_transcoder_init()
                };
            });
    }

    fn finish(&self, app: &mut App) {
        let device = app
            .sub_app_mut(RenderApp)
            .world()
            .resource::<RenderDevice>();
        let features = device.features();
        app.register_asset_loader(BasisuLoader::from_features(features));
    }
}
