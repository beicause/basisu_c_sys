use bevy::prelude::*;
use bevy::render::{RenderApp, renderer::RenderDevice};

mod loader;

pub use loader::*;

/// Provides a loader for Basis Universal KTX2 textures.
///
/// The file extension must be `.basisu.ktx2` to use this loader. All basis universal compressed formats (ETC1S, UASTC, XUASTC) are supported. Zstd supercompression is always supported. No support for `.basis` files.
///
/// Transcode Target Selection:
///
/// | BasisU format                  | target selection                                               |
/// | ------------------------------ | -------------------------------------------------------------- |
/// | ETC1S                          | Bc7Rgba/Bc5Rg/Bc4R > Etc2Rgba8/Etc2Rgb8/EacRg11/EacR11 > Rgba8 |
/// | UASTC_LDR, ASTC_LDR, XUASTC_LDR| Astc > Bc7Rgba > Etc2Rgba8/Etc2Rgb8/EacRg11/EacR11 > Rgba8     |
/// | UASTC_HDR, ASTC_HDR            | Astc > Bc6hRgbUfloat > Rgba16Float                             |
///
pub struct BasisuLoaderPlugin;

impl Plugin for BasisuLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.preregister_asset_loader::<BasisuLoader>(&["basisu.ktx2"])
            .add_systems(PreStartup, || {
                #[cfg(all(
                    target_arch = "wasm32",
                    target_vendor = "unknown",
                    target_os = "unknown",
                ))]
                bevy::tasks::IoTaskPool::get()
                    .spawn_local(async {
                        bevy_basisu_loader_sys::basisu_sys_init_vendor().await;
                        unsafe { bevy_basisu_loader_sys::basisu_transcoder_init() };
                    })
                    .detach();
                #[cfg(not(all(
                    target_arch = "wasm32",
                    target_vendor = "unknown",
                    target_os = "unknown",
                )))]
                unsafe {
                    bevy_basisu_loader_sys::basisu_transcoder_init()
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
