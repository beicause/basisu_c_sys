use bevy::{
    core_pipeline::{Skybox, tonemapping::Tonemapping},
    log::LogPlugin,
    prelude::*,
};
use bevy_basisu_saver::BasisuSaverPlugin;

use crate::skybox_processor::SkyboxProcessor;
mod skybox_processor;

fn main() {
    App::new()
    .add_plugins(
    DefaultPlugins
            .set(LogPlugin {
                filter: "bevy_basisu_loader=debug,bevy_basisu_saver=debug,bevy_asset=debug,wgpu=warn".to_string(),
                ..Default::default()
            })
            .set(AssetPlugin {
                mode: AssetMode::Processed,
                ..Default::default()
            }),
    )
    .add_plugins(BasisuSaverPlugin::default())
    .register_asset_processor(SkyboxProcessor)
    .set_default_asset_processor::<SkyboxProcessor>("pack.ron")
    .add_systems(Startup, setup)
    .run();
}

const IMAGE_PATH_ALPHA0: &str = "alpha0.png";
const IMAGE_PATH_DESK: &str = "Desk_fixed_6x6.exr";
const IMAGE_PATH_KODIM20: &str = "kodim20.png";
const IMAGE_PATH_TOUGH: &str = "tough_fixed.png";
const IMAGE_PATH_WIKIPEDIA: &str = "wikipedia_fixed_6x6.png";

const IMAGES: &[&str] = &[
    IMAGE_PATH_ALPHA0,
    IMAGE_PATH_DESK,
    IMAGE_PATH_KODIM20,
    IMAGE_PATH_TOUGH,
    IMAGE_PATH_WIKIPEDIA,
];

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera3d::default(),
        Tonemapping::None,
        Skybox {
            image: Some(asset_server.load("skybox.pack.ron")),
            brightness: 1000.0,
            ..Default::default()
        },
    ));
    commands
        .spawn(Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            align_content: AlignContent::FlexStart,
            align_items: AlignItems::FlexStart,
            width: percent(100),
            height: percent(100),
            padding: UiRect::all(px(12)),
            column_gap: px(12),
            row_gap: px(12),
            ..Default::default()
        })
        .with_children(|func| {
            for &img in IMAGES {
                func.spawn((
                    Node {
                        height: px(256),
                        ..Default::default()
                    },
                    ImageNode::new(asset_server.load(img)),
                ));
            }
        });
}
