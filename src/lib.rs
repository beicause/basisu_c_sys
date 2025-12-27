use bevy::prelude::*;
use bevy::render::{RenderApp, renderer::RenderDevice};

mod loader;

pub use loader::*;

pub struct BasisuLoaderPlugin;

impl Plugin for BasisuLoaderPlugin {
    fn build(&self, app: &mut App) {
        // SAFETY: Initialization of the basis universal transcoder should always succeed.
        unsafe {
            basisu_sys::transcoding::basisu_transcoder_init();
        }
        app.preregister_asset_loader::<BasisuLoader>(&["ktx2"]);
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
