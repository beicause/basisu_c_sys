use bevy::{
    core_pipeline::{Skybox, tonemapping::Tonemapping},
    log::LogPlugin,
    math::Affine2,
    prelude::*,
    render::{render_resource::TextureFormat, view::Hdr},
};
use bevy_basisu_loader::{BasisuLoaderPlugin, BasisuLoaderSettings};

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
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    filter: "bevy_basisu_loader=debug".to_string(),
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
    let skybox_handle = asset_server.load("gl_skybox_etc1s_cubemap_mips_12.basisu_ktx2");
    // camera
    commands.spawn((
        Camera3d::default(),
        Hdr,
        Tonemapping::None,
        Msaa::Off,
        Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        Skybox {
            image: skybox_handle.clone(),
            brightness: 1000.0,
            ..default()
        },
    ));

    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(1.0, 1.0).mesh().build())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("tough_uastc_ldr_4x4_mips_11.basisu_ktx2")),
            unlit: true,
            ..Default::default()
        })),
        Transform::from_xyz(-1.0, 0.0, -2.5),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(0.644 * 3.0, 0.874 * 3.0).mesh().build())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("desk_uastc_hdr_6x6_mips_10.basisu_ktx2")),
            unlit: true,
            ..Default::default()
        })),
        Transform::from_xyz(1.0, 0.0, -5.0),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(0.644 * 3.0, 0.874 * 3.0).mesh().build())),
        MeshMaterial3d(materials.add(StandardMaterial {
            uv_transform: Affine2::from_scale(Vec2::new(2., 2.)),
            base_color_texture: Some(asset_server.load_with_settings(
                "desk_uastc_hdr_4x4_mips_10.basisu_ktx2",
                |s: &mut BasisuLoaderSettings| {
                    s.force_transcode_target = Some(TextureFormat::Rgb9e5Ufloat);
                    s.sampler =
                        bevy::image::ImageSampler::Descriptor(bevy::image::ImageSamplerDescriptor {
                            address_mode_u: bevy::image::ImageAddressMode::Repeat,
                            address_mode_v: bevy::image::ImageAddressMode::Repeat,
                            ..Default::default()
                        })
                },
            )),
            unlit: true,
            ..Default::default()
        })),
        Transform::from_xyz(3.0, 0.0, -3.0)
            * Transform::from_rotation(Quat::from_rotation_y(-45.0)),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(1.0, 1.0).mesh().build())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load_with_settings(
                "alpha0_etc1s.basisu_ktx2",
                |s: &mut BasisuLoaderSettings| {
                    s.channel_type_hint = bevy_basisu_loader::ChannelType::Rg;
                },
            )),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..Default::default()
        })),
        Transform::from_xyz(-2.0, 0.0, -2.0)
            * Transform::from_rotation(Quat::from_rotation_y(45.0)),
    ));

    // UI
    commands.spawn((
        Text::new("Press Q, E (or ArrowLeft, ArrowRight) to rotate camera."),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
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
