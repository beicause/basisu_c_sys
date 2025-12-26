use bevy::prelude::*;
use bevy::render::{RenderApp, renderer::RenderDevice};

#[expect(
    dead_code,
    non_upper_case_globals,
    reason = "Generated rust bindings are OK to be unused or non upper case"
)]
mod transcoding_wrapper {
    include!(concat!(env!("OUT_DIR"), "/transcoding_wrapper.rs"));
}
mod loader;

pub use loader::*;

pub struct BasisuLoaderPlugin;

impl Plugin for BasisuLoaderPlugin {
    fn build(&self, app: &mut App) {
        // SAFETY: Initialization of the basis universal transcoder should always succeed.
        unsafe {
            transcoding_wrapper::basisu_transcoder_init();
        }
        app.init_asset::<BasisuTranscodedImage>()
            .preregister_asset_loader::<BasisuLoader>(&["ktx2"]);
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
