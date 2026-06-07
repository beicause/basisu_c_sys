use bevy::{
    asset::AssetMetaCheck,
    core_pipeline::{Skybox, tonemapping::Tonemapping},
    log::LogPlugin,
    math::Affine2,
    prelude::*,
};
use bevy_basisu_loader::{BasisuLoaderPlugin, BasisuLoaderSettings};

const IMAGE_PATH_ALPHA0: &str = "alpha0_etc1s_mips.basisu.ktx2";
const IMAGE_PATH_DESK1: &str = "desk_uastc_hdr_4x4.basisu.ktx2";
const IMAGE_PATH_DESK2: &str = "desk_uastc_hdr_6x6_mips.basisu.ktx2";
const IMAGE_PATH_KODIM20: &str = "kodim20_astc_ldr_8x8_mips.basisu.ktx2";
const IMAGE_PATH_SKYBOX: &str = "skybox_xuastc_ldr_8x8_cubemap_mips.basisu.ktx2";
const IMAGE_PATH_TOUGH: &str = "tough_uastc_ldr_4x4.basisu.ktx2";
const IMAGE_PATH_WIKIPEDIA: &str = "wikipedia_xuastc_ldr_8x8_mips.basisu.ktx2";

#[bevy_main]
pub fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        // Bind to canvas included in `index.html`
                        canvas: Some("#bevy".to_owned()),
                        fit_canvas_to_parent: true,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..Default::default()
                })
                .set(LogPlugin {
                    filter: "bevy_basisu_loader=debug,wgpu=warn".to_string(),
                    ..Default::default()
                }),
        )
        .add_plugins(BasisuLoaderPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_camera)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let skybox_handle = asset_server.load(IMAGE_PATH_SKYBOX);
    // camera
    commands.spawn((
        Camera3d::default(),
        Tonemapping::None,
        Transform::from_xyz(0.0, 0.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
        Skybox {
            image: Some(skybox_handle.clone()),
            brightness: 1000.0,
            ..Default::default()
        },
    ));

    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(1.0, 1.0).mesh().build())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load(IMAGE_PATH_TOUGH)),
            unlit: true,
            ..Default::default()
        })),
        Transform::from_xyz(-1.0, 1.0, -2.5),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(0.644 * 3.0, 0.874 * 3.0).mesh().build())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load(IMAGE_PATH_DESK1)),
            unlit: true,
            ..Default::default()
        })),
        Transform::from_xyz(1.0, 1.0, -5.0),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(0.644 * 3.0, 0.874 * 3.0).mesh().build())),
        MeshMaterial3d(
            materials.add(StandardMaterial {
                uv_transform: Affine2::from_scale(Vec2::new(2., 2.)),
                base_color_texture: Some(
                    asset_server
                        .load_builder()
                        .with_settings(|s: &mut BasisuLoaderSettings| {
                            s.force_transcode_target =
                                Some(bevy_basisu_loader::sys::TranscodeTargetFormat::Rgb9e5);
                            s.sampler = bevy::image::ImageSampler::Descriptor(
                                bevy::image::ImageSamplerDescriptor {
                                    address_mode_u: bevy::image::ImageAddressMode::Repeat,
                                    address_mode_v: bevy::image::ImageAddressMode::Repeat,
                                    ..Default::default()
                                },
                            )
                        })
                        .load(IMAGE_PATH_DESK2),
                ),
                unlit: true,
                ..Default::default()
            }),
        ),
        Transform::from_xyz(3.0, 1.0, -5.0).with_rotation(Quat::from_rotation_y(-0.5)),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(1.0, 1.0).mesh().build())),
        MeshMaterial3d(
            materials.add(StandardMaterial {
                base_color_texture: Some(
                    asset_server
                        .load_builder()
                        .with_settings(|s: &mut BasisuLoaderSettings| {
                            s.channel_type_hint = bevy_basisu_loader::sys::extra::ChannelType::Rg;
                        })
                        .load(IMAGE_PATH_ALPHA0),
                ),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..Default::default()
            }),
        ),
        Transform::from_xyz(-2.0, 1.0, -2.0).with_rotation(Quat::from_rotation_y(0.5)),
    ));

    commands.spawn((
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            column_gap: px(8),
            position_type: PositionType::Absolute,
            bottom: px(8),
            right: px(8),
            left: px(8),
            width: percent(100),
            height: percent(25),
            ..Default::default()
        },
        children![
            ImageNode::new(asset_server.load(IMAGE_PATH_WIKIPEDIA)),
            ImageNode::new(asset_server.load(IMAGE_PATH_KODIM20)),
        ],
    ));

    // UI
    commands.spawn((
        Text::new("Press Q, E (or ArrowLeft, ArrowRight) to rotate camera."),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..Default::default()
        },
    ));
}

fn rotate_camera(
    mut query: Query<&mut Transform, With<Camera3d>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let rotate = if keyboard_input.pressed(KeyCode::KeyQ)
        || keyboard_input.pressed(KeyCode::ArrowLeft)
    {
        0.05
    } else if keyboard_input.pressed(KeyCode::KeyE) || keyboard_input.pressed(KeyCode::ArrowRight) {
        -0.05
    } else {
        0.0
    };
    for mut transform in &mut query {
        transform.rotate_y(rotate);
    }
}
