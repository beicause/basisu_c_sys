use bevy::prelude::*;
use bevy::render::{RenderApp, renderer::RenderDevice};
mod transcoder;

#[cfg(all(
    target_arch = "wasm32",
    target_vendor = "unknown",
    target_os = "unknown",
))]
use bevy::platform::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

mod loader;

pub use loader::*;

/// Provides a loader for Basis Universal KTX2 textures.
///
/// The file extension must be `.basisu.ktx2` to use this loader. All basis universal compressed formats (ETC1S, UASTC, XUASTC) are supported. Zstd supercompression is always supported. No support for `.basis` files.
///
/// Default transcode target selection:
///
/// | BasisU format                  | Target selection                                               |
/// | ------------------------------ | -------------------------------------------------------------- |
/// | ETC1S                          | Bc7Rgba/Bc5Rg/Bc4R > Etc2Rgba8/Etc2Rgb8/EacRg11/EacR11 > Rgba8 |
/// | UASTC_LDR, ASTC_LDR, XUASTC_LDR| Astc > Bc7Rgba > Etc2Rgba8/Etc2Rgb8/EacRg11/EacR11 > Rgba8     |
/// | UASTC_HDR, ASTC_HDR            | Astc > Bc6hRgbUfloat > Rgba16Float                             |
///
pub struct BasisuLoaderPlugin;

#[cfg(all(
    target_arch = "wasm32",
    target_vendor = "unknown",
    target_os = "unknown",
))]
#[derive(Resource, Clone, Deref)]
struct BasisuWasmReady(Arc<AtomicUsize>);

impl Plugin for BasisuLoaderPlugin {
    fn build(&self, app: &mut App) {
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
                    crate::transcoder::basisu_transcoder_init().await;
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
        bevy::tasks::block_on(crate::transcoder::basisu_transcoder_init());
        app.preregister_asset_loader::<BasisuLoader>(&["basisu.ktx2"]);
    }

    #[cfg(all(
        target_arch = "wasm32",
        target_vendor = "unknown",
        target_os = "unknown",
    ))]
    fn ready(&self, app: &App) -> bool {
        app.world()
            .resource::<BasisuWasmReady>()
            .load(Ordering::Acquire)
            != 0
    }

    fn finish(&self, app: &mut App) {
        #[cfg(all(
            target_arch = "wasm32",
            target_vendor = "unknown",
            target_os = "unknown",
        ))]
        app.world_mut().remove_resource::<BasisuWasmReady>();
        let device = app
            .sub_app_mut(RenderApp)
            .world()
            .resource::<RenderDevice>();
        let features = device.features();
        app.register_asset_loader(BasisuLoader::from_features(features));
    }
}
